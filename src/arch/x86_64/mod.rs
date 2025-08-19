//! x86_64 specific modules.
//!
//! This module contains architecture‑specific code used by the SIS
//! kernel.  It is separated from the core kernel logic to make
//! porting to other architectures possible in the future.

#[cfg(feature = "apic")]
pub mod apic;
pub mod context_switch;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io;
#[cfg(feature = "smp")]
pub mod ipi;
pub mod irqvec;
pub mod memory;
#[cfg(feature = "smp")]
pub mod percpu;
#[cfg(feature = "smp")]
pub mod percpu_clean;
pub mod pit;
#[cfg(feature = "smp")]
pub mod shootdown;
#[cfg(feature = "smp")]
pub mod smp;
#[cfg(feature = "smp")]
pub mod topology;

// When smp is off, provide minimal no-op exports so cfg paths compile.
#[cfg(not(feature = "smp"))]
pub mod topology {
    #[inline]
    pub fn online_cpus() -> &'static [u32] {
        &[]
    }
    #[inline]
    pub fn cpu_index_from_apic(_apic_id: u32) -> Option<usize> {
        None
    }
}
#[cfg(feature = "per-task-mm")]
pub mod as_isolation;
#[cfg(feature = "idt-selftest")]
pub mod idt_selftest;
#[cfg(feature = "iommu")]
pub mod iommu;
#[cfg(feature = "ipc")]
pub mod ipc_selftest;
#[cfg(feature = "pf-matrix")]
pub mod pf_matrix;
#[cfg(feature = "scheduler")]
pub mod scheduler_selftest;

// AI acceleration fallback modules for x86_64
pub mod simd_fallback;
pub mod host_memory;
pub mod thermal_control;
pub mod mock_probe;
pub mod predictive_power;

// ============================================================================
// HAL Implementation for x86_64
// ============================================================================

use crate::kernel::hal::{Hal, HalCapability};

/// x86_64 HAL implementation
pub struct X86_64Hal;

/// Global HAL instance
pub static X86_64_HAL: X86_64Hal = X86_64Hal;

impl Hal for X86_64Hal {
    fn init(&self) -> Result<(), &'static str> {
        // x86_64 architecture initialization
        Ok(())
    }
    
    fn idle(&self) {
        // x86_64 HLT instruction
        cpu::halt();
    }
    
    fn send_ipi(&self, cpu_id: u32, vector: u8) {
        #[cfg(feature = "apic")]
        {
            // Use APIC to send IPI - placeholder for now
            let _ = (cpu_id, vector);
        }
        #[cfg(not(feature = "apic"))]
        {
            // No APIC available - stub implementation
            let _ = (cpu_id, vector);
        }
    }
    
    fn enable_interrupts(&self) {
        cpu::enable_interrupts();
    }
    
    fn disable_interrupts(&self) {
        cpu::disable_interrupts();
    }
    
    fn has_capability(&self, cap: HalCapability) -> bool {
        match cap {
            HalCapability::NeuralEngine => false, // x86_64 doesn't have Neural Engine
            HalCapability::GpuCompute => true,    // Can have discrete GPU
            HalCapability::SimdExtensions => true, // SSE/AVX available
            HalCapability::HardwareRng => false,   // Conservative default
            HalCapability::Virtualization => false, // Conservative default
        }
    }
    
    fn cpu_count(&self) -> u32 {
        #[cfg(feature = "smp")]
        {
            // Placeholder - would use actual SMP detection
            1
        }
        #[cfg(not(feature = "smp"))]
        {
            1
        }
    }
    
    fn current_cpu(&self) -> u32 {
        #[cfg(feature = "smp")]
        {
            // Placeholder - would read APIC ID
            0
        }
        #[cfg(not(feature = "smp"))]
        {
            0
        }
    }
    
    fn memory_barrier(&self) {
        // x86_64 memory fence
        unsafe {
            core::arch::asm!("mfence", options(nomem, nostack, preserves_flags));
        }
    }
    
    fn timer_init(&self, frequency_hz: u64) {
        // Initialize PIT or APIC timer - stub for now
        let _ = frequency_hz;
    }
    
    fn timer_ticks(&self) -> u64 {
        // Use TSC for high-resolution timer
        unsafe {
            core::arch::x86_64::_rdtsc()
        }
    }
}

/// Initialize x86_64 architecture
/// 
/// Entry point for x86_64-specific initialization
pub fn init() -> Result<(), &'static str> {
    X86_64_HAL.init()
}
