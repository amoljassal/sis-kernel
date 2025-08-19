//! Predictive Power Management for AI-Native Kernel
//!
//! Hierarchical prediction system based on Multi-AI consultation:
//! - Tactical Layer: <1μs EWMA/Holt predictions with fixed-point math
//! - Strategic Layer: Asynchronous learning and model adaptation
//! - Thermal-aware DVFS with proactive scaling
//! - Battery optimization with race-to-sleep patterns
//!
//! Based on production insights from Google TPU, Apple Neural Engine,
//! and Android Adaptive Battery systems.

use core::sync::atomic::{AtomicU32, AtomicU16, AtomicU8, Ordering};
use alloc::vec::Vec;
use crate::kernel::ai::{CognitivePriority, WorkloadType, ComputeResource};

/// Fixed-point math utilities for kernel (no floating point)
pub struct Q15Math;

impl Q15Math {
    /// Convert float to Q15 (compile-time constant)
    #[inline(always)]
    pub const fn from_float(x: f32) -> i32 {
        (x * 32768.0) as i32
    }
    
    /// Multiply two Q15 numbers
    #[inline(always)]
    pub fn mul(a: i32, b: i32) -> i32 {
        ((a as i64 * b as i64) >> 15) as i32
    }
    
    /// Clamp to positive u32 range
    #[inline(always)]
    pub fn clamp_u32(x: i64) -> u32 {
        x.max(0).min(u32::MAX as i64) as u32
    }
    
    /// Q16.16 multiply for thermal calculations
    #[inline(always)]
    pub fn mul_q16(a: i32, b: i32) -> i32 {
        ((a as i64 * b as i64) >> 16) as i32
    }
}

/// EWMA predictor with Q15 fixed-point arithmetic
#[repr(C)]
#[derive(Debug, Clone)]
pub struct EwmaQ15 {
    /// Current estimate (Q15 format)
    pub y: i32,
    /// Smoothing factor alpha (Q15 format: 0x0000-0x7FFF)
    pub alpha: i32,
}

impl EwmaQ15 {
    /// Create new EWMA with alpha coefficient
    pub const fn new(alpha_f32: f32) -> Self {
        Self {
            y: 0,
            alpha: Q15Math::from_float(alpha_f32),
        }
    }
    
    /// Update with new sample (constant time)
    #[inline(always)]
    pub fn update(&mut self, sample: i32) -> i32 {
        // y = α*sample + (1-α)*y = y + α*(sample - y)
        let dy = sample - self.y;
        self.y += Q15Math::mul(self.alpha, dy);
        self.y
    }
    
    /// Get current prediction
    #[inline(always)]
    pub fn predict(&self) -> i32 {
        self.y
    }
}

/// Holt linear trend predictor (double exponential smoothing)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HoltQ15 {
    /// Level component (Q15)
    pub level: i32,
    /// Trend component (Q15)
    pub trend: i32,
    /// Level smoothing factor (Q15)
    pub alpha: i32,
    /// Trend smoothing factor (Q15)
    pub beta: i32,
}

impl HoltQ15 {
    /// Create new Holt predictor
    pub const fn new(alpha_f32: f32, beta_f32: f32) -> Self {
        Self {
            level: 0,
            trend: 0,
            alpha: Q15Math::from_float(alpha_f32),
            beta: Q15Math::from_float(beta_f32),
        }
    }
    
    /// Update with new sample and return one-step forecast
    #[inline(always)]
    pub fn update(&mut self, sample: i32) -> i32 {
        let level_prev = self.level;
        
        // Update level: L_t = α*S_t + (1-α)*(L_{t-1} + B_{t-1})
        let level_pred = level_prev + self.trend;
        self.level = level_pred + Q15Math::mul(self.alpha, sample - level_pred);
        
        // Update trend: B_t = β*(L_t - L_{t-1}) + (1-β)*B_{t-1}
        let level_delta = self.level - level_prev;
        self.trend += Q15Math::mul(self.beta, level_delta - self.trend);
        
        // One-step forecast: F_{t+1} = L_t + B_t
        self.level + self.trend
    }
    
