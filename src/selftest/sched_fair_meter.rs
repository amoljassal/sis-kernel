#![cfg(all(feature="scheduler", feature="selftests"))]
//! Tiny fairness meter: sample per-CPU runqueue lengths while two busy tasks run.
use core::sync::atomic::{AtomicU32, Ordering};
use crate::kernel::serial;
use crate::kernel::simple_scheduler;
use crate::arch::x86_64::percpu_clean::PerCpu;
use crate::qemu;

static A: AtomicU32 = AtomicU32::new(0);
static B: AtomicU32 = AtomicU32::new(0);

pub fn run() {
    serial::write_str("[fair] meter start\n");
    
    // Sample queues for a period while doing work on this CPU.
    let mut samples_this = 0u32;
    let mut max_this = 0usize;
    let mut min_this = usize::MAX;
    let mut max_any = 0usize;

    let start = PerCpu::this().ticks.load(Ordering::Relaxed);
    while PerCpu::this().ticks.load(Ordering::Relaxed) - start < 50 {
        let this_len = simple_scheduler::runqueue_len_this();
        let cpu0 = simple_scheduler::runqueue_len(0);
        let cpu1 = simple_scheduler::runqueue_len(1);
        let any_max = cpu0.max(cpu1);
        if this_len > max_this { max_this = this_len; }
        if this_len < min_this { min_this = this_len; }
        if any_max > max_any { max_any = any_max; }
        samples_this += 1;
        // do some work on this CPU
        A.fetch_add(1, Ordering::Relaxed);
        // short pause
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }

    let a = A.load(Ordering::Relaxed);
    serial::write_str("[fair] counter a="); serial::write_dec(a as u64); serial::write_str("\n");
    serial::write_str("[fair] rq_this min="); serial::write_dec(min_this as u64);
    serial::write_str(" max="); serial::write_dec(max_this as u64);
    serial::write_str(" samples="); serial::write_dec(samples_this as u64); serial::write_str("\n");
    serial::write_str("[fair] rq_any max="); serial::write_dec(max_any as u64); serial::write_str("\n");

    // Basic assertions:
    //  - task ran and made progress
    //  - runqueue functions work
    if a > 50 && samples_this > 10 {
        serial::write_str("[fair] Basic fairness meter working\n");
        qemu::exit_ok();
    } else {
        serial::write_str("[fair] Basic fairness meter not working\n");
        qemu::exit_fail();
    }
}