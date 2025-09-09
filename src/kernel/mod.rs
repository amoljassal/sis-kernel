//! Kernel modules for tasks, scheduling, syscalls and devices.
//!
//! The kernel modules implement high level functionality on top of
//! the architecture specific code.  These modules are responsible
//! for creating and managing tasks, dispatching system calls,
//! scanning PCI devices and logging via the serial port.

pub mod affinity;
pub mod boot;
pub mod hal;
#[cfg(feature = "ipc")]
pub mod caps;
pub mod current;
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "smp")]
pub mod mm;
pub mod pci;
#[cfg(feature = "scheduler")]
pub mod sched_preempt;
pub mod scheduler;
pub mod serial;
#[cfg(feature = "scheduler")]
pub mod simple_scheduler;
#[cfg(feature = "smp")]
pub mod smp_scheduler;
pub mod spawn;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod task_table;
pub mod validation;
pub mod ai_syscalls;
pub mod auth;  // Soulprint Protocol authentication
pub mod memory;
pub mod vdso_manager;
pub mod hybrid_kernel;  // L4 microkernel + AI runtime hybrid
pub mod capability;     // EROS/CHERI-style capability system
pub mod ai_capability_bft; // Enhanced AI capabilities with distributed BFT consensus
pub mod ai_memory_safety; // Linear tensor types with Verus verification and DMA bounds checking
pub mod ai_dma_isolation; // DMA bounds checking for AI workload isolation
pub mod distributed_cognitive; // Network-transparent cognitive fabric with RDMA support
pub mod ai_migration; // Cross-device AI migration with checkpoint-restart system
pub mod ai_bft; // AI Byzantine Fault Tolerance with HotStuff consensus protocol
pub mod sis_fs;        // SIS File System with CoW and AI features
pub mod osemn_pipeline; // OSEMN cognitive pipeline with kernel acceleration
pub mod cognitive_runtime; // Cognitive runtime with dual-hemisphere coordination
pub mod asymmetric_scheduler; // Asymmetric scheduler for analytical vs creative tasks
pub mod hardware_optimization; // Hardware-specific optimizations (AMX, Neural Engine, multi-GPU)
pub mod power_thermal; // Power and thermal management with DVFS and predictive control
pub mod ai_training_lab; // AI Training Lab with MLX drivers and distributed training
pub mod types_old;
pub mod types;
pub mod vfio;
pub mod vfio_stubs;
#[cfg(feature = "scheduler")]
pub mod waitqueue;
#[cfg(all(feature = "smp", feature = "ipc"))]
pub mod xcpu_ipc;
#[cfg(feature = "smp")]
pub mod xcpu_mbox;

#[cfg(feature = "userland")]
pub mod user;

#[cfg(not(feature = "userland"))]
pub mod vfs;

#[cfg(not(feature = "userland"))]
pub mod initfs;

// AI-Native Kernel Subsystem
pub mod ai;

// Multi-AI synthesis modules for ARM64 compilation fixes
pub mod no_std_shims;

// Multi-AI Boot Framework modules
pub mod boot_metrics;
pub mod boot_recovery;

// Phase 1D Security Framework
pub mod security_framework;

// Phase 5: Production Hardening
pub mod formal_verification;
pub mod secure_enclave;
pub mod mlperf_benchmarking;
pub mod kernel_testing;
pub mod performance_validation;

// Phase 2: Security Layer
pub mod capabilities;
pub mod tpm;
pub mod security_test;
pub mod security;

// Phase 3: AI/ML Runtime
pub mod ai_runtime;
pub mod ai_scheduler;
pub mod ai_test;

// Provide stable re-exports for callers
pub use pci::read_id;
pub use pci::{cfg_read32, cfg_write32, find_first_e1000, PciId};
pub use types::Bdf;

/// Initialize kernel subsystems during boot (Multi-AI boot framework)
pub fn init_subsystems() -> Result<(), &'static str> {
    // Initialize vDSO manager for AI inference interface
    vdso_manager::init_vdso_manager()?;
    
    // Initialize memory management subsystems
    memory::init_memory_subsystems()?;
    
    // Initialize synchronization primitives
    sync::init_sync_primitives()?;
    
    // Initialize enhanced AI capability system with BFT consensus
    ai_capability_bft::init_ai_capability_system()?;
    
    // Initialize AI memory safety with linear types and DMA bounds checking
    ai_memory_safety::init_ai_memory_safety()?;
    ai_memory_safety::init_ai_memory_allocator()?;
    
    // Initialize DMA isolation system with IOMMU and real-time bounds checking
    ai_dma_isolation::init_ai_dma_isolation()?;
    ai_dma_isolation::init_dma_isolation_manager()?;
    
    // Initialize network-transparent cognitive fabric with RDMA support
    distributed_cognitive::init_distributed_cognitive()?;
    
    // Initialize cross-device AI migration with checkpoint-restart system
    ai_migration::init_ai_migration()?;
    
    // Initialize AI Byzantine Fault Tolerance with HotStuff consensus protocol
    ai_bft::init_ai_bft()?;
    
    // Initialize comprehensive kernel testing framework with metamorphic AI validation
    kernel_testing::init_kernel_testing()?;
    
    // Initialize performance validation system for <40μs inference and <500ns context switch targets
    performance_validation::init_performance_validation()?;
    
    // Phase 3: Initialize AI/ML Runtime subsystems
    ai_runtime::init()?;
    ai_scheduler::init()?;
    
    Ok(())
}

/// Initialize memory subsystems for boot
pub mod boot_memory {
    use super::*;
    
    pub fn init_boot_memory() -> Result<(), &'static str> {
        // Initialize early memory allocation
        // This will integrate with existing memory management
        memory::init_early_memory()?;
        Ok(())
    }
}
