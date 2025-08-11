//! Minimal arch module for firewall mode

pub mod cpu {
    pub fn halt() -> ! {
        loop {
            x86_64::instructions::hlt();
        }
    }
}

pub mod gdt {
    pub fn init() {
        // Minimal GDT init stub
    }
}

pub mod idt {
    pub fn init_idt() {
        // Minimal IDT init stub  
    }
}

pub mod memory {
    pub fn enable_nxe_once() {
        // NXE stub
    }
}