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

Implementation status (current repo)
- Phase 1 foundations are in: Graph/Operator/Channel/Tensor scaffolding, SPSC ring, zero‑copy tensor handles, typed DataTensor header (schema_id/records/quality/lineage), graph demo (A→B), basic per‑operator time attribution and METRICs.
- Control plane V0 is implemented in‑kernel with a minimal binary frame format; for bring‑up it is exercised via shell commands (`graphctl`, `ctlhex`). A VirtIO control path exists behind a feature flag and remains opt‑in; it emits control‑plane metrics (`ctl_frames_rx/tx/errors/backpressure_drops`, `ctl_roundtrip_us`, `ctl_selected_port/ctl_port_bound`) and sends `OK\n`/`ERR\n` ACKs.
- PMU helpers exist with a QEMU caveat (cycles reliable; other events may read as 0).
- Strict lint gate is available (`strict` feature) and CI/test runner prefer real context‑switch metrics with a minimum non‑zero sample requirement. Operator/channel traces are emitted (`[TRACE] op_queued/start/end`, `[TRACE] ch_depth`).

### Implementation Status Matrix (Current Repo)

| Area | Item | Status | Notes / Code References |
|---|---|---|---|
| Executive Blueprint | Dataflow‑first (Graph/Operator/Channel/Tensor; SPSC; zero‑copy) | Done | `crates/kernel/src/{graph.rs, channel/spsc.rs, tensor/*}`; METRICs in `graph.rs` |
| Executive Blueprint | Capabilities‑only security | Partial | Capability types/rights; used for model ops; not pervasive on all kernel objects. `crates/kernel/src/{cap.rs, model.rs}` |
| Executive Blueprint | Determinism by design (CBS+EDF) | Partial | Scheduler + admission control and demo; not default graph runtime. `crates/kernel/src/deterministic.rs`, `graph.rs::deterministic_demo()` |
| Executive Blueprint | Built‑in observability (Phase‑1) | Done | Operator p50/p95/p99, channel backpressure, scheduler timing; schema v1; validation script; dashboard card. `graph.rs`, `docs/schemas/*`, `crates/testing/src/{performance,reporting}` |
| Executive Blueprint | Minimal SSI (multi‑node) | Planned | RemoteChannel/transport/placement not implemented. |
| Control Plane | V0 binary control plane via shell (`graphctl`, `ctlhex`) | Done | `crates/kernel/src/{shell.rs, control.rs}`; emits `METRIC graph_stats_ops/channels` |
| Control Plane | VirtIO console host path | Partial | Opt‑in MMIO virtio‑console RX → `control::handle_frame`; QEMU devices in `scripts/uefi_run.sh`; host tool `tools/sis_datactl.py`. Capability token required in payload. (Binary framing, not CBOR.) `crates/kernel/src/{virtio_console.rs, virtio.rs, control.rs}` |
| Object Model & Data | OSEMN stage classification | Done | `graph.rs::Stage` + `graphctl --stage` mapping |
| Object Model & Data | Zero‑copy tensors/arena | Done | Bump arena + handle passing; typed DataTensor header present (`schema_id/quality/lineage`). `tensor/*`, metrics `zero_copy_*` |
| Object Model & Data | Channels & backpressure | Done | SPSC ring; metrics `channel_ab_depth_max/stalls/drops`. `channel/spsc.rs`, `graph.rs` |
| Data Analyst Pipelines | Control Plane MVP (virtio‑serial + CBOR; Acquire→Clean→Model) | Partial | End‑to‑end control via binary V0; A→B demo; Connect semantics via AddOperator in/out; CBOR not used. `shell.rs`, `control.rs`, `graph.rs` |
| Data Analyst Pipelines | Typed DataTensor (schema_id/quality/lineage; connect checks) | Done | Header fields implemented; strict connect‑time schema enforcement in graph; shell/control reject mismatches. `tensor/mod.rs`, `graph.rs::add_operator_strict`, `control.rs` |
| Data Analyst Pipelines | Error frames & quality counters | Partial | Quality counters (`quality_warns`) exist; dedicated error frames not implemented yet. |
| Data Analyst Pipelines | Deterministic subgraph | Partial | Deterministic demo with metrics; not wired to OSEMN pipeline. `deterministic.rs`, `graph.rs` |
| Data Analyst Pipelines | Data connectors (files/streams → batches) | Planned | Not implemented. |
| Scheduling | Static priority scheduler (Phase‑1) | Done | Highest‑priority runnable op, run‑to‑completion. `graph.rs::run_steps` |
| Scheduling | Deterministic CBS+EDF (admission, invariants) | Partial | Module + demo; invariants enforced in demo; integration pending. `deterministic.rs` |
| Security & Models | Capability core | Partial | Present & used by model ops; not yet gating all kernel objects/paths. `cap.rs` |
| Security & Models | Signed model packages + measurement log | Partial | Placeholder crypto (hash/signature) + audit + metrics. `model.rs` |
| Observability & Metrics | Tracepoint taxonomy (queued/started/completed) | Partial | Low‑overhead METRICs exist; textual trace in a few spots; not full taxonomy. `trace::trace`, `graph.rs` |
| Observability & Metrics | PMU attribution per‑operator | Partial | Guarded by `perf-verbose`; attribution in demo is minimized for QEMU variability. `pmu.rs`, `graph.rs` |
| Observability & Metrics | JSON Schema v1 + structured graphs | Done | `docs/schemas/sis-metrics-v1.schema.json`; `graphs` section optional; parser exports `graphs`. `crates/testing/src/performance/mod.rs` |
| Multi‑Node SSI | RemoteChannel & transport (virtio‑net) | Planned | Not implemented. |
| Multi‑Node SSI | Placement, heartbeats/epochs | Planned | Not implemented. |
| Validation & CI | Schemas + validation script + dashboard | Done | `scripts/validate-metrics.sh`; dashboard `target/testing/dashboard.html`. |
| Validation & CI | QEMU‑aware thresholds | Done | Runner selects thresholds via `SIS_CI_ENV` / `SIS_QEMU`. `crates/testing/src/bin/main.rs` |
| Validation & CI | Phase acceptance gates (P1/P2/P3) | Partial | Deterministic admission/miss metrics exist; zero‑copy vs copy 2× and remote p95 gates not implemented. |
| Immediate Actions (doc) | Modules graph/channel/tensor/cap | Done | Implemented. |
| Immediate Actions (doc) | Define schema v1 + update runner | Done | Implemented. |
| Immediate Actions (doc) | SPSC + arenas + tracepoints | Done | Implemented (tracepoints minimal; METRICs strong). |
| Immediate Actions (doc) | Sobel chain demo | Planned | Not implemented (current demo is A→B). |

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

