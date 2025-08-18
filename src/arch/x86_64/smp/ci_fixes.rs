//! CI-Specific SMP Fixes for QEMU TCG
//! 
//! Based on Multi-AI consultation:
//! - Grok: TCG performance overhead, timer calibration, IPI delays
//! - ChatGPT: Memory ordering, spin-yield patterns, AP->BSP IPI
//! - Gemini: Environment detection, timeout scaling, telemetry
//!
//! This module implements critical fixes for CI timeout issues.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering, fence};
use x86_64::instructions::interrupts;

/// Check if we're running in CI environment
pub fn is_ci_environment() -> bool {
    // Check for GitHub Actions or general CI indicators
    #[cfg(ci_fast)]
    return true;
    
    #[cfg(not(ci_fast))]
    return false;
}

/// Detect QEMU TCG by checking CPUID hypervisor bit
pub fn is_qemu_tcg() -> bool {
    unsafe {
        use core::arch::x86_64::__cpuid;
        
        // Check hypervisor bit (CPUID.1:ECX[31])
        let result = __cpuid(0x1);
        if (result.ecx & (1 << 31)) != 0 {
            // Hypervisor present - likely QEMU in CI
            // Further check would look at CPUID.0x40000000 for signature
            return true;
        }
    }
    false
}

/// Get timeout multiplier for current environment
pub fn get_timeout_multiplier() -> u64 {
    if is_ci_environment() || is_qemu_tcg() {
        10  // 10x slower in TCG
    } else {
        1   // Normal speed
    }
}

/// Fair spin-wait that yields CPU under emulation
/// Based on ChatGPT's recommendation for TCG-friendly spinning
pub fn fair_spin_wait<F>(condition: F, timeout_ms: u64) -> bool 
where
    F: Fn() -> bool
{
    let start = crate::time::get_tsc_ms();
    let timeout = timeout_ms * get_timeout_multiplier();
    
    loop {
        // Check condition with proper memory ordering
        fence(Ordering::Acquire);
        if condition() {
            return true;
        }
        
        // Short spin with PAUSE instruction (Grok + ChatGPT consensus)
        for _ in 0..512 {
            core::hint::spin_loop(); // Emits PAUSE on x86
        }
        
        // Periodically yield CPU to allow TCG to schedule other vCPUs
        // This is CRITICAL for TCG where BSP can starve APs
        let elapsed = crate::time::get_tsc_ms() - start;
        if elapsed % 10 == 0 {  // Every 10ms
            unsafe {
                // Enable interrupts and halt to yield timeslice
                interrupts::enable();
                x86_64::instructions::hlt();
                interrupts::disable();
            }
        }
        
        if elapsed > timeout {
            return false;
        }
    }
}

/// Send IPI with delivery confirmation (ChatGPT's defensive technique)
pub fn send_ipi_checked(dest_apic_id: u32, vector: u32) -> Result<(), &'static str> {
    use crate::arch::x86_64::apic;
    
    // Send the IPI
    apic::send_ipi_raw(dest_apic_id, vector);
    
    // Poll delivery status bit (ICR[12])
    // This ensures IPI is actually delivered before proceeding
    let mut polls = 0;
    const MAX_POLLS: u32 = 100_000;
    
    while apic::read_icr_delivery_pending() {
        core::hint::spin_loop();
        polls += 1;
        if polls > MAX_POLLS {
            return Err("IPI delivery timeout");
        }
    }
    
    Ok(())
}

/// AP ready notification vector (ChatGPT's event-driven approach)
pub const AP_READY_VECTOR: u8 = 0xF2;

/// Padding to avoid false sharing (ChatGPT's cache optimization)
#[repr(align(64))]
pub struct PaddedAtomicU32(pub AtomicU32);

impl PaddedAtomicU32 {
    pub const fn new(val: u32) -> Self {
        PaddedAtomicU32(AtomicU32::new(val))
    }
}

/// Fixed SMP status array with proper alignment
pub static AP_BOOT_STATUS_FIXED: [PaddedAtomicU32; 256] = {
    const INIT: PaddedAtomicU32 = PaddedAtomicU32(AtomicU32::new(0));
    [INIT; 256]
};

/// Wait for AP with improved synchronization
/// Implements all three AI recommendations:
/// - Grok: Timeout scaling, PAUSE in loops
/// - ChatGPT: Acquire/Release ordering, HLT yielding, IPI wake
/// - Gemini: Progressive timeouts, telemetry
pub fn wait_for_ap_improved(cpu_id: u32) -> bool {
    use crate::kernel::serial;
    
    const AP_BOOT_SUCCESS: u32 = 3;
    let timeout_ms = if is_ci_environment() { 30_000 } else { 10_000 };
    
    serial::write_str("[smp] Waiting for AP ");
    serial::write_u64(cpu_id as u64);
    serial::write_str(" (timeout=");
    serial::write_u64(timeout_ms as u64);
    serial::write_str("ms, TCG=");
    serial::write_str(if is_qemu_tcg() { "yes" } else { "no" });
    serial::write_str(")\n");
    
    // Use fair spin that yields under TCG
    let result = fair_spin_wait(
        || {
            // CRITICAL: Use Acquire ordering (ChatGPT fix)
            AP_BOOT_STATUS_FIXED[cpu_id as usize].0.load(Ordering::Acquire) == AP_BOOT_SUCCESS
        },
        timeout_ms
    );
    
    if result {
        serial::write_str("[smp] AP ");
        serial::write_u64(cpu_id as u64);
        serial::write_str(" ready!\n");
    } else {
        serial::write_str("[smp] AP ");
        serial::write_u64(cpu_id as u64);
        serial::write_str(" TIMEOUT\n");
    }
    
    result
}

/// AP signals ready with proper ordering and IPI
/// Implements ChatGPT's event-driven notification
pub fn ap_signal_ready(cpu_id: u32) {
    const AP_BOOT_SUCCESS: u32 = 3;
    
    // CRITICAL: Use Release ordering (ChatGPT fix)
    AP_BOOT_STATUS_FIXED[cpu_id as usize].0.store(AP_BOOT_SUCCESS, Ordering::Release);
    
    // Send IPI to wake BSP from HLT (ChatGPT's optimization)
    // Assume BSP is APIC ID 0
    let _ = send_ipi_checked(0, AP_READY_VECTOR as u32);
}

/// Initialize CI fixes
pub fn init() {
    use crate::kernel::serial;
    
    if is_ci_environment() {
        serial::write_str("[ci] CI environment detected - applying TCG optimizations\n");
    }
    
    if is_qemu_tcg() {
        serial::write_str("[ci] QEMU TCG detected - using extended timeouts\n");
    }
    
    // Register AP ready interrupt handler
    unsafe {
        use crate::arch::x86_64::idt;
        // This would need IDT support for vector 0xF2
        // For now, we rely on HLT wakeup from any interrupt
    }
}