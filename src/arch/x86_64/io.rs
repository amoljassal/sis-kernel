#![allow(dead_code)]

use core::arch::asm;

/// QEMU isa-debug-exit convention (0x50100 * status + 1)
#[inline(always)]
pub unsafe fn qemu_exit(code: u32) -> ! {
    let value: u32 = (0x50100 * code) + 1;
    asm!(
        "out dx, eax",
        in("dx") 0xF4u16,
        in("eax") value,
        options(noreturn, preserves_flags)
    )
}

#[inline(always)]
pub unsafe fn qemu_exit_ok() -> ! {
    qemu_exit(0)
}
#[inline(always)]
pub unsafe fn qemu_exit_fail() -> ! {
    qemu_exit(0xFF)
}
#[inline(always)]
pub unsafe fn qemu_exit_skip() -> ! {
    qemu_exit(0x50)
}

// Functions for port-based I/O
#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack));
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack));
    ret
}

#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack));
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let ret: u32;
    asm!("in eax, dx", out("eax") ret, in("dx") port, options(nomem, nostack));
    ret
}
