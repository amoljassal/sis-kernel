//! Interrupt Descriptor Table (IDT) setup and handlers.
//!
//! The IDT dispatches exceptions, hardware interrupts and system
//! calls to appropriate handlers.  We install handlers for common
//! CPU exceptions, the PIT timer and the syscall vector (0x80).

use crate::arch::x86_64::{gdt, pit, interrupts};
use crate::kernel::{scheduler, syscall, serial};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, InterruptStackFrameValue, HandlerFunc};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Exceptions
        idt.divide_error.set_handler_fn(divide_by_zero);
        idt.page_fault.set_handler_fn(page_fault);
        idt.general_protection_fault.set_handler_fn(gp_fault);
        idt.double_fault.set_handler_fn(double_fault)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        // Timer interrupt (IRQ0 mapped to 32)
        idt[32].set_handler_fn(timer_interrupt);
        // Syscall (vector 0x80) uses user privilege level 3
        idt[0x80].set_handler_fn(syscall_handler)
            .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn divide_by_zero(stack_frame: &mut InterruptStackFrame) {
    serial::write_str("[exception] Divide by zero\n");
    serial::write_str("Halting\n");
    loop { interrupts::disable(); crate::arch::x86_64::cpu::halt(); }
}

extern "x86-interrupt" fn gp_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    serial::write_str("[exception] General protection fault\n");
    loop { interrupts::disable(); crate::arch::x86_64::cpu::halt(); }
}

extern "x86-interrupt" fn double_fault(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    serial::write_str("[exception] Double fault\n");
    loop { interrupts::disable(); crate::arch::x86_64::cpu::halt(); }
}

extern "x86-interrupt" fn page_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    use x86_64::registers::control::Cr2;
    let addr = Cr2::read();
    serial::write_str("[exception] Page fault at ");
    // For simplicity we do not print the address; real code would convert
    // the address to hex.
    let _ = addr;
    loop { interrupts::disable(); crate::arch::x86_64::cpu::halt(); }
}

/// Timer interrupt handler.  Acknowledge the PIC, advance the
/// scheduler and return.
extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
    // Acknowledge the PIC
    pit::ack();
    // Advance the scheduler and perform a context switch
    scheduler::tick();
}

/// System call handler.  Dispatch to the syscall module and
/// automatically restore registers on return.  Syscalls originate
/// from user space (ring3) and return via `iretq`.
extern "x86-interrupt" fn syscall_handler(_stack_frame: &mut InterruptStackFrame) {
    syscall::dispatch();
}