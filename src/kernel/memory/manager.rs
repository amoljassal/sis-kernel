//! Memory manager for vDSO kernel integration
//!
//! Provides frame allocation and kernel mapping capabilities
//! Based on ChatGPT's safe memory management patterns

use super::types::{PhysFrame, VirtPage, MemoryError};
use crate::kernel::serial;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Memory manager for kernel-side operations
pub struct MemoryManager {
    /// Free frame allocator
    frame_allocator: Mutex<FrameAllocator>,
    
    /// Kernel virtual memory allocator
    kernel_allocator: Mutex<KernelVirtualAllocator>,
    
    /// Statistics
    stats: MemoryStats,
}

/// Simple frame allocator for demonstration
/// 
/// Real implementation would integrate with existing SIS kernel allocator
struct FrameAllocator {
    /// Start of allocatable frames
    start_frame: PhysFrame,
    
    /// End of allocatable frames
    end_frame: PhysFrame,
    
    /// Next available frame
    next_frame: PhysFrame,
    
    /// Free frames list (for real implementation)
    free_frames: Vec<PhysFrame>,
}

impl FrameAllocator {
    /// Create new frame allocator
    pub fn new(start: PhysFrame, end: PhysFrame) -> Self {
        Self {
            start_frame: start,
            end_frame: end,
            next_frame: start,
            free_frames: Vec::new(),
        }
    }
    
    /// Allocate a physical frame
    pub fn alloc_frame(&mut self) -> Option<PhysFrame> {
        // Try free list first
        if let Some(frame) = self.free_frames.pop() {
            return Some(frame);
        }
        
        // Allocate from linear region
        if self.next_frame < self.end_frame {
            let frame = self.next_frame;
            self.next_frame = PhysFrame::new(self.next_frame.addr() + 4096);
            Some(frame)
        } else {
            None
        }
    }
    
    /// Free a physical frame
    pub fn free_frame(&mut self, frame: PhysFrame) {
        // Add to free list
        self.free_frames.push(frame);
    }
    
    /// Get allocator statistics
    pub fn stats(&self) -> AllocatorStats {
        let total_frames = (self.end_frame.addr() - self.start_frame.addr()) / 4096;
        let allocated_frames = (self.next_frame.addr() - self.start_frame.addr()) / 4096;
        let free_frames = self.free_frames.len() as u64;
        
        AllocatorStats {
            total_frames,
            allocated_frames: allocated_frames - free_frames,
            free_frames,
        }
    }
}

/// Kernel virtual memory allocator
struct KernelVirtualAllocator {
    /// Start of kernel virtual region
    start_va: VirtPage,
    
    /// End of kernel virtual region  
    end_va: VirtPage,
    
    /// Next available virtual address
    next_va: VirtPage,
    
    /// Active mappings for cleanup
    mappings: Vec<KernelMapping>,
}

impl KernelVirtualAllocator {
    /// Create new kernel virtual allocator
    pub fn new(start: VirtPage, end: VirtPage) -> Self {
        Self {
            start_va: start,
            end_va: end,
            next_va: start,
            mappings: Vec::new(),
        }
    }
    
    /// Allocate kernel virtual address
    pub fn alloc_kernel_va(&mut self) -> Option<VirtPage> {
        if self.next_va < self.end_va {
            let va = self.next_va;
            self.next_va = VirtPage::new(self.next_va.addr() + 4096);
            Some(va)
        } else {
            None
        }
    }
    
    /// Add mapping record
    pub fn add_mapping(&mut self, mapping: KernelMapping) {
        self.mappings.push(mapping);
    }
    
    /// Remove mapping record
    pub fn remove_mapping(&mut self, va: VirtPage) -> Option<KernelMapping> {
        if let Some(pos) = self.mappings.iter().position(|m| m.va == va) {
            Some(self.mappings.remove(pos))
        } else {
            None
        }
    }
}

/// Kernel mapping record
#[derive(Debug, Clone)]
struct KernelMapping {
    va: VirtPage,
    frame: PhysFrame,
    temporary: bool,
}

/// Memory statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub frames_allocated: AtomicU64,
    pub frames_freed: AtomicU64,
    pub kernel_mappings: AtomicU64,
    pub kernel_unmappings: AtomicU64,
}

/// Allocator statistics
#[derive(Debug, Clone, Copy)]
pub struct AllocatorStats {
    pub total_frames: u64,
    pub allocated_frames: u64,
    pub free_frames: u64,
}

impl MemoryManager {
    /// Create new memory manager
    pub fn new() -> Self {
        // Initialize with placeholder addresses
        // Real implementation would get these from bootloader/device tree
        let frame_start = PhysFrame::new(0x4000_0000); // 1GB start
        let frame_end = PhysFrame::new(0x8000_0000);   // 2GB end (1GB available)
        
        let kernel_va_start = VirtPage::new(0xFFFF_8000_0000_0000); // Kernel virtual space
        let kernel_va_end = VirtPage::new(0xFFFF_C000_0000_0000);   // 256GB kernel space
        
        Self {
            frame_allocator: Mutex::new(FrameAllocator::new(frame_start, frame_end)),
            kernel_allocator: Mutex::new(KernelVirtualAllocator::new(kernel_va_start, kernel_va_end)),
            stats: MemoryStats::default(),
        }
    }
    
    /// Allocate a physical frame
    pub fn alloc_frame(&self) -> Option<PhysFrame> {
        let frame = self.frame_allocator.lock().alloc_frame();
        if frame.is_some() {
            self.stats.frames_allocated.fetch_add(1, Ordering::Relaxed);
        }
        frame
    }
    
