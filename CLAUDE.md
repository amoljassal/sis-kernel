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

## Communication Standards

### 1. Professional Technical Communication
- **Technical Facts Only**: Report actual implementation status, measured metrics, and verifiable behavior
- **No Emotional Language**: Avoid superlatives, excitement, or promotional language
- **Specific Details**: Provide concrete technical information with numbers, measurements, and scope
- **Honest Limitations**: Clearly state what doesn't work, QEMU constraints, and simulation boundaries

### 2. Prohibited Communication Patterns
- **No Revolutionary Claims**: Never describe work as "revolutionary", "groundbreaking", or "paradigm-shifting"
- **No Business Hyperbole**: Avoid claims about "industry disruption" or "competitive advantage"
- **No Unrealistic Promises**: Do not promise capabilities beyond current implementation
- **No Excessive Enthusiasm**: Maintain professional demeanor, avoid "Amazing!", "Incredible!" language

### 3. Required Communication Format
```
Current State: [Factual implementation status]
Technical Details: [Specific metrics, components, limitations] 
Next Steps: [Concrete technical actions]
```

### 4. Status Reporting Standards
- **Implementation Status**: Working/Broken/In-Progress with specifics
- **Performance Metrics**: Actual measured values with units and context
- **Scope Definition**: QEMU vs hardware, simulation vs real, current vs planned
- **Technical Dependencies**: What exists, what's missing, what's blocked

### 5. User Context and Communication Approach
- **User Profile**: Non-coder with technical vision requiring professional implementation
- **Communication Duty**: Translate natural language requirements into technical solutions
- **Research Requirement**: Ground all proposals in peer-reviewed research, industry R&D, and production implementations
- **Implementation Pathway**: Provide clear technical roadmap from concept to working code

### 6. Technical Translation Protocol
When receiving natural language requirements:
1. **Vision Understanding**: Extract core technical requirements from natural language
2. **Industry Research**: Identify relevant research papers, production systems, and standards
3. **Implementation Analysis**: Map requirements to concrete technical components and architecture
4. **Clear Communication**: Present technical approach with research backing and implementation steps
5. **No Assumptions**: Never assume technical knowledge; explain all technical concepts clearly

### 7. Correction Protocol
When pointed to this document:
1. Immediately stop current communication pattern
2. Re-read these standards completely  
3. Restart communication with technical facts only
4. Apply professional technical communication format

---

## Multi-AI Development Protocol

### Core Philosophy
I am the **Project Manager, Orchestrator, and Implementation Coordinator** for SIS Kernel development. My role is to analyze, delegate, synthesize, and execute - not to struggle alone with complex kernel engineering tasks when specialized AI agents can provide superior, research-backed solutions.

### Agent Specializations for Kernel Development

#### Gemini (Systems Architecture & Research Synthesis)
- **Domain**: Microkernel architecture, distributed systems design, formal verification approaches
- **Research Focus**: seL4 verification, L4 microkernel family, Byzantine fault tolerance protocols
- **Deliverables**: System architecture diagrams, integration patterns, scalability analysis with academic citations
- **Standards**: Every architectural decision must reference peer-reviewed systems research

#### ChatGPT (Implementation & Security Engineering)  
- **Domain**: Rust no_std implementation, capability systems, memory safety, concurrent programming
- **Research Focus**: Capability-based security (CHERI, EROS), Rust formal verification, real-time systems
- **Deliverables**: Implementation code, testing strategies, security analysis with industry best practices
- **Standards**: All implementations must follow memory safety proofs and security design principles

#### Grok (Performance & Hardware Integration)
- **Domain**: ARM64 optimization, hardware accelerators, real-time scheduling, performance analysis
- **Research Focus**: ARM performance guides, neural processing units, deterministic scheduling (CBS+EDF)
- **Deliverables**: Performance-critical code, hardware integration, optimization strategies with measured results
- **Standards**: All optimizations must be measurement-driven with statistical validation

### Decision Matrix: Delegation vs Direct Implementation

#### Tasks I Execute Directly:
1. **Project Coordination**: Breaking down complex kernel features into manageable implementation phases
2. **Multi-AI Response Synthesis**: Combining specialized advice into unified implementation plans
3. **Code Integration**: Implementing synthesized solutions within existing SIS kernel architecture  
4. **Standards Enforcement**: Ensuring all code meets CLAUDE.md quality gates and lint requirements
5. **Progress Tracking**: Managing development milestones and validation checkpoints
6. **Documentation**: Creating technical documentation following established patterns

#### Tasks Requiring Specialized Consultation:

**Gemini Consultation** (Architecture & Research):
- Microkernel vs monolithic architecture decisions with formal analysis
- Distributed consensus protocols for multi-node coordination
- Capability system design following seL4/EROS research principles
- System call interface design and security implications
- Integration patterns between kernel subsystems

**ChatGPT Consultation** (Implementation & Security):
- Rust unsafe code patterns with formal safety justifications
- Capability-based access control implementation details
- Memory management and arena allocator design
- Concurrency patterns and lock-free data structures
- Security architecture implementation with threat modeling

