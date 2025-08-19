//! Memory management for vDSO operations
//!
//! Implements capability-based addressing and cache management
//! Based on ChatGPT's zero-copy safety patterns

use super::{Region, RegionFlags, RegionId, RegionTable, ScatterGatherEntry, VdsoError};
use core::sync::atomic::Ordering;

/// Validate scatter-gather entry against region table
/// 
/// Returns (pointer, length, flags) if valid
#[inline(always)]
pub fn validate_sge(
    table: &RegionTable,
    sge: &ScatterGatherEntry,
    required_flags: u32,
) -> Result<(*mut u8, usize, u32), VdsoError> {
    // Bounds check region ID
    if sge.region_id.0 >= table.count {
        return Err(VdsoError::NoRegion);
    }
    
    // Get region descriptor
    let region = &table.regions[sge.region_id.0 as usize];
    
    // Check offset and length within region
    if sge.offset as u64 + sge.length as u64 > region.length as u64 {
        return Err(VdsoError::Bounds);
    }
    
    // Check required permissions
    if (region.flags.0 & required_flags) != required_flags {
        return Err(VdsoError::Permission);
    }
    
    // Compute user virtual address (safe: validated bounds)
    let ptr = (region.base_va + sge.offset as u64) as *mut u8;
    
    Ok((ptr, sge.length as usize, region.flags.0))
}

/// Find region containing a slice
/// 
/// Returns (RegionId, offset) if found
#[inline(always)]
pub fn locate_region(
    table: &RegionTable,
    slice: &[u8],
) -> Result<(RegionId, u32), VdsoError> {
    let slice_addr = slice.as_ptr() as u64;
    let slice_len = slice.len() as u64;
    
    // Linear search (could optimize with binary search if sorted)
    for i in 0..table.count {
        let region = &table.regions[i as usize];
        
        // Check if slice starts within this region
        if slice_addr >= region.base_va {
            let offset = slice_addr - region.base_va;
            if offset + slice_len <= region.length as u64 {
                // Check read permission
                if region.flags.0 & RegionFlags::READ == 0 {
                    return Err(VdsoError::Permission);
                }
                return Ok((RegionId(i), offset as u32));
            }
        }
    }
    
    Err(VdsoError::NoRegion)
}

/// Find region containing a mutable slice
#[inline(always)]
pub fn locate_region_mut(
    table: &RegionTable,
    slice: &mut [u8],
) -> Result<(RegionId, u32), VdsoError> {
    let slice_addr = slice.as_mut_ptr() as u64;
    let slice_len = slice.len() as u64;
    
    for i in 0..table.count {
        let region = &table.regions[i as usize];
        
        if slice_addr >= region.base_va {
            let offset = slice_addr - region.base_va;
            if offset + slice_len <= region.length as u64 {
                // Check write permission
                if region.flags.0 & RegionFlags::WRITE == 0 {
                    return Err(VdsoError::Permission);
                }
                return Ok((RegionId(i), offset as u32));
            }
        }
    }
    
    Err(VdsoError::NoRegion)
}

/// ARM64 cache management for DMA operations
/// 
/// Based on Grok's optimization strategies
pub mod cache {
    use super::RegionFlags;
    
    /// Prepare memory for device access (CPU -> Device)
    #[inline(always)]
    pub fn publish_to_device(ptr: *mut u8, len: usize, flags: u32) {
        if flags & RegionFlags::DMA_COHERENT != 0 {
            // Coherent memory: only need memory barrier
            unsafe {
                core::arch::asm!("dmb ishst", options(nostack, nomem, preserves_flags));
            }
        } else {
            // Non-coherent: clean cache + barrier
            unsafe {
                dcache_clean_range(ptr as usize, len);
                core::arch::asm!("dmb ishst", options(nostack, nomem, preserves_flags));
            }
        }
    }
    
    /// Prepare memory for CPU access (Device -> CPU)
    #[inline(always)]
    pub fn acquire_from_device(ptr: *mut u8, len: usize, flags: u32) {
        if flags & RegionFlags::DMA_COHERENT != 0 {
            // Coherent memory: only need memory barrier
            unsafe {
                core::arch::asm!("dmb ish", options(nostack, nomem, preserves_flags));
            }
        } else {
            // Non-coherent: barrier + invalidate cache
            unsafe {
                core::arch::asm!("dmb ish", options(nostack, nomem, preserves_flags));
                dcache_invalidate_range(ptr as usize, len);
            }
        }
    }
    
    /// Clean data cache by virtual address range
    #[inline(always)]
    unsafe fn dcache_clean_range(addr: usize, len: usize) {
        const CACHE_LINE_SIZE: usize = 64;
        
        // Align to cache line boundaries
        let start = addr & !(CACHE_LINE_SIZE - 1);
        let end = (addr + len + CACHE_LINE_SIZE - 1) & !(CACHE_LINE_SIZE - 1);
        
        let mut current = start;
        while current < end {
            // DC CVAC: Clean data cache by VA to PoC
            unsafe {
                core::arch::asm!(
                    "dc cvac, {}",
                    in(reg) current,
                    options(nostack, preserves_flags)
                );
            }
            current += CACHE_LINE_SIZE;
        }
        
        // Data synchronization barrier
        unsafe {
            core::arch::asm!("dsb ish", options(nostack, nomem, preserves_flags));
        }
    }
    
