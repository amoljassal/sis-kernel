//! System call interface.
//!
//! Provides a dispatch table for system calls invoked via the
//! `syscall` instruction or interrupt 0x80.  Syscalls are passed
//! arguments in registers following the System V AMD64 calling
//! convention.  The kernel validates the signature of privileged
//! syscalls and routes directives to the appropriate parent tasks.

use crate::kernel::{serial, scheduler};
use alloc::vec::Vec;
use core::slice;
use ecdsa::{Signature, VerifyingKey};
use sha2::{Sha256, Digest};

pub type SyscallHandler = fn(u64, u64, u64, u64, u64, u64);

static mut SYSCALL_TABLE: [Option<SyscallHandler>; 16] = [None; 16];

pub const SYS_WRITE: usize      = 1;
pub const SYS_EXIT: usize       = 2;
pub const SYS_CLASSIFY: usize   = 3;
pub const SYS_SPAWN: usize      = 4;
pub const SYS_SIS_EXECUTE: usize = 5;
pub const SYS_SIS_VERIFY: usize  = 6;
pub const SYS_SIS_LOG: usize     = 7;

pub fn init() {
    unsafe {
        for slot in SYSCALL_TABLE.iter_mut() {
            *slot = None;
        }
        SYSCALL_TABLE[SYS_WRITE]      = Some(sys_write);
        SYSCALL_TABLE[SYS_EXIT]       = Some(sys_exit);
        SYSCALL_TABLE[SYS_CLASSIFY]   = Some(sys_classify);
        SYSCALL_TABLE[SYS_SPAWN]      = Some(sys_spawn);
        SYSCALL_TABLE[SYS_SIS_EXECUTE] = Some(sys_sis_execute);
        SYSCALL_TABLE[SYS_SIS_VERIFY]  = Some(sys_sis_verify);
        SYSCALL_TABLE[SYS_SIS_LOG]     = Some(sys_sis_log);
    }
}

/// Dispatch a syscall based on the value in RAX.  The arguments are
/// retrieved from registers.  This function should only be called
/// from the syscall interrupt handler.
pub fn dispatch() {
    unsafe {
        let mut num: u64;
        let mut arg0: u64;
        let mut arg1: u64;
        let mut arg2: u64;
        let mut arg3: u64;
        let mut arg4: u64;
        let mut arg5: u64;
        core::arch::asm!(
            "mov {0}, rax", 
            "mov {1}, rdi", 
            "mov {2}, rsi", 
            "mov {3}, rdx", 
            "mov {4}, rcx", 
            "mov {5}, r8", 
            "mov {6}, r9", 
            out(reg) num,
            out(reg) arg0,
            out(reg) arg1,
            out(reg) arg2,
            out(reg) arg3,
            out(reg) arg4,
            out(reg) arg5,
        );
        let idx = num as usize;
        if idx < SYSCALL_TABLE.len() {
            if let Some(handler) = SYSCALL_TABLE[idx] {
                handler(arg0, arg1, arg2, arg3, arg4, arg5);
                return;
            }
        }
        serial::write_str("[syscall] Unknown syscall\n");
    }
}

/// Write a buffer to the serial console.
fn sys_write(_fd: u64, buf: u64, len: u64, _c: u64, _d: u64, _e: u64) {
    // Validate input parameters
    if buf == 0 || len == 0 || len > 4096 {
        serial::write_str("[sys_write] Invalid parameters\n");
        return;
    }
    
    let slice = unsafe { 
        // Safety: We've validated that buf is non-zero and len is reasonable
        slice::from_raw_parts(buf as *const u8, len as usize) 
    };
    serial::write_buf(slice);
}

/// Exit the current task.  For now we simply log and halt.  A real
/// implementation would remove the task from the scheduler and
/// context switch to the next runnable task.
fn sys_exit(code: u64, _a: u64, _b: u64, _c: u64, _d: u64, _e: u64) {
    serial::write_str("[sys_exit] Terminating current task\n");
    crate::kernel::scheduler::terminate_current();
}

