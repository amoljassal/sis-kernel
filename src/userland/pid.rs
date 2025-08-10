//! Minimal PID allocator with wrap safety.
//! Reserves PID 0 (invalid) and 1 (usually init).
use core::sync::atomic::{AtomicU32, Ordering};

static NEXT_PID: AtomicU32 = AtomicU32::new(2);

#[derive(Debug, Clone, Copy)]
pub enum PidError {
    WrapExhausted,
}

/// Allocate the next PID. Never returns 0 or 1. On u32 wrap, returns error.
pub fn alloc_pid() -> Result<u32, PidError> {
    let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);
    if pid == 0 || pid == 1 {
        // extremely unlikely due to startup value, but protect invariants
        return Err(PidError::WrapExhausted);
    }
    if pid == u32::MAX {
        return Err(PidError::WrapExhausted);
    }
    Ok(pid)
}

/// Placeholder for future PID recycling (Phase 4.2/5).
pub fn free_pid(_pid: u32) {
    // For deterministic tests we don't recycle yet.
}