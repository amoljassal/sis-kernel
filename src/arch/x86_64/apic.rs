//! Minimal xAPIC + IOAPIC bring-up and a periodic LAPIC timer.
//! Feature-gated behind `apic`. Leaves legacy PIC/PIT intact in non-APIC builds.
//!
//! QEMU tip: use `-machine q35` (already in your harness). IOAPIC is present by default on q35.

use core::ptr::{read_volatile, write_volatile};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::model_specific::Msr;
use crate::kernel::serial;
use crate::arch::x86_64::memory;

// ===== xAPIC MSR =====
const IA32_APIC_BASE: u32 = 0x1B;
const IA32_APIC_BASE_ENABLE: u64 = 1 << 11; // APIC global enable bit
const IA32_APIC_BASE_BSP: u64    = 1 << 8;
const IA32_APIC_BASE_DEFAULT: u64 = 0xFEE0_0000;

// ===== LAPIC registers (offsets from base) =====
const LAPIC_REG_ID: u32       = 0x020;
const LAPIC_REG_VERSION: u32  = 0x030;
const LAPIC_REG_TPR: u32      = 0x080;
const LAPIC_REG_EOI: u32      = 0x0B0;
const LAPIC_REG_SVR: u32      = 0x0F0;
const LAPIC_REG_LVT_TIMER: u32= 0x320;
const LAPIC_REG_LVT_LINT0: u32= 0x350;
const LAPIC_REG_LVT_LINT1: u32= 0x360;
const LAPIC_REG_TMR_INIT: u32 = 0x380;
const LAPIC_REG_TMR_CURR: u32 = 0x390;
const LAPIC_REG_TMR_DIV: u32  = 0x3E0;

// LVT bits
const LVT_MASKED: u32 = 1 << 16;
const LVT_TIMER_PERIODIC: u32 = 1 << 17;
const SVR_APIC_ENABLE: u32 = 1 << 8;

// ===== IOAPIC MMIO =====
const IOAPIC_DEFAULT_PHYS: u64 = 0xFEC0_0000;
const IOAPIC_REGSEL: usize = 0x00;
const IOAPIC_IOWIN:  usize = 0x10;

// IOAPIC register indices
const IOAPIC_REG_ID:      u32 = 0x00;
const IOAPIC_REG_VER:     u32 = 0x01;
const IOAPIC_REG_REDIR0:  u32 = 0x10; // first redir entry (low 32 bits)

// We keep MMIO mappings here after map.
static mut LAPIC_BASE_VA: Option<*mut u32> = None;
static mut IOAPIC_BASE_VA: Option<*mut u32> = None;

#[inline(always)]
unsafe fn lapic_mmio() -> *mut u32 {
    LAPIC_BASE_VA.expect("LAPIC not mapped")
}

#[inline(always)]
pub fn lapic_base_is_mapped() -> bool { 
    unsafe { LAPIC_BASE_VA.is_some() } 
}

#[inline(always)]
unsafe fn ioapic_regsel() -> *mut u32 {
    IOAPIC_BASE_VA.expect("IOAPIC not mapped").byte_add(IOAPIC_REGSEL)
}
#[inline(always)]
unsafe fn ioapic_iowin() -> *mut u32 {
    IOAPIC_BASE_VA.expect("IOAPIC not mapped").byte_add(IOAPIC_IOWIN)
}

#[inline(always)]
unsafe fn lapic_read(off: u32) -> u32 {
    let p = lapic_mmio().byte_add(off as usize);
    read_volatile(p)
}
#[inline(always)]
unsafe fn lapic_write(off: u32, val: u32) {
    let p = lapic_mmio().byte_add(off as usize);
    write_volatile(p, val);
}

unsafe fn ioapic_write_reg(index: u32, value: u32) {
    write_volatile(ioapic_regsel(), index);
    write_volatile(ioapic_iowin(), value);
}
unsafe fn ioapic_read_reg(index: u32) -> u32 {
    write_volatile(ioapic_regsel(), index);
    read_volatile(ioapic_iowin())
}