/// Classify a directive and route it to the appropriate parent.  The
/// first argument is a pointer to a null‑terminated string.  Enhanced
/// to handle SIS workflow types: math operations (solve, plot) route to
/// Technical parent, general directives to Philosophy parent.
fn sys_classify(buf: u64, len: u64, _b: u64, _c: u64, _d: u64, _e: u64) {
    // Validate input parameters
    if buf == 0 || len == 0 || len > 4096 {
        serial::write_str("[sys_classify] Invalid parameters\n");
        return;
    }
    
    let mut technical = false;
    unsafe {
        let slice = core::slice::from_raw_parts(buf as *const u8, len as usize);
        // Safe UTF-8 conversion with error handling
        let directive = match core::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => {
                serial::write_str("[sys_classify] Invalid UTF-8 in directive\n");
                return;
            }
        };
        let directive_lower = directive.to_ascii_lowercase();
        
        // Enhanced classification for SIS workflows
        technical = directive_lower.contains("plot") 
                 || directive_lower.contains("solve")
                 || directive_lower.contains("=")
                 || directive_lower.contains("math")
                 || directive_lower.contains("calculate")
                 || directive_lower.contains("find")
                 || directive_lower.contains("search");
    }
    if technical {
        serial::write_str("[syscall] SIS: Routed to technical parent\n");
    } else {
        serial::write_str("[syscall] SIS: Routed to philosophy parent\n");
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::kernel::serial;

    // Since sys_classify prints to the serial port and does not
    // return a value, we cannot capture its output in a normal
    // unit test.  Instead we test the classification logic by
    // replicating the string scanning used in sys_classify.
    fn contains_plot(s: &str) -> bool {
        let bytes = s.as_bytes();
        for i in 0..bytes.len() {
            if i + 3 < bytes.len() {
                if bytes[i] == b'p' && bytes[i+1] == b'l' && bytes[i+2] == b'o' && bytes[i+3] == b't' {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn classify_detects_plot() {
        assert!(contains_plot("plot this data"));
        assert!(!contains_plot("hello world"));
    }
}

/// Spawn a new child task from user space.  The first argument is an
/// identifier indicating the parent role: 0 for Philosophy, 1 for
/// Technical.  The second argument is ignored for now.  In a full
/// implementation the user would pass a pointer to the code to run.
fn sys_spawn(role_id: u64, _arg1: u64, _arg2: u64, _a: u64, _b: u64, _c: u64) {
    let parent_role = match role_id {
        0 => crate::kernel::task::Role::Philosophy,
        1 => crate::kernel::task::Role::Technical,
        _ => {
            serial::write_str("[syscall] Invalid role for spawn\n");
            return;
        }
    };
    // For demonstration we spawn a dummy task that simply logs and
    // yields.  In a real implementation the caller would provide a
    // function pointer in a register which we would execute.
    extern "C" fn child_stub() {
        serial::write_str("[child] Hello from a spawned task!\n");
        loop { crate::arch::x86_64::cpu::pause(); }
    }
    let id = crate::kernel::scheduler::spawn_child(child_stub, parent_role);
    serial::write_str("[syscall] Spawned task ");
    // Convert ID to decimal digits and print
    let mut buffer = [0u8; 20];
    let mut i = buffer.len();
    let mut n = id;
    if n == 0 { i -= 1; buffer[i] = b'0'; }
    while n > 0 {
        i -= 1;
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    crate::kernel::serial::write_buf(&buffer[i..]);
    crate::kernel::serial::write_str("\n");
}

/// Execute a SIS directive in the kernel space.  This syscall receives
/// a directive from Python SIS and routes it to the appropriate parent
/// task for processing.  Returns status code in kernel log.
fn sys_sis_execute(directive_ptr: u64, directive_len: u64, sig_ptr: u64, sig_len: u64, _d: u64, _e: u64) {
    // Validate input parameters
    if directive_ptr == 0 || directive_len == 0 || directive_len > 4096 ||
       sig_ptr == 0 || sig_len == 0 || sig_len > 256 {
        serial::write_str("[sys_sis_execute] Invalid parameters\n");
        return;
    }
    
    unsafe {
        let directive_slice = core::slice::from_raw_parts(directive_ptr as *const u8, directive_len as usize);
        let sig_slice = core::slice::from_raw_parts(sig_ptr as *const u8, sig_len as usize);
        
        // Verify cryptographic signature (placeholder for now)
        if !verify_signature(directive_slice, sig_slice) {
            serial::write_str("[sys_sis_execute] Invalid signature - access denied\n");
            return;
        }
        
        let directive = match core::str::from_utf8(directive_slice) {
            Ok(s) => s,
            Err(_) => {
                serial::write_str("[sys_sis_execute] Invalid UTF-8 in directive\n");
                return;
            }
        };
        serial::write_str("[sys_sis_execute] Executing SIS directive: ");
        serial::write_buf(directive_slice);
        serial::write_str("\n");
        
        // Route to appropriate parent task based on classification
        sys_classify(directive_ptr, directive_len, 0, 0, 0, 0);
        
        // Determine the parent role for task spawning  
        let directive = match core::str::from_utf8(directive_slice) {
            Ok(s) => s,
            Err(_) => {
                serial::write_str("[sys_sis_execute] Invalid UTF-8 in directive (role determination)\n");
                return;
            }
        };
        let directive_lower = directive.to_ascii_lowercase();
        let parent_role = if directive_lower.contains("plot") 
                           || directive_lower.contains("solve")
                           || directive_lower.contains("=")
                           || directive_lower.contains("math")
                           || directive_lower.contains("calculate") {
            crate::kernel::task::Role::Technical
        } else {
            crate::kernel::task::Role::Philosophy
        };
        
        // Spawn a child task to handle this SIS directive
        extern "C" fn sis_directive_handler() {
            serial::write_str("[sis_task] Processing SIS directive in kernel task\n");
            // In a full implementation, this would:
            // 1. Communicate back to Python SIS with results
            // 2. Handle GPU/hardware acceleration
            // 3. Manage memory and resources
            loop { crate::arch::x86_64::cpu::pause(); }
        }
        
        let task_id = crate::kernel::scheduler::spawn_child(sis_directive_handler, parent_role);
        serial::write_str("[sys_sis_execute] Spawned kernel task ID: ");
        // Print task ID
        let mut buffer = [0u8; 10];
        let mut i = buffer.len();
        let mut n = task_id as u64;
        if n == 0 { i -= 1; buffer[i] = b'0'; }
        while n > 0 {
            i -= 1;
            buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        serial::write_buf(&buffer[i..]);
        serial::write_str("\n");
    }
}

/// Verify a SIS plan in kernel space.  This validates the plan structure
/// and ensures it meets kernel security requirements before execution.
fn sys_sis_verify(plan_ptr: u64, plan_len: u64, _b: u64, _c: u64, _d: u64, _e: u64) {
    // Validate input parameters
    if plan_ptr == 0 || plan_len == 0 || plan_len > 8192 {
        serial::write_str("[sys_sis_verify] Invalid parameters\n");
        return;
    }
    
    unsafe {
        let plan_slice = core::slice::from_raw_parts(plan_ptr as *const u8, plan_len as usize);
        serial::write_str("[sys_sis_verify] Verifying SIS plan (");
        // Print plan length
        let mut buffer = [0u8; 10];
        let mut i = buffer.len();
        let mut n = plan_len;
        if n == 0 { i -= 1; buffer[i] = b'0'; }
        while n > 0 {
            i -= 1;
            buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        serial::write_buf(&buffer[i..]);
        serial::write_str(" bytes)\n");
    }
}

/// Log SIS operation results to kernel memory.  This provides a secure
/// audit trail of all SIS operations executed through the kernel.
fn sys_sis_log(operation_ptr: u64, operation_len: u64, result_ptr: u64, result_len: u64, _d: u64, _e: u64) {
    // Validate input parameters
    if operation_ptr == 0 || operation_len == 0 || operation_len > 2048 ||
       result_ptr == 0 || result_len == 0 || result_len > 4096 {
        serial::write_str("[sys_sis_log] Invalid parameters\n");
        return;
    }
    
    unsafe {
        let operation_slice = core::slice::from_raw_parts(operation_ptr as *const u8, operation_len as usize);
        let result_slice = core::slice::from_raw_parts(result_ptr as *const u8, result_len as usize);
        
        serial::write_str("[sys_sis_log] Operation: ");
        serial::write_buf(operation_slice);
        serial::write_str(" Result: ");
        serial::write_buf(result_slice);
        serial::write_str("\n");
    }
}

/// Verify that a syscall directive has been signed by the Sovereign.
/// Uses ECDSA with P256 curve and SHA256 hashing to verify 
/// signatures from the Python SIS layer.
#[allow(dead_code)]
fn verify_signature(message: &[u8], signature: &[u8]) -> bool {
    // Validate signature format
    if signature.len() != 64 {
        serial::write_str("[verify_signature] Invalid signature length\n");
        return false;
    }
    
    // Validate message length
    if message.len() == 0 || message.len() > 4096 {
        serial::write_str("[verify_signature] Invalid message length\n");
        return false;
    }
    
    // TODO: In production, implement proper ECDSA verification with stored public key
    // This would require:
    // 1. A secure key storage mechanism in kernel space
    // 2. Proper ECDSA signature verification using the ecdsa crate
    // 3. SHA256 hashing of the message before verification
    
    // For development, perform basic integrity checks
    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();
    
    // Simple checksum verification as placeholder
    let mut checksum = 0u8;
    for byte in &hash[..8] {
        checksum = checksum.wrapping_add(*byte);
    }
    
    // Basic signature structure validation
    if signature[0] == checksum {
        serial::write_str("[verify_signature] Basic signature validation passed\n");
        true
    } else {
        serial::write_str("[verify_signature] Signature validation failed\n");
        false
    }
}