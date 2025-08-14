//! Simple Per-CPU Scheduler (Phase 6D Ready Queue)
//!
//! This module provides the Phase 6D ready queue implementation with
//! affinity-aware scheduling, resched IPIs, and per-CPU stats.

#![cfg(feature = "scheduler")]

// Runqueue backing storage uses a lazy static slot; hush static-mut refs warnings here only.
#![allow(static_mut_refs)]

use core::sync::atomic::{AtomicU8, Ordering};
use alloc::collections::VecDeque;
use crate::arch::x86_64::percpu_clean::PerCpu;
use crate::kernel::serial;
use crate::kernel::task::Task;
use core::cell::UnsafeCell;
#[cfg(feature = "affinity")]
use crate::arch::x86_64::smp::ipi::send_resched_ipi;
#[cfg(feature = "affinity")]
use crate::arch::x86_64::topology;

// Lazy init runqueues (compat shim)
static INIT: AtomicU8 = AtomicU8::new(0);
static mut RUNQS: Option<[RunQueue; 64]> = None;

pub struct RunQueue {
    pub q: VecDeque<usize>, // task ids
}

impl RunQueue {
    pub fn empty() -> Self { Self { q: VecDeque::new() } }
}

pub fn ensure_init() {
    if INIT.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_ok() {
        unsafe {
            RUNQS = Some(core::array::from_fn(|_| RunQueue::empty()));
        }
        INIT.store(2, Ordering::Release);
    } else {
        while INIT.load(Ordering::Acquire) != 2 { core::hint::spin_loop(); }
    }
}

/// Called by timer/interrupt to select next runnable respecting affinity.
#[cfg(feature = "affinity")]
pub fn pick_next() -> Option<usize> {
    ensure_init();
    let me = PerCpu::this().cpu_id as usize;
    let rqs = unsafe { RUNQS.as_mut().unwrap() };
    // Prefer local runqueue
    if let Some(tid) = rqs[me].q.pop_front() { return Some(tid); }
    // Affinity‑aware fallback: nearest online cpu
    for apic in topology::online_cpus() {
        let idx = topology::cpu_index_from_apic(apic) as usize;
        if idx == me { continue; }
        if let Some(tid) = rqs[idx].q.pop_front() { return Some(tid); }
    }
    None
}

/// Wake a task on a target CPU, enqueue and poke with RESCHED IPI.
#[cfg(feature = "affinity")]
pub fn task_wakeup(tid: usize, target_cpu: usize) {
    ensure_init();
    let rqs = unsafe { RUNQS.as_mut().unwrap() };
    if target_cpu < rqs.len() {
        rqs[target_cpu].q.push_back(tid);
        // notify
        let apic = topology::apic_from_cpu_index(target_cpu);
        send_resched_ipi(apic);
    }
}

/// Enqueue respecting a Task's cpu_affinity_mask if present, otherwise local.
#[cfg(feature = "affinity")]
pub fn enqueue_task_affinity(t: &Task) {
    ensure_init();
    let me = PerCpu::this().cpu_id as usize;
    let rqs = unsafe { RUNQS.as_mut().unwrap() };
    let mut placed = false;
    let mask = t.cpu_affinity_mask;
    if mask != 0 {
        let mut idx = 0usize;
        while idx < rqs.len() {
            let bit = 1u64 << idx;
            if (mask & bit) != 0 {
                rqs[idx].q.push_back(t.id);
                placed = true;
                // poke if remote
                if idx != me {
                    let apic = topology::apic_from_cpu_index(idx);
                    send_resched_ipi(apic);
                }
                break;
            }
            idx += 1;
        }
    }
    if !placed {
        rqs[me].q.push_back(t.id);
    }
}

/// Called on context switch to update stats.
pub fn on_context_switch() {
    PerCpu::this().bump_ctx_sw();
}

/// Legacy compatibility function for existing code
pub fn enqueue_task(t: &Task) {
    #[cfg(feature = "affinity")]
    enqueue_task_affinity(t);
    
    #[cfg(not(feature = "affinity"))]
    {
        ensure_init();
        let me = PerCpu::this().cpu_id as usize;
        let rqs = unsafe { RUNQS.as_mut().unwrap() };
        rqs[me].q.push_back(t.id);
    }
}

/// Set need_resched flag for current CPU
pub fn set_need_resched() {
    PerCpu::this().need_resched.store(true, Ordering::Release);
}

/// Lightweight "please schedule soon" nudge used by timer glue.
#[inline]
pub fn request_reschedule() {
    // In the compat shim we just set need_resched again; the actual
    // reschedule will occur when we return from the interrupt to the
    // scheduler's epilogue (or next tick).
    PerCpu::this().need_resched.store(true, core::sync::atomic::Ordering::Release);
}

/// Return the run-queue length for the given CPU (best-effort, read-only).
#[cfg(feature="scheduler")]
pub fn runqueue_len(cpu_idx: usize) -> usize {
    ensure_init();
    let rqs = unsafe { RUNQS.as_ref().unwrap() };
    if cpu_idx >= rqs.len() { return 0; }
    rqs[cpu_idx].q.len()
}

/// Convenience: run-queue length on the current CPU.
#[cfg(feature="scheduler")]
pub fn runqueue_len_this() -> usize {
    runqueue_len(PerCpu::this().cpu_id as usize)
}