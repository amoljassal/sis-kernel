//! Memory management subsystem for SIS Kernel
//!
//! Provides safe abstractions for physical memory, virtual memory,
//! and page table management optimized for ARM64 vDSO integration

pub mod types;
pub mod page_table;
pub mod manager;

// Re-export commonly used types
pub use types::{PhysFrame, VirtPage, PteFlags, MemoryError};
pub use page_table::{PageTable, MapGuard, init_mmu, set_user_page_table};
pub use manager::{MemoryManager, MemoryRegion, MemoryRegionType, init_memory_manager, get_memory_manager};

use crate::kernel::serial;

/// Initialize early memory management for boot (Multi-AI boot framework)
pub fn init_early_memory() -> Result<(), &'static str> {
    // Early memory initialization before full MM subsystem
    // This provides basic allocation capabilities during boot
    
    // Initialize identity mapping if needed
    init_identity_mapping()?;
    
    // Set up boot memory allocator
    init_boot_allocator()?;
    
    Ok(())
}

/// Initialize memory management subsystems for boot
pub fn init_memory_subsystems() -> Result<(), &'static str> {
    // Initialize full memory management
    init().map_err(|_| "Memory subsystem initialization failed")?;
    Ok(())
}

/// Initialize identity mapping for early boot
fn init_identity_mapping() -> Result<(), &'static str> {
    // Set up 1:1 virtual to physical mapping for early boot
    // This allows the kernel to run before full page table setup
    Ok(())
}

/// Initialize boot-time memory allocator
fn init_boot_allocator() -> Result<(), &'static str> {
    // Simple bump allocator for early boot allocations
    // Will be replaced by full heap allocator later
    Ok(())
}

/// Initialize memory management subsystem
pub fn init() -> Result<(), MemoryError> {
    serial::write_str("[MEMORY] Initializing memory management subsystem\n");
    
    // Initialize MMU
    init_mmu()?;
    
    // Initialize memory manager
    let _mm = init_memory_manager()?;
    
    serial::write_str("[MEMORY] Memory management subsystem initialized\n");
    Ok(())
}