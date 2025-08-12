//! IPI vectors and helpers (feature = "smp")
#![allow(dead_code)]
#[cfg(feature = "smp")]
use crate::kernel::serial;

// Chosen high vectors (not overlapping exceptions or timer)
pub const IPI_RESCHED: u8 = 0xF0;
pub const IPI_TLB: u8      = 0xF1;

#[cfg(feature = "smp")]
pub fn install_ipi_handlers() {
    use crate::arch::x86_64::idt;
    unsafe {
        idt::install_ipi(ISR_resched as usize, IPI_RESCHED);
        idt::install_ipi(ISR_tlb as usize, IPI_TLB);
    }
    serial::write_str("[ipi] handlers installed\n");
}

#[cfg(feature = "smp")]
extern "x86-interrupt" fn ISR_resched(_sf: x86_64::structures::idt::InterruptStackFrame) {
    // Minimal: just mark need_resched; scheduler tick will switch
    crate::arch::x86_64::percpu_clean::this().need_resched.store(true, core::sync::atomic::Ordering::Release);
    
    // Send EOI to LAPIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
}

#[cfg(feature = "smp")]
extern "x86-interrupt" fn ISR_tlb(_sf: x86_64::structures::idt::InterruptStackFrame) {
    crate::arch::x86_64::shootdown::ack_tlb_ipi();
    
    // Send EOI to LAPIC
    #[cfg(feature = "apic")]
    {
        crate::arch::x86_64::apic::eoi();
    }
}

#[cfg(feature = "smp")]
pub fn send_ipi_resched(apic_id: u32) {
    crate::arch::x86_64::apic::send_ipi(apic_id, IPI_RESCHED);
}

#[cfg(feature = "smp")]
pub fn send_ipi_tlb(apic_id: u32) {
    crate::arch::x86_64::apic::send_ipi(apic_id, IPI_TLB);
}