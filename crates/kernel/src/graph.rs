//! Minimal graph and operator scaffolding for Phase 1.
//! Includes a simple two-operator demo wiring an SPSC channel.

use crate::channel::spsc::Spsc;
use crate::trace::{metric_kv};

#[derive(Copy, Clone)]
pub struct OperatorId(pub u32);

pub struct Operator<'a> {
    pub id: OperatorId,
    pub run: fn(&mut OperatorCtx<'a>),
}

pub struct OperatorCtx<'a> {
    pub produced: &'a Spsc<u32, 64>,
    pub consumed: &'a Spsc<u32, 64>,
}

pub struct GraphDemo {
    ch_ab: Spsc<u32, 64>,
    ch_bc: Spsc<u32, 64>,
    pub n_items: usize,
}

impl GraphDemo {
    pub const fn new(n_items: usize) -> Self {
        Self { ch_ab: Spsc::new(), ch_bc: Spsc::new(), n_items }
    }

    /// Run a trivial A->B pipeline to demonstrate scheduling and metrics.
    pub fn run(&self) {
        // Operators: A produces 0..n, B consumes and forwards (or accumulates).
        let op_a = Operator { id: OperatorId(1), run: op_a_run };
        let op_b = Operator { id: OperatorId(2), run: op_b_run };

        let mut produced = 0usize;
        let mut consumed = 0usize;

        let t0 = now_cycles();
        while consumed < self.n_items {
            // Producer: try enqueue next item
            if produced < self.n_items {
                let mut ctx_a = OperatorCtx { produced: &self.ch_ab, consumed: &self.ch_bc };
                (op_a.run)(&mut ctx_a);
                // op_a_run enqueues when space; it uses an internal counter via static mut.
                // We approximate by attempting to enqueue "produced" value here for clarity.
                if self.ch_ab.try_enqueue(produced as u32).is_ok() { produced += 1; }
            }

            // Consumer: try dequeue from A and enqueue to B (or count)
            if let Some(_v) = self.ch_ab.try_dequeue() {
                let mut ctx_b = OperatorCtx { produced: &self.ch_bc, consumed: &self.ch_ab };
                (op_b.run)(&mut ctx_b);
                // For demo, just count as consumed
                consumed += 1;
            }
        }
        let t1 = now_cycles();
        let ns = cycles_to_ns(t1.saturating_sub(t0));
        metric_kv("graph_demo_total_ns", ns as usize);
        metric_kv("graph_demo_items", self.n_items);
        if self.n_items > 0 { metric_kv("graph_demo_avg_ns_per_item", (ns / (self.n_items as u64)) as usize); }
    }
}

fn op_a_run(_ctx: &mut OperatorCtx) {
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
