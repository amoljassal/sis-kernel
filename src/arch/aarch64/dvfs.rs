//! Predictive DVFS (Dynamic Voltage and Frequency Scaling) Controller for ARM64
//!
//! Implements intelligent power management for AI workloads with predictive control
//! based on workload characteristics and thermal constraints. Optimizes for both
//! performance and energy efficiency during AI inference.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use spinning_top::Spinlock;
use alloc::vec::Vec;
use alloc::collections::VecDeque;

/// CPU frequency levels for ARM64 (in MHz)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FrequencyLevel {
    /// Ultra-low power mode (600 MHz)
    UltraLow = 600,
    /// Low power mode (1200 MHz)
    Low = 1200,
    /// Efficiency mode (1800 MHz)
    Efficient = 1800,
    /// Balanced mode (2400 MHz)
    Balanced = 2400,
    /// Performance mode (3000 MHz)
    Performance = 3000,
    /// Turbo mode (3600 MHz)
    Turbo = 3600,
    /// AI Boost mode (4000 MHz) - for Neural Engine workloads
    AIBoost = 4000,
}

impl FrequencyLevel {
    /// Get voltage (in mV) for frequency level
    pub fn voltage_mv(&self) -> u32 {
        match self {
            FrequencyLevel::UltraLow => 650,
            FrequencyLevel::Low => 700,
            FrequencyLevel::Efficient => 750,
            FrequencyLevel::Balanced => 850,
            FrequencyLevel::Performance => 950,
            FrequencyLevel::Turbo => 1050,
            FrequencyLevel::AIBoost => 1150,
        }
    }
    
    /// Get power consumption estimate (in mW)
    pub fn power_mw(&self) -> u32 {
        // P = C * V^2 * F (simplified model)
        let voltage = self.voltage_mv() as f32 / 1000.0;
        let freq = *self as u32;
        ((voltage * voltage * freq as f32 * 0.5) * 1000.0) as u32
    }
    
    /// Get next higher frequency level
    pub fn step_up(&self) -> Option<FrequencyLevel> {
        match self {
            FrequencyLevel::UltraLow => Some(FrequencyLevel::Low),
            FrequencyLevel::Low => Some(FrequencyLevel::Efficient),
            FrequencyLevel::Efficient => Some(FrequencyLevel::Balanced),
            FrequencyLevel::Balanced => Some(FrequencyLevel::Performance),
            FrequencyLevel::Performance => Some(FrequencyLevel::Turbo),
            FrequencyLevel::Turbo => Some(FrequencyLevel::AIBoost),
            FrequencyLevel::AIBoost => None,
        }
    }
    
    /// Get next lower frequency level
    pub fn step_down(&self) -> Option<FrequencyLevel> {
        match self {
            FrequencyLevel::UltraLow => None,
            FrequencyLevel::Low => Some(FrequencyLevel::UltraLow),
            FrequencyLevel::Efficient => Some(FrequencyLevel::Low),
            FrequencyLevel::Balanced => Some(FrequencyLevel::Efficient),
            FrequencyLevel::Performance => Some(FrequencyLevel::Balanced),
            FrequencyLevel::Turbo => Some(FrequencyLevel::Performance),
            FrequencyLevel::AIBoost => Some(FrequencyLevel::Turbo),
        }
    }
}

/// Workload prediction model for DVFS decisions
#[derive(Debug, Clone)]
pub struct WorkloadPredictor {
    /// Historical utilization samples (rolling window)
    utilization_history: VecDeque<f32>,
    /// Historical inference latencies
    latency_history: VecDeque<u64>,
    /// Predicted next utilization
    predicted_utilization: f32,
    /// Prediction confidence (0.0 to 1.0)
    confidence: f32,
    /// Machine learning model weights (simplified linear model)
    weights: [f32; 4],
}

impl WorkloadPredictor {
    pub fn new() -> Self {
        Self {
            utilization_history: VecDeque::with_capacity(32),
            latency_history: VecDeque::with_capacity(32),
            predicted_utilization: 0.5,
            confidence: 0.0,
            weights: [0.3, 0.3, 0.2, 0.2], // Initial weights
        }
    }
    
    /// Add utilization sample and update prediction
    pub fn add_sample(&mut self, utilization: f32, latency: u64) {
        // Maintain rolling window
        if self.utilization_history.len() >= 32 {
            self.utilization_history.pop_front();
        }
        if self.latency_history.len() >= 32 {
            self.latency_history.pop_front();
        }
        
        self.utilization_history.push_back(utilization);
        self.latency_history.push_back(latency);
        
        // Update prediction using exponential moving average and trend
        self.update_prediction();
    }
    
