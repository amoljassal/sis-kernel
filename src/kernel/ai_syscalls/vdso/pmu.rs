//! Hardware Performance Monitoring Unit (PMU) integration
//!
//! Based on Grok's performance counter strategies for sub-50ns measurement
//! Provides cycle-accurate timing and performance analysis

use core::arch::asm;

/// PMU event types for ARM64
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PmuEvent {
    /// CPU cycles
    CpuCycles = 0x11,
    
    /// Instructions retired
    Instructions = 0x08,
    
    /// L1 data cache refill
    L1DCacheRefill = 0x03,
    
    /// L1 data cache access
    L1DCacheAccess = 0x04,
    
    /// L1 instruction cache refill
    L1ICacheRefill = 0x01,
    
    /// L2 cache refill
    L2CacheRefill = 0x17,
    
    /// Branch misprediction
    BranchMispredict = 0x10,
    
    /// Memory access
    MemoryAccess = 0x13,
    
    /// TLB refill
    TlbRefill = 0x05,
    
    /// Bus access
    BusAccess = 0x19,
}

/// PMU configuration for vDSO operations
pub struct PmuConfig {
    /// Enable user-mode access
    pub user_enable: bool,
    
    /// Events to monitor
    pub events: [Option<PmuEvent>; 6],  // ARM64 has 6 counters
    
    /// Enable cycle counter
    pub cycle_counter: bool,
}

/// Initialize PMU for user-mode access
/// 
/// From Grok: Direct PMU access for cycle-accurate measurement
#[inline(always)]
pub unsafe fn init_pmu() {
    // Enable user-mode access to PMU
    // Note: This requires kernel support via PMUSERENR_EL0
    unsafe {
        asm!(
            "mov x0, #1",
            "msr pmuserenr_el0, x0",       // Enable user access
            options(nostack, nomem)
        );
    }
    
    // Reset and enable cycle counter
    unsafe {
        asm!(
            "mov x0, #0x41",                // Enable + reset counters
            "msr pmcr_el0, x0",
            options(nostack, nomem)
        );
    }
    
    // Enable cycle counter specifically
    unsafe {
        asm!(
            "mov x0, #0x80000000",          // Cycle counter bit
            "msr pmcntenset_el0, x0",
            options(nostack, nomem)
        );
    }
}

/// Configure a specific PMU counter
#[inline(always)]
pub unsafe fn configure_counter(counter: u32, event: PmuEvent) {
    if counter >= 6 {
        return; // Invalid counter
    }
    
    match counter {
        0 => unsafe { asm!(
            "msr pmevtyper0_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        1 => unsafe { asm!(
            "msr pmevtyper1_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        2 => unsafe { asm!(
            "msr pmevtyper2_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        3 => unsafe { asm!(
            "msr pmevtyper3_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        4 => unsafe { asm!(
            "msr pmevtyper4_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        5 => unsafe { asm!(
            "msr pmevtyper5_el0, {}",
            in(reg) event as u64,
            options(nostack, nomem)
        ) },
        _ => {},
    }
    
    // Enable the counter
    let enable_bit = 1u64 << counter;
    unsafe {
        asm!(
            "msr pmcntenset_el0, {}",
            in(reg) enable_bit,
            options(nostack, nomem)
        );
    }
}

/// Read cycle counter (PMCCNTR_EL0)
/// 
/// Most accurate timing method on ARM64
#[inline(always)]
pub fn read_cycle_counter() -> u64 {
    let cycles: u64;
    unsafe {
        asm!(
            "mrs {}, pmccntr_el0",
            out(reg) cycles,
            options(nostack, nomem, preserves_flags)
        );
    }
    cycles
}

