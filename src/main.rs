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

#[cfg(feature = "firewall")]
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
        // Full implementation when firewall is disabled - placeholder for now
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