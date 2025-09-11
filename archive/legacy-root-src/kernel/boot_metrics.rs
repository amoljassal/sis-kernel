//! Boot Performance Metrics Collection
//!
//! Grok's performance monitoring implementation for boot-time metrics

use crate::boot::BootStage;
use crate::kernel::sync::InitCell;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global boot metrics instance
pub static BOOT_METRICS: InitCell<BootMetricsCollector> = InitCell::new();

/// Boot metrics collector
pub struct BootMetricsCollector {
    pub start_cycles: u64,
    pub stage_timings: [Option<StageMetrics>; 11],
    pub neural_detected: bool,
    pub neural_init_cycles: AtomicU64,
    pub memory_init_cycles: AtomicU64,
    pub total_boot_cycles: AtomicU64,
}

/// Per-stage metrics
#[derive(Debug, Clone, Copy)]
pub struct StageMetrics {
    pub stage: BootStage,
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub success: bool,
}

impl BootMetricsCollector {
    /// Create new metrics collector
    pub const fn new() -> Self {
        Self {
            start_cycles: 0,
            stage_timings: [None; 11],
            neural_detected: false,
            neural_init_cycles: AtomicU64::new(0),
            memory_init_cycles: AtomicU64::new(0),
            total_boot_cycles: AtomicU64::new(0),
        }
    }
    
    /// Initialize metrics collection
    pub fn init(&mut self, start_cycles: u64) {
        self.start_cycles = start_cycles;
    }
    
    /// Record stage start
    pub fn stage_start(&mut self, stage: BootStage, cycles: u64) {
        let index = stage as usize;
        if index < self.stage_timings.len() {
            self.stage_timings[index] = Some(StageMetrics {
                stage,
                start_cycles: cycles,
                end_cycles: 0,
                success: false,
            });
        }
    }
    
    /// Record stage completion
    pub fn stage_complete(&mut self, stage: BootStage, cycles: u64, success: bool) {
        let index = stage as usize;
        if index < self.stage_timings.len() {
            if let Some(ref mut metrics) = self.stage_timings[index] {
                metrics.end_cycles = cycles;
                metrics.success = success;
            }
        }
        
        // Track specific stage metrics
        match stage {
            BootStage::S15_MemoryInit => {
                if let Some(metrics) = self.stage_timings[index] {
                    let duration = metrics.end_cycles - metrics.start_cycles;
                    self.memory_init_cycles.store(duration, Ordering::Release);
                }
            }
            BootStage::S25_NeuralProbe | BootStage::S45_NeuralOnline => {
                if let Some(metrics) = self.stage_timings[index] {
                    let duration = metrics.end_cycles - metrics.start_cycles;
                    let current = self.neural_init_cycles.load(Ordering::Acquire);
                    self.neural_init_cycles.store(current + duration, Ordering::Release);
                }
            }
            _ => {}
        }
    }
    
    /// Mark Neural Engine as detected
    pub fn set_neural_detected(&mut self, detected: bool) {
        self.neural_detected = detected;
    }
    
    /// Calculate total boot time
    pub fn finalize(&mut self, end_cycles: u64) {
        let total = end_cycles - self.start_cycles;
        self.total_boot_cycles.store(total, Ordering::Release);
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let total_cycles = self.total_boot_cycles.load(Ordering::Acquire);
        let memory_cycles = self.memory_init_cycles.load(Ordering::Acquire);
        let neural_cycles = self.neural_init_cycles.load(Ordering::Acquire);
        
        // Convert cycles to milliseconds (rough estimate)
        let cycles_to_ms = |cycles: u64| -> u32 {
            #[cfg(target_arch = "x86_64")]
            { (cycles / 3_000_000) as u32 } // Assume 3GHz
            #[cfg(target_arch = "aarch64")]
            { (cycles / 2_000_000) as u32 } // Assume 2GHz
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            { (cycles / 1_000_000) as u32 } // Generic
        };
        
        PerformanceReport {
            total_boot_ms: cycles_to_ms(total_cycles),
            memory_init_ms: cycles_to_ms(memory_cycles),
            neural_init_ms: cycles_to_ms(neural_cycles),
            neural_detected: self.neural_detected,
            stages_completed: self.stage_timings.iter().filter(|s| s.is_some()).count() as u32,
        }
    }
    
    /// Check if performance targets are met (Grok's requirements)
    pub fn validate_performance(&self) -> PerformanceValidation {
        let report = self.generate_report();
        
        PerformanceValidation {
            boot_time_ok: report.total_boot_ms <= 1000, // <1s requirement
            boot_time_target: report.total_boot_ms <= 500, // <500ms target
            memory_init_ok: report.memory_init_ms <= 200, // <200ms requirement
            neural_init_ok: !report.neural_detected || report.neural_init_ms <= 100, // <100ms if present
        }
    }
}

/// Performance report structure
#[derive(Debug)]
pub struct PerformanceReport {
    pub total_boot_ms: u32,
    pub memory_init_ms: u32,
    pub neural_init_ms: u32,
    pub neural_detected: bool,
    pub stages_completed: u32,
}

/// Performance validation results
#[derive(Debug)]
pub struct PerformanceValidation {
    pub boot_time_ok: bool,
    pub boot_time_target: bool,
    pub memory_init_ok: bool,
    pub neural_init_ok: bool,
}

impl PerformanceValidation {
    /// Check if all requirements are met
    pub fn all_ok(&self) -> bool {
        self.boot_time_ok && self.memory_init_ok && self.neural_init_ok
    }
    
    /// Check if all targets are met
    pub fn all_targets_met(&self) -> bool {
        self.boot_time_target && self.memory_init_ok && self.neural_init_ok
    }
}

/// Export metrics for testing frameworks
pub fn export_metrics_for_testing() -> alloc::string::String {
    use alloc::format;
    
    if let Some(metrics) = BOOT_METRICS.get() {
        let report = metrics.generate_report();
        format!(
            "[METRICS] total_boot_ms={} memory_init_ms={} neural_init_ms={} neural_detected={} stages_completed={}",
            report.total_boot_ms,
            report.memory_init_ms,
            report.neural_init_ms,
            report.neural_detected,
            report.stages_completed
        )
    } else {
        alloc::string::String::from("[METRICS] not_initialized")
    }
}

// External alloc dependency
extern crate alloc;