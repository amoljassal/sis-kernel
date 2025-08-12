//! Per-CPU data structures and GS-base optimization for SMP
//! 
//! This module provides the foundation for Phase 6A SMP support by implementing
//! efficient per-CPU data access using x86_64 GS segment base register.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::segmentation::{GS, Segment};
use x86_64::registers::model_specific::Msr;

/// Maximum number of CPUs supported (can be increased later)
pub const MAX_CPUS: usize = 64;

/// Per-CPU data structure containing all CPU-local state
#[repr(C, align(64))] // Cache line aligned to prevent false sharing
pub struct PerCpu {
    /// CPU ID (0-based logical CPU number)
    pub id: u32,
    
    /// LAPIC ID (hardware identifier)  
    pub lapic_id: u32,
    
    /// CPU online status
    pub online: bool,
    
    /// Per-CPU tick counter for LAPIC timer
    pub ticks: AtomicU64,
    
    /// Current running task pointer (for future scheduler integration)
    pub current_task: *mut u8, // Will be proper Task pointer later
    
    /// Per-CPU runqueue head (for future scheduler)
    pub runqueue_head: *mut u8, // Will be proper runqueue later
    
    /// Scratch space for interrupt handlers
    pub scratch: [u64; 8],
    
    /// Per-CPU stack pointer for interrupt handling
    pub interrupt_stack: u64,
    
    /// Reserved for future expansion
    pub reserved: [u64; 16],
}

impl PerCpu {
    /// Create a new PerCpu structure for given CPU ID and LAPIC ID
    pub const fn new(id: u32, lapic_id: u32) -> Self {
        Self {
            id,
            lapic_id,
            online: false,
            ticks: AtomicU64::new(0),
            current_task: core::ptr::null_mut(),
            runqueue_head: core::ptr::null_mut(),
            scratch: [0; 8],
            interrupt_stack: 0,
            reserved: [0; 16],
        }
    }
    
    /// Mark this CPU as online
    pub fn set_online(&mut self) {
        self.online = true;
    }
    
    /// Increment tick counter (called from LAPIC timer interrupt)
    pub fn increment_ticks(&self) -> u64 {
        self.ticks.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Get current tick count
    pub fn get_ticks(&self) -> u64 {
        self.ticks.load(Ordering::SeqCst)
    }
}

/// Global per-CPU data array
static mut PER_CPU_DATA: [PerCpu; MAX_CPUS] = [
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
    PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0), PerCpu::new(0, 0),
];

/// Number of online CPUs
static ONLINE_CPUS: AtomicU32 = AtomicU32::new(0);

/// Initialize per-CPU data for a given CPU
/// 
/// This must be called during AP boot sequence to set up the per-CPU
/// data structure and configure GS base for fast access.
pub unsafe fn init_percpu(cpu_id: u32, lapic_id: u32) -> Result<(), &'static str> {
    if cpu_id as usize >= MAX_CPUS {
        return Err("CPU ID exceeds MAX_CPUS");
    }
    
    // Initialize the per-CPU structure
    PER_CPU_DATA[cpu_id as usize] = PerCpu::new(cpu_id, lapic_id);
    PER_CPU_DATA[cpu_id as usize].set_online();
    
    // Set up GS base to point to this CPU's data
    let percpu_ptr = &mut PER_CPU_DATA[cpu_id as usize] as *mut PerCpu;
    
    // Use GSBASE MSR to set GS segment base
    let mut gs_base_msr = Msr::new(0xC0000101); // IA32_GS_BASE
    gs_base_msr.write(percpu_ptr as u64);
    
    // Increment online CPU count
    ONLINE_CPUS.fetch_add(1, Ordering::SeqCst);
    
    crate::kernel::serial::write_str("[percpu] CPU ");
    crate::kernel::serial::write_u64(cpu_id as u64);
    crate::kernel::serial::write_str(" initialized (LAPIC=");
    crate::kernel::serial::write_u64(lapic_id as u64);
    crate::kernel::serial::write_str(")\n");
    
    Ok(())
}

/// Fast per-CPU data access using GS base
/// 
/// This provides extremely fast access to current CPU's data by reading
/// directly from GS segment base register.
#[inline(always)]  
pub fn get_cpu() -> &'static mut PerCpu {
    unsafe {
        let gs_base: u64;
        // Read the GS base MSR directly since we set it to point to PerCpu
        let mut gs_base_msr = Msr::new(0xC0000101);
        gs_base = gs_base_msr.read();
        &mut *(gs_base as *mut PerCpu)
    }
}

/// Get current CPU ID quickly
#[inline(always)]
pub fn cpu_id() -> u32 {
    get_cpu().id
}

/// Get current LAPIC ID quickly  
#[inline(always)]
pub fn lapic_id() -> u32 {
    get_cpu().lapic_id
}

/// Get number of online CPUs
pub fn online_cpu_count() -> u32 {
    ONLINE_CPUS.load(Ordering::SeqCst)
}

/// Check if a CPU is online
pub fn is_cpu_online(cpu_id: u32) -> bool {
    if cpu_id as usize >= MAX_CPUS {
        return false;
    }
    unsafe { PER_CPU_DATA[cpu_id as usize].online }
}

/// Per-CPU tick increment (called from LAPIC timer interrupt)
pub fn percpu_tick() {
    get_cpu().increment_ticks();
}

/// Get per-CPU data for a specific CPU (for cross-CPU access)
pub fn get_percpu(cpu_id: u32) -> Option<&'static mut PerCpu> {
    if cpu_id as usize >= MAX_CPUS {
        return None;
    }
    unsafe {
        if PER_CPU_DATA[cpu_id as usize].online {
            Some(&mut PER_CPU_DATA[cpu_id as usize])
        } else {
            None
        }
    }
}

/// Initialize BSP (Boot Strap Processor) per-CPU data
pub fn init_bsp_percpu() -> Result<(), &'static str> {
    let lapic_id = crate::arch::x86_64::apic::lapic_id();
    unsafe { init_percpu(0, lapic_id) }
}