    /// Free a physical frame
    pub fn free_frame(&self, frame: PhysFrame) {
        self.frame_allocator.lock().free_frame(frame);
        self.stats.frames_freed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Map frame into kernel space temporarily
    /// 
    /// From ChatGPT: Temporary mapping for initialization
    pub fn map_kernel_temp(&self, frame: PhysFrame) -> Result<VirtPage, MemoryError> {
        let mut allocator = self.kernel_allocator.lock();
        let va = allocator.alloc_kernel_va().ok_or(MemoryError::OutOfMemory)?;
        
        // Create mapping record
        let mapping = KernelMapping {
            va,
            frame,
            temporary: true,
        };
        allocator.add_mapping(mapping);
        
        // In real implementation, would set up page table entry here
        // For now, just return the virtual address
        
        self.stats.kernel_mappings.fetch_add(1, Ordering::Relaxed);
        
        serial::write_str("[MM] Temporary kernel mapping: ");
        serial::write_hex64(frame.addr());
        serial::write_str(" -> ");
        serial::write_hex64(va.addr());
        serial::write_str("\n");
        
        Ok(va)
    }
    
    /// Unmap temporary kernel mapping
    pub fn unmap_kernel_temp(&self, va: VirtPage) -> Result<(), MemoryError> {
        let mut allocator = self.kernel_allocator.lock();
        
        if let Some(mapping) = allocator.remove_mapping(va) {
            if !mapping.temporary {
                return Err(MemoryError::InvalidAddress);
            }
            
            // In real implementation, would clear page table entry here
            
            self.stats.kernel_unmappings.fetch_add(1, Ordering::Relaxed);
            
            serial::write_str("[MM] Unmapped temporary kernel mapping: ");
            serial::write_hex64(va.addr());
            serial::write_str("\n");
            
            Ok(())
        } else {
            Err(MemoryError::NotMapped)
        }
    }
    
    /// Map frame into kernel space permanently
    /// 
    /// For vDSO communication page access
    pub fn map_kernel(&self, frame: PhysFrame) -> Result<VirtPage, MemoryError> {
        let mut allocator = self.kernel_allocator.lock();
        let va = allocator.alloc_kernel_va().ok_or(MemoryError::OutOfMemory)?;
        
        // Create mapping record
        let mapping = KernelMapping {
            va,
            frame,
            temporary: false,
        };
        allocator.add_mapping(mapping);
        
        // In real implementation, would set up page table entry here
        // For demonstration, we'll just track the mapping
        
        self.stats.kernel_mappings.fetch_add(1, Ordering::Relaxed);
        
        serial::write_str("[MM] Permanent kernel mapping: ");
        serial::write_hex64(frame.addr());
        serial::write_str(" -> ");
        serial::write_hex64(va.addr());
        serial::write_str("\n");
        
        Ok(va)
    }
    
    /// Unmap permanent kernel mapping
    pub fn unmap_kernel(&self, va: VirtPage) -> Result<(), MemoryError> {
        let mut allocator = self.kernel_allocator.lock();
        
        if let Some(_mapping) = allocator.remove_mapping(va) {
            // In real implementation, would clear page table entry here
            
            self.stats.kernel_unmappings.fetch_add(1, Ordering::Relaxed);
            
            serial::write_str("[MM] Unmapped permanent kernel mapping: ");
            serial::write_hex64(va.addr());
            serial::write_str("\n");
            
            Ok(())
        } else {
            Err(MemoryError::NotMapped)
        }
    }
    
    /// Get memory manager statistics
    pub fn get_stats(&self) -> (MemoryStats, AllocatorStats) {
        let mem_stats = MemoryStats {
            frames_allocated: AtomicU64::new(self.stats.frames_allocated.load(Ordering::Relaxed)),
            frames_freed: AtomicU64::new(self.stats.frames_freed.load(Ordering::Relaxed)),
            kernel_mappings: AtomicU64::new(self.stats.kernel_mappings.load(Ordering::Relaxed)),
            kernel_unmappings: AtomicU64::new(self.stats.kernel_unmappings.load(Ordering::Relaxed)),
        };
        
        let alloc_stats = self.frame_allocator.lock().stats();
        
        (mem_stats, alloc_stats)
    }
    
    /// Initialize memory manager with system memory map
    /// 
    /// This would typically be called during kernel boot
    pub fn init_with_memory_map(&mut self, _memory_map: &[MemoryRegion]) -> Result<(), MemoryError> {
        // In real implementation, would parse memory map and set up allocators
        // For now, we use the placeholder addresses from new()
        
        serial::write_str("[MM] Memory manager initialized\n");
        Ok(())
    }
}

/// Memory region from bootloader/device tree
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Available,
    Reserved,
    AcpiReclaimable,
    AcpiNonVolatile,
    BadMemory,
    Device,
}

/// Global memory manager instance
static mut MEMORY_MANAGER: Option<MemoryManager> = None;

/// Initialize global memory manager
pub fn init_memory_manager() -> Result<&'static mut MemoryManager, MemoryError> {
    unsafe {
        if MEMORY_MANAGER.is_none() {
            MEMORY_MANAGER = Some(MemoryManager::new());
        }
        
        MEMORY_MANAGER.as_mut().ok_or(MemoryError::OutOfMemory)
    }
}

/// Get global memory manager reference
pub fn get_memory_manager() -> Result<&'static mut MemoryManager, MemoryError> {
    unsafe {
        MEMORY_MANAGER.as_mut().ok_or(MemoryError::OutOfMemory)
    }
}