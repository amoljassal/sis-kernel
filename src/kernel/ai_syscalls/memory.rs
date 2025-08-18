//! Memory Management for AI-Native Syscalls
//!
//! Zero-copy memory management based on multi-AI consultation:
//! - ChatGPT: Pinned region patterns and safety guarantees
//! - Grok: ARM64 cache management and DMA coherency
//! - Gemini: Hardware abstraction for AI memory operations

use super::{ScatterGatherEntry, CognitiveError};
use crate::kernel::serial;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Maximum number of pinned regions per process
const MAX_PINNED_REGIONS: usize = 1024;

/// Page size for ARM64 (4KB standard)
const PAGE_SIZE: usize = 4096;
const PAGE_MASK: usize = PAGE_SIZE - 1;

/// Memory access flags for pinned regions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessFlags {
    pub bits: u32,
}

impl AccessFlags {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const DMA_COHERENT: u32 = 1 << 2;
    pub const EXECUTABLE: u32 = 1 << 3;
    pub const LOCKED: u32 = 1 << 4;
}

/// Physical page frame information
#[derive(Debug, Clone, Copy)]
pub struct PhysFrame {
    pub addr: u64,
    pub flags: u32,
}

/// Pinned memory region for zero-copy operations (ChatGPT design)
pub struct PinnedTensorRegion {
    pub user_va: u64,
    pub phys_pages: Vec<PhysFrame>,
    pub len: usize,
    pub dma_coherent: bool,
    pub access_flags: AccessFlags,
    pub process_id: u64,
    pub region_id: u64,
}

impl PinnedTensorRegion {
    /// Create pinned region from scatter-gather entry
    pub fn from_sge(
        sge: &ScatterGatherEntry, 
        access: AccessFlags,
        process_id: u64,
    ) -> Result<Self, CognitiveError> {
        // Validate address and length
        if sge.user_addr == 0 || sge.len == 0 {
            return Err(CognitiveError::Invalid);
        }
        
        // Check alignment requirements (ARM64 requires 8-byte alignment)
        if sge.user_addr % 8 != 0 {
            return Err(CognitiveError::Invalid);
        }
        
        // Calculate page-aligned range
        let start_addr = sge.user_addr & !(PAGE_SIZE as u64 - 1);
        let end_addr = (sge.user_addr + sge.len as u64 + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);
        let total_len = (end_addr - start_addr) as usize;
        let num_pages = total_len / PAGE_SIZE;
        
        // Pin user pages (placeholder - integrate with actual page management)
        let phys_pages = pin_user_pages(start_addr, num_pages)?;
        
        // Check for DMA coherency
        let dma_coherent = (sge.flags & super::CognitiveFlags::DMA_COHERENT) != 0 ||
                          is_dma_coherent_memory(sge.user_addr);
        
        let region_id = allocate_region_id();
        
        Ok(PinnedTensorRegion {
            user_va: sge.user_addr,
            phys_pages,
            len: sge.len as usize,
            dma_coherent,
            access_flags: access,
            process_id,
            region_id,
        })
    }
    
    /// Prepare region for device access (Grok's cache management)
    pub fn prepare_for_device(&self) {
        if !self.dma_coherent {
            // Clean cache lines to ensure device sees latest data
            unsafe {
                let addr = self.user_va as usize;
                dcache_clean_range(addr, self.len);
                memory_barrier_release();
            }
        } else {
            // Just memory barrier for coherent memory
            unsafe { memory_barrier_release(); }
        }
    }
    
    /// Prepare region for CPU access after device operation
    pub fn prepare_for_cpu(&self) {
        if !self.dma_coherent {
            // Invalidate cache to see device-written data
            unsafe {
                memory_barrier_acquire();
                dcache_invalidate_range(self.user_va as usize, self.len);
            }
        } else {
            // Just memory barrier for coherent memory
            unsafe { memory_barrier_acquire(); }
        }
    }
    
    /// Get physical address for DMA operations
    pub fn get_dma_address(&self) -> Result<u64, CognitiveError> {
        if self.phys_pages.is_empty() {
            return Err(CognitiveError::Invalid);
        }
        
        // For single page or contiguous pages, return first page address
        let page_offset = self.user_va & (PAGE_SIZE as u64 - 1);
        Ok(self.phys_pages[0].addr + page_offset)
    }
    
    /// Check if region is physically contiguous
    pub fn is_contiguous(&self) -> bool {
        if self.phys_pages.len() <= 1 {
            return true;
        }
        
        for i in 1..self.phys_pages.len() {
            let expected_addr = self.phys_pages[i-1].addr + PAGE_SIZE as u64;
            if self.phys_pages[i].addr != expected_addr {
                return false;
            }
        }
        true
    }
    
