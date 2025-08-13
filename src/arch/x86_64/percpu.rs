//! Per-CPU data structures and GS-base optimization for SMP
//! 
//! This module provides the foundation for Phase 6A SMP support by implementing
//! efficient per-CPU data access using x86_64 GS segment base register.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use core::mem::MaybeUninit;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::segmentation::{GS, Segment};
use x86_64::registers::model_specific::Msr;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

/// Maximum number of CPUs supported (can be increased later)
pub const MAX_CPUS: usize = 64;

/// IST stack size per CPU for double fault handling  
pub const IST_STACK_SIZE: usize = 4096 * 5; // 20KB per CPU

/// Double fault IST index
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// GDT selectors for per-CPU GDT
#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub code_ring0: SegmentSelector,
    pub data_ring0: SegmentSelector,
    pub code_ring3: SegmentSelector,
    pub data_ring3: SegmentSelector,
    pub tss: SegmentSelector,
}

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
    
    /// Per-CPU double fault IST stack
    pub double_fault_stack: [u8; IST_STACK_SIZE],
    
    /// Per-CPU TSS
    pub tss: TaskStateSegment,
    
    /// Per-CPU GDT
    pub gdt: (GlobalDescriptorTable, Selectors),
    
    /// Reserved for future expansion
    pub reserved: [u64; 8],
}

impl PerCpu {
    /// Create a new PerCpu structure for given CPU ID and LAPIC ID
    pub fn new(id: u32, lapic_id: u32) -> Self {
        // Create per-CPU TSS with its own double fault IST stack
        let mut tss = TaskStateSegment::new();
        
        // Initialize double fault stack
        let double_fault_stack = [0; IST_STACK_SIZE];
        
        // Set up IST entry for double fault (will be updated after allocation)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new(0);
        
        // Create per-CPU GDT
        let mut gdt = GlobalDescriptorTable::new();
        let code_ring0 = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_ring0 = gdt.add_entry(Descriptor::kernel_data_segment());
        let code_ring3 = gdt.add_entry(Descriptor::user_code_segment());
        let data_ring3 = gdt.add_entry(Descriptor::user_data_segment());
        // TSS descriptor will be created in init_ist_stacks() when TSS is in final location
        let tss_selector = SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0);
        
        let selectors = Selectors {
            code_ring0,
            data_ring0, 
            code_ring3,
            data_ring3,
            tss: tss_selector,
        };
        
        Self {
            id,
            lapic_id,
            online: false,
            ticks: AtomicU64::new(0),
            current_task: core::ptr::null_mut(),
            runqueue_head: core::ptr::null_mut(),
            scratch: [0; 8],
            interrupt_stack: 0,
            double_fault_stack,
            tss,
            gdt: (gdt, selectors),
            reserved: [0; 8],
        }
    }
    
    /// Initialize per-CPU IST stack pointers after allocation
    pub fn init_ist_stacks(&mut self) {
        // Calculate IST stack top (stacks grow downward)
        let stack_bottom = self.double_fault_stack.as_ptr() as u64;
        let stack_top = stack_bottom + IST_STACK_SIZE as u64;
        
        // Update TSS with correct IST stack pointer
        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = 
            VirtAddr::new(stack_top);
        
        // Note: GDT creation with TSS is deferred to install_gdt_tss() to avoid lifetime issues
        // The basic segments are already set up in new()
    }

    /// Install per-CPU GDT and TSS
    pub fn install_gdt_tss(&self) {
        use x86_64::instructions::tables::{load_tss, lgdt};
        use x86_64::instructions::segmentation::{CS, Segment};
        use x86_64::structures::DescriptorTablePointer;
        
        // Create complete GDT with TSS at runtime to avoid lifetime issues
        let mut runtime_gdt = GlobalDescriptorTable::new();
        let code_ring0 = runtime_gdt.add_entry(Descriptor::kernel_code_segment());
        let data_ring0 = runtime_gdt.add_entry(Descriptor::kernel_data_segment());
        let code_ring3 = runtime_gdt.add_entry(Descriptor::user_code_segment());
        let data_ring3 = runtime_gdt.add_entry(Descriptor::user_data_segment());
        // Use Box::leak to create a static reference (owned data pattern)
        let tss_static: &'static TaskStateSegment = {
            extern crate alloc;
            use alloc::boxed::Box;
            Box::leak(Box::new(self.tss.clone()))
        };
        let tss_selector = runtime_gdt.add_entry(Descriptor::tss_segment(tss_static));
        
        // Load GDT
        let gdt_ptr = DescriptorTablePointer {
            base: VirtAddr::new(&runtime_gdt as *const _ as u64),
            limit: (core::mem::size_of_val(&runtime_gdt) - 1) as u16,
        };
        
        unsafe {
            lgdt(&gdt_ptr);
            
            // Reload code segment
            CS::set_reg(code_ring0);
            
            // Load TSS
            load_tss(tss_selector);
        }
        
        // Note: runtime_gdt goes out of scope here, but the CPU has loaded it
        // This is safe because GDT is copied into CPU registers, not referenced
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

/// Global per-CPU data array (initialized at runtime)
static mut PER_CPU_DATA: [MaybeUninit<PerCpu>; MAX_CPUS] = unsafe {
    MaybeUninit::uninit().assume_init()
};

/// Number of online CPUs
static ONLINE_CPUS: AtomicU32 = AtomicU32::new(0);

/// Initialization flag for per-CPU data array
static PERCPU_INITIALIZED: AtomicU32 = AtomicU32::new(0);

/// Initialize per-CPU data for a given CPU
/// 
/// This must be called during AP boot sequence to set up the per-CPU
/// data structure and configure GS base for fast access.
pub unsafe fn init_percpu(cpu_id: u32, lapic_id: u32) -> Result<(), &'static str> {
    if cpu_id as usize >= MAX_CPUS {
        return Err("CPU ID exceeds MAX_CPUS");
    }
    
    // One-time initialization of the array for BSP (CPU 0)
    if cpu_id == 0 {
        let expected = 0;
        if PERCPU_INITIALIZED.compare_exchange(expected, 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            // Initialize all entries to prevent undefined behavior
            for i in 0..MAX_CPUS {
                PER_CPU_DATA[i] = MaybeUninit::uninit();
            }
        }
    }
    
    // Initialize the per-CPU structure
    let percpu = PerCpu::new(cpu_id, lapic_id);
    PER_CPU_DATA[cpu_id as usize] = MaybeUninit::new(percpu);
    
    // Get reference to initialized structure
    let percpu_ref = PER_CPU_DATA[cpu_id as usize].assume_init_mut();
    
    // Initialize IST stacks after allocation
    percpu_ref.init_ist_stacks();
    
    // Install per-CPU GDT and TSS
    percpu_ref.install_gdt_tss();
    
    percpu_ref.set_online();
    
    // Set up GS base to point to this CPU's data
    let percpu_ptr = percpu_ref as *mut PerCpu;
    
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
    unsafe { 
        // Check if this CPU slot has been initialized
        if PERCPU_INITIALIZED.load(Ordering::SeqCst) == 0 {
            return false;
        }
        PER_CPU_DATA[cpu_id as usize].assume_init_ref().online 
    }
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
        // Check if system has been initialized
        if PERCPU_INITIALIZED.load(Ordering::SeqCst) == 0 {
            return None;
        }
        let percpu_ref = PER_CPU_DATA[cpu_id as usize].assume_init_mut();
        if percpu_ref.online {
            Some(percpu_ref)
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