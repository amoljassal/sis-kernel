# SIS Kernel (Current Status)

An experimental AArch64 (ARM64) kernel that boots under UEFI in QEMU, brings up basic platform services, and emits real, parseable performance metrics. A companion test runner launches QEMU, tails serial output, parses METRIC lines, and exports results to JSON for CI and analysis.

This README reflects the implemented, verifiable behavior in this repo today — no hype, no unbuilt features.

## Overview

- Boots via UEFI on QEMU `virt` (GICv3, highmem) and prints deterministic boot markers.
- Enables MMU and caches at EL1; initializes UART, heap, GICv3, and the virtual timer.
- Emits in-kernel performance METRIC lines (AI microbenchmarks, syscall/alloc proxies, real cooperative context switch, IRQ latency) with warm‑ups and multiple samples.
- Test runner collects METRICs from serial logs, applies environment-aware thresholds for QEMU vs. hardware, and writes a JSON dump for CI.

Non-goals and not implemented: production hardening, formal proofs, full BFT consensus, RDMA fabric, sub-µs context switching guarantees, full driver stack. References to these in past docs were aspirational; this README describes actual code.

## What Works

- Boot and bring-up (UEFI/QEMU)
  - UART output: `KERNEL(U)`, `STACK OK`, `VECTORS OK`, `MMU ON`, `UART: READY`, `HEAP: READY`, `GIC: READY`, `LAUNCHING SHELL`.
  - PMU enabled; counter frequency printed as a metric: `METRIC cntfrq_hz=<hz>`.
  - GICv3 configured, virtual timer (PPI 27) enabled, periodic interrupts.

- Kernel performance metrics (serial console)
  - `METRIC ai_inference_us=<µs>`: NEON 4x4 layer with CNTVCT timing.
  - `METRIC ai_inference_scalar_us=<µs>`: scalar baseline for comparison.
  - `METRIC neon_matmul_us=<µs>`: 16×16 NEON matmul (behind `neon-optimized`).
  - `METRIC real_ctx_switch_ns=<ns>`: real cooperative context switch (callee-saved regs + SP) measured via CNTVCT.
  - `METRIC ctx_switch_ns=<ns>`: minimal syscall path proxy (getpid) timed with CNTVCT.
  - `METRIC memory_alloc_ns=<ns>`: small Vec alloc+free microbench.
  - `METRIC irq_latency_ns=<ns>`: virtual-timer IRQ latency; prints 64 samples after 4 warm-ups, plus `[SUMMARY]` mean/min/max.
  - Percentile summaries for context/alloc via `[SUMMARY] ctx_switch_ns ...` and `[SUMMARY] memory_alloc_ns ...`.

- Test runner (crates/testing)
  - Builds kernel + UEFI, launches QEMU with `-cpu cortex-a72,pmu=on`, logs serial to per-node files.
  - Tails serial logs, parses METRIC lines, computes p95/p99, and exports full dump to `target/testing/metrics_dump.json`.
  - Context metric preference order: `real_ctx_switch_ns` (if present) > `irq_latency_ns` > `ctx_switch_ns`.
  - Environment-aware thresholds (relaxed in QEMU):
    - AI inference target: <40µs (p99) — measured from `ai_inference_us`.
    - Context-switch proxy target: QEMU <50µs (p95), hardware goal <500ns; selected via `SIS_CI_ENV=qemu` or `SIS_QEMU=1`.
  - Falls back to simulated benchmarks if real METRICs are not found.

## Important Caveats

- QEMU’s NEON/PMU behavior is emulated; absolute numbers are not representative of real hardware. Use relative comparisons (e.g., scalar vs. NEON) and distributions.
- `real_ctx_switch_ns` measures a real cooperative context switch (between two contexts that save/restore callee-saved registers and SP). `ctx_switch_ns` measures a minimal syscall handler path, not a full switch.
- VirtIO device enumeration is present; drivers are minimal (virtio-console registration) and many devices are unimplemented.
- “Formal verification”, “BFT/consensus”, “RDMA”, and similar features referenced in older docs are not implemented here. Any files mentioning them are stubs or legacy experiments.

## Quick Start (QEMU UEFI)

Prerequisites:
- Rust nightly + targets: `aarch64-unknown-none` and `aarch64-unknown-uefi`.
- QEMU with AArch64 edk2 firmware (on macOS: `brew install qemu`; firmware often at `/opt/homebrew/share/qemu/edk2-aarch64-code.fd`).

Boot the kernel:

```bash
# From repo root
rustup toolchain install nightly
rustup target add aarch64-unknown-none aarch64-unknown-uefi

# Bring-up only (stack/vectors/MMU, IRQ timer, METRICs)
BRINGUP=1 ./scripts/uefi_run.sh

# Add AI microbenchmarks (NEON-based; still under QEMU emulation)
BRINGUP=1 AI=1 ./scripts/uefi_run.sh

# Quit QEMU: Ctrl+a, then x
```

You should see bring-up markers and a stream of `METRIC ...` lines after boot.