    /// Get scatter-gather list for non-contiguous regions
    pub fn get_sg_list(&self) -> Vec<(u64, usize)> {
        let mut sg_list = Vec::new();
        let mut remaining = self.len;
        let page_offset = (self.user_va & (PAGE_SIZE as u64 - 1)) as usize;
        
        for (i, page) in self.phys_pages.iter().enumerate() {
            let offset = if i == 0 { page_offset } else { 0 };
            let page_bytes = core::cmp::min(PAGE_SIZE - offset, remaining);
            
            sg_list.push((page.addr + offset as u64, page_bytes));
            remaining -= page_bytes;
            
            if remaining == 0 {
                break;
            }
        }
        
        sg_list
    }
}

impl Drop for PinnedTensorRegion {
    fn drop(&mut self) {
        // Unpin pages when region is dropped
        if let Err(e) = unpin_user_pages(&self.phys_pages) {
            serial::write_str("[MEMORY] Warning: Failed to unpin pages: ");
            serial::write_dec(e as i32 as u64);
            serial::write_str("\n");
        }
        
        // Remove from registry
        let mut registry = PINNED_REGIONS.lock();
        registry.remove(&self.region_id);
    }
}

/// Memory pool for AI tensor allocations
pub struct TensorMemoryPool {
    pool_id: u32,
    total_size: usize,
    allocated_size: AtomicU64,
    allocation_count: AtomicU64,
    contiguous_regions: Vec<ContiguousRegion>,
}

/// Contiguous memory region for large tensors
#[derive(Debug, Clone)]
pub struct ContiguousRegion {
    pub phys_addr: u64,
    pub size: usize,
    pub allocated: bool,
    pub dma_coherent: bool,
}

impl TensorMemoryPool {
    /// Create new tensor memory pool
    pub fn new(pool_id: u32, size: usize) -> Result<Self, CognitiveError> {
        // Allocate contiguous regions for tensor storage
        let contiguous_regions = allocate_contiguous_regions(size)?;
        
        Ok(TensorMemoryPool {
            pool_id,
            total_size: size,
            allocated_size: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            contiguous_regions,
        })
    }
    
    /// Allocate tensor region from pool
    pub fn allocate_tensor(&mut self, size: usize, alignment: usize) -> Result<ContiguousRegion, CognitiveError> {
        // Find suitable contiguous region
        for region in &mut self.contiguous_regions {
            if !region.allocated && region.size >= size {
                region.allocated = true;
                self.allocated_size.fetch_add(size as u64, Ordering::Relaxed);
                self.allocation_count.fetch_add(1, Ordering::Relaxed);
                return Ok(region.clone());
            }
        }
        
        Err(CognitiveError::NoMem)
    }
    
    /// Free tensor region back to pool
    pub fn free_tensor(&mut self, region: &ContiguousRegion) -> Result<(), CognitiveError> {
        for pool_region in &mut self.contiguous_regions {
            if pool_region.phys_addr == region.phys_addr {
                pool_region.allocated = false;
                self.allocated_size.fetch_sub(region.size as u64, Ordering::Relaxed);
                return Ok(());
            }
        }
        
        Err(CognitiveError::NoEnt)
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            pool_id: self.pool_id,
            total_size: self.total_size,
            allocated_size: self.allocated_size.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            available_regions: self.contiguous_regions.iter().filter(|r| !r.allocated).count(),
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub pool_id: u32,
    pub total_size: usize,
    pub allocated_size: u64,
    pub allocation_count: u64,
    pub available_regions: usize,
}

/// Global registries for memory management
static PINNED_REGIONS: Mutex<BTreeMap<u64, PinnedTensorRegion>> = Mutex::new(BTreeMap::new());
static TENSOR_POOLS: Mutex<BTreeMap<u32, TensorMemoryPool>> = Mutex::new(BTreeMap::new());
static NEXT_REGION_ID: AtomicU64 = AtomicU64::new(1);

/// Initialize memory management subsystem
pub fn init() -> Result<(), &'static str> {
    serial::write_str("[MEMORY] Initializing AI memory management\n");
    
    // Clear existing regions and pools
    PINNED_REGIONS.lock().clear();
    TENSOR_POOLS.lock().clear();
    
    // Create default tensor pool
    let default_pool = TensorMemoryPool::new(0, 64 * 1024 * 1024)?; // 64MB default pool
    TENSOR_POOLS.lock().insert(0, default_pool);
    
    serial::write_str("[MEMORY] AI memory management initialized\n");
    Ok(())
}

