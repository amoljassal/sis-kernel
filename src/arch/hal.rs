//! Hardware Abstraction Layer (HAL) trait definitions
//!
//! Provides unified interfaces for AI-native kernel operations across different architectures

use crate::kernel::memory::{PhysFrame, VirtPage, PteFlags, MemoryError};

/// CPU abstraction trait
pub trait CpuHal {
    fn halt() -> !;
    fn pause();
    fn enable_interrupts();
    fn disable_interrupts();
    fn get_cpu_id() -> u32;
}

/// I/O abstraction trait  
pub trait IoHal {
    unsafe fn port_read_u8(port: u16) -> u8;
    unsafe fn port_write_u8(port: u16, value: u8);
    unsafe fn port_read_u32(port: u16) -> u32;
    unsafe fn port_write_u32(port: u16, value: u32);
}

/// Memory management abstraction trait
pub trait MemoryHal {
    fn map_page(virt: VirtPage, phys: PhysFrame, flags: PteFlags) -> Result<(), MemoryError>;
    fn unmap_page(virt: VirtPage) -> Result<(), MemoryError>;
    fn flush_tlb();
    fn flush_tlb_page(virt: VirtPage);
}

/// Interrupt management abstraction trait
pub trait InterruptHal {
    fn enable_interrupts();
    fn disable_interrupts();
    fn are_interrupts_enabled() -> bool;
    fn register_handler(vector: u8, handler: fn());
}

/// Architecture initialization trait
pub trait ArchInit {
    type Error: core::fmt::Debug;
    
    fn init() -> Result<(), Self::Error>;
    fn init_memory() -> Result<(), Self::Error>;
    fn init_interrupts() -> Result<(), Self::Error>;
}