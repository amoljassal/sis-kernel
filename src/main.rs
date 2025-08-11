#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

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

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialise serial logging first
    serial::init();
    
    // Immediate identification for debug
    serial::write_str("\n=== SIS KERNEL ENTRY ===\n");
    
    // Debug bootloader 0.11.x mapping options
    let po = boot_info.physical_memory_offset.into_option();
    let ri = boot_info.recursive_index.into_option();
    serial::write_str("[boot] phys_off=");
    match po { 
        Some(v) => serial::write_hex64(v), 
        None => serial::write_str("none") 
    };
    serial::write_str(" rec_idx=");
    match ri { Some(v) => serial::write_hex8(v as u8), None => serial::write_str("none") };
    serial::write_str("\n");
    
    #[cfg(feature = "firewall")]
    {
        serial::write_str("=== FIREWALL MODE - MINIMAL BOOT ===\n");
        loop { arch_x86::cpu::halt(); }
    }
    
    #[cfg(not(feature = "firewall"))]
    {
        // This will succeed now because build.rs set Mapping::Dynamic:
        crate::arch::x86_64::memory::init_boot_mappings(boot_info);
        
        // Continue with full kernel initialization - placeholder for now
        serial::write_str("[kernel] memory initialized, entering main loop\n");
        
        // VFIO Phase 5B selftest entry points
        #[cfg(all(feature = "vfio", selftest_VFIO_BIND_E1000))]
        {
            serial::write_str("[selftest] starting VFIO_BIND_E1000 test...\n");
            match crate::kernel::user::selftest::run_vfio_bind_e1000() {
                Ok(_) => {
                    serial::write_str("[PASS: VFIO_BIND_E1000] Device binding successful\n");
                    unsafe { crate::arch::x86_64::qemu_exit(0x00); } // success
                },
                Err(e) => {
                    serial::write_str("[FAIL: VFIO_BIND_E1000] ");
                    serial::write_str(e);
                    serial::write_str("\n");
                    unsafe { crate::arch::x86_64::qemu_exit(0x01); } // failure
                }
            }
        }
        
        loop { arch_x86::cpu::halt(); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Simple panic handler without alloc
    loop { arch_x86::cpu::halt(); }
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    loop { arch_x86::cpu::halt(); }
}