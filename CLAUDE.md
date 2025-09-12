# CLAUDE.md - SIS Kernel Development Standards

## Project: SIS Kernel
**AI-Native AArch64 Kernel with Dataflow Observability**

This document establishes the technical standards, architectural principles, and implementation guidelines for AI agents working on the SIS Kernel project. These standards must be maintained or elevated, never lowered.

---

## Core Philosophy

### 1. Factual Engineering Over Hype
- **Documentation Rule**: Document only implemented, verifiable behavior. No aspirational claims.
- **README Standard**: "This README reflects the implemented, verifiable behavior in this repo today — no hype, no unbuilt features."
- **Metric Claims**: Every performance metric must be measurable and reproducible.
- **Feature Claims**: Only describe features that compile, run, and produce expected output.

### 2. Minimal, Focused Implementation
- **Module Principle**: Each module should do one thing exceptionally well.
- **Incremental Growth**: Add functionality in small, verifiable increments.
- **Scaffolding Approach**: Build minimal scaffolding first, then expand with proven patterns.
- **Technical Debt**: Prefer working minimal implementation over incomplete complex features.

---

## Code Quality Standards

### 1. Lint Gate (Zero Tolerance)
```rust
#![cfg_attr(feature = "strict", deny(warnings))]
```
- **Enforcement**: All code must pass `cargo check -p sis_kernel --features strict` with zero warnings.
- **CI Integration**: Strict mode is used in CI pipelines.
- **No Exceptions**: Fix warnings immediately, never suppress unless absolutely justified.

### 2. Documentation Standards

#### Module Documentation
```rust
//! Brief, factual description of module purpose.
//! Technical details about implementation approach.
//! Feature gates, dependencies, and usage notes.
```

#### Function Documentation
```rust
/// Brief description of what the function does (not why).
/// 
/// # Arguments
/// - `param`: Technical description of parameter constraints
/// 
/// # Returns
/// Technical description of return value and possible states
/// 
/// # Safety (for unsafe functions)
/// Precise safety requirements and caller obligations
```

#### Inline Comments
- **When**: Complex algorithms, unsafe blocks, performance-critical sections
- **Style**: Technical, concise, explain the "why" not the "what"
- **Architecture**: Document architectural decisions and trade-offs

### 3. Error Handling Standards

#### Custom Error Enums
```rust
pub enum ModuleError {
    BadInput,
    NotReady, 
    InternalFailure,
}
```
- **Naming**: Clear, specific error variants
- **Propagation**: Use `Result<T, E>` consistently
- **Context**: Provide sufficient context for debugging

#### Error Messages
- **Specificity**: Precise error descriptions for debugging
- **No User-Facing**: Errors are for developers, not end users
- **Actionable**: Include hints for resolution when possible

### 4. Performance Standards

#### Critical Path Optimization
```rust
#[inline(always)]
fn performance_critical_function() {
    // Implementation
}
```
- **Profiling-Driven**: Optimize based on actual measurements
- **Inline Annotations**: Use `#[inline(always)]` for hot paths
- **Zero-Copy**: Prefer zero-copy operations where possible
- **Lock-Free**: Use lock-free data structures for concurrent access

#### Memory Management
```rust
// Prefer stack allocation
let buffer: [u8; 1024] = [0; 1024];

// Use bump allocator for temporary data
let handle = arena.alloc(size, alignment)?;
```
- **Stack First**: Prefer stack allocation for small, fixed-size data
- **Bump Allocator**: Use arena allocation for temporary data
- **No Heap Fragmentation**: Minimize dynamic allocation in critical paths

### 5. Safety Standards

#### Unsafe Code
```rust
unsafe {
    // SAFETY: Detailed explanation of why this is safe
    // including all invariants and preconditions
    core::ptr::write(ptr, value);
}
```
- **Justification**: Every unsafe block must have SAFETY comment
- **Minimization**: Keep unsafe blocks as small as possible  
- **Encapsulation**: Wrap unsafe operations in safe abstractions

#### Concurrency
```rust
static mut GLOBAL_STATE: Option<State> = None;
```
- **Single-Writer**: Use single-writer, multiple-reader patterns
- **Atomic Operations**: Use atomic types for shared counters
- **Lock-Free**: Prefer lock-free data structures (SPSC queues)

---

## Architecture Standards

### 1. Dataflow-First Design
```rust
pub struct GraphDemo {
    arena: BumpArena<8192>,
    graph: GraphApi,
    operators: Vec<OpNode>,
    channels: Vec<Box<Spsc<TensorHandle, 64>>>,
}
```
- **Operator Model**: All computation as graph operators
- **Channel Communication**: SPSC channels for zero-copy data flow
- **Tensor Handles**: Handle-based tensor passing, not direct copying
- **Stage Classification**: OSEMN framework integration

### 2. Observability-First
```rust
// Every operation must be measurable
metric_kv("op_a_p95_ns", percentile_ns(&samples, 0.95) as usize);
metric_kv("channel_stalls", stall_count);
```
- **Comprehensive Metrics**: Track latency, throughput, and backpressure
- **Percentile Reporting**: p50/p95/p99 for latency distributions
- **PMU Integration**: Hardware counter attribution when available
- **Structured Export**: JSON Schema validated metrics

### 3. Feature Gate Architecture
```rust
#[cfg(feature = "graph-demo")]
pub fn run_graph_demo() { /* ... */ }

#[cfg(feature = "perf-verbose")]
use crate::pmu::aarch64 as pmu;
```
- **Modular Compilation**: Optional features behind gates
- **Runtime Detection**: Environment-aware behavior (QEMU vs hardware)
- **Debugging Features**: Verbose logging behind debug gates

---

## Implementation Patterns