    /// Get current forecast
    #[inline(always)]
    pub fn predict(&self) -> i32 {
        self.level + self.trend
    }
}

/// DVFS power state with frequency and minimum residency
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PowerState {
    /// Frequency in MHz
    pub freq_mhz: u16,
    /// Minimum residency in microseconds
    pub min_residency_us: u16,
    /// Estimated power consumption in milliwatts
    pub power_mw: u16,
}

/// Power state presets for different compute resources
pub struct PowerStates;

impl PowerStates {
    /// M1 Neural Engine power states (based on Apple Silicon research)
    pub const NEURAL_ENGINE: &'static [PowerState] = &[
        PowerState { freq_mhz: 600,  min_residency_us: 100, power_mw: 800 },   // PowerSaver
        PowerState { freq_mhz: 1200, min_residency_us: 50,  power_mw: 1600 },  // Balanced
        PowerState { freq_mhz: 2400, min_residency_us: 25,  power_mw: 3200 },  // HighPerf
        PowerState { freq_mhz: 3200, min_residency_us: 10,  power_mw: 4800 },  // Boost
    ];
    
    /// ARM Cortex CPU power states (efficiency cores)
    pub const CPU_NEON: &'static [PowerState] = &[
        PowerState { freq_mhz: 600,  min_residency_us: 200, power_mw: 400 },
        PowerState { freq_mhz: 1200, min_residency_us: 100, power_mw: 800 },
        PowerState { freq_mhz: 1800, min_residency_us: 50,  power_mw: 1200 },
        PowerState { freq_mhz: 2400, min_residency_us: 25,  power_mw: 1800 },
    ];
    
    /// Metal GPU power states (integrated graphics)
    pub const GPU_METAL: &'static [PowerState] = &[
        PowerState { freq_mhz: 400,  min_residency_us: 500, power_mw: 2000 },
        PowerState { freq_mhz: 800,  min_residency_us: 200, power_mw: 4000 },
        PowerState { freq_mhz: 1200, min_residency_us: 100, power_mw: 6000 },
        PowerState { freq_mhz: 1600, min_residency_us: 50,  power_mw: 8000 },
    ];
    
    /// Get power states for resource
    pub fn for_resource(resource: ComputeResource) -> &'static [PowerState] {
        match resource {
            ComputeResource::NeuralEngine => Self::NEURAL_ENGINE,
            ComputeResource::CpuNeon => Self::CPU_NEON,
            ComputeResource::GpuMetal => Self::GPU_METAL,
        }
    }
}

/// Thermal headroom classification
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalClass {
    /// >85°C headroom - full performance
    Cool = 0,
    /// 70-85°C - slight reduction
    Warm = 1,
    /// 55-70°C - moderate reduction
    Hot = 2,
    /// <55°C - aggressive throttling
    Critical = 3,
}

impl ThermalClass {
    /// Classify thermal headroom from temperature (Celsius * 100)
    pub fn from_temp_c100(temp_c100: u32) -> Self {
        if temp_c100 < 5500 {
            ThermalClass::Critical
        } else if temp_c100 < 7000 {
            ThermalClass::Hot
        } else if temp_c100 < 8500 {
            ThermalClass::Warm
        } else {
            ThermalClass::Cool
        }
    }
    
    /// Power state bias (negative = reduce power)
    pub fn power_bias(self) -> i8 {
        match self {
            ThermalClass::Cool => 0,
            ThermalClass::Warm => -1,
            ThermalClass::Hot => -2,
            ThermalClass::Critical => -3,
        }
    }
}

/// Battery state classification
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatteryClass {
    /// >50% SoC - normal performance
    Full = 0,
    /// 20-50% SoC - balanced
    Medium = 1,
    /// <20% SoC - power saver
    Low = 2,
    /// External power - boost mode
    Plugged = 3,
}

impl BatteryClass {
    /// Classify battery from SoC percentage
    pub fn from_soc_percent(soc: u8) -> Self {
        if soc >= 50 {
            BatteryClass::Full
        } else if soc >= 20 {
            BatteryClass::Medium
        } else {
            BatteryClass::Low
        }
    }
    
