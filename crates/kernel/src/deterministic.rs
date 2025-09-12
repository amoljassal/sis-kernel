//! Deterministic scheduler with CBS+EDF (Phase 2):
//! - Admission control using utilization bounds
//! - CBS (Constant Bandwidth Server) per deterministic graph
//! - EDF ordering of operator activations within CBS servers
//! - Timer discipline with ARM architected timer programming
//! - Constraint enforcement (no dynamic alloc, unbounded loops, indefinite blocking)

use crate::trace::metric_kv;

#[derive(Copy, Clone)]
pub struct TaskSpec {
    pub id: u32,
    pub wcet_ns: u64,
    pub period_ns: u64,
    pub deadline_ns: u64,
}

/// Fixed-point utilization accounting (ppm = parts per million)
/// util_ppm = (wcet / period) * 1_000_000
#[derive(Copy, Clone)]
pub struct AdmissionController {
    bound_ppm: u32,    // e.g., 850_000 for 85%
    used_ppm: u32,
    accepted: u32,
    rejected: u32,
}

impl AdmissionController {
    pub const fn new(bound_ppm: u32) -> Self {
        Self { bound_ppm, used_ppm: 0, accepted: 0, rejected: 0 }
    }

    #[inline(always)]
    pub fn util_ppm(spec: &TaskSpec) -> u32 {
        if spec.period_ns == 0 { return u32::MAX; }
        let num = (spec.wcet_ns as u128) * 1_000_000u128;
        let den = spec.period_ns as u128;
        let u = num / den; // floor
        if u > u32::MAX as u128 { u32::MAX } else { u as u32 }
    }

    pub fn try_admit(&mut self, spec: &TaskSpec) -> bool {
        let u = Self::util_ppm(spec);
        let next = self.used_ppm.saturating_add(u);
        if next > self.bound_ppm { self.rejected += 1; return false; }
        self.used_ppm = next;
        self.accepted += 1;
        true
    }

    pub fn stats(&self) -> (u32, u32, u32) { (self.used_ppm, self.accepted, self.rejected) }
}

#[derive(Copy, Clone)]
pub struct EdfNode { pub id: u32, pub abs_deadline_ns: u64 }

pub struct EdfQueue<const N: usize> {
    heap: [Option<EdfNode>; N],
    len: usize,
}

impl<const N: usize> EdfQueue<N> {
    pub const fn new() -> Self { Self { heap: [None; N], len: 0 } }

    pub fn push(&mut self, n: EdfNode) -> bool {
        if self.len >= N { return false; }
        self.heap[self.len] = Some(n);
        self.sift_up(self.len);
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<EdfNode> {
        if self.len == 0 { return None; }
        let root = self.heap[0].take();
        self.len -= 1;
        if self.len > 0 {
            self.heap[0] = self.heap[self.len].take();
            self.sift_down(0);
        }
        root
    }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let p = (i - 1) / 2;
            if self.cmp(i, p) { self.heap.swap(i, p); i = p; } else { break; }
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        loop {
            let l = 2 * i + 1;
            let r = 2 * i + 2;
            let mut best = i;
            if l < self.len && self.cmp(l, best) { best = l; }
            if r < self.len && self.cmp(r, best) { best = r; }
            if best != i { self.heap.swap(i, best); i = best; } else { break; }
        }
    }

    #[inline(always)]
    fn cmp(&self, a: usize, b: usize) -> bool {
        let na = self.heap[a].unwrap();
        let nb = self.heap[b].unwrap();
        na.abs_deadline_ns < nb.abs_deadline_ns
    }
}

/// Demo: attempt to admit a few tasks and emit METRICs.
pub fn demo_admission() {
    let mut ac = AdmissionController::new(850_000); // 85%
    // Three example tasks
    let t1 = TaskSpec { id: 1, wcet_ns: 300_000, period_ns: 1_000_000, deadline_ns: 1_000_000 };
    let t2 = TaskSpec { id: 2, wcet_ns: 200_000, period_ns: 1_000_000, deadline_ns: 1_000_000 };
    let t3 = TaskSpec { id: 3, wcet_ns: 400_000, period_ns: 1_000_000, deadline_ns: 1_000_000 };

    let _ = ac.try_admit(&t1);
    let _ = ac.try_admit(&t2);
    let _ = ac.try_admit(&t3);

    let (used_ppm, acc, rej) = ac.stats();
    metric_kv("det_admission_used_ppm", used_ppm as usize);
    metric_kv("det_admission_accepted", acc as usize);
    metric_kv("det_admission_rejected", rej as usize);
}

