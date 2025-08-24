//! AI Workload Scheduler - Unified Architecture
//!
//! Hierarchical AI-aware scheduler implementing Multi-AI consultation recommendations:
//! - Unified cost-based resource arbitration (Gemini)
//! - EDF for real-time, DRR for interactive/background (ChatGPT)  
//! - Practical priority+deadline hybrid approach (Grok reality check)
//! - Lock-free SPSC queues and atomic operations for safety

use crate::kernel::ai::{CognitivePriority, WorkloadType};
use crate::arch::ai::{ai_power, ne_hal, ai_mem, predictive_power};
use crate::kernel::serial;
use crate::kernel::sync::InitCell;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicU8, Ordering};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// AI Task QoS Classes (based on Multi-AI recommendations)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiQoS {
    /// Real-time inference: <50μs target (revised from <25μs per Grok's reality check)
    RealTimeInference = 0,
    /// Interactive AI: <10ms target for UI responsiveness
    InteractiveAI = 1,
    /// Background training: Throughput-optimized, preemptable
    BackgroundTraining = 2,
    /// System maintenance: Model updates, cleanup, lowest priority
    SystemMaintenance = 3,
}

/// AI Compute Resource Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeResource {
    /// M1 Neural Engine (primary AI accelerator)
    NeuralEngine,
    /// CPU NEON SIMD (fallback for small models)
    CpuNeon,
    /// GPU Metal Compute (for training workloads)
    GpuMetal,
}

/// AI Task Deadline (absolute timing)
#[derive(Debug, Clone, Copy)]
pub struct AiDeadline {
    /// Task submission time (ARM64 cycle counter)
    pub submit_cycles: u64,
    /// Absolute deadline (ARM64 cycle counter)
    pub deadline_cycles: u64,
    /// Relative deadline from submission (nanoseconds)
    pub deadline_ns: u32,
}

/// AI Task Descriptor
#[derive(Debug, Clone)]
pub struct AiTask {
    /// Unique task ID
    pub task_id: u64,
    /// Quality of Service class
    pub qos: AiQoS,
    /// Absolute deadline for completion
    pub deadline: AiDeadline,
    /// AI workload type
    pub workload_type: WorkloadType,
    /// Model ID for performance fingerprinting
    pub model_id: u32,
    /// Input tensor specifications
    pub input_size: usize,
    pub input_addr: u64,
    /// Output tensor specifications  
    pub output_size: usize,
    pub output_addr: u64,
    /// Preferred compute resource (hint)
    pub resource_hint: ComputeResource,
    /// User callback for completion
    pub completion_callback: Option<u64>,
    /// Task priority within QoS class
    pub priority_weight: u32,
}

/// Performance Fingerprint (simplified from Gemini's recommendation)
#[derive(Debug, Clone)]
pub struct PerformanceFingerprint {
    /// Model ID
    pub model_id: u32,
    /// Estimated execution time per resource (nanoseconds)
    pub exec_time_ns: [u32; 3], // [NE, CPU, GPU]
    /// Power consumption estimate (milliwatts)
    pub power_mw: [u32; 3],
    /// Thermal impact coefficient  
    pub thermal_coefficient: [f32; 3],
    /// Memory bandwidth requirements (MB/s)
    pub memory_bandwidth_mbs: u32,
    /// Confidence in estimates (0-100)
    pub confidence: u8,
}

/// Resource Cost Calculation (Gemini's unified cost model)
#[derive(Debug, Clone)]
pub struct ResourceCost {
    /// Predicted finish time (includes queuing delay)
    pub predicted_finish_ns: u64,
    /// Power consumption cost
    pub power_cost: u32,
    /// Thermal penalty (higher when thermal headroom low)
    pub thermal_penalty: u32,
    /// Resource utilization factor
    pub utilization_factor: f32,
    /// Total weighted cost
    pub total_cost: f32,
}