    /// Power state bias
    pub fn power_bias(self) -> i8 {
        match self {
            BatteryClass::Plugged => 1,  // Allow boost
            BatteryClass::Full => 0,
            BatteryClass::Medium => -1,
            BatteryClass::Low => -2,
        }
    }
}

/// First-order thermal RC model (Q16.16 fixed-point)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ThermalModel {
    /// Current temperature (°C * 2^16)
    pub temp_q16: i32,
    /// Ambient temperature (°C * 2^16)
    pub ambient_q16: i32,
    /// Thermal time constant reciprocal (Δt/τ) * 2^16
    pub tau_inv_q16: i32,
    /// Thermal resistance (°C/W) * 2^16
    pub rth_q16: i32,
}

impl ThermalModel {
    /// Create new thermal model
    pub fn new(ambient_c: f32, tau_ms: f32, rth_c_per_w: f32, dt_ms: f32) -> Self {
        Self {
            temp_q16: ((ambient_c * 65536.0) as i32),
            ambient_q16: ((ambient_c * 65536.0) as i32),
            tau_inv_q16: ((dt_ms / tau_ms * 65536.0) as i32),
            rth_q16: ((rth_c_per_w * 65536.0) as i32),
        }
    }
    
    /// Step thermal model forward with power input
    #[inline(always)]
    pub fn step(&mut self, power_mw_q16: i32) -> i32 {
        // ΔT = (P*R_th + T_amb - T) * (Δt/τ)
        let pr = Q15Math::mul_q16(power_mw_q16, self.rth_q16);
        let diff = (pr + self.ambient_q16) - self.temp_q16;
        self.temp_q16 += Q15Math::mul_q16(diff, self.tau_inv_q16);
        self.temp_q16
    }
    
    /// Get current temperature in Celsius * 100
    #[inline(always)]
    pub fn temp_c100(&self) -> u32 {
        Q15Math::clamp_u32((self.temp_q16 >> 6) as i64) // Convert Q16.16 to C*100
    }
}

/// Fast tactical predictor for single resource
#[repr(C)]
#[derive(Debug)]
pub struct FastPredictor {
    /// Arrival rate predictor (jobs per microsecond, Q15)
    pub lambda: EwmaQ15,
    /// Service time predictor (microseconds, Q15 with Holt trends)
    pub service: HoltQ15,
    /// Queue depth change (signed)
    pub queue_delta: i16,
    /// Last queue depth
    pub queue_prev: u16,
    /// Utilization prediction cache (Q15)
    pub util_cache: AtomicU32,
}

impl FastPredictor {
    /// Create new fast predictor
    pub const fn new() -> Self {
        Self {
            lambda: EwmaQ15::new(0.4),      // α=0.4 for bursty arrivals
            service: HoltQ15::new(0.15, 0.05), // α=0.15, β=0.05 for service trends
            queue_delta: 0,
            queue_prev: 0,
            util_cache: AtomicU32::new(0),
        }
    }
    
    /// Update on task enqueue (hot path <200ns)
    #[inline(always)]
    pub fn on_enqueue(&mut self, interarrival_us_q15: i32, queue_depth: u16) {
        // Update arrival rate (inverse of inter-arrival time)
        self.lambda.update(interarrival_us_q15);
        
        // Track queue pressure
        self.queue_delta = (queue_depth as i16) - (self.queue_prev as i16);
        self.queue_prev = queue_depth;
        
        // Cache utilization prediction
        let util = Q15Math::mul(self.lambda.predict(), self.service.predict());
        self.util_cache.store(util as u32, Ordering::Relaxed);
    }
    
    /// Update on task completion
    #[inline(always)]
    pub fn on_complete(&mut self, service_time_us_q15: i32) {
        self.service.update(service_time_us_q15);
        
        // Update cached utilization
        let util = Q15Math::mul(self.lambda.predict(), self.service.predict());
        self.util_cache.store(util as u32, Ordering::Relaxed);
    }
    
