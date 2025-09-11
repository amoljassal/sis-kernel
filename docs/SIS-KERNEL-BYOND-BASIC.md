# SIS Kernel — Beyond Basic

An end‑to‑end blueprint to evolve SIS from a metrics‑capable bring‑up into a research‑grade, AI‑native operating system. This document serves developers and AI agents as a stable reference for architecture, roadmap, validation, and collaboration prompts.

## Overview & Vision

SIS treats AI workloads as first‑class: the kernel schedules dataflow graphs, manages tensors, enforces model‑centric security, offers deterministic execution modes, and scales to multi‑node topologies. Success means reproducible performance, analyzable behavior, strong isolation, and publishable results under CI.

## Executive Blueprint

- Dataflow‑First: Operators inside a Graph communicate via bounded Channels carrying Tensor handles. Threads/processes are not the primary abstraction.
- Capabilities‑Only Security: Unforgeable capability tokens gate rights on all kernel objects; models are principals.
- Determinism by Design: Deterministic mode with admission control and deadline enforcement for AI control loops.
- Built‑In Observability: Low‑overhead tracepoints and PMU attribution tied to operators and graphs; stable JSON exports.
- Minimal SSI: A consensus‑free data plane with a narrow control plane to place and supervise graphs across nodes.

### 16–20 Week Roadmap (Phases)

| Weeks | Phase | Milestones |
| --- | --- | --- |
| 1–6 | Phase 1: Dataflow + Observability | Kernel objects (Graph/Operator/Channel/Tensor/Capability), SPSC channels, static graph scheduler, tracepoints + PMU attribution, “Graph OS” demo with clean METRICs/JSON. |
| 7–12 | Phase 2: Deterministic + Model Capabilities | CBS+EDF deterministic scheduler, admission control, banned nondeterministic ops, signed model packages, capability checks and measurement log; deterministic demo with jitter bounds. |
| 13–16 | Phase 3: Multi‑Node SSI Prototype | RemoteChannel, minimal transport, manual placement, node discovery/heartbeats, backpressure across nodes; 2‑node throughput scaling and graceful degradation demo. |
| 17–20 | Validation & Hardening | Benchmarks, schema stability, CI thresholds, perf polishing; paper‑ready results. |

## Architecture: Dataflow‑First Kernel

### Kernel Object Model (Minimal, Capability‑Gated)

- Graph: Unit of isolation and scheduling; owns operators, channels, and a memory arena.
- Operator: Non‑preemptible compute function with typed input/output ports; no hidden state.
- Port: Directional binding between an operator and a channel.
- Channel: Bounded SPSC queue of Tensor handles with backpressure.
- Tensor: Page‑backed, cache‑aligned buffer + metadata (shape, dtype, version). Zero‑copy sharing across operators.
- Model: Immutable, signed package (graph + weights) with measured load and execution constraints.
- Capability: Unforgeable handle granting rights (e.g., WRITE on Channel, RUN on Graph, EXECUTE on Model).

Lifecycle (typical): GraphCreate → OperatorAdd/ChannelCreate → GraphStart → GraphStop/Destroy. All APIs return capabilities; every use path checks rights.

### Zero‑Copy Tensors

- Memory: Per‑graph slab/arena; 64‑byte alignment; avoid fragmentation; bump alloc for deterministic graphs.
- Sharing: Read‑sharing via refcounts; mutation bumps version (copy‑on‑write at explicit mutation points).
- Handles: Small, fixed‑size descriptors enqueued on channels; bulk data never copied on local paths.

### Channels & Backpressure

- Phase 1: SPSC lock‑free ring buffers only (fast, analyzable). Watermark backpressure with metrics on depth, stalls, drops.
- Future: MPSC/MPMC via credits if needed; keep deterministic graphs on SPSC.

## Scheduling

### Phase 1: Static Priority Scheduler

- Topologically sorted operators; run‑to‑completion; pick highest static priority among runnable ops (all inputs ready, output space available).
- Yield only at op completion; blocked producers become runnable when space exists.

### Phase 2: Deterministic (CBS + EDF Hybrid)

