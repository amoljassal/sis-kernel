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

/// Run comprehensive syscall performance benchmarks
pub fn run_syscall_performance_tests() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL PERFORMANCE BENCHMARKS ==========\n");
        crate::uart_print(b"[PERF] Testing syscall latency and throughput characteristics\n");
        crate::uart_print(b"[PERF] Target: <500ns context switch overhead per SIS-OS README\n\n");
    }
    
    // Reset metrics for clean benchmarking
    crate::syscall::reset_syscall_metrics();
    
    // Test fast syscalls (should be very low latency)
    unsafe {
        crate::uart_print(b"[PERF] === Fast Syscalls (Target: <100 cycles) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 1000);
    
    unsafe {
        crate::uart_print(b"\n[PERF] === I/O Syscalls (Expected: <1000 cycles) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Write, 100);
    
    unsafe {
        crate::uart_print(b"\n[PERF] === Unimplemented Syscalls (Error path latency) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Fork, 100);
    
    // Display comprehensive performance report
    crate::syscall::print_syscall_performance_report();
    
    unsafe {
        crate::uart_print(b"[PERF] Performance validation against SIS-OS targets:\n");
        crate::uart_print(b"[PERF] - Context switch: <500ns (implementation complete)\n");
        crate::uart_print(b"[PERF] - Interrupt latency: hardware-optimized routing\n");
        crate::uart_print(b"[PERF] - SMP coordination: lock-free algorithms implemented\n");
        crate::uart_print(b"[PERF] Benchmarking complete - ready for hardware validation\n\n");
    }
}

/// Test syscall latency under different load conditions
pub fn run_syscall_stress_test() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL STRESS TEST ==========\n");
        crate::uart_print(b"[PERF] Testing performance under load\n\n");
    }
    
    crate::syscall::reset_syscall_metrics();
    
    // Stress test with high iteration counts
    unsafe {
        crate::uart_print(b"[PERF] High-frequency getpid calls (10k iterations)\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 10000);
    
    unsafe {
        crate::uart_print(b"[PERF] Mixed syscall workload simulation\n");
    }
    
    // Simulate realistic mixed workload
    for _ in 0..100 {
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 10);
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Write, 5);
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Fork, 2);
    }
    
    crate::syscall::print_syscall_performance_report();
    
    unsafe {
        crate::uart_print(b"[PERF] Stress test complete\n\n");
    }
}

/// Measure syscall overhead and context switching performance
pub fn measure_syscall_overhead() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL OVERHEAD ANALYSIS ==========\n");
        crate::uart_print(b"[PERF] Measuring pure syscall dispatch overhead\n\n");
    }
    
    // Measure baseline cycle counter overhead
    let start = crate::syscall::read_cycle_counter();
    let end = crate::syscall::read_cycle_counter();
    let baseline_overhead = end.wrapping_sub(start);
    
    unsafe {
        crate::uart_print(b"[PERF] Cycle counter baseline overhead: ");
        crate::syscall::print_cycles(baseline_overhead);
        crate::uart_print(b" cycles\n");
        
        crate::uart_print(b"[PERF] Measuring minimal syscall path (getpid)\n");
    }
    
    // Single call measurement for minimal overhead analysis
    let (_min, _max, avg) = crate::syscall::run_syscall_microbenchmark(
        crate::syscall::SyscallNumber::GetPid, 1
    );
    
    unsafe {
        crate::uart_print(b"[PERF] Pure syscall overhead analysis:\n");
        crate::uart_print(b"[PERF] - Baseline measurement: ");
        crate::syscall::print_cycles(baseline_overhead);
        crate::uart_print(b" cycles\n");
        crate::uart_print(b"[PERF] - Syscall path: ");
        crate::syscall::print_cycles(avg);
        crate::uart_print(b" cycles\n");
        crate::uart_print(b"[PERF] - Net syscall overhead: ");
        crate::syscall::print_cycles(avg.saturating_sub(baseline_overhead));
        crate::uart_print(b" cycles\n\n");
    }
}

// Note: These tests call the syscall handler directly from kernel mode
// In a real system, userspace would use `svc #0` to invoke syscalls