//! Simple benchmarking utilities for ARM64
//!
//! Provides basic timing and measurement primitives.

/// ARM64 timer for microsecond measurements
pub struct ARM64Timer {
    frequency: u64,
}

impl ARM64Timer {
    /// Initialize timer
    pub fn new() -> Self {
        let frequency = unsafe {
            let mut freq: u64;
            core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
            freq
        };

        Self { frequency }
    }

    /// Read cycle counter
    #[inline(always)]
    pub fn read_cycles(&self) -> u64 {
        unsafe {
            let mut count: u64;
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) count);
            count
        }
    }

    /// Convert cycles to microseconds
    #[inline(always)]
    pub fn cycles_to_us(&self, cycles: u64) -> u64 {
        (cycles * 1_000_000) / self.frequency
    }

    /// Get timer frequency
    pub fn frequency(&self) -> u64 {
        self.frequency
    }
}

/// Simple benchmark result
#[derive(Debug, Clone, Copy)]
pub struct BenchmarkResult {
    pub sample_count: usize,
    pub mean_cycles: u64,
    pub mean_us: u64,
    pub min_cycles: u64,
    pub max_cycles: u64,
}

/// Simple benchmarking context
pub struct BenchmarkContext {
    timer: ARM64Timer,
    warmup_iterations: usize,
    measurement_iterations: usize,
}

impl BenchmarkContext {
    /// Create new benchmark context
    pub fn new() -> Self {
        Self {
            timer: ARM64Timer::new(),
            warmup_iterations: 100,
            measurement_iterations: 1000,
        }
    }

    /// Execute benchmark
    pub fn benchmark<F, R>(&self, mut operation: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = operation();
        }

        // Measurements
        let mut measurements = alloc::vec::Vec::new();
        
        for _ in 0..self.measurement_iterations {
            let start = self.timer.read_cycles();
            let _ = operation();
            let end = self.timer.read_cycles();
            
            let cycles = end.saturating_sub(start);
            measurements.push(cycles);
        }

        self.analyze_measurements(measurements)
    }

    /// Analyze measurements
    fn analyze_measurements(&self, measurements: alloc::vec::Vec<u64>) -> BenchmarkResult {
        if measurements.is_empty() {
            return BenchmarkResult {
                sample_count: 0,
                mean_cycles: 0,
                mean_us: 0,
                min_cycles: 0,
                max_cycles: 0,
            };
        }

        let count = measurements.len();
        let sum: u64 = measurements.iter().sum();
        let mean_cycles = sum / count as u64;
        let mean_us = self.timer.cycles_to_us(mean_cycles);
        
        let min_cycles = *measurements.iter().min().unwrap();
        let max_cycles = *measurements.iter().max().unwrap();

        BenchmarkResult {
            sample_count: count,
            mean_cycles,
            mean_us,
            min_cycles,
            max_cycles,
        }
    }
}