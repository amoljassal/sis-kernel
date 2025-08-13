//! Interrupt control helpers.
//!
//! Provides simple functions for enabling and disabling CPU
//! interrupts.  These wrappers use inline assembly to execute the
//! `sti` and `cli` instructions.

use core::arch::asm;

/// Enable interrupts (Set Interrupt Flag).
#[inline]
pub fn enable() {
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Disable interrupts (Clear Interrupt Flag).
#[inline]
pub fn disable() {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}