## Data Analyst Pipelines (Design & Plan)

This section defines a higher‑level “Data Analyst” framework built on the kernel graph, without adding heavy logic to the kernel. The kernel remains a fast, deterministic data plane, while a userspace application orchestrates pipelines composed of five literal stages:

- Acquire Data: Ingestion from files/streams/endpoints into DataTensor batches
- Clean Data: Validation, normalization, transformation; quality metrics updated
- Explore Data: Statistics, sampling, and summary materialization
- Model Data: Training/inference operators (deterministic subgraphs where needed)
- Explain Results: Formatting, reporting, and actuation-ready outputs

### Kernel Hooks (implemented/planned)

- Implemented now:
  - Stage enum on operators: Acquire/Clean/Explore/Model/Explain (for attribution and filtering).
  - Minimal control plane V0: CreateGraph, AddChannel, AddOperator, StartGraph.
  - Shell front-ends: `graphctl` (friendly) and `ctlhex` (raw frames) to drive graphs during bring‑up.
  - Graph demo (A→B) with METRICs for totals, items, ns/item, per‑operator counts/times, arena remaining.
  - PMU demo/setup with QEMU caveat (cycles reliable; other events may be 0).

- Planned in P1 hardening:
  - VirtIO control path behind a feature flag (multiport binding, RX IRQ path, backpressure).
  - JSON schema v1 for graphs/operators/channels with stable keys.
  - Depth/backpressure metrics and zero‑copy counters in steady‑state runs.

### Kernel Support (Additions)

- DataTensor Header (minimal, typed):
  - `schema_id: u32` – identifies a schema known to userspace
  - `records: u32` – row count for batch semantics
  - `quality: { nulls: u32, errors: u32, flags: u32 }` – quick quality indicators
  - `lineage: { run_id: u64, upstream_op: u32, seq: u32 }` – provenance for replay/debug
  - All fields ride alongside `TensorHandle`; payload stays zero‑copy

- Stage Tags and Operator Hints:
  - `enum Stage { AcquireData, CleanData, ExploreData, ModelData, ExplainResults }`
  - Stored per operator; used in metrics, capability checks, and validation

- Sideband Metadata and Errors:
  - Per‑edge metadata frames to carry schema updates, quality rollups, and counters
  - Error frames: `(code, count, sample_offset)` with channel‑level error counters
  - Backpressure signals extended to allow “drain” or “quarantine” paths from CleanData

- Control Plane (message‑based, userspace driven):
  - Dedicated virtio‑serial port for control (e.g., `sis.datactl`), distinct from console
  - Framed CBOR/FlatBuffers commands:
    - `CreateGraph { graph_id }`
    - `AddOperator { graph_id, op_id, stage, config_id }`
    - `Connect { graph_id, src, dst }`
    - `StartGraph { graph_id }`, `StopGraph { graph_id }`, `DestroyGraph { graph_id }`
    - `EmitStats { graph_id }`
  - Capability checks on control ops: DataSourceCap for AcquireData, ModelCap for ModelData, SinkCap for ExplainResults

