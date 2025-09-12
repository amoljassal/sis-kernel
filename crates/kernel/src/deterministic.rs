//! Deterministic scheduler scaffolding (Phase 2):
//! - Admission control using utilization bounds
//! - Minimal EDF queue
//! - (Optional) CBS server structs for future enforcement
//! Emits simple METRICs in demo mode.

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
