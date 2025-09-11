//! ARM64 Performance Monitoring Unit (PMU) support
//!
//! This module implements PMU support for performance monitoring and profiling,
//! essential for achieving the <40μs AI inference target and optimizing
//! cognitive workload distribution across cores.
//!
//! Geometric Principle: Performance metrics form a coordinate system for
//! optimization, with each counter representing a dimension in the performance space.

use core::sync::atomic::{AtomicU64, Ordering};
use crate::arch::aarch64::percpu::PerCpu;

/// PMU event types for AI workload profiling
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum PmuEvent {
    // Architectural events (guaranteed available)
    SwIncrement = 0x00,        // Software increment
    L1ICacheRefill = 0x01,      // L1 instruction cache refill
    L1ITlbRefill = 0x02,        // L1 instruction TLB refill
    L1DCacheRefill = 0x03,      // L1 data cache refill
    L1DCacheAccess = 0x04,      // L1 data cache access
    L1DTlbRefill = 0x05,        // L1 data TLB refill
    LoadRetired = 0x06,         // Load instruction retired
    StoreRetired = 0x07,        // Store instruction retired
    InstructionRetired = 0x08,  // Instruction architecturally executed
    Exception = 0x09,           // Exception taken
    ExceptionReturn = 0x0A,     // Exception return
    ContextIdWrite = 0x0B,      // Write to CONTEXTIDR
    BranchMispredicted = 0x10,  // Branch mispredicted
    CycleCount = 0x11,          // Cycle counter
    BranchPredicted = 0x12,     // Predictable branch
    DataMemAccess = 0x13,       // Data memory access
    L1ICacheAccess = 0x14,      // L1 instruction cache access
    L1DCacheEviction = 0x15,    // L1 data cache eviction
    L2DCacheAccess = 0x16,      // L2 data cache access
    L2DCacheRefill = 0x17,      // L2 data cache refill
    L2DCacheEviction = 0x18,    // L2 data cache write-back
    BusAccess = 0x19,           // Bus access
    LocalMemError = 0x1A,       // Local memory error
    InstructionSpeculated = 0x1B, // Instruction speculatively executed
    TtbwWrite = 0x1C,           // Write to TTBR
    BusCycle = 0x1D,            // Bus cycle
    ChainEvent = 0x1E,          // For chaining counters
    L1DCacheAllocate = 0x1F,    // L1 data cache allocation
    
    // Microarchitectural events (implementation defined)
    NeonInstruction = 0x64,     // NEON/ASIMD instruction
    FpInstruction = 0x65,       // Floating-point instruction
    CryptoInstruction = 0x66,   // Crypto instruction
    SveInstruction = 0x67,      // SVE instruction (if available)
}

/// PMU configuration for a performance counter
#[derive(Debug, Clone, Copy)]
pub struct PmuConfig {
    pub event: PmuEvent,
    pub enable_interrupt: bool,
    pub enable_export: bool,
    pub count_kernel: bool,
    pub count_user: bool,
}

impl Default for PmuConfig {
    fn default() -> Self {
        Self {
            event: PmuEvent::CycleCount,
            enable_interrupt: false,
            enable_export: false,
            count_kernel: true,
            count_user: true,
        }
    }
}

/// PMU counter state
pub struct PmuCounter {
    pub config: PmuConfig,
    pub value: AtomicU64,
    pub overflow_count: AtomicU64,
}

/// ARM64 PMU system
pub struct Pmu {
    /// Number of available counters (excluding cycle counter)
    pub num_counters: u32,
    /// Counter width in bits
    pub counter_width: u32,
    /// Whether PMU is available
    pub available: bool,
}

impl Pmu {
    /// Initialize PMU system
    pub fn init() -> Result<Self, &'static str> {
        // Check if PMU is available
        let mut pmu = Self {
            num_counters: 0,
            counter_width: 0,
            available: false,
        };
        