    /// Invalidate data cache by virtual address range
    #[inline(always)]
    unsafe fn dcache_invalidate_range(addr: usize, len: usize) {
        const CACHE_LINE_SIZE: usize = 64;
        
        let start = addr & !(CACHE_LINE_SIZE - 1);
        let end = (addr + len + CACHE_LINE_SIZE - 1) & !(CACHE_LINE_SIZE - 1);
        
        let mut current = start;
        while current < end {
            // DC IVAC: Invalidate data cache by VA to PoC
            unsafe {
                core::arch::asm!(
                    "dc ivac, {}",
                    in(reg) current,
                    options(nostack, preserves_flags)
                );
            }
            current += CACHE_LINE_SIZE;
        }
        
        // Data synchronization barrier
        unsafe {
            core::arch::asm!("dsb ish", options(nostack, nomem, preserves_flags));
        }
    }
    
    /// Prefetch data into L1 cache
    /// 
    /// From Grok: Prefetch 64-128 bytes ahead for predictable access
    #[inline(always)]
    pub fn prefetch_read(ptr: *const u8) {
        unsafe {
            // PRFM PLDL1KEEP: Prefetch for load, L1, temporal
            core::arch::asm!(
                "prfm pldl1keep, [{}]",
                in(reg) ptr,
                options(nostack, preserves_flags)
            );
        }
    }
    
    /// Prefetch data for write into L1 cache
    #[inline(always)]
    pub fn prefetch_write(ptr: *mut u8) {
        unsafe {
            // PRFM PSTL1KEEP: Prefetch for store, L1, temporal
            core::arch::asm!(
                "prfm pstl1keep, [{}]",
                in(reg) ptr,
                options(nostack, preserves_flags)
            );
        }
    }
}

/// Memory allocation hints for AI workloads
pub mod hints {
    /// Hint that memory will be accessed sequentially
    #[inline(always)]
    pub fn sequential_access(ptr: *const u8, len: usize) {
        // Prefetch ahead for sequential access
        const PREFETCH_DISTANCE: usize = 128;
        
        if len > PREFETCH_DISTANCE {
            unsafe {
                let prefetch_ptr = ptr.add(PREFETCH_DISTANCE);
                super::cache::prefetch_read(prefetch_ptr);
            }
        }
    }
    
    /// Hint that memory access will be strided
    #[inline(always)]
    pub fn strided_access(ptr: *const u8, stride: usize, count: usize) {
        // Prefetch first few strides
        const MAX_PREFETCH: usize = 4;
        
        let prefetch_count = core::cmp::min(count, MAX_PREFETCH);
        for i in 0..prefetch_count {
            unsafe {
                let prefetch_ptr = ptr.add(i * stride);
                super::cache::prefetch_read(prefetch_ptr);
            }
        }
    }
    
    /// Hint for random access pattern (no prefetch)
    #[inline(always)]
    pub fn random_access(_ptr: *const u8, _len: usize) {
        // No prefetch for random access
        // Prefetching would pollute cache
    }
}

/// Fast memory copy optimized for ARM64
/// 
/// Uses NEON for large copies, unrolled loops for small
#[inline(always)]
pub unsafe fn fast_copy(dst: *mut u8, src: *const u8, len: usize) {
    if len < 16 {
        // Small copy: byte-by-byte
        for i in 0..len {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
        }
    } else if len < 64 {
        // Medium copy: 8-byte chunks
        let chunks = len / 8;
        for i in 0..chunks {
            unsafe {
                let src_ptr = src.add(i * 8) as *const u64;
                let dst_ptr = dst.add(i * 8) as *mut u64;
                *dst_ptr = *src_ptr;
            }
        }
        
        // Copy remainder
        let remainder = len % 8;
        let offset = chunks * 8;
        for i in 0..remainder {
            unsafe {
                *dst.add(offset + i) = *src.add(offset + i);
            }
        }
    } else {
        // Large copy: Use NEON if available
        #[cfg(target_feature = "neon")]
        {
            unsafe {
                neon_copy(dst, src, len);
            }
        }
        
        #[cfg(not(target_feature = "neon"))]
        {
            // Fallback to memcpy
            core::ptr::copy_nonoverlapping(src, dst, len);
        }
    }
}

/// NEON-optimized memory copy
#[cfg(target_feature = "neon")]
#[inline(always)]
unsafe fn neon_copy(dst: *mut u8, src: *const u8, len: usize) {
    use core::arch::aarch64::*;
    
    // Copy 64-byte chunks using NEON
    let chunks = len / 64;
    for i in 0..chunks {
        unsafe {
            let src_ptr = src.add(i * 64);
            let dst_ptr = dst.add(i * 64);
            
            // Load 4x16 bytes
            let v0 = vld1q_u8(src_ptr);
            let v1 = vld1q_u8(src_ptr.add(16));
            let v2 = vld1q_u8(src_ptr.add(32));
            let v3 = vld1q_u8(src_ptr.add(48));
            
            // Store 4x16 bytes
            vst1q_u8(dst_ptr, v0);
            vst1q_u8(dst_ptr.add(16), v1);
            vst1q_u8(dst_ptr.add(32), v2);
            vst1q_u8(dst_ptr.add(48), v3);
        }
    }
    
    // Copy remainder
    let remainder = len % 64;
    if remainder > 0 {
        unsafe {
            let offset = chunks * 64;
            core::ptr::copy_nonoverlapping(
                src.add(offset),
                dst.add(offset),
                remainder
            );
        }
    }
}