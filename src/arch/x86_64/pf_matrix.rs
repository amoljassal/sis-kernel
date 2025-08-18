//! Page-Fault Matrix (PFM) self-test framework.
//!
//! Tests systematic page-fault scenarios with precise error code validation:
//!
//! v1 cases:
//! - NP_U_R: Not-present user page read (error_code=4: P=0,W/R=0,U/S=1)
//! - NP_U_W: Not-present user page write (error_code=6: P=0,W/R=1,U/S=1)
//! - US_VIOL: User/supervisor violation (error_code=5: P=1,W/R=0,U/S=1)
//!
//! v2 cases (added):
//! - PROT_U_W: Write to present RO user page (error_code=7: P=1,W/R=1,U/S=1,ID=0)
//! - NX_EXEC: Execute from present NX user page (error_code=17: P=1,W/R=0,U/S=1,ID=1)
//! - GUARD_UNDER: Write into unmapped page below stack (error_code=6: P=0,W/R=1,U/S=1,ID=0)
//! - GUARD_OVER: Write into unmapped page above stack (error_code=6: P=0,W/R=1,U/S=1,ID=0)
//!
//! All tests execute from Ring-3 user mode to trigger real privilege violations.

use crate::arch::x86_64::{gdt, memory};
use crate::kernel::serial;
use x86_64::{structures::gdt::SegmentSelector, PhysAddr, VirtAddr};

/// Page-fault test case: Not-present user page read
#[cfg(all(feature = "pf-matrix", selftest_PFM_NP_U_R))]
pub fn test_np_u_r() -> ! {
    serial::write_str("[pfm] Test NP_U_R: Ring-3 read of unmapped user page\n");

    // Set up ring-3 execution context
    let user_addr = 0x600000u64; // Unmapped user virtual address

    // Create ring-3 trampoline that reads from unmapped page
    unsafe {
        trigger_ring3_fault(user_addr, false); // false = read operation
    }
}

/// Page-fault test case: Not-present user page write  
#[cfg(all(feature = "pf-matrix", selftest_PFM_NP_U_W))]
pub fn test_np_u_w() -> ! {
    serial::write_str("[pfm] Test NP_U_W: Ring-3 write to unmapped user page\n");

    // Set up ring-3 execution context
    let user_addr = 0x700000u64; // Unmapped user virtual address

    // Create ring-3 trampoline that writes to unmapped page
    unsafe {
        trigger_ring3_fault(user_addr, true); // true = write operation
    }
}

/// Page-fault test case: User/supervisor privilege violation
#[cfg(all(feature = "pf-matrix", selftest_PFM_US_VIOL))]
pub fn test_us_viol() -> ! {
    serial::write_str("[pfm] Test US_VIOL: Ring-3 read of supervisor page\n");

    // Map a supervisor-only page (present but U/S=0)
    let supervisor_addr = 0x800000u64;

    // Map page as supervisor-only (no U/S bit)
    if let Err(_) = memory::map_supervisor_page(VirtAddr::new(supervisor_addr)) {
        serial::write_str("[pfm] Failed to map supervisor page\n");
        unsafe {
            crate::arch::x86_64::io::qemu_exit(0x11);
        }
    }

    // Create ring-3 trampoline that reads from supervisor page
    unsafe {
        trigger_ring3_fault(supervisor_addr, false); // false = read operation
    }
}

/// Create Ring-3 execution context and trigger page fault
unsafe fn trigger_ring3_fault(fault_addr: u64, is_write: bool) -> ! {
    // Get GDT selectors for ring-3 segments
    let selectors = gdt::selectors();
    let user_cs = selectors.code_ring3.0 as u64;
    let user_ss = selectors.data_ring3.0 as u64;

    // Allocate user stack page
    let user_stack_top = match memory::map_user_page(VirtAddr::new(0x500000)) {
        Ok(page) => page.start_address().as_u64() + 4096,
        Err(_) => {
            serial::write_str("[pfm] Failed to map user stack\n");
            crate::arch::x86_64::io::qemu_exit(0x11);
        }
    };

    // Prepare IRET frame on current (kernel) stack
    core::arch::asm!(
        // Save current stack pointer
        "mov r11, rsp",

        // Set up IRET frame for ring-3
        "push {user_ss}",           // SS (ring-3 data segment)
        "push {user_stack}",        // RSP (user stack)
        "pushfq",                   // RFLAGS (enable interrupts)
        "push {user_cs}",           // CS (ring-3 code segment)
        "push 2f",                  // RIP (ring-3 code address)

        // Jump to ring-3
        "iretq",

        // Ring-3 code starts here (label 2)
        "2:",

        // Perform the fault-triggering operation
        "mov rax, {fault_addr}",

        // Branch based on operation type
        "cmp {is_write}, 0",
        "je 3f",

        // Write operation: store to [rax]
        "mov qword ptr [rax], 0x1234",
        "jmp 4f",

        // Read operation: load from [rax]
        "3:",
        "mov rbx, qword ptr [rax]",

        // Should never reach here due to page fault
        "4:",
        "ud2",  // Trigger #UD if we somehow get here

        user_ss = in(reg) user_ss,
        user_stack = in(reg) user_stack_top,
        user_cs = in(reg) user_cs,
        fault_addr = in(reg) fault_addr,
        is_write = in(reg) if is_write { 1u64 } else { 0u64 },
        options(noreturn)
    );
}

