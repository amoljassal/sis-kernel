//! Per-CPU data & helpers (feature = "smp")
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(feature = "smp")]
use x86_64::registers::model_specific::Msr;

#[repr(C, align(64))]
pub struct PerCpu {
    pub cpu_id: u8,
    pub lapic_id: u32,
    pub ticks: AtomicU64,
    pub need_resched: AtomicBool,
    /// Quantum ticks consumed on this CPU (reset on preempt)
    pub rr_ticks: core::sync::atomic::AtomicU32,
    /// voluntary + preempt context switches
    pub ctx_sw: AtomicU64,
    /// received IPI counters (telemetry)
    pub ipi_rx_resched: AtomicU64,
    pub ipi_rx_tlb: AtomicU64,
    /// mailbox receive count (Phase 6C)
    pub mbox_rx: AtomicU64,
}

impl PerCpu {
    pub const fn new() -> Self {
        PerCpu {
            cpu_id: 0,
            lapic_id: 0,
            ticks: AtomicU64::new(0),
            need_resched: AtomicBool::new(false),
            rr_ticks: core::sync::atomic::AtomicU32::new(0),
            ctx_sw: AtomicU64::new(0),
            ipi_rx_resched: AtomicU64::new(0),
            ipi_rx_tlb: AtomicU64::new(0),
            mbox_rx: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn this<'a>() -> &'a PerCpu {
        unsafe { &*gs_base_ptr() }
    }

    #[inline(always)]
    pub fn bump_ctx_sw(&self) {
        let _ = self.ctx_sw.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "smp")]
#[inline(always)]
fn gs_base_ptr() -> *const PerCpu {
    let base = unsafe { Msr::new(0xC0000101).read() } as *const PerCpu;
    base
}

#[cfg(feature = "smp")]
static mut BSP_PCPU: PerCpu = PerCpu::new();

#[cfg(feature = "smp")]
#[inline(always)]
pub fn init_bsp(lapic_id: u32) {
    unsafe {
        BSP_PCPU.cpu_id = 0;
        BSP_PCPU.lapic_id = lapic_id;
        // Point GS base to BSP per-cpu
        let base = &BSP_PCPU as *const _ as u64;
        unsafe {
            Msr::new(0xC0000101).write(base);
        } // IA32_GS_BASE
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
    static PCPU: PerCpu = PerCpu::new();
    &PCPU
}
