//! Per-CPU data structures for ARM64 SMP support
//!
//! This module provides efficient per-CPU data access using ARM64 TPIDR_EL1 register,
//! following the geometric architecture principles of the SIS kernel.
//!
//! Geometric Principle: Each CPU represents a distinct computational space with
//! its own local state, forming a HYPERCUBE of parallel execution contexts.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::mem::MaybeUninit;
use alloc::vec::Vec;
use crate::kernel::sync::SpinLock;

/// Maximum number of CPUs supported (expandable for HYPERCUBE scaling)
pub const MAX_CPUS: usize = 256;

/// Per-CPU data structure containing all CPU-local state
/// Represents a single vertex in the HYPERCUBE of computational spaces
#[repr(C, align(64))] // Cache line aligned to prevent false sharing
pub struct PerCpu {
    /// CPU ID (0-based logical CPU number)
    pub cpu_id: u32,
    
    /// MPIDR affinity value (hardware identifier)
    pub mpidr: u64,
    
    /// CPU online status
    pub online: AtomicBool,
    
    /// Generic timer tick counter
    pub ticks: AtomicU64,
    
    /// Current running task (for cognitive scheduling)
    pub current_task: AtomicU64,
    
    /// Per-CPU cognitive workload queue
    pub cognitive_queue: AtomicU64,
    
    /// Scratch space for exception handlers
    pub scratch: [u64; 16],
    
    /// Exception stack pointer
    pub exception_stack: u64,
    
    /// Interrupt stack pointer  
    pub interrupt_stack: u64,
    
    /// GICv3 redistributor base address
    pub gicr_base: u64,
    
    /// Performance monitoring counters
    pub perf_counters: PerfCounters,
    
    /// IPI pending mask
    pub ipi_pending: AtomicU32,
    
    /// CPU capabilities
    pub capabilities: CpuCapabilities,
    
    /// AI workload statistics
    pub ai_stats: AiWorkloadStats,
}

/// Performance monitoring counters for AI workload optimization
#[derive(Debug, Default)]
pub struct PerfCounters {
    /// CPU cycle counter
    pub cycles: AtomicU64,
    /// Instructions executed
    pub instructions: AtomicU64,
    /// L1 data cache misses
    pub l1d_misses: AtomicU64,
    /// L1 instruction cache misses
    pub l1i_misses: AtomicU64,
    /// Branch mispredictions
    pub branch_misses: AtomicU64,
    /// AI inference operations
    pub ai_ops: AtomicU64,
}

/// CPU capabilities detected at runtime
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuCapabilities {
    /// NEON/ASIMD support
    pub has_neon: bool,
    /// Crypto extensions
    pub has_crypto: bool,
    /// FP16 support
    pub has_fp16: bool,
    /// SVE support
    pub has_sve: bool,
    /// Pointer authentication
    pub has_pauth: bool,
    /// Performance (1) or Efficiency (0) core
    pub core_type: u8,
}

/// AI workload statistics for cognitive scheduling
#[derive(Debug, Default)]
pub struct AiWorkloadStats {
    /// Total AI tasks executed
    pub tasks_executed: AtomicU64,
    /// Total inference latency in microseconds
    pub total_latency_us: AtomicU64,
    /// Tasks meeting <40μs target
    pub sub_40us_tasks: AtomicU64,
    /// Neural engine utilization percentage
    pub neural_utilization: AtomicU32,
}

/// Static per-CPU data array (HYPERCUBE vertices)
static mut PERCPU_DATA: MaybeUninit<[PerCpu; MAX_CPUS]> = MaybeUninit::uninit();

/// Number of initialized CPUs
static CPU_COUNT: AtomicU32 = AtomicU32::new(0);

/// Per-CPU initialization lock
static PERCPU_INIT_LOCK: SpinLock<()> = SpinLock::new(());

impl PerCpu {
    /// Create a new per-CPU structure
    pub fn new(cpu_id: u32, mpidr: u64) -> Self {
        Self {
            cpu_id,
            mpidr,
            online: AtomicBool::new(false),
            ticks: AtomicU64::new(0),
            current_task: AtomicU64::new(0),
            cognitive_queue: AtomicU64::new(0),
            scratch: [0; 16],
            exception_stack: 0,
            interrupt_stack: 0,
            gicr_base: 0,
            perf_counters: PerfCounters::default(),
            ipi_pending: AtomicU32::new(0),
            capabilities: CpuCapabilities::detect(),
            ai_stats: AiWorkloadStats::default(),
        }
    }
    
