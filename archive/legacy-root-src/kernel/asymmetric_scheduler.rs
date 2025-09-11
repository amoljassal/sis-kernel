//! Asymmetric Scheduler for Analytical vs Creative Tasks
//! Implements dual-hemisphere scheduling with hardware awareness and performance optimization

use core::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::RwLock;

use crate::kernel::cognitive_runtime::{CognitiveTask, TaskType, Hemisphere, Priority};

/// Asymmetric scheduler optimizing for analytical vs creative workloads
pub struct AsymmetricScheduler {
    /// Analytical (Left Hemisphere) scheduler
    pub analytical: AnalyticalScheduler,
    /// Creative (Right Hemisphere) scheduler  
    pub creative: CreativeScheduler,
    /// Load balancer between hemispheres
    pub load_balancer: HemisphereLoadBalancer,
    /// Performance monitoring
    pub perf_monitor: PerformanceMonitor,
    /// Scheduling policies
    pub policies: SchedulingPolicies,
}

impl AsymmetricScheduler {
    pub fn new() -> Self {
        Self {
            analytical: AnalyticalScheduler::new(),
            creative: CreativeScheduler::new(),
            load_balancer: HemisphereLoadBalancer::new(),
            perf_monitor: PerformanceMonitor::new(),
            policies: SchedulingPolicies::default(),
        }
    }

    /// Schedule a task to the appropriate hemisphere
    pub fn schedule(&mut self, task: CognitiveTask) -> Result<(), SchedulerError> {
        // Update performance metrics
        self.perf_monitor.record_task_arrival(&task);
        
        // Determine optimal hemisphere
        let hemisphere = self.determine_hemisphere(&task)?;
        
        // Route to appropriate scheduler
        match hemisphere {
            Hemisphere::Left => {
                self.analytical.enqueue(task)?;
            }
            Hemisphere::Right => {
                self.creative.enqueue(task)?;
            }
            Hemisphere::Both => {
                // Split task across both hemispheres
                self.schedule_hybrid(task)?;
            }
        }
        
        // Update load balancer
        self.load_balancer.update_load(&self.analytical, &self.creative);
        
        Ok(())
    }

    /// Get next task for execution from either hemisphere
    pub fn get_next_task(&mut self, preferred_hemisphere: Hemisphere) 
        -> Option<(CognitiveTask, Hemisphere)> {
        
        match preferred_hemisphere {
            Hemisphere::Left => {
                // Check analytical queue first, then creative if analytical is idle
                if let Some(task) = self.analytical.dequeue() {
                    Some((task, Hemisphere::Left))
                } else if self.policies.allow_cross_hemisphere {
                    self.creative.dequeue().map(|t| (t, Hemisphere::Right))
                } else {
                    None
                }
            }
            Hemisphere::Right => {
                // Check creative queue first, then analytical if creative is idle
                if let Some(task) = self.creative.dequeue() {
                    Some((task, Hemisphere::Right))
                } else if self.policies.allow_cross_hemisphere {
                    self.analytical.dequeue().map(|t| (t, Hemisphere::Left))
                } else {
                    None
                }
            }
            Hemisphere::Both => {
                // Load-balanced selection
                self.load_balancer.select_next_task(&mut self.analytical, &mut self.creative)
            }
        }
    }

    fn determine_hemisphere(&self, task: &CognitiveTask) -> Result<Hemisphere, SchedulerError> {
        match self.policies.assignment_strategy {
            AssignmentStrategy::Static => self.static_assignment(task),
            AssignmentStrategy::LoadBalanced => self.load_balanced_assignment(task),
            AssignmentStrategy::PerformanceBased => self.performance_based_assignment(task),
            AssignmentStrategy::Adaptive => self.adaptive_assignment(task),
        }
    }

    fn static_assignment(&self, task: &CognitiveTask) -> Result<Hemisphere, SchedulerError> {
        match task.task_type {
            TaskType::Analytical | TaskType::Sequential => Ok(Hemisphere::Left),
            TaskType::Creative | TaskType::Parallel => Ok(Hemisphere::Right),
            TaskType::Hybrid => Ok(Hemisphere::Both),
        }
    }

    fn load_balanced_assignment(&self, task: &CognitiveTask) -> Result<Hemisphere, SchedulerError> {
        let left_load = self.analytical.get_load();
        let right_load = self.creative.get_load();
        
        // Prefer hemisphere with lower load, but respect task type
        if task.task_type == TaskType::Hybrid {
            Ok(Hemisphere::Both)
        } else if left_load < right_load {
            Ok(Hemisphere::Left)
        } else {
            Ok(Hemisphere::Right)
        }
    }

