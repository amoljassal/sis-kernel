//! IDT self-tests: deliberately trigger exceptions under controlled conditions.
//! Each test prints a clear serial banner, then triggers the exception.
//! Handlers should log and halt deterministically (no triple faults).
//! Feature-gated behind `idt-selftest`.
//! Selection: RUSTFLAGS="--cfg selftest_TAG" with TAG in {DIV0,GP,PF,DF,SYSCALL,TIMER,RING3}
#![allow(dead_code)]

use crate::kernel::serial;
use crate::arch::x86_64::{gdt, memory};
use x86_64::{VirtAddr, structures::paging::Size4KiB};
use core::arch::asm;

#[inline(never)]
fn banner(tag: &str) {
    serial::write_str(tag);
    serial::write_str("\n");
}

/// Trigger: Divide-by-zero (DIV0)
#[inline(never)]
pub fn trigger_divide_by_zero() -> ! {
    banner("[selftest] about to trigger DIV0");
    unsafe {
        // RAX = 1, RCX = 0, then div rcx → #DE
        asm!(
            "xor rcx, rcx",
            "mov rax, 1",
            "div rcx",
            options(noreturn)
        );
    }
}

/// Trigger: General Protection (#GP)
/// Load an invalid segment selector into DS; on x86_64 this causes #GP.
#[inline(never)]
pub fn trigger_gp_fault() -> ! {
    banner("[selftest] about to trigger GP");
    unsafe {
        asm!(
            "xor eax, eax",
            "mov ds, ax",    // load null selector -> #GP
            options(noreturn)
        );
    }
}

/// Trigger: Page Fault (#PF) by touching NULL (unmapped)
#[inline(never)]
pub fn trigger_page_fault() -> ! {
    banner("[selftest] about to trigger PF");
    unsafe {
        let ptr: *mut u64 = 0x0 as *mut u64;
        core::ptr::read_volatile(ptr); // read from unmapped -> #PF
        asm!("ud2", options(noreturn)); // should not reach
    }
}

/// Trigger: Double Fault (#DF) via stack overflow recursion.
/// With DF IST configured, this should land in DF handler and halt.
#[inline(never)]
fn deep_recurse(n: usize) -> ! {
    // Prevent tail-call elimination and make frames "real"
    unsafe { core::ptr::read_volatile(&(n as u64)); }
    deep_recurse(n + 1)
}

#[inline(never)]
pub fn trigger_double_fault() -> ! {
    banner("[selftest] about to trigger DF (stack overflow)");
    deep_recurse(0)
}

/// Trigger: Syscall (int 0x80) from ring0 (handler & exit prove path).
#[inline(never)]
pub fn trigger_syscall_ping() -> ! {
    banner("[selftest] about to trigger SYSCALL ping");
    unsafe {
        core::arch::asm!("int 0x80", options(nostack, preserves_flags));
        // handler will qemu_exit(0x00). If we return here, fall back to halt.
        core::arch::asm!("hlt");
        loop { core::arch::asm!("hlt"); }
    }
}

