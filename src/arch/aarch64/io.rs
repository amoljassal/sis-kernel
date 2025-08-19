//! ARM64 I/O operations and MMIO access
//!
//! ARM64 doesn't have port-based I/O like x86_64, so this provides
//! Memory-Mapped I/O (MMIO) operations for device access

use core::ptr::{read_volatile, write_volatile};

/// ARM64 I/O operations (MMIO-based)
pub struct Io;

impl Io {
    /// Read 8-bit value from MMIO address
    #[inline]
    pub unsafe fn read_u8(addr: u64) -> u8 {
        read_volatile(addr as *const u8)
    }

    /// Write 8-bit value to MMIO address  
    #[inline]
    pub unsafe fn write_u8(addr: u64, value: u8) {
        write_volatile(addr as *mut u8, value);
    }

    /// Read 16-bit value from MMIO address
    #[inline]
    pub unsafe fn read_u16(addr: u64) -> u16 {
        read_volatile(addr as *const u16)
    }

    /// Write 16-bit value to MMIO address
    #[inline]
    pub unsafe fn write_u16(addr: u64, value: u16) {
        write_volatile(addr as *mut u16, value);
    }

    /// Read 32-bit value from MMIO address
    #[inline]
    pub unsafe fn read_u32(addr: u64) -> u32 {
        read_volatile(addr as *const u32)
    }

    /// Write 32-bit value to MMIO address
    #[inline]
    pub unsafe fn write_u32(addr: u64, value: u32) {
        write_volatile(addr as *mut u32, value);
    }

    /// Read 64-bit value from MMIO address
    #[inline]
    pub unsafe fn read_u64(addr: u64) -> u64 {
        read_volatile(addr as *const u64)
    }

    /// Write 64-bit value to MMIO address
    #[inline]
    pub unsafe fn write_u64(addr: u64, value: u64) {
        write_volatile(addr as *mut u64, value);
    }
}

/// Legacy port I/O compatibility layer (no-ops for ARM64)
/// These functions exist for compatibility with x86_64 code but do nothing

/// Read from I/O port (no-op on ARM64)
#[inline]
pub unsafe fn port_read_u8(_port: u16) -> u8 {
    0 // ARM64 has no port I/O
}

/// Write to I/O port (no-op on ARM64)  
#[inline]
pub unsafe fn port_write_u8(_port: u16, _value: u8) {
    // ARM64 has no port I/O - this is a no-op
}

/// Read 32-bit from I/O port (no-op on ARM64)
#[inline]
pub unsafe fn port_read_u32(_port: u16) -> u32 {
    0 // ARM64 has no port I/O
}

/// Write 32-bit to I/O port (no-op on ARM64)
#[inline]
pub unsafe fn port_write_u32(_port: u16, _value: u32) {
    // ARM64 has no port I/O - this is a no-op
}

/// QEMU exit for ARM64 (virt machine)
/// Uses QEMU's ARM64 virt machine power management
pub unsafe fn qemu_exit(exit_code: u8) -> ! {
    // QEMU ARM64 virt machine uses PSCI (Power State Coordination Interface)
    // Call PSCI_SYSTEM_OFF function
    let function_id = 0x84000008u64; // PSCI_SYSTEM_OFF
    
    core::arch::asm!(
        "mov x0, {}",
        "hvc #0", // Hypervisor Call to QEMU
        in(reg) function_id,
        options(noreturn)
    );
}

/// Power off the system (PSCI)
pub unsafe fn power_off() -> ! {
    qemu_exit(0)
}

/// Reset the system (PSCI)  
pub unsafe fn reset() -> ! {
    let function_id = 0x84000009u64; // PSCI_SYSTEM_RESET
    
    core::arch::asm!(
        "mov x0, {}",
        "hvc #0",
        in(reg) function_id,
        options(noreturn)
    );
}

/// Common MMIO device addresses for ARM64 platforms
pub mod devices {
    /// UART addresses for common ARM64 platforms
    pub mod uart {
        /// ARM PL011 UART base address (QEMU virt)
        pub const PL011_BASE: u64 = 0x0900_0000;
        
        /// UART data register offset
        pub const UARTDR: u64 = 0x000;
        
        /// UART flag register offset
        pub const UARTFR: u64 = 0x018;
        
        /// UART control register offset  
        pub const UARTCR: u64 = 0x030;
        
        /// UART interrupt mask register offset
        pub const UARTIMSC: u64 = 0x038;
    }

    /// Generic Interrupt Controller (GIC) addresses
    pub mod gic {
        /// GIC Distributor base (QEMU virt)
        pub const GICD_BASE: u64 = 0x0800_0000;
        
        /// GIC CPU interface base (GICv2, deprecated in GICv3+)
        pub const GICC_BASE: u64 = 0x0801_0000;
        
        /// GIC Redistributor base (GICv3+)
        pub const GICR_BASE: u64 = 0x0808_0000;
    }

    /// Timer addresses
    pub mod timer {
        /// Generic Timer Control/Status register
        pub const CNTCTL_BASE: u64 = 0x0902_0000;
    }
}

/// Read from ARM PL011 UART for debug output
pub unsafe fn uart_read() -> Option<u8> {
    let uart_base = devices::uart::PL011_BASE;
    let flags = Io::read_u32(uart_base + devices::uart::UARTFR);
    
    if (flags & 0x10) == 0 { // RXFE bit clear means data available
        Some(Io::read_u8(uart_base + devices::uart::UARTDR))
    } else {
        None
    }
}

/// Write to ARM PL011 UART for debug output  
pub unsafe fn uart_write(byte: u8) {
    let uart_base = devices::uart::PL011_BASE;
    
    // Wait for TXFF bit to be clear (transmit FIFO not full)
    while (Io::read_u32(uart_base + devices::uart::UARTFR) & 0x20) != 0 {
        core::arch::asm!("nop");
    }
    
    Io::write_u8(uart_base + devices::uart::UARTDR, byte);
}