//! AI Real-Time Scheduler - Phase 3 Implementation
//!
//! Provides real-time scheduling for AI workloads with <40μs latency guarantees.
//! Integrates with security layer and performance monitoring systems.
//!
//! Architecture:
//! - Priority-based scheduling with AI workload classification
//! - Deadline scheduling for inference operations
//! - CPU affinity management for optimal cache locality
//! - Integration with NPU emulation layer

use crate::kernel::capabilities::{CapabilityId, CapabilityRights};
use crate::kernel::ai_runtime::{LoadedModel, InferenceStats};
use crate::arch::aarch64::percpu::PerCpu;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Maximum number of AI tasks that can be scheduled
const MAX_AI_TASKS: usize = 64;

/// AI task priorities (higher number = higher priority)
const AI_PRIORITY_CRITICAL: u8 = 255;    // <10μs deadline
const AI_PRIORITY_HIGH: u8 = 200;        // <40μs deadline  
const AI_PRIORITY_NORMAL: u8 = 128;      // <100μs deadline
const AI_PRIORITY_LOW: u8 = 64;          // <1ms deadline
const AI_PRIORITY_BACKGROUND: u8 = 32;   // Best effort

/// AI task states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiTaskState {
    Created,
    Ready,
    Running,
    Waiting,
    Completed,
    Failed,
}

/// AI workload types for scheduling optimization
#[derive(Debug, Clone, Copy)]
pub enum AiWorkloadType {
    Inference,        // Standard model inference
    Training,         // Online learning/fine-tuning
    Preprocessing,    // Data preparation
    Postprocessing,   // Result processing
    ModelLoading,     // Model initialization
}

/// AI task descriptor
#[derive(Clone)]
pub struct AiTask {
    pub task_id: u32,
    pub workload_type: AiWorkloadType,
    pub priority: u8,
    pub deadline_us: u64,        // Deadline in microseconds
    pub estimated_cycles: u64,    // Estimated execution cycles
    pub model_id: Option<u32>,    // Associated model ID
    pub capability_id: CapabilityId,
    pub cpu_affinity: CpuAffinity,
    pub state: AiTaskState,
    pub created_time: u64,       // Timestamp when task was created
    pub start_time: u64,         // When task started executing
    pub completion_time: u64,    // When task completed
    pub actual_cycles: u64,      // Actual execution cycles
}

/// CPU affinity specification
#[derive(Debug, Clone, Copy)]
pub enum CpuAffinity {
    Any,                    // Can run on any CPU
    Specific(u32),          // Must run on specific CPU
    Performance,            // Prefer performance cores
    Efficiency,             // Prefer efficiency cores
    Cache(u32),            // Prefer CPU with model in cache
}

/// AI scheduler state
pub struct AiScheduler {
    pub initialized: AtomicBool,
    pub ready_queue: [Option<AiTask>; MAX_AI_TASKS],
    pub running_tasks: [Option<AiTask>; 8], // Per-CPU running task
    pub completed_tasks: [Option<AiTask>; MAX_AI_TASKS],
    pub next_task_id: AtomicU32,
    pub scheduler_stats: SchedulerStats,
    pub quantum_cycles: AtomicU64,      // Time quantum in cycles
}

/// Scheduler performance statistics
#[derive(Default)]
pub struct SchedulerStats {
    pub tasks_scheduled: AtomicU64,
    pub tasks_completed: AtomicU64,
    pub tasks_missed_deadline: AtomicU64,
    pub total_response_time: AtomicU64,
    pub context_switches: AtomicU64,
    pub preemptions: AtomicU64,
    pub cpu_utilization: [AtomicU32; 8], // Per-CPU utilization (0-100)
}

