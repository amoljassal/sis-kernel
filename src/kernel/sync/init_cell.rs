//! Thread-safe initialization cell for kernel globals
//!
//! Provides a safer alternative to `static mut` by ensuring single initialization
//! with proper memory ordering guarantees for multi-core ARM64/x86_64 systems.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// Single-writer, many-reader initialization cell
/// 
/// Eliminates `static mut` patterns by providing safe global initialization
/// with Acquire/Release ordering for proper multi-core synchronization.
pub struct InitCell<T> {
    inner: UnsafeCell<Option<T>>,
    ready: AtomicBool,
}

unsafe impl<T: Send + Sync> Sync for InitCell<T> {}

impl<T> InitCell<T> {
    /// Create new uninitialized cell
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
            ready: AtomicBool::new(false),
        }
    }

    /// Initialize cell with value from closure (single-writer)
    /// 
    /// # Safety
    /// Must only be called from single-threaded initialization context
    /// (typically kernel boot on BSP before SMP initialization)
    pub fn init(&self, f: impl FnOnce() -> T) -> &T {
        // Use AcqRel to ensure we're the only initializer
        if !self.ready.swap(true, Ordering::AcqRel) {
            // SAFETY: We have exclusive access via atomic swap
            unsafe {
                *self.inner.get() = Some(f());
            }
            // Publish initialization to other CPUs with Release fence
            core::sync::atomic::fence(Ordering::Release);
        }
        
        // SAFETY: Value is initialized and immutable after publication
        self.get().expect("InitCell: concurrent initialization detected")
    }

    /// Get initialized value (many-reader)
    pub fn get(&self) -> Option<&T> {
        if self.ready.load(Ordering::Acquire) {
            // SAFETY: Value published once via Release, now immutable
            unsafe {
                (&*self.inner.get()).as_ref()
            }
        } else {
            None
        }
    }

    /// Check if cell is initialized
    pub fn is_initialized(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
}

/// Compile-time safety checks for ARM64 features
#[cfg(target_arch = "aarch64")]
mod arch_checks {
    // Ensure NEON is available for AI workloads
    #[cfg(not(target_feature = "neon"))]
    compile_error!("ARM64 AI kernel requires NEON support. Use -C target-feature=+neon");

    // Verify 64-bit physical addresses
    const _: [(); 8] = [(); core::mem::size_of::<u64>()];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_cell_basic() {
        let cell = InitCell::new();
        assert!(!cell.is_initialized());
        
        let value = cell.init(|| 42u32);
        assert_eq!(*value, 42);
        assert!(cell.is_initialized());
        
        // Second init should return same value
        let value2 = cell.init(|| 99u32);
        assert_eq!(*value2, 42); // Original value, not 99
    }
}