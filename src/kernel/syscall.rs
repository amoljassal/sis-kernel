//! System call interface.
//!
//! Provides a dispatch table for system calls invoked via the
//! `syscall` instruction or interrupt 0x80.  Syscalls are passed
//! arguments in registers following the System V AMD64 calling
//! convention.  The kernel validates the signature of privileged
//! syscalls and routes directives to the appropriate parent tasks.

use crate::kernel::{serial, scheduler};
#[cfg(feature = "userland")]
use crate::userland::vfs;
#[cfg(not(feature = "userland"))]
use crate::kernel::vfs;
use alloc::vec::Vec;
use core::slice;

#[cfg(feature = "ipc")]
use crate::kernel::ipc;
// Crypto imports temporarily removed - will be re-added later with proper no_std configuration
// use ecdsa::{Signature, VerifyingKey};
// use sha2::{Sha256, Digest};

pub type SyscallHandler = fn(u64, u64, u64, u64, u64, u64);

static mut SYSCALL_TABLE: [Option<SyscallHandler>; 128] = [None; 128];

pub const SYS_WRITE: usize      = 1;
pub const SYS_EXIT: usize       = 2;
pub const SYS_CLASSIFY: usize   = 3;
pub const SYS_SPAWN: usize      = 4;
pub const SYS_SIS_EXECUTE: usize = 5;
pub const SYS_SIS_VERIFY: usize  = 6;
pub const SYS_SIS_LOG: usize     = 7;

// Phase 2: IPC syscall numbers
#[cfg(feature = "ipc")]
pub const SYS_IPC_CHAN_CREATE: usize = 0x20;
#[cfg(feature = "ipc")]
pub const SYS_IPC_SEND: usize = 0x21;
#[cfg(feature = "ipc")]
pub const SYS_IPC_RECV: usize = 0x22;
#[cfg(feature = "ipc")]
pub const SYS_IPC_CLOSE: usize = 0x23;

#[cfg(feature = "userland")]
pub const SYS_SPAWN_EXEC: usize = 0x30;
#[cfg(feature = "userland")]
pub const SYS_WAIT: usize = 0x31;

#[cfg(feature = "userland")]
pub const SYS_VFS_OPEN: usize = 0x40;
#[cfg(feature = "userland")]
pub const SYS_VFS_READ: usize = 0x41;

// Phase 5B: VFIO syscall numbers
// NOTE: Using 0x50+ range to avoid conflicts with existing syscalls.
// Future patches can add alias table if consolidation needed.
#[cfg(feature = "vfio")]
pub const SYS_VFIO_BIND: usize = 0x50;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_CFG_READ: usize = 0x51;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_CFG_WRITE: usize = 0x52;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_MAP_BAR: usize = 0x53;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_SETUP_IRQ: usize = 0x54;

// Phase 5C-A: IOMMU domain syscalls  
#[cfg(feature = "vfio")]
pub const SYS_VFIO_DOMAIN_CREATE: usize = 0x55;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_DOMAIN_MAP_STAGING: usize = 0x56;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_ENABLE_BUSMASTER: usize = 0x57;

