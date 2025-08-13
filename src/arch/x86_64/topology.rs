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