/// Lock-free SPSC Ring Buffer for AI Tasks
pub struct AiTaskQueue<const N: usize> {
    /// Task storage ring
    tasks: [core::cell::UnsafeCell<Option<AiTask>>; N],
    /// Head index (consumer)
    head: AtomicU32,
    /// Tail index (producer)
    tail: AtomicU32,
    /// Queue statistics
    enqueue_count: AtomicU64,
    dequeue_count: AtomicU64,
}

impl<const N: usize> AiTaskQueue<N> {
    /// Create new task queue
    pub const fn new() -> Self {
        // Initialize with None values in UnsafeCell
        const NONE_TASK: core::cell::UnsafeCell<Option<AiTask>> = core::cell::UnsafeCell::new(None);
        Self {
            tasks: [NONE_TASK; N],
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            enqueue_count: AtomicU64::new(0),
            dequeue_count: AtomicU64::new(0),
        }
    }

    /// Enqueue task (single producer)
    pub fn enqueue(&self, task: AiTask) -> Result<(), AiTask> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        
        // Check if queue is full
        if (tail.wrapping_add(1) % N as u32) == head {
            return Err(task);
        }
        
        // Store task
        unsafe {
            *self.tasks[tail as usize].get() = Some(task);
        }
        
        // Update tail with release ordering
        self.tail.store(tail.wrapping_add(1) % N as u32, Ordering::Release);
        self.enqueue_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    /// Dequeue task (single consumer)
    pub fn dequeue(&self) -> Option<AiTask> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        
        // Check if queue is empty
        if head == tail {
            return None;
        }
        
        // Load task
        let task = unsafe {
            (*self.tasks[head as usize].get()).take()
        };
        
        // Update head with release ordering
        self.head.store(head.wrapping_add(1) % N as u32, Ordering::Release);
        self.dequeue_count.fetch_add(1, Ordering::Relaxed);
        
        task
    }

    /// Get queue depth
    pub fn depth(&self) -> u32 {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        tail.wrapping_sub(head) % N as u32
    }
}

/// Per-Resource AI Scheduler
pub struct ResourceScheduler {
    /// Compute resource type
    pub resource: ComputeResource,
    /// Per-QoS task queues (lock-free SPSC)
    pub qos_queues: [AiTaskQueue<256>; 4],
    /// Real-time EDF min-heap (simplified)
    pub rt_heap: Vec<(u64, u64)>, // (deadline_cycles, task_id)
    /// DRR scheduling state for non-RT queues
    pub drr_deficit: [u32; 4],
    pub drr_quantum: [u32; 4],
    /// Resource utilization tracking
    pub utilization_ewma: AtomicU32, // Fixed-point percentage * 100
    /// Thermal headroom (Celsius * 100)
    pub thermal_headroom: AtomicU32,
    /// Performance statistics
    pub tasks_completed: AtomicU64,
    pub tasks_missed_deadline: AtomicU64,
    pub average_latency_ns: AtomicU64,
}

impl ResourceScheduler {
    /// Create new resource scheduler
    pub fn new(resource: ComputeResource) -> Self {
        // Set DRR quantum based on resource type and QoS
        let drr_quantum = match resource {
            ComputeResource::NeuralEngine => [0, 4096, 2048, 1024], // RT uses EDF, not DRR
            ComputeResource::CpuNeon => [0, 2048, 1024, 512],
            ComputeResource::GpuMetal => [0, 8192, 4096, 2048],
        };

        Self {
            resource,
            qos_queues: [
                AiTaskQueue::new(),
                AiTaskQueue::new(), 
                AiTaskQueue::new(),
                AiTaskQueue::new()
            ],
            rt_heap: Vec::with_capacity(128),
            drr_deficit: [0; 4],
            drr_quantum,
            utilization_ewma: AtomicU32::new(0),
            thermal_headroom: AtomicU32::new(8500), // 85°C * 100
            tasks_completed: AtomicU64::new(0),
            tasks_missed_deadline: AtomicU64::new(0),
            average_latency_ns: AtomicU64::new(0),
        }
    }

