//! Minimal graph and operator scaffolding for Phase 1.
//! Includes a simple two-operator demo wiring an SPSC channel.

use crate::channel::spsc::Spsc;
use crate::tensor::{BumpArena, TensorHandle};
use crate::trace::metric_kv;
#[cfg(feature = "perf-verbose")]
use crate::pmu::aarch64 as pmu;

#[derive(Copy, Clone)]
pub struct OperatorId(pub u32);

pub struct Operator<'a> {
    pub id: OperatorId,
    pub run: fn(&mut OperatorCtx<'a>),
}

pub struct OperatorCtx<'a> {
    pub produced: &'a Spsc<TensorHandle, 64>,
    pub consumed: &'a Spsc<TensorHandle, 64>,
}

pub struct GraphDemo {
    pub n_items: usize,
    arena: BumpArena<8192>,
    graph: GraphApi,
    #[allow(dead_code)]
    op_a_idx: usize,
    #[allow(dead_code)]
    op_b_idx: usize,
    ch_ab_idx: usize,
    ch_bc_idx: usize,
}

impl GraphDemo {
    pub fn new(n_items: usize) -> Self {
        let mut graph = GraphApi::create();
        let ch_ab_idx = graph.add_channel(ChannelSpec { capacity: 64 });
        let ch_bc_idx = graph.add_channel(ChannelSpec { capacity: 64 });
        let op_a_idx = graph.add_operator(OperatorSpec { id: 1, func: op_a_run, in_ch: None, out_ch: Some(ch_ab_idx), priority: 10, stage: None });
        let op_b_idx = graph.add_operator(OperatorSpec { id: 2, func: op_b_run, in_ch: Some(ch_ab_idx), out_ch: Some(ch_bc_idx), priority: 5, stage: None });
        Self { n_items, arena: BumpArena::new(), graph, op_a_idx, op_b_idx, ch_ab_idx, ch_bc_idx }
    }