    /// Update prediction model
    fn update_prediction(&mut self) {
        if self.utilization_history.len() < 4 {
            return;
        }
        
        // Calculate trend (simple linear regression)
        let n = self.utilization_history.len();
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        for (i, &util) in self.utilization_history.iter().enumerate() {
            let x = i as f32;
            sum_x += x;
            sum_y += util;
            sum_xy += x * util;
            sum_x2 += x * x;
        }
        
        let slope = (n as f32 * sum_xy - sum_x * sum_y) / (n as f32 * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n as f32;
        
        // Predict next value
        let next_x = n as f32;
        let trend_prediction = slope * next_x + intercept;
        
        // Combine with exponential moving average
        let ema_alpha = 0.3;
        let recent_avg = self.utilization_history.iter().rev().take(4).sum::<f32>() / 4.0;
        let ema_prediction = ema_alpha * recent_avg + (1.0 - ema_alpha) * self.predicted_utilization;
        
        // Weighted combination
        self.predicted_utilization = self.weights[0] * trend_prediction + self.weights[1] * ema_prediction;
        self.predicted_utilization = self.predicted_utilization.clamp(0.0, 1.0);
        
        // Update confidence based on prediction accuracy
        if self.utilization_history.len() > 8 {
            let actual = self.utilization_history.back().unwrap();
            let error = (actual - self.predicted_utilization).abs();
            self.confidence = (1.0 - error).max(0.0);
        }
    }
    
    /// Get workload prediction
    pub fn predict(&self) -> (f32, f32) {
        (self.predicted_utilization, self.confidence)
    }
    
    /// Predict if AI workload is incoming
    pub fn predict_ai_workload(&self) -> bool {
        // Detect AI workload patterns (sudden spikes, regular patterns)
        if self.latency_history.len() < 4 {
            return false;
        }
        
        // Check for characteristic AI inference patterns
        let recent_latencies: Vec<_> = self.latency_history.iter().rev().take(4).collect();
        let avg_latency = recent_latencies.iter().map(|&&l| l).sum::<u64>() / 4;
        
        // AI workloads typically have consistent, short latencies
        avg_latency < 50_000 && self.predicted_utilization > 0.7
    }
}

/// Thermal management for DVFS decisions
#[derive(Debug, Clone, Copy)]
pub struct ThermalState {
    /// Current temperature (in millidegrees Celsius)
    pub temperature_mc: u32,
    /// Temperature trend (positive = heating, negative = cooling)
    pub trend: i32,
    /// Thermal throttling active
    pub throttling: bool,
}

impl ThermalState {
    pub fn new() -> Self {
        Self {
            temperature_mc: 40_000, // 40°C initial
            trend: 0,
            throttling: false,
        }
    }
    
    /// Update thermal state from sensor
    pub fn update(&mut self, new_temp_mc: u32) {
        self.trend = new_temp_mc as i32 - self.temperature_mc as i32;
        self.temperature_mc = new_temp_mc;
        
        // Thermal limits
        const THROTTLE_TEMP_MC: u32 = 85_000; // 85°C
        const CRITICAL_TEMP_MC: u32 = 95_000; // 95°C
        
        self.throttling = self.temperature_mc >= THROTTLE_TEMP_MC;
    }
    
    /// Get maximum allowed frequency based on thermal state
    pub fn max_frequency(&self) -> FrequencyLevel {
        if self.temperature_mc >= 95_000 {
            FrequencyLevel::UltraLow // Critical temperature
        } else if self.temperature_mc >= 85_000 {
            FrequencyLevel::Efficient // Thermal throttling
        } else if self.temperature_mc >= 75_000 {
            FrequencyLevel::Performance // Warm
        } else {
            FrequencyLevel::AIBoost // Cool, full performance
        }
    }
}

/// DVFS Controller with predictive capabilities
pub struct DVFSController {
    /// Current frequency level
    current_freq: AtomicU32,
    /// Current voltage (in mV)
    current_voltage: AtomicU32,
    /// Workload predictor
    predictor: Spinlock<WorkloadPredictor>,
    /// Thermal state
    thermal: Spinlock<ThermalState>,
    /// Power budget (in mW)
    power_budget_mw: AtomicU32,
    /// AI workload active
    ai_workload_active: AtomicBool,
    /// Performance statistics
    frequency_changes: AtomicU64,
    prediction_hits: AtomicU64,
    prediction_misses: AtomicU64,
    thermal_throttle_events: AtomicU64,
}

impl DVFSController {
    pub fn new() -> Self {
        Self {
            current_freq: AtomicU32::new(FrequencyLevel::Balanced as u32),
            current_voltage: AtomicU32::new(FrequencyLevel::Balanced.voltage_mv()),
            predictor: Spinlock::new(WorkloadPredictor::new()),
            thermal: Spinlock::new(ThermalState::new()),
            power_budget_mw: AtomicU32::new(15_000), // 15W default
            ai_workload_active: AtomicBool::new(false),
            frequency_changes: AtomicU64::new(0),
            prediction_hits: AtomicU64::new(0),
            prediction_misses: AtomicU64::new(0),
            thermal_throttle_events: AtomicU64::new(0),
        }
    }
    