/// Read specific performance counter
#[inline(always)]
pub fn read_counter(counter: u32) -> u64 {
    let value: u64;
    unsafe {
        match counter {
            0 => asm!(
                "mrs {}, pmevcntr0_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            1 => asm!(
                "mrs {}, pmevcntr1_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            2 => asm!(
                "mrs {}, pmevcntr2_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            3 => asm!(
                "mrs {}, pmevcntr3_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            4 => asm!(
                "mrs {}, pmevcntr4_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            5 => asm!(
                "mrs {}, pmevcntr5_el0",
                out(reg) value,
                options(nostack, nomem, preserves_flags)
            ),
            _ => return 0,
        }
    }
    value
}

/// Reset a performance counter
#[inline(always)]
pub unsafe fn reset_counter(counter: u32) {
    if counter >= 6 {
        return;
    }
    
    // Disable counter
    let disable_bit = 1u64 << counter;
    unsafe {
        asm!(
            "msr pmcntenclr_el0, {}",
            in(reg) disable_bit,
            options(nostack, nomem)
        );
    }
    
    // Clear counter value (write to counter register)
    unsafe {
        match counter {
            0 => asm!("msr pmevcntr0_el0, xzr", options(nostack, nomem)),
            1 => asm!("msr pmevcntr1_el0, xzr", options(nostack, nomem)),
            2 => asm!("msr pmevcntr2_el0, xzr", options(nostack, nomem)),
            3 => asm!("msr pmevcntr3_el0, xzr", options(nostack, nomem)),
            4 => asm!("msr pmevcntr4_el0, xzr", options(nostack, nomem)),
            5 => asm!("msr pmevcntr5_el0, xzr", options(nostack, nomem)),
            _ => {},
        }
    }
    
    // Re-enable counter
    unsafe {
        asm!(
            "msr pmcntenset_el0, {}",
            in(reg) disable_bit,
            options(nostack, nomem)
        );
    }
}

/// Performance measurement scope guard
/// 
/// Automatically measures cycles for a code block
pub struct PerfScope {
    start_cycles: u64,
    start_instructions: Option<u64>,
    start_cache_misses: Option<u64>,
}

