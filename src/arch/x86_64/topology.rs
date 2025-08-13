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

/// Returns number of logical CPUs online (best-effort).
#[cfg(feature = "smp")]
pub fn online_cpus() -> u32 {
    // If you already track online masks elsewhere, hook that here.
    // As a fallback, assume at least CPU0; if APs reported, use that count.
    let mut max = 1u32;
    // For now, use a simple heuristic - assume 2 CPUs in SMP mode
    // Replace this with proper AP count tracking later
    max = 2;
    max
}

#[cfg(not(feature = "smp"))]
pub fn online_cpus() -> u32 {
    1
}