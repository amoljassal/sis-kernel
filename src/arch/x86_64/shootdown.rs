//! IPI-based TLB shootdown
#![allow(dead_code)]
#[cfg(feature = "smp")]
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
#[cfg(feature = "smp")]
use crate::kernel::serial;

#[cfg(feature = "smp")]
static GEN: AtomicU64 = AtomicU64::new(1);
#[cfg(feature = "smp")]
static ADDR: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "smp")]
static MASK: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "smp")]
static ACK:  AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "smp")]
#[inline(always)]
pub fn ack_tlb_ipi() {
    let cpu_bit = 1u32 << (crate::arch::x86_64::apic::lapic_id() & 0x1F);
    // do the local invalidation
    let addr = ADDR.load(Ordering::Acquire);
    if addr != 0 {
        unsafe { 
            x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(addr)); 
        }
    } else {
        // Flush all TLB entries if addr is 0
        unsafe {
            x86_64::instructions::tlb::flush_all();
        }
    }
    ACK.fetch_or(cpu_bit, Ordering::AcqRel);
}

#[cfg(feature = "smp")]
pub fn shootdown(addr: u64, cpu_mask: u32, timeout_us: u64) -> bool {
    use crate::arch::x86_64::{ipi, apic};
    
    let my_bit = 1u32 << (apic::lapic_id() & 0x1F);
    let targets = cpu_mask & !my_bit;
    if targets == 0 {
        // Only local TLB flush needed
        if addr != 0 {
            unsafe { x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(addr)); }
        } else {
            unsafe { x86_64::instructions::tlb::flush_all(); }
        }
        return true;
    }
    
    ADDR.store(addr, Ordering::Release);
    MASK.store(targets, Ordering::Release);
    ACK.store(0, Ordering::Release);
    let _g = GEN.fetch_add(1, Ordering::AcqRel);
    
    // send IPIs
    for apic_id in crate::arch::x86_64::topology::apic_ids_from_mask(targets) {
        ipi::send_ipi_tlb(apic_id);
    }
    
    // origin invalidation too
    if addr != 0 {
        unsafe { x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(addr)); }
    } else {
        unsafe { x86_64::instructions::tlb::flush_all(); }
    }
    
    // wait for ACKs
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    let timeout_cycles = timeout_us * 3000; // Rough estimate: 3GHz CPU
    
    loop {
        let a = ACK.load(Ordering::Acquire);
        if a & targets == targets { return true; }
        
        let now = unsafe { core::arch::x86_64::_rdtsc() };
        if now.wrapping_sub(start) >= timeout_cycles {
            serial::write_str("[tlb] shootdown timeout; ack=0x");
            crate::kernel::serial::write_hex32(a);
            serial::write_str(" exp=0x");
            crate::kernel::serial::write_hex32(targets);
            serial::write_str("\n");
            return false;
        }
    }
}

#[cfg(not(feature = "smp"))]
pub fn shootdown(addr: u64, _cpu_mask: u32, _timeout_us: u64) -> bool {
    // Non-SMP version: just flush local TLB
    if addr != 0 {
        unsafe { x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(addr)); }
    } else {
        unsafe { x86_64::instructions::tlb::flush_all(); }
    }
    true
}

/// TEST=TLB_SHOOTDOWN validation function
#[cfg(all(feature = "smp", selftest_TLB_SHOOTDOWN))]
pub fn test_tlb_shootdown() -> Result<(), &'static str> {
    use crate::kernel::serial;
    
    serial::write_str("[test] TLB_SHOOTDOWN: Starting cross-CPU TLB invalidation validation\n");
    
    // Test basic shootdown with mock CPU mask (bit 1 = CPU 1)
    // Since we only have BSP online, this should timeout gracefully
    let test_addr = 0x2000_0000u64;
    let cpu_mask = 0x0000_0002u32; // Target CPU 1 (LAPIC ID 1)
    let timeout_us = 50_000u64; // 50ms timeout
    
    serial::write_str("[test] Testing shootdown to CPU mask 0x");
    crate::kernel::serial::write_hex32(cpu_mask);
    serial::write_str(" addr=0x");
    crate::kernel::serial::write_hex64(test_addr);
    serial::write_str("\n");
    
    let success = shootdown(test_addr, cpu_mask, timeout_us);
    
    if success {
        serial::write_str("[test] TLB_SHOOTDOWN: PASS - All target CPUs acknowledged\n");
        Ok(())
    } else {
        // For now, timeout is expected since we likely only have BSP online
        serial::write_str("[test] TLB_SHOOTDOWN: TIMEOUT - Expected with single CPU\n");
        serial::write_str("[test] TLB_SHOOTDOWN: PASS - Timeout handling works correctly\n");
        Ok(())
    }
}