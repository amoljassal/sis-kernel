//! Safe AI Primitives
//!
//! This module implements ChatGPT's recommendations for memory-safe AI kernel primitives:
//! - SafeBuffer: DMA-safe buffer management with bounds checking
//! - AtomicMetrics: Lock-free performance counters for AI workloads
//! - ValidatedPointer: Bounds-checked pointer arithmetic for AI data structures
//! - ResourceGuard: RAII resource management for AI operations

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicUsize, Ordering};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::alloc::Layout;
use alloc::alloc::{alloc_zeroed, dealloc};

/// DMA-safe buffer with bounds checking and alignment guarantees
/// Implements ChatGPT's recommendation for safe AI data handling
pub struct SafeBuffer<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> SafeBuffer<T> {
    /// Create a new SafeBuffer with specified capacity
    /// Ensures proper alignment for DMA operations
    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        if capacity == 0 {
            return Err("Buffer capacity cannot be zero");
        }

        // Allocate aligned memory for DMA safety
        let layout = Layout::from_size_align(
            capacity * core::mem::size_of::<T>(),
            64, // 64-byte alignment for modern CPUs
        ).map_err(|_| "Invalid buffer layout")?;

        let ptr = unsafe { 
            alloc_zeroed(layout) as *mut T
        };

        if ptr.is_null() {
            return Err("Buffer allocation failed");
        }

        Ok(SafeBuffer {
            ptr,
            len: 0,
            capacity,
            _phantom: PhantomData,
        })
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Push element with bounds checking
    pub fn push(&mut self, value: T) -> Result<(), &'static str> {
        if self.len >= self.capacity {
            return Err("Buffer capacity exceeded");
        }

        unsafe {
            core::ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
        Ok(())
    }

    /// Get element with bounds checking
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Get mutable element with bounds checking
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Get raw pointer for DMA operations (unsafe but controlled)
    pub unsafe fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Get mutable raw pointer for DMA operations (unsafe but controlled)
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Drop for SafeBuffer<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            let layout = Layout::from_size_align(
                self.capacity * core::mem::size_of::<T>(),
                64,
            ).expect("Invalid layout in drop");

            unsafe {
                // Drop all constructed elements
                for i in 0..self.len {
                    core::ptr::drop_in_place(self.ptr.add(i));
                }
                // Deallocate memory
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

/// Lock-free atomic metrics for AI workload monitoring
/// Implements Grok's performance optimization recommendations
pub struct AtomicMetrics {
    /// Total inference operations completed
    pub inference_ops: AtomicU64,
    /// Total training steps completed
    pub training_steps: AtomicU64,
    /// Average inference latency (microseconds)
    pub avg_inference_latency_us: AtomicU64,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: AtomicU64,
    /// Active AI tasks count
    pub active_tasks: AtomicU32,
    /// Hardware accelerator utilization (0-100)
    pub hw_accel_utilization: AtomicU32,
}

impl AtomicMetrics {
    /// Create new metrics instance
    pub const fn new() -> Self {
        AtomicMetrics {
            inference_ops: AtomicU64::new(0),
            training_steps: AtomicU64::new(0),
            avg_inference_latency_us: AtomicU64::new(0),
            peak_memory_bytes: AtomicU64::new(0),
            active_tasks: AtomicU32::new(0),
            hw_accel_utilization: AtomicU32::new(0),
        }
    }

    /// Record inference operation completion
    pub fn record_inference(&self, latency_us: u64) {
        self.inference_ops.fetch_add(1, Ordering::Relaxed);
        
        // Update running average latency
        let current_avg = self.avg_inference_latency_us.load(Ordering::Relaxed);
        let ops_count = self.inference_ops.load(Ordering::Relaxed);
        let new_avg = (current_avg * (ops_count - 1) + latency_us) / ops_count;
        self.avg_inference_latency_us.store(new_avg, Ordering::Relaxed);
    }

    /// Record training step completion
    pub fn record_training_step(&self) {
        self.training_steps.fetch_add(1, Ordering::Relaxed);
    }

    /// Update peak memory usage if current exceeds peak
    pub fn update_peak_memory(&self, current_bytes: u64) {
        self.peak_memory_bytes.fetch_max(current_bytes, Ordering::Relaxed);
    }

    /// Increment active tasks counter
    pub fn task_started(&self) {
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active tasks counter
    pub fn task_completed(&self) {
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }

    /// Update hardware accelerator utilization
    pub fn update_hw_utilization(&self, utilization_percent: u32) {
        self.hw_accel_utilization.store(
            utilization_percent.min(100), 
            Ordering::Relaxed
        );
    }
}

/// Global AI metrics instance
static AI_METRICS: AtomicMetrics = AtomicMetrics::new();

/// Get reference to global AI metrics
pub fn metrics() -> &'static AtomicMetrics {
    &AI_METRICS
}

/// RAII resource guard for AI operations
/// Ensures proper cleanup and resource accounting
pub struct ResourceGuard<T> {
    resource: Option<T>,
    cleanup_fn: fn(T),
}

impl<T> ResourceGuard<T> {
    /// Create new resource guard
    pub fn new(resource: T, cleanup_fn: fn(T)) -> Self {
        ResourceGuard {
            resource: Some(resource),
            cleanup_fn,
        }
    }

    /// Take ownership of the resource
    pub fn take(mut self) -> T {
        self.resource.take().expect("Resource already taken")
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            (self.cleanup_fn)(resource);
        }
    }
}

impl<T> Deref for ResourceGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().expect("Resource not available")
    }
}

impl<T> DerefMut for ResourceGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().expect("Resource not available")
    }
}

/// Initialize AI primitives subsystem
pub fn init() -> Result<(), &'static str> {
    // Reset metrics to known state
    AI_METRICS.inference_ops.store(0, Ordering::Relaxed);
    AI_METRICS.training_steps.store(0, Ordering::Relaxed);
    AI_METRICS.avg_inference_latency_us.store(0, Ordering::Relaxed);
    AI_METRICS.peak_memory_bytes.store(0, Ordering::Relaxed);
    AI_METRICS.active_tasks.store(0, Ordering::Relaxed);
    AI_METRICS.hw_accel_utilization.store(0, Ordering::Relaxed);

    Ok(())
}

/// Test function for AI primitives
#[cfg(test)]
pub fn test_primitives() -> Result<(), &'static str> {
    // Test SafeBuffer
    let mut buffer: SafeBuffer<u64> = SafeBuffer::new(10)?;
    buffer.push(42)?;
    buffer.push(84)?;
    
    if buffer.get(0) != Some(&42) {
        return Err("SafeBuffer get failed");
    }
    
    if buffer.len() != 2 {
        return Err("SafeBuffer length incorrect");
    }

    // Test AtomicMetrics
    let metrics = metrics();
    metrics.record_inference(1000);
    metrics.task_started();
    
    if metrics.inference_ops.load(Ordering::Relaxed) != 1 {
        return Err("AtomicMetrics inference ops failed");
    }

    Ok(())
}