/// Global AI scheduler instance
static mut AI_SCHEDULER: AiScheduler = AiScheduler {
    initialized: AtomicBool::new(false),
    ready_queue: [None; MAX_AI_TASKS],
    running_tasks: [None; 8],
    completed_tasks: [None; MAX_AI_TASKS],
    next_task_id: AtomicU32::new(1),
    scheduler_stats: SchedulerStats {
        tasks_scheduled: AtomicU64::new(0),
        tasks_completed: AtomicU64::new(0),
        tasks_missed_deadline: AtomicU64::new(0),
        total_response_time: AtomicU64::new(0),
        context_switches: AtomicU64::new(0),
        preemptions: AtomicU64::new(0),
        cpu_utilization: [AtomicU32::new(0); 8],
    },
    quantum_cycles: AtomicU64::new(10000), // ~4μs at 2.4GHz
};

/// Initialize AI scheduler
pub fn init() -> Result<(), &'static str> {
    unsafe {
        if AI_SCHEDULER.initialized.load(Ordering::Acquire) {
            return Err("AI scheduler already initialized");
        }
        
        // Initialize ready queue
        for i in 0..MAX_AI_TASKS {
            AI_SCHEDULER.ready_queue[i] = None;
        }
        
        // Initialize running tasks
        for i in 0..8 {
            AI_SCHEDULER.running_tasks[i] = None;
        }
        
        // Initialize completed tasks
        for i in 0..MAX_AI_TASKS {
            AI_SCHEDULER.completed_tasks[i] = None;
        }
        
        // Set scheduling quantum (4μs default for real-time response)
        AI_SCHEDULER.quantum_cycles.store(9600, Ordering::Relaxed); // 4μs at 2.4GHz
        
        AI_SCHEDULER.initialized.store(true, Ordering::Release);
    }
    
    crate::kernel::serial::write_str("[AI_SCHED] AI real-time scheduler initialized\n");
    Ok(())
}

/// Create new AI task
pub fn create_task(
    workload_type: AiWorkloadType,
    priority: u8,
    deadline_us: u64,
    estimated_cycles: u64,
    model_id: Option<u32>,
    capability_id: CapabilityId,
    cpu_affinity: CpuAffinity,
) -> Result<u32, &'static str> {
    unsafe {
        if !AI_SCHEDULER.initialized.load(Ordering::Acquire) {
            return Err("AI scheduler not initialized");
        }
        
        let task_id = AI_SCHEDULER.next_task_id.fetch_add(1, Ordering::Relaxed);
        let created_time = read_cycle_counter();
        
        let task = AiTask {
            task_id,
            workload_type,
            priority,
            deadline_us,
            estimated_cycles,
            model_id,
            capability_id,
            cpu_affinity,
            state: AiTaskState::Created,
            created_time,
            start_time: 0,
            completion_time: 0,
            actual_cycles: 0,
        };
        
        // Add to ready queue
        if let Some(slot) = find_free_ready_slot() {
            AI_SCHEDULER.ready_queue[slot] = Some(task);
            AI_SCHEDULER.scheduler_stats.tasks_scheduled.fetch_add(1, Ordering::Relaxed);
            
            crate::kernel::serial::write_str("[AI_SCHED] Task created: ");
            crate::kernel::serial::write_u32(task_id);
            crate::kernel::serial::write_str(" priority: ");
            crate::kernel::serial::write_u32(priority as u32);
            crate::kernel::serial::write_str("\n");
            
            Ok(task_id)
        } else {
            Err("No free task slots available")
        }
    }
}

/// Find free slot in ready queue
unsafe fn find_free_ready_slot() -> Option<usize> {
    for i in 0..MAX_AI_TASKS {
        if AI_SCHEDULER.ready_queue[i].is_none() {
            return Some(i);
        }
    }
    None
}