    /// Schedule AI task on this resource
    pub fn schedule_task(&mut self, task: AiTask) -> Result<(), AiTask> {
        match task.qos {
            AiQoS::RealTimeInference => {
                // Insert into RT heap (EDF)
                self.rt_heap.push((task.deadline.deadline_cycles, task.task_id));
                self.rt_heap.sort_by_key(|&(deadline, _)| deadline);
                self.qos_queues[0].enqueue(task)
            }
            _ => {
                // Enqueue in appropriate QoS queue
                let qos_idx = task.qos as usize;
                self.qos_queues[qos_idx].enqueue(task)
            }
        }
    }

    /// Get next task using hybrid EDF+DRR scheduling
    pub fn get_next_task(&mut self) -> Option<AiTask> {
        // 1. Real-time tasks first (EDF)
        if let Some((deadline_cycles, task_id)) = self.rt_heap.first().copied() {
            // Check if deadline missed
            if self.deadline_missed(deadline_cycles) {
                serial::write_str("[AI Scheduler] RT task missed deadline\n");
                self.tasks_missed_deadline.fetch_add(1, Ordering::Relaxed);
                self.rt_heap.remove(0);
                return self.get_next_task(); // Try next task
            }
            
            // Try to get RT task from queue
            if let Some(task) = self.qos_queues[0].dequeue() {
                self.rt_heap.remove(0);
                return Some(task);
            }
        }

        // 2. Interactive, Background, Maintenance via DRR
        for qos_idx in 1..4 {
            if self.drr_deficit[qos_idx] > 0 {
                if let Some(task) = self.qos_queues[qos_idx].dequeue() {
                    let task_cost = self.estimate_task_cost(&task);
                    self.drr_deficit[qos_idx] = self.drr_deficit[qos_idx].saturating_sub(task_cost);
                    return Some(task);
                }
            }
        }

        // 3. Replenish DRR deficits
        for qos_idx in 1..4 {
            self.drr_deficit[qos_idx] += self.drr_quantum[qos_idx];
        }

        None
    }

    /// Check if deadline is missed
    fn deadline_missed(&self, deadline_cycles: u64) -> bool {
        let now_cycles = self.read_cycle_counter();
        now_cycles > deadline_cycles
    }

    /// Estimate task computational cost for DRR
    fn estimate_task_cost(&self, task: &AiTask) -> u32 {
        // Simplified cost based on input size
        (task.input_size / 1024) as u32 + 1
    }

    /// Read ARM64 cycle counter
    #[inline(always)]
    fn read_cycle_counter(&self) -> u64 {
        let cycles: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles, options(nomem, nostack));
        }
        cycles
    }
}

/// Unified AI Scheduler (main orchestrator)
pub struct UnifiedAiScheduler {
    /// Per-resource schedulers
    pub neural_engine: ResourceScheduler,
    pub cpu_neon: ResourceScheduler,
    pub gpu_metal: ResourceScheduler,
    /// Performance fingerprint database
    pub fingerprints: BTreeMap<u32, PerformanceFingerprint>,
    /// Cost calculation weights (tunable via RL later)
    pub cost_weights: CostWeights,
    /// Predictive power management
    pub power_manager: predictive_power::PredictivePowerManager,
    /// Global statistics
    pub total_tasks_scheduled: AtomicU64,
    pub resource_utilization: [AtomicU32; 3], // [NE, CPU, GPU]
    /// Next task ID
    pub next_task_id: AtomicU64,
}

/// Cost calculation weights (Gemini's recommendation)
#[derive(Debug, Clone)]
pub struct CostWeights {
    /// Deadline weight (higher for real-time tasks)
    pub deadline_weight: f32,
    /// Power consumption weight
    pub power_weight: f32,
    /// Thermal penalty weight  
    pub thermal_weight: f32,
    /// Queue wait weight
    pub queue_weight: f32,
}

impl Default for CostWeights {
    fn default() -> Self {
        Self {
            deadline_weight: 1.0,
            power_weight: 0.3,
            thermal_weight: 0.5,
            queue_weight: 0.2,
        }
    }
}

