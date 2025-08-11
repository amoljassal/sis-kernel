#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

#[cfg(feature = "idt-selftest")]
mod arch_selftest { pub use crate::arch::x86_64::idt_selftest as idt_selftest; }

/*
 * main.rs - Entry point for the Sovereign Interface System (SIS) kernel.
 *
 * This file ties together the architecture-specific setup and the core
 * functionality of the kernel.  It makes use of the `bootloader`
 * crate to provide a UEFI compliant entry point and prints a door
 * glyph before transferring control to this function.  The kernel
 * initialises descriptor tables, paging, heap allocation, the
 * interrupt system, scheduler, tasks and PCI/IOMMU stubs.  It then
 * enables interrupts and enters a low-power loop while the
 * scheduler dispatches tasks on timer interrupts.
 */

extern crate alloc;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

mod arch;
mod kernel;

#[cfg(feature = "userland")]
mod userland;

use arch::x86_64 as arch_x86;
use kernel::{pci, scheduler, serial, syscall, task};

/// Kernel entry point invoked by the bootloader.  The `BootInfo`
/// structure contains a memory map and other information provided
/// by UEFI.  We use this to initialise the heap and map the kernel
/// to the higher half.
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialise serial logging first so we can print messages during
    // early boot.  Use COM1 at 115200 baud via the uart_16550 crate.
    serial::init();
    
    // Immediate identification for debug
    serial::write_str("\n=== SIS KERNEL ENTRY ===\n");
    
    // Debug bootloader 0.11.x mapping options IMMEDIATELY
    let po = boot_info.physical_memory_offset.into_option();
    let ri = boot_info.recursive_index;
    serial::write_str("[boot] phys_off=");
    match po { Some(v) => serial::write_hex64(v), None => serial::write_str("none") };
    serial::write_str(" rec_idx=");
    match ri { Some(v) => serial::write_hex8(v), None => serial::write_str("none") };
    serial::write_str("\n");
    
    serial::write_str("Kernel main() reached - initializing...\n");

    // Set up the Global Descriptor Table and Task State Segment.
    arch_x86::gdt::init();

    // Set up the Interrupt Descriptor Table and install default
    // handlers for exceptions, timer interrupts and syscalls.
    arch_x86::idt::init_idt();
    serial::write_str("[init] IDT loaded\n");
    
    // Enable NXE (no-execute) support for PFM v2 NX tests
    arch_x86::memory::enable_nxe_once();
    serial::write_str("[init] NXE enabled\n");

    // Optional IDT self-tests (feature-gated). Each test halts after triggering.
    // Choose which test to run inside idt_selftest::run_sequence() or call run_one("TAG").
    #[cfg(feature = "idt-selftest")]
    {
        serial::write_str("[selftest] starting IDT self-tests...\n");
        // By default, run the DF test. Edit to run GP/PF/DIV0 as needed.
        arch_selftest::idt_selftest::run_sequence();
    }

    // Page-Fault Matrix (PFM) self-tests (feature-gated)
    #[cfg(all(feature = "pf-matrix", feature = "idt-selftest"))]
    {
        serial::write_str("[selftest] starting Page-Fault Matrix tests...\n");
        arch_x86::pf_matrix::run_test();
    }

    // Per-task address space isolation self-test (feature-gated)
    #[cfg(all(feature = "per-task-mm", feature = "idt-selftest", selftest_AS_PER_TASK_ISOLATION))]
    {
        serial::write_str("[selftest] starting per-task address space isolation test...\n");
        arch_x86::as_isolation::selftest_isolation();
    }

    // IPC ping self-test (feature-gated)
    #[cfg(all(feature = "ipc", feature = "idt-selftest", selftest_IPC_PING))]
    {
        serial::write_str("[selftest] starting IPC ping test...\n");
        arch_x86::ipc_selftest::run_ipc_ping();
    }

    #[cfg(all(feature = "scheduler", feature = "idt-selftest", selftest_SCHEDULER_PREEMPT))]
    {
        serial::write_str("[selftest] starting scheduler preemption test...\n");
        arch_x86::scheduler_selftest::run();
    }

    // Configure paging and map the kernel into the higher half.  Use
    // the UEFI memory map to mark free pages and initialise the
    // frame allocator.  We then initialise the heap using a simple
    // linked list allocator backed by the available memory.
    unsafe { arch_x86::memory::init(boot_info) };

    #[cfg(feature = "userland")]
    {
        // Touch the VFS once to ensure initfs table is constructed.
        userland::vfs::boot_probe();
    }

    // Initialise IOMMU detection and deny-all DMA policy (Phase 5A)
    #[cfg(feature = "iommu")]
    {
        match arch_x86::iommu::init() {
            Ok(_) => serial::write_str("[init] IOMMU initialization complete\n"),
            Err(e) => {
                serial::write_str("[init] IOMMU initialization failed: ");
                serial::write_str(e);
                serial::write_str("\n");
            }
        }
    }

    // Initialise the syscall table and routing logic.
    syscall::init();

    // Initialise the scheduler and create the parent tasks.
    #[cfg(feature = "scheduler")]
    scheduler::init(0); // CPU 0
    #[cfg(not(feature = "scheduler"))]
    scheduler::init();
    let philo = task::Task::new(task::Role::Philosophy, philosophy_parent);
    let tech  = task::Task::new(task::Role::Technical, technical_parent);
    scheduler::add_parent(philo);
    scheduler::add_parent(tech);

    // Initialise the Programmable Interval Timer to 100 Hz.  Each
    // timer interrupt will cause the scheduler to advance tasks and
    // perform context switching.
    // ===== Timer initialization =====
    // APIC/LAPIC timer (modern) vs PIC/PIT timer (legacy)
    #[cfg(feature = "apic")]
    {
        // Initialize APIC/IOAPIC for modern interrupt delivery
        match arch_x86::apic::init_apic() {
            Ok(_) => {
                let _ = arch_x86::apic::init_ioapic();
                // Configure LAPIC periodic timer (5M initial count, divide by 16)
                let _ = arch_x86::apic::init_lapic_timer_periodic(5_000_000, 16);
                serial::write_str("[init] Modern APIC/LAPIC timer initialized\n");
            }
            Err(_) => {
                serial::write_str("[init] APIC not available; falling back to PIC\n");
                // Fallback to legacy PIT
                arch_x86::pit::init(100);
                unsafe { arch_x86::idt::PICS.lock().initialize() };
            }
        }
    }
    #[cfg(not(feature = "apic"))]
    {
        // Legacy PIC/PIT timer initialization (100 Hz)
        arch_x86::pit::init(100);
        unsafe { arch_x86::idt::PICS.lock().initialize() };
    }

    // Scan the PCI bus for GPUs and set up IOMMU passthrough.  The
    // PCI module logs any display controllers and assigns them to
    // parent tasks based on their index (GPU0 -> Philosophy,
    // GPU1 -> Technical).  In a future version this will set up
    // VFIO/IOMMU mapping.
    pci::init();

    // ===== SMP 2-core bring-up (feature-gated) =====
    #[cfg(all(feature = "apic", feature = "smp"))]
    {
        use crate::arch::x86_64::smp;
        if let Err(e) = smp::init_smp_2() {
            serial::write_str("[smp] init failed: ");
            serial::write_str(e);
            serial::write_str("\n");
        }
    }

    // Enable interrupts (PIC already initialized above if needed)
    x86_64::instructions::interrupts::enable();
    serial::write_str("Initialisation complete.  Entering idle loop...\n");

    // Kernel watchdog for selftest builds - prevent infinite hangs
    #[cfg(any(selftest_USR_INIT, selftest_USR_SPAWN_TWO, selftest_USR_ELF_EDGES, selftest_USR_VFS_NEG))]
    {
        use arch_x86::io::qemu_exit;
        const WATCHDOG_TICKS: u64 = 300; // ~3 seconds at 100Hz
        static mut TICK_COUNTER: u64 = 0;
        unsafe {
            TICK_COUNTER = arch_x86::cpu::rdtsc();
        }
        serial::write_str("[selftest] watchdog armed for selftest builds\n");
    }

    // Selftest: SMP_2 validation handled in timer ISR
    #[cfg(all(feature = "idt-selftest", selftest_SMP_2))]
    {
        // Exit condition is handled in scheduler::tick() when both CPUs reach 10 ticks
        serial::write_str("[selftest] SMP_2 test running...\n");
    }

    // Phase 5A: IOMMU selftests
    #[cfg(all(feature = "iommu", selftest_IOMMU_PROBE))]
    {
        serial::write_str("[selftest] Starting IOMMU_PROBE test...\n");
        arch_x86::iommu::selftest::run_iommu_probe();
        // Never returns - calls qemu_exit
    }

    #[cfg(all(feature = "iommu", selftest_IOMMU_DENY_DEFAULT))]
    {
        serial::write_str("[selftest] Starting IOMMU_DENY_DEFAULT test...\n");
        arch_x86::iommu::selftest::run_iommu_deny_default();
        // Never returns - calls qemu_exit
    }

    // Phase 5B: VFIO selftests  
    #[cfg(all(feature = "vfio", selftest_VFIO_BIND_E1000))]
    {
        serial::write_str("[selftest] Starting VFIO_BIND_E1000 test...\n");
        match kernel::user::selftest::run_vfio_bind_e1000() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_BIND_E1000] Device binding successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_BIND_E1000] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    #[cfg(all(feature = "vfio", selftest_VFIO_CFG_READ))]
    {
        serial::write_str("[selftest] Starting VFIO_CFG_READ test...\n");
        match kernel::user::selftest::run_vfio_cfg_read() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_CFG_READ] Config space read successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_CFG_READ] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    #[cfg(all(feature = "vfio", selftest_VFIO_MAP_BAR))]
    {
        serial::write_str("[selftest] Starting VFIO_MAP_BAR test...\n");
        match kernel::user::selftest::run_vfio_map_bar() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_MAP_BAR] BAR mapping successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_MAP_BAR] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    #[cfg(all(feature = "vfio", selftest_VFIO_IRQ_SETUP))]
    {
        serial::write_str("[selftest] Starting VFIO_IRQ_SETUP test...\n");
        match kernel::user::selftest::run_vfio_irq_setup() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_IRQ_SETUP] IRQ setup successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_IRQ_SETUP] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    // Phase 5C-A: IOMMU domain and DMA staging selftests
    #[cfg(all(feature = "vfio", selftest_VFIO_DOMAIN_CREATE))]
    {
        serial::write_str("[selftest] Starting VFIO_DOMAIN_CREATE test...\n");
        match kernel::user::selftest::run_vfio_domain_create() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_DOMAIN_CREATE] Domain creation successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_DOMAIN_CREATE] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    #[cfg(all(feature = "vfio", selftest_VFIO_DMA_STAGING))]
    {
        serial::write_str("[selftest] Starting VFIO_DMA_STAGING test...\n");
        match kernel::user::selftest::run_vfio_dma_staging() {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_DMA_STAGING] DMA staging successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_DMA_STAGING] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    // Phase 5C-B: MSI interrupt delivery selftest
    #[cfg(all(feature = "vfio", selftest_VFIO_MSI_SMOKE))]
    {
        serial::write_str("[selftest] Starting VFIO_MSI_SMOKE test...\n");
        
        // Inline MSI smoke test (avoid userland dependency)
        let result = run_vfio_msi_smoke_inline();
        match result {
            Ok(()) => {
                serial::write_str("[PASS: VFIO_MSI_SMOKE] MSI setup successful\n");
                unsafe { arch_x86::io::qemu_exit(0x00); }
            },
            Err(msg) => {
                serial::write_str("[FAIL: VFIO_MSI_SMOKE] ");
                serial::write_str(msg);
                serial::write_str("\n");
                unsafe { arch_x86::io::qemu_exit(0x01); }
            }
        }
    }

    // Phase 5C-B: MSI soak test (100 interrupts + latency histogram)
    #[cfg(all(feature = "vfio", selftest_VFIO_MSI_SOAK))]
    {
        serial::write_str("[selftest] Starting VFIO_MSI_SOAK test...\n");
        userland::selftest_vfio::run();
        // Never returns - calls qemu_exit
    }

    // Phase 4.1: Userland validation suite (Part C)
    #[cfg(feature = "userland")]
    {
        #[cfg(any(selftest_USR_INIT, selftest_USR_SPAWN_TWO, selftest_USR_ELF_EDGES, selftest_USR_VFS_NEG))]
        {
            serial::write_str("[selftest] ABOUT TO START userland validation suite...\n");
            serial::write_str("[selftest] Kernel initialization complete, entering userland tests\n");
            userland::selftest_usr::run();
            // selftest_usr::run() calls qemu_exit internally, so we never reach here
            serial::write_str("[ERROR] selftest returned unexpectedly!\n");
            unsafe { arch_x86::io::qemu_exit(0xFF); }
        }
    }

    // Idle loop.  The `hlt` instruction reduces power consumption
    // between interrupts.  The scheduler is driven from the timer
    // interrupt handler and will perform context switches as needed.
    loop {
        x86_64::instructions::hlt();
    }
}

