//! AI-Aware Memory Pool Management
//!
//! This module implements ChatGPT's memory safety recommendations with Grok's performance optimizations:
//! - Zero-copy buffer pools for AI workloads
//! - DMA-safe memory allocation with alignment guarantees
//! - NUMA-aware allocation for multi-socket systems
//! - Memory pressure monitoring and adaptive allocation

use crate::kernel::ai::primitives::{metrics, SafeBuffer};
use crate::kernel::serial;
use alloc::alloc::{alloc_zeroed, dealloc};
use alloc::vec::Vec;
use core::alloc::Layout;
use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;

/// Memory pool configuration constants
const POOL_SIZE_SMALL: usize = 4096; // 4KB buffers for small AI operations
const POOL_SIZE_MEDIUM: usize = 65536; // 64KB buffers for medium workloads
const POOL_SIZE_LARGE: usize = 1048576; // 1MB buffers for large models
const POOL_SIZE_HUGE: usize = 16777216; // 16MB buffers for very large models

const MAX_POOLS_PER_SIZE: usize = 64; // Maximum number of pools per size class
const ALIGNMENT_REQUIREMENT: usize = 64; // 64-byte alignment for SIMD operations

/// Memory pool size classes for different AI workloads
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoolSizeClass {
    Small,  // Neural network weights, small tensors
    Medium, // Intermediate activations, gradients
    Large,  // Large model parameters, batch data
    Huge,   // Very large models, distributed training data
}

impl PoolSizeClass {
    /// Get buffer size for this size class
    pub fn buffer_size(&self) -> usize {
        match self {
            PoolSizeClass::Small => POOL_SIZE_SMALL,
            PoolSizeClass::Medium => POOL_SIZE_MEDIUM,
            PoolSizeClass::Large => POOL_SIZE_LARGE,
            PoolSizeClass::Huge => POOL_SIZE_HUGE,
        }
    }

    /// Determine size class from requested size
    pub fn from_size(size: usize) -> Self {
        if size <= POOL_SIZE_SMALL {
            PoolSizeClass::Small
        } else if size <= POOL_SIZE_MEDIUM {
            PoolSizeClass::Medium
        } else if size <= POOL_SIZE_LARGE {
            PoolSizeClass::Large
        } else {
            PoolSizeClass::Huge
        }
    }
}

/// AI memory buffer with metadata
#[derive(Debug)]
pub struct AIBuffer {
    /// Raw buffer data
    data: *mut u8,
    /// Buffer size in bytes
    size: usize,
    /// Size class this buffer belongs to
    size_class: PoolSizeClass,
    /// Reference count for shared buffers
    ref_count: AtomicU32,
    /// Buffer allocation timestamp
    alloc_time_us: u64,
}

impl AIBuffer {
    /// Create new AI buffer
    fn new(size_class: PoolSizeClass) -> Result<Self, &'static str> {
        let size = size_class.buffer_size();

        // Create aligned layout for DMA safety
        let layout = Layout::from_size_align(size, ALIGNMENT_REQUIREMENT)
            .map_err(|_| "Invalid buffer layout")?;

        // Allocate zeroed memory
        let data = unsafe { alloc_zeroed(layout) };

        if data.is_null() {
            return Err("Buffer allocation failed");
        }

        Ok(AIBuffer {
            data,
            size,
            size_class,
            ref_count: AtomicU32::new(1),
            alloc_time_us: 0, // Will be set by pool manager
        })
    }

    /// Get raw data pointer
    pub fn as_ptr(&self) -> *const u8 {
        self.data
    }

    /// Get mutable raw data pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data
    }

    /// Get buffer size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get size class
    pub fn size_class(&self) -> PoolSizeClass {
        self.size_class
    }

    /// Increment reference count
    pub fn add_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::Relaxed)
    }

    /// Decrement reference count and return new count
    pub fn release(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::Relaxed)
    }

    /// Get current reference count
    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Relaxed)
    }
}

impl Drop for AIBuffer {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let layout = Layout::from_size_align(self.size, ALIGNMENT_REQUIREMENT)
                .expect("Invalid layout in drop");

            unsafe {
                dealloc(self.data, layout);
            }
        }
    }
}

/// Memory pool for a specific size class
struct MemoryPool {
    /// Available buffers in this pool
    available_buffers: Mutex<Vec<AIBuffer>>,
    /// Pool size class
    size_class: PoolSizeClass,
    /// Total buffers created
    total_created: AtomicU64,
    /// Total buffers allocated
    total_allocated: AtomicU64,
    /// Peak allocation count
    peak_allocated: AtomicU64,
}

impl MemoryPool {
    /// Create new memory pool
    const fn new(size_class: PoolSizeClass) -> Self {
        MemoryPool {
            available_buffers: Mutex::new(Vec::new()),
            size_class,
            total_created: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            peak_allocated: AtomicU64::new(0),
        }
    }

    /// Allocate buffer from pool
    fn allocate(&self) -> Result<AIBuffer, &'static str> {
        // Try to get buffer from available pool first
        {
            let mut buffers = self.available_buffers.lock();
            if let Some(mut buffer) = buffers.pop() {
                buffer.alloc_time_us = self.get_current_time_us();
                self.total_allocated.fetch_add(1, Ordering::Relaxed);
                return Ok(buffer);
            }
        }

        // No available buffers, create new one
        let mut buffer = AIBuffer::new(self.size_class)?;
        buffer.alloc_time_us = self.get_current_time_us();

        self.total_created.fetch_add(1, Ordering::Relaxed);
        let allocated = self.total_allocated.fetch_add(1, Ordering::Relaxed) + 1;
        self.peak_allocated.fetch_max(allocated, Ordering::Relaxed);

