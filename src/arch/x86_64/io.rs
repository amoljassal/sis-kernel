//! Low level port I/O operations for x86_64.
//!
//! These functions wrap the `in` and `out` instructions to read
//! and write bytes, words and double words from I/O ports.  They
//! are marked `unsafe` because incorrect use can crash the machine.

use core::arch::asm;

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    val
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    asm!("in ax, dx", in("dx") port, out("ax") val, options(nomem, nostack, preserves_flags));
    val
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    asm!("in eax, dx", in("dx") port, out("eax") val, options(nomem, nostack, preserves_flags));
    val
}

/// Exit QEMU immediately via isa-debug-exit device at I/O port 0xF4.
/// Use with `-device isa-debug-exit,iobase=0xf4,iosize=0x04`.
/// Convention: 0x00 = success; non-zero = failure code.
#[inline(always)]
pub unsafe fn qemu_exit(code: u8) -> ! {
    // Some QEMU variants exit with (code << 1) | 1; harness should handle both.
    outb(0xF4, code);
    // If device not present, fall back to halt loop to avoid undefined behavior.
    loop {
        crate::arch::x86_64::cpu::halt();
    }
}
