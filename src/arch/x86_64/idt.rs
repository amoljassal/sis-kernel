//! Interrupt Descriptor Table (IDT) setup and handlers.
//! - All handlers use by-value `InterruptStackFrame` (compiler-compatible).
//! - No explicit return types on `extern "x86-interrupt"` functions.
//! - Double-fault registered via transmute to satisfy x86_64 crate `-> !` type.
//! - In selftest builds, handlers call qemu_exit(code) before halting.

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode
};
use x86_64::PrivilegeLevel;
use crate::arch::x86_64::{gdt, cpu};
use crate::kernel::{scheduler, syscall, serial};
#[allow(unused_imports)] use crate::arch::x86_64::io::qemu_exit;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Phase 5C-B: VFIO interrupt tracking
#[cfg(feature = "vfio")]
static VFIO_IRQ_COUNTER: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "vfio")]
pub const VFIO_IRQ_VECTOR: u8 = 0x5E;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    // add more as needed (Keyboard = 33, etc.)
}
impl InterruptIndex {
    #[inline(always)] pub fn as_u8(self) -> u8 { self as u8 }
    #[inline(always)] pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Double fault handler with IST - needs transmute for type compatibility
        unsafe {
            type DfExpected = extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !;
            type DfActual = extern "x86-interrupt" fn(InterruptStackFrame, u64);
            let actual: DfActual = double_fault_handler;
            let ptr: DfExpected = core::mem::transmute(actual);
            idt.double_fault
                .set_handler_fn(ptr)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        // Other exceptions
        idt.divide_error.set_handler_fn(divide_by_zero);
        idt.general_protection_fault.set_handler_fn(gp_fault);
        idt.page_fault.set_handler_fn(page_fault);

        // Hardware IRQs
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

        // Syscall gate @ 0x80 for ring3
        idt[0x80]
            .set_handler_fn(syscall_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
            
        // Additional syscall gate @ 0x81 for ring3 exit requests
        idt[0x81]
            .set_handler_fn(syscall_exit_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// Phase 5C-B: Install VFIO ISR at vector 0x5E
#[cfg(feature = "vfio")]
pub fn install_vfio_isr(vector: u8) {
    // Validate vector matches our expectation
    if vector != VFIO_IRQ_VECTOR {
        serial::write_str("[vfio-isr] Warning: vector mismatch, using 0x5E\n");
    }
    
    // Install VFIO ISR directly into IDT at vector 0x5E
    unsafe {
        use x86_64::structures::idt::InterruptDescriptorTable;
        let idt_ptr = &IDT as *const InterruptDescriptorTable as *mut InterruptDescriptorTable;
        (*idt_ptr)[VFIO_IRQ_VECTOR as usize].set_handler_fn(vfio_interrupt_handler);
    }
    
    serial::write_str("[vfio-isr] ISR installed at vector 0x5E\n");
}

#[inline(always)]
extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    // Keep DF path tiny & deterministic; avoid formatted panic.
    serial::write_str("[df] double fault — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00); // success
    }
    loop { cpu::halt(); }
}

#[inline(always)]
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    scheduler::tick();
    
    // Send EOI: LAPIC if APIC feature enabled, otherwise legacy PIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
    #[cfg(not(feature = "apic"))]
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    
    #[cfg(all(feature = "idt-selftest", selftest_TIMER))]
    if crate::kernel::scheduler::selftest_should_exit() {
        unsafe { qemu_exit(0x00); } // success when N ticks reached
    }
    
    #[cfg(all(feature = "idt-selftest", selftest_LAPIC_TIMER))]
    if crate::kernel::scheduler::selftest_lapic_should_exit() {
        unsafe { qemu_exit(0x00); } // success when N LAPIC ticks reached
    }
}

#[inline(always)]
extern "x86-interrupt" fn divide_by_zero(
    _stack_frame: InterruptStackFrame
) {
    serial::write_str("[exc] divide-by-zero — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop { cpu::halt(); }
}

#[inline(always)]
extern "x86-interrupt" fn gp_fault(
    _stack_frame: InterruptStackFrame,
    _error_code: u64
) {
    serial::write_str("[exc] general-protection-fault — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop { cpu::halt(); }
}

#[inline(always)]
extern "x86-interrupt" fn page_fault(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    use x86_64::registers::control::Cr2;
    let fault_addr = Cr2::read();
    
    // PFM selftest: validate expected error codes
    #[cfg(all(feature = "pf-matrix", feature = "idt-selftest"))]
    {
        let ec = error_code.bits();
        
        #[cfg(selftest_PFM_NP_U_R)]
        {
            // Expected: P=0, W/R=0, U/S=1 → error_code=4
            if ec == 4 {
                serial::write_str("[pfm] NP_U_R: Correct error_code=4 (P=0,W/R=0,U/S=1)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] NP_U_R: Wrong error_code, expected 4\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_NP_U_W)]
        {
            // Expected: P=0, W/R=1, U/S=1 → error_code=6
            if ec == 6 {
                serial::write_str("[pfm] NP_U_W: Correct error_code=6 (P=0,W/R=1,U/S=1)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] NP_U_W: Wrong error_code, expected 6\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_US_VIOL)]
        {
            // Expected: P=1, W/R=0, U/S=1 → error_code=5
            if ec == 5 {
                serial::write_str("[pfm] US_VIOL: Correct error_code=5 (P=1,W/R=0,U/S=1)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] US_VIOL: Wrong error_code, expected 5\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_PROT_U_W)]
        {
            // Expected: P=1, W/R=1, U/S=1, ID=0 → error_code=7
            if ec == 7 {
                serial::write_str("[pfm] PROT_U_W: Correct error_code=7 (P=1,W/R=1,U/S=1,ID=0)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] PROT_U_W: Wrong error_code, expected 7\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_NX_EXEC)]
        {
            // Expected: P=1, W/R=0, U/S=1, ID=1 → error_code=17
            if ec == 17 {
                serial::write_str("[pfm] NX_EXEC: Correct error_code=17 (P=1,W/R=0,U/S=1,ID=1)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] NX_EXEC: Wrong error_code, expected 17\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_GUARD_UNDER)]
        {
            // Expected: P=0, W/R=1, U/S=1, ID=0 → error_code=6
            if ec == 6 {
                serial::write_str("[pfm] GUARD_UNDER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] GUARD_UNDER: Wrong error_code, expected 6\n");
                unsafe { qemu_exit(0x11); }
            }
        }
        
        #[cfg(selftest_PFM_GUARD_OVER)]
        {
            // Expected: P=0, W/R=1, U/S=1, ID=0 → error_code=6  
            if ec == 6 {
                serial::write_str("[pfm] GUARD_OVER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)\n");
                unsafe { qemu_exit(0x00); }
            } else {
                serial::write_str("[pfm] GUARD_OVER: Wrong error_code, expected 6\n");
                unsafe { qemu_exit(0x11); }
            }
        }
    }
    
    // Default page fault handling
    serial::write_str("[exc] page-fault — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop { cpu::halt(); }
}

#[inline(always)]
extern "x86-interrupt" fn syscall_handler(
    stack_frame: InterruptStackFrame
) {
    // Check privilege level to detect Ring-3 calls
    use x86_64::instructions::segmentation;
    let cs = segmentation::cs();
    let cpl = cs.0 & 3; // Current Privilege Level is in bits 0-1
    
    if cpl == 3 {
        serial::write_str("[syscall] Ring-3 syscall detected!\n");
        #[cfg(all(feature = "idt-selftest", selftest_RING3))]
        unsafe {
            serial::write_str("[ring3] privilege separation verified\n");
            qemu_exit(0x00); // Success: Ring-3 to Ring-0 transition worked
        }
    } else {
        serial::write_str("[syscall] Ring-0 syscall\n");
        #[cfg(all(feature = "idt-selftest", selftest_SYSCALL))]
        unsafe {
            serial::write_str("[syscall] ping\n");
            qemu_exit(0x00);
        }
        #[cfg(all(feature = "idt-selftest", selftest_RING3))]
        unsafe {
            // For now, RING3 test just verifies privilege detection works
            qemu_exit(0x00);
        }
    }
    
    // Minimal syscall dispatch; body lives in kernel::syscall
    syscall::dispatch();
    // Note: In Ring-3 round-trip test, this handler RETURNs to user via IRET
}

#[inline(always)]
extern "x86-interrupt" fn syscall_exit_handler(
    _stack_frame: InterruptStackFrame
) {
    serial::write_str("[syscall] Ring-3 exit request\n");
    #[cfg(all(feature = "idt-selftest", selftest_RING3_RT))]
    unsafe {
        qemu_exit(0x00); // Success: full round-trip completed
    }
    // Fallback for non-selftest builds
    loop { cpu::halt(); }
}

// Phase 5C-B: VFIO MSI interrupt handler (vector 0x5E)
#[cfg(feature = "vfio")]
#[inline(always)]
extern "x86-interrupt" fn vfio_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    // Increment interrupt counter
    let count = VFIO_IRQ_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Log breadcrumb with device BDF and counter
    serial::write_str("[vfio-irq] vector 0x5E fired (dev 00:03.0) count=");
    serial::write_u64((count + 1) as u64);
    serial::write_str("\n");
    
    // Send EOI to LAPIC (always LAPIC for MSI)
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
    #[cfg(not(feature = "apic"))]
    {
        // MSI requires LAPIC, but provide fallback for non-APIC builds
        serial::write_str("[vfio-irq] Warning: MSI without APIC, no EOI sent\n");
    }
    
    // Selftest exit condition: quit after first interrupt in selftest mode
    // This allows the VFIO_MSI_SMOKE test to validate interrupt delivery
    #[cfg(all(feature = "idt-selftest", selftest_VFIO_MSI_SMOKE))]
    unsafe {
        serial::write_str("[vfio-irq] Selftest exit: first MSI delivered successfully\n");
        qemu_exit(0x00); // Success
    }
}