//! AI-Native Kernel Subsystem
//!
//! This module implements the core AI subsystem for SIS Kernel, providing:
//! - Cognitive task scheduling with <1ms latency guarantees
//! - AI-aware memory management and resource allocation
//! - Distributed AI orchestration via Cognitive Fabric
//! - Hardware acceleration integration (NPU/GPU)
//! - Real-time inference coordination
//!
//! Design Philosophy:
//! - Memory safety first with safe Rust primitives
//! - Lock-free data structures for real-time guarantees
//! - Zero-copy operations for minimal overhead
//! - Predictable latency for cognitive workloads

pub mod cognitive_scheduler;
pub mod distributed;
pub mod fabric;
pub mod hardware_accel;
pub mod inference;
pub mod memory_pool;
pub mod primitives;

#[cfg(target_arch = "aarch64")]
pub mod cognitive_scheduler_arm64;

use crate::kernel::serial;
use crate::kernel::types::Tid;

/// AI subsystem initialization state
static mut AI_INITIALIZED: bool = false;

/// Initialize the AI subsystem
///
/// This must be called during kernel boot after basic memory management
/// and SMP initialization are complete.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        if AI_INITIALIZED {
            return Ok(());
        }

        serial::write_str("[ai] Initializing AI-Native Kernel Subsystem...\n");

        // Initialize core AI primitives
        primitives::init()?;
        serial::write_str("[ai] Safe primitives initialized\n");

        // Initialize AI-aware memory pools
        memory_pool::init()?;
        serial::write_str("[ai] AI memory pools initialized\n");

        // Initialize cognitive scheduler
        cognitive_scheduler::init()?;
        serial::write_str("[ai] Cognitive scheduler initialized\n");

        // Initialize Cognitive Fabric for distributed coordination
        fabric::init()?;
        serial::write_str("[ai] Cognitive Fabric initialized\n");

        // Initialize hardware acceleration subsystem
        hardware_accel::init()?;
        serial::write_str("[ai] Hardware acceleration initialized\n");

        // Initialize ARM64 AI acceleration if available
        #[cfg(target_arch = "aarch64")]
        {
            crate::arch::arch_impl::init()?;
            cognitive_scheduler_arm64::init()?;
            serial::write_str("[ai] ARM64 AI acceleration initialized\n");
        }

        AI_INITIALIZED = true;
        serial::write_str("[ai] AI subsystem fully initialized\n");
        Ok(())
    }
}

/// Check if AI subsystem is ready
pub fn is_initialized() -> bool {
    unsafe { AI_INITIALIZED }
}

/// AI task priority levels for cognitive scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CognitivePriority {
    /// Real-time inference tasks (<1ms latency guarantee)
    RealTimeInference = 0,
    /// Interactive AI responses (<10ms target)
    Interactive = 1,
    /// Background model training and optimization
    Background = 2,
    /// System maintenance and cleanup
    Maintenance = 3,
}

/// AI workload type classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkloadType {
    /// Neural network inference
    Inference,
    /// Model training/fine-tuning
    Training,
    /// Data preprocessing
    Preprocessing,
    /// Model serving/deployment
    Serving,
    /// Data processing operations
    DataProcessing,
}

/// Request cognitive scheduling for a task
pub fn schedule_cognitive_task(
    task_id: Tid,
    priority: CognitivePriority,
    workload_type: WorkloadType,
) -> Result<(), &'static str> {
    if !is_initialized() {
        return Err("AI subsystem not initialized");
    }

    cognitive_scheduler::schedule_task(task_id, priority, workload_type)
}
