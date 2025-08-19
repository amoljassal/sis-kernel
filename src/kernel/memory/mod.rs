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