    /// Get utilization prediction (ρ = λ × C)
    #[inline(always)]
    pub fn predict_utilization_q15(&self) -> i32 {
        self.util_cache.load(Ordering::Relaxed) as i32
    }
    
    /// Check if experiencing burst (rising queue)
    #[inline(always)]
    pub fn is_bursting(&self) -> bool {
        self.queue_delta > 0
    }
}

/// Power state selection bias tables
pub struct PowerBias;

impl PowerBias {
    /// Thermal bias table (indexed by ThermalClass)
    pub const THERMAL: [i8; 4] = [0, -1, -2, -3];
    
    /// Battery bias table (indexed by BatteryClass)
    pub const BATTERY: [i8; 4] = [-2, -1, 0, 1];
}

/// Fast power state selector (<300ns execution time)
pub struct FastPowerSelector {
    /// Utilization buckets count
    bucket_count: usize,
}

impl FastPowerSelector {
    /// Create new power selector
    pub const fn new() -> Self {
        Self { bucket_count: 16 }
    }
    
    /// Bucketize utilization (Q15) to power state index
    #[inline(always)]
    fn bucketize_q15(&self, util_q15: i32, power_states: &[PowerState]) -> usize {
        // Map 0..1.0 (Q15) to 0..(N-1) power states
        let util = util_q15.max(0).min(0x7FFF) as u32;
        let bucket = (util * (power_states.len() as u32 - 1)) >> 15;
        bucket as usize
    }
    
    /// Select optimal power state (sub-microsecond hot path)
    #[inline(always)]
    pub fn select_fast(
        &self,
        resource: ComputeResource,
        util_q15: i32,
        queue_rising: bool,
        thermal: ThermalClass,
        battery: BatteryClass,
    ) -> PowerState {
        let power_states = PowerStates::for_resource(resource);
        
        // Base selection from utilization
        let mut idx = self.bucketize_q15(util_q15, power_states);
        
        // Burst detection: bump up one level
        if queue_rising {
            idx = (idx + 1).min(power_states.len() - 1);
        }
        
        // Apply thermal and battery bias
        let thermal_bias = PowerBias::THERMAL[thermal as usize];
        let battery_bias = PowerBias::BATTERY[battery as usize];
        let total_bias = thermal_bias + battery_bias;
        
        // Clamp to valid range
        let final_idx = if total_bias >= 0 {
            (idx + total_bias as usize).min(power_states.len() - 1)
        } else {
            idx.saturating_sub((-total_bias) as usize)
        };
        
        power_states[final_idx]
    }
}

/// Per-resource predictive power manager
pub struct ResourcePowerManager {
    /// Compute resource type
    pub resource: ComputeResource,
    /// Fast tactical predictor
    pub predictor: FastPredictor,
    /// Current power state
    pub current_state: AtomicU8,
    /// Power state selector
    pub selector: FastPowerSelector,
    /// Last state change timestamp
    pub last_change_us: AtomicU32,
    /// Statistics
    pub state_changes: AtomicU32,
    pub avg_utilization: AtomicU16,
}

impl ResourcePowerManager {
    /// Create new resource power manager
    pub fn new(resource: ComputeResource) -> Self {
        Self {
            resource,
            predictor: FastPredictor::new(),
            current_state: AtomicU8::new(1), // Start in balanced state
            selector: FastPowerSelector::new(),
            last_change_us: AtomicU32::new(0),
            state_changes: AtomicU32::new(0),
            avg_utilization: AtomicU16::new(0),
        }
    }
    