#[inline(always)]
pub fn lapic_id() -> u32 {
    unsafe { (lapic_read(LAPIC_REG_ID) >> 24) & 0xFF }
}

/// Map LAPIC & IOAPIC ranges (uncached/strong). Uses simplified identity mapping for now.
unsafe fn map_apic_mmio() -> Result<(), &'static str> {
    // LAPIC base from MSR or default.
    let apic_msr = Msr::new(IA32_APIC_BASE);
    let base_val = apic_msr.read();
    let lapic_phys = (base_val & 0xFFFF_F000) as u64; // page-aligned phys
    let lapic_phys = if lapic_phys == 0 { IA32_APIC_BASE_DEFAULT } else { lapic_phys };

    // For simplicity, use identity mapping in higher half
    let lapic_va = VirtAddr::new(0xFFFF_FF80_0000_0000u64 + lapic_phys);
    let ioapic_va = VirtAddr::new(0xFFFF_FF80_0000_0000u64 + IOAPIC_DEFAULT_PHYS);
    
    // Map using existing memory infrastructure
    memory::map_user_page(lapic_va).ok(); // Try to map, ignore errors for now
    memory::map_user_page(ioapic_va).ok();
    
    LAPIC_BASE_VA = Some(lapic_va.as_mut_ptr());
    IOAPIC_BASE_VA = Some(ioapic_va.as_mut_ptr());
    
    Ok(())
}

fn has_xapic() -> bool {
    // CPUID: feature flags ECX/EDX; keep simple—assume QEMU/q35 has it.
    true
}

/// Enable xAPIC globally via IA32_APIC_BASE MSR and map MMIOs.
pub fn init_apic() -> Result<(), &'static str> {
    if !has_xapic() {
        return Err("xAPIC not supported");
    }
    unsafe {
        // Enable global APIC in MSR.
        let mut apic = Msr::new(IA32_APIC_BASE);
        let mut v = apic.read();
        v |= IA32_APIC_BASE_ENABLE;
        apic.write(v);

        map_apic_mmio()?;

        // Set Spurious Vector Register: enable APIC, set vector 0xFF (masked anyway)
        let svr = 0xFF | SVR_APIC_ENABLE;
        lapic_write(LAPIC_REG_SVR, svr);

        // Mask LINT0/LINT1 initially.
        lapic_write(LAPIC_REG_LVT_LINT0, LVT_MASKED);
        lapic_write(LAPIC_REG_LVT_LINT1, LVT_MASKED);

        serial::write_str("[init] LAPIC enabled\n");
    }
    Ok(())
}

/// Minimal IOAPIC init: read version/ID; leave IRQ0 masked (we'll use LAPIC timer, not PIT).
pub fn init_ioapic() -> Result<(), &'static str> {
    unsafe {
        let ver = ioapic_read_reg(IOAPIC_REG_VER);
        let max_redir = ((ver >> 16) & 0xFF) as usize;
        let id  = ioapic_read_reg(IOAPIC_REG_ID);
        let _id = (id >> 24) & 0xF;
        let _entries = max_redir + 1;
        serial::write_str("[init] IOAPIC present\n");

        // Example: ensure IRQ0 redirection is masked (we are moving to LAPIC timer).
        // Redirection entry n has low part at 0x10+2n, high at 0x11+2n.
        let irq0 = 0;
        let lo_index = IOAPIC_REG_REDIR0 + (irq0 * 2) as u32;
        let hi_index = lo_index + 1;
        // Route to LAPIC ID 0 (BSP), vector 0x20 (but masked).
        ioapic_write_reg(hi_index, 0x00 << 24);     // dest APIC ID
        ioapic_write_reg(lo_index, 0x20 | (1<<16)); // masked=1
    }
    Ok(())
}