impl UnifiedAiScheduler {
    /// Create new unified AI scheduler
    pub fn new() -> Self {
        Self {
            neural_engine: ResourceScheduler::new(ComputeResource::NeuralEngine),
            cpu_neon: ResourceScheduler::new(ComputeResource::CpuNeon),
            gpu_metal: ResourceScheduler::new(ComputeResource::GpuMetal),
            fingerprints: BTreeMap::new(),
            cost_weights: CostWeights::default(),
            power_manager: predictive_power::PredictivePowerManager::new(),
            total_tasks_scheduled: AtomicU64::new(0),
            resource_utilization: [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)],
            next_task_id: AtomicU64::new(1),
        }
    }

    /// Schedule AI task with unified cost-based arbitration
    pub fn schedule_task(
        &mut self,
        workload_type: WorkloadType,
        priority: CognitivePriority,
        model_id: u32,
        input_addr: u64,
        input_size: usize,
        output_addr: u64,
        output_size: usize,
        deadline_ns: u32,
    ) -> Result<u64, &'static str> {
        let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        
        // Convert priority to QoS class
        let qos = match priority {
            CognitivePriority::RealTimeInference => AiQoS::RealTimeInference,
            CognitivePriority::Interactive => AiQoS::InteractiveAI,
            CognitivePriority::Background => AiQoS::BackgroundTraining,
            CognitivePriority::Maintenance => AiQoS::SystemMaintenance,
        };

        // Create deadline
        let now_cycles = self.read_cycle_counter();
        let cycles_per_ns = self.get_timer_frequency() / 1_000_000_000;
        let deadline_cycles = now_cycles + (deadline_ns as u64 * cycles_per_ns);
        
        let deadline = AiDeadline {
            submit_cycles: now_cycles,
            deadline_cycles,
            deadline_ns,
        };

        // Select optimal resource using cost-based arbitration
        let resource = self.select_optimal_resource(model_id, qos, workload_type)?;

        // Create AI task
        let task = AiTask {
            task_id,
            qos,
            deadline,
            workload_type,
            model_id,
            input_size,
            input_addr,
            output_size,
            output_addr,
            resource_hint: resource,
            completion_callback: None,
            priority_weight: 100,
        };

        // Schedule on selected resource
        let schedule_result = match resource {
            ComputeResource::NeuralEngine => self.neural_engine.schedule_task(task.clone()),
            ComputeResource::CpuNeon => self.cpu_neon.schedule_task(task.clone()),
            ComputeResource::GpuMetal => self.gpu_metal.schedule_task(task.clone()),
        }.map_err(|_| "Resource queue full")?;

        // Update predictive power management
        self.update_power_predictions(resource, &task);

        self.total_tasks_scheduled.fetch_add(1, Ordering::Relaxed);
        Ok(task_id)
    }

    /// Select optimal resource using unified cost model
    fn select_optimal_resource(
        &self,
        model_id: u32,
        qos: AiQoS,
        workload_type: WorkloadType,
    ) -> Result<ComputeResource, &'static str> {
        // Get performance fingerprint (or create default)
        let fingerprint = self.fingerprints.get(&model_id)
            .cloned()
            .unwrap_or_else(|| self.create_default_fingerprint(model_id, workload_type));

        // Calculate cost for each resource
        let ne_cost = self.calculate_resource_cost(ComputeResource::NeuralEngine, &fingerprint, qos);
        let cpu_cost = self.calculate_resource_cost(ComputeResource::CpuNeon, &fingerprint, qos);
        let gpu_cost = self.calculate_resource_cost(ComputeResource::GpuMetal, &fingerprint, qos);

        // Select resource with minimum cost
        if ne_cost.total_cost <= cpu_cost.total_cost && ne_cost.total_cost <= gpu_cost.total_cost {
            Ok(ComputeResource::NeuralEngine)
        } else if cpu_cost.total_cost <= gpu_cost.total_cost {
            Ok(ComputeResource::CpuNeon)
        } else {
            Ok(ComputeResource::GpuMetal)
        }
    }

    /// Calculate cost for specific resource (Gemini's cost model)
    fn calculate_resource_cost(
        &self,
        resource: ComputeResource,
        fingerprint: &PerformanceFingerprint,
        qos: AiQoS,
    ) -> ResourceCost {
        let resource_idx = match resource {
            ComputeResource::NeuralEngine => 0,
            ComputeResource::CpuNeon => 1,
            ComputeResource::GpuMetal => 2,
        };

        // Adjust weights based on QoS
        let weights = match qos {
            AiQoS::RealTimeInference => CostWeights {
                deadline_weight: 10.0, // Heavily weight deadline
                power_weight: 0.1,
                thermal_weight: 0.2,
                queue_weight: 5.0,
            },
            AiQoS::InteractiveAI => self.cost_weights.clone(),
            AiQoS::BackgroundTraining => CostWeights {
                deadline_weight: 0.1,
                power_weight: 1.0, // Weight power efficiency
                thermal_weight: 1.0,
                queue_weight: 0.5,
            },
            AiQoS::SystemMaintenance => CostWeights {
                deadline_weight: 0.05,
                power_weight: 0.8,
                thermal_weight: 0.8,
                queue_weight: 0.1, // Can wait
            },
        };

        // Get queue depth for queueing delay
        let queue_depth = match resource {
            ComputeResource::NeuralEngine => self.neural_engine.qos_queues[qos as usize].depth(),
            ComputeResource::CpuNeon => self.cpu_neon.qos_queues[qos as usize].depth(),
            ComputeResource::GpuMetal => self.gpu_metal.qos_queues[qos as usize].depth(),
        };

        let exec_time_ns = fingerprint.exec_time_ns[resource_idx] as u64;
        let queue_delay_ns = queue_depth as u64 * exec_time_ns / 4; // Estimate
        let predicted_finish_ns = exec_time_ns + queue_delay_ns;

        let power_cost = fingerprint.power_mw[resource_idx];
        
        // Get thermal headroom
        let thermal_headroom = match resource {
            ComputeResource::NeuralEngine => self.neural_engine.thermal_headroom.load(Ordering::Relaxed),
            ComputeResource::CpuNeon => self.cpu_neon.thermal_headroom.load(Ordering::Relaxed),
            ComputeResource::GpuMetal => self.gpu_metal.thermal_headroom.load(Ordering::Relaxed),
        };
        
        let thermal_penalty = if thermal_headroom < 1000 { // <10°C headroom
            1000
        } else {
            (10000 / thermal_headroom).min(1000) // Inverse relationship
        };

        let total_cost = weights.deadline_weight * predicted_finish_ns as f32
            + weights.power_weight * power_cost as f32
            + weights.thermal_weight * thermal_penalty as f32
            + weights.queue_weight * queue_delay_ns as f32;

        ResourceCost {
            predicted_finish_ns,
            power_cost,
            thermal_penalty,
            utilization_factor: queue_depth as f32 / 256.0,
            total_cost,
        }
    }

    /// Create default performance fingerprint
    fn create_default_fingerprint(&self, model_id: u32, workload_type: WorkloadType) -> PerformanceFingerprint {
        // Simplified defaults based on workload type
        let (ne_time, cpu_time, gpu_time) = match workload_type {
            WorkloadType::RealTimeInference => (100_000, 500_000, 200_000), // ns
            WorkloadType::Training => (1_000_000, 5_000_000, 800_000),
            WorkloadType::DataProcessing => (50_000, 200_000, 100_000),
            WorkloadType::Preprocessing => (80_000, 300_000, 150_000),
            WorkloadType::Serving => (120_000, 400_000, 180_000),
            WorkloadType::Interactive => (90_000, 350_000, 160_000),
            WorkloadType::Background => (200_000, 800_000, 300_000),
        };

        PerformanceFingerprint {
            model_id,
            exec_time_ns: [ne_time, cpu_time, gpu_time],
            power_mw: [2000, 1500, 3000], // Rough estimates
            thermal_coefficient: [0.8, 0.4, 1.2],
            memory_bandwidth_mbs: 1000,
            confidence: 50, // Default confidence
        }
    }

    /// Get timer frequency
    fn get_timer_frequency(&self) -> u64 {
        // ARM64 system counter frequency (typically 24MHz)
        24_000_000
    }

    /// Read ARM64 cycle counter
    #[inline(always)]
    fn read_cycle_counter(&self) -> u64 {
        let cycles: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles, options(nomem, nostack));
        }
        cycles
    }
    
    /// Update predictive power management with task information
    fn update_power_predictions(&mut self, resource: ComputeResource, task: &AiTask) {
        // Capture immutable values first
        let current_cycles = self.read_cycle_counter();
        let timer_freq = self.get_timer_frequency();
        
        let resource_mgr = self.power_manager.get_resource_manager(resource);
        let current_time_us = (current_cycles / (timer_freq / 1_000_000)) as u32;
        
        // Get queue depth for current resource
        let queue_depth = match resource {
            ComputeResource::NeuralEngine => self.neural_engine.qos_queues[task.qos as usize].depth() as u16,
            ComputeResource::CpuNeon => self.cpu_neon.qos_queues[task.qos as usize].depth() as u16,
            ComputeResource::GpuMetal => self.gpu_metal.qos_queues[task.qos as usize].depth() as u16,
        };
        
        // Calculate inter-arrival time (Q15 format: estimate 1000 microseconds as baseline)
        let interarrival_q15 = predictive_power::Q15Math::from_float(1000.0);
        
        // Update predictor on enqueue
        resource_mgr.predictor.on_enqueue(interarrival_q15, queue_depth);
        
        // Check if race-to-sleep strategy should be applied
        let battery_class = predictive_power::BatteryClass::from_soc_percent(75); // TODO: Get real SoC
        let should_race = predictive_power::RaceToSleep::should_race(
            task.workload_type,
            match task.qos {
                AiQoS::RealTimeInference => CognitivePriority::RealTimeInference,
                AiQoS::InteractiveAI => CognitivePriority::Interactive,
                AiQoS::BackgroundTraining => CognitivePriority::Background,
                AiQoS::SystemMaintenance => CognitivePriority::Maintenance,
            },
            battery_class,
            queue_depth,
        );
        
        if should_race {
            // Request higher power state for race-to-sleep
            self.request_power_boost(resource, current_time_us);
        }
    }
    
    /// Request power boost for race-to-sleep optimization
    fn request_power_boost(&mut self, resource: ComputeResource, current_time_us: u32) {
        let thermal_class = predictive_power::ThermalClass::Cool; // TODO: Get real thermal state
        let battery_class = predictive_power::BatteryClass::Full; // TODO: Get real battery state
        
        let resource_mgr = self.power_manager.get_resource_manager(resource);
        
        // Temporarily boost utilization prediction to trigger higher power state
        let current_util = resource_mgr.predictor.predict_utilization_q15();
        let _boosted_util = (current_util * 130) / 100; // 30% boost
        
        // This would normally be handled by the power management update cycle
        // For now, we just trigger an immediate update
        let _ = resource_mgr.update_power_state(thermal_class, battery_class, current_time_us);
    }
    
    /// Update on task completion for power predictions
    pub fn on_task_complete(&mut self, resource: ComputeResource, service_time_us: u32) {
        let resource_mgr = self.power_manager.get_resource_manager(resource);
        
        // Convert service time to Q15 format
        let service_q15 = predictive_power::Q15Math::from_float(service_time_us as f32);
        
        // Update service time predictions
        resource_mgr.predictor.on_complete(service_q15);
    }

    /// Register performance fingerprint for model
    pub fn register_fingerprint(&mut self, fingerprint: PerformanceFingerprint) {
        let model_id = fingerprint.model_id;
        self.fingerprints.insert(fingerprint.model_id, fingerprint);
        serial::write_str("[AI Scheduler] Registered performance fingerprint for model ");
        serial::write_dec(model_id as u64);
        serial::write_str("\n");
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> AiSchedulerStats {
        AiSchedulerStats {
            total_tasks_scheduled: self.total_tasks_scheduled.load(Ordering::Relaxed),
            neural_engine_utilization: self.resource_utilization[0].load(Ordering::Relaxed),
            cpu_neon_utilization: self.resource_utilization[1].load(Ordering::Relaxed),
            gpu_metal_utilization: self.resource_utilization[2].load(Ordering::Relaxed),
            rt_tasks_completed: self.neural_engine.tasks_completed.load(Ordering::Relaxed),
            rt_deadline_misses: self.neural_engine.tasks_missed_deadline.load(Ordering::Relaxed),
            average_latency_ns: self.neural_engine.average_latency_ns.load(Ordering::Relaxed),
            registered_fingerprints: self.fingerprints.len() as u32,
            power_stats: self.power_manager.system_stats(),
        }
    }
}

