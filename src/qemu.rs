//! QEMU utilities for Phase 6B patch compatibility

/// Exit QEMU with success code
pub fn exit_ok() {
    unsafe {
        crate::arch::x86_64::io::qemu_exit(0x00);
    }
}

/// Exit QEMU with failure code
pub fn exit_fail(code: u8) {
    unsafe {
        crate::arch::x86_64::io::qemu_exit(code);
    }
}

/// Exit QEMU with skip code (test was skipped)
pub fn exit_skip(code: u8) {
    unsafe {
        crate::arch::x86_64::io::qemu_exit(code);
    }
}