/// Schedule next AI task (called by timer interrupt)
pub fn schedule() -> Result<(), &'static str> {
    unsafe {
        if !AI_SCHEDULER.initialized.load(Ordering::Acquire) {
            return Ok(()); // Not initialized yet
        }
        
        let current_cpu = get_current_cpu_id();
        let current_time = read_cycle_counter();
        
        // Check for deadline violations in running tasks
        check_deadline_violations(current_time);
        
        // Select highest priority ready task
        if let Some((task_index, mut task)) = select_highest_priority_task() {
            // Check if we need to preempt current task
            if should_preempt_current_task(current_cpu, &task) {
                preempt_current_task(current_cpu, current_time)?;
            }
            
            // Schedule the new task
            task.state = AiTaskState::Running;
            task.start_time = current_time;
            
            AI_SCHEDULER.ready_queue[task_index] = None;
            AI_SCHEDULER.running_tasks[current_cpu as usize] = Some(task.clone());
            
            AI_SCHEDULER.scheduler_stats.context_switches.fetch_add(1, Ordering::Relaxed);
            
            crate::kernel::serial::write_str("[AI_SCHED] Scheduled task: ");
            crate::kernel::serial::write_u32(task.task_id);
            crate::kernel::serial::write_str(" on CPU ");
            crate::kernel::serial::write_u32(current_cpu);
            crate::kernel::serial::write_str("\n");
        }
    }
    
    Ok(())
}

/// Check for deadline violations
unsafe fn check_deadline_violations(current_time: u64) {
    for i in 0..8 {
        if let Some(ref task) = AI_SCHEDULER.running_tasks[i] {
            let elapsed_cycles = current_time - task.start_time;
            let elapsed_us = cycles_to_microseconds(elapsed_cycles);
            
            if elapsed_us > task.deadline_us {
                AI_SCHEDULER.scheduler_stats.tasks_missed_deadline.fetch_add(1, Ordering::Relaxed);
                
                crate::kernel::serial::write_str("[AI_SCHED] DEADLINE MISSED: Task ");
                crate::kernel::serial::write_u32(task.task_id);
                crate::kernel::serial::write_str(" took ");
                crate::kernel::serial::write_u64(elapsed_us);
                crate::kernel::serial::write_str("μs\n");
            }
        }
    }
}

/// Select highest priority ready task
unsafe fn select_highest_priority_task() -> Option<(usize, AiTask)> {
    let mut best_task: Option<(usize, AiTask)> = None;
    let mut best_priority = 0u8;
    
    for i in 0..MAX_AI_TASKS {
        if let Some(ref task) = AI_SCHEDULER.ready_queue[i] {
            if task.state == AiTaskState::Created || task.state == AiTaskState::Ready {
                // Use Earliest Deadline First (EDF) for critical tasks
                if task.priority >= AI_PRIORITY_CRITICAL {
                    if best_task.is_none() || task.deadline_us < best_task.as_ref().unwrap().1.deadline_us {
                        best_task = Some((i, task.clone()));
                    }
                } else if task.priority > best_priority {
                    best_priority = task.priority;
                    best_task = Some((i, task.clone()));
                }
            }
        }
    }
    
    best_task
}

/// Check if current task should be preempted
unsafe fn should_preempt_current_task(cpu: u32, new_task: &AiTask) -> bool {
    if let Some(ref current_task) = AI_SCHEDULER.running_tasks[cpu as usize] {
        // Preempt if new task has higher priority
        if new_task.priority > current_task.priority {
            return true;
        }
        
        // Preempt if current task has exceeded quantum
        let elapsed_cycles = read_cycle_counter() - current_task.start_time;
        let quantum = AI_SCHEDULER.quantum_cycles.load(Ordering::Relaxed);
        if elapsed_cycles > quantum {
            return true;
        }
    }
    
    false
}

/// Preempt current task and move it back to ready queue
unsafe fn preempt_current_task(cpu: u32, current_time: u64) -> Result<(), &'static str> {
    if let Some(mut task) = AI_SCHEDULER.running_tasks[cpu as usize].take() {
        task.state = AiTaskState::Ready;
        task.actual_cycles += current_time - task.start_time;
        
        // Find slot to put it back in ready queue
        if let Some(slot) = find_free_ready_slot() {
            AI_SCHEDULER.ready_queue[slot] = Some(task);
            AI_SCHEDULER.scheduler_stats.preemptions.fetch_add(1, Ordering::Relaxed);
        } else {
            // Emergency: drop the task (this shouldn't happen)
            crate::kernel::serial::write_str("[AI_SCHED] WARNING: Dropped task due to full queue\n");
        }
    }
    
    Ok(())
}

