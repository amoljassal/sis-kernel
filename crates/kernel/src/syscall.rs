//! System call interface for SIS kernel
//!
//! Implements ARM64 system call handling with EL0 -> EL1 transitions.
//! Provides POSIX-compatible system calls for userspace applications.

use core::arch::asm;

/// System call numbers (following Linux ARM64 convention)
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Read = 63,
    Write = 64,
    Exit = 93,
    Fork = 220,
    Exec = 221,
    Open = 56,
    Close = 57,
    Mmap = 222,
    Munmap = 215,
    Brk = 214,
    GetPid = 172,
    GetPpid = 173,
    Wait4 = 260,
    /// Invalid system call number
    Invalid = u64::MAX,
}

impl From<u64> for SyscallNumber {
    fn from(num: u64) -> Self {
        match num {
            63 => SyscallNumber::Read,
            64 => SyscallNumber::Write,
            93 => SyscallNumber::Exit,
            220 => SyscallNumber::Fork,
            221 => SyscallNumber::Exec,
            56 => SyscallNumber::Open,
            57 => SyscallNumber::Close,
            222 => SyscallNumber::Mmap,
            215 => SyscallNumber::Munmap,
            214 => SyscallNumber::Brk,
            172 => SyscallNumber::GetPid,
            173 => SyscallNumber::GetPpid,
            260 => SyscallNumber::Wait4,
            _ => SyscallNumber::Invalid,
        }
    }
}

/// System call arguments passed in ARM64 registers
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
}

/// System call result
pub type SyscallResult = Result<u64, SyscallError>;

/// System call errors (negative errno values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SyscallError {
    /// Invalid argument
    EINVAL = -22,
    /// Permission denied  
    EACCES = -13,
    /// No such file or directory
    ENOENT = -2,
    /// Bad file descriptor
    EBADF = -9,
    /// Out of memory
    ENOMEM = -12,
    /// Function not implemented
    ENOSYS = -38,
    /// No such process
    ESRCH = -3,
    /// Resource temporarily unavailable
    EAGAIN = -11,
    /// No child processes
    ECHILD = -10,
}

impl From<SyscallError> for u64 {
    fn from(err: SyscallError) -> u64 {
        (err as i32) as u64
    }
}

/// Saved processor state during system call
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallFrame {
    /// General purpose registers x0-x30
    pub gpr: [u64; 31],
    /// Stack pointer (EL0)
    pub sp_el0: u64,
    /// Exception link register
    pub elr_el1: u64,
    /// Saved program status register
    pub spsr_el1: u64,
}

impl SyscallFrame {
    /// Get system call number from x8 register
    pub fn syscall_number(&self) -> SyscallNumber {
        SyscallNumber::from(self.gpr[8])
    }

    /// Get system call arguments from registers
    pub fn args(&self) -> SyscallArgs {
        SyscallArgs {
            x0: self.gpr[0],
            x1: self.gpr[1],
            x2: self.gpr[2],
            x3: self.gpr[3],
            x4: self.gpr[4],
            x5: self.gpr[5],
        }
    }

    /// Set return value in x0 register
    pub fn set_return_value(&mut self, value: u64) {
        self.gpr[0] = value;
    }
}