impl PerfScope {
    /// Start performance measurement
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            start_cycles: read_cycle_counter(),
            start_instructions: None,
            start_cache_misses: None,
        }
    }
    
    /// Start with instruction counting
    #[inline(always)]
    pub fn with_instructions() -> Self {
        unsafe {
            configure_counter(0, PmuEvent::Instructions);
        }
        Self {
            start_cycles: read_cycle_counter(),
            start_instructions: Some(read_counter(0)),
            start_cache_misses: None,
        }
    }
    
    /// Start with cache miss tracking
    #[inline(always)]
    pub fn with_cache_misses() -> Self {
        unsafe {
            configure_counter(1, PmuEvent::L1DCacheRefill);
        }
        Self {
            start_cycles: read_cycle_counter(),
            start_instructions: None,
            start_cache_misses: Some(read_counter(1)),
        }
    }
    
    /// Get elapsed cycles
    #[inline(always)]
    pub fn elapsed_cycles(&self) -> u64 {
        read_cycle_counter() - self.start_cycles
    }
    
    /// Get instructions per cycle (IPC)
    #[inline(always)]
    pub fn ipc(&self) -> f32 {
        if let Some(start_inst) = self.start_instructions {
            let elapsed_inst = read_counter(0) - start_inst;
            let elapsed_cycles = self.elapsed_cycles();
            if elapsed_cycles > 0 {
                (elapsed_inst as f32) / (elapsed_cycles as f32)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// Lightweight performance statistics
#[derive(Debug, Clone, Copy)]
pub struct PerfStats {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub branch_mispredicts: u64,
}

impl PerfStats {
    /// Capture current performance counters
    #[inline(always)]
    pub fn capture() -> Self {
        Self {
            cycles: read_cycle_counter(),
            instructions: read_counter(0),
            cache_misses: read_counter(1),
            branch_mispredicts: read_counter(2),
        }
    }
    
    /// Calculate difference between two captures
    #[inline(always)]
    pub fn delta(&self, earlier: &PerfStats) -> PerfStats {
        PerfStats {
            cycles: self.cycles.saturating_sub(earlier.cycles),
            instructions: self.instructions.saturating_sub(earlier.instructions),
            cache_misses: self.cache_misses.saturating_sub(earlier.cache_misses),
            branch_mispredicts: self.branch_mispredicts.saturating_sub(earlier.branch_mispredicts),
        }
    }
    
    /// Calculate IPC (Instructions Per Cycle)
    #[inline(always)]
    pub fn ipc(&self) -> f32 {
        if self.cycles > 0 {
            (self.instructions as f32) / (self.cycles as f32)
        } else {
            0.0
        }
    }
    
    /// Calculate cache miss rate
    #[inline(always)]
    pub fn cache_miss_rate(&self) -> f32 {
        if self.instructions > 0 {
            (self.cache_misses as f32) / (self.instructions as f32)
        } else {
            0.0
        }
    }
    
    /// Calculate branch misprediction rate
    #[inline(always)]
    pub fn branch_mispredict_rate(&self) -> f32 {
        if self.instructions > 0 {
            (self.branch_mispredicts as f32) / (self.instructions as f32)
        } else {
            0.0
        }
    }
}

/// Validate sub-50ns operation timing
/// 
/// From Grok: Measure with cycle-accurate precision
#[inline(never)] // Don't inline measurement functions
pub fn validate_sub_50ns_operation<F>(op: F, freq_hz: u64) -> bool
where
    F: Fn(),
{
    // Warm up cache and branch predictors
    for _ in 0..10 {
        op();
    }
    
    // Measure multiple iterations
    const ITERATIONS: usize = 1000;
    let start = read_cycle_counter();
    
    for _ in 0..ITERATIONS {
        op();
    }
    
    let end = read_cycle_counter();
    let total_cycles = end - start;
    let cycles_per_op = total_cycles / ITERATIONS as u64;
    
    // Convert to nanoseconds
    let ns_per_op = if freq_hz > 0 {
        (cycles_per_op * 1_000_000_000) / freq_hz
    } else {
        // Assume 3GHz if frequency unknown
        cycles_per_op / 3
    };
    
    ns_per_op < 50
}

/// Apple M1/M2 specific PMU events
/// 
/// Note: These are reverse-engineered and may not be fully accurate
#[cfg(target_os = "macos")]
pub mod apple_silicon {
    use super::*;
    
    /// M1/M2 Neural Engine activity indicator
    pub const ANE_ACTIVE: u32 = 0x1A0;
    
    /// M1/M2 GPU activity indicator
    pub const GPU_ACTIVE: u32 = 0x1A1;
    
    /// Check if Neural Engine is active
    #[inline(always)]
    pub fn is_neural_engine_active() -> bool {
        // This would require special kernel support
        // Placeholder for demonstration
        false
    }
}

/// Benchmark helper for vDSO operations
pub struct VdsoBenchmark {
    samples: [u64; 128],
    count: usize,
}

impl VdsoBenchmark {
    pub const fn new() -> Self {
        Self {
            samples: [0; 128],
            count: 0,
        }
    }
    
    /// Add a sample
    pub fn add_sample(&mut self, cycles: u64) {
        if self.count < self.samples.len() {
            self.samples[self.count] = cycles;
            self.count += 1;
        }
    }
    
    /// Get median cycles
    pub fn median_cycles(&self) -> u64 {
        if self.count == 0 {
            return 0;
        }
        
        let mut sorted = [0u64; 128];
        sorted[..self.count].copy_from_slice(&self.samples[..self.count]);
        sorted[..self.count].sort_unstable();
        
        sorted[self.count / 2]
    }
    
    /// Get 99th percentile cycles
    pub fn p99_cycles(&self) -> u64 {
        if self.count == 0 {
            return 0;
        }
        
        let mut sorted = [0u64; 128];
        sorted[..self.count].copy_from_slice(&self.samples[..self.count]);
        sorted[..self.count].sort_unstable();
        
        let p99_index = (self.count * 99) / 100;
        sorted[p99_index.min(self.count - 1)]
    }
}