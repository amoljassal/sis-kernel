# Multi-AI Consultation: Critical CI Timeout Resolution (Exit Code 124)

## Problem Statement
The SIS Kernel CI pipeline consistently fails with exit code 124 (timeout) on all Extended Lane tests:
- SCHED_FAIR_METER - SMP scheduler fairness test
- SCHED_PREEMPT_RR - Preemptive round-robin scheduler test  
- SMP_AFFINITY - CPU affinity test
- IPC_XCPU_PING - Cross-CPU IPC test
- VFIO_MSI_SMOKE - VFIO MSI interrupt test

All tests timeout after 60 seconds despite having ci_fast configuration enabled which reduces iteration counts.

## Environment Details
- **CI Platform**: GitHub Actions on Ubuntu 22.04
- **QEMU Version**: qemu-system-x86_64 with TCG acceleration (no KVM in CI)
- **Rust Toolchain**: nightly-2025-08-01
- **Architecture**: x86_64-unknown-none (bare metal)
- **Key Features**: SMP (2 CPUs), APIC, scheduler, selftests

## Symptoms
1. Tests work locally on Mac M1 (ARM64 host)
2. Tests fail consistently in CI with timeout exit code 124
3. Serial output suggests kernel boots but tests hang
4. Timeout occurs even with reduced iteration counts (ci_fast)
5. Fast Lane tests (USR_INIT) pass successfully

## Code Context
```rust
// Test configuration with ci_fast
#[cfg(ci_fast)]
let (samples, delay_ms) = (40, 3);  // Reduced for CI
#[cfg(not(ci_fast))]
let (samples, delay_ms) = (200, 5); // Normal iterations

// SMP initialization timeout
const SMP_TIMEOUT_MS: u64 = 10_000; // 10 second timeout
```

---

## Prompt for Grok (Performance & Low-Level Expert)

**Context**: I'm developing a bare-metal x86_64 kernel in Rust. Our CI tests are timing out with exit code 124 when running SMP/scheduler tests under QEMU TCG (software emulation, no KVM). The tests pass locally on ARM64 Mac but fail in GitHub Actions Ubuntu CI.

**Key observations**:
1. QEMU runs with `-machine q35,accel=tcg` (no hardware acceleration)
2. Tests use 2 SMP cores (`-smp 2`)
3. Timeout happens after 60 seconds
4. Tests use busy-wait loops and spin delays
5. The kernel implements custom LAPIC timer, IPI, and scheduler

**Question**: What are the most likely causes for SMP tests timing out under QEMU TCG emulation? Consider:
- TCG performance overhead for SMP synchronization
- LAPIC timer calibration issues under emulation
- IPI delivery delays in software emulation
- Spin loop behavior differences between TCG and real hardware
- Potential deadlocks or race conditions exposed by TCG's timing

Please provide specific optimization strategies for making SMP tests more robust under QEMU TCG.

---

## Prompt for ChatGPT (Systems Architecture & Safety Expert)

**Context**: I have a bare-metal OS kernel with custom SMP initialization that's experiencing consistent timeouts in CI but works locally. The kernel implements:
- Custom AP (Application Processor) boot via INIT-SIPI-SIPI
- Per-CPU data structures with atomic synchronization
- Lock-free scheduler with multiple runqueues
- Cross-CPU IPC via mailboxes

**Problem**: All SMP-related tests timeout after 60 seconds in GitHub Actions CI (QEMU TCG) but work on local hardware. The timeout suggests a hang or infinite loop rather than just slow execution.

**Code patterns**:
```rust
// AP boot synchronization
static AP_BOOT_STATUS: [AtomicU32; MAX_CPUS] = /* ... */;

// Spin-wait for AP boot
for elapsed_ms in 0..SMP_TIMEOUT_MS {
    if AP_BOOT_STATUS[cpu].load(Ordering::Relaxed) == AP_BOOT_SUCCESS {
        break;
    }
    simple_delay_us(1000); // 1ms delay
}
```

**Questions**:
1. What synchronization primitives might behave differently under QEMU TCG vs real hardware?
2. Could memory ordering issues cause APs to never signal completion?
3. What defensive programming techniques would make SMP initialization more robust?
4. Should we implement different timeout/retry strategies for emulated environments?

Please suggest concrete fixes for reliable SMP initialization under emulation.

---

## Prompt for Gemini (Distributed Systems & Testing Expert)

**Context**: Our OS kernel CI pipeline has 100% failure rate on SMP tests with timeout (exit code 124). The test matrix includes:

| Test | Features | SMP | Purpose |
|------|----------|-----|---------|
| SCHED_FAIR_METER | scheduler, smp | 2 | Verify fair scheduling |
| SMP_AFFINITY | affinity, smp | 2 | Test CPU pinning |
| IPC_XCPU_PING | smp | 2 | Cross-CPU messaging |
| SCHED_PREEMPT_RR | scheduler, smp | 2 | Round-robin preemption |
| VFIO_MSI_SMOKE | vfio, apic | 1 | MSI interrupt handling |

**Environment differences**:
- **Local (WORKS)**: Mac M1, QEMU with HVF acceleration, ARM64 host
- **CI (FAILS)**: Ubuntu 22.04, QEMU with TCG (software), x86_64 host, containerized

**Question**: Design a comprehensive debugging strategy to identify why these tests timeout. Consider:
1. How to add telemetry without affecting timing
2. Techniques to detect hangs vs slow execution  
3. Methods to reproduce CI environment locally
4. Progressive test isolation strategies
5. Timeout detection and recovery mechanisms

Provide a systematic approach to diagnose and fix these timeout issues.

---

## Prompt for Claude (Synthesis & Implementation)

After receiving responses from Grok, ChatGPT, and Gemini about the CI timeout issues, synthesize their recommendations into an actionable implementation plan. 

**Grok's likely focus**: TCG performance bottlenecks, timer calibration, spin loop optimizations
**ChatGPT's likely focus**: Memory ordering, synchronization primitives, defensive coding
**Gemini's likely focus**: Test infrastructure, debugging tools, environment parity

Create a prioritized implementation plan that:
1. Identifies the most probable root cause based on consensus
2. Provides immediate mitigation (quick fixes)
3. Implements robust long-term solutions
4. Adds diagnostic capabilities for future issues
5. Ensures tests work reliably across all environments

Generate the actual code changes needed, focusing on:
- SMP initialization robustness
- Timer calibration for emulated environments
- Test timeout detection and graceful failure
- CI-specific optimizations

---

## Expected Outcomes

### Immediate Fixes (Priority 1)
- Increase spin delays for TCG detection
- Add progress indicators in SMP init
- Implement early timeout detection

### Robust Solutions (Priority 2)  
- Adaptive timing based on environment detection
- Fallback paths for failed SMP initialization
- Separate CI test configurations

### Diagnostic Improvements (Priority 3)
- Detailed logging with timestamps
- Hang detection mechanisms
- Performance profiling markers

## Success Criteria
- All Extended Lane tests pass in CI within 60 seconds
- Tests remain deterministic and reliable
- No regression in local test performance
- Clear diagnostics when failures occur