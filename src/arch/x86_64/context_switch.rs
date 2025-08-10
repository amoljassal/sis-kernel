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

#[cfg(feature = "per-task-mm")]
use crate::kernel::task::Task;
#[cfg(feature = "per-task-mm")]
use x86_64::registers::control::Cr3;

#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn switch_context(old: *mut TaskContext, new: *const TaskContext) {
    core::arch::naked_asm!(
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
        "0:"
    );
}

/// High-level context switch that handles both register context and CR3 switching.
/// This function combines register context switching with address space switching
/// for per-task memory management.
#[cfg(feature = "per-task-mm")]
pub unsafe fn switch_task_context(old_task: &mut Task, new_task: &Task) {
    // First switch the register context
    switch_context(&mut old_task.context, &new_task.context);
    
    // Then switch the address space (CR3) if the new task has its own
    if let Some(new_cr3) = new_task.cr3_root {
        let current_cr3 = Cr3::read().0;
        if current_cr3 != new_cr3 {
            Cr3::write(new_cr3, x86_64::registers::control::Cr3Flags::empty());
        }
    }
}

/// Fallback context switch for when per-task-mm is disabled.
/// Only switches register context, no CR3 switching.
#[cfg(not(feature = "per-task-mm"))]
pub unsafe fn switch_task_context(old_task: &mut crate::kernel::task::Task, new_task: &crate::kernel::task::Task) {
    switch_context(&mut old_task.context, &new_task.context);
}