    /// Update power state based on predictions (hot path)
    #[inline(always)]
    pub fn update_power_state(
        &mut self,
        thermal: ThermalClass,
        battery: BatteryClass,
        current_time_us: u32,
    ) -> Option<PowerState> {
        let util_q15 = self.predictor.predict_utilization_q15();
        let queue_rising = self.predictor.is_bursting();
        
        // Select optimal power state
        let new_state = self.selector.select_fast(
            self.resource,
            util_q15,
            queue_rising,
            thermal,
            battery,
        );
        
        // Check if change is needed
        let current_idx = self.current_state.load(Ordering::Relaxed);
        let power_states = PowerStates::for_resource(self.resource);
        
        if current_idx as usize >= power_states.len() {
            return None;
        }
        
        let current = power_states[current_idx as usize];
        
        // Apply minimum residency constraint
        let last_change = self.last_change_us.load(Ordering::Relaxed);
        if current_time_us.saturating_sub(last_change) < current.min_residency_us as u32 {
            return None; // Too soon to change
        }
        
        // Check if state actually needs to change
        if new_state.freq_mhz != current.freq_mhz {
            // Find new state index
            for (idx, state) in power_states.iter().enumerate() {
                if state.freq_mhz == new_state.freq_mhz {
                    self.current_state.store(idx as u8, Ordering::Relaxed);
                    self.last_change_us.store(current_time_us, Ordering::Relaxed);
                    self.state_changes.fetch_add(1, Ordering::Relaxed);
                    
                    // Update utilization average
                    let util_percent = (util_q15 >> 7) as u16; // Q15 to percent
                    self.avg_utilization.store(util_percent, Ordering::Relaxed);
                    
                    return Some(new_state);
                }
            }
        }
        
        None
    }
    
    /// Get current power state
    pub fn current_power_state(&self) -> PowerState {
        let idx = self.current_state.load(Ordering::Relaxed);
        let power_states = PowerStates::for_resource(self.resource);
        power_states.get(idx as usize).copied().unwrap_or(power_states[1])
    }
    
    /// Get statistics
    pub fn stats(&self) -> ResourcePowerStats {
        ResourcePowerStats {
            resource: self.resource,
            current_state_idx: self.current_state.load(Ordering::Relaxed),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            avg_utilization_percent: self.avg_utilization.load(Ordering::Relaxed),
            predicted_util_q15: self.predictor.predict_utilization_q15(),
            is_bursting: self.predictor.is_bursting(),
        }
    }
}

/// Resource power management statistics
#[derive(Debug, Clone)]
pub struct ResourcePowerStats {
    pub resource: ComputeResource,
    pub current_state_idx: u8,
    pub state_changes: u32,
    pub avg_utilization_percent: u16,
    pub predicted_util_q15: i32,
    pub is_bursting: bool,
}

/// Global predictive power manager (unified system)
pub struct PredictivePowerManager {
    /// Per-resource power managers
    pub neural_engine: ResourcePowerManager,
    pub cpu_neon: ResourcePowerManager,
    pub gpu_metal: ResourcePowerManager,
    
    /// Thermal model
    pub thermal: ThermalModel,
    
    /// System state
    pub battery_class: AtomicU8,
    pub thermal_class: AtomicU8,
    
    /// Global statistics
    pub total_power_mw: AtomicU32,
    pub thermal_throttles: AtomicU32,
    pub battery_limits: AtomicU32,
}

impl PredictivePowerManager {
    /// Create new predictive power manager
    pub fn new() -> Self {
        Self {
            neural_engine: ResourcePowerManager::new(ComputeResource::NeuralEngine),
            cpu_neon: ResourcePowerManager::new(ComputeResource::CpuNeon),
            gpu_metal: ResourcePowerManager::new(ComputeResource::GpuMetal),
            thermal: ThermalModel::new(25.0, 5000.0, 0.1, 1.0), // 25°C, 5s tau, 0.1°C/W
            battery_class: AtomicU8::new(BatteryClass::Full as u8),
            thermal_class: AtomicU8::new(ThermalClass::Cool as u8),
            total_power_mw: AtomicU32::new(0),
            thermal_throttles: AtomicU32::new(0),
            battery_limits: AtomicU32::new(0),
        }
    }
    