/// Simulated EDF tick demo: schedule a few jobs by deadlines and count misses.
pub fn edf_tick_demo() {
    let mut q: EdfQueue<16> = EdfQueue::new();
    // Simulated time in ns
    let mut now_ns: u64 = 0;
    // Insert three periodic tasks with different deadlines
    let periods = [10_000u64, 15_000u64, 20_000u64];
    let mut next_dead = [10_000u64, 15_000u64, 20_000u64];
    let mut miss_count: u32 = 0;

    for _ in 0..64 {
        // Enqueue next jobs
        for (i, nd) in next_dead.iter_mut().enumerate() {
            if *nd <= now_ns {
                let _ = q.push(EdfNode { id: (i as u32) + 1, abs_deadline_ns: *nd });
                *nd = nd.saturating_add(periods[i]);
            }
        }
        // Run the earliest-deadline job if any
        if let Some(job) = q.pop() {
            // If deadline already passed, count a miss
            if job.abs_deadline_ns < now_ns { miss_count = miss_count.saturating_add(1); }
        }
        // Advance time by base quantum
        now_ns = now_ns.saturating_add(5_000);
    }

    metric_kv("det_deadline_miss_count", miss_count as usize);
}

/// CBS (Constant Bandwidth Server) for deterministic graph isolation
#[derive(Clone)]
pub struct CbsServer {
    pub server_id: u32,
    pub graph_id: u32,
    pub budget_ns: u64,        // Allocated execution budget
    pub period_ns: u64,        // Server period
    pub remaining_budget_ns: u64, // Current remaining budget
    pub next_replenish_ns: u64,   // When budget gets replenished
    pub deadline_ns: u64,      // Current server deadline
    pub active: bool,          // Server is currently active
}

impl CbsServer {
    pub fn new(server_id: u32, graph_id: u32, wcet_ns: u64, period_ns: u64) -> Self {
        Self {
            server_id,
            graph_id,
            budget_ns: wcet_ns,
            period_ns,
            remaining_budget_ns: wcet_ns,
            next_replenish_ns: period_ns,
            deadline_ns: period_ns,
            active: false,
        }
    }
    
    /// Replenish server budget at period boundary
    pub fn replenish(&mut self, now_ns: u64) {
        if now_ns >= self.next_replenish_ns {
            self.remaining_budget_ns = self.budget_ns;
            self.next_replenish_ns = now_ns + self.period_ns;
            self.deadline_ns = self.next_replenish_ns;
        }
    }
    
    /// Consume budget for operator execution
    pub fn consume_budget(&mut self, consumed_ns: u64) -> bool {
        if consumed_ns <= self.remaining_budget_ns {
            self.remaining_budget_ns -= consumed_ns;
            true
        } else {
            // Budget exhausted - server becomes inactive until replenishment
            self.active = false;
            false
        }
    }
    
    pub fn has_budget(&self) -> bool {
        self.remaining_budget_ns > 0
    }
}

/// Deterministic Graph Scheduler with CBS+EDF
pub struct DeterministicScheduler<const MAX_SERVERS: usize> {
    servers: [Option<CbsServer>; MAX_SERVERS],
    server_count: usize,
    edf_queue: EdfQueue<MAX_SERVERS>,
    admission_controller: AdmissionController,
    jitter_samples_ns: [u64; 64], // Jitter tracking for Phase 2 metrics
    jitter_count: usize,
    deadline_misses: u32,
}

impl<const MAX_SERVERS: usize> DeterministicScheduler<MAX_SERVERS> {
    pub const fn new(admission_bound_ppm: u32) -> Self {
        Self {
            servers: [const { None }; MAX_SERVERS],
            server_count: 0,
            edf_queue: EdfQueue::new(),
            admission_controller: AdmissionController::new(admission_bound_ppm),
            jitter_samples_ns: [0; 64],
            jitter_count: 0,
            deadline_misses: 0,
        }
    }
    
    /// Admit a new deterministic graph with CBS server
    pub fn admit_graph(&mut self, graph_id: u32, spec: TaskSpec) -> Result<u32, ()> {
        // Check admission control
        if !self.admission_controller.try_admit(&spec) {
            return Err(());
        }
        
        // Create CBS server
        if self.server_count >= MAX_SERVERS {
            return Err(());
        }
        
        let server_id = self.server_count as u32;
        let server = CbsServer::new(server_id, graph_id, spec.wcet_ns, spec.period_ns);
        self.servers[self.server_count] = Some(server);
        self.server_count += 1;
        
        Ok(server_id)
    }
    
