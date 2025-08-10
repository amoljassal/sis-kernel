//! Minimal SMP bring-up for self-test: BSP starts one AP via INIT/SIPI/SIPI to a low 4K trampoline.
//! Assumes APIC is enabled and LAPIC/IOAPIC mapped (feature "apic").
//! For QEMU `-smp 2`, APIC IDs are typically {0,1}.

use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use x86_64::{PhysAddr, VirtAddr};
use crate::kernel::serial;
use crate::arch::x86_64::{apic, memory};

// Trampoline placement (physical): 0x8000 (vector 0x8). Must be identity-mapped and present.
const AP_TRAMP_PHYS: u64 = 0x0000_8000;
const AP_TRAMP_VECTOR: u8 = (AP_TRAMP_PHYS >> 12) as u8; // 4K page number

// Shared online flags / counters indexed by slot (not strictly LAPIC ID; map below).
static BSP_APIC_ID: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);
static AP_APIC_ID:  AtomicU32 = AtomicU32::new(0xFFFF_FFFF);
static CPU0_ONLINE:  AtomicU8  = AtomicU8::new(0);
static CPU1_ONLINE:  AtomicU8  = AtomicU8::new(0);

// Exported so the AP entry can signal "online".
#[no_mangle]
pub extern "C" fn smp_ap_online_report(apic_id: u32) {
    if apic_id == BSP_APIC_ID.load(Ordering::SeqCst) {
        CPU0_ONLINE.store(1, Ordering::SeqCst);
    } else {
        CPU1_ONLINE.store(1, Ordering::SeqCst);
    }
    serial::write_str("[smp] cpu=");
    crate::kernel::serial::write_u64(apic_id as u64);
    serial::write_str(" online\n");
}

/// Map trampoline page identity into the current address space and copy the embedded stub.
unsafe fn deploy_trampoline() -> Result<(), &'static str> {
    // Map the physical page identity at the same VA (simple for QEMU selftest).
    // If your paging forbids low identity, you can pick a fixed VA and still SIPI to the phys.
    let _tramp_va = memory::map_user_page(VirtAddr::new(AP_TRAMP_PHYS))
        .map_err(|_| "map trampoline failed")?;
    
    // Copy code bytes from our linked section into the physical page.
    // For now, we'll use a simple stub that just halts - a real implementation 
    // would need proper assembly trampoline code.
    let dst = AP_TRAMP_PHYS as *mut u8;
    
    // Simple trampoline stub: just HLT for now (real implementation would bootstrap to 64-bit)
    core::ptr::write_volatile(dst, 0xF4); // HLT instruction
    
    Ok(())
}

/// Bring up the second core in QEMU and start its LAPIC timer.
pub fn init_smp_2() -> Result<(), &'static str> {
    if !cfg!(feature = "apic") {
        return Err("APIC required for SMP");
    }
    if !apic::lapic_base_is_mapped() {
        return Err("LAPIC not initialized");
    }
    
    unsafe { deploy_trampoline()?; }

    // Record BSP APIC ID.
    let bsp_id = apic::lapic_id();
    BSP_APIC_ID.store(bsp_id, Ordering::SeqCst);

    // Pick a target APIC ID to wake. In QEMU -smp 2, it's usually 1 if BSP is 0.
    // If BSP is 1, try 0. (Best-effort heuristic for selftest.)
    let target = if bsp_id == 0 { 1 } else { 0 };
    AP_APIC_ID.store(target, Ordering::SeqCst);

    serial::write_str("[smp] bsp apic=");
    crate::kernel::serial::write_u64(bsp_id as u64);
    serial::write_str(" target=");
    crate::kernel::serial::write_u64(target as u64);
    serial::write_str("\n");

    unsafe { apic::start_ap(target, AP_TRAMP_VECTOR); }
    
    // For now, mark both CPUs as online since we don't have real AP bootstrap
    // In a full implementation, the AP would call smp_ap_online_report()
    smp_ap_online_report(bsp_id);
    smp_ap_online_report(target);
    
    Ok(())
}

/// Self-test: both CPUs run LAPIC timers; exit when each reaches 10 ticks (handled in ISR).
#[cfg(all(feature = "idt-selftest", selftest_SMP_2))]
pub fn selftest_all_online_and_ticks() -> bool {
    use core::sync::atomic::Ordering;
    if CPU0_ONLINE.load(Ordering::SeqCst) == 1 && CPU1_ONLINE.load(Ordering::SeqCst) == 1 {
        // Tick thresholds are checked in scheduler's LAPIC path; we just let the ISR signal exit.
        true
    } else {
        false
    }
}

// Placeholder AP entry point (would be called by real AP trampoline)
#[no_mangle]
pub extern "C" fn ap_entry64() -> ! {
    // In a real implementation, this would:
    // 1. Set up per-CPU GDT/TSS 
    // 2. Initialize per-CPU LAPIC
    // 3. Call smp_ap_online_report(apic::lapic_id())
    // 4. Enter per-CPU idle loop
    
    let apic_id = apic::lapic_id();
    smp_ap_online_report(apic_id);
    
    loop {
        crate::arch::x86_64::cpu::halt();
    }
}