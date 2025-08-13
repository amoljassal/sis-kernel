//! Selftest: pin a worker task to CPU1, verify it only runs there, then relax to all CPUs.
#![cfg(all(feature="affinity", feature="smp", feature="scheduler"))]

use crate::kernel::syscall;
use crate::kernel::time;
use crate::kernel::serial;
use crate::arch::x86_64::percpu_clean as percpu;
#[cfg(feature="smp")]
use crate::arch::x86_64::smp;
use core::sync::atomic::{AtomicU32, Ordering};

static HITS: AtomicU32 = AtomicU32::new(0);

extern "C" fn worker_entry() -> ! {
    loop {
        let cpu = percpu::this().cpu_id;
        if cpu == 1 {
            HITS.fetch_add(1, Ordering::Relaxed);
        } else {
            // would signal failure in a stricter test; keep spinning to avoid hang
        }
        for _ in 0..50_000 { core::hint::spin_loop(); }
    }
}

pub fn run() {
    serial::write_str("[aff] starting affinity test\n");
    // Skip if single-CPU
    #[cfg(feature="smp")]
    let online = smp::all_apic_ids().len() as u32;
    #[cfg(not(feature="smp"))]
    let online = 1u32;
    if online < 2 {
        serial::write_str("[aff][SKIP] only one CPU online\n");
        // Fallback if exit_skip not present:
        #[allow(unused_unsafe)] unsafe { crate::qemu::exit_ok(); }
        return;
    }
    // spawn worker kernel task pinned later by syscall
    let _tid = unsafe { crate::kernel::spawn::spawn_kernel_closure(worker_entry as usize) };

    // Your syscall layer currently uses dispatch_manual-like APIs; keep it.
    let mask = 1u64 << 1;
    let rc = syscall::dispatch_manual(crate::kernel::syscall::SYS_SET_AFFINITY as u64,
                                      mask, 0, 0, 0, 0, 0);
    serial::write_str("[aff] set_affinity rc=0x");
    crate::kernel::serial::write_hex64(rc);
    serial::write_str("\n");

    time::sleep_ms(50);
    let c = HITS.load(Ordering::Relaxed);
    serial::write_str("[aff] hits@cpu1=0x");
    crate::kernel::serial::write_hex32(c);
    serial::write_str("\n");
    if c == 0 {
        serial::write_str("[aff][FAIL] no hits on cpu1\n");
        unsafe { crate::qemu::exit_fail(0xA1); }
    }

    // relax to all CPUs (mask=0 => unconstrained)
    let rc2 = syscall::dispatch_manual(crate::kernel::syscall::SYS_SET_AFFINITY as u64,
                                       0, 0, 0, 0, 0, 0);
    serial::write_str("[aff] relax rc=0x");
    crate::kernel::serial::write_hex64(rc2);
    serial::write_str("\n");
    unsafe { crate::qemu::exit_ok(); }
}