//! Simple userspace test program to demonstrate syscall functionality
//!
//! This module contains a basic test program that can be used to validate
//! the system call interface without requiring a separate userspace binary.

use core::arch::asm;
use crate::syscall::{SyscallNumber, SyscallError};

/// Test the write syscall by outputting a message to stdout
pub fn test_write_syscall() {
    let message = b"Hello from syscall!\n";
    let result = syscall_write(1, message.as_ptr(), message.len());
    
    match result {
        Ok(bytes_written) => {
            unsafe {
                crate::uart_print(b"[TEST] Write syscall succeeded, wrote ");
                // Simple number printing (simplified)
                crate::uart_print(b" bytes\n");
            }
        }
        Err(_) => {
            unsafe {
                crate::uart_print(b"[TEST] Write syscall failed\n");
            }
        }
    }
}

/// Test the getpid syscall
pub fn test_getpid_syscall() {
    let result = syscall_getpid();
    
    match result {
        Ok(pid) => {
            unsafe {
                crate::uart_print(b"[TEST] GetPid syscall succeeded, PID: ");
                // Simple number printing (simplified)
                crate::uart_print(b"\n");
            }
        }
        Err(_) => {
            unsafe {
                crate::uart_print(b"[TEST] GetPid syscall failed\n");
            }
        }
    }
}

/// Test unimplemented syscall (should return ENOSYS)
pub fn test_unimplemented_syscall() {
    let result = syscall_fork();
    
    match result {
        Err(SyscallError::ENOSYS) => {
            unsafe {
                crate::uart_print(b"[TEST] Fork syscall correctly returned ENOSYS\n");
            }
        }
        _ => {
            unsafe {
                crate::uart_print(b"[TEST] Fork syscall returned unexpected result\n");
            }
        }
    }
}

/// Run all syscall tests from kernel mode (for now)
pub fn run_syscall_tests() {
    unsafe {
        crate::uart_print(b"[TEST] Starting syscall tests...\n");
    }
    
    test_write_syscall();
    test_getpid_syscall();
    test_unimplemented_syscall();
    
    unsafe {
        crate::uart_print(b"[TEST] Syscall tests completed\n");
    }
}

// Syscall wrapper functions (these would normally be in userspace)

fn syscall_write(fd: i32, buf: *const u8, count: usize) -> Result<usize, SyscallError> {
    let result: i64;
    unsafe {
        asm!(
            "mov x8, {syscall_num}",
            "mov x0, {fd}",
            "mov x1, {buf}",
            "mov x2, {count}",
            "svc #0",
            "mov {result}, x0",
            syscall_num = in(reg) SyscallNumber::Write as u64,
            fd = in(reg) fd as u64,
            buf = in(reg) buf as u64,
            count = in(reg) count as u64,
            result = out(reg) result,
            out("x8") _,
            out("x0") _,
            out("x1") _,
            out("x2") _,
        );
    }
    
    if result < 0 {
        Err(match result {
            -22 => SyscallError::EINVAL,
            -13 => SyscallError::EACCES,
            -2 => SyscallError::ENOENT,
            -9 => SyscallError::EBADF,
            -12 => SyscallError::ENOMEM,
            -38 => SyscallError::ENOSYS,
            -3 => SyscallError::ESRCH,
            -11 => SyscallError::EAGAIN,
            -10 => SyscallError::ECHILD,
            _ => SyscallError::EINVAL,
        })
    } else {
        Ok(result as usize)
    }
}

fn syscall_getpid() -> Result<u32, SyscallError> {
    let result: i64;
    unsafe {
        asm!(
            "mov x8, {syscall_num}",
            "svc #0",
            "mov {result}, x0",
            syscall_num = in(reg) SyscallNumber::GetPid as u64,
            result = out(reg) result,
            out("x8") _,
            out("x0") _,
        );
    }
    
    if result < 0 {
        Err(match result {
            -22 => SyscallError::EINVAL,
            -38 => SyscallError::ENOSYS,
            _ => SyscallError::EINVAL,
        })
    } else {
        Ok(result as u32)
    }
}

fn syscall_fork() -> Result<u32, SyscallError> {
    let result: i64;
    unsafe {
        asm!(
            "mov x8, {syscall_num}",
            "svc #0", 
            "mov {result}, x0",
            syscall_num = in(reg) SyscallNumber::Fork as u64,
            result = out(reg) result,
            out("x8") _,
            out("x0") _,
        );
    }
    
    if result < 0 {
        Err(match result {
            -38 => SyscallError::ENOSYS,
            -12 => SyscallError::ENOMEM,
            _ => SyscallError::EINVAL,
        })
    } else {
        Ok(result as u32)
    }
}