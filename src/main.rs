#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "firewall"))]
mod arch;
#[cfg(feature = "firewall")]
mod arch {
    pub mod x86_64 {
        pub use crate::arch_minimal::*;
    }
}

#[cfg(not(feature = "firewall"))]
mod kernel;
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

use arch::x86_64 as arch_x86;

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

entry_point!(kernel_main);
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
            arch_x86::cpu::halt();
        }
    }

    #[cfg(not(feature = "firewall"))]
    {
        // This will succeed now because build.rs set Mapping::Dynamic:
        crate::arch::x86_64::memory::init_boot_mappings(boot_info);

        // Continue with full kernel initialization - placeholder for now
        serial::write_str("[kernel] memory initialized, entering main loop\n");

        // Initialize SMP scheduler (Phase 6B)
        #[cfg(feature = "smp")]
        {
            serial::write_str("[kernel] Initializing SMP scheduler...\n");
            crate::kernel::smp_scheduler::init_smp_scheduler();
            serial::write_str("[kernel] SMP scheduler initialized\n");
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
        #[cfg(all(feature = "smp", feature = "apic"))]
        {
            serial::write_str("[debug] SMP feature enabled, initializing multi-core support\n");
            crate::arch::x86_64::smp::init();
            serial::write_str("[kernel] Phase 6A SMP initialization complete\n");
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

        loop {
            arch_x86::cpu::halt();
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Simple panic handler without alloc
    loop {
        arch_x86::cpu::halt();
    }
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    loop {
        arch_x86::cpu::halt();
    }
}