        unsafe {
            // Read ID_AA64DFR0_EL1 to check PMU version
            let dfr0: u64;
            core::arch::asm!(
                "mrs {}, id_aa64dfr0_el1",
                out(reg) dfr0,
                options(nomem, nostack)
            );
            
            let pmu_version = (dfr0 >> 8) & 0xF;
            if pmu_version == 0 || pmu_version == 0xF {
                return Err("PMU not available");
            }
            
            pmu.available = true;
            
            // Read PMCR_EL0 for PMU configuration
            let pmcr: u64;
            core::arch::asm!(
                "mrs {}, pmcr_el0",
                out(reg) pmcr,
                options(nomem, nostack)
            );
            
            // Extract number of counters (bits 15:11)
            pmu.num_counters = ((pmcr >> 11) & 0x1F) as u32;
            
            // Counter width is typically 32 or 64 bits
            // Check PMCR.LC (bit 6) for long counter support
            pmu.counter_width = if (pmcr & (1 << 6)) != 0 { 64 } else { 32 };
            
            // Enable PMU
            let pmcr_enable = pmcr | 0x1; // Set E bit
            core::arch::asm!(
                "msr pmcr_el0, {}",
                in(reg) pmcr_enable,
                options(nomem, nostack)
            );
            
            // Enable user-mode access
            let pmuserenr: u64 = 0xF; // EN, SW, CR, ER bits
            core::arch::asm!(
                "msr pmuserenr_el0, {}",
                in(reg) pmuserenr,
                options(nomem, nostack)
            );
            
            // Clear all counters
            Self::reset_all_counters();
        }
        
        crate::kernel::serial::write_str("[PMU] Initialized: ");
        crate::kernel::serial::write_u32(pmu.num_counters);
        crate::kernel::serial::write_str(" counters, ");
        crate::kernel::serial::write_u32(pmu.counter_width);
        crate::kernel::serial::write_str("-bit width\n");
        
        Ok(pmu)
    }
    
    /// Reset all PMU counters
    pub fn reset_all_counters() {
        unsafe {
            // Reset cycle counter
            core::arch::asm!(
                "msr pmccntr_el0, xzr",
                options(nomem, nostack)
            );
            
            // Reset and clear overflow flags
            let pmcr: u64;
            core::arch::asm!(
                "mrs {}, pmcr_el0",
                out(reg) pmcr,
                options(nomem, nostack)
            );
            
            // Set C (clock counter reset) and P (event counter reset) bits
            let pmcr_reset = pmcr | (1 << 2) | (1 << 1);
            core::arch::asm!(
                "msr pmcr_el0, {}",
                in(reg) pmcr_reset,
                options(nomem, nostack)
            );
        }
    }
    
    /// Configure a performance counter
    pub fn configure_counter(counter: u32, config: &PmuConfig) -> Result<(), &'static str> {
        unsafe {
            // Select counter
            core::arch::asm!(
                "msr pmselr_el0, {}",
                in(reg) counter as u64,
                options(nomem, nostack)
            );
            
            // Configure event
            let pmxevtyper = (config.event as u64) |
                            (if config.count_kernel { 0 } else { 1 << 31 }) |
                            (if config.count_user { 0 } else { 1 << 30 });
            
            core::arch::asm!(
                "msr pmxevtyper_el0, {}",
                in(reg) pmxevtyper,
                options(nomem, nostack)
            );
            
            // Enable counter
            let enable_mask = 1u64 << counter;
            core::arch::asm!(
                "msr pmcntenset_el0, {}",
                in(reg) enable_mask,
                options(nomem, nostack)
            );
        }
        
        Ok(())
    }
    
    /// Read cycle counter
    #[inline(always)]
    pub fn read_cycle_counter() -> u64 {
        let cycles: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, pmccntr_el0",
                out(reg) cycles,
                options(nomem, nostack)
            );
        }
        cycles
    }
    
    /// Read performance counter
    #[inline(always)]
    pub fn read_counter(counter: u32) -> u64 {
        unsafe {
            // Select counter
            core::arch::asm!(
                "msr pmselr_el0, {}",
                in(reg) counter as u64,
                options(nomem, nostack)
            );
            
            // Read counter value
            let value: u64;
            core::arch::asm!(
                "mrs {}, pmxevcntr_el0",
                out(reg) value,
                options(nomem, nostack)
            );
            
            value
        }
    }
    
    /// Start counting
    pub fn start_counting() {
        unsafe {
            // Enable cycle counter
            core::arch::asm!(
                "msr pmcntenset_el0, {}",
                in(reg) 0x80000000u64, // Bit 31 for cycle counter
                options(nomem, nostack)
            );
        }
    }
    
    /// Stop counting
    pub fn stop_counting() {
        unsafe {
            // Disable all counters
            core::arch::asm!(
                "msr pmcntenclr_el0, {}",
                in(reg) 0xFFFFFFFFu64,
                options(nomem, nostack)
            );
        }
    }
}