    fn performance_based_assignment(&self, task: &CognitiveTask) -> Result<Hemisphere, SchedulerError> {
        // Use historical performance data
        let left_perf = self.perf_monitor.get_hemisphere_performance(Hemisphere::Left, &task.task_type);
        let right_perf = self.perf_monitor.get_hemisphere_performance(Hemisphere::Right, &task.task_type);
        
        if left_perf > right_perf {
            Ok(Hemisphere::Left)
        } else {
            Ok(Hemisphere::Right)
        }
    }

    fn adaptive_assignment(&self, task: &CognitiveTask) -> Result<Hemisphere, SchedulerError> {
        // Combine load balancing with performance history
        let static_pref = self.static_assignment(task)?;
        let load_pref = self.load_balanced_assignment(task)?;
        let perf_pref = self.performance_based_assignment(task)?;
        
        // Weighted decision
        if static_pref == load_pref && load_pref == perf_pref {
            Ok(static_pref)
        } else {
            // Fallback to load balancing
            Ok(load_pref)
        }
    }

    fn schedule_hybrid(&mut self, task: CognitiveTask) -> Result<(), SchedulerError> {
        // Split hybrid task into analytical and creative components
        let (analytical_part, creative_part) = self.split_hybrid_task(task)?;
        
        self.analytical.enqueue(analytical_part)?;
        self.creative.enqueue(creative_part)?;
        
        Ok(())
    }

    fn split_hybrid_task(&self, task: CognitiveTask) -> Result<(CognitiveTask, CognitiveTask), SchedulerError> {
        // Create analytical component
        let analytical_task = CognitiveTask {
            id: task.id * 2,  // Ensure unique IDs
            task_type: TaskType::Analytical,
            priority: task.priority,
            query: task.query.clone(),
            prompt: task.prompt.clone(),
            data: task.data.clone(),
            deadline: task.deadline,
        };
        
        // Create creative component
        let creative_task = CognitiveTask {
            id: task.id * 2 + 1,
            task_type: TaskType::Creative,
            priority: task.priority,
            query: task.query,
            prompt: task.prompt,
            data: task.data,
            deadline: task.deadline,
        };
        
        Ok((analytical_task, creative_task))
    }
}

/// Scheduler optimized for analytical tasks (sequential, logical)
pub struct AnalyticalScheduler {
    /// Priority queues for analytical tasks
    queues: BTreeMap<Priority, AnalyticalQueue>,
    /// Sequential execution policy
    execution_policy: SequentialPolicy,
    /// Current load
    load: AtomicUsize,
    /// Performance metrics
    metrics: AnalyticalMetrics,
}

impl AnalyticalScheduler {
    pub fn new() -> Self {
        let mut queues = BTreeMap::new();
        for priority in [Priority::Realtime, Priority::High, Priority::Normal, Priority::Low] {
            queues.insert(priority, AnalyticalQueue::new());
        }
        
        Self {
            queues,
            execution_policy: SequentialPolicy::FairShare,
            load: AtomicUsize::new(0),
            metrics: AnalyticalMetrics::new(),
        }
    }

    pub fn enqueue(&mut self, task: CognitiveTask) -> Result<(), SchedulerError> {
        let queue = self.queues.get_mut(&task.priority)
            .ok_or(SchedulerError::InvalidPriority)?;
        
        queue.enqueue(task);
        self.load.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<CognitiveTask> {
        // Check queues in priority order
        for (_, queue) in self.queues.iter_mut().rev() {
            if let Some(task) = queue.dequeue() {
                self.load.fetch_sub(1, Ordering::Relaxed);
                self.metrics.record_dequeue(&task);
                return Some(task);
            }
        }
        None
    }

    pub fn get_load(&self) -> usize {
        self.load.load(Ordering::Relaxed)
    }
}

/// Scheduler optimized for creative tasks (parallel, intuitive)
pub struct CreativeScheduler {
    /// Work-stealing queues for parallel execution
    work_queues: Vec<CreativeWorkQueue>,
    /// Parallel execution policy
    execution_policy: ParallelPolicy,
    /// Current load across all queues
    total_load: AtomicUsize,
    /// Performance metrics
    metrics: CreativeMetrics,
}

impl CreativeScheduler {
    pub fn new() -> Self {
        // Create multiple work queues for parallelism
        let num_queues = 4;  // Adjust based on hardware
        let mut work_queues = Vec::with_capacity(num_queues);
        
        for i in 0..num_queues {
            work_queues.push(CreativeWorkQueue::new(i));
        }
        
        Self {
            work_queues,
            execution_policy: ParallelPolicy::WorkStealing,
            total_load: AtomicUsize::new(0),
            metrics: CreativeMetrics::new(),
        }
    }