- Admission Control: For each deterministic graph submit (WCET, period, deadline); accept only if utilization bound holds.
- CBS Server per deterministic graph isolates runtime; EDF orders operator activations inside the server.
- Timer Discipline: Program ARM architected timer per deadline (event‑driven), not heavy periodic ticks.
- Invariants: No dynamic allocation, no unbounded loops, no indefinite blocking in deterministic operators; enforced by kernel checks and audits.

## Security & Model Capabilities

- Capability Core: 64‑bit id + sealed tag in a kernel table; O(1) rights checks; no crypto on the fast path.
- Signed Model Packages: SHA‑256 + Ed25519 recommended; verify on load; store measurement (hash, time, id) in a log.
- Model Permissions: LOAD/EXECUTE/INSPECT/EXPORT/ATTEST with constraints (memory cap, allowed ops, compute budget).
- Secure Model Store (interface): Encrypted, versioned, rollback‑aware storage is orchestrated by user space; kernel enforces measurement and capability policy at load/run.

## Observability & Metrics

### Tracepoint Taxonomy (Low Overhead)

- Graph: loaded/started/completed.
- Operator: queued/started/completed (cycles), queue depth changes.
- Tensor: allocated/shared/freed; backpressure events (channel full, action taken).
- Deterministic: deadline programmed/met/missed; preempt reasons (if any).

### PMU Attribution

- Per‑operator: fixed set of counters (cycles, instructions, L1D miss, branch miss). Configure once per run or per activation with care to avoid reprogramming overhead.

### Metrics & JSON (Schema v1)

- Metrics to emit (phase‑gated):
  - P1: op_latency_p50/p95/p99_ns, channel_depth_max, scheduler_run_us, zero_copy_count.
  - P2: deterministic_deadline_miss_count, deterministic_jitter_ns, model_load_success/fail.
  - P3: remote_channel_latency_p95_us, retry_rate, placement_time_ms, node_failures.
- JSON export: stable keys with a top‑level schema version; dedicated sections for graphs, operators, channels, and percentiles.

## Multi‑Node SSI Prototype

### RemoteChannel & Transport

- RemoteChannel: Same SPSC API; internally frames tensor handles + metadata and sends over transport; propagates backpressure via credits.
- Transport (QEMU virtio‑net): Framed messages with seq, len, crc, flags; credit‑based flow control; retries with exponential backoff.

### Placement & Control Plane

- Phase 3a: Manual placement map; kernel inserts RemoteChannels across node boundaries.
- Heartbeats & Epochs: Failure detection; on node failure, bump epoch and tear down affected graphs; user‑space restarts.
- Phase 3b (optional): Minimal leader + replicated placement log; never in the data plane.

## Validation & CI

- Reproducibility: Fixed seeds, config snapshots, 100‑run distributions; QEMU and hardware thresholds maintained separately.
- Phase Acceptance Gates:
  - P1: Zero‑copy speedup vs copy (>2× for ≥64 KiB tensors); operator p95 tracked; clean METRIC/JSON.
  - P2: Deterministic admission rejects overload; jitter p99 within target; unsigned model load rejected with audit.
  - P3: Remote p95 latency target (<1 ms in QEMU), retry rate <1%, throughput scaling >1.5× for pipelined graphs.

## Decision Pack (Key Choices)

1) Graph as the Scheduling Unit (vs. threads)
- Pros: Natural backpressure, analyzable execution, fits AI workloads. Cons: Less general‑purpose. Recommendation: Adopt.

2) Capabilities‑Only Security (no ACLs/users)
- Pros: Minimal, strong isolation; aligns with seL4 practices. Cons: Capability lifecycle complexity. Recommendation: Adopt.

3) Static Schedules for Deterministic Graphs (dynamic for best‑effort)
- Pros: Predictable latency; easy validation. Cons: Less flexibility. Recommendation: Adopt (hybrid model overall).

4) Consensus‑Free Data Plane
- Pros: Lowest latency; simpler correctness; scalable. Cons: Control‑plane single‑point risks (mitigated by replication). Recommendation: Adopt.

## Risk Register (Top)