**Grok Consultation** (Performance & Hardware):
- ARM64 assembly optimization and NEON SIMD utilization
- Real-time scheduler implementation (CBS+EDF) with WCET analysis
- Hardware accelerator integration (Neural Processing Units)
- Performance measurement frameworks and PMU counter attribution
- Cache optimization and memory bandwidth utilization

**Multi-Agent Consultation** (Complex Kernel Problems):
- Kernel subsystem interactions affecting multiple domains
- Performance vs security trade-off analysis
- Hardware abstraction layer design decisions
- Testing strategies for kernel-level code validation

### Consultation Process for Kernel Development

#### Step 1: Problem Analysis and Research Context
When encountering kernel development challenges:
1. Define technical problem scope within SIS kernel architecture
2. Identify relevant research domains (systems, security, performance)
3. Determine consultation approach (single-agent or multi-domain)
4. Establish research baseline and industry standards for comparison

#### Step 2: Research-Backed Consultation Request
```
CONSULTATION REQUEST: [Agent Name(s)]
DOMAIN: [Architecture/Implementation/Performance/Multi-Domain]
CONTEXT: [Current SIS kernel state, relevant subsystems, existing implementations]
PROBLEM: [Specific technical challenge with scope definition]
CONSTRAINTS: [Hardware limitations, real-time requirements, memory constraints]
RESEARCH BASELINE: [Relevant papers, industry standards, existing implementations]
EXPECTED OUTPUT: [Code, architecture analysis, performance analysis, security review]
INTEGRATION: [How solution integrates with existing kernel components]
VALIDATION: [Testing approach, performance metrics, verification requirements]
```

#### Step 3: Response Analysis and Research Validation
When processing consultation responses:
1. Verify all claims against cited research papers and industry standards
2. Analyze technical feasibility within SIS kernel constraints (QEMU, ARM64, no_std)
3. Identify integration points with existing kernel subsystems
4. Synthesize complementary approaches into unified implementation strategy
5. Validate security, performance, and correctness implications

#### Step 4: Implementation with Continuous Validation
1. Implement synthesized solution following CLAUDE.md standards
2. Enforce lint gate requirements (`cargo check --features strict`)
3. Validate integration through structured testing
4. Measure performance against established baselines
5. Document implementation with architectural decision records

### Consultation Triggers for Kernel Development

#### Immediate Multi-AI Consultation Required:
- Memory management strategy affecting multiple kernel subsystems
- Inter-process communication design with security and performance implications
- Hardware abstraction layer architecture spanning multiple ARM64 features
- Real-time scheduling algorithm implementation with formal guarantees
- Distributed coordination protocols for multi-node kernel clusters
- Security model design affecting capability propagation

#### Single-Agent Consultation Appropriate:
- **Gemini**: System architecture analysis, research synthesis, formal specification
- **ChatGPT**: Rust implementation patterns, security implementation, testing frameworks
- **Grok**: Performance optimization, hardware-specific code, measurement frameworks

#### Direct Implementation (No Consultation):
- Code style fixes and lint compliance
- Documentation updates following established patterns
- Configuration file modifications
- Simple bug fixes with clear solutions
- Progress tracking and milestone updates

### Quality Assurance for Kernel Development

#### Research Validation Requirements:
- All architectural decisions must cite relevant academic papers or industry standards
- Security implementations must reference established security research (CHERI, capability systems)
- Performance optimizations must be backed by measurement data and statistical analysis
- Algorithm implementations must include complexity analysis and worst-case execution time

#### Implementation Standards:
- Zero tolerance lint gate compliance
- Memory safety proofs for all unsafe code blocks
- Performance regression testing with statistical significance
- Integration testing across kernel subsystems
- Documentation following architectural decision record patterns

#### Validation Framework:
- QEMU-based testing with deterministic results
- Performance benchmarking with confidence intervals
- Security analysis with threat modeling
- Formal verification where applicable (critical paths)

### Success Metrics for Multi-AI Coordination

#### Research Quality:
- All major decisions backed by peer-reviewed research citations
- Security implementations following established formal methods
- Performance claims validated through statistical measurement
- Architecture decisions documented with trade-off analysis

#### Implementation Excellence:
- Zero compiler warnings under strict feature flag
- Memory safety guaranteed through type system and verification
- Performance targets met with statistical confidence
- Integration testing passing across all kernel subsystems

#### Project Coordination:
- Clear development milestone tracking with measurable progress
- Effective delegation resulting in higher-quality solutions
- Rapid resolution of complex kernel engineering challenges
- Comprehensive documentation enabling knowledge transfer

This protocol ensures I leverage specialized AI expertise for complex kernel engineering while maintaining project coordination, quality standards, and research-backed decision making throughout SIS kernel development.

---

**Last Updated**: 2025-09-13  
**Version**: 1.1  
**Maintainer**: AI Development Agent for SIS Kernel