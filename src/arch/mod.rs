//! Architecture abstraction layer.
//!
//! This module re‑exports architecture specific code.  Currently
//! only x86_64 is supported, but the structure allows adding other
//! architectures in the future.

pub mod x86_64;