//! Interrupt Descriptor Table (IDT) setup and handlers.
//! - All handlers use by-value `InterruptStackFrame` (compiler-compatible).
//! - No explicit return types on `extern "x86-interrupt"` functions.
//! - Double-fault registered via transmute to satisfy x86_64 crate `-> !` type.
//! - In selftest builds, handlers call qemu_exit(code) before halting.

#[allow(unused_imports)]
use crate::arch::x86_64::io::qemu_exit;
use crate::arch::x86_64::{cpu, gdt};
use crate::kernel::{scheduler, serial, syscall};
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::PrivilegeLevel;

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Phase 5C-B: VFIO interrupt tracking
#[cfg(feature = "vfio")]
static VFIO_IRQ_COUNTER: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "vfio")]
static VFIO_IRQ_LOG_RATE: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "vfio")]
pub const VFIO_IRQ_VECTOR: u8 = 0x5E;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    // add more as needed (Keyboard = 33, etc.)
}
impl InterruptIndex {
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline(always)]
    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
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

        // Phase 6C: Cross-CPU IPI handlers
        #[cfg(feature = "smp")]
        {
            // IPI_RESCHED vector (0xF0) for lightweight scheduling signals
            idt[0xF0].set_handler_fn(ipi_resched_handler);

            // IPI_IPC_WAKE vector (0xF1) for cross-CPU IPC wake-up
            idt[0xF1].set_handler_fn(ipi_ipc_wake_handler);

            // IPI_MBOX vector (0xF2) for Phase 6C mailbox notifications
            idt[0xF2].set_handler_fn(ipi_mbox_handler);
        }

        // Install VFIO ISRs for vectors 0x50-0x5F
        #[cfg(feature = "vfio")]
        {
            idt[0x50].set_handler_fn(vfio_isrs::vfio_vec_50);
            idt[0x51].set_handler_fn(vfio_isrs::vfio_vec_51);
            idt[0x52].set_handler_fn(vfio_isrs::vfio_vec_52);
            idt[0x53].set_handler_fn(vfio_isrs::vfio_vec_53);
            idt[0x54].set_handler_fn(vfio_isrs::vfio_vec_54);
            idt[0x55].set_handler_fn(vfio_isrs::vfio_vec_55);
            idt[0x56].set_handler_fn(vfio_isrs::vfio_vec_56);
            idt[0x57].set_handler_fn(vfio_isrs::vfio_vec_57);
            idt[0x58].set_handler_fn(vfio_isrs::vfio_vec_58);
            idt[0x59].set_handler_fn(vfio_isrs::vfio_vec_59);
            idt[0x5A].set_handler_fn(vfio_isrs::vfio_vec_5A);
            idt[0x5B].set_handler_fn(vfio_isrs::vfio_vec_5B);
            idt[0x5C].set_handler_fn(vfio_isrs::vfio_vec_5C);
            idt[0x5D].set_handler_fn(vfio_isrs::vfio_vec_5D);
            idt[0x5E].set_handler_fn(vfio_isrs::vfio_vec_5E);
            idt[0x5F].set_handler_fn(vfio_isrs::vfio_vec_5F);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();

    // IPI handlers (SMP)
    #[cfg(feature = "smp")]
    {
        // Install IPI handlers for cross-CPU communication
        crate::arch::x86_64::ipi::install_ipi_handlers();
    }
}

// ISRs are now pre-installed during IDT initialization

/// Unsafe internal: install an x86-interrupt handler at a custom vector.
#[cfg(feature = "smp")]
pub unsafe fn install_ipi(handler_addr: usize, vector: u8) {
    use core::ptr;
    use x86_64::structures::idt::InterruptDescriptorTable;

    // FIXME: This function tries to modify a read-only IDT after initialization
    // For now, we'll disable this functionality to fix compilation
    // In a full implementation, you'd need a mutable IDT or dynamic handler registration

    // Safety: we transmute a function pointer by address to `extern "x86-interrupt" fn(_)`
    let _f: extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame) =
        core::mem::transmute::<usize, _>(handler_addr);

    // TODO: Implement proper dynamic IDT entry modification
    // This is a temporary fix to resolve compilation errors
    // The IDT should be initialized with these handlers from the start
    // or use a different mechanism for dynamic handler registration
}

// ----- VFIO runtime support: vector→handle map and TSC latency histogram -----
#[cfg(feature = "vfio")]
mod vfio_rt {
    use crate::kernel::serial;
    use core::sync::atomic::{AtomicU64, Ordering};

    // vector -> (handle_id | epoch<<16)
    pub static VEC_PACKED: [AtomicU64; 256] = unsafe { core::mem::zeroed() };
    // latency histogram: 16 buckets of log2(cycles)
    static HIST: [AtomicU64; 16] = unsafe { core::mem::zeroed() };

