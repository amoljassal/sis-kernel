//! Timer-driven preemption glue for the simple scheduler.
#![cfg(all(feature="scheduler"))]
use core::sync::atomic::Ordering;
use crate::arch::x86_64::percpu_clean::PerCpu;
use crate::kernel::serial;

/// Default RR quantum in ticks (keep it tiny for selftests; tune later)
pub const RR_QUANTUM_TICKS: u32 = 5;

/// Called from the timer ISR on every tick (LAPIC/PIT).
#[inline]
pub fn on_timer_tick() {
    let pc = PerCpu::this();
    let _ = pc.ticks.fetch_add(1, Ordering::Relaxed);
    let t = pc.rr_ticks.fetch_add(1, Ordering::Relaxed) + 1;

    // Either a remote wake poked us, or our quantum expired.
    let need = pc.need_resched.swap(false, Ordering::AcqRel)
           || t >= RR_QUANTUM_TICKS;

    if need {
        // Reset quantum and request a schedule at the next safe point.
        pc.rr_ticks.store(0, Ordering::Relaxed);
        // Hand off to the scheduler fast path.
        super::simple_scheduler::request_reschedule();
    }
}

/// Optional: tiny debug aid (rate-limited).
#[allow(dead_code)]
pub fn log_tick_marker() {
    let pc = PerCpu::this();
    let tk = pc.ticks.load(Ordering::Relaxed);
    if (tk & 0x3f) == 0 {
        serial::write_str("[tick] cpu="); serial::write_dec(pc.cpu_id as u64);
        serial::write_str(" ticks="); serial::write_dec(tk as u64);
        serial::write_str("\n");
    }
}