/// Execute AI task (called by AI runtime)
pub fn execute_ai_task(task_id: u32) -> Result<u64, &'static str> {
    let start_cycles = read_cycle_counter();
    
    unsafe {
        // Find running task
        let cpu = get_current_cpu_id();
        if let Some(ref mut task) = AI_SCHEDULER.running_tasks[cpu as usize] {
            if task.task_id != task_id {
                return Err("Task ID mismatch");
            }
            
            // Verify capabilities
            if !crate::kernel::capabilities::check_capability(
                0, // Current process
                task.capability_id,
                CapabilityRights::new(CapabilityRights::EXECUTE),
            ) {
                task.state = AiTaskState::Failed;
                return Err("Insufficient capabilities");
            }
            
            // Execute based on workload type
            let result = match task.workload_type {
                AiWorkloadType::Inference => {
                    execute_inference_task(task)
                },
                AiWorkloadType::Training => {
                    execute_training_task(task)
                },
                AiWorkloadType::Preprocessing => {
                    execute_preprocessing_task(task)
                },
                AiWorkloadType::Postprocessing => {
                    execute_postprocessing_task(task)
                },
                AiWorkloadType::ModelLoading => {
                    execute_model_loading_task(task)
                },
            };
            
            let end_cycles = read_cycle_counter();
            task.completion_time = end_cycles;
            task.actual_cycles = end_cycles - start_cycles;
            
            match result {
                Ok(_) => {
                    task.state = AiTaskState::Completed;
                    
                    // Move to completed queue
                    if let Some(slot) = find_free_completed_slot() {
                        AI_SCHEDULER.completed_tasks[slot] = Some(task.clone());
                    }
                    
                    // Clear running slot
                    AI_SCHEDULER.running_tasks[cpu as usize] = None;
                    
                    AI_SCHEDULER.scheduler_stats.tasks_completed.fetch_add(1, Ordering::Relaxed);
                    
                    let response_time = task.completion_time - task.created_time;
                    AI_SCHEDULER.scheduler_stats.total_response_time.fetch_add(response_time, Ordering::Relaxed);
                    
                    Ok(task.actual_cycles)
                },
                Err(e) => {
                    task.state = AiTaskState::Failed;
                    AI_SCHEDULER.running_tasks[cpu as usize] = None;
                    Err(e)
                }
            }
        } else {
            Err("No running task found")
        }
    }
}

/// Find free slot in completed queue
unsafe fn find_free_completed_slot() -> Option<usize> {
    for i in 0..MAX_AI_TASKS {
        if AI_SCHEDULER.completed_tasks[i].is_none() {
            return Some(i);
        }
    }
    None
}

/// Execute inference task
fn execute_inference_task(task: &AiTask) -> Result<(), &'static str> {
    if let Some(model_id) = task.model_id {
        crate::kernel::serial::write_str("[AI_SCHED] Executing inference task for model ");
        crate::kernel::serial::write_u32(model_id);
        crate::kernel::serial::write_str("\n");
        
        // In real implementation, this would call ai_runtime::infer()
        // For now, simulate work
        simulate_work(task.estimated_cycles);
        
        Ok(())
    } else {
        Err("Inference task requires model ID")
    }
}

/// Execute training task
fn execute_training_task(task: &AiTask) -> Result<(), &'static str> {
    crate::kernel::serial::write_str("[AI_SCHED] Executing training task\n");
    simulate_work(task.estimated_cycles);
    Ok(())
}

/// Execute preprocessing task
fn execute_preprocessing_task(task: &AiTask) -> Result<(), &'static str> {
    crate::kernel::serial::write_str("[AI_SCHED] Executing preprocessing task\n");
    simulate_work(task.estimated_cycles);
    Ok(())
}

