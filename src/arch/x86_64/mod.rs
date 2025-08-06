//! x86_64 specific modules.
//!
//! This module contains architecture‑specific code used by the SIS
//! kernel.  It is separated from the core kernel logic to make
//! porting to other architectures possible in the future.

pub mod io;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod pit;
pub mod interrupts;
pub mod cpu;
pub mod context_switch;