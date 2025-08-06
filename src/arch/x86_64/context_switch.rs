//! Context switch implementation.
//!
//! Provides an assembly routine to switch from one task's context to
//! another.  The function saves callee‑saved registers of the
//! current task and restores those of the next task.  It is marked
//! `naked` to allow writing raw assembly without prologue/epilogue.
//!
//! Safety: This routine must only be called with valid pointers to
//! `TaskContext` structures.  Misuse will corrupt the stack and
//! registers.

use core::arch::asm;
use crate::kernel::task::TaskContext;

#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn switch_context(old: *mut TaskContext, new: *const TaskContext) {
    asm!(
        // Save callee‑saved registers into the old context
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",
        // Save current stack pointer and return address
        "mov [rdi + 0x38], rsp",
        "lea rax, [rip + 0f]",      // label to return after switch
        "mov [rdi + 0x30], rax",
        // Load new context into registers
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",
        "mov rsp, [rsi + 0x38]",
        "jmp qword ptr [rsi + 0x30]",
        // Return label
        "0:",
        options(noreturn)
    );
}