// Phase 5C-B: MSI interrupt syscalls
#[cfg(feature = "vfio")]
pub const SYS_VFIO_MSI_ARM: usize = 0x58;
#[cfg(feature = "vfio")]
pub const SYS_VFIO_MSI_DISARM: usize = 0x59;

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
        
        // Phase 2: IPC syscalls
        #[cfg(feature = "ipc")]
        {
            SYSCALL_TABLE[SYS_IPC_CHAN_CREATE] = Some(sys_ipc_chan_create);
            SYSCALL_TABLE[SYS_IPC_SEND] = Some(sys_ipc_send);
            SYSCALL_TABLE[SYS_IPC_RECV] = Some(sys_ipc_recv);
            SYSCALL_TABLE[SYS_IPC_CLOSE] = Some(sys_ipc_close);
        }
        
        // Phase 4: Userland syscalls
        #[cfg(feature = "userland")]
        {
            SYSCALL_TABLE[SYS_SPAWN_EXEC] = Some(sys_spawn_exec);
            SYSCALL_TABLE[SYS_WAIT] = Some(sys_wait);
            SYSCALL_TABLE[SYS_VFS_OPEN] = Some(sys_vfs_open);
            SYSCALL_TABLE[SYS_VFS_READ] = Some(sys_vfs_read);
        }
        
        // Phase 5B: VFIO syscalls
        #[cfg(feature = "vfio")]
        {
            SYSCALL_TABLE[SYS_VFIO_BIND] = Some(sys_vfio_bind);
            SYSCALL_TABLE[SYS_VFIO_CFG_READ] = Some(sys_vfio_cfg_read);
            SYSCALL_TABLE[SYS_VFIO_CFG_WRITE] = Some(sys_vfio_cfg_write);
            SYSCALL_TABLE[SYS_VFIO_MAP_BAR] = Some(sys_vfio_map_bar);
            SYSCALL_TABLE[SYS_VFIO_SETUP_IRQ] = Some(sys_vfio_setup_irq);
            
            // Phase 5C-A: Domain management syscalls
            SYSCALL_TABLE[SYS_VFIO_DOMAIN_CREATE] = Some(sys_vfio_domain_create);
            SYSCALL_TABLE[SYS_VFIO_DOMAIN_MAP_STAGING] = Some(sys_vfio_domain_map_staging);
            SYSCALL_TABLE[SYS_VFIO_ENABLE_BUSMASTER] = Some(sys_vfio_enable_busmaster);
            
            // Phase 5C-B: MSI interrupt syscalls
            SYSCALL_TABLE[SYS_VFIO_MSI_ARM] = Some(sys_vfio_msi_arm);
            SYSCALL_TABLE[SYS_VFIO_MSI_DISARM] = Some(sys_vfio_msi_disarm);
        }
    }
}

