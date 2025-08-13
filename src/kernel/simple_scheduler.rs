//! Simple Per-CPU Scheduler (Phase 6B Patch Compatible)
//!
//! This module provides the simple scheduler interface expected by the Phase 6B patch.
//! It maintains per-CPU runqueues and basic task management.

#![cfg(feature = "scheduler")]

use alloc::collections::VecDeque;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering, AtomicU8};
use crate::arch::x86_64::percpu_clean as percpu;
#[cfg(feature="smp")]
use crate::arch::x86_64::smp::ipi;
use crate::kernel::task::{Task, State as TaskState};

static PREEMPT: AtomicBool = AtomicBool::new(true);

#[cfg(feature="scheduler")]
#[inline(always)]
fn is_cpu_allowed(mask: u64, cpu: u32) -> bool {
    if mask == 0 { return true; } // 0 => unconstrained
    let bit = 1u64 << cpu;
    (mask & bit) != 0
}

pub struct RunQueue { pub q: VecDeque<usize> } // task ids (usize)
impl RunQueue {
    pub fn new() -> Self { Self { q: VecDeque::new() } }
    pub fn push(&mut self, t: usize) { self.q.push_back(t); }
    pub fn pop(&mut self) -> Option<usize> { self.q.pop_front() }
    pub fn is_empty(&self) -> bool { self.q.is_empty() }
}

// Per-CPU runqueues; lazily initialized on first use to avoid adding new boot hooks
static mut RUNQS: MaybeUninit<[RunQueue; 64]> = MaybeUninit::uninit();
static RUNQS_INIT: AtomicU8 = AtomicU8::new(0);

#[inline(always)]
fn ensure_runqs() {
    if RUNQS_INIT.load(Ordering::Acquire) != 0 { return; }
    if RUNQS_INIT
        .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        let arr = core::array::from_fn(|_| RunQueue::new());
        unsafe { RUNQS.write(arr); }
        RUNQS_INIT.store(2, Ordering::Release);
        return;
    }
    // wait for other core
    while RUNQS_INIT.load(Ordering::Acquire) != 2 { core::hint::spin_loop() }
}

#[inline(always)]
fn runq_mut(cpu: usize) -> &'static mut RunQueue {
    ensure_runqs();
    unsafe { &mut (*RUNQS.as_mut_ptr())[cpu] }
}

pub fn enqueue_task(t: &Task) {
    // Prefer: last CPU (hint) if allowed by affinity; else: first allowed CPU; else fallback 0
    let this = unsafe { percpu::this().cpu_id };
    let mut target = this as usize;
    
    #[cfg(feature="affinity")]
    {
        if !is_cpu_allowed(t.cpu_affinity_mask, this) {
            // pick the lowest-indexed allowed CPU
            let mut found = None;
            for c in 0..64u32 {
                if is_cpu_allowed(t.cpu_affinity_mask, c) {
                    found = Some(c as usize);
                    break;
                }
            }
            target = found.unwrap_or(0);
        } else {
            target = this as usize;
        }
    }
    
    #[cfg(not(feature="affinity"))]
    {
        // prefer this CPU, else fall back to hint
        target = this as usize;
    }
    
    // Your Task uses `id: usize` instead of `tid: u64`
    runq_mut(target).push(t.id);
    
    // if placed on a remote CPU, poke it
    let cur = unsafe { percpu::this().cpu_id } as usize;
    if target != cur {
        #[cfg(feature="smp")]
        unsafe { ipi::send_resched(target as u32); }
    }
}

pub fn pick_next() -> Option<usize> {
    let cpu = unsafe { percpu::this().cpu_id as usize };
    if let Some(t) = runq_mut(cpu).pop() { return Some(t); }
    // simple steal
    for i in 0..64usize {
        if i == cpu { continue; }
        if let Some(t) = runq_mut(i).pop() { return Some(t); }
    }
    None
}

pub fn set_need_resched() { 
    PREEMPT.store(true, Ordering::SeqCst); 
}

#[cfg(feature="affinity")]
pub fn migrate_if_violates_affinity(task: &Task) {
    let cur = unsafe { percpu::this().cpu_id };
    if !is_cpu_allowed(task.cpu_affinity_mask, cur) {
        enqueue_task(task);
        // yield current
        set_need_resched();
    }
}