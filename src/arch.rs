//! Architecture path-switch shim
//! 
//! Based on ChatGPT's recommendation: This module ensures only one architecture
//! tree is compiled based on the target, preventing x86_64 code from being
//! compiled on ARM64 and vice versa.

#[cfg_attr(target_arch = "x86_64", path = "arch/x86_64/mod.rs")]
#[cfg_attr(target_arch = "aarch64", path = "arch/aarch64/mod.rs")]
pub mod arch_impl;

// Re-export architecture implementation
pub use arch_impl::*;