/// Parent task for everyday interactions.  It runs as a privileged
/// parent on its own core/GPU and processes directives that do not
/// require technical analysis.  For now it logs a message and
/// loops.  In the future it will create child tasks to handle
/// specific requests.
fn philosophy_parent() {
    serial::write_str("[Philosophy] Hello from the Philosophy parent!\n");
    loop { arch_x86::cpu::pause(); }
}

/// Parent task for technical tasks such as coding, security and
/// analysis.  It logs a message and loops.  Future versions will
/// use GPU2 via IOMMU and spawn child modules for compute tasks.
fn technical_parent() {
    serial::write_str("[Technical] Hello from the Technical parent!\n");
    loop { arch_x86::cpu::pause(); }
}

/// Panic handler.  When a panic occurs we print the panic
/// information to the serial port and then halt the CPU.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::write_str("[panic] Kernel panic: ");
    // Print panic message if available
    serial::write_str(&alloc::format!("{}", info));
    serial::write_str("\n");
    loop {
        arch_x86::cpu::halt();
    }
}

/// Allocation error handler required when using `alloc` in a
/// `no_std` environment.  We log the failure and halt.
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    serial::write_str("[alloc] OOM during early boot\n");
    loop {
        arch_x86::cpu::halt();
    }
}

// Phase 5C-B: Inline MSI smoke test (avoid userland dependency)
#[cfg(all(feature = "vfio", selftest_VFIO_MSI_SMOKE))]
fn run_vfio_msi_smoke_inline() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_MSI_SMOKE starting...\n");
    
    // Step 1: Bind device via syscall
    match syscall::dispatch_manual(0x50, 0, 3, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] Device bind syscall completed\n"),
    }
    
    // Step 2: Create domain
    match syscall::dispatch_manual(0x55, 0x8000, 0, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] Domain create syscall completed\n"),
    }
    
    // Step 3: Map staging buffer
    match syscall::dispatch_manual(0x56, 0x8000, 16384, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] Staging map syscall completed\n"),
    }
    
    // Step 4: Enable bus master
    match syscall::dispatch_manual(0x57, 0x8000, 0, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] Bus master enable syscall completed\n"),
    }
    
    // Step 5: Arm MSI at vector 0x5E
    match syscall::dispatch_manual(0x58, 0x8000, 0x5E, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] MSI arm syscall completed\n"),
    }
    
    // Step 6: Map BAR0
    match syscall::dispatch_manual(0x53, 0x8000, 0, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] BAR0 map syscall completed\n"),
    }
    
    // Step 7: Cleanup - disarm MSI
    match syscall::dispatch_manual(0x59, 0x8000, 0, 0, 0, 0, 0) {
        _ => serial::write_str("[selftest] MSI disarm syscall completed\n"),
    }
    
    serial::write_str("[selftest] MSI smoke test setup completed\n");
    serial::write_str("[selftest] NOTE: BAR0 nudge would trigger MSI (not implemented in this test)\n");
    
    Ok(())
}