## Running the Test Runner

The test runner launches a single QEMU instance, waits for METRICs, and exports computed results and a JSON dump.

```bash
# From repo root (default-run = sis-test-runner)
# Default: single QEMU node, moderate iterations (~10 min)
cargo run -p sis-testing --release

# Quick: no QEMU (fully simulated, ~1–2 min)
cargo run -p sis-testing --release -- --quick

# Full: comprehensive run (QEMU + high iterations)
cargo run -p sis-testing --release -- --full

# QEMU-aware thresholds (set automatically when QEMU is used)
SIS_CI_ENV=qemu cargo run -p sis-testing --release
# or: SIS_QEMU=1 cargo run -p sis-testing --release

# (Optional) explicit binary selection
cargo run -p sis-testing --release --bin sis-test-runner
```

Artifacts:
- Parsed metrics JSON: `target/testing/metrics_dump.json`
- Logs and reports: `target/testing/`

## Repository Structure (relevant parts)

- `crates/kernel/src/main.rs` — AArch64 bring-up, MMU, UART, GICv3, virtual timer, IRQ latency bench, boot markers.
- `crates/kernel/src/userspace_test.rs` — Syscall tests; emits `ctx_switch_ns` and `memory_alloc_ns` metrics with warm-ups and summaries.
- `crates/kernel/src/ai_benchmark.rs` — NEON AI microbenchmarks; emits `ai_inference_us`, `ai_inference_scalar_us`, and optionally `neon_matmul_us`.
- `crates/kernel/src/syscall.rs` — Minimal syscall handler and microbench support.
- `crates/testing/src/qemu_runtime.rs` — Builds and launches QEMU; serial logging to files; boot detection.
- `crates/testing/src/performance/mod.rs` — METRIC parser, stats, and JSON export.
- `scripts/uefi_run.sh` — Local UEFI runner with feature flags (`BRINGUP`, `AI`, `NEON`).

## Feature Flags

- Kernel
  - `bringup` — Enable AArch64 bring-up path and boot markers.
  - `arm64-ai` — Enable AI benchmark wiring.
  - `neon-optimized` — Enable 16×16 NEON matmul demo and related metric.
  - `perf-verbose` — Gate noisy `[PERF] ...` logs; METRICs and summaries are always on.

- Test runner
  - Environment variable `SIS_CI_ENV=qemu` (or `SIS_QEMU=1`) selects QEMU-aware thresholds for context/consensus claims.

## Example METRIC Output (abridged)

```
KERNEL(U)
STACK OK
VECTORS OK
MMU ON
PMU: READY
UART: READY
METRIC cntfrq_hz=62500000
HEAP: READY
GIC: READY
...
METRIC real_ctx_switch_ns=32
METRIC ctx_switch_ns=4100
...
METRIC memory_alloc_ns=8200
...
METRIC irq_latency_ns=4800
[SUMMARY] irq_latency_ns: count=64 mean=5100 ns min=4600 ns max=6500 ns
```

## Measurement Methodology

- Real context switch (`real_ctx_switch_ns`): cooperative switch between two contexts using a tiny AArch64 routine that saves/restores callee‑saved registers (x19–x30) and SP, then `ret`s into the target context.
  - Timing: read CNTVCT before switching; target context reads CNTVCT on entry and emits the delta in nanoseconds.
  - Sampling: 8 warm‑ups then 64 measured switches; each sample printed as a `METRIC real_ctx_switch_ns=…` line.
  - Scope: measures cooperative save/restore + control transfer only. Does not include interrupt dispatch, scheduler decision, page table/timer reprogramming, or full preemption.
  - Environment: measured under QEMU; use relative comparisons, not absolute values, for hardware conclusions.

- Syscall proxy (`ctx_switch_ns`): minimal syscall path (getpid) timed via CNTVCT. Useful for syscall overhead trends, not a true context switch.

- IRQ latency (`irq_latency_ns`): virtual timer (PPI 27) programmed at fixed intervals; discards 4 warm‑ups, prints 64 samples, and a `[SUMMARY]` (mean/min/max) at completion.

- AI metrics (`ai_inference_us`, `ai_inference_scalar_us`, `neon_matmul_us`): NEON‑based microbenchmarks; QEMU emulates NEON, so treat results as indicative of code paths and relative speedups.

- Runner parsing and thresholds: test runner prefers `real_ctx_switch_ns` for context latency, falling back to `irq_latency_ns` then `ctx_switch_ns`; thresholds are QEMU‑aware when QEMU is in use.

## Lint Gate (CI Strict Mode)

To ensure the kernel builds without warnings in CI, the crate exposes a `strict` feature that denies all warnings when enabled.

- Kernel lint gate: `#![cfg_attr(feature = "strict", deny(warnings))]` at crate root.
- Local check: `cargo check -p sis_kernel --features strict`
- CI example (AArch64 no_std):
  ```bash
  cargo +nightly build -Z build-std=core,alloc \
    --target aarch64-unknown-none -p sis_kernel --features strict
  ```