    /// Schedule next graph for execution using CBS+EDF
    pub fn schedule_next(&mut self, now_ns: u64) -> Option<u32> {
        // Replenish budgets for all servers
        for i in 0..self.server_count {
            if let Some(ref mut server) = self.servers[i] {
                server.replenish(now_ns);
                
                // Add active servers with budget to EDF queue
                if server.has_budget() && !server.active {
                    server.active = true;
                    let _ = self.edf_queue.push(EdfNode {
                        id: server.graph_id,
                        abs_deadline_ns: server.deadline_ns,
                    });
                }
            }
        }
        
        // Select earliest-deadline graph
        if let Some(node) = self.edf_queue.pop() {
            // Check for deadline miss
            if now_ns > node.abs_deadline_ns {
                self.deadline_misses += 1;
                metric_kv("deterministic_deadline_miss_count", self.deadline_misses as usize);
            }
            
            Some(node.id)
        } else {
            None
        }
    }
    
    /// Record execution completion and consume server budget
    pub fn complete_execution(&mut self, graph_id: u32, actual_runtime_ns: u64, expected_ns: u64) {
        // Find the server for this graph
        for i in 0..self.server_count {
            if let Some(ref mut server) = self.servers[i] {
                if server.graph_id == graph_id {
                    // Consume budget
                    let _ = server.consume_budget(actual_runtime_ns);
                    
                    // Track jitter for Phase 2 metrics
                    if self.jitter_count < 64 {
                        let jitter = if actual_runtime_ns > expected_ns {
                            actual_runtime_ns - expected_ns
                        } else {
                            expected_ns - actual_runtime_ns
                        };
                        self.jitter_samples_ns[self.jitter_count] = jitter;
                        self.jitter_count += 1;
                    }
                    break;
                }
            }
        }
    }
    
    /// Emit Phase 2 deterministic metrics
    pub fn emit_metrics(&self) {
        let (used_ppm, accepted, rejected) = self.admission_controller.stats();
        metric_kv("det_admission_used_ppm", used_ppm as usize);
        metric_kv("det_admission_accepted", accepted as usize);
        metric_kv("det_admission_rejected", rejected as usize);
        metric_kv("deterministic_deadline_miss_count", self.deadline_misses as usize);
        
        // Emit jitter statistics
        if self.jitter_count > 0 {
            let mut sorted_jitter = [0u64; 64];
            sorted_jitter[..self.jitter_count].copy_from_slice(&self.jitter_samples_ns[..self.jitter_count]);
            sorted_jitter[..self.jitter_count].sort_unstable();
            
            let p99_idx = ((self.jitter_count - 1) as f32 * 0.99) as usize;
            metric_kv("deterministic_jitter_p99_ns", sorted_jitter[p99_idx] as usize);
        }
    }
}

/// Deterministic operation constraints enforcement
pub struct ConstraintEnforcer {
    /// Track allocations to prevent dynamic allocation in deterministic ops
    allocation_count: u32,
    /// Track loop iterations to detect unbounded loops
    max_loop_iterations: u32,
    /// Track blocking calls to prevent indefinite blocking
    blocking_call_count: u32,
}

impl ConstraintEnforcer {
    pub const fn new(max_loops: u32) -> Self {
        Self {
            allocation_count: 0,
            max_loop_iterations: max_loops,
            blocking_call_count: 0,
        }
    }
    
    /// Check if dynamic allocation is allowed (should be NO for deterministic ops)
    pub fn check_allocation(&mut self) -> bool {
        self.allocation_count += 1;
        // In deterministic mode, no dynamic allocations allowed
        false
    }
    
    /// Check loop iteration count to prevent unbounded loops
    pub fn check_loop_iteration(&self, current_iteration: u32) -> bool {
        current_iteration < self.max_loop_iterations
    }
    
    /// Check if blocking call is allowed (should be NO for deterministic ops)
    pub fn check_blocking_call(&mut self) -> bool {
        self.blocking_call_count += 1;
        // In deterministic mode, no indefinite blocking allowed
        false
    }
    
    /// Reset constraints for new execution cycle
    pub fn reset(&mut self) {
        self.allocation_count = 0;
        self.blocking_call_count = 0;
    }
    
    /// Get constraint violation stats
    pub fn stats(&self) -> (u32, u32) {
        (self.allocation_count, self.blocking_call_count)
    }
}

/// Deterministic operation verification
pub fn verify_deterministic_constraints(op_id: u32, enforcer: &mut ConstraintEnforcer) -> bool {
    // In a real implementation, this would:
    // 1. Check that the operator doesn't call malloc/free
    // 2. Verify all loops have compile-time bounds
    // 3. Ensure no indefinite blocking operations (mutex_lock, etc.)
    // 4. Validate that all memory accesses are within predetermined bounds
    
    // For Phase 2 demo, perform basic checks
    let (allocs, blocks) = enforcer.stats();
    
    if allocs > 0 {
        metric_kv("det_constraint_violation_alloc", allocs as usize);
        return false;
    }
    
    if blocks > 0 {
        metric_kv("det_constraint_violation_block", blocks as usize);
        return false;
    }
    
    // Log successful constraint verification
    metric_kv("det_constraint_verified", op_id as usize);
    true
}
