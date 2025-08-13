//! Time utilities for Phase 6B patch compatibility

/// Sleep for specified milliseconds (simplified for patch)
pub fn sleep_ms(ms: u64) {
    // Simple busy wait - in real implementation would yield to scheduler
    let iterations = ms * 1000; // Rough approximation
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}
