#![allow(dead_code)]
extern crate alloc;
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
pub fn online_cpu_count() -> u32 {
    // If you already track online masks elsewhere, hook that here.
    // As a fallback, assume at least CPU0; if APs reported, use that count.
    let mut max = 1u32;
    // For now, use a simple heuristic - assume 2 CPUs in SMP mode
    // Replace this with proper AP count tracking later
    max = 2;
    max
}

#[cfg(not(feature = "smp"))]
pub fn online_cpu_count() -> u32 {
    1
}

/// Returns collection of online APIC IDs for iteration.
#[cfg(feature = "smp")]
pub fn online_cpus() -> &'static [u32] {
    // Return APIC IDs 0 and 1 for 2-CPU SMP
    &[0, 1]
}

#[cfg(not(feature = "smp"))]
pub fn online_cpus() -> &'static [u32] {
    &[0]
}

/// Convert APIC ID to CPU index (0-based).
#[cfg(feature = "smp")]
pub fn cpu_index_from_apic(apic_id: u32) -> Option<usize> {
    // Simple 1:1 mapping for now
    if apic_id < 64 { Some(apic_id as usize) } else { None }
}

#[cfg(not(feature = "smp"))]
pub fn cpu_index_from_apic(_apic_id: u32) -> Option<usize> {
    Some(0)
}