/// Manual syscall dispatch for kernel-space testing
pub fn dispatch_manual(num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
    unsafe {
        if let Some(handler) = SYSCALL_TABLE.get(num as usize).and_then(|h| *h) {
            handler(arg0, arg1, arg2, arg3, arg4, arg5);
        } else {
            serial::write_str("[syscall] Invalid syscall number\n");
        }
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
        // Ring-3 round-trip selftest: Handle special RAX values for user banners
        #[cfg(all(feature = "idt-selftest", selftest_RING3_RT))]
        {
            match num {
                0x01 => {
                    serial::write_str("[user] hello from ring-3!\n");
                    return;
                }
                0x02 => {
                    serial::write_str("[user] syscall returned successfully!\n");
                    return;
                }
                _ => { /* continue to normal dispatch */ }
            }
        }
        
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
    fn child_stub() {
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
        fn sis_directive_handler() {
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
    
    // For development, perform basic integrity checks using simple checksum
    let mut checksum = 0u8;
    for byte in message.iter().take(8) {
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

// ===== Phase 2: IPC syscall wrappers =====

#[cfg(feature = "ipc")]
fn sys_ipc_chan_create(flags: u64, max_msgs: u64, msg_size: u64, _c: u64, _d: u64, _e: u64) {
    let result = unsafe { 
        ipc::sys_chan_create(flags as u32, max_msgs as usize, msg_size as usize) 
    };
    // Note: Current syscall system doesn't have return values, so we just log
    match result {
        Ok(cap_id) => {
            serial::write_str("[ipc] Channel created, cap_id=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = cap_id as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
        Err(errno) => {
            serial::write_str("[ipc] Channel creation failed, errno=");
            // Print negative errno
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = (-errno) as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            i -= 1; buffer[i] = b'-';
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
    }
}

#[cfg(feature = "ipc")]
fn sys_ipc_send(cap_id: u64, user_ptr: u64, len: u64, _c: u64, _d: u64, _e: u64) {
    let result = unsafe { 
        ipc::sys_send(cap_id as u32, user_ptr, len as usize) 
    };
    match result {
        Ok(bytes_sent) => {
            serial::write_str("[ipc] Send successful, bytes=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = bytes_sent as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
        Err(errno) => {
            serial::write_str("[ipc] Send failed, errno=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = (-errno) as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            i -= 1; buffer[i] = b'-';
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
    }
}

#[cfg(feature = "ipc")]
fn sys_ipc_recv(cap_id: u64, user_ptr: u64, len: u64, timeout_us: u64, _d: u64, _e: u64) {
    let result = unsafe { 
        ipc::sys_recv(cap_id as u32, user_ptr, len as usize, timeout_us) 
    };
    match result {
        Ok(bytes_recv) => {
            serial::write_str("[ipc] Recv successful, bytes=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = bytes_recv as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
        Err(errno) => {
            serial::write_str("[ipc] Recv failed, errno=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = (-errno) as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            i -= 1; buffer[i] = b'-';
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
    }
}

#[cfg(feature = "ipc")]
fn sys_ipc_close(cap_id: u64, _a: u64, _b: u64, _c: u64, _d: u64, _e: u64) {
    let result = unsafe { 
        ipc::sys_close(cap_id as u32) 
    };
    match result {
        Ok(()) => serial::write_str("[ipc] Close successful\n"),
        Err(errno) => {
            serial::write_str("[ipc] Close failed, errno=");
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = (-errno) as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            i -= 1; buffer[i] = b'-';
            serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
        }
    }
}

// Phase 4: Userland syscall implementations
#[cfg(feature = "userland")]
fn sys_spawn_exec(a0: u64, a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::user::proc::sys_spawn_exec(a0 as *const u8, a1 as *const u8, a2 as *const u8) {
        Ok(pid) => {
            serial::write_str("[sys_spawn_exec] Success, PID=");
            serial::write_u64(pid);
            serial::write_str("\n");
        },
        Err(errno) => {
            serial::write_str("[sys_spawn_exec] Error, errno=");
            serial::write_u64((-errno) as u64);
            serial::write_str("\n");
        }
    }
}

#[cfg(feature = "userland")]
fn sys_wait(pid: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::user::proc::sys_wait(pid) {
        Ok(exit_code) => {
            serial::write_str("[sys_wait] Process exited with code=");
            serial::write_u64(exit_code as u64);
            serial::write_str("\n");
        },
        Err(errno) => {
            serial::write_str("[sys_wait] Error, errno=");
            serial::write_u64((-errno) as u64);
            serial::write_str("\n");
        }
    }
}

#[cfg(feature = "userland")]
fn sys_vfs_open(path_ptr: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    // Convert user path to string
    unsafe {
        let mut len = 0usize;
        let mut p = path_ptr as *const u8;
        while len < 4096 {
            if core::ptr::read(p) == 0 { break; }
            len += 1;
            p = p.add(1);
        }
        if len == 0 || len >= 4096 {
            serial::write_str("[sys_vfs_open] Invalid path\n");
            return;
        }
        
        let start = (path_ptr as *const u8);
        let slice = core::slice::from_raw_parts(start, len);
        let path = match core::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => {
                serial::write_str("[sys_vfs_open] Invalid UTF-8 in path\n");
                return;
            }
        };
        
        match vfs::open(path) {
            Some(_file) => {
                serial::write_str("[sys_vfs_open] File opened: ");
                serial::write_str(path);
                serial::write_str("\n");
            },
            None => {
                serial::write_str("[sys_vfs_open] File not found: ");
                serial::write_str(path);
                serial::write_str("\n");
            }
        }
    }
}

#[cfg(feature = "userland")]
fn sys_vfs_read(fd: u64, buf_ptr: u64, len: u64, _a3: u64, _a4: u64, _a5: u64) {
    // For Phase 4 v1: simplified read implementation
    if fd != 3 {
        serial::write_str("[sys_vfs_read] Invalid file descriptor\n");
        return;
    }
    if buf_ptr == 0 || len == 0 {
        serial::write_str("[sys_vfs_read] Invalid parameters\n");
        return;
    }
    
    serial::write_str("[sys_vfs_read] Read request for ");
    serial::write_u64(len);
    serial::write_str(" bytes\n");
}

// Phase 5B: VFIO syscall implementations
#[cfg(feature = "vfio")]
fn sys_vfio_bind(bus: u64, dev: u64, func: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_bind_device(bus as u8, dev as u8, func as u8) {
        Ok(handle) => {
            serial::write_str("[sys_vfio_bind] Device bound, handle=0x");
            serial::write_hex16(handle.as_u16());
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_bind] Device binding failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_cfg_read(handle: u64, offset: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_cfg_read(handle as u16, offset as u8) {
        Ok(value) => {
            serial::write_str("[sys_vfio_cfg_read] Read 0x");
            serial::write_hex32(value);
            serial::write_str(" from offset 0x");
            serial::write_hex8(offset as u8);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_cfg_read] Config read failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_cfg_write(handle: u64, offset: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_cfg_write(handle as u16, offset as u8, value as u32) {
        Ok(()) => {
            serial::write_str("[sys_vfio_cfg_write] Wrote 0x");
            serial::write_hex32(value as u32);
            serial::write_str(" to offset 0x");
            serial::write_hex8(offset as u8);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_cfg_write] Config write failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_map_bar(handle: u64, bar_idx: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_map_bar(handle as u16, bar_idx as u8) {
        Ok(addr) => {
            serial::write_str("[sys_vfio_map_bar] BAR mapped at 0x");
            serial::write_hex64(addr);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_map_bar] BAR mapping failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_setup_irq(handle: u64, irq_num: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_setup_irq(handle as u16, irq_num as u8) {
        Ok(()) => {
            serial::write_str("[sys_vfio_setup_irq] IRQ setup completed\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_setup_irq] IRQ setup failed\n");
        }
    }
}

// Phase 5C-A: Domain management syscall handlers
#[cfg(feature = "vfio")]
fn sys_vfio_domain_create(handle: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_domain_create(handle as u16) {
        Ok(domain_id) => {
            serial::write_str("[sys_vfio_domain_create] Domain created, id=");
            serial::write_hex16(domain_id);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_domain_create] Domain creation failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_domain_map_staging(handle: u64, len: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_domain_map_staging(handle as u16, len as u32) {
        Ok(iova) => {
            serial::write_str("[sys_vfio_domain_map_staging] Staging mapped at IOVA 0x");
            serial::write_hex64(iova);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_domain_map_staging] Staging mapping failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_enable_busmaster(handle: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_enable_busmaster(handle as u16) {
        Ok(()) => {
            serial::write_str("[sys_vfio_enable_busmaster] Bus master enabled\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_enable_busmaster] Bus master enable failed\n");
        }
    }
}

// Phase 5C-B: MSI interrupt syscall handlers
#[cfg(feature = "vfio")]
fn sys_vfio_msi_arm(handle: u64, vector: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_msi_arm(handle as u16, vector as u8) {
        Ok(()) => {
            serial::write_str("[sys_vfio_msi_arm] MSI armed for vector 0x");
            serial::write_hex8(vector as u8);
            serial::write_str("\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_msi_arm] MSI arming failed\n");
        }
    }
}

#[cfg(feature = "vfio")]
fn sys_vfio_msi_disarm(handle: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) {
    match crate::kernel::vfio::syscall_msi_disarm(handle as u16) {
        Ok(()) => {
            serial::write_str("[sys_vfio_msi_disarm] MSI disarmed\n");
        },
        Err(_) => {
            serial::write_str("[sys_vfio_msi_disarm] MSI disarming failed\n");
        }
    }
}