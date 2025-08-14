//! Memory management with TLB shootdown support
#![cfg(feature = "smp")]

use crate::arch::x86_64::shootdown;
use crate::arch::x86_64::topology;

/// Unmap in a target task with cross‑CPU shootdown to maintain coherency.
#[cfg(feature = "smp")]
pub fn unmap_with_shootdown(target_cpus_mask: u64, vaddr: usize, len: usize) -> Result<(), &'static str> {
    // do page table edits locally (holding appropriate locks)
    // ... existing unmap implementation ...
    // then shootdown
    let mask = if target_cpus_mask == 0 {
        // broadcast to all online except self
        let mut m = 0u64;
        for apic in topology::online_cpus() {
            let idx = topology::cpu_index_from_apic(apic) as u64;
            if idx != (topology::cpu_index_this() as u64) {
                m |= 1u64 << idx;
            }
        }
        m
    } else {
        target_cpus_mask
    };
    shootdown::invalidate_range(mask, vaddr, len, /*timeout_ticks*/ 10_000)?;
    Ok(())
}