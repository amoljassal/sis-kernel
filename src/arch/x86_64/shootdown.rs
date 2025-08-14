//! Phase 6D: Cross‑CPU TLB shootdown with ACK bitmask + timeout.
#![cfg(feature = "smp")]

use core::sync::atomic::{AtomicU64, Ordering};
use crate::arch::x86_64::smp::ipi::send_tlb_ipi;
use crate::arch::x86_64::topology;
use crate::kernel::serial;

static PENDING_MASK: AtomicU64 = AtomicU64::new(0);
static ACK_MASK: AtomicU64 = AtomicU64::new(0);
// Range for local worker (single slot for simplicity)
static mut PENDING_VA: usize = 0;
static mut PENDING_LEN: usize = 0;

/// Installed on remote CPUs by the IPI_TLB handler.
pub fn ack_this_cpu() {
    let me = topology::cpu_index_this() as u64;
    let bit = 1u64 << me;
    let _ = ACK_MASK.fetch_or(bit, Ordering::Release);
}

/// Controller: set pending range for all CPUs in `mask`, send IPIs, wait for ACK or timeout.
pub fn invalidate_range(mask: u64, vaddr: usize, len: usize, timeout_ticks: u64) -> Result<(), &'static str> {
    if mask == 0 { return Ok(()); }
    unsafe {
        PENDING_VA = vaddr;
        PENDING_LEN = len;
    }
    PENDING_MASK.store(mask, Ordering::Release);
    ACK_MASK.store(0, Ordering::Relaxed);

    // fan‑out
    for apic in topology::online_cpus() {
        let idx = topology::cpu_index_from_apic(apic) as u64;
        if (mask & (1u64 << idx)) != 0 {
            send_tlb_ipi(apic);
        }
    }

    // wait
    let mut waited = 0u64;
    while waited < timeout_ticks {
        let a = ACK_MASK.load(Ordering::Acquire);
        if a & mask == mask {
            return Ok(());
        }
        // Simple delay loop instead of time module
        for _ in 0..1000 { core::hint::spin_loop(); }
        waited += 1;
    }
    serial::write_str("[tlb] shootdown timeout mask=0x"); serial::write_hex64(mask); serial::write_str(" ack=0x"); serial::write_hex64(ACK_MASK.load(Ordering::Relaxed)); serial::write_str("\n");
    Err("shootdown timeout")
}

/// Local worker invoked by IPI_TLB handler.
pub fn apply_pending_local() {
    use x86_64::instructions::tlb::flush;
    // For simplicity: invalidate by page stepping
    let (start, len) = unsafe { (PENDING_VA, PENDING_LEN) };
    if len == 0 { return; }
    let end = start + len;
    let mut addr = start;
    while addr < end {
        unsafe { x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(addr as u64)); }
        addr += 4096;
    }
    // also serialize
    unsafe { core::arch::asm!("mfence", "sfence", "lfence", options(nostack, preserves_flags)); }
}