    /// Run a trivial A->B pipeline to demonstrate scheduling and metrics.
    pub fn run(&mut self) {
        // Operators: A produces 0..n, B consumes and forwards (or accumulates).
        #[cfg(feature = "perf-verbose")]
        let _op_a_id = 1u32;
        #[cfg(feature = "perf-verbose")]
        let _op_b_id = 2u32;

        let mut _produced = 0usize;
        let mut _consumed = 0usize;
        let mut zero_copy_count = 0usize;
        let mut zero_copy_handle_count = 0usize;
        let mut ch_ab_depth_max = 0usize;
        let mut ch_ab_stalls = 0usize;
        let mut ch_ab_drops = 0usize;
        let mut op_a_runs = 0usize;
        let mut op_b_runs = 0usize;
        let mut op_a_cycles: u64 = 0;
        let mut op_b_cycles: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_a_inst: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_b_inst: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_a_l1d: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_b_l1d: u64 = 0;
        // PMU attribution is intentionally disabled in the demo to avoid
        // QEMU variability; keep perf-verbose for other parts of the boot.

        // Collect per-operator latency samples (ns) for percentiles (window 128)
        let mut lat_a: [u64; 128] = [0; 128];
        let mut lat_b: [u64; 128] = [0; 128];
        let mut lat_a_n: usize = 0;
        let mut lat_b_n: usize = 0;

        let t0 = now_cycles();
        for i in 0..self.n_items {
            // Producer work (no channel dependency)
            let ta0 = now_cycles();
            #[cfg(feature = "perf-verbose")]
            let s0a = unsafe { pmu::read_snapshot() };
            (op_a_run)(&mut OperatorCtx {
                produced: self.graph.channel(self.ch_ab_idx),
                consumed: self.graph.channel(self.ch_ab_idx),
            });
            if let Some(h) = self.arena.alloc(64, 64) {
                if !h.is_null() { zero_copy_handle_count += 1; }
            }
            _produced += 1;
            let ta1 = now_cycles();
            let cyc_a = ta1.saturating_sub(ta0);
            op_a_cycles = op_a_cycles.saturating_add(cyc_a);
            if lat_a_n < lat_a.len() { lat_a[lat_a_n] = cycles_to_ns(cyc_a); lat_a_n += 1; }
            #[cfg(feature = "perf-verbose")]
            {
                let s1a = unsafe { pmu::read_snapshot() };
                op_a_inst = op_a_inst.saturating_add(s1a.inst.saturating_sub(s0a.inst));
                op_a_l1d = op_a_l1d.saturating_add(s1a.l1d_refill.saturating_sub(s0a.l1d_refill));
            }
            op_a_runs += 1;

            // Consumer work (no channel dependency)
            // Track channel AB depth for backpressure visibility
            let d = self.graph.channel(self.ch_ab_idx).depth();
            if d > ch_ab_depth_max { ch_ab_depth_max = d; }
            // Track stalls: if channel is full, count as stall
            if self.graph.channel(self.ch_ab_idx).is_full() {
                ch_ab_stalls += 1;
            }
            // Track potential drops: if depth is at capacity-1 and still producing
            if d >= 63 { // Channel capacity is 64, so 63+ indicates near-full
                ch_ab_drops += 1;
            }
            let tb0 = now_cycles();
            #[cfg(feature = "perf-verbose")]
            let s0b = unsafe { pmu::read_snapshot() };
            (op_b_run)(&mut OperatorCtx {
                produced: self.graph.channel(self.ch_bc_idx),
                consumed: self.graph.channel(self.ch_ab_idx),
            });
            _consumed += 1;
            zero_copy_count += 1;
            let tb1 = now_cycles();
            let cyc_b = tb1.saturating_sub(tb0);
            op_b_cycles = op_b_cycles.saturating_add(cyc_b);
            if lat_b_n < lat_b.len() { lat_b[lat_b_n] = cycles_to_ns(cyc_b); lat_b_n += 1; }
            #[cfg(feature = "perf-verbose")]
            {
                let s1b = unsafe { pmu::read_snapshot() };
                op_b_inst = op_b_inst.saturating_add(s1b.inst.saturating_sub(s0b.inst));
                op_b_l1d = op_b_l1d.saturating_add(s1b.l1d_refill.saturating_sub(s0b.l1d_refill));
            }
            op_b_runs += 1;
            if (i & 7) == 7 {
                crate::trace::trace("GRAPH DEMO: progressed 8 items");
            }
        }
        let t1 = now_cycles();
        let ns = cycles_to_ns(t1.saturating_sub(t0));
        metric_kv("graph_demo_total_ns", ns as usize);
        metric_kv("graph_demo_items", self.n_items);
        if self.n_items > 0 { metric_kv("graph_demo_avg_ns_per_item", (ns / (self.n_items as u64)) as usize); }
        // Scheduler batch timing (us)
        metric_kv("scheduler_run_us", (ns / 1000) as usize);
        metric_kv("channel_ab_depth_max", ch_ab_depth_max);
        metric_kv("channel_ab_stalls", ch_ab_stalls);
        metric_kv("channel_ab_drops", ch_ab_drops);
        metric_kv("zero_copy_count", zero_copy_count);
        metric_kv("zero_copy_handle_count", zero_copy_handle_count);
        // Operator summaries
        metric_kv("op_a_runs", op_a_runs);
        metric_kv("op_b_runs", op_b_runs);
        metric_kv("op_a_total_ns", cycles_to_ns(op_a_cycles) as usize);
        metric_kv("op_b_total_ns", cycles_to_ns(op_b_cycles) as usize);
        // Percentiles for operator latencies
        if lat_a_n > 0 {
            let p50 = percentile_ns(&mut lat_a, lat_a_n, 0.50);
            let p95 = percentile_ns(&mut lat_a, lat_a_n, 0.95);
            let p99 = percentile_ns(&mut lat_a, lat_a_n, 0.99);
            metric_kv("op_a_p50_ns", p50 as usize);
            metric_kv("op_a_p95_ns", p95 as usize);
            metric_kv("op_a_p99_ns", p99 as usize);
        }
        if lat_b_n > 0 {
            let p50 = percentile_ns(&mut lat_b, lat_b_n, 0.50);
            let p95 = percentile_ns(&mut lat_b, lat_b_n, 0.95);
            let p99 = percentile_ns(&mut lat_b, lat_b_n, 0.99);
            metric_kv("op_b_p50_ns", p50 as usize);
            metric_kv("op_b_p95_ns", p95 as usize);
            metric_kv("op_b_p99_ns", p99 as usize);
        }
        #[cfg(feature = "perf-verbose")]
        {
            metric_kv("op_a_pmu_inst", op_a_inst as usize);
            metric_kv("op_b_pmu_inst", op_b_inst as usize);
            metric_kv("op_a_pmu_l1d_refill", op_a_l1d as usize);
            metric_kv("op_b_pmu_l1d_refill", op_b_l1d as usize);
        }
        // Arena remaining bytes (sanity check for bump behavior)
        metric_kv("arena_remaining_bytes", self.arena.remaining());
    }
}

