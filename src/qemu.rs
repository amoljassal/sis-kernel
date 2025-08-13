#![allow(dead_code)]
use crate::arch::x86_64::io;

/// Exit QEMU with success code (diverging function for type compatibility)
#[inline(always)]
pub fn exit_ok() -> ! {
    unsafe { io::qemu_exit_ok() }
}

/// Exit QEMU with failure code (diverging function for type compatibility)
#[inline(always)]
pub fn exit_fail(code: u8) -> ! {
    unsafe { io::qemu_exit(code) }
}

/// Exit QEMU with skip code (test was skipped - diverging function)
#[inline(always)]
pub fn exit_skip(_code: u32) -> ! {
    unsafe { io::qemu_exit_skip() }
}
