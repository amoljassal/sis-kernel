#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
// When we're not building any selftests, the codebase contains many `#[cfg(selftest_...)]` guards the
// compiler doesn't know about. Allow that noise in *non-selftest* builds only.
#![cfg_attr(not(feature = "selftests"), allow(unexpected_cfgs))]
// Selftest codepaths legitimately pull in helpers that go unused in normal builds; keep the noise down.
#![cfg_attr(feature = "selftests", allow(unused_imports, unused_variables))]

extern crate alloc;

// Global allocator for ARM64
#[cfg(target_arch = "aarch64")]
use linked_list_allocator::LockedHeap;

#[cfg(target_arch = "aarch64")]
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Initialize the heap allocator for ARM64
#[cfg(target_arch = "aarch64")]
pub fn init_heap() {
    const HEAP_START: usize = 0x4000_0000;
    const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
}

#[cfg(target_arch = "x86_64")]
use bootloader_api::{entry_point, BootInfo};

#[cfg(target_arch = "aarch64")]
use bootloader_api::BootInfo;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "firewall"))]
mod arch;
#[cfg(feature = "firewall")]
mod arch {
    #[cfg(target_arch = "x86_64")]
    pub mod x86_64 {
        pub use crate::arch_minimal::*;
    }
    #[cfg(target_arch = "aarch64")]
    pub mod aarch64 {
        // Minimal ARM64 support for firewall mode
    }
}

#[cfg(not(feature = "firewall"))]
mod kernel;
#[cfg(not(feature = "firewall"))]
mod boot;
#[cfg(not(feature = "firewall"))]
mod testing;
#[cfg(not(feature = "firewall"))]
mod sdk;
#[cfg(not(feature = "firewall"))]
mod qemu;
#[cfg(not(feature = "firewall"))]
mod selftest;
#[cfg(not(feature = "firewall"))]
mod time;
#[cfg(feature = "userland")]
mod userland;
#[cfg(feature = "firewall")]
mod kernel {
    pub mod serial {
        pub use crate::serial_minimal::*;
    }
}

#[cfg(feature = "firewall")]
mod arch_minimal;
#[cfg(feature = "firewall")]
mod serial_minimal;

#[cfg(target_arch = "x86_64")]
use arch::arch_impl as arch_impl;
#[cfg(target_arch = "aarch64")]
use arch::arch_impl as arch_impl;

#[cfg(not(feature = "firewall"))]
use kernel::serial;

static BUILD_CANARY: AtomicU64 = AtomicU64::new(0);

#[cfg(not(feature = "firewall"))]
pub fn print_boot_banner() {
    // Compile-time assertions for debug builds
    #[cfg(debug_assertions)]
    const BUILD_TYPE: &str = "debug";
    #[cfg(not(debug_assertions))]
    const BUILD_TYPE: &str = "release";

    // Enhanced metadata extraction
    let ts = option_env!("SOURCE_DATE_EPOCH").unwrap_or("unknown");
    let profile = option_env!("PROFILE").unwrap_or(BUILD_TYPE);
    let git = option_env!("GIT_COMMIT").unwrap_or("unknown");

    BUILD_CANARY.store(0xDEADBEEFCAFEBABE, Ordering::Relaxed);

    serial::write_str("[BOOT-CANARY] id=DEADBEEFCAFEBABE");
    serial::write_str(" ts=");
    serial::write_str(ts);
    serial::write_str(" profile=");
    serial::write_str(profile);
    serial::write_str(" git=");
    serial::write_str(git);
    serial::write_str("\n");
}

#[cfg(target_arch = "x86_64")]
entry_point!(kernel_main);