#[inline(always)]
fn percentile_ns(buf: &mut [u64; 128], n: usize, p: f32) -> u64 {
    if n == 0 { return 0; }
    // simple in-place sort of the used prefix
    let slice = &mut buf[..n];
    slice.sort_unstable();
    let idx = ((n - 1) as f32 * p) as usize;
    slice[idx]
}

pub fn op_a_run(_ctx: &mut OperatorCtx) {
    // Placeholder for producer work (could fill a tensor)
}

fn op_b_run(_ctx: &mut OperatorCtx) {
    // Placeholder for consumer work (could transform a tensor)
}

#[inline(always)]
fn now_cycles() -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut v: u64; core::arch::asm!("isb; mrs {x}, cntvct_el0", x = out(reg) v, options(nomem, nostack, preserves_flags)); v
    }
    #[cfg(not(target_arch = "aarch64"))]
    { 0 }
}

#[inline(always)]
fn cntfrq_hz() -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut v: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) v, options(nomem, nostack, preserves_flags)); v
    }
    #[cfg(not(target_arch = "aarch64"))]
    { 1 }
}

#[inline(always)]
fn cycles_to_ns(cycles: u64) -> u64 {
    let f = cntfrq_hz();
    if f == 0 { return 0; }
    (cycles.saturating_mul(1_000_000_000u64)) / f
}

// Minimal Graph API surface (Phase 1 scaffolding)
#[allow(dead_code)]
pub enum PortDir { In, Out }

#[allow(dead_code)]
pub struct ChannelSpec { pub capacity: usize }

#[allow(dead_code)]
pub struct OperatorSpec {
    pub id: u32,
    pub func: fn(&mut OperatorCtx),
    pub in_ch: Option<usize>,
    pub out_ch: Option<usize>,
    pub priority: u8,
    #[allow(dead_code)]
    pub stage: Option<Stage>,
}

pub struct GraphApi {
    channels: alloc::vec::Vec<alloc::boxed::Box<Spsc<TensorHandle, 64>>>,
    ops: alloc::vec::Vec<OpNode>,
    #[cfg(feature = "deterministic")]
    det_ac: crate::deterministic::AdmissionController,
}

struct OpNode {
    #[allow(dead_code)]
    id: u32,
    in_ch: Option<usize>,
    out_ch: Option<usize>,
    #[allow(dead_code)]
    priority: u8,
    func: fn(&mut OperatorCtx),
    #[allow(dead_code)]
    stage: Option<Stage>,
}