    /// Get the current CPU's PerCpu structure
    #[inline(always)]
    pub fn current() -> &'static Self {
        unsafe {
            let ptr: u64;
            core::arch::asm!(
                "mrs {}, tpidr_el1",
                out(reg) ptr,
                options(nomem, nostack)
            );
            &*(ptr as *const Self)
        }
    }
    
    /// Get mutable reference to current CPU's PerCpu
    #[inline(always)]
    pub fn current_mut() -> &'static mut Self {
        unsafe {
            let ptr: u64;
            core::arch::asm!(
                "mrs {}, tpidr_el1",
                out(reg) ptr,
                options(nomem, nostack)
            );
            &mut *(ptr as *mut Self)
        }
    }
    
    /// Set the current CPU's PerCpu pointer
    #[inline(always)]
    pub unsafe fn set_current(percpu: &Self) {
        let ptr = percpu as *const Self as u64;
        core::arch::asm!(
            "msr tpidr_el1, {}",
            in(reg) ptr,
            options(nomem, nostack)
        );
    }
    
    /// Get PerCpu for a specific CPU
    pub fn for_cpu(cpu_id: u32) -> Option<&'static Self> {
        if cpu_id >= MAX_CPUS as u32 {
            return None;
        }
        unsafe {
            let percpu_array = PERCPU_DATA.assume_init_ref();
            if cpu_id < CPU_COUNT.load(Ordering::Acquire) {
                Some(&percpu_array[cpu_id as usize])
            } else {
                None
            }
        }
    }
    
    /// Update AI workload statistics
    pub fn record_ai_task(&self, latency_us: u64) {
        self.ai_stats.tasks_executed.fetch_add(1, Ordering::Relaxed);
        self.ai_stats.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        
        if latency_us < 40 {
            self.ai_stats.sub_40us_tasks.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl CpuCapabilities {
    /// Detect CPU capabilities from system registers
    pub fn detect() -> Self {
        let mut caps = Self::default();
        
        unsafe {
            // Read ID_AA64ISAR0_EL1 for instruction set attributes
            let isar0: u64;
            core::arch::asm!(
                "mrs {}, id_aa64isar0_el1",
                out(reg) isar0,
                options(nomem, nostack)
            );
            
            // Check for crypto extensions
            caps.has_crypto = ((isar0 >> 4) & 0xF) >= 1;
            
            // Read ID_AA64PFR0_EL1 for processor features
            let pfr0: u64;
            core::arch::asm!(
                "mrs {}, id_aa64pfr0_el1",
                out(reg) pfr0,
                options(nomem, nostack)
            );
            
            // NEON is mandatory in ARMv8-A
            caps.has_neon = true;
            
            // FP16 support
            caps.has_fp16 = ((pfr0 >> 16) & 0xF) >= 1;
            
            // SVE support
            caps.has_sve = ((pfr0 >> 32) & 0xF) >= 1;
            
            // Read ID_AA64ISAR1_EL1 for additional features
            let isar1: u64;
            core::arch::asm!(
                "mrs {}, id_aa64isar1_el1",
                out(reg) isar1,
                options(nomem, nostack)
            );
            
            // Pointer authentication
            caps.has_pauth = ((isar1 >> 4) & 0xF) >= 1;
            
            // Detect core type from MIDR_EL1
            let midr: u64;
            core::arch::asm!(
                "mrs {}, midr_el1",
                out(reg) midr,
                options(nomem, nostack)
            );
            
            // Simple heuristic for performance vs efficiency cores
            let partnum = (midr >> 4) & 0xFFF;
            caps.core_type = if partnum & 0x100 != 0 { 1 } else { 0 };
        }
        
        caps
    }
}

/// Initialize per-CPU data for the boot CPU
pub fn init_boot_cpu() -> Result<(), &'static str> {
    let _lock = PERCPU_INIT_LOCK.lock();
    
    unsafe {
        // Get MPIDR for boot CPU
        let mpidr: u64;
        core::arch::asm!(
            "mrs {}, mpidr_el1",
            out(reg) mpidr,
            options(nomem, nostack)
        );
        
        // Initialize first per-CPU structure
        let percpu_array = PERCPU_DATA.as_mut_ptr() as *mut PerCpu;
        let boot_percpu = &mut *percpu_array;
        *boot_percpu = PerCpu::new(0, mpidr);
        
        // Allocate stacks
        const STACK_SIZE: usize = 16 * 1024; // 16KB
        
        let exception_stack = alloc::vec![0u8; STACK_SIZE];
        boot_percpu.exception_stack = exception_stack.as_ptr() as u64 + STACK_SIZE as u64;
        core::mem::forget(exception_stack);
        
        let interrupt_stack = alloc::vec![0u8; STACK_SIZE];
        boot_percpu.interrupt_stack = interrupt_stack.as_ptr() as u64 + STACK_SIZE as u64;
        core::mem::forget(interrupt_stack);
        
        // Mark boot CPU as online
        boot_percpu.online.store(true, Ordering::Release);
        
        // Set TPIDR_EL1
        PerCpu::set_current(boot_percpu);
        
        // Update CPU count
        CPU_COUNT.store(1, Ordering::Release);
        
        // Log capabilities
        let caps = &boot_percpu.capabilities;
        crate::kernel::serial::write_str("[PERCPU] Boot CPU capabilities: ");
        if caps.has_neon { crate::kernel::serial::write_str("NEON "); }
        if caps.has_crypto { crate::kernel::serial::write_str("CRYPTO "); }
        if caps.has_fp16 { crate::kernel::serial::write_str("FP16 "); }
        if caps.has_sve { crate::kernel::serial::write_str("SVE "); }
        crate::kernel::serial::write_str("\n");
    }
    
    Ok(())
}

/// Initialize per-CPU data for a secondary CPU
pub fn init_secondary_cpu(cpu_id: u32) -> Result<(), &'static str> {
    if cpu_id >= MAX_CPUS as u32 {
        return Err("CPU ID exceeds maximum");
    }
    
    let _lock = PERCPU_INIT_LOCK.lock();
    
    unsafe {
        // Get MPIDR for this CPU
        let mpidr: u64;
        core::arch::asm!(
            "mrs {}, mpidr_el1",
            out(reg) mpidr,
            options(nomem, nostack)
        );
        
        // Initialize per-CPU structure
        let percpu_array = PERCPU_DATA.as_mut_ptr() as *mut PerCpu;
        let percpu = &mut *percpu_array.add(cpu_id as usize);
        *percpu = PerCpu::new(cpu_id, mpidr);
        
        // Allocate stacks
        const STACK_SIZE: usize = 16 * 1024;
        
        let exception_stack = alloc::vec![0u8; STACK_SIZE];
        percpu.exception_stack = exception_stack.as_ptr() as u64 + STACK_SIZE as u64;
        core::mem::forget(exception_stack);
        
        let interrupt_stack = alloc::vec![0u8; STACK_SIZE];
        percpu.interrupt_stack = interrupt_stack.as_ptr() as u64 + STACK_SIZE as u64;
        core::mem::forget(interrupt_stack);
        
        // Set TPIDR_EL1
        PerCpu::set_current(percpu);
        
        // Mark CPU as online
        percpu.online.store(true, Ordering::Release);
        
        // Update CPU count
        let current = CPU_COUNT.load(Ordering::Acquire);
        if cpu_id >= current {
            CPU_COUNT.store(cpu_id + 1, Ordering::Release);
        }
    }
    
    Ok(())
}

/// Get current CPU ID
#[inline(always)]
pub fn current_cpu_id() -> u32 {
    PerCpu::current().cpu_id
}

/// Get total CPU count
#[inline(always)]
pub fn cpu_count() -> u32 {
    CPU_COUNT.load(Ordering::Acquire)
}

/// Check if CPU is online
pub fn is_cpu_online(cpu_id: u32) -> bool {
    PerCpu::for_cpu(cpu_id)
        .map(|percpu| percpu.online.load(Ordering::Acquire))
        .unwrap_or(false)
}