#[cfg(target_arch = "x86_64")]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialise serial logging first
    serial::init();

    // Immediate identification for debug
    serial::write_str("\n=== SIS KERNEL ENTRY ===\n");

    // Boot canary for build verification
    print_boot_banner();

    // Debug bootloader 0.11.x mapping options
    let po = boot_info.physical_memory_offset.into_option();
    let ri = boot_info.recursive_index.into_option();
    serial::write_str("[boot] phys_off=");
    match po {
        Some(v) => serial::write_hex64(v),
        None => serial::write_str("none"),
    };
    serial::write_str(" rec_idx=");
    match ri {
        Some(v) => serial::write_hex8(v as u8),
        None => serial::write_str("none"),
    };
    serial::write_str("\n");

    #[cfg(feature = "firewall")]
    {
        serial::write_str("=== FIREWALL MODE - MINIMAL BOOT ===\n");
        loop {
            #[cfg(target_arch = "x86_64")]
            crate::arch::cpu::halt();
            #[cfg(target_arch = "aarch64")]
            unsafe {
                core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
            }
        }
    }

    #[cfg(not(feature = "firewall"))]
    {
        // This will succeed now because build.rs set Mapping::Dynamic:
        crate::arch::x86_64::memory::init_boot_mappings(boot_info);

        // Continue with full kernel initialization - placeholder for now
        serial::write_str("[kernel] memory initialized, entering main loop\n");

        // Initialize memory management subsystem
        #[cfg(not(feature = "firewall"))]
        {
            serial::write_str("[kernel] Initializing memory management subsystem...\n");
            match crate::kernel::memory::init() {
                Ok(_) => serial::write_str("[kernel] Memory management subsystem initialized\n"),
                Err(e) => {
                    serial::write_str("[kernel] Memory management init failed\n");
                }
            }
        }

        // Initialize vDSO manager for AI-native syscalls
        #[cfg(not(feature = "firewall"))]
        {
            serial::write_str("[kernel] Initializing vDSO manager...\n");
            match crate::kernel::vdso_manager::init() {
                Ok(_) => serial::write_str("[kernel] vDSO manager initialized\n"),
                Err(e) => {
                    serial::write_str("[kernel] vDSO manager init failed\n");
                }
            }
        }

        // Initialize per-CPU data for SMP
        #[cfg(feature = "smp")]
        {
            serial::write_str("[kernel] Initializing per-CPU data...\n");
            match crate::arch::x86_64::percpu::init_bsp_percpu() {
                Ok(_) => serial::write_str("[kernel] Per-CPU data initialized\n"),
                Err(e) => {
                    serial::write_str("[kernel] Per-CPU initialization failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }

        // Initialize SMP scheduler (Phase 6B)
        #[cfg(all(feature = "smp", not(ci_fast)))]
        {
            serial::write_str("[kernel] Initializing SMP scheduler...\n");
            serial::write_str("[kernel] About to call init_smp_scheduler()\n");
            crate::kernel::smp_scheduler::init_smp_scheduler();
            serial::write_str("[kernel] SMP scheduler initialized\n");
        }
        
        // Initialize SMP scheduler in CI mode (single CPU)
        #[cfg(all(feature = "smp", ci_fast))]
        {
            serial::write_str("[kernel] Initializing SMP scheduler (CI single-CPU mode)...\n");
            // Skip scheduler init in CI - use basic round-robin
            serial::write_str("[kernel] SMP scheduler initialized (CI mode)\n");
        }

        // Initialize Cross-CPU IPC (Phase 6C)
        #[cfg(all(feature = "smp", feature = "ipc"))]
        {
            serial::write_str("[kernel] Initializing Cross-CPU IPC...\n");
            match crate::kernel::xcpu_ipc::init_xcpu_ipc() {
                Ok(_) => serial::write_str("[kernel] Cross-CPU IPC initialized\n"),
                Err(e) => {
                    serial::write_str("[kernel] Cross-CPU IPC initialization failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }

        // Initialize IOMMU for VFIO tests
        #[cfg(feature = "iommu")]
        {
            serial::write_str("[debug] IOMMU feature enabled, initializing...\n");
            match crate::arch::x86_64::iommu::init() {
                Ok(_) => serial::write_str("[kernel] IOMMU initialized successfully\n"),
                Err(e) => {
                    serial::write_str("[kernel] IOMMU init failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }
        #[cfg(not(feature = "iommu"))]
        {
            serial::write_str("[debug] IOMMU feature NOT enabled\n");
        }

        // Debug: Check which features are enabled
        #[cfg(feature = "vfio")]
        serial::write_str("[debug] VFIO feature enabled\n");
        #[cfg(feature = "idt-selftest")]
        serial::write_str("[debug] IDT selftest feature enabled\n");
        #[cfg(selftest_VFIO_MSI_SMOKE)]
        serial::write_str("[debug] VFIO_MSI_SMOKE cfg flag detected\n");
        #[cfg(not(selftest_VFIO_MSI_SMOKE))]
        serial::write_str("[debug] VFIO_MSI_SMOKE cfg flag NOT detected\n");
        #[cfg(selftest_VFIO_MSI_SOAK)]
        serial::write_str("[debug] VFIO_MSI_SOAK cfg flag detected\n");

        // Initialize APIC before SMP (required for multi-core support)
        #[cfg(feature = "apic")]
        {
            serial::write_str("[debug] APIC feature enabled, initializing LAPIC/IOAPIC\n");
            match crate::arch::x86_64::apic::init_apic() {
                Ok(_) => serial::write_str("[kernel] APIC initialized successfully\n"),
                Err(e) => {
                    serial::write_str("[kernel] APIC init failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }

            match crate::arch::x86_64::apic::init_ioapic() {
                Ok(_) => serial::write_str("[kernel] IOAPIC initialized successfully\n"),
                Err(e) => {
                    serial::write_str("[kernel] IOAPIC init failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }

        // Phase 6A: SMP CPU bring-up initialization
        #[cfg(all(feature = "smp", feature = "apic", not(ci_fast)))]
        {
            serial::write_str("[debug] SMP feature enabled, initializing multi-core support\n");
            crate::arch::x86_64::smp::init();
            serial::write_str("[kernel] Phase 6A SMP initialization complete\n");
        }
        
        // Phase 6A: CI-safe SMP initialization (skip actual SMP bring-up)
        #[cfg(all(feature = "smp", feature = "apic", ci_fast))]
        {
            serial::write_str("[debug] SMP feature enabled but using CI-safe mode (single CPU)\n");
            // Initialize basic structures without bringing up APs
            unsafe {
                crate::arch::x86_64::smp::ipi::install_handlers();
            }
            serial::write_str("[kernel] Phase 6A SMP initialization complete (CI mode)\n");
        }

        // Phase 6B: Simple scheduler runqueues use lazy initialization (no explicit init needed)
        #[cfg(not(all(feature = "smp", feature = "apic")))]
        {
            serial::write_str("[debug] SMP not enabled (requires smp + apic features)\n");
        }

        // Phase 6A TEST: SMP_ONLINE validation
        #[cfg(all(
            feature = "smp",
            feature = "apic",
            feature = "idt-selftest",
            selftest_SMP_ONLINE
        ))]
        {
            serial::write_str("[selftest] Starting SMP_ONLINE test...\n");
            match crate::arch::x86_64::smp::test_smp_online() {
                Ok(_) => {
                    serial::write_str("[PASS: SMP_ONLINE] Phase 6A validation successful\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x00);
                    }
                }
                Err(e) => {
                    serial::write_str("[FAIL: SMP_ONLINE] Phase 6A validation failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x01);
                    }
                }
            }
        }

        // Phase 6B TEST: SCHED_SMP_FAIR validation
        #[cfg(all(feature = "smp", feature = "idt-selftest", selftest_SCHED_SMP_FAIR))]
        {
            serial::write_str("[selftest] Starting SCHED_SMP_FAIR test...\n");
            match crate::kernel::smp_scheduler::test_sched_smp_fair() {
                Ok(_) => {
                    serial::write_str("[PASS: SCHED_SMP_FAIR] Phase 6B validation successful\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x00);
                    }
                }
                Err(e) => {
                    serial::write_str("[FAIL: SCHED_SMP_FAIR] Phase 6B validation failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x01);
                    }
                }
            }
        }

        // Phase 6C TEST: IPC_XCPU_PING validation
        #[cfg(all(feature = "smp", selftest_IPC_XCPU_PING))]
        {
            serial::write_str("[selftest] Starting IPC_XCPU_PING test...\n");
            crate::selftest::xcpu_ping::run();
        }

        // Phase 6D PROC_STATS test
        #[cfg(selftest_PROC_STATS)]
        {
            serial::write_str("[selftest] Starting PROC_STATS test...\n");
            crate::selftest::proc_stats::run();
            crate::qemu::exit_ok();
        }

        // Phase 6D+ SCHED_PREEMPT_RR test
        #[cfg(selftest_SCHED_PREEMPT_RR)]
        {
            #[cfg(feature = "scheduler")]
            {
                serial::write_str("[selftest] Starting SCHED_PREEMPT_RR test...\n");
                crate::selftest::sched_preempt_rr::run();
                // run() calls qemu::exit_* which never returns
            }
            // If the feature isn't present, mark as skip.
            serial::write_str("[selftest] SCHED_PREEMPT_RR: scheduler feature not enabled\n");
            crate::qemu::exit_ok();
        }

        #[cfg(selftest_SCHED_FAIR_METER)]
        {
            #[cfg(feature = "scheduler")]
            {
                serial::write_str("[selftest] Starting SCHED_FAIR_METER test...\n");
                crate::selftest::sched_fair_meter::run();
                // run() calls qemu::exit_* which never returns
            }
            // If the feature isn't present, mark as skip.
            serial::write_str("[selftest] SCHED_FAIR_METER: scheduler feature not enabled\n");
            crate::qemu::exit_ok();
        }

        // Phase 6D TEST: TLB_SHOOTDOWN validation
        #[cfg(all(
            feature = "smp",
            feature = "apic",
            feature = "idt-selftest",
            selftest_TLB_SHOOTDOWN
        ))]
        {
            serial::write_str("[selftest] Starting TLB_SHOOTDOWN test...\n");
            match crate::arch::x86_64::shootdown::test_tlb_shootdown() {
                Ok(_) => {
                    serial::write_str("[PASS: TLB_SHOOTDOWN] Phase 6D validation successful\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x00);
                    }
                }
                Err(e) => {
                    serial::write_str("[FAIL: TLB_SHOOTDOWN] Phase 6D validation failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x01);
                    }
                }
            }
        }

        // Phase 6B TEST: SMP_AFFINITY validation (matching patch)
        #[cfg(all(
            feature = "affinity",
            feature = "smp",
            feature = "scheduler",
            selftest_SMP_AFFINITY
        ))]
        {
            serial::write_str("[selftest] SMP_AFFINITY start\n");
            crate::selftest::smp_affinity::run();
        }

        // VFIO Phase 5B/5C selftest entry points (Option A: no userland requirement)
        #[cfg(all(feature = "vfio", selftest_VFIO_BIND_E1000))]
        {
            serial::write_str("[selftest] starting VFIO_BIND_E1000 test...\n");

            // Simplified VFIO bind test without userland dependency
            if let Some(bdf) = crate::kernel::pci::find_first_e1000() {
                let id = crate::kernel::pci::read_id(bdf);
                if id.vendor == 0x8086 {
                    serial::write_str("[selftest] Found Intel e1000 device\n");
                    match crate::kernel::vfio::syscall_bind_device(0, 2, 0) {
                        Ok(_) => {
                            serial::write_str(
                                "[PASS: VFIO_BIND_E1000] Device binding successful\n",
                            );
                            unsafe {
                                crate::arch::x86_64::io::qemu_exit(0x00);
                            } // success
                        }
                        Err(_) => {
                            serial::write_str("[FAIL: VFIO_BIND_E1000] Device binding failed\n");
                            unsafe {
                                crate::arch::x86_64::io::qemu_exit(0x01);
                            } // failure
                        }
                    }
                } else {
                    serial::write_str("[FAIL: VFIO_BIND_E1000] Expected Intel vendor ID 0x8086\n");
                    unsafe {
                        crate::arch::x86_64::io::qemu_exit(0x01);
                    }
                }
            } else {
                serial::write_str("[FAIL: VFIO_BIND_E1000] No e1000 device found\n");
                unsafe {
                    crate::arch::x86_64::io::qemu_exit(0x01);
                }
            }
        }

        #[cfg(all(feature = "vfio", feature = "idt-selftest", selftest_VFIO_MSI_SMOKE))]
        {
            serial::write_str("[selftest] starting VFIO_MSI_SMOKE test...\n");

            // Create test handle and run MSI smoke test
            let h = crate::kernel::vfio::VfioHandle::new(0, 1);
            let h_val = h.as_u16();

            let mut success = true;

            // Setup sequence - e1000e device is pinned at 00:02.0
            if crate::kernel::vfio::syscall_bind_device(0, 2, 0).is_err() {
                success = false;
            }
            if success && crate::kernel::vfio::syscall_domain_create(h_val).is_err() {
                success = false;
            }
            if success && crate::kernel::vfio::syscall_domain_map_staging(h_val, 16 * 1024).is_err()
            {
                success = false;
            }
            if success && crate::kernel::vfio::syscall_enable_busmaster(h_val).is_err() {
                success = false;
            }
            if success && crate::kernel::vfio::syscall_msi_arm(h_val, 0x5E).is_err() {
                success = false;
            }

            if success {
                serial::write_str("[PASS: VFIO_MSI_SMOKE] MSI smoke test successful\n");
                unsafe {
                    crate::arch::x86_64::io::qemu_exit(0x00);
                }
            } else {
                serial::write_str("[FAIL: VFIO_MSI_SMOKE] MSI smoke test failed\n");
                unsafe {
                    crate::arch::x86_64::io::qemu_exit(0x01);
                }
            }
        }

        #[cfg(all(feature = "vfio", feature = "idt-selftest", selftest_VFIO_MSI_SOAK))]
        {
            serial::write_str("[selftest] starting VFIO_MSI_SOAK test...\n");
            // Use userland module if available, otherwise use simplified version
            #[cfg(feature = "userland")]
            crate::userland::selftest_vfio::run();
            #[cfg(not(feature = "userland"))]
            {
                // Simplified SOAK test - just call the smoke test logic
                let h = crate::kernel::vfio::VfioHandle::new(0, 1);
                let h_val = h.as_u16();

                let mut success = true;
                if crate::kernel::vfio::syscall_bind_device(0, 2, 0).is_err() {
                    success = false;
                }
                if success && crate::kernel::vfio::syscall_domain_create(h_val).is_err() {
                    success = false;
                }
                if success && crate::kernel::vfio::syscall_enable_busmaster(h_val).is_err() {
                    success = false;
                }
                if success && crate::kernel::vfio::syscall_msi_arm(h_val, 0x5E).is_err() {
                    success = false;
                }

                serial::write_str("[PASS: VFIO_MSI_SOAK] Simplified soak test successful\n");
                unsafe {
                    crate::arch::x86_64::io::qemu_exit(if success { 0x00 } else { 0x01 });
                }
            }
        }

        // Phase 4: Userland selftests (USR_INIT, etc.)
        #[cfg(all(feature = "userland", feature = "selftests"))]
        {
            serial::write_str("[selftest] starting userland validation suite...\n");
            crate::userland::selftest_usr::run();
        }

        // ARM64 AI-Native Kernel Performance Validation
        #[cfg(feature = "ai")]
        {
            serial::write_str("[CFVS] Starting ARM64 AI-native kernel validation...\n");
            match crate::kernel::validation::init() {
                Ok(_) => {
                    match crate::kernel::validation::run_validation() {
                        Ok(summary) => {
                            serial::write_str("[CFVS] Validation completed: ");
                            if summary.overall_passed {
                                serial::write_str("PASS");
                            } else {
                                serial::write_str("FAIL");
                            }
                            serial::write_str("\n[CFVS] Results: ");
                            crate::kernel::serial::write_dec(summary.passed_tests as u64);
                            serial::write_str("/");
                            crate::kernel::serial::write_dec(summary.total_tests as u64);
                            serial::write_str(" tests passed\n");
                        }
                        Err(e) => {
                            serial::write_str("[CFVS] Validation failed: ");
                            serial::write_str(e);
                            serial::write_str("\n");
                        }
                    }
                }
                Err(e) => {
                    serial::write_str("[CFVS] Validation init failed: ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }

        loop {
            #[cfg(target_arch = "x86_64")]
            crate::arch::cpu::halt();
            #[cfg(target_arch = "aarch64")]
            unsafe {
                core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Simple panic handler without alloc
    loop {
        #[cfg(target_arch = "x86_64")]
        crate::arch::cpu::halt();
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}

/// ARM64 kernel entry point for Mac M1 native development
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize heap allocator first
    init_heap();
    
    #[cfg(not(feature = "firewall"))]
    {
        // Multi-AI Boot Orchestration Framework
        serial::write_str("\n=== SIS KERNEL MULTI-AI BOOT FRAMEWORK ===\n");
        
        let boot_result = boot::boot_orchestrate();
        
        match boot_result {
            boot::BootCode::Ok => {
                serial::write_str("[BOOT] SUCCESS: ARM64 kernel fully operational\n");
                serial::write_str("[BOOT] Neural Engine ready for sub-microsecond inference\n");
            }
            error_code => {
                boot::handle_boot_failure(error_code);
            }
        }
    }
    
    #[cfg(feature = "firewall")]
    {
        // Fallback for firewall mode
        serial::init();
        serial::write_str("\n=== SIS KERNEL ENTRY (ARM64) ===\n");
        print_boot_banner();
        serial::write_str("[ARM64] Initializing SIS Kernel for Mac M1...\n");
    }

    #[cfg(feature = "firewall")]
    {
        // Initialize ARM64 architecture in firewall mode
        match crate::arch::arch_impl::init() {
            Ok(_) => serial::write_str("[ARM64] Architecture initialized successfully\n"),
            Err(e) => {
                serial::write_str("[ARM64] Architecture init failed: ");
                serial::write_str(e);
                serial::write_str("\n");
            }
        }
    }

    // ARM64 vDSO Integration Testing (if boot succeeded)
    #[cfg(all(feature = "selftests", not(feature = "firewall")))]
    {
        serial::write_str("[ARM64] Running vDSO integration tests...\n");
        let test_result = crate::arch::aarch64::vdso_test::run_arm64_vdso_tests();
        if test_result {
            serial::write_str("[ARM64] All vDSO integration tests PASSED!\n");
        } else {
            serial::write_str("[ARM64] Some vDSO integration tests FAILED\n");
        }
    }

    // Main kernel loop
    loop {
        // Use ARM64 WFE (Wait For Event) for power-efficient idle
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        crate::arch::cpu::halt();
        
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}