    pub fn enqueue(&mut self, task: CognitiveTask) -> Result<(), SchedulerError> {
        // Find queue with minimum load
        let mut min_load = usize::MAX;
        let mut best_queue = 0;
        
        for (i, queue) in self.work_queues.iter().enumerate() {
            let load = queue.get_load();
            if load < min_load {
                min_load = load;
                best_queue = i;
            }
        }
        
        self.work_queues[best_queue].enqueue(task);
        self.total_load.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<CognitiveTask> {
        // Try work stealing approach
        match self.execution_policy {
            ParallelPolicy::WorkStealing => {
                self.work_stealing_dequeue()
            }
            ParallelPolicy::RoundRobin => {
                self.round_robin_dequeue()
            }
        }
    }

    fn work_stealing_dequeue(&mut self) -> Option<CognitiveTask> {
        // First try local queues
        for queue in &mut self.work_queues {
            if let Some(task) = queue.try_dequeue() {
                self.total_load.fetch_sub(1, Ordering::Relaxed);
                self.metrics.record_dequeue(&task);
                return Some(task);
            }
        }
        
        // Then try stealing from other queues
        for i in 0..self.work_queues.len() {
            for j in 0..self.work_queues.len() {
                if i != j {
                    if let Some(task) = self.work_queues[j].try_steal() {
                        self.total_load.fetch_sub(1, Ordering::Relaxed);
                        self.metrics.record_steal(&task);
                        return Some(task);
                    }
                }
            }
        }
        
        None
    }

    fn round_robin_dequeue(&mut self) -> Option<CognitiveTask> {
        static NEXT_QUEUE: AtomicUsize = AtomicUsize::new(0);
        
        let start_queue = NEXT_QUEUE.load(Ordering::Relaxed);
        
        for i in 0..self.work_queues.len() {
            let queue_idx = (start_queue + i) % self.work_queues.len();
            
            if let Some(task) = self.work_queues[queue_idx].try_dequeue() {
                NEXT_QUEUE.store((queue_idx + 1) % self.work_queues.len(), Ordering::Relaxed);
                self.total_load.fetch_sub(1, Ordering::Relaxed);
                return Some(task);
            }
        }
        
        None
    }

    pub fn get_load(&self) -> usize {
        self.total_load.load(Ordering::Relaxed)
    }
}

/// Load balancer between hemispheres
pub struct HemisphereLoadBalancer {
    /// Current load distribution
    load_distribution: RwLock<LoadDistribution>,
    /// Balancing strategy
    strategy: BalancingStrategy,
    /// Migration threshold
    migration_threshold: f32,
}

impl HemisphereLoadBalancer {
    pub fn new() -> Self {
        Self {
            load_distribution: RwLock::new(LoadDistribution::default()),
            strategy: BalancingStrategy::ThresholdBased,
            migration_threshold: 0.3,  // 30% load imbalance triggers migration
        }
    }

    pub fn update_load(&self, analytical: &AnalyticalScheduler, creative: &CreativeScheduler) {
        let mut dist = self.load_distribution.write();
        dist.left_load = analytical.get_load();
        dist.right_load = creative.get_load();
        dist.last_update = Self::current_time();
    }

