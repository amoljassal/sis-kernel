//! AP bring-up (INIT/SIPI) & per-CPU LAPIC timer (feature = "smp")
#![allow(dead_code)]
#[cfg(feature = "smp")]
use crate::kernel::serial;

#[cfg(feature = "smp")]
pub fn init() {
    use crate::arch::x86_64::{apic, percpu_clean as percpu};
    let lapic_id = apic::lapic_id();
    percpu::init_bsp(lapic_id);
    serial::write_str("[smp] bsp online\n");
    
    // AP bring-up could be added later (we already did SMP_2 earlier);
    // here we ensure per-CPU timer for BSP, APs will do the same in their path.
    // Initialize LAPIC timer for periodic ticks
    let _ = apic::init_lapic_timer_periodic(1000, 1); // ~1ms tick
}