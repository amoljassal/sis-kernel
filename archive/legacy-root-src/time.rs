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
#[cfg(target_arch = "x86_64")]
pub fn get_tsc_ms() -> u64 {
    // Assumes ~2GHz CPU for simplicity
    unsafe { core::arch::x86_64::_rdtsc() / 2_000_000 }
}

#[cfg(target_arch = "aarch64")]
pub fn get_tsc_ms() -> u64 {
    // Use ARM64 system timer (24MHz on most ARM systems)
    unsafe {
        let count: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) count);
        count / 24_000 // Convert to milliseconds from 24MHz counter
    }
}
