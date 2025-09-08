//! Simple interactive shell for SIS kernel
//!
//! Provides basic command-line interface functionality with built-in commands.
//! Demonstrates userspace-like interaction through the syscall interface.

use crate::syscall::{SyscallNumber, SyscallError};
use core::arch::asm;

/// Maximum command line length
const MAX_CMD_LEN: usize = 256;

/// Shell command buffer
static mut CMD_BUFFER: [u8; MAX_CMD_LEN] = [0; MAX_CMD_LEN];

/// Shell prompt
const SHELL_PROMPT: &[u8] = b"sis> ";

/// Simple shell implementation
pub struct Shell {
    running: bool,
}

impl Shell {
    /// Create new shell instance
    pub fn new() -> Self {
        Shell { running: true }
    }

    /// Main shell loop
    pub fn run(&mut self) {
        unsafe {
            crate::uart_print(b"\n=== SIS Kernel Shell ===\n");
            crate::uart_print(b"Type 'help' for available commands\n\n");
        }

        while self.running {
            self.print_prompt();
            
            // For now, simulate reading a command since we don't have UART input yet
            // In a real implementation, this would read from UART
            self.simulate_command_input();
            
            if !self.running {
                break;
            }
        }
        
        unsafe {
            crate::uart_print(b"Shell terminated\n");
        }
    }

    /// Print shell prompt
    fn print_prompt(&self) {
        unsafe {
            crate::uart_print(SHELL_PROMPT);
        }
    }

    /// Simulate command input (since we don't have UART input yet)
    fn simulate_command_input(&mut self) {
        // Simulate different commands for demonstration
        static mut CMD_INDEX: usize = 0;
        
        let demo_commands: &[&[u8]] = &[
            b"help",
            b"echo Hello from SIS shell!",
            b"info", 
            b"test",
            b"exit",
        ];
        
        unsafe {
            if CMD_INDEX < demo_commands.len() {
                let cmd = demo_commands[CMD_INDEX];
                
                // Print the simulated input
                crate::uart_print(cmd);
                crate::uart_print(b"\n");
                
                // Copy to command buffer
                let len = cmd.len().min(MAX_CMD_LEN - 1);
                CMD_BUFFER[..len].copy_from_slice(&cmd[..len]);
                CMD_BUFFER[len] = 0; // Null terminate
                
                self.process_command(len);
                CMD_INDEX += 1;
                
                // Add delay for readability
                for _ in 0..1000000 {
                    core::hint::spin_loop();
                }
            } else {
                self.running = false;
            }
        }
    }

    /// Process a command
    fn process_command(&mut self, cmd_len: usize) {
        if cmd_len == 0 {
            return;
        }

        unsafe {
            let cmd_str = core::str::from_utf8_unchecked(&CMD_BUFFER[..cmd_len]);
            let parts: heapless::Vec<&str, 8> = cmd_str.split_whitespace().collect();
            
            if parts.is_empty() {
                return;
            }

            match parts[0] {
                "help" => self.cmd_help(),
                "echo" => self.cmd_echo(&parts[1..]),
                "info" => self.cmd_info(),
                "test" => self.cmd_test(),
                "exit" => self.cmd_exit(),
                _ => self.cmd_unknown(parts[0]),
            }
        }
    }

    /// Help command
    fn cmd_help(&self) {
        unsafe {
            crate::uart_print(b"Available commands:\n");
            crate::uart_print(b"  help    - Show this help message\n");
            crate::uart_print(b"  echo    - Echo text to output\n");
            crate::uart_print(b"  info    - Show kernel information\n");
            crate::uart_print(b"  test    - Run syscall tests\n");
            crate::uart_print(b"  exit    - Exit shell\n");
        }
    }

    /// Echo command
    fn cmd_echo(&self, args: &[&str]) {
        unsafe {
            if args.is_empty() {
                crate::uart_print(b"\n");
            } else {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(arg.as_bytes());
                }
                crate::uart_print(b"\n");
            }
        }
    }

    /// Info command
    fn cmd_info(&self) {
        unsafe {
            crate::uart_print(b"SIS Kernel Information:\n");
            crate::uart_print(b"  Architecture: ARM64 (AArch64)\n");
            crate::uart_print(b"  Boot Method: UEFI\n");
            crate::uart_print(b"  MMU Status: Enabled\n");
            crate::uart_print(b"  Syscalls: 13 POSIX-compatible\n");
            crate::uart_print(b"  Exception Level: EL1\n");
            
            // Get current PID via syscall
            match self.syscall_getpid() {
                Ok(pid) => {
                    crate::uart_print(b"  Current PID: ");
                    self.print_number(pid as u64);
                    crate::uart_print(b"\n");
                }
                Err(_) => {
                    crate::uart_print(b"  Current PID: Error\n");
                }
            }
        }
    }

    /// Test command
    fn cmd_test(&self) {
        unsafe {
            crate::uart_print(b"Running syscall tests...\n");
        }
        crate::userspace_test::run_syscall_tests();
    }

    /// Exit command
    fn cmd_exit(&mut self) {
        unsafe {
            crate::uart_print(b"Goodbye!\n");
        }
        self.running = false;
    }

    /// Unknown command handler
    fn cmd_unknown(&self, cmd: &str) {
        unsafe {
            crate::uart_print(b"Unknown command: ");
            crate::uart_print(cmd.as_bytes());
            crate::uart_print(b"\nType 'help' for available commands\n");
        }
    }

    /// Print a number (simple implementation)
    fn print_number(&self, mut num: u64) {
        if num == 0 {
            unsafe {
                crate::uart_print(b"0");
            }
            return;
        }

        let mut digits = [0u8; 20];
        let mut i = 0;
        
        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }
        
        // Print digits in reverse order
        while i > 0 {
            i -= 1;
            unsafe {
                crate::uart_print(&[digits[i]]);
            }
        }
    }

    /// Get PID syscall wrapper
    fn syscall_getpid(&self) -> Result<u32, SyscallError> {
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
                -38 => SyscallError::ENOSYS,
                _ => SyscallError::EINVAL,
            })
        } else {
            Ok(result as u32)
        }
    }
}

/// Initialize and run the shell
pub fn run_shell() {
    let mut shell = Shell::new();
    shell.run();
}