/// Entry point dispatcher for PFM tests based on compile-time cfg
pub fn run_test() -> ! {
    #[cfg(all(feature = "pf-matrix", selftest_PFM_NP_U_R))]
    test_np_u_r();

    #[cfg(all(feature = "pf-matrix", selftest_PFM_NP_U_W))]
    test_np_u_w();

    #[cfg(all(feature = "pf-matrix", selftest_PFM_US_VIOL))]
    test_us_viol();

    // v2 cases
    #[cfg(all(feature = "pf-matrix", selftest_PFM_PROT_U_W))]
    test_prot_u_w();

    #[cfg(all(feature = "pf-matrix", selftest_PFM_NX_EXEC))]
    test_nx_exec();

    #[cfg(all(feature = "pf-matrix", selftest_PFM_GUARD_UNDER))]
    test_guard_under();

    #[cfg(all(feature = "pf-matrix", selftest_PFM_GUARD_OVER))]
    test_guard_over();

    // Should never reach here
    serial::write_str("[pfm] ERROR: No PFM test selected\n");
    unsafe {
        crate::arch::x86_64::io::qemu_exit(0x11);
    }
}

// ===== PFM v2 constants and test cases =====

// v2 user test region (guarded stack & mappable pages)
const USER_REGION_BASE: u64 = 0x0000_7FFF_E000_0000;
const GUARD_UNDER_VA: u64 = USER_REGION_BASE + 0x0000;
const STACK_PAGE_VA: u64 = USER_REGION_BASE + 0x1000;
const GUARD_OVER_VA: u64 = USER_REGION_BASE + 0x2000;
const RO_PAGE_VA: u64 = USER_REGION_BASE + 0x3000;
const NX_PAGE_VA: u64 = USER_REGION_BASE + 0x4000;

/// Page-fault test case: Write to present read-only user page  
#[cfg(all(feature = "pf-matrix", selftest_PFM_PROT_U_W))]
pub fn test_prot_u_w() -> ! {
    serial::write_str("[pfm] Test PROT_U_W: Ring-3 write to present RO user page\n");

    // Map a RO user page, then try to write to it from Ring-3
    let paddr = memory::alloc_frame().expect("Failed to alloc frame for RO test");
    memory::map_user_ro_page(paddr, VirtAddr::new(RO_PAGE_VA)).expect("Failed to map RO page");

    // Create ring-3 trampoline that writes to RO page
    unsafe {
        trigger_ring3_fault(RO_PAGE_VA, true); // true = write operation
    }
}

/// Page-fault test case: Execute from present NX user page
#[cfg(all(feature = "pf-matrix", selftest_PFM_NX_EXEC))]
pub fn test_nx_exec() -> ! {
    serial::write_str("[pfm] Test NX_EXEC: Ring-3 execute from present NX user page\n");

    // Map a NX user page, then try to execute from it
    let paddr = memory::alloc_frame().expect("Failed to alloc frame for NX test");
    memory::map_user_nx_page(paddr, VirtAddr::new(NX_PAGE_VA)).expect("Failed to map NX page");

    // Create ring-3 trampoline that calls into NX page
    unsafe {
        trigger_ring3_exec_fault(NX_PAGE_VA);
    }
}

