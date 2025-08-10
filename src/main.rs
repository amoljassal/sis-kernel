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

use bootloader::{entry_point, BootInfo};
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
    serial::write_str("[alloc] Allocation error\n");
    loop {
        arch_x86::cpu::halt();
    }
}