- P1: Scheduler overhead hurts gains → Keep static/simple; trace hotspots; iterate after correctness.
- P2: Tight jitter goals are hard on QEMU → Separate QEMU vs hardware thresholds; focus on reproducible bounds first.
- P3: Network overhead erodes SSI win → Batch frames; propagate backpressure; measure and cut copies.

## Multi‑AI Consultation Protocol (Adapted)

Use these prompts to obtain focused, complementary guidance from AI advisors. Paste the respective blocks into each system.

### Claude (Research Synthesis & Blueprint)

CONSULTATION REQUEST: Claude
DOMAIN: Architecture, Implementation, Optimization, Research Synthesis
CONTEXT: SIS AI‑Native Kernel (Rust no_std, AArch64, QEMU UEFI). Dataflow‑first direction, determinism goals, capability security, built‑in metrics/JSON, and a strict CI lint gate.
OBJECTIVE: Provide best practices and a staged plan for Phases 1–3, with references and actionable takeaways.
DELIVERABLES: Executive blueprint; object model and APIs; scheduling (static → CBS+EDF); model security (signed packages, capability policy, measurement log); observability (trace/PMU/JSON); SSI transport/placement; validation gates; risk mitigations. Include annotated, recent references and a “Decision Pack”.
INTEGRATION: Minimal kernel APIs, tensor lifetimes, METRIC keys, JSON schema, QEMU‑viable validation, optional hardware notes.

### Gemini 2.5 Pro (Architecture & Validation Plan)

Role: Principal Architect for dataflow kernel and SSI.
Focus: Interfaces/diagrams, object relationships, backpressure semantics, deterministic invariants, CI validation.
Ask: Produce diagrams (component/state/sequence), kernel API surfaces, scheduling policy evolution, capability model for Models, observability taxonomy, SSI transport and placement plan, and a week‑by‑week roadmap. Provide references with 1–2 sentence annotations and what to borrow.

### Super Grok 4 (Performance & Implementation)

Role: Low‑level systems advisor.
Focus: Rust no_std data structures (SPSC rings, tensor pools), AArch64 atomics/barriers, EDF heap, admission control logic, PMU setup, minimal RPC with credits.
Ask: Provide pseudocode and select Rust snippets for channels, tensor pool APIs, EDF/CBS core, admission checks, PMU attribution hooks, RPC send/recv with backpressure, plus pitfalls (AArch64 memory model, cache/TLB, QEMU artifacts) and actionable references.

## Immediate Next Actions (Week 1)

- Create kernel modules: `graph/`, `channel/spsc/`, `tensor/`, `cap/` with stubs and compile‑time traits.
- Define METRIC schema v1 and update test runner to validate it.
- Implement SPSC ring and per‑graph arenas; add tracepoints for op start/end and channel depth.
- Build “Sobel chain” demo graph; measure end‑to‑end latency and zero‑copy gains.

## Appendix A: Representative Snippets (Illustrative)

Operator Scheduling (Phase 1, static):

```rust
fn run_graph(g: &Graph) {
    loop {
        if let Some(op) = g.next_runnable_op() {
            trace_op_start(op.id);
            op.execute();
            trace_op_end(op.id);
        } else { break; }
    }
}
```

SPSC Ring (lock‑free outline):

```rust
struct Spsc<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}
```

Admission Control (EDF bound):

```rust
fn admit(wcet: u64, period: u64, current_util: f64) -> bool {
    current_util + (wcet as f64 / period as f64) <= 1.0
}
```

## References (Selected, With Takeaways)

- seL4 (SOSP 2009): Minimal capability kernel; keep TCB tiny; capability discipline throughout.
- Linux SCHED_DEADLINE (Lelli et al.): EDF + admission control; utilization bounds and timer discipline.
- Abeni & Buttazzo CBS (RTSS 1998): Bandwidth servers to isolate mixed workloads.
- TensorFlow TFRT: Static graph runtime; separate compile vs run; small, efficient ops.
- Barrelfish: Multikernel message‑passing; separate mechanism from policy; per‑core independence.
- Google Dapper: Tracing principles; low overhead, context propagation.
- ARM ARM + PMU Docs: Barriers, memory model, counter programming; avoid over‑fencing; attribute wisely.

