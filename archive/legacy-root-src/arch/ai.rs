//! Architecture-Specific AI Module Re-exports
//!
//! This module provides a unified interface to architecture-specific AI
//! acceleration modules, enabling cross-platform compilation through
//! conditional re-exports.
//!
//! Based on Multi-AI consultation synthesis:
//! - ChatGPT: Immediate unblock via architecture shim
//! - Grok: Zero-cost abstractions with compile-time dispatch
//! - Gemini: Scalable patterns for distributed execution

// ARM64 (Apple M1/M2, ARM Cortex) - Production implementations
#[cfg(target_arch = "aarch64")]
pub use crate::arch::aarch64::{
    m1_neural_hal as ne_hal,
    neural_memory as ai_mem,
    neural_power as ai_power,
    predictive_power,
    atomic_bitmap,
    mmio_barriers,
    neural_hardware_probe as hw_probe,
};

// x86_64 - Fallback implementations for development and testing
#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::{
    simd_fallback as ne_hal,
    host_memory as ai_mem,
    thermal_control as ai_power,
    predictive_power, // This can be generic across architectures
    mock_probe as hw_probe,
};

// Compile-time validation for unsupported architectures
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("SIS kernel currently supports only aarch64 and x86_64 architectures");

/// AI Acceleration Capabilities - Architecture-Independent Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiCapabilities {
    /// Hardware Neural Processing Unit availability
    pub has_npu: bool,
    /// Peak inference throughput (TOPS)
    pub peak_tops: f32,
    /// Minimum inference latency (microseconds)
    pub min_latency_us: u32,
    /// Memory bandwidth (GB/s)
    pub memory_bandwidth_gbps: u32,
    /// Power management capabilities
    pub has_dvfs: bool,
    /// Distributed processing support
    pub supports_clustering: bool,
}

/// Platform-specific capability detection
pub fn detect_capabilities() -> AiCapabilities {
    hw_probe::detect_ai_capabilities()
}

/// Architecture-agnostic timer interface
pub mod timer {
    /// Read high-resolution timer counter
    #[cfg(target_arch = "aarch64")]
    pub fn read_counter() -> u64 {
        crate::arch::aarch64::cpu::read_timer_counter()
    }
    
    #[cfg(target_arch = "x86_64")]
    pub fn read_counter() -> u64 {
        // Use TSC for high-resolution timing on x86_64
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    
    /// Get timer frequency in Hz
    #[cfg(target_arch = "aarch64")]
    pub fn frequency() -> u64 {
        crate::arch::aarch64::cpu::get_timer_frequency()
    }
    
    #[cfg(target_arch = "x86_64")]
    pub fn frequency() -> u64 {
        // Approximate TSC frequency - would be calibrated at boot
        2_500_000_000 // 2.5 GHz typical
    }
}

/// Common AI acceleration error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiAccelError {
    /// Hardware not available
    HardwareUnavailable,
    /// Operation not supported on this platform
    Unsupported,
    /// Resource exhausted
    OutOfResources,
    /// Invalid parameters
    InvalidRequest,
    /// Hardware fault
    HardwareFault,
}

/// Result type for AI acceleration operations
pub type AiResult<T> = Result<T, AiAccelError>;

// Architecture-specific compile-time validation
#[cfg(all(target_arch = "x86_64", feature = "neural-engine"))]
compile_error!("Neural Engine support is only available on ARM64 (Apple Silicon)");

#[cfg(all(target_arch = "aarch64", feature = "avx-fallback"))]
compile_error!("AVX fallback is only available on x86_64 architecture");