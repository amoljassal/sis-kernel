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
pub mod neural_acceleration;
pub mod scheduler;
pub mod simulator;
pub mod validation;
pub mod property_tests;
pub mod fault_injection;
pub mod cfvs;
pub mod chaos_engineering;
pub mod dcon;
pub mod cross_domain_sync;
pub mod software_synthesis;
pub mod natural_language;
pub mod design_graph;
pub mod rtl_safety;
pub mod hardware_synthesis;
pub mod eda_orchestration;
pub mod yosys_driver;

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

        // Initialize unified AI workload scheduler
        scheduler::init_ai_scheduler()?;
        serial::write_str("[ai] Unified AI scheduler initialized\n");

        // Initialize Neural Engine behavioral simulator
        simulator::init_neural_simulator()?;
        serial::write_str("[ai] Neural Engine simulator initialized\n");

        // Initialize AI validation framework
        validation::init_validation_framework()?;
        serial::write_str("[ai] AI validation framework initialized\n");

        // Initialize property-based testing framework
        property_tests::init_property_testing()?;
        serial::write_str("[ai] Property-based testing framework initialized\n");

        // Initialize fault injection and recovery validation
        fault_injection::init_fault_injection()?;
        serial::write_str("[ai] Fault injection framework initialized\n");

        // Initialize CFVS distributed testing orchestrator
        cfvs::init_cfvs()?;
        serial::write_str("[ai] CFVS distributed testing orchestrator initialized\n");

        // Initialize chaos engineering framework
        chaos_engineering::init_chaos_engineering()?;
        serial::write_str("[ai] Chaos engineering framework initialized\n");

        // Initialize Design Contract (DCON) system
        dcon::init()?;
        serial::write_str("[ai] Design Contract (DCON) system initialized\n");

        // Initialize Cross-Domain Synchronization Engine
        cross_domain_sync::init()?;
        serial::write_str("[ai] Cross-Domain Synchronization Engine initialized\n");

        // Initialize Software Synthesis Engine
        software_synthesis::init()?;
        serial::write_str("[ai] Software Synthesis Engine initialized\n");

        // Initialize Natural Language Interface
        natural_language::init()?;
        serial::write_str("[ai] Natural Language Interface initialized\n");

        // Initialize Design Graph Database (Phase 2)
        design_graph::init()?;
        serial::write_str("[ai] Design Graph Database initialized\n");

        // Initialize RTL Safety Validation Pipeline (Phase 2)
        rtl_safety::init()?;
        serial::write_str("[ai] RTL Safety Validation Pipeline initialized\n");

        // Initialize Hardware Synthesis Engine (Phase 2)
        hardware_synthesis::init()?;
        serial::write_str("[ai] Hardware Synthesis Engine initialized\n");

        // Initialize EDA Tool Orchestration (Phase 2B)
        eda_orchestration::init()?;
        serial::write_str("[ai] EDA Tool Orchestration initialized\n");

        // Register Yosys driver
        if let Err(_) = yosys_driver::register_yosys_with_orchestrator() {
            serial::write_str("[ai] Warning: Could not register Yosys driver\n");
        }

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
    /// Real-time neural network inference
    RealTimeInference,
    /// Interactive AI responses
    Interactive,
    /// Background model training/fine-tuning
    Background,
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
