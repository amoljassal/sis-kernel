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
pub mod irqvec;
pub mod interrupts;
pub mod cpu;
pub mod context_switch;
#[cfg(feature = "apic")]
pub mod apic;
#[cfg(feature = "smp")]
pub mod smp;
#[cfg(feature = "idt-selftest")]
pub mod idt_selftest;
#[cfg(feature = "pf-matrix")]
pub mod pf_matrix;
#[cfg(feature = "per-task-mm")]
pub mod as_isolation;
#[cfg(feature = "ipc")]
pub mod ipc_selftest;
#[cfg(feature = "scheduler")]
pub mod scheduler_selftest;
#[cfg(feature = "iommu")]
pub mod iommu;