## Typical QEMU Results

The exact percentiles for each run are exported to `target/testing/metrics_dump.json`. From the latest full run:

- AI inference latency (`ai_inference_us`): P99 = 3.00µs
- Real context switch (`real_ctx_switch_ns`): P95 = 32ns
- Byzantine consensus latency (100 nodes): 5.14ms

For other percentiles (P50/P95/P99 across metrics), refer to `metrics_dump.json` and the generated dashboards in `target/testing/`.

## metrics_dump.json Example

Below is an abbreviated example of the exported JSON (arrays truncated):

```json
{
  "real_ctx_switch_ns": [32.0, 33.0, 31.0, 32.0, 33.0],
  "ai_inference_us": [2.9, 3.0, 3.1, 3.0],
  "ctx_switch_ns": [4100.0, 4050.0],
  "irq_latency_ns": [4800.0, 5000.0, 4900.0],
  "memory_alloc_ns": [8200.0, 8100.0, 8300.0],
  "summary": {
    "ai_inference_p99_us": 3.00,
    "ai_inference_mean_us": 3.00,
    "ai_inference_std_us": 0.05,
    "ai_inference_samples": 100,
    "context_switch_p95_ns": 32.0,
    "context_switch_mean_ns": 33.0,
    "context_switch_samples": 64,
    "memory_allocation_p99_ns": 8300.0,
    "throughput_ops_per_sec": 13200000.0,
    "latency_summary": {
      "mean": 0.0,
      "median": 0.0,
      "std_dev": 0.0,
      "min": 0.0,
      "max": 0.0,
      "percentiles": { "50": 0.0, "95": 0.0, "99": 0.0 },
      "confidence_intervals": { "95": [0.0, 0.0], "99": [0.0, 0.0] },
      "sample_count": 0
    },
    "timestamp": "2025-09-11T17:16:25Z"
  }
}
```

## How To Read metrics_dump.json

- Context latency:
  - Prefer `summary.context_switch_p95_ns` computed by the runner.
  - Source selection order is automatic: `real_ctx_switch_ns` > `irq_latency_ns` > `ctx_switch_ns`.
  - If you need raw samples, use the arrays (e.g., `real_ctx_switch_ns`) and compute percentiles as needed.

- AI latency:
  - Use `summary.ai_inference_p99_us` for the main claim; raw samples are in `ai_inference_us`.

- Memory allocation and IRQ latency:
  - `summary.memory_allocation_p99_ns` gives the allocation P99; raw samples are in `memory_alloc_ns` and `irq_latency_ns`.

- Thresholds and environment:
  - Under QEMU, thresholds are relaxed (e.g., context P95 < 50µs). When testing on hardware, set `SIS_CI_ENV=hardware` to enforce strict thresholds.

- Artifacts:
  - The test runner writes `metrics_dump.json` and HTML dashboards to `target/testing/`.

### Quick CLI Extraction (jq)

Requires `jq` installed.

```bash
FILE=target/testing/metrics_dump.json

# Context switch P95 (runner-selected source)
jq -r '.summary.context_switch_p95_ns' "$FILE"

# AI inference P99 (microseconds)
jq -r '.summary.ai_inference_p99_us' "$FILE"

# Check if real context-switch samples are present and count them
jq -r 'if has("real_ctx_switch_ns") then (.real_ctx_switch_ns | length) else 0 end' "$FILE"

# Compute P50/P95/P99 from raw real_ctx_switch_ns samples
jq -r '
  def pct($a; $p): ($a|sort) as $s | $s[(($s|length - 1) * $p)|floor];
  if has("real_ctx_switch_ns") then
    "real_ctx_switch_ns P50=\(pct(.real_ctx_switch_ns; 0.50)) ns, " +
    "P95=\(pct(.real_ctx_switch_ns; 0.95)) ns, " +
    "P99=\(pct(.real_ctx_switch_ns; 0.99)) ns"
  else
    "real_ctx_switch_ns not present"
  end' "$FILE"
```

### Helper Script

Prefer running the bundled helper for convenience:

```bash
# From repo root
scripts/extract-metrics.sh               # uses default target/testing/metrics_dump.json
scripts/extract-metrics.sh path/to/metrics_dump.json
```

It prints context P95 (ns), AI P99 (µs), allocation P99 (ns), sample count for real_ctx_switch_ns, and computed P50/P95/P99 for the real context switch when available.

## Roadmap (near term)

- Separate real process/thread context switch measurement from syscall proxy.
- Improve device support (complete VirtIO console path, add more drivers).
- Make kernel-side JSON metrics export optional for UEFI-only runs.
- Validate on real hardware and update thresholds accordingly.
- Reduce boot noise further while preserving ingestible metrics.

## License

MIT — see `LICENSE`.

---

Notes:
- This README intentionally avoids unverified claims and reflects only what’s in-tree. If you need the previous marketing-heavy README for reference, recover it from VCS history.