/// Performance profiling for AI workloads
pub struct AiPerfProfile {
    pub start_cycles: u64,
    pub start_instructions: u64,
    pub start_cache_misses: u64,
    pub start_branch_misses: u64,
}

impl AiPerfProfile {
    /// Start profiling
    pub fn start() -> Self {
        // Configure counters for AI profiling
        let _ = Pmu::configure_counter(0, &PmuConfig {
            event: PmuEvent::InstructionRetired,
            ..Default::default()
        });
        
        let _ = Pmu::configure_counter(1, &PmuConfig {
            event: PmuEvent::L1DCacheRefill,
            ..Default::default()
        });
        
        let _ = Pmu::configure_counter(2, &PmuConfig {
            event: PmuEvent::BranchMispredicted,
            ..Default::default()
        });
        
        Pmu::start_counting();
        
        Self {
            start_cycles: Pmu::read_cycle_counter(),
            start_instructions: Pmu::read_counter(0),
            start_cache_misses: Pmu::read_counter(1),
            start_branch_misses: Pmu::read_counter(2),
        }
    }
    
    /// Stop profiling and calculate metrics
    pub fn stop(&self) -> AiPerfMetrics {
        let end_cycles = Pmu::read_cycle_counter();
        let end_instructions = Pmu::read_counter(0);
        let end_cache_misses = Pmu::read_counter(1);
        let end_branch_misses = Pmu::read_counter(2);
        
        AiPerfMetrics {
            cycles: end_cycles.saturating_sub(self.start_cycles),
            instructions: end_instructions.saturating_sub(self.start_instructions),
            cache_misses: end_cache_misses.saturating_sub(self.start_cache_misses),
            branch_misses: end_branch_misses.saturating_sub(self.start_branch_misses),
            ipc: Self::calculate_ipc(
                end_instructions.saturating_sub(self.start_instructions),
                end_cycles.saturating_sub(self.start_cycles)
            ),
        }
    }
    
    /// Calculate instructions per cycle
    fn calculate_ipc(instructions: u64, cycles: u64) -> f32 {
        if cycles == 0 {
            0.0
        } else {
            (instructions as f32) / (cycles as f32)
        }
    }
}

/// AI performance metrics
#[derive(Debug, Clone)]
pub struct AiPerfMetrics {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub branch_misses: u64,
    pub ipc: f32,
}

impl AiPerfMetrics {
    /// Check if metrics meet AI performance targets
    pub fn meets_ai_targets(&self) -> bool {
        // Assuming 1GHz CPU, <40μs = <40,000 cycles
        self.cycles < 40_000 && self.ipc > 1.5
    }
    
    /// Log performance metrics
    pub fn log(&self) {
        crate::kernel::serial::write_str("[PMU] Cycles: ");
        crate::kernel::serial::write_u64(self.cycles);
        crate::kernel::serial::write_str(", IPC: ");
        crate::kernel::serial::write_u32((self.ipc * 100.0) as u32);
        crate::kernel::serial::write_str("/100");
        
        if self.meets_ai_targets() {
            crate::kernel::serial::write_str(" ✓ <40μs target\n");
        } else {
            crate::kernel::serial::write_str(" ✗ exceeds target\n");
        }
    }
}

/// Update per-CPU performance counters
pub fn update_percpu_counters() {
    let percpu = PerCpu::current();
    
    // Update cycle counter
    let cycles = Pmu::read_cycle_counter();
    percpu.perf_counters.cycles.store(cycles, Ordering::Relaxed);
    
    // Update instruction counter
    let instructions = Pmu::read_counter(0);
    percpu.perf_counters.instructions.store(instructions, Ordering::Relaxed);
    
    // Update cache miss counters
    let l1d_misses = Pmu::read_counter(1);
    percpu.perf_counters.l1d_misses.store(l1d_misses, Ordering::Relaxed);
}