//! Process lifecycle glue: spawn_exec/wait/exit built on Phase 1–3 primitives.
use crate::kernel::serial;
#[cfg(feature = "userland")]
use crate::userland::vfs;
#[cfg(not(feature = "userland"))]
use crate::kernel::vfs;
use crate::kernel::user::elf;
use alloc::vec::Vec;

static mut NEXT_PID: u64 = 2; // PID 1 is reserved for init
static mut EXIT_CODES: [(u64, i32); 32] = [(0, 0); 32]; // Simple exit code storage
static mut EXIT_COUNT: usize = 0;

pub fn sys_exit(code: i32) -> ! {
    serial::write_str("[proc] sys_exit code=");
    serial::write_u64(code as u64);
    serial::write_str("\n");
    
    // For Phase 4 v1: store exit code and halt
    // In full implementation, this would mark task as terminated and wake waiters
    unsafe {
        if EXIT_COUNT < EXIT_CODES.len() {
            EXIT_CODES[EXIT_COUNT] = (1, code); // Use PID 1 for simplicity
            EXIT_COUNT += 1;
        }
    }
    
    // Exit via QEMU for now
    if code == 0 {
        unsafe { crate::arch::x86_64::io::qemu_exit(0x00); }
    } else {
        unsafe { crate::arch::x86_64::io::qemu_exit(0x01); }
    }
}

pub fn sys_wait(pid: u64) -> Result<i32, i32> {
    serial::write_str("[proc] sys_wait pid=");
    serial::write_u64(pid);
    serial::write_str("\n");
    
    // For Phase 4 v1: simple lookup in exit codes
    unsafe {
        for i in 0..EXIT_COUNT {
            if EXIT_CODES[i].0 == pid {
                return Ok(EXIT_CODES[i].1);
            }
        }
    }
    
    // In full implementation, this would block until the process exits
    Err(10) // ECHILD equivalent
}

pub fn sys_spawn_exec(path_ptr: *const u8, _argv: *const u8, _env: *const u8) -> Result<u64, i32> {
    // Read path from user memory (v1: trusted test harness uses kernel string)
    let path = unsafe { cstr_from_user(path_ptr).ok_or(22)? }; // EINVAL
    
    serial::write_str("[proc] sys_spawn_exec path=");
    serial::write_str(path);
    serial::write_str("\n");
    
    // Load file from VFS
    let mut file = vfs::open(path).ok_or(2)?; // ENOENT
    let mut buf = Vec::new();
    buf.resize(1 << 20, 0); // v1: up to 1 MiB
    let n = vfs::read(&mut file, &mut buf);
    buf.truncate(n);
    
    if buf.len() < 64 {
        return Err(8); // ENOEXEC - file too small to be valid ELF
    }
    
    // Parse ELF (validation only for Phase 4 v1)
    let _load_result = elf::load_into_new_as(&buf).map_err(|_| 8)?; // ENOEXEC
    
    // Assign PID
    let pid = unsafe {
        let p = NEXT_PID;
        NEXT_PID += 1;
        p
    };
    
    serial::write_str("[proc] ELF parsed successfully, assigned PID=");
    serial::write_u64(pid);
    serial::write_str("\n");
    
    Ok(pid)
}

unsafe fn cstr_from_user(mut p: *const u8) -> Option<&'static str> {
    // v1: trust small, NUL-terminated within a page
    let mut len = 0usize;
    while len < 4096 {
        let b = core::ptr::read(p);
        if b == 0 { 
            break; 
        }
        len += 1;
        p = p.add(1);
    }
    if len == 0 || len >= 4096 { 
        return None; 
    }
    let start = p.sub(len);
    let s = core::slice::from_raw_parts(start, len);
    core::str::from_utf8(s).ok()
}