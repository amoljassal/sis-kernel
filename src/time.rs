//! Time utilities for Phase 6B patch compatibility

/// Sleep for specified milliseconds (simplified for patch)
pub fn sleep_ms(ms: u64) {
    // Simple busy wait - in real implementation would yield to scheduler
    let iterations = ms * 1000; // Rough approximation
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

/// Get TSC value in milliseconds (rough approximation)
/// Assumes ~2GHz CPU for simplicity
pub fn get_tsc_ms() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() / 2_000_000 }
}
