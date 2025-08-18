#![allow(dead_code)]
use crate::arch::x86_64::io;

#[inline(always)]
pub fn exit_ok() -> ! {
    unsafe { io::qemu_exit_ok() }
}
#[inline(always)]
pub fn exit_fail() -> ! {
    unsafe { io::qemu_exit_fail() }
}
#[inline(always)]
pub fn exit_skip(_code: u32) -> ! {
    unsafe { io::qemu_exit_skip() }
}
