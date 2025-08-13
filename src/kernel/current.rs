//! Current task tracking for Phase 6B patch compatibility

use core::sync::atomic::{AtomicU64, Ordering};

static CURRENT_TID: AtomicU64 = AtomicU64::new(1);

/// Get current task ID (simplified for Phase 6B patch)
pub fn tid() -> u64 {
    CURRENT_TID.load(Ordering::Relaxed)
}

/// Set current task ID (for scheduling)
pub fn set_tid(new_tid: u64) {
    CURRENT_TID.store(new_tid, Ordering::Relaxed);
}