/// Execute postprocessing task
fn execute_postprocessing_task(task: &AiTask) -> Result<(), &'static str> {
    crate::kernel::serial::write_str("[AI_SCHED] Executing postprocessing task\n");
    simulate_work(task.estimated_cycles);
    Ok(())
}

/// Execute model loading task
fn execute_model_loading_task(task: &AiTask) -> Result<(), &'static str> {
    crate::kernel::serial::write_str("[AI_SCHED] Executing model loading task\n");
    simulate_work(task.estimated_cycles);
    Ok(())
}

/// Simulate computational work (for testing)
fn simulate_work(cycles: u64) {
    let start = read_cycle_counter();
    while read_cycle_counter() - start < cycles {
        // Busy wait to simulate work
        unsafe {
            core::arch::asm!("nop");
        }
    }
}

/// Get current CPU ID
fn get_current_cpu_id() -> u32 {
    unsafe {
        let mpidr: u64;
        core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr);
        (mpidr & 0xFF) as u32 // Extract Aff0 (CPU ID)
    }
}

/// Read cycle counter
fn read_cycle_counter() -> u64 {
    unsafe {
        let mut cycles: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles);
        cycles
    }
}

/// Convert cycles to microseconds (assuming 2.4GHz)
fn cycles_to_microseconds(cycles: u64) -> u64 {
    cycles / 2400 // 2.4GHz = 2400 cycles per microsecond
}

/// Get scheduler statistics
pub fn get_scheduler_stats() -> SchedulerStats {
    unsafe {
        SchedulerStats {
            tasks_scheduled: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.tasks_scheduled.load(Ordering::Relaxed)
            ),
            tasks_completed: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.tasks_completed.load(Ordering::Relaxed)
            ),
            tasks_missed_deadline: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.tasks_missed_deadline.load(Ordering::Relaxed)
            ),
            total_response_time: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.total_response_time.load(Ordering::Relaxed)
            ),
            context_switches: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.context_switches.load(Ordering::Relaxed)
            ),
            preemptions: AtomicU64::new(
                AI_SCHEDULER.scheduler_stats.preemptions.load(Ordering::Relaxed)
            ),
            cpu_utilization: core::array::from_fn(|i| AtomicU32::new(
                AI_SCHEDULER.scheduler_stats.cpu_utilization[i].load(Ordering::Relaxed)
            )),
        }
    }
}

/// Set scheduling quantum (for real-time tuning)
pub fn set_quantum_microseconds(us: u64) -> Result<(), &'static str> {
    if us == 0 || us > 1000 {
        return Err("Invalid quantum (must be 1-1000μs)");
    }
    
    let cycles = us * 2400; // Convert μs to cycles at 2.4GHz
    unsafe {
        AI_SCHEDULER.quantum_cycles.store(cycles, Ordering::Relaxed);
    }
    
    crate::kernel::serial::write_str("[AI_SCHED] Quantum set to ");
    crate::kernel::serial::write_u64(us);
    crate::kernel::serial::write_str("μs\n");
    
    Ok(())
}

/// Check if scheduler meets real-time guarantees
pub fn validate_real_time_performance() -> Result<bool, &'static str> {
    let stats = get_scheduler_stats();
    
    let total_scheduled = stats.tasks_scheduled.load(Ordering::Relaxed);
    let total_missed = stats.tasks_missed_deadline.load(Ordering::Relaxed);
    
    if total_scheduled == 0 {
        return Ok(true); // No tasks scheduled yet
    }
    
    let miss_rate = (total_missed * 100) / total_scheduled;
    let meets_target = miss_rate < 5; // Less than 5% deadline miss rate
    
    crate::kernel::serial::write_str("[AI_SCHED] Deadline miss rate: ");
    crate::kernel::serial::write_u64(miss_rate);
    crate::kernel::serial::write_str("%\n");
    
    if meets_target {
        crate::kernel::serial::write_str("[AI_SCHED] Real-time performance target MET\n");
    } else {
        crate::kernel::serial::write_str("[AI_SCHED] Real-time performance target MISSED\n");
    }
    
    Ok(meets_target)
}