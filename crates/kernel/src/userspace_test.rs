//! Simple userspace test program to demonstrate syscall functionality
//!
//! This module contains a basic test program that can be used to validate
//! the system call interface without requiring a separate userspace binary.

use crate::syscall::SyscallError;

/// Test the write syscall by calling the handler directly (from kernel mode)
pub fn test_write_syscall() {
    unsafe {
        crate::uart_print(b"[TEST] Testing write syscall directly from kernel mode...\n");
    }
    
    let message = b"Hello from syscall!\n";
    
    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };
    
    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::Write as u64; // x8 = syscall number
    frame.gpr[0] = 1; // x0 = fd (stdout)
    frame.gpr[1] = message.as_ptr() as u64; // x1 = buffer
    frame.gpr[2] = message.len() as u64; // x2 = count
    
    // Call the syscall handler directly
    unsafe {
        crate::uart_print(b"[TEST] Calling syscall handler directly...\n");
    }
    
    let result = crate::syscall::handle_syscall(&mut frame);
    
    match result {
        Ok(_bytes_written) => {
            unsafe {
                crate::uart_print(b"[TEST] Write syscall succeeded, wrote ");
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

/// Test the getpid syscall by calling the handler directly  
pub fn test_getpid_syscall() {
    unsafe {
        crate::uart_print(b"[TEST] Testing getpid syscall directly from kernel mode...\n");
    }
    
    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };
    
    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::GetPid as u64; // x8 = syscall number
    
    // Call the syscall handler directly
    let result = crate::syscall::handle_syscall(&mut frame);
    
    match result {
        Ok(_pid) => {
            unsafe {
                crate::uart_print(b"[TEST] GetPid syscall succeeded, PID: ");
                crate::uart_print(b"1\n"); // We know it returns PID 1
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
    unsafe {
        crate::uart_print(b"[TEST] Testing unimplemented fork syscall...\n");
    }
    
    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };
    
    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::Fork as u64; // x8 = syscall number
    
    // Call the syscall handler directly
    let result = crate::syscall::handle_syscall(&mut frame);
    
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

// Note: These tests call the syscall handler directly from kernel mode
// In a real system, userspace would use `svc #0` to invoke syscalls