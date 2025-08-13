//! Inter-Processor Interrupt (IPI) support for SMP scheduling
//!
//! This module provides IPI infrastructure for Phase 6B CPU affinity and
//! Phase 6C cross-CPU mailbox support.

use core::sync::atomic::{AtomicU32, Ordering};
use crate::kernel::serial;
use crate::arch::x86_64::percpu_clean as percpu;
use crate::arch::x86_64::apic;

/// IPI vectors
pub const IPI_RESCHED: u8 = 0xF0_u8;
pub const IPI_TLB: u8     = 0xF1_u8;
pub const IPI_MBOX: u8    = 0xF2_u8; // Phase 6C mailbox poke

/// Install IPI handlers in global IDT
pub unsafe fn install_handlers() {
    // IPI handlers will be automatically installed by the IDT module
    // This function exists for compatibility and can be extended later
    // TODO: Add actual IPI handler registration once IDT provides the API
    serial::write_str("[ipi] IPI handlers ready (vectors 0xF0-0xF2)\n");
}

// IPI handlers are now defined in arch::x86_64::idt module

#[inline]
pub fn send_resched_ipi(apic_id: u32) {
    unsafe { apic::send_ipi(apic_id, IPI_RESCHED); }
}

#[inline]
pub fn send_tlb_ipi(apic_id: u32) {
    unsafe { apic::send_ipi(apic_id, IPI_TLB); }
}

#[inline]
pub fn send_mailbox_ipi(apic_id: u32) {
    unsafe { apic::send_ipi(apic_id, IPI_MBOX); }
}