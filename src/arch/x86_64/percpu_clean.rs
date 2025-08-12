//! Per-CPU data & helpers (feature = "smp")
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(feature = "smp")]
use x86_64::registers::model_specific::Msr;

#[repr(align(64))]
#[derive(Default)]
pub struct PerCpu {
    pub cpu_id: u32,
    pub lapic_id: u32,
    pub ticks: AtomicU64,
    pub need_resched: AtomicBool,
    pub scratch: u64,
}

#[cfg(feature = "smp")]
static mut BSP_PCPU: PerCpu = PerCpu {
    cpu_id: 0,
    lapic_id: 0,
    ticks: AtomicU64::new(0),
    need_resched: AtomicBool::new(false),
    scratch: 0,
};

#[cfg(feature = "smp")]
#[inline(always)]
pub fn init_bsp(lapic_id: u32) {
    unsafe {
        BSP_PCPU.cpu_id = 0;
        BSP_PCPU.lapic_id = lapic_id;
        // Point GS base to BSP per-cpu
        let base = &BSP_PCPU as *const _ as u64;
        unsafe { Msr::new(0xC0000101).write(base); } // IA32_GS_BASE
    }
}

#[cfg(feature = "smp")]
#[inline(always)]
pub fn this() -> &'static PerCpu {
    // SAFETY: IA32_GS_BASE is set per CPU to its PerCpu
    let base = unsafe { Msr::new(0xC0000101).read() } as *const PerCpu; // IA32_GS_BASE
    unsafe { &*base }
}

#[cfg(not(feature = "smp"))]
pub fn this() -> &'static PerCpu {
    // Minimal shim for non-smp builds: a single static
    static PCPU: PerCpu = PerCpu {
        cpu_id: 0, lapic_id: 0,
        ticks: AtomicU64::new(0),
        need_resched: AtomicBool::new(false),
        scratch: 0,
    };
    &PCPU
}