/// Ring-3 trampoline: Full round-trip privilege separation test.
/// Creates a real Ring-3 user mode context and performs syscalls back to Ring-0.
#[inline(never)]
pub fn trigger_ring3_syscall() -> ! {
    banner("[selftest] about to trigger RING3 syscall trampoline");
    serial::write_str("[selftest] starting RING3 round-trip...\n");
    serial::write_str("[selftest] jumping to user mode...\n");
    
    // Map user-accessible pages for stack and code
    let user_stack_addr = VirtAddr::new(0x00007F00_00001000);
    let user_code_addr = VirtAddr::new(0x00007F00_00002000);
    
    // Map user pages (simplified for selftest)
    let _ = memory::map_user_page(user_stack_addr);
    let _ = memory::map_user_page(user_code_addr);
    
    // Get GDT selectors for Ring-3
    let selectors = gdt::selectors();
    let user_cs = selectors.code_ring3.0 as u64;
    let user_ss = selectors.data_ring3.0 as u64;
    
    // Write Ring-3 code: two syscalls (0x80) then exit syscall (0x81)
    unsafe {
        let code_ptr = user_code_addr.as_u64() as *mut u8;
        let mut offset = 0;
        
        // mov rax, 0x01
        *code_ptr.offset(offset) = 0x48; offset += 1; // REX.W prefix
        *code_ptr.offset(offset) = 0xC7; offset += 1; // MOV r/m64, imm32
        *code_ptr.offset(offset) = 0xC0; offset += 1; // ModR/M for RAX
        *code_ptr.offset(offset) = 0x01; offset += 1; // immediate 0x01
        *code_ptr.offset(offset) = 0x00; offset += 1;
        *code_ptr.offset(offset) = 0x00; offset += 1;
        *code_ptr.offset(offset) = 0x00; offset += 1;
        
        // int 0x80
        *code_ptr.offset(offset) = 0xCD; offset += 1; // INT
        *code_ptr.offset(offset) = 0x80; offset += 1; // 0x80
        
        // mov rax, 0x02
        *code_ptr.offset(offset) = 0x48; offset += 1; // REX.W prefix
        *code_ptr.offset(offset) = 0xC7; offset += 1;
        *code_ptr.offset(offset) = 0xC0; offset += 1;
        *code_ptr.offset(offset) = 0x02; offset += 1; // immediate 0x02
        *code_ptr.offset(offset) = 0x00; offset += 1;
        *code_ptr.offset(offset) = 0x00; offset += 1;
        *code_ptr.offset(offset) = 0x00; offset += 1;
        
        // int 0x80
        *code_ptr.offset(offset) = 0xCD; offset += 1;
        *code_ptr.offset(offset) = 0x80; offset += 1;
        
        // int 0x81 (exit)
        *code_ptr.offset(offset) = 0xCD; offset += 1;
        *code_ptr.offset(offset) = 0x81; offset += 1;
        
        // hlt (should not reach)
        *code_ptr.offset(offset) = 0xF4; offset += 1;
    }
    
    // Craft IRET frame to jump to Ring-3
    let user_rip = user_code_addr.as_u64();
    let user_rsp = user_stack_addr.as_u64() + 4096 - 8;
    let user_rflags = 0x202; // IF=1, reserved bit=1
    
    unsafe {
        asm!(
            // Push IRET frame (SS, RSP, RFLAGS, CS, RIP)
            "push {user_ss}",
            "push {user_rsp}",  
            "push {user_rflags}",
            "push {user_cs}",
            "push {user_rip}",
            
            // Set data segments to Ring-3 (Intel syntax)
            "mov ax, {user_ds:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            
            // IRET to Ring-3
            "iretq",
            
            user_ss = in(reg) user_ss,
            user_rsp = in(reg) user_rsp,
            user_rflags = in(reg) user_rflags,
            user_cs = in(reg) user_cs,
            user_rip = in(reg) user_rip,
            user_ds = in(reg) user_ss,
            options(noreturn)
        );
    }
}

// TIMER self-test is driven by the PIT ISR. Here we just print a banner and idle.
#[inline(never)]
pub fn trigger_timer_wait() -> ! { 
    banner("[selftest] about to wait for TIMER ticks"); 
    loop { crate::arch::x86_64::cpu::halt(); } 
}

/// Run a single test by tag for flexible harnesses.
pub fn run_one(tag: &str) -> ! {
    match tag {
        "DIV0" => trigger_divide_by_zero(),
        "GP"   => trigger_gp_fault(),
        "PF"   => trigger_page_fault(),
        "SYSCALL" => trigger_syscall_ping(),
        "TIMER"   => trigger_timer_wait(),
        "RING3"   => trigger_ring3_syscall(),
        "RING3_RT" => trigger_ring3_syscall(), // Use same implementation for round-trip
        "DF"   => trigger_double_fault(),
        _      => {
            serial::write_str("[selftest] unknown tag\n");
            loop { crate::arch::x86_64::cpu::halt(); }
        }
    }
}

/// Run in sequence; useful for quick manual testing. First one halts the CPU.
pub fn run_sequence() -> ! {
    // Env-driven selection (via RUSTFLAGS --cfg selftest_TAG).
    #[cfg(selftest_DIV0)] { return run_one("DIV0"); }
    #[cfg(selftest_GP)]   { return run_one("GP"); }
    #[cfg(selftest_PF)]   { return run_one("PF"); }
    #[cfg(selftest_SYSCALL)] { return run_one("SYSCALL"); }
    #[cfg(selftest_TIMER)]   { return run_one("TIMER"); }
    #[cfg(selftest_RING3)]   { return run_one("RING3"); }
    #[cfg(selftest_RING3_RT)] { return run_one("RING3_RT"); }
    // Default if none specified:
    run_one("DF")
}