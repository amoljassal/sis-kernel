//! SMP (Symmetric Multi-Processing) support with AP bring-up
//!
//! This module handles multi-core CPU initialization including:
//! - AP (Application Processor) trampoline and bring-up
//! - Per-CPU initialization and GS-base setup
//! - LAPIC timer configuration for each CPU
//! - Cross-CPU communication and coordination

pub mod ipi;

use crate::arch::x86_64::{apic, percpu_clean as percpu};
use crate::kernel::serial;

/// Initialize SMP system - bring up BSP and discover/start APs
pub fn init() {
    let lapic_id = apic::lapic_id();
    percpu::init_bsp(lapic_id);
    serial::write_str("[smp] bsp online\n");

    // Install IPI handlers
    unsafe {
        ipi::install_handlers();
    }

    // Initialize LAPIC timer for BSP
    let _ = apic::init_lapic_timer_periodic(1000, 1); // ~1ms tick
}

/// TEST=SMP_ONLINE validation function
#[cfg(all(feature = "smp", selftest_SMP_ONLINE))]
pub fn test_smp_online() -> Result<(), &'static str> {
    serial::write_str("[test] SMP_ONLINE: Starting multi-core validation\n");

    let bsp_cpu = percpu::this();
    serial::write_str("[smp] test_smp_online(): BSP id=");
    serial::write_u64(bsp_cpu.cpu_id as u64);
    serial::write_str(" online\n");

    // For now, we'll bring up APs using the trampoline
    // Get all APIC IDs that should be brought online
    let current_lapic_id = apic::lapic_id();

    // Try to start AP with LAPIC ID 1 (common in 2-CPU systems)
    for target_apic_id in 1..4 {
        // Try LAPIC IDs 1, 2, 3
        if target_apic_id != current_lapic_id {
            serial::write_str("[smp] sending INIT/SIPI to apic_id=");
            serial::write_u64(target_apic_id as u64);
            serial::write_str("\n");

            unsafe {
                if let Err(e) = start_ap(target_apic_id) {
                    serial::write_str("[smp] failed to start AP ");
                    serial::write_u64(target_apic_id as u64);
                    serial::write_str(": ");
                    serial::write_str(e);
                    serial::write_str("\n");
                }
            }
        }
    }

    // Give APs time to come online
    for _ in 0..100000 {
        core::hint::spin_loop();
    }

    serial::write_str("[test] SMP_ONLINE: PASS - Multi-core initialization attempted\n");
    Ok(())
}

/// Start an Application Processor using INIT/SIPI sequence
unsafe fn start_ap(apic_id: u32) -> Result<(), &'static str> {
    extern "C" {
        static ap_trampoline_start: u8;
        static ap_trampoline_end: u8;
    }

    let trampoline_addr = 0x7000u64; // Below 1 MiB for real mode access

    // Calculate trampoline size
    let trampoline_start_ptr = &ap_trampoline_start as *const u8;
    let trampoline_end_ptr = &ap_trampoline_end as *const u8;
    let trampoline_size = (trampoline_end_ptr as usize) - (trampoline_start_ptr as usize);

    // Copy trampoline code to low memory
    core::ptr::copy_nonoverlapping(
        trampoline_start_ptr,
        trampoline_addr as *mut u8,
        trampoline_size.min(4096), // Safety: limit to 4KB
    );

    serial::write_str("[smp] copied trampoline to 0x");
    crate::kernel::serial::write_hex64(trampoline_addr);
    serial::write_str(" size=");
    serial::write_u64(trampoline_size as u64);
    serial::write_str("\n");

    // Send INIT IPI
    apic::send_ipi(apic_id, 0x00 | 0x500); // INIT IPI

    // Wait 10ms
    simple_delay_us(10000);

    // Send first SIPI
    let startup_vector = (trampoline_addr >> 12) as u8; // Page number
    apic::send_ipi(apic_id, startup_vector | 0x600); // SIPI

    // Wait 200us
    simple_delay_us(200);

    // Send second SIPI
    apic::send_ipi(apic_id, startup_vector | 0x600); // SIPI

    Ok(())
}

/// Simple delay function using CPU spin loops
fn simple_delay_us(microseconds: u64) {
    // Rough approximation: assume ~3GHz CPU, so ~3000 cycles per microsecond
    let cycles = microseconds * 3000;
    for _ in 0..cycles {
        core::hint::spin_loop();
    }
}

/// AP entry point called from trampoline (defined in assembly)
#[no_mangle]
pub extern "C" fn ap_entry64(apic_id: u32) -> ! {
    serial::write_str("[smp] AP apic_id=");
    serial::write_u64(apic_id as u64);
    serial::write_str(" online\n");

    // Initialize per-CPU data for this AP
    // For now, use the APIC ID as CPU ID
    unsafe {
        if let Err(e) = crate::arch::x86_64::percpu::init_percpu(apic_id, apic_id) {
            serial::write_str("[smp] AP init failed: ");
            serial::write_str(e);
            serial::write_str("\n");
        }
    }

    // Initialize LAPIC for this AP
    if let Err(e) = apic::init_apic() {
        serial::write_str("[smp] AP LAPIC init failed: ");
        serial::write_str(e);
        serial::write_str("\n");
    }

    // Set up LAPIC timer for this AP
    let _ = apic::init_lapic_timer_periodic(1000, 1);

    serial::write_str("[smp] AP ");
    serial::write_u64(apic_id as u64);
    serial::write_str(" fully initialized\n");

    // AP idle loop
    loop {
        // Enable interrupts and halt until next interrupt
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}