/// Main system call dispatcher
pub fn handle_syscall(frame: &mut SyscallFrame) -> SyscallResult {
    unsafe {
        crate::uart_print(b"[SYSCALL] Raw x8 register: ");
        let raw_x8 = frame.gpr[8];
        // Print the raw x8 register value in hex
        crate::uart_print(b"0x");
        for i in (0..16).rev() {
            let nibble = (raw_x8 >> (i * 4)) & 0xF;
            let c = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
            crate::uart_print(&[c]);
        }
        crate::uart_print(b"\n");
    }
    
    let syscall_num = frame.syscall_number();
    let args = frame.args();

    // Performance measurement start
    let start_cycles = read_cycle_counter();

    unsafe {
        crate::uart_print(b"[SYSCALL] Dispatching syscall number: ");
        // Print the actual syscall number for debugging
        let num = syscall_num as u64;
        if num == 64 {
            crate::uart_print(b"WRITE(64)");
        } else if num == 63 {
            crate::uart_print(b"READ(63)");
        } else if num == 93 {
            crate::uart_print(b"EXIT(93)");
        } else if num == 172 {
            crate::uart_print(b"GETPID(172)");
        } else if num == 220 {
            crate::uart_print(b"FORK(220)");
        } else if num == u64::MAX {
            crate::uart_print(b"INVALID(MAX)");
        } else {
            crate::uart_print(b"UNKNOWN(");
            // Print raw number in a simple way
            if num < 10 {
                crate::uart_print(&[b'0' + num as u8]);
            } else if num < 100 {
                crate::uart_print(&[b'0' + (num / 10) as u8, b'0' + (num % 10) as u8]);
            } else {
                crate::uart_print(b"XXX");
            }
            crate::uart_print(b")");
        }
        crate::uart_print(b"\n");
    }
    
    let result = match syscall_num {
        SyscallNumber::Read => sys_read(args.x0 as i32, args.x1 as *mut u8, args.x2),
        SyscallNumber::Write => {
            unsafe {
                crate::uart_print(b"[SYSCALL] Calling sys_write with fd=");
                crate::uart_print(if args.x0 == 1 { b"1(stdout)" } else { b"OTHER" });
                crate::uart_print(b"\n");
            }
            sys_write(args.x0 as i32, args.x1 as *const u8, args.x2)
        }
        SyscallNumber::Exit => sys_exit(args.x0 as i32),
        SyscallNumber::Fork => sys_fork(),
        SyscallNumber::Exec => sys_exec(args.x0 as *const u8, args.x1 as *const *const u8),
        SyscallNumber::Open => sys_open(args.x0 as *const u8, args.x1 as i32, args.x2 as u32),
        SyscallNumber::Close => sys_close(args.x0 as i32),
        SyscallNumber::Mmap => sys_mmap(args.x0, args.x1, args.x2 as i32, args.x3 as i32, args.x4 as i32, args.x5 as i64),
        SyscallNumber::Munmap => sys_munmap(args.x0, args.x1),
        SyscallNumber::Brk => sys_brk(args.x0),
        SyscallNumber::GetPid => sys_getpid(),
        SyscallNumber::GetPpid => sys_getppid(),
        SyscallNumber::Wait4 => sys_wait4(args.x0 as i32, args.x1 as *mut i32, args.x2 as i32),
        SyscallNumber::Invalid => Err(SyscallError::ENOSYS),
    };

    // Performance measurement end
    let end_cycles = read_cycle_counter();
    let latency_cycles = end_cycles.wrapping_sub(start_cycles);

    // Record syscall performance metrics
    record_syscall_metrics(syscall_num, latency_cycles, result.is_ok());

    result
}

/// Read system call - read from file descriptor
fn sys_read(fd: i32, buf: *mut u8, count: u64) -> SyscallResult {
    // Validate file descriptor
    if fd < 0 {
        return Err(SyscallError::EBADF);
    }

    // Validate buffer pointer and size
    if buf.is_null() || count == 0 {
        return Err(SyscallError::EINVAL);
    }

    // For now, implement basic UART read for stdin (fd 0)
    if fd == 0 {
        // TODO: Implement UART input buffering
        let bytes_read = uart_read_bytes(buf, count as usize)?;
        Ok(bytes_read as u64)
    } else {
        // TODO: Integrate with SIS filesystem module
        Err(SyscallError::ENOSYS)
    }
}

/// Write system call - write to file descriptor  
fn sys_write(fd: i32, buf: *const u8, count: u64) -> SyscallResult {
    // Validate file descriptor
    if fd < 0 {
        return Err(SyscallError::EBADF);
    }

    // Validate buffer pointer and size
    if buf.is_null() || count == 0 {
        return Ok(0);
    }

    // Implement UART write for stdout/stderr (fd 1, 2)
    if fd == 1 || fd == 2 {
        unsafe {
            crate::uart_print(b"[SYSCALL] sys_write: fd is stdout/stderr, calling uart_write_bytes\n");
        }
        let bytes_written = uart_write_bytes(buf, count as usize)?;
        unsafe {
            crate::uart_print(b"[SYSCALL] sys_write: uart_write_bytes succeeded\n");
        }
        Ok(bytes_written as u64)
    } else {
        unsafe {
            crate::uart_print(b"[SYSCALL] sys_write: fd is not stdout/stderr, returning ENOSYS\n");
        }
        // TODO: Integrate with SIS filesystem module
        Err(SyscallError::ENOSYS)
    }
}

