//! x86_64 specific modules.
//!
//! This module contains architecture‑specific code used by the SIS
//! kernel.  It is separated from the core kernel logic to make
//! porting to other architectures possible in the future.

#[cfg(feature = "apic")]
pub mod apic;
pub mod context_switch;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io;
#[cfg(feature = "smp")]
pub mod ipi;
pub mod irqvec;
pub mod memory;
#[cfg(feature = "smp")]
pub mod percpu;
#[cfg(feature = "smp")]
pub mod percpu_clean;
pub mod pit;
#[cfg(feature = "smp")]
pub mod shootdown;
#[cfg(feature = "smp")]
pub mod smp;
#[cfg(feature = "smp")]
pub mod topology;

// When smp is off, provide minimal no-op exports so cfg paths compile.
#[cfg(not(feature = "smp"))]
pub mod topology {
    #[inline]
    pub fn online_cpus() -> &'static [u32] {
        &[]
    }
    #[inline]
    pub fn cpu_index_from_apic(_apic_id: u32) -> Option<usize> {
        None
    }
}
#[cfg(feature = "per-task-mm")]
pub mod as_isolation;
#[cfg(feature = "idt-selftest")]
pub mod idt_selftest;
#[cfg(feature = "iommu")]
pub mod iommu;
#[cfg(feature = "ipc")]
pub mod ipc_selftest;
#[cfg(feature = "pf-matrix")]
pub mod pf_matrix;
#[cfg(feature = "scheduler")]
pub mod scheduler_selftest;