#[allow(dead_code)]
impl GraphApi {
    pub fn create() -> Self {
        Self {
            channels: alloc::vec::Vec::new(),
            ops: alloc::vec::Vec::new(),
            #[cfg(feature = "deterministic")]
            det_ac: crate::deterministic::AdmissionController::new(850_000),
        }
    }
    pub fn add_channel(&mut self, _spec: ChannelSpec) -> usize {
        let idx = self.channels.len();
        self.channels.push(alloc::boxed::Box::new(Spsc::new()));
        idx
    }
    pub fn add_operator(&mut self, spec: OperatorSpec) -> usize {
        let idx = self.ops.len();
        self.ops.push(OpNode { id: spec.id, in_ch: spec.in_ch, out_ch: spec.out_ch, priority: spec.priority, func: spec.func, stage: spec.stage });
        idx
    }
    pub fn is_runnable(&self, op_idx: usize) -> bool {
        if let Some(op) = self.ops.get(op_idx) {
            let in_ready = match op.in_ch { Some(i) => !self.channels[i].is_empty(), None => true };
            let out_ready = match op.out_ch { Some(i) => !self.channels[i].is_full(), None => true };
            in_ready && out_ready
        } else { false }
    }
    pub fn channel(&self, idx: usize) -> &Spsc<TensorHandle, 64> { &self.channels[idx] }

    /// Execute up to `steps` runnable operators in static-priority order (highest first).
    pub fn run_steps(&mut self, steps: usize) {
        if steps == 0 { return; }
        // Simple O(n^2) selection for now (tiny n)
        for _ in 0..steps {
            let mut ran = false;
            let mut best_idx: Option<usize> = None;
            let mut best_pri: u8 = 0;
            for (i, op) in self.ops.iter().enumerate() {
                if self.is_runnable(i) && op.priority >= best_pri { best_pri = op.priority; best_idx = Some(i); }
            }
            if let Some(i) = best_idx {
                let op = &self.ops[i];
                let out = op.out_ch.map(|k| &*self.channels[k]).unwrap_or_else(|| &*self.channels[0]);
                let inp = op.in_ch.map(|k| &*self.channels[k]).unwrap_or_else(|| out);
                let mut ctx = OperatorCtx { produced: out, consumed: inp };
                (op.func)(&mut ctx);
                ran = true;
            }
            if !ran { break; }
        }
    }

    /// Return simple counts for ops and channels (for diagnostics).
    pub fn counts(&self) -> (usize, usize) {
        (self.ops.len(), self.channels.len())
    }

    #[cfg(feature = "deterministic")]
    pub fn admit_deterministic(&mut self, wcet_ns: u64, period_ns: u64, deadline_ns: u64) -> bool {
        let spec = crate::deterministic::TaskSpec { id: 0, wcet_ns, period_ns, deadline_ns };
        let ok = self.det_ac.try_admit(&spec);
        let (used_ppm, acc, rej) = self.det_ac.stats();
        crate::trace::metric_kv("det_admission_used_ppm", used_ppm as usize);
        crate::trace::metric_kv("det_admission_accepted", acc as usize);
        crate::trace::metric_kv("det_admission_rejected", rej as usize);
        if ok { crate::trace::metric_kv("det_admit_ok", 1) } else { crate::trace::metric_kv("det_admit_reject", 1) }
        ok
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Stage { AcquireData=0, CleanData=1, ExploreData=2, ModelData=3, ExplainResults=4 }

// Control-plane can call op_a_run directly (pub)

/// Debug SPSC ring behavior with progress prints (u32 payloads). Feature-gated.
#[cfg(feature = "graph-spsc-debug")]
pub fn run_spsc_debug(n: usize) {
    use crate::trace::trace;
    let q: Spsc<u32, 64> = Spsc::new();
    let mut produced = 0usize;
    let mut consumed = 0usize;
    trace("SPSC DEBUG: start");
    while consumed < n {
        if produced < n {
            let v = produced as u32;
            if q.try_enqueue(v).is_ok() {
                produced += 1;
                if produced % 8 == 0 { trace("SPSC DEBUG: produced 8"); }
            }
        }
        if let Some(_v) = q.try_dequeue() {
            consumed += 1;
            if consumed % 8 == 0 { trace("SPSC DEBUG: consumed 8"); }
        }
    }
    crate::trace::metric_kv("spsc_debug_done", 1);
}
