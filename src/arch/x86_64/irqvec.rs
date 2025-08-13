//! Per-CPU MSI vector allocator for 0x50..=0x5F.
//! Simple bitmap per CPU; lock-free with Atomics.

use core::sync::atomic::{AtomicU16, Ordering};
use spin::Once;

const BASE: u8 = 0x50;
const COUNT: usize = 16; // 0x50..=0x5F
const MAX_CPUS: usize = 8; // conservative cap; adjust if your cpu_count() > 8

static INIT: Once<()> = Once::new();
static mut BITMAPS: [AtomicU16; MAX_CPUS] = [const { AtomicU16::new(0) }; MAX_CPUS];

fn ensure_init() {
    INIT.call_once(|| {
        // Nothing dynamic to do; placeholder for parity with other init flows.
    });
}

#[inline]
pub fn range() -> core::ops::RangeInclusive<u8> {
    BASE..=BASE + (COUNT as u8 - 1)
}

#[inline]
fn bit_for(vec: u8) -> u16 {
    1u16 << (vec - BASE)
}

#[inline]
fn idx_for_cpu(cpu: usize) -> usize {
    // Clamp to MAX_CPUS-1 if cpu_count() exceeds cap.
    if cpu < MAX_CPUS {
        cpu
    } else {
        MAX_CPUS - 1
    }
}

/// Allocate a free vector on this CPU. Returns vector or None.
pub fn alloc_vector(cpu: usize) -> Option<u8> {
    ensure_init();
    let i = idx_for_cpu(cpu);
    let m = unsafe { &BITMAPS[i] };
    loop {
        let cur = m.load(Ordering::Relaxed);
        if cur == 0xFFFF {
            return None;
        } // full
          // find first zero bit within COUNT
        let mut free = None;
        for b in 0..COUNT {
            if (cur & (1u16 << b)) == 0 {
                free = Some(b);
                break;
            }
        }
        let b = free?;
        let new = cur | (1u16 << b);
        if m.compare_exchange(cur, new, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return Some(BASE + b as u8);
        }
    }
}

/// Free a previously allocated vector on this CPU.
pub fn free_vector(cpu: usize, vec: u8) {
    ensure_init();
    if !range().contains(&vec) {
        return;
    }
    let i = idx_for_cpu(cpu);
    let m = unsafe { &BITMAPS[i] };
    loop {
        let cur = m.load(Ordering::Relaxed);
        let new = cur & !bit_for(vec);
        if m.compare_exchange(cur, new, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
    }
}
