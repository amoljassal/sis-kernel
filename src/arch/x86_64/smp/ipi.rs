//! Inter-Processor Interrupt (IPI) support for SMP scheduling
//!
//! This module provides a simple interface to the existing IPI infrastructure
//! for Phase 6B CPU affinity support.

use crate::kernel::serial;
use crate::arch::x86_64::apic;

/// IPI vector for reschedule requests (matches existing IDT setup)
pub const IPI_RESCHED_VEC: u8 = 0xF0;

/// Install IPI handlers - delegates to existing IDT setup
pub unsafe fn install_handlers() {
    // The IDT already has IPI handlers installed at initialization
    // This is a no-op for compatibility with the patch
    serial::write_str("[ipi] Using existing IPI handlers (vector 0xF0)\n");
}

/// Send reschedule IPI to target CPU
pub unsafe fn send_resched(target_apic_id: u32) {
    serial::write_str("[ipi] Sending reschedule IPI to APIC ID ");
    serial::write_u64(target_apic_id as u64);
    serial::write_str("\n");
    
    // Send fixed IPI with reschedule vector
    apic::send_ipi(target_apic_id, (IPI_RESCHED_VEC as u32) | 0x4000); // Fixed delivery mode
}