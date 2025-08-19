//! Architecture abstraction layer with HAL (Hardware Abstraction Layer)
//!
//! This module provides a unified interface for both x86_64 and ARM64 architectures
//! supporting AI-native kernel operations across different hardware platforms.

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// HAL (Hardware Abstraction Layer) trait definitions
pub mod hal;

/// Unified AI acceleration interface across architectures
pub mod ai;

// Re-export architecture implementation based on target
#[cfg(target_arch = "x86_64")]
pub use x86_64 as arch_impl;

#[cfg(target_arch = "aarch64")]
pub use aarch64 as arch_impl;

// Common architecture abstractions - re-export from arch_impl
#[cfg(target_arch = "x86_64")]
pub use x86_64::{cpu, io, memory, interrupts, gdt, idt, irqvec};

#[cfg(target_arch = "aarch64")]
pub use aarch64::{cpu, io, memory, interrupts};

// Error type for architecture operations
#[derive(Debug)]
pub enum ArchError {
    InitializationFailed(&'static str),
    UnsupportedOperation,
    HardwareError(&'static str),
}

impl From<&'static str> for ArchError {
    fn from(msg: &'static str) -> Self {
        ArchError::InitializationFailed(msg)
    }
}

/// Initialize architecture-specific components
pub fn init() -> Result<(), ArchError> {
    arch_impl::init()
}