    pub fn select_next_task(&self, analytical: &mut AnalyticalScheduler, creative: &mut CreativeScheduler) 
        -> Option<(CognitiveTask, Hemisphere)> {
        
        match self.strategy {
            BalancingStrategy::RoundRobin => {
                static LEFT_TURN: AtomicBool = AtomicBool::new(true);
                
                if LEFT_TURN.load(Ordering::Relaxed) {
                    LEFT_TURN.store(false, Ordering::Relaxed);
                    analytical.dequeue().map(|t| (t, Hemisphere::Left))
                        .or_else(|| creative.dequeue().map(|t| (t, Hemisphere::Right)))
                } else {
                    LEFT_TURN.store(true, Ordering::Relaxed);
                    creative.dequeue().map(|t| (t, Hemisphere::Right))
                        .or_else(|| analytical.dequeue().map(|t| (t, Hemisphere::Left)))
                }
            }
            BalancingStrategy::LoadBased => {
                let dist = self.load_distribution.read();
                if dist.left_load <= dist.right_load {
                    analytical.dequeue().map(|t| (t, Hemisphere::Left))
                        .or_else(|| creative.dequeue().map(|t| (t, Hemisphere::Right)))
                } else {
                    creative.dequeue().map(|t| (t, Hemisphere::Right))
                        .or_else(|| analytical.dequeue().map(|t| (t, Hemisphere::Left)))
                }
            }
            BalancingStrategy::ThresholdBased => {
                let dist = self.load_distribution.read();
                let total = dist.left_load + dist.right_load;
                
                if total == 0 {
                    return None;
                }
                
                let imbalance = (dist.left_load as f32 - dist.right_load as f32).abs() / total as f32;
                
                if imbalance > self.migration_threshold {
                    // Prefer underloaded hemisphere
                    if dist.left_load < dist.right_load {
                        creative.dequeue().map(|t| (t, Hemisphere::Right))  // Move from overloaded
                            .or_else(|| analytical.dequeue().map(|t| (t, Hemisphere::Left)))
                    } else {
                        analytical.dequeue().map(|t| (t, Hemisphere::Left))  // Move from overloaded
                            .or_else(|| creative.dequeue().map(|t| (t, Hemisphere::Right)))
                    }
                } else {
                    // Normal round-robin
                    analytical.dequeue().map(|t| (t, Hemisphere::Left))
                        .or_else(|| creative.dequeue().map(|t| (t, Hemisphere::Right)))
                }
            }
        }
    }

    fn current_time() -> u64 {
        // Would use actual system time
        0
    }
}

/// Performance monitoring for scheduling decisions
pub struct PerformanceMonitor {
    /// Task execution history
    history: RwLock<TaskHistory>,
    /// Performance metrics per hemisphere and task type
    hemisphere_perf: RwLock<BTreeMap<(Hemisphere, TaskType), PerformanceMetrics>>,
    /// Window size for moving averages
    window_size: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(TaskHistory::new()),
            hemisphere_perf: RwLock::new(BTreeMap::new()),
            window_size: 100,  // 100 tasks
        }
    }

    pub fn record_task_arrival(&self, task: &CognitiveTask) {
        let mut history = self.history.write();
        history.record_arrival(task.id, Self::current_time());
    }

    pub fn record_task_completion(&self, task_id: u64, hemisphere: Hemisphere, latency: u64) {
        let mut history = self.history.write();
        let task_info = history.get_task(task_id);
        
        if let Some(info) = task_info {
            let key = (hemisphere, info.task_type);
            let mut perf = self.hemisphere_perf.write();
            
            if !perf.contains_key(&key) {
                perf.insert(key, PerformanceMetrics::new());
            }
            perf.get_mut(&key).unwrap().record_completion(latency);
        }
    }

    pub fn get_hemisphere_performance(&self, hemisphere: Hemisphere, task_type: &TaskType) -> f64 {
        let perf = self.hemisphere_perf.read();
        let key = (hemisphere, *task_type);
        
        perf.get(&key)
            .map(|m| m.average_latency())
            .unwrap_or(f64::MAX)
    }

    fn current_time() -> u64 {
        0  // Would use actual system time
    }
}

// Supporting structures

pub struct AnalyticalQueue {
    tasks: Vec<CognitiveTask>,
    fair_share_index: AtomicUsize,
}

impl AnalyticalQueue {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            fair_share_index: AtomicUsize::new(0),
        }
    }

    fn enqueue(&mut self, task: CognitiveTask) {
        // Insert in priority order
        let pos = self.tasks.iter().position(|t| t.priority < task.priority)
            .unwrap_or(self.tasks.len());
        self.tasks.insert(pos, task);
    }

    fn dequeue(&mut self) -> Option<CognitiveTask> {
        if self.tasks.is_empty() {
            None
        } else {
            Some(self.tasks.remove(0))
        }
    }
}

pub struct CreativeWorkQueue {
    id: usize,
    local_queue: Vec<CognitiveTask>,
    steal_queue: Vec<CognitiveTask>,
    load: AtomicUsize,
}

impl CreativeWorkQueue {
    fn new(id: usize) -> Self {
        Self {
            id,
            local_queue: Vec::new(),
            steal_queue: Vec::new(),
            load: AtomicUsize::new(0),
        }
    }