/// AI Scheduler Statistics
#[derive(Debug, Clone)]
pub struct AiSchedulerStats {
    pub total_tasks_scheduled: u64,
    pub neural_engine_utilization: u32,
    pub cpu_neon_utilization: u32,
    pub gpu_metal_utilization: u32,
    pub rt_tasks_completed: u64,
    pub rt_deadline_misses: u64,
    pub average_latency_ns: u64,
    pub registered_fingerprints: u32,
    pub power_stats: predictive_power::SystemPowerStats,
}

/// Global unified AI scheduler instance
static AI_SCHEDULER: InitCell<spin::Mutex<UnifiedAiScheduler>> = InitCell::new();

/// Initialize unified AI scheduler
pub fn init_ai_scheduler() -> Result<(), &'static str> {
    serial::write_str("[AI Scheduler] Initializing unified AI workload scheduler\n");
    
    let scheduler = UnifiedAiScheduler::new();
    AI_SCHEDULER.init(|| spin::Mutex::new(scheduler));
    
    serial::write_str("[AI Scheduler] Unified AI scheduler initialized successfully\n");
    Ok(())
}

/// Schedule AI task (global interface)
pub fn schedule_ai_task(
    workload_type: WorkloadType,
    priority: CognitivePriority,
    model_id: u32,
    input_addr: u64,
    input_size: usize,
    output_addr: u64,
    output_size: usize,
    deadline_ns: u32,
) -> Result<u64, &'static str> {
    match AI_SCHEDULER.get() {
        Some(scheduler) => {
            scheduler.lock().schedule_task(
                workload_type, priority, model_id,
                input_addr, input_size, output_addr, output_size, deadline_ns
            )
        }
        None => Err("AI scheduler not initialized"),
    }
}

/// Get AI scheduler statistics (global interface)
pub fn get_ai_scheduler_stats() -> Option<AiSchedulerStats> {
    AI_SCHEDULER.get().map(|s| s.lock().get_stats())
}

/// Register performance fingerprint (global interface)
pub fn register_model_fingerprint(fingerprint: PerformanceFingerprint) -> Result<(), &'static str> {
    match AI_SCHEDULER.get() {
        Some(scheduler) => {
            scheduler.lock().register_fingerprint(fingerprint);
            Ok(())
        }
        None => Err("AI scheduler not initialized"),
    }
}

/// Initialize AI scheduler for boot (Multi-AI boot framework)
pub fn init_scheduler() -> Result<(), &'static str> {
    // Initialize cognitive scheduler during HYPERCUBE layer
    
    // Check if already initialized
    if AI_SCHEDULER.get().is_some() {
        return Ok(());
    }
    
    // Initialize with hardware capabilities
    init_ai_scheduler().map_err(|_| "AI scheduler initialization failed")?;
    
    Ok(())
}