#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::io;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn exit_ok() -> ! {
    unsafe { io::qemu_exit_ok() }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn exit_fail() -> ! {
    unsafe { io::qemu_exit_fail() }
}

// ARM64 equivalents (placeholder for now - would integrate with real ARM64 QEMU exit)
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn exit_ok() -> ! {
    crate::kernel::serial::write_str("[QEMU] ARM64 exit_ok\n");
    loop {
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn exit_fail() -> ! {
    crate::kernel::serial::write_str("[QEMU] ARM64 exit_fail\n");
    loop {
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn exit_skip(_code: u32) -> ! {
    unsafe { io::qemu_exit_skip() }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn exit_skip(_code: u32) -> ! {
    crate::kernel::serial::write_str("[QEMU] ARM64 exit_skip\n");
    loop {
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}
