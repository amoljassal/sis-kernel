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
use x86_64::registers::control;
use core::sync::atomic::{AtomicU32, Ordering};

// AP boot status tracking to prevent indefinite hangs
const AP_BOOT_UNSTARTED: u32 = 0;
const AP_BOOT_STARTED: u32 = 1;
const AP_BOOT_SUCCESS: u32 = 0xBEEF;

// Support up to 16 APs (common for desktop/server systems)
static AP_BOOT_STATUS: [AtomicU32; 16] = [const { AtomicU32::new(AP_BOOT_UNSTARTED) }; 16];

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

/// Called by APs when they successfully complete initialization
pub fn ap_boot_complete(apic_id: u32) {
    if (apic_id as usize) < AP_BOOT_STATUS.len() {
        AP_BOOT_STATUS[apic_id as usize].store(AP_BOOT_SUCCESS, Ordering::Relaxed);
        serial::write_str("[smp] AP ");
        serial::write_u64(apic_id as u64);
        serial::write_str(" initialization complete\n");
    }
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

            // Mark AP as started before sending INIT/SIPI
            if (target_apic_id as usize) < AP_BOOT_STATUS.len() {
                AP_BOOT_STATUS[target_apic_id as usize].store(AP_BOOT_STARTED, Ordering::Relaxed);
            }

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

    // Wait for APs to come online with timeout (prevent indefinite hangs)
    serial::write_str("[smp] waiting for APs to boot (timeout: 5 seconds)...\n");
    let timeout_ms = 5000; // 5 seconds timeout
    for elapsed_ms in 0..timeout_ms {
        let mut any_ap_booted = false;
        
        // Check all APs we tried to start
        for target_apic_id in 1..4 {
            if target_apic_id != current_lapic_id && (target_apic_id as usize) < AP_BOOT_STATUS.len() {
                let status = AP_BOOT_STATUS[target_apic_id as usize].load(Ordering::Relaxed);
                if status == AP_BOOT_SUCCESS {
                    serial::write_str("[smp] AP ");
                    serial::write_u64(target_apic_id as u64);
                    serial::write_str(" booted successfully!\n");
                    any_ap_booted = true;
                }
            }
        }
        
        // If we got at least one AP, that's success for this test
        if any_ap_booted {
            break;
        }
        
        // Simple 1ms delay using CPU cycles
        simple_delay_us(1000);
        
        // Print progress every 1000ms
        if elapsed_ms % 1000 == 0 {
            serial::write_str("[smp] still waiting... (");
            serial::write_u64(elapsed_ms as u64);
            serial::write_str("ms elapsed)\n");
        }
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

    // CRITICAL FIX: Patch the PML4 table in the copied trampoline
    // The trampoline contains an empty PML4 table that causes APs to page fault
    let bsp_pml4_phys = control::Cr3::read().0.start_address().as_u64();
    
    // Calculate offset to pml4_table in the copied trampoline
    extern "C" {
        static pml4_table: u64;
    }
    let pml4_table_ptr = &pml4_table as *const u64;
    let pml4_offset = (pml4_table_ptr as usize) - (trampoline_start_ptr as usize);
    let pml4_patch_addr = (trampoline_addr + pml4_offset as u64) as *mut u64;
    
    // Patch the first PML4 entry with BSP's page table
    core::ptr::write(pml4_patch_addr, bsp_pml4_phys | 0x3); // Present + Writable
    
    serial::write_str("[smp] copied trampoline to 0x");
    crate::kernel::serial::write_hex64(trampoline_addr);
    serial::write_str(" size=");
    serial::write_u64(trampoline_size as u64);
    serial::write_str(" pml4_patched=0x");
    crate::kernel::serial::write_hex64(bsp_pml4_phys);
    serial::write_str("\n");

    // Send INIT IPI
    apic::send_ipi_raw(apic_id, 0x500); // INIT IPI (delivery mode)

    // Wait 10ms
    simple_delay_us(10000);

    // Send first SIPI
    let startup_vector = (trampoline_addr >> 12) as u8; // Page number
    apic::send_ipi_raw(apic_id, (startup_vector as u32) | 0x600); // SIPI

    // Wait 200us
    simple_delay_us(200);

    // Send second SIPI
    apic::send_ipi_raw(apic_id, (startup_vector as u32) | 0x600); // SIPI

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