/// Pin scatter-gather entry for zero-copy operations
pub fn pin_sge_region(
    sge: &ScatterGatherEntry,
    access: AccessFlags,
    process_id: u64,
) -> Result<u64, CognitiveError> {
    let region = PinnedTensorRegion::from_sge(sge, access, process_id)?;
    let region_id = region.region_id;
    
    // Register pinned region
    let mut registry = PINNED_REGIONS.lock();
    if registry.len() >= MAX_PINNED_REGIONS {
        return Err(CognitiveError::NoMem);
    }
    
    registry.insert(region_id, region);
    Ok(region_id)
}

/// Get pinned region by ID
pub fn get_pinned_region(region_id: u64) -> Result<&'static PinnedTensorRegion, CognitiveError> {
    // This is a simplified version - actual implementation would need proper lifetime management
    Err(CognitiveError::NoEnt)
}

/// Unpin region by ID
pub fn unpin_region(region_id: u64) -> Result<(), CognitiveError> {
    let mut registry = PINNED_REGIONS.lock();
    registry.remove(&region_id).ok_or(CognitiveError::NoEnt)?;
    Ok(())
}

/// ARM64 cache management functions (Grok's optimization)
#[inline(always)]
unsafe fn dcache_clean_range(addr: usize, len: usize) {
    let cache_line_size = 64; // ARM64 typical cache line size
    let start = addr & !(cache_line_size - 1);
    let end = (addr + len + cache_line_size - 1) & !(cache_line_size - 1);
    
    let mut current = start;
    while current < end {
        core::arch::asm!("dc cvac, {}", in(reg) current);
        current += cache_line_size;
    }
}

#[inline(always)]
unsafe fn dcache_invalidate_range(addr: usize, len: usize) {
    let cache_line_size = 64;
    let start = addr & !(cache_line_size - 1);
    let end = (addr + len + cache_line_size - 1) & !(cache_line_size - 1);
    
    let mut current = start;
    while current < end {
        core::arch::asm!("dc ivac, {}", in(reg) current);
        current += cache_line_size;
    }
}

#[inline(always)]
unsafe fn memory_barrier_release() {
    core::arch::asm!("dmb ishst", options(nostack, nomem));
}

#[inline(always)]
unsafe fn memory_barrier_acquire() {
    core::arch::asm!("dmb ish", options(nostack, nomem));
}

/// Check if memory region is DMA coherent
fn is_dma_coherent_memory(addr: u64) -> bool {
    // On ARM64, check if address is in coherent memory range
    // This is platform-specific and would need actual IOMMU integration
    false // Conservative default
}

/// Pin user pages (placeholder - integrate with actual page management)
fn pin_user_pages(start_addr: u64, num_pages: usize) -> Result<Vec<PhysFrame>, CognitiveError> {
    let mut pages = Vec::with_capacity(num_pages);
    
    // Placeholder implementation - would integrate with actual page tables
    for i in 0..num_pages {
        pages.push(PhysFrame {
            addr: 0x10000000 + (i * PAGE_SIZE) as u64, // Placeholder physical address
            flags: 0,
        });
    }
    
    Ok(pages)
}

/// Unpin user pages
fn unpin_user_pages(pages: &[PhysFrame]) -> Result<(), CognitiveError> {
    // Placeholder implementation - would integrate with actual page management
    Ok(())
}

/// Allocate contiguous regions for tensor pool
fn allocate_contiguous_regions(total_size: usize) -> Result<Vec<ContiguousRegion>, CognitiveError> {
    let mut regions = Vec::new();
    
    // Create several regions of different sizes for flexibility
    let region_sizes = [
        1024 * 1024,    // 1MB regions
        4 * 1024 * 1024, // 4MB regions
        16 * 1024 * 1024, // 16MB regions
    ];
    
    let mut remaining = total_size;
    for &size in region_sizes.iter().rev() {
        while remaining >= size {
            regions.push(ContiguousRegion {
                phys_addr: 0x20000000 + (total_size - remaining) as u64, // Placeholder
                size,
                allocated: false,
                dma_coherent: false,
            });
            remaining -= size;
        }
    }
    
    Ok(regions)
}

/// Allocate unique region ID
fn allocate_region_id() -> u64 {
    NEXT_REGION_ID.fetch_add(1, Ordering::Relaxed)
}

/// Get memory statistics
pub fn get_memory_stats() -> MemoryStats {
    let pinned_count = PINNED_REGIONS.lock().len();
    let pool_count = TENSOR_POOLS.lock().len();
    
    MemoryStats {
        pinned_regions: pinned_count,
        tensor_pools: pool_count,
        total_pinned_size: pinned_count * PAGE_SIZE, // Approximate
    }
}

/// Memory subsystem statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub pinned_regions: usize,
    pub tensor_pools: usize,
    pub total_pinned_size: usize,
}