### 1. Metric Emission Pattern
```rust
fn emit_performance_metrics(&self) {
    let start = now_cycles();
    // ... work ...
    let end = now_cycles();
    let ns = cycles_to_ns(end.saturating_sub(start));
    metric_kv("operation_ns", ns as usize);
}
```
- **Consistent Naming**: `operation_metric_unit` pattern
- **Overflow Protection**: Use `saturating_*` operations
- **Unit Clarity**: Always include units in metric names

### 2. Percentile Calculation Pattern
```rust
#[inline(always)]
fn percentile_ns(samples: &mut [u64; N], count: usize, p: f32) -> u64 {
    if count == 0 { return 0; }
    let slice = &mut samples[..count];
    slice.sort_unstable();
    let idx = ((count - 1) as f32 * p) as usize;
    slice[idx]
}
```
- **Fixed-Size Buffers**: Use compile-time sized arrays
- **In-Place Sorting**: Avoid allocation in hot paths
- **Bounds Checking**: Always validate array access

### 3. Control Plane Pattern
```rust
pub fn handle_frame(frame: &[u8]) -> Result<(), CtrlError> {
    // Validate frame header
    if frame.len() < HEADER_SIZE { return Err(CtrlError::BadFrame); }
    
    // Parse command
    match frame[CMD_OFFSET] {
        0x01 => handle_create_graph(),
        0x02 => handle_add_channel(&frame[PAYLOAD_OFFSET..]),
        _ => Err(CtrlError::Unsupported),
    }
}
```
- **Input Validation**: Check all inputs before processing
- **Binary Protocols**: Use little-endian for cross-platform compatibility
- **Extensible Commands**: Design for future command addition

---

## Testing Standards

### 1. Environment-Aware Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation_performance() {
        // Test should adapt to QEMU vs hardware
        let threshold = if cfg!(feature = "qemu-testing") {
            50_000 // Relaxed for QEMU
        } else {
            500 // Strict for hardware
        };
        assert!(measure_latency() < threshold);
    }
}
```
- **Adaptive Thresholds**: Different expectations for QEMU vs hardware
- **Deterministic Results**: Tests should not depend on timing variance
- **Metrics Validation**: Validate JSON Schema compliance

### 2. Performance Testing
```rust
fn benchmark_operation() -> Duration {
    // Warm-up phase
    for _ in 0..4 { operation(); }
    
    // Measurement phase
    let start = Instant::now();
    for _ in 0..100 { operation(); }
    start.elapsed() / 100
}
```
- **Warm-Up**: Always include warm-up iterations
- **Statistical Significance**: Use sufficient sample sizes
- **Outlier Handling**: Filter obvious outliers (zero deltas, etc.)

---

## Documentation Standards

### 1. Technical Accuracy
- **Measurement-Based**: All performance claims backed by measurements
- **Environment Context**: Clearly state QEMU vs hardware context
- **Reproducible Instructions**: Step-by-step reproduction procedures
- **Known Limitations**: Document current limitations and caveats

### 2. Schema Validation
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SIS Metrics Dump (v1)",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "schema_version": { "type": "string", "const": "v1" }
  }
}
```
- **Version Control**: All JSON exports must have schema versions
- **Validation Scripts**: Provide validation tools for exported data
- **Breaking Changes**: Increment schema version for breaking changes

### 3. Architectural Decision Records
```markdown
## ADR: Zero-Copy Tensor Passing

**Context**: Need high-performance tensor data movement between operators

**Decision**: Use handle-based tensor passing with bump allocator

**Rationale**: Eliminates copy overhead while maintaining memory safety

**Consequences**: Requires lifetime management and arena coordination
```
- **Decision Context**: Document why decisions were made
- **Trade-offs**: Explain alternatives considered and why rejected
- **Impact Assessment**: Document performance and complexity implications

---

## Quality Gates

### 1. Pre-Commit Validation
```bash
# All must pass before committing
cargo check -p sis_kernel --features strict
cargo test -p sis_kernel
cargo clippy -p sis_kernel --features strict -- -D warnings
```

### 2. Performance Regression Detection
- **Baseline Metrics**: Maintain performance baselines for key operations
- **Threshold Alerts**: Fail CI if performance degrades beyond thresholds
- **Trend Analysis**: Track performance trends over time

### 3. Schema Compliance
```bash
# Validate all JSON exports
scripts/validate-metrics.sh target/testing/
jsonschema -i metrics_dump.json docs/schemas/sis-metrics-v1.schema.json
```

---

## Escalation Levels

### Level 1: Maintain Standards
- Follow all existing patterns and conventions
- Add incremental improvements within established architecture
- Maintain or improve performance characteristics

### Level 2: Elevate Standards  
- Identify and eliminate technical debt
- Improve error handling and edge case coverage
- Enhance observability and debugging capabilities
- Optimize performance-critical paths

### Level 3: Architectural Evolution
- Design new subsystems following established principles
- Extend observability framework for new use cases
- Integrate advanced hardware features (PMU, SIMD, etc.)
- Scale architecture for production workloads

**Never**: Lower code quality, reduce test coverage, or compromise safety standards.

---

## Success Metrics

### Code Quality
- Zero warnings under `--features strict`
- 100% schema validation compliance
- No performance regressions
- All features documented and testable

### Architecture Health
- Clear module boundaries and responsibilities
- Minimal coupling between subsystems
- Consistent error handling patterns
- Comprehensive observability coverage

### Development Velocity
- Fast build times (< 30 seconds for incremental)
- Reliable test suite (< 5% flaky test rate)
- Easy debugging with structured metrics
- Clear development workflow documentation

---

**Last Updated**: 2025-09-12  
**Version**: 1.0  
**Maintainer**: AI Development Agent for SIS Kernel