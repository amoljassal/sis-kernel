//! DMA-safe buffer management with typestate for ARM64
//!
//! Provides memory-safe DMA buffer management using typestate pattern
//! to enforce proper cache maintenance for Neural Engine and other DMA devices.

use core::marker::PhantomData;
use core::ptr::NonNull;
use super::mmio::{dcache_clean_range, dcache_invalidate_range, dmb_ish, dmb_ishst};

/// CPU owns the buffer - safe for CPU access
pub enum CpuOwned {}

/// Device owns the buffer - CPU must not access
pub enum DeviceOwned {}

/// DMA-safe buffer with enforced cache coherency via typestate
pub struct DmaBuffer<State> {
    ptr: NonNull<u8>,
    len: usize,
    align: usize,
    _state: PhantomData<State>,
}

impl<State> DmaBuffer<State> {
    /// Get buffer length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get buffer alignment
    pub fn align(&self) -> usize {
        self.align
    }

    /// Get raw pointer (use with caution)
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl DmaBuffer<CpuOwned> {
    /// Create DMA buffer from raw allocation
    /// 
    /// # Safety
    /// - `ptr` must be valid DMA-capable memory with `len` bytes
    /// - `ptr` must be aligned to `align` bytes
    /// - Caller must ensure exclusive access during buffer lifetime
    pub unsafe fn from_raw(ptr: *mut u8, len: usize, align: usize) -> Self {
        debug_assert!(!ptr.is_null());
        debug_assert!(ptr as usize % align == 0, "Buffer not properly aligned");
        
        Self {
            ptr: NonNull::new_unchecked(ptr),
            len,
            align,
            _state: PhantomData,
        }
    }

    /// Get mutable slice for CPU access
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }

    /// Get immutable slice for CPU access
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    /// Prepare buffer for device access
    /// 
    /// Flushes CPU caches and transfers ownership to device.
    /// CPU must not access the buffer after this call until map_for_cpu().
    pub fn map_for_device(self) -> DmaBuffer<DeviceOwned> {
        unsafe {
            // Clean CPU caches to ensure device sees latest data
            dcache_clean_range(self.ptr.as_ptr() as usize, self.len);
            
            // Memory barrier to ensure cache operations complete
            dmb_ishst();
        }

        DmaBuffer {
            ptr: self.ptr,
            len: self.len,
            align: self.align,
            _state: PhantomData,
        }
    }

    /// Get NEON-aligned chunks for vectorized operations
    /// 
    /// Returns 16-byte aligned head chunk and remaining bytes.
    /// Useful for NEON SIMD operations on ARM64.
    pub fn as_neon_chunks(&mut self) -> Option<(&mut [u8; 16], &mut [u8])> {
        if self.len < 16 || self.ptr.as_ptr() as usize % 16 != 0 {
            return None;
        }

        unsafe {
            let head = &mut *(self.ptr.as_ptr() as *mut [u8; 16]);
            let tail = core::slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(16),
                self.len.saturating_sub(16)
            );
            Some((head, tail))
        }
    }
}

impl DmaBuffer<DeviceOwned> {
    /// Reclaim buffer from device for CPU access
    /// 
    /// Invalidates CPU caches and transfers ownership back to CPU.
    /// This ensures CPU sees any changes made by the device.
    pub fn map_for_cpu(self) -> DmaBuffer<CpuOwned> {
        unsafe {
            // Memory barrier to ensure device operations complete
            dmb_ish();
            
            // Invalidate CPU caches to ensure we see device changes
            dcache_invalidate_range(self.ptr.as_ptr() as usize, self.len);
        }

        DmaBuffer {
            ptr: self.ptr,
            len: self.len,
            align: self.align,
            _state: PhantomData,
        }
    }

    /// Get raw pointer for device programming
    /// 
    /// Safe to use for setting up device descriptors since
    /// CPU cannot access the buffer contents in this state.
    pub fn device_addr(&self) -> usize {
        self.ptr.as_ptr() as usize
    }
}

/// NEON-optimized operations for AI workloads
impl DmaBuffer<CpuOwned> {
    /// Vectorized memory clear using NEON
    /// 
    /// Faster than scalar memset for large buffers on ARM64.
    #[cfg(target_feature = "neon")]
    pub fn neon_clear(&mut self) {
        let slice = self.as_mut_slice();
        let len = slice.len();
        
        if len < 16 {
            // Fall back to scalar for small buffers
            slice.fill(0);
            return;
        }

        unsafe {
            let mut ptr = slice.as_mut_ptr();
            let end = ptr.add(len);
            let aligned_end = ptr.add(len & !15); // 16-byte aligned end

            // NEON vectorized clear (16 bytes at a time)
            let zero = core::arch::aarch64::vdupq_n_u8(0);
            while ptr < aligned_end {
                core::arch::aarch64::vst1q_u8(ptr, zero);
                ptr = ptr.add(16);
            }

            // Handle remaining bytes
            while ptr < end {
                *ptr = 0;
                ptr = ptr.add(1);
            }
        }
    }

    /// Vectorized copy using NEON
    #[cfg(target_feature = "neon")]
    pub fn neon_copy_from(&mut self, src: &[u8]) {
        let dst_slice = self.as_mut_slice();
        let copy_len = dst_slice.len().min(src.len());
        
        if copy_len < 16 {
            dst_slice[..copy_len].copy_from_slice(&src[..copy_len]);
            return;
        }

        unsafe {
            let mut src_ptr = src.as_ptr();
            let mut dst_ptr = dst_slice.as_mut_ptr();
            let aligned_len = copy_len & !15;

            // Vectorized copy (16 bytes at a time)
            for _ in 0..(aligned_len / 16) {
                let data = core::arch::aarch64::vld1q_u8(src_ptr);
                core::arch::aarch64::vst1q_u8(dst_ptr, data);
                src_ptr = src_ptr.add(16);
                dst_ptr = dst_ptr.add(16);
            }

            // Handle remaining bytes
            let remaining = copy_len - aligned_len;
            if remaining > 0 {
                core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, remaining);
            }
        }
    }
}

/// Allocator for DMA-safe buffers
pub struct DmaAllocator {
    // In real implementation, would track allocations
}

impl DmaAllocator {
    /// Create new DMA allocator
    pub const fn new() -> Self {
        Self {}
    }

    /// Allocate DMA-safe buffer
    /// 
    /// Returns buffer aligned to 64 bytes for optimal NEON/cache performance.
    pub fn alloc(&self, size: usize) -> Result<DmaBuffer<CpuOwned>, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};

        const DMA_ALIGN: usize = 64; // Cache line + NEON alignment

        let layout = Layout::from_size_align(size, DMA_ALIGN)
            .map_err(|_| "Invalid DMA buffer layout")?;

        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("DMA buffer allocation failed");
        }

        Ok(unsafe { DmaBuffer::from_raw(ptr, size, DMA_ALIGN) })
    }
}

/// Global DMA allocator instance
pub static DMA_ALLOCATOR: DmaAllocator = DmaAllocator::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_buffer_typestate() {
        let mut buffer = DMA_ALLOCATOR.alloc(64).unwrap();
        
        // Can access when CPU-owned
        let slice = buffer.as_mut_slice();
        slice[0] = 42;
        
        // Transfer to device
        let device_buffer = buffer.map_for_device();
        
        // Can get device address but not access contents
        let _addr = device_buffer.device_addr();
        
        // Transfer back to CPU
        let mut cpu_buffer = device_buffer.map_for_cpu();
        
        // Can access again
        assert_eq!(cpu_buffer.as_slice()[0], 42);
    }
}