/// Configure LAPIC timer in periodic mode firing at vector 0x20 (Timer).
/// This reuses your IDT[32] handler (scheduler::tick()).
pub fn init_lapic_timer_periodic(initial_count: u32, divide: u32) -> Result<(), &'static str> {
    unsafe {
        // Divide configuration: 1,2,4,8,16,32,64,128 (encoded)
        // divide reg encoding: 0->2, 1->4, 2->8, 3->16, 8->1, 9->32, A->64, B->128
        let div_bits = match divide {
            1 => 0b1000,
            2 => 0b0000,
            4 => 0b0001,
            8 => 0b0010,
            16 => 0b0011,
            32 => 0b1001,
            64 => 0b1010,
            128 => 0b1011,
            _ => 0b0000, // default 2
        } as u32;
        lapic_write(LAPIC_REG_TMR_DIV, div_bits);

        // LVT Timer: vector 32, periodic, unmasked
        let lvt = (32u32) | LVT_TIMER_PERIODIC;
        lapic_write(LAPIC_REG_LVT_TIMER, lvt);

        lapic_write(LAPIC_REG_TMR_INIT, initial_count);
        serial::write_str("[init] LAPIC timer periodic\n");
    }
    Ok(())
}

/// Send EOI (APIC path).
pub fn eoi() {
    unsafe { lapic_write(LAPIC_REG_EOI, 0); }
}

// ===== IPI support (SMP) =====
const LAPIC_REG_ICR_LOW:  u32 = 0x300;
const LAPIC_REG_ICR_HIGH: u32 = 0x310;

const ICR_DELIVERY_INIT:      u32 = 0b101 << 8;
const ICR_DELIVERY_STARTUP:   u32 = 0b110 << 8;
const ICR_LEVEL_ASSERT:       u32 = 1 << 14;
const ICR_TRIGGER_EDGE:       u32 = 0 << 15;
const ICR_DEST_PHYSICAL:      u32 = 0 << 11;
const ICR_NO_SHORTHAND:       u32 = 0 << 18;

/// Send IPI to target APIC ID with given ICR low bits.
/// Raw IPI send function (low-level)
pub unsafe fn send_ipi_raw(apic_id: u32, icr_low: u32) {
    // Write hi then low.
    lapic_write(LAPIC_REG_ICR_HIGH, apic_id << 24);
    lapic_write(LAPIC_REG_ICR_LOW,  icr_low | ICR_NO_SHORTHAND);
    // Busy-wait until delivery finished (bit12: Delivery Status).
    while (lapic_read(LAPIC_REG_ICR_LOW) & (1 << 12)) != 0 { 
        core::hint::spin_loop(); 
    }
}

/// Send regular IPI with specific vector to target CPU
/// Used for cross-CPU communication and scheduling signals
pub fn send_ipi(target_cpu: u32, vector: u8) {
    unsafe {
        let target_apic_id = cpu_to_apic_id(target_cpu);
        send_ipi_raw(target_apic_id, ICR_DEST_PHYSICAL | ICR_TRIGGER_EDGE | (vector as u32));
    }
}

/// Convert CPU ID to APIC ID (simplified 1:1 mapping for most systems)
pub fn cpu_to_apic_id(cpu_id: u32) -> u32 {
    // In most systems, CPU ID == APIC ID, but this could be more complex
    // For Phase 6C, we use simple 1:1 mapping
    cpu_id
}

/// INIT + SIPI + SIPI sequence to a given APIC ID; `vector` is 4K-aligned physical page / 0x1000.
pub unsafe fn start_ap(apic_id: u32, vector: u8) {
    // INIT (deasserted INIT IPI with ASSERT level is enough in QEMU)
    send_ipi_raw(apic_id, ICR_DEST_PHYSICAL | ICR_TRIGGER_EDGE | ICR_LEVEL_ASSERT | ICR_DELIVERY_INIT);
    // Small delay
    for _ in 0..100000 { core::hint::spin_loop(); }
    // SIPI #1
    send_ipi_raw(apic_id, ICR_DEST_PHYSICAL | ICR_TRIGGER_EDGE | (ICR_DELIVERY_STARTUP | (vector as u32)));
    for _ in 0..100000 { core::hint::spin_loop(); }
    // SIPI #2 (recommended)
    send_ipi_raw(apic_id, ICR_DEST_PHYSICAL | ICR_TRIGGER_EDGE | (ICR_DELIVERY_STARTUP | (vector as u32)));
}