    fn enqueue(&mut self, task: CognitiveTask) {
        self.local_queue.push(task);
        self.load.fetch_add(1, Ordering::Relaxed);
    }

    fn try_dequeue(&mut self) -> Option<CognitiveTask> {
        if let Some(task) = self.local_queue.pop() {
            self.load.fetch_sub(1, Ordering::Relaxed);
            Some(task)
        } else {
            None
        }
    }

    fn try_steal(&mut self) -> Option<CognitiveTask> {
        // Steal from the front (oldest task)
        if !self.local_queue.is_empty() {
            let task = self.local_queue.remove(0);
            self.load.fetch_sub(1, Ordering::Relaxed);
            Some(task)
        } else {
            None
        }
    }

    fn get_load(&self) -> usize {
        self.load.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct LoadDistribution {
    left_load: usize,
    right_load: usize,
    last_update: u64,
}

struct TaskHistory {
    tasks: BTreeMap<u64, TaskInfo>,
}

impl TaskHistory {
    fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
        }
    }

    fn record_arrival(&mut self, task_id: u64, arrival_time: u64) {
        self.tasks.insert(task_id, TaskInfo {
            arrival_time,
            task_type: TaskType::Analytical,  // Would be properly set
        });
    }

    fn get_task(&self, task_id: u64) -> Option<&TaskInfo> {
        self.tasks.get(&task_id)
    }
}

struct TaskInfo {
    arrival_time: u64,
    task_type: TaskType,
}

struct PerformanceMetrics {
    total_latency: u64,
    completed_tasks: u64,
    latency_history: Vec<u64>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            total_latency: 0,
            completed_tasks: 0,
            latency_history: Vec::new(),
        }
    }

    fn record_completion(&mut self, latency: u64) {
        self.total_latency += latency;
        self.completed_tasks += 1;
        self.latency_history.push(latency);
        
        // Keep only recent history
        if self.latency_history.len() > 100 {
            self.latency_history.remove(0);
        }
    }

    fn average_latency(&self) -> f64 {
        if self.completed_tasks == 0 {
            f64::MAX
        } else {
            self.total_latency as f64 / self.completed_tasks as f64
        }
    }
}

struct AnalyticalMetrics {
    tasks_processed: AtomicU64,
    sequential_efficiency: AtomicU64,
}

impl AnalyticalMetrics {
    fn new() -> Self {
        Self {
            tasks_processed: AtomicU64::new(0),
            sequential_efficiency: AtomicU64::new(0),
        }
    }

    fn record_dequeue(&self, task: &CognitiveTask) {
        self.tasks_processed.fetch_add(1, Ordering::Relaxed);
    }
}

struct CreativeMetrics {
    tasks_processed: AtomicU64,
    steals_performed: AtomicU64,
    parallel_efficiency: AtomicU64,
}

impl CreativeMetrics {
    fn new() -> Self {
        Self {
            tasks_processed: AtomicU64::new(0),
            steals_performed: AtomicU64::new(0),
            parallel_efficiency: AtomicU64::new(0),
        }
    }

    fn record_dequeue(&self, task: &CognitiveTask) {
        self.tasks_processed.fetch_add(1, Ordering::Relaxed);
    }

    fn record_steal(&self, task: &CognitiveTask) {
        self.steals_performed.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Default)]
pub struct SchedulingPolicies {
    pub assignment_strategy: AssignmentStrategy,
    pub allow_cross_hemisphere: bool,
    pub preemption_enabled: bool,
    pub load_balancing_interval: u64,
}

#[derive(Clone, Copy)]
pub enum AssignmentStrategy {
    Static,
    LoadBalanced,
    PerformanceBased,
    Adaptive,
}

impl Default for AssignmentStrategy {
    fn default() -> Self {
        Self::Adaptive
    }
}

#[derive(Clone, Copy)]
enum SequentialPolicy {
    FIFO,
    Priority,
    FairShare,
}

#[derive(Clone, Copy)]
enum ParallelPolicy {
    WorkStealing,
    RoundRobin,
}

#[derive(Clone, Copy)]
enum BalancingStrategy {
    RoundRobin,
    LoadBased,
    ThresholdBased,
}

#[derive(Debug)]
pub enum SchedulerError {
    InvalidPriority,
    QueueFull,
    TaskNotFound,
    HardwareError,
}