- Deterministic Subgraphs:
  - Apply admission control and EDF scheduling to ModelData/ExplainResults subgraphs when requested
  - Enforce “no dynamic alloc/unbounded loops/indefinite blocking” invariants for deterministic operators

### Userspace Application (First Userspace App)

- Python “Data Analyst” Orchestrator (initial target):
  - `sis_data_analyst.Graph()` builder; methods `.acquire()`, `.clean()`, `.explore()`, `.model()`, `.explain()`
  - Talks to the kernel via `/tmp/sis-datactl.sock` (virtio‑serial mapped socket)
  - Converts high‑level pipeline to control messages; pushes data batches for AcquireData
  - Integrates with pandas/pyarrow/scikit/PyTorch for cleansing, stats, and model logic where appropriate

- Data Types and Schemas:
  - Arrow‑like schema at the boundary (IDs carried in kernel), with mapping to kernel `schema_id`
  - Kernel validates `schema_id` continuity on channel connects; userspace is source of truth for full schema

- Observability & Lineage:
  - Kernel emits per‑op METRICs; userspace correlates with run_id and op IDs
  - Userspace collects lineage and quality summaries; persists runs for replay

### Milestones (Userspace + Kernel)

1) Control Plane MVP:
   - Virtio‑serial port + CBOR command schema; `CreateGraph/AddOperator/Connect/StartGraph` end‑to‑end
   - Synthetic Acquire→Clean→Model pipeline with METRICs

2) Typed DataTensor:
   - Add `schema_id`, `records`, `quality`, `lineage` to headers; enforce `schema_id` checks on connects

3) Error Frames & Backpressure:
   - CleanData emits error frames + quality counters; graph exposes per‑edge error METRICs

4) Deterministic Subgraph:
   - Admit and run a small deterministic ModelData subgraph; emit admission and miss metrics

5) Data Connectors:
   - Userspace AcquireData pulls from files/streams → kernel batches; measure end‑to‑end latency/throughput

### Security & Capabilities

- DataSourceCap: allows AcquireData from a specific source URI
- ModelCap: allows ModelData execution for a signed model package (hash + measurement)
- SinkCap: allows ExplainResults to write to configured sinks (files/streams)
- All control ops are capability‑checked; kernel remains mechanism, userspace defines policy

## Scheduling

### Phase 1: Static Priority Scheduler

- Topologically sorted operators; run‑to‑completion; pick highest static priority among runnable ops (all inputs ready, output space available).
- Yield only at op completion; blocked producers become runnable when space exists.

### Phase 2: Deterministic (CBS + EDF Hybrid)

- Admission Control: For each deterministic graph submit (WCET, period, deadline); accept only if utilization bound holds.
- CBS Server per deterministic graph isolates runtime; EDF orders operator activations inside the server.
- Timer Discipline: Program ARM architected timer per deadline (event‑driven), not heavy periodic ticks.
- Invariants: No dynamic allocation, no unbounded loops, no indefinite blocking in deterministic operators; enforced by kernel checks and audits.

Deterministic Data Analyst Subgraphs

- Apply deterministic mode to ModelData and ExplainResults where deadlines/SLAs are required. Admission control and EDF ordering ensure predictable behavior; non‑deterministic stages (Acquire/Clean/Explore) remain best‑effort.

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
  - QEMU caveat: cycles are reliable; other architectural events may read as 0 depending on the build/CPU model. Attribute only supported events in emulation.

### Metrics & JSON (Schema v1)

- Metrics to emit (phase‑gated):
  - P1: op_latency_p50/p95/p99_ns, channel_depth_max, scheduler_run_us, zero_copy_count.
  - P2: deterministic_deadline_miss_count, deterministic_jitter_ns, model_load_success/fail.
  - P3: remote_channel_latency_p95_us, retry_rate, placement_time_ms, node_failures.
- JSON export: stable keys with a top‑level schema version; dedicated sections for graphs, operators, channels, and percentiles.

Current in‑tree METRIC keys (Phase 1, for reference):
- Graph demo: `graph_demo_total_ns`, `graph_demo_items`, `graph_demo_avg_ns_per_item`, `zero_copy_count`, `zero_copy_handle_count`, `op_a_runs`, `op_b_runs`, `op_a_total_ns`, `op_b_total_ns`, `arena_remaining_bytes`.
- PMU demo: `pmu_cycles`, `pmu_inst`, `pmu_l1d_refill` (subject to QEMU caveat above).
- Timing harness: `irq_latency_ns` samples + `[SUMMARY]`, `real_ctx_switch_ns` samples + `[SUMMARY]` (min count enforced by runner), `ctx_switch_ns`, `memory_alloc_ns`.

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
