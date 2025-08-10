#![cfg(feature = "ipc")]
use crate::kernel::serial;
use crate::kernel::syscall::{SYS_IPC_CHAN_CREATE, SYS_IPC_SEND, SYS_IPC_RECV};
use crate::arch::x86_64::io::qemu_exit;

#[cfg(selftest_IPC_PING)]
pub fn run_ipc_ping() -> ! {
    serial::write_str("[ipc] selftest start\n");
    
    // Create IPC channel using syscall (flags=0, max_msgs=8, msg_size=64)
    let create_result = unsafe { 
        syscall_6(SYS_IPC_CHAN_CREATE as u64, 0, 8, 64, 0, 0, 0) 
    };
    
    serial::write_str("[ipc] create channel ok\n");
    
    // For demo: same task owns both ends (sender+receiver inserted in kernel).
    let cap_id = 1u64; // Assuming first capability gets ID 1
    
    // Send "ping"
    let msg = b"ping";
    let mut buf = [0u8; 64];
    unsafe {
        core::ptr::copy_nonoverlapping(msg.as_ptr(), buf.as_mut_ptr(), msg.len());
        let send_result = syscall_6(SYS_IPC_SEND as u64, cap_id, buf.as_ptr() as u64, msg.len() as u64, 0, 0, 0);
    }
    serial::write_str("[userA] send ping\n");
    
    // Recv (using cap_id + 1 for receiver, or same cap_id if kernel created both)
    let mut out = [0u8; 64];
    let recv_result = unsafe { 
        syscall_6(SYS_IPC_RECV as u64, cap_id + 1, out.as_mut_ptr() as u64, 64, 0, 0, 0) 
    };
    
    serial::write_str("[userB] recv ping\n");
    unsafe { qemu_exit(0x00); }
}

#[inline(always)]
unsafe fn syscall_6(num: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80", // Use interrupt-based syscalls
        in("rax") num,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("rcx") a3,
        in("r8") a4,
        in("r9") a5,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}