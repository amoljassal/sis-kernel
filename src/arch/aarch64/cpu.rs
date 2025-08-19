//! ARM64 CPU operations and control
//!
//! ARM64-specific CPU operations optimized for AI-native workloads
//! Based on Grok's ARM64 optimization guidance

use core::arch::asm;

/// Halt the CPU using WFI (Wait For Interrupt)
/// ARM64 power-efficient idle state
#[inline]
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

/// CPU pause hint using WFE (Wait For Event)
/// Optimizes spin loops and reduces power consumption
#[inline]
pub fn pause() {
    unsafe {
        asm!("wfe", options(nomem, nostack, preserves_flags));
    }
}

/// Enable interrupts by clearing DAIF.I bit
#[inline]
pub fn enable_interrupts() {
    unsafe {
        asm!("msr daifclr, #2", options(nomem, nostack));
    }
}

/// Disable interrupts by setting DAIF.I bit
#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("msr daifset, #2", options(nomem, nostack));
    }
}

/// Check if interrupts are enabled by reading DAIF register
#[inline]
pub fn are_interrupts_enabled() -> bool {
    let daif: u64;
    unsafe {
        asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
    }
    (daif & 0x80) == 0 // DAIF.I bit (bit 7) cleared means interrupts enabled
}

/// Get current CPU ID from MPIDR_EL1 register
#[inline]
pub fn get_cpu_id() -> u32 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nomem, nostack));
    }
    (mpidr & 0xFF) as u32 // Extract Aff0 field
}

/// Read system counter frequency from CNTFRQ_EL0
#[inline]
pub fn get_timer_frequency() -> u64 {
    let freq: u64;
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nomem, nostack));
    }
    freq
}

/// Read virtual counter value from CNTVCT_EL0  
#[inline]
pub fn read_timer_counter() -> u64 {
    let counter: u64;
    unsafe {
        asm!("mrs {}, cntvct_el0", out(reg) counter, options(nomem, nostack));
    }
    counter
}

/// Data Synchronization Barrier
#[inline]
pub fn dsb() {
    unsafe {
        asm!("dsb ish", options(nomem, nostack, preserves_flags));
    }
}

/// Data Memory Barrier  
#[inline]
pub fn dmb() {
    unsafe {
        asm!("dmb ish", options(nomem, nostack, preserves_flags));
    }
}

/// Instruction Synchronization Barrier
#[inline]
pub fn isb() {
    unsafe {
        asm!("isb", options(nomem, nostack, preserves_flags));
    }
}

/// Send Event to wake up CPUs in WFE
#[inline]
pub fn sev() {
    unsafe {
        asm!("sev", options(nomem, nostack, preserves_flags));
    }
}

/// Send Event Local (same CPU only)
#[inline]
pub fn sevl() {
    unsafe {
        asm!("sevl", options(nomem, nostack, preserves_flags));
    }
}

/// Read Main ID Register for CPU identification
#[inline]
pub fn read_midr() -> u64 {
    let midr: u64;
    unsafe {
        asm!("mrs {}, midr_el1", out(reg) midr, options(nomem, nostack));
    }
    midr
}

/// Read Current Exception Level
#[inline]
pub fn current_el() -> u32 {
    let current_el: u64;
    unsafe {
        asm!("mrs {}, currentel", out(reg) current_el, options(nomem, nostack));
    }
    ((current_el >> 2) & 0x3) as u32
}

/// Detect CPU core type for scheduling optimization
pub fn detect_core_type() -> crate::arch::aarch64::ARM64CoreType {
    let midr = read_midr();
    let implementer = (midr >> 24) & 0xFF;
    let part_num = (midr >> 4) & 0xFFF;
    
    match (implementer, part_num) {
        (0x41, 0xD05) => crate::arch::aarch64::ARM64CoreType::CortexA55,
        (0x41, 0xD08) => crate::arch::aarch64::ARM64CoreType::CortexA72,
        (0x41, 0xD0B) => crate::arch::aarch64::ARM64CoreType::CortexA76,
        // Apple Silicon cores (estimated part numbers)
        (0x61, _) => {
            // Apple implementer - distinguish by CPU ID or other means
            if get_cpu_id() < 4 {
                crate::arch::aarch64::ARM64CoreType::AppleFirestorm // Performance cores
            } else {
                crate::arch::aarch64::ARM64CoreType::AppleIcestorm  // Efficiency cores
            }
        }
        _ => crate::arch::aarch64::ARM64CoreType::CortexA72, // Default fallback
    }
}

/// CPU cache operations for AI workload optimization
pub mod cache {
    use core::arch::asm;

    /// Clean data cache by virtual address
    #[inline]
    pub fn clean_dcache_va(addr: u64) {
        unsafe {
            asm!("dc cvac, {}", in(reg) addr, options(nomem, nostack));
        }
    }

    /// Invalidate instruction cache by virtual address
    #[inline]
    pub fn invalidate_icache_va(addr: u64) {
        unsafe {
            asm!("ic ivau, {}", in(reg) addr, options(nomem, nostack));
        }
    }

    /// Clean and invalidate data cache by virtual address
    #[inline]
    pub fn clean_invalidate_dcache_va(addr: u64) {
        unsafe {
            asm!("dc civac, {}", in(reg) addr, options(nomem, nostack));
        }
    }
}