#![allow(dead_code)]

#[cfg(feature = "smp")]
pub fn online_cpus() -> alloc::vec::Vec<u32> {
    extern crate alloc;
    // For CI, assume 2 CPUs when smp enabled; real impl can probe APIC IDs
    let mut v = alloc::vec::Vec::new();
    v.push(0);
    v.push(1);
    v
}

#[cfg(not(feature = "smp"))]
pub fn online_cpus() -> [u32; 1] {
    [0]
}

// Phase 6D: Additional topology functions
pub fn cpu_index_this() -> usize {
    // Simplified: return 0 for BSP, real impl would read from per-CPU data
    0
}

pub fn cpu_index_from_apic(apic_id: u32) -> usize {
    // Simplified mapping: APIC ID == CPU index for first few CPUs
    apic_id as usize
}

pub fn apic_from_cpu_index(cpu_idx: usize) -> u32 {
    // Simplified mapping: CPU index == APIC ID for first few CPUs
    cpu_idx as u32
}