    /// Initialize DVFS hardware registers
    pub fn init_hardware(&self) -> Result<(), &'static str> {
        unsafe {
            // Initialize ARM64 performance counters for utilization tracking
            core::arch::asm!(
                // Enable user-mode access to performance counters
                "msr pmuserenr_el0, {val}",
                // Enable cycle counter
                "msr pmcntenset_el0, {cycles}",
                // Reset cycle counter
                "msr pmccntr_el0, xzr",
                val = in(reg) 1u64,
                cycles = in(reg) (1u64 << 31),
            );
        }
        
        unsafe {
            crate::kernel::serial::write_str("[DVFS] Predictive DVFS controller initialized\n");
            crate::kernel::serial::write_str("[DVFS] Frequency levels: 600MHz to 4000MHz (AI Boost)\n");
            crate::kernel::serial::write_str("[DVFS] Power budget: 15W\n");
            crate::kernel::serial::write_str("[DVFS] Thermal limits: 85°C throttle, 95°C critical\n");
        }
        
        Ok(())
    }
    
    /// Update DVFS state with new measurements
    pub fn update(&self, cpu_utilization: f32, inference_latency: u64, temperature_mc: u32) {
        // Update predictor
        {
            let mut predictor = self.predictor.lock();
            predictor.add_sample(cpu_utilization, inference_latency);
        }
        
        // Update thermal state
        {
            let mut thermal = self.thermal.lock();
            let was_throttling = thermal.throttling;
            thermal.update(temperature_mc);
            
            if thermal.throttling && !was_throttling {
                self.thermal_throttle_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // Make DVFS decision
        self.make_dvfs_decision(cpu_utilization);
    }
    
    /// Make DVFS frequency/voltage decision
    fn make_dvfs_decision(&self, current_utilization: f32) {
        let predictor = self.predictor.lock();
        let (predicted_util, confidence) = predictor.predict();
        let ai_predicted = predictor.predict_ai_workload();
        drop(predictor);
        
        // Update AI workload flag
        self.ai_workload_active.store(ai_predicted, Ordering::Relaxed);
        
        // Get thermal constraints
        let thermal = self.thermal.lock();
        let max_freq = thermal.max_frequency();
        let thermal_throttling = thermal.throttling;
        drop(thermal);
        
        // Determine target frequency
        let target_freq = if ai_predicted && !thermal_throttling {
            // AI workload detected - boost to maximum allowed
            FrequencyLevel::AIBoost.min(max_freq)
        } else if predicted_util > 0.9 {
            FrequencyLevel::Turbo.min(max_freq)
        } else if predicted_util > 0.7 {
            FrequencyLevel::Performance.min(max_freq)
        } else if predicted_util > 0.5 {
            FrequencyLevel::Balanced.min(max_freq)
        } else if predicted_util > 0.3 {
            FrequencyLevel::Efficient.min(max_freq)
        } else if predicted_util > 0.1 {
            FrequencyLevel::Low.min(max_freq)
        } else {
            FrequencyLevel::UltraLow
        };
        
        // Apply frequency change if needed
        let current_freq = self.current_freq.load(Ordering::Relaxed);
        if target_freq as u32 != current_freq {
            self.set_frequency(target_freq);
        }
        
        // Track prediction accuracy
        let actual_target = if current_utilization > 0.9 {
            FrequencyLevel::Turbo
        } else if current_utilization > 0.7 {
            FrequencyLevel::Performance  
        } else if current_utilization > 0.5 {
            FrequencyLevel::Balanced
        } else {
            FrequencyLevel::Efficient
        };
        
        if (target_freq as u32).abs_diff(actual_target as u32) <= 600 {
            self.prediction_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.prediction_misses.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Set CPU frequency and voltage
    fn set_frequency(&self, freq: FrequencyLevel) {
        let old_freq = self.current_freq.load(Ordering::Relaxed);
        let old_voltage = self.current_voltage.load(Ordering::Relaxed);
        
        let new_freq = freq as u32;
        let new_voltage = freq.voltage_mv();
        
        // Voltage-first for frequency increase, frequency-first for decrease
        if new_freq > old_freq {
            self.set_voltage_internal(new_voltage);
            self.set_frequency_internal(new_freq);
        } else {
            self.set_frequency_internal(new_freq);
            self.set_voltage_internal(new_voltage);
        }
        
        self.current_freq.store(new_freq, Ordering::Relaxed);
        self.current_voltage.store(new_voltage, Ordering::Relaxed);
        self.frequency_changes.fetch_add(1, Ordering::Relaxed);
        
        // Log significant changes
        if new_freq == FrequencyLevel::AIBoost as u32 && old_freq != FrequencyLevel::AIBoost as u32 {
            unsafe {
                crate::kernel::serial::write_str("[DVFS] AI Boost mode activated (4.0GHz)\n");
            }
        }
    }
    
    /// Set CPU frequency via hardware registers
    fn set_frequency_internal(&self, freq_mhz: u32) {
        // In a real implementation, this would write to SoC-specific registers
        // For Apple Silicon, this would interact with the performance controller
        unsafe {
            // Simulated frequency change
            core::arch::asm!(
                "dsb sy",
                "isb",
                options(nomem, nostack)
            );
        }
    }
    
    /// Set CPU voltage via PMIC
    fn set_voltage_internal(&self, voltage_mv: u32) {
        // In a real implementation, this would communicate with the PMIC
        // For Apple Silicon, voltage is typically managed by the performance controller
        unsafe {
            // Simulated voltage change with stabilization delay
            core::arch::asm!(
                "dsb sy",
                "isb",
                options(nomem, nostack)
            );
        }
    }
    
    /// Get current DVFS state
    pub fn get_state(&self) -> DVFSState {
        DVFSState {
            frequency_mhz: self.current_freq.load(Ordering::Relaxed),
            voltage_mv: self.current_voltage.load(Ordering::Relaxed),
            ai_workload_active: self.ai_workload_active.load(Ordering::Relaxed),
            thermal_throttling: self.thermal.lock().throttling,
            prediction_accuracy: self.get_prediction_accuracy(),
        }
    }
    
    /// Get prediction accuracy percentage
    fn get_prediction_accuracy(&self) -> f32 {
        let hits = self.prediction_hits.load(Ordering::Relaxed);
        let misses = self.prediction_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            (hits as f32 / total as f32) * 100.0
        }
    }
    
    /// Force AI boost mode for critical inference
    pub fn enable_ai_boost(&self) {
        self.ai_workload_active.store(true, Ordering::Relaxed);
        let thermal = self.thermal.lock();
        let max_freq = thermal.max_frequency();
        drop(thermal);
        
        self.set_frequency(FrequencyLevel::AIBoost.min(max_freq));
    }
    
    /// Return to normal operation
    pub fn disable_ai_boost(&self) {
        self.ai_workload_active.store(false, Ordering::Relaxed);
    }
}

/// DVFS state information
#[derive(Debug, Clone, Copy)]
pub struct DVFSState {
    pub frequency_mhz: u32,
    pub voltage_mv: u32,
    pub ai_workload_active: bool,
    pub thermal_throttling: bool,
    pub prediction_accuracy: f32,
}

/// Global DVFS controller instance
pub static DVFS_CONTROLLER: DVFSController = DVFSController::new();

/// DVFS utility functions
impl FrequencyLevel {
    /// Find optimal frequency for given utilization
    pub fn from_utilization(util: f32) -> Self {
        if util > 0.9 {
            FrequencyLevel::Turbo
        } else if util > 0.7 {
            FrequencyLevel::Performance
        } else if util > 0.5 {
            FrequencyLevel::Balanced
        } else if util > 0.3 {
            FrequencyLevel::Efficient
        } else if util > 0.1 {
            FrequencyLevel::Low
        } else {
            FrequencyLevel::UltraLow
        }
    }
    
    /// Minimum comparison for frequency levels
    pub fn min(self, other: FrequencyLevel) -> FrequencyLevel {
        if (self as u32) < (other as u32) {
            self
        } else {
            other
        }
    }
}