/// Exit system call - terminate current process
fn sys_exit(status: i32) -> SyscallResult {
    // Use existing uart_print function
    unsafe {
        crate::uart_print(b"[SYSCALL] Process exit with status: ");
        // Convert status to string and print (simplified)
        crate::uart_print(b"\n");
    }
    
    // For single process system, halt
    loop {}
}

/// Fork system call - create new process
fn sys_fork() -> SyscallResult {
    // TODO: Implement process management integration
    Err(SyscallError::ENOSYS)
}

/// Exec system call - replace current process image
fn sys_exec(path: *const u8, argv: *const *const u8) -> SyscallResult {
    // TODO: Integrate with ELF loader
    Err(SyscallError::ENOSYS)
}

/// Open system call - open file
fn sys_open(path: *const u8, flags: i32, mode: u32) -> SyscallResult {
    // TODO: Integrate with filesystem
    Err(SyscallError::ENOSYS)
}

/// Close system call - close file descriptor
fn sys_close(fd: i32) -> SyscallResult {
    // TODO: Integrate with file descriptor table
    Err(SyscallError::ENOSYS)
}

/// Memory map system call
fn sys_mmap(addr: u64, length: u64, prot: i32, flags: i32, fd: i32, offset: i64) -> SyscallResult {
    // TODO: Integrate with memory management
    Err(SyscallError::ENOSYS)
}

/// Memory unmap system call
fn sys_munmap(addr: u64, length: u64) -> SyscallResult {
    // TODO: Integrate with memory management
    Err(SyscallError::ENOSYS)
}

/// Program break system call - adjust heap size
fn sys_brk(addr: u64) -> SyscallResult {
    // TODO: Integrate with heap management
    Err(SyscallError::ENOSYS)
}

/// Get process ID
fn sys_getpid() -> SyscallResult {
    // TODO: Integrate with process manager
    Ok(1) // Temporary: return PID 1 for init process
}

/// Get parent process ID
fn sys_getppid() -> SyscallResult {
    // TODO: Integrate with process manager
    Ok(0) // Temporary: return PID 0 for kernel
}

/// Wait for child process
fn sys_wait4(pid: i32, status: *mut i32, options: i32) -> SyscallResult {
    // TODO: Integrate with process scheduler
    Err(SyscallError::ENOSYS)
}

/// UART read implementation
fn uart_read_bytes(buf: *mut u8, count: usize) -> Result<usize, SyscallError> {
    // TODO: Implement UART input buffering
    Ok(0)
}

/// UART write implementation using existing uart_print
fn uart_write_bytes(buf: *const u8, count: usize) -> Result<usize, SyscallError> {
    unsafe {
        let slice = core::slice::from_raw_parts(buf, count);
        crate::uart_print(slice);
    }
    Ok(count)
}

/// Read ARM64 cycle counter for performance measurement
#[inline(always)]
fn read_cycle_counter() -> u64 {
    unsafe {
        let mut count: u64;
        asm!("mrs {}, cntvct_el0", out(reg) count);
        count
    }
}

/// Record system call performance metrics
fn record_syscall_metrics(syscall: SyscallNumber, cycles: u64, success: bool) {
    // TODO: Integrate with performance monitoring
    const HIGH_LATENCY_THRESHOLD: u64 = 1000; // cycles
    
    if cycles > HIGH_LATENCY_THRESHOLD {
        unsafe {
            crate::uart_print(b"[PERF] High latency syscall detected\n");
        }
    }
}

/// System call exception handler (called from assembly)
#[no_mangle]
pub extern "C" fn syscall_handler(frame: *mut SyscallFrame) {
    unsafe {
        crate::uart_print(b"[SYSCALL] Handler called\n");
        let frame_ref = &mut *frame;
        crate::uart_print(b"[SYSCALL] About to handle syscall\n");
        match handle_syscall(frame_ref) {
            Ok(result) => {
                crate::uart_print(b"[SYSCALL] Success, setting return value\n");
                frame_ref.set_return_value(result);
            }
            Err(error) => {
                crate::uart_print(b"[SYSCALL] Error, setting error value\n");
                frame_ref.set_return_value(error.into());
            }
        }
        crate::uart_print(b"[SYSCALL] Handler returning\n");
    }
}