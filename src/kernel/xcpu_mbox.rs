//! Cross-CPU mailbox (Phase 6C)
//! Lock-free per-CPU ring buffers for tiny u64 messages.
#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

#[cfg(feature = "smp")]
use crate::kernel::serial;
#[cfg(feature = "smp")]
use crate::arch::x86_64::percpu_clean as percpu;
#[cfg(feature = "smp")]
use crate::arch::x86_64::smp::ipi;

// Conservative max CPU count (align with your percpu/topology)
pub const MAX_CPUS: usize = 64;
const RING_LEN: usize = 256; // power-of-two
const RING_MASK: usize = RING_LEN - 1;

#[repr(align(64))]
struct Aligned<T>(T);

#[repr(C)]
struct Ring {
    head: AtomicUsize,
    tail: AtomicUsize,
    buf: UnsafeCell<[u64; RING_LEN]>,
}

unsafe impl Sync for Ring {}

impl Ring {
    const fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            buf: UnsafeCell::new([0; RING_LEN]),
        }
    }
    #[inline]
    fn try_push(&self, v: u64) -> Result<(), ()> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) == RING_LEN {
            return Err(()); // full
        }
        // SAFE: single writer per logical push; index masked
        unsafe {
            (*self.buf.get())[head & RING_MASK] = v;
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }
    #[inline]
    fn try_pop(&self) -> Option<u64> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        // SAFE: single consumer per CPU; index masked
        let v = unsafe { (*self.buf.get())[tail & RING_MASK] };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(v)
    }
}

#[repr(align(64))]
struct Mailbox {
    ring: Ring,
}

impl Mailbox {
    const fn new() -> Self {
        Self { ring: Ring::new() }
    }
}

#[cfg(feature = "smp")]
static mut MBOXES: [core::mem::MaybeUninit<Aligned<Mailbox>>; MAX_CPUS] = 
    unsafe { core::mem::MaybeUninit::uninit().assume_init() };

#[cfg(feature = "smp")]
static MBOXES_INIT: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

#[cfg(feature = "smp")]
#[inline]
fn mb_for_cpu(cpu: usize) -> &'static Mailbox {
    use core::sync::atomic::Ordering;
    
    // Initialize mailboxes on first use
    if !MBOXES_INIT.load(Ordering::Acquire) {
        // Use compare_exchange to ensure only one thread initializes
        if MBOXES_INIT.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            unsafe {
                for i in 0..MAX_CPUS {
                    MBOXES[i] = core::mem::MaybeUninit::new(Aligned(Mailbox::new()));
                }
            }
        }
    }
    
    // SAFE: array is initialized; cpu index validated by caller
    unsafe { &MBOXES.get_unchecked(cpu).assume_init_ref().0 }
}

/// Enqueue a message to target APIC id and poke with IPI.
#[cfg(feature = "smp")]
pub fn send(apic_id: u32, msg: u64) -> Result<(), i64> {
    let target_cpu = crate::arch::x86_64::topology::cpu_index_from_apic(apic_id)
        .ok_or(-22i64)?;
    let mb = mb_for_cpu(target_cpu);
    mb.ring.try_push(msg).map_err(|_| -11i64)?;
    ipi::send_mailbox_ipi(apic_id);
    Ok(())
}

/// Non-blocking receive on the current CPU.
#[cfg(feature = "smp")]
pub fn try_recv() -> Option<u64> {
    let cpu = percpu::this().cpu_id as usize;
    let mb = mb_for_cpu(cpu);
    mb.ring.try_pop()
}

/// Drain and count messages; used by IPI handler or polling.
#[cfg(feature = "smp")]
pub fn drain(max: usize) -> usize {
    let mut n = 0usize;
    while n < max {
        if try_recv().is_some() { n += 1; } else { break; }
    }
    n
}