    /// Update system power management (called periodically)
    pub fn update_system(&mut self, current_time_us: u32) {
        let battery = BatteryClass::from_soc_percent(75); // TODO: Get real SoC
        let thermal_temp = self.thermal.temp_c100();
        let thermal = ThermalClass::from_temp_c100(thermal_temp);
        
        // Update atomic state
        self.battery_class.store(battery as u8, Ordering::Relaxed);
        self.thermal_class.store(thermal as u8, Ordering::Relaxed);
        
        // Update per-resource power states
        let ne_change = self.neural_engine.update_power_state(thermal, battery, current_time_us);
        let cpu_change = self.cpu_neon.update_power_state(thermal, battery, current_time_us);
        let gpu_change = self.gpu_metal.update_power_state(thermal, battery, current_time_us);
        
        // Calculate total power consumption
        let ne_power = self.neural_engine.current_power_state().power_mw as u32;
        let cpu_power = self.cpu_neon.current_power_state().power_mw as u32;
        let gpu_power = self.gpu_metal.current_power_state().power_mw as u32;
        let total_power = ne_power + cpu_power + gpu_power;
        
        self.total_power_mw.store(total_power, Ordering::Relaxed);
        
        // Update thermal model
        let power_q16 = (total_power as i32) << 16; // Convert to Q16.16
        self.thermal.step(power_q16);
        
        // Track throttling events
        if thermal != ThermalClass::Cool && (ne_change.is_some() || cpu_change.is_some() || gpu_change.is_some()) {
            self.thermal_throttles.fetch_add(1, Ordering::Relaxed);
        }
        
        if battery == BatteryClass::Low && (ne_change.is_some() || cpu_change.is_some() || gpu_change.is_some()) {
            self.battery_limits.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Get resource power manager for AI scheduler integration
    pub fn get_resource_manager(&mut self, resource: ComputeResource) -> &mut ResourcePowerManager {
        match resource {
            ComputeResource::NeuralEngine => &mut self.neural_engine,
            ComputeResource::CpuNeon => &mut self.cpu_neon,
            ComputeResource::GpuMetal => &mut self.gpu_metal,
        }
    }
    
    /// Get system power statistics
    pub fn system_stats(&self) -> SystemPowerStats {
        SystemPowerStats {
            total_power_mw: self.total_power_mw.load(Ordering::Relaxed),
            thermal_temp_c100: self.thermal.temp_c100(),
            thermal_class: unsafe { core::mem::transmute(self.thermal_class.load(Ordering::Relaxed)) },
            battery_class: unsafe { core::mem::transmute(self.battery_class.load(Ordering::Relaxed)) },
            thermal_throttles: self.thermal_throttles.load(Ordering::Relaxed),
            battery_limits: self.battery_limits.load(Ordering::Relaxed),
            ne_stats: self.neural_engine.stats(),
            cpu_stats: self.cpu_neon.stats(),
            gpu_stats: self.gpu_metal.stats(),
        }
    }
}

/// System-wide power management statistics
#[derive(Debug, Clone)]
pub struct SystemPowerStats {
    pub total_power_mw: u32,
    pub thermal_temp_c100: u32,
    pub thermal_class: ThermalClass,
    pub battery_class: BatteryClass,
    pub thermal_throttles: u32,
    pub battery_limits: u32,
    pub ne_stats: ResourcePowerStats,
    pub cpu_stats: ResourcePowerStats,
    pub gpu_stats: ResourcePowerStats,
}

/// Race-to-sleep power optimization
pub struct RaceToSleep;

impl RaceToSleep {
    /// Determine if task should use race-to-sleep strategy
    pub fn should_race(
        workload: WorkloadType,
        priority: CognitivePriority,
        battery: BatteryClass,
        queue_depth: u16,
    ) -> bool {
        match (workload, priority, battery) {
            // Real-time inference always races
            (WorkloadType::Inference, CognitivePriority::RealTimeInference, _) => true,
            // Interactive tasks race when battery is good and queue is light
            (_, CognitivePriority::Interactive, BatteryClass::Full | BatteryClass::Plugged) 
                if queue_depth < 4 => true,
            // Never race on low battery for background tasks
            (_, CognitivePriority::Background | CognitivePriority::Maintenance, BatteryClass::Low) => false,
            // Default: moderate approach
            _ => queue_depth < 2,
        }
    }
}