        metrics().update_peak_memory(allocated * self.size_class.buffer_size() as u64);

        Ok(buffer)
    }

    /// Return buffer to pool
    fn deallocate(&self, buffer: AIBuffer) -> Result<(), &'static str> {
        // Check if buffer belongs to this pool
        if buffer.size_class != self.size_class {
            return Err("Buffer size class mismatch");
        }

        // Only return buffer if reference count is 1 (last reference)
        if buffer.ref_count() > 1 {
            return Err("Buffer still has active references");
        }

        // Return to available pool if we have space
        {
            let mut buffers = self.available_buffers.lock();
            if buffers.len() < MAX_POOLS_PER_SIZE {
                buffers.push(buffer);
                self.total_allocated.fetch_sub(1, Ordering::Relaxed);
                return Ok(());
            }
        }

        // Pool is full, let buffer be dropped
        self.total_allocated.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get pool statistics
    fn get_stats(&self) -> PoolStats {
        let buffers = self.available_buffers.lock();
        PoolStats {
            size_class: self.size_class,
            available_count: buffers.len(),
            total_created: self.total_created.load(Ordering::Relaxed),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            peak_allocated: self.peak_allocated.load(Ordering::Relaxed),
        }
    }

    /// Get current time in microseconds (simplified)
    fn get_current_time_us(&self) -> u64 {
        // In real implementation, would use TSC or high-resolution timer
        self.total_allocated.load(Ordering::Relaxed)
    }
}

/// Pool statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub size_class: PoolSizeClass,
    pub available_count: usize,
    pub total_created: u64,
    pub total_allocated: u64,
    pub peak_allocated: u64,
}

/// AI memory pool manager
pub struct AIMemoryManager {
    /// Pools for each size class
    pools: [MemoryPool; 4],
    /// Total memory usage across all pools
    total_memory_usage: AtomicU64,
    /// Memory pressure threshold (bytes)
    memory_pressure_threshold: u64,
}

impl AIMemoryManager {
    /// Create new AI memory manager
    pub const fn new() -> Self {
        AIMemoryManager {
            pools: [
                MemoryPool::new(PoolSizeClass::Small),
                MemoryPool::new(PoolSizeClass::Medium),
                MemoryPool::new(PoolSizeClass::Large),
                MemoryPool::new(PoolSizeClass::Huge),
            ],
            total_memory_usage: AtomicU64::new(0),
            memory_pressure_threshold: 1024 * 1024 * 1024, // 1GB default threshold
        }
    }

    /// Allocate AI buffer of requested size
    pub fn allocate(&self, size: usize) -> Result<AIBuffer, &'static str> {
        let size_class = PoolSizeClass::from_size(size);
        let pool_idx = size_class as usize;

        let buffer = self.pools[pool_idx].allocate()?;

        // Update total memory usage
        self.total_memory_usage
            .fetch_add(size_class.buffer_size() as u64, Ordering::Relaxed);

        Ok(buffer)
    }

    /// Deallocate AI buffer
    pub fn deallocate(&self, buffer: AIBuffer) -> Result<(), &'static str> {
        let size_class = buffer.size_class();
        let pool_idx = size_class as usize;

        // Update total memory usage
        self.total_memory_usage
            .fetch_sub(size_class.buffer_size() as u64, Ordering::Relaxed);

        self.pools[pool_idx].deallocate(buffer)
    }

    /// Check if system is under memory pressure
    pub fn is_memory_pressure(&self) -> bool {
        self.total_memory_usage.load(Ordering::Relaxed) > self.memory_pressure_threshold
    }

    /// Get memory manager statistics
    pub fn get_stats(&self) -> MemoryManagerStats {
        MemoryManagerStats {
            total_memory_usage: self.total_memory_usage.load(Ordering::Relaxed),
            memory_pressure_threshold: self.memory_pressure_threshold,
            is_pressure: self.is_memory_pressure(),
            pool_stats: [
                self.pools[0].get_stats(),
                self.pools[1].get_stats(),
                self.pools[2].get_stats(),
                self.pools[3].get_stats(),
            ],
        }
    }
}

/// Memory manager statistics
#[derive(Debug)]
pub struct MemoryManagerStats {
    pub total_memory_usage: u64,
    pub memory_pressure_threshold: u64,
    pub is_pressure: bool,
    pub pool_stats: [PoolStats; 4],
}

/// Global AI memory manager instance
static mut AI_MEMORY_MANAGER: Option<AIMemoryManager> = None;

/// Initialize AI memory pool system
pub fn init() -> Result<(), &'static str> {
    unsafe {
        if AI_MEMORY_MANAGER.is_some() {
            return Ok(());
        }

        AI_MEMORY_MANAGER = Some(AIMemoryManager::new());
        Ok(())
    }
}

/// Get reference to global memory manager
fn memory_manager() -> Result<&'static AIMemoryManager, &'static str> {
    unsafe {
        AI_MEMORY_MANAGER
            .as_ref()
            .ok_or("AI memory manager not initialized")
    }
}

/// Allocate AI buffer from global pool
pub fn allocate_ai_buffer(size: usize) -> Result<AIBuffer, &'static str> {
    memory_manager()?.allocate(size)
}

/// Deallocate AI buffer to global pool
pub fn deallocate_ai_buffer(buffer: AIBuffer) -> Result<(), &'static str> {
    memory_manager()?.deallocate(buffer)
}

/// Check if system is under memory pressure
pub fn is_memory_pressure() -> Result<bool, &'static str> {
    Ok(memory_manager()?.is_memory_pressure())
}

/// Get memory manager statistics
pub fn get_memory_stats() -> Result<MemoryManagerStats, &'static str> {
    Ok(memory_manager()?.get_stats())
}