/// Page-fault test case: Write into unmapped guard page below stack
#[cfg(all(feature = "pf-matrix", selftest_PFM_GUARD_UNDER))]
pub fn test_guard_under() -> ! {
    serial::write_str("[pfm] Test GUARD_UNDER: Ring-3 write to unmapped page below stack\n");

    // Map only the stack page, leaving guard pages unmapped
    let stack_paddr = memory::alloc_frame().expect("Failed to alloc frame for stack");
    memory::map_user_rw_page(stack_paddr, VirtAddr::new(STACK_PAGE_VA))
        .expect("Failed to map stack page");

    // Create ring-3 trampoline that writes to guard page below stack
    unsafe {
        trigger_ring3_guard_fault(GUARD_UNDER_VA, STACK_PAGE_VA + 0x800);
    }
}

/// Page-fault test case: Write into unmapped guard page above stack  
#[cfg(all(feature = "pf-matrix", selftest_PFM_GUARD_OVER))]
pub fn test_guard_over() -> ! {
    serial::write_str("[pfm] Test GUARD_OVER: Ring-3 write to unmapped page above stack\n");

    // Map only the stack page, leaving guard pages unmapped
    let stack_paddr = memory::alloc_frame().expect("Failed to alloc frame for stack");
    memory::map_user_rw_page(stack_paddr, VirtAddr::new(STACK_PAGE_VA))
        .expect("Failed to map stack page");

    // Create ring-3 trampoline that writes to guard page above stack
    unsafe {
        trigger_ring3_guard_fault(GUARD_OVER_VA, STACK_PAGE_VA + 0xFF0);
    }
}

/// Create Ring-3 execution context and trigger execution fault (for NX testing)
unsafe fn trigger_ring3_exec_fault(exec_addr: u64) -> ! {
    // Get GDT selectors for ring-3 segments
    let selectors = gdt::selectors();
    let user_cs = selectors.code_ring3.0 as u64;
    let user_ss = selectors.data_ring3.0 as u64;

    // Allocate user stack page
    let user_stack_top = match memory::map_user_page(VirtAddr::new(0x500000)) {
        Ok(page) => page.start_address().as_u64() + 4096,
        Err(_) => {
            serial::write_str("[pfm] Failed to map user stack for exec test\n");
            crate::arch::x86_64::io::qemu_exit(0x11);
        }
    };

    // Prepare IRET frame on current (kernel) stack
    core::arch::asm!(
        // Save current stack pointer
        "mov r11, rsp",

        // Set up IRET frame for ring-3
        "push {user_ss}",           // SS (ring-3 data segment)
        "push {user_stack}",        // RSP (user stack)
        "pushfq",                   // RFLAGS (enable interrupts)
        "push {user_cs}",           // CS (ring-3 code segment)
        "push 2f",                  // RIP (ring-3 code address)

        // Jump to ring-3
        "iretq",

        // Ring-3 code starts here (label 2)
        "2:",

        // Try to call into the NX page (this should trigger #PF with ID=1)
        "mov rax, {exec_addr}",
        "call rax",                 // This will fault with ID=1 (instruction fetch)

        // Should never reach here due to page fault
        "ud2",  // Trigger #UD if we somehow get here

        user_ss = in(reg) user_ss,
        user_stack = in(reg) user_stack_top,
        user_cs = in(reg) user_cs,
        exec_addr = in(reg) exec_addr,
        options(noreturn)
    );
}

/// Create Ring-3 execution context and trigger guard page fault
unsafe fn trigger_ring3_guard_fault(guard_addr: u64, stack_ptr: u64) -> ! {
    // Get GDT selectors for ring-3 segments
    let selectors = gdt::selectors();
    let user_cs = selectors.code_ring3.0 as u64;
    let user_ss = selectors.data_ring3.0 as u64;

    // Prepare IRET frame on current (kernel) stack
    core::arch::asm!(
        // Save current stack pointer
        "mov r11, rsp",

        // Set up IRET frame for ring-3
        "push {user_ss}",           // SS (ring-3 data segment)
        "push {stack_ptr}",         // RSP (user stack at specified position)
        "pushfq",                   // RFLAGS (enable interrupts)
        "push {user_cs}",           // CS (ring-3 code segment)
        "push 2f",                  // RIP (ring-3 code address)

        // Jump to ring-3
        "iretq",

        // Ring-3 code starts here (label 2)
        "2:",

        // Write to guard page address (this should trigger #PF)
        "mov rax, {guard_addr}",
        "mov qword ptr [rax], 0x1234",  // This will fault

        // Should never reach here due to page fault
        "ud2",  // Trigger #UD if we somehow get here

        user_ss = in(reg) user_ss,
        stack_ptr = in(reg) stack_ptr,
        user_cs = in(reg) user_cs,
        guard_addr = in(reg) guard_addr,
        options(noreturn)
    );
}
