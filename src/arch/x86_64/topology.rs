#![allow(dead_code)]
#[cfg(feature = "smp")]
pub fn apic_ids_from_mask(mask: u32) -> impl Iterator<Item=u32> {
    // Minimal stub: assume LAPIC IDs are [0..31] mapping to bit positions.
    // Replace with MADT parsing later.
    (0..32).filter(move |id| (mask & (1u32 << id)) != 0).map(|id| id)
}

#[cfg(not(feature = "smp"))]
pub fn apic_ids_from_mask(_mask: u32) -> impl Iterator<Item=u32> {
    // Non-SMP stub: empty iterator
    core::iter::empty()
}