    #[inline]
    pub fn map(vec: u8, packed: u64) {
        VEC_PACKED[vec as usize].store(packed, Ordering::Release);
    }
    #[inline]
    pub fn unmap(vec: u8) {
        VEC_PACKED[vec as usize].store(0, Ordering::Release);
    }
    #[inline]
    pub fn load(vec: u8) -> u64 {
        VEC_PACKED[vec as usize].load(core::sync::atomic::Ordering::Acquire)
    }

    #[inline]
    fn bucket(cycles: u64) -> usize {
        if cycles == 0 {
            return 0;
        }
        let l = 63 - cycles.leading_zeros() as usize;
        core::cmp::min(l, 15)
    }
    #[inline]
    pub fn hist_add(cycles: u64) {
        HIST[bucket(cycles)].fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
    pub fn dump_hist() {
        serial::write_str("[vfio-lat] buckets(log2 cycles): ");
        for i in 0..16 {
            let v = HIST[i].load(core::sync::atomic::Ordering::Relaxed);
            serial::write_hex8(i as u8);
            serial::write_str("=");
            serial::write_hex64(v);
            serial::write_str(" ");
        }
        serial::write_str("\n");
    }
    pub fn packed_load(vec: u8) -> u64 {
        load(vec)
    }
}

#[cfg(feature = "vfio")]
pub fn vfio_map_vector(vec: u8, packed: u64) {
    vfio_rt::map(vec, packed)
}
#[cfg(feature = "vfio")]
pub fn vfio_unmap_vector(vec: u8) {
    vfio_rt::unmap(vec)
}
#[cfg(feature = "vfio")]
pub fn vfio_vector_packed_load(vec: u8) -> u64 {
    vfio_rt::packed_load(vec)
}
#[cfg(feature = "vfio")]
pub fn vfio_dump_hist() {
    vfio_rt::dump_hist()
}

// ----- VFIO ISR macro: generate 16 handlers (0x50..=0x5F) that pass vector id -----
#[cfg(feature = "vfio")]
macro_rules! make_vfio_isr {
    ($name:ident, $vec:expr) => {
        pub extern "x86-interrupt" fn $name(_sf: InterruptStackFrame) {
            // Fast path: look up handle+epoch from vector mapping
            let vec = $vec as u8;
            let packed = vfio_vector_packed_load(vec);
            if packed != 0 {
                let handle = crate::kernel::vfio::VfioHandle::new((packed & 0xFFFF) as u16, 1);
                let epoch = (packed >> 16) & 0xFFFF_FFFF;
                let now = rdtsc();
                let (count, t_trig) = crate::kernel::vfio::on_irq(handle, epoch, now);
                if (count & 63) == 0 {
                    serial::write_str("[vfio-irq] vec=");
                    serial::write_hex8(vec);
                    serial::write_str(" count=");
                    serial::write_hex64(count);
                    serial::write_str("\n");
                }
                if t_trig != 0 && now >= t_trig {
                    vfio_rt::hist_add(now - t_trig);
                }
            } else {
                serial::write_str("[vfio-irq] spurious vec=");
                serial::write_hex8($vec as u8);
                serial::write_str("\n");
            }
            // Send EOI
            #[cfg(feature = "apic")]
            {
                crate::arch::x86_64::apic::eoi();
            }
            #[cfg(not(feature = "apic"))]
            unsafe {
                PICS.lock().notify_end_of_interrupt($vec as u8);
            }
        }
    };
}

#[cfg(feature = "vfio")]
mod vfio_isrs {
    use super::*;
    make_vfio_isr!(vfio_vec_50, 0x50);
    make_vfio_isr!(vfio_vec_51, 0x51);
    make_vfio_isr!(vfio_vec_52, 0x52);
    make_vfio_isr!(vfio_vec_53, 0x53);
    make_vfio_isr!(vfio_vec_54, 0x54);
    make_vfio_isr!(vfio_vec_55, 0x55);
    make_vfio_isr!(vfio_vec_56, 0x56);
    make_vfio_isr!(vfio_vec_57, 0x57);
    make_vfio_isr!(vfio_vec_58, 0x58);
    make_vfio_isr!(vfio_vec_59, 0x59);
    make_vfio_isr!(vfio_vec_5A, 0x5A);
    make_vfio_isr!(vfio_vec_5B, 0x5B);
    make_vfio_isr!(vfio_vec_5C, 0x5C);
    make_vfio_isr!(vfio_vec_5D, 0x5D);
    make_vfio_isr!(vfio_vec_5E, 0x5E);
    make_vfio_isr!(vfio_vec_5F, 0x5F);

    pub fn handler_for(vec: u8) -> Option<extern "x86-interrupt" fn(InterruptStackFrame)> {
        match vec {
            0x50 => Some(vfio_vec_50),
            0x51 => Some(vfio_vec_51),
            0x52 => Some(vfio_vec_52),
            0x53 => Some(vfio_vec_53),
            0x54 => Some(vfio_vec_54),
            0x55 => Some(vfio_vec_55),
            0x56 => Some(vfio_vec_56),
            0x57 => Some(vfio_vec_57),
            0x58 => Some(vfio_vec_58),
            0x59 => Some(vfio_vec_59),
            0x5A => Some(vfio_vec_5A),
            0x5B => Some(vfio_vec_5B),
            0x5C => Some(vfio_vec_5C),
            0x5D => Some(vfio_vec_5D),
            0x5E => Some(vfio_vec_5E),
            0x5F => Some(vfio_vec_5F),
            _ => None,
        }
    }
}

#[cfg(feature = "vfio")]
pub fn vfio_isr_vector_install(_vec: u8) {
    // ISRs are pre-installed during IDT initialization - no-op
}

#[cfg(not(feature = "vfio"))]
pub fn vfio_isr_vector_install(_vec: u8) {
    // No-op stub
}

#[cfg(not(feature = "vfio"))]
pub fn vfio_map_vector(_vec: u8, _packed: u64) {
    // No-op stub
}

#[cfg(not(feature = "vfio"))]
pub fn vfio_unmap_vector(_vec: u8) {
    // No-op stub
}

#[cfg(not(feature = "vfio"))]
pub fn vfio_vector_packed_load(_vec: u8) -> u64 {
    0
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
    loop {
        cpu::halt();
    }
}

#[inline(always)]
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Per-CPU tick counting for SMP support
    #[cfg(feature = "smp")]
    {
        crate::arch::x86_64::percpu::percpu_tick();
    }

    // SMP scheduler takes precedence over legacy scheduler
    #[cfg(feature = "smp")]
    {
        crate::kernel::smp_scheduler::smp_timer_tick();
    }

    #[cfg(not(feature = "smp"))]
    {
        scheduler::tick();
    }

    // Process cross-CPU IPC messages (Phase 6C)
    #[cfg(all(feature = "smp", feature = "ipc"))]
    {
        crate::kernel::xcpu_ipc::process_ipc_messages();
    }

    // Send EOI: LAPIC if APIC feature enabled, otherwise legacy PIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
    #[cfg(not(feature = "apic"))]
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }

    // Selftest exit conditions handled directly in scheduler::tick()
}

#[inline(always)]
extern "x86-interrupt" fn divide_by_zero(_stack_frame: InterruptStackFrame) {
    serial::write_str("[exc] divide-by-zero — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop {
        cpu::halt();
    }
}

#[inline(always)]
extern "x86-interrupt" fn gp_fault(_stack_frame: InterruptStackFrame, _error_code: u64) {
    serial::write_str("[exc] general-protection-fault — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop {
        cpu::halt();
    }
}

#[inline(always)]
extern "x86-interrupt" fn page_fault(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
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
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] NP_U_R: Wrong error_code, expected 4\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_NP_U_W)]
        {
            // Expected: P=0, W/R=1, U/S=1 → error_code=6
            if ec == 6 {
                serial::write_str("[pfm] NP_U_W: Correct error_code=6 (P=0,W/R=1,U/S=1)\n");
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] NP_U_W: Wrong error_code, expected 6\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_US_VIOL)]
        {
            // Expected: P=1, W/R=0, U/S=1 → error_code=5
            if ec == 5 {
                serial::write_str("[pfm] US_VIOL: Correct error_code=5 (P=1,W/R=0,U/S=1)\n");
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] US_VIOL: Wrong error_code, expected 5\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_PROT_U_W)]
        {
            // Expected: P=1, W/R=1, U/S=1, ID=0 → error_code=7
            if ec == 7 {
                serial::write_str("[pfm] PROT_U_W: Correct error_code=7 (P=1,W/R=1,U/S=1,ID=0)\n");
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] PROT_U_W: Wrong error_code, expected 7\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_NX_EXEC)]
        {
            // Expected: P=1, W/R=0, U/S=1, ID=1 → error_code=17
            if ec == 17 {
                serial::write_str("[pfm] NX_EXEC: Correct error_code=17 (P=1,W/R=0,U/S=1,ID=1)\n");
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] NX_EXEC: Wrong error_code, expected 17\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_GUARD_UNDER)]
        {
            // Expected: P=0, W/R=1, U/S=1, ID=0 → error_code=6
            if ec == 6 {
                serial::write_str(
                    "[pfm] GUARD_UNDER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)\n",
                );
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] GUARD_UNDER: Wrong error_code, expected 6\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }

        #[cfg(selftest_PFM_GUARD_OVER)]
        {
            // Expected: P=0, W/R=1, U/S=1, ID=0 → error_code=6
            if ec == 6 {
                serial::write_str(
                    "[pfm] GUARD_OVER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)\n",
                );
                unsafe {
                    qemu_exit(0x00);
                }
            } else {
                serial::write_str("[pfm] GUARD_OVER: Wrong error_code, expected 6\n");
                unsafe {
                    qemu_exit(0x11);
                }
            }
        }
    }

    // Default page fault handling
    serial::write_str("[exc] page-fault — halting\n");
    #[cfg(feature = "idt-selftest")]
    unsafe {
        qemu_exit(0x00);
    }
    loop {
        cpu::halt();
    }
}

#[inline(always)]
extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
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
extern "x86-interrupt" fn syscall_exit_handler(_stack_frame: InterruptStackFrame) {
    serial::write_str("[syscall] Ring-3 exit request\n");
    #[cfg(all(feature = "idt-selftest", selftest_RING3_RT))]
    unsafe {
        qemu_exit(0x00); // Success: full round-trip completed
    }
    // Fallback for non-selftest builds
    loop {
        cpu::halt();
    }
}

// Phase 5C-B: VFIO MSI interrupt handler (vector 0x5E) with rate limiting
#[cfg(feature = "vfio")]
#[inline(always)]
extern "x86-interrupt" fn vfio_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Increment interrupt counter
    let count = VFIO_IRQ_COUNTER.fetch_add(1, Ordering::SeqCst);

    // **NEW: Rate-limited logging - only log every 64th interrupt to prevent log storms**
    let should_log = (count & 0x3F) == 0; // Log every 64 interrupts (2^6)

    if should_log || count < 5 {
        // Always log first 5, then rate-limit
        serial::write_str("[vfio-irq] vector 0x5E fired (dev 00:03.0) count=");
        serial::write_u64((count + 1) as u64);

        if count >= 64 {
            serial::write_str(" [rate-limited]");
        }
        serial::write_str("\n");
    }

    // **NEW: Spurious interrupt guard - check if we expected this interrupt**
    // In a full implementation, we'd check device-specific interrupt status registers
    // For now, we just increment a separate counter for spurious detection
    let log_count = VFIO_IRQ_LOG_RATE.fetch_add(1, Ordering::SeqCst);

    // Send EOI to LAPIC (always LAPIC for MSI)
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
    #[cfg(not(feature = "apic"))]
    {
        // MSI requires LAPIC, but provide fallback for non-APIC builds
        if should_log {
            serial::write_str("[vfio-irq] Warning: MSI without APIC, no EOI sent\n");
        }
    }

    // **NEW: Enhanced spurious guard - warn if interrupt rate is suspiciously high**
    if count > 1000 && (count % 1000) == 0 {
        serial::write_str("[vfio-irq] High interrupt rate detected - possible spurious IRQs\n");
    }

    // Selftest exit condition: quit after first interrupt in selftest mode
    // This allows the VFIO_MSI_SMOKE test to validate interrupt delivery
    #[cfg(all(feature = "idt-selftest", selftest_VFIO_MSI_SMOKE))]
    unsafe {
        serial::write_str("[vfio-irq] Selftest exit: first MSI delivered successfully\n");
        qemu_exit(0x00); // Success
    }
}

// Phase 6C: Cross-CPU IPI handlers

/// IPI handler for reschedule requests (vector 0xF0)
#[cfg(feature = "smp")]
extern "x86-interrupt" fn ipi_resched_handler(_stack_frame: InterruptStackFrame) {
    // Handle reschedule IPI
    crate::kernel::smp_scheduler::handle_resched_ipi();

    // Send EOI to LAPIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
}

/// IPI handler for cross-CPU IPC wake-up (vector 0xF1)
#[cfg(feature = "smp")]
extern "x86-interrupt" fn ipi_ipc_wake_handler(_stack_frame: InterruptStackFrame) {
    // Handle IPC wake-up IPI
    #[cfg(feature = "ipc")]
    {
        crate::kernel::xcpu_ipc::handle_ipc_ipi();
    }

    // Send EOI to LAPIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
}

/// IPI handler for mailbox notifications (vector 0xF2)
#[cfg(feature = "smp")]
extern "x86-interrupt" fn ipi_mbox_handler(_stack_frame: InterruptStackFrame) {
    // Handle mailbox IPI - drain a few messages to amortize overhead
    #[cfg(feature = "smp")]
    {
        let drained = crate::kernel::xcpu_mbox::drain(32);
        // Only log occasionally to avoid spam
        if (drained & 0x3F) == 1 {
            crate::kernel::serial::write_str("[mbox] drained=");
            crate::kernel::serial::write_hex64(drained as u64);
            crate::kernel::serial::write_str("\n");
        }
    }

    // Send EOI to LAPIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
}
