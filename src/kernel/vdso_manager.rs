//! vDSO Manager - Kernel-side vDSO integration
//!
//! Based on Multi-AI consultation synthesis:
//! - ChatGPT: Safe memory management with type-safe page table manipulation
//! - Gemini: Robust lifecycle management and process integration
//! - Grok: ARM64 hardware optimizations for minimal overhead

use crate::kernel::memory::{PhysFrame, VirtPage, MemoryError, get_memory_manager, PageTable, PteFlags};
use crate::kernel::task::Task;
use crate::kernel::ai_syscalls::vdso::{VdsoHeader, LiveStatus, VDSO_MAGIC, VDSO_ABI_VERSION, VdsoFlags};
use crate::kernel::serial;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use core::mem::size_of;
use spin::Mutex;

/// vDSO memory regions in virtual address space
/// Positioned near top of user space to avoid conflicts
const VDSO_REGION_BASE: u64 = 0x7FFF_E000_0000;
const VDSO_CODE_VA: u64 = VDSO_REGION_BASE;
const VDSO_COMM_VA_START: u64 = VDSO_REGION_BASE + 0x1000_0000;

/// Page size (4KB on ARM64)
const PAGE_SIZE: usize = 4096;

/// vDSO error types
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum VdsoError {
    Success = 0,
    Uninit = -1,
    NoMem = -12,
    Map = -14,
    AlreadyInstalled = -17,
    InvalidTask = -22,
}

impl From<VdsoError> for i64 {
    fn from(err: VdsoError) -> i64 {
        err as i32 as i64
    }
}

/// Global vDSO code page (shared across all processes)
/// 
/// From ChatGPT: Single physical frame containing vDSO executable code
pub struct VdsoCode {
    /// Physical frame containing vDSO code
    frame: PhysFrame,
    /// Reference count for cleanup (atomic for thread safety)
    ref_count: AtomicU64,
}

impl VdsoCode {
    /// Create new vDSO code instance
    pub fn new(frame: PhysFrame) -> Self {
        Self {
            frame,
            ref_count: AtomicU64::new(0),
        }
    }
    
    /// Get physical frame
    pub fn frame(&self) -> PhysFrame {
        self.frame
    }
    
    /// Increment reference count
    pub fn add_ref(&self) -> u64 {
        self.ref_count.fetch_add(1, Ordering::Relaxed)
    }
    
    /// Decrement reference count
    pub fn remove_ref(&self) -> u64 {
        self.ref_count.fetch_sub(1, Ordering::Relaxed)
    }
    
    /// Get current reference count
    pub fn ref_count(&self) -> u64 {
        self.ref_count.load(Ordering::Relaxed)
    }
}

/// Per-process vDSO state
/// 
/// From Gemini: Process-specific vDSO mappings and communication
#[derive(Debug)]
pub struct TaskVdso {
    /// Code page virtual address in user space
    pub code_user_va: VirtPage,
    
    /// Communication page virtual address in user space
    pub comm_user_va: VirtPage,
    
    /// Communication page kernel virtual address for updates
    pub comm_kva: VirtPage,
    
    /// Physical frame for communication page
    pub comm_frame: PhysFrame,
    
    /// Process ID for tracking
    pub process_id: u64,
}

impl TaskVdso {
    /// Create new task vDSO state
    pub fn new(
        code_va: VirtPage,
        comm_va: VirtPage,
        comm_kva: VirtPage,
        comm_frame: PhysFrame,
        process_id: u64,
    ) -> Self {
        Self {
            code_user_va: code_va,
            comm_user_va: comm_va,
            comm_kva,
            comm_frame,
            process_id,
        }
    }
}

/// RAII guard for page table mappings
/// 
/// From ChatGPT: Ensures automatic cleanup on error paths
pub struct MapGuard<'a> {
    page_table: &'a mut PageTable,
    virt_page: VirtPage,
    mapped: bool,
}

impl<'a> MapGuard<'a> {
    /// Create new map guard
    pub fn new(page_table: &'a mut PageTable, virt_page: VirtPage) -> Self {
        Self {
            page_table,
            virt_page,
            mapped: true,
        }
    }
    
    /// Release guard without unmapping (transfer ownership)
    pub fn release(mut self) {
        self.mapped = false;
    }
    
    /// Commit the mapping (consume guard, prevent auto-unmap)
    /// 
    /// From Multi-AI synthesis: Zero-cost abstraction for successful mappings
    #[inline(always)]
    pub fn commit(mut self) {
        self.mapped = false;
        // Guard is consumed, Drop will not be called
    }
}

impl Drop for MapGuard<'_> {
    fn drop(&mut self) {
        if self.mapped {
            let _ = self.page_table.unmap_user(self.virt_page);
        }
    }
}

/// Global vDSO manager state
static VDSO_MANAGER: Mutex<Option<VdsoManager>> = Mutex::new(None);

/// vDSO Manager - Central coordination for kernel vDSO integration
/// 
/// From Gemini: Centralized management with robust lifecycle tracking
pub struct VdsoManager {
    /// Global shared code page
    vdso_code: VdsoCode,
    
    /// Active process tracking for live status updates
    active_processes: BTreeMap<u64, TaskVdso>,
    
    /// Next available communication page virtual address
    next_comm_va: AtomicU64,
    
    /// Statistics
    processes_created: AtomicU64,
    processes_destroyed: AtomicU64,
}

impl VdsoManager {
    /// Create new vDSO manager
    fn new(vdso_code: VdsoCode) -> Self {
        Self {
            vdso_code,
            active_processes: BTreeMap::new(),
            next_comm_va: AtomicU64::new(VDSO_COMM_VA_START),
            processes_created: AtomicU64::new(0),
            processes_destroyed: AtomicU64::new(0),
        }
    }
    
    /// Allocate next communication page virtual address
    fn alloc_comm_va(&self) -> VirtPage {
        let va = self.next_comm_va.fetch_add(PAGE_SIZE as u64, Ordering::Relaxed);
        VirtPage::new(va)
    }
    
    /// Register active process
    fn register_process(&mut self, task_vdso: TaskVdso) {
        let pid = task_vdso.process_id;
        self.active_processes.insert(pid, task_vdso);
        self.processes_created.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Unregister process
    fn unregister_process(&mut self, process_id: u64) -> Option<TaskVdso> {
        if let Some(task_vdso) = self.active_processes.remove(&process_id) {
            self.processes_destroyed.fetch_add(1, Ordering::Relaxed);
            Some(task_vdso)
        } else {
            None
        }
    }
    
    /// Update live status for all processes
    pub fn update_global_status(&mut self, update_fn: impl Fn(&mut LiveStatus)) {
        for task_vdso in self.active_processes.values() {
            // Safety: We own the comm page mapping and it's valid for process lifetime
            let live_status = unsafe {
                &mut *(task_vdso.comm_kva.as_ptr() as *mut LiveStatus)
            };
            update_fn(live_status);
        }
        
        // Memory barrier to ensure updates are visible (architecture-specific)
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("dmb ishst", options(nostack, nomem, preserves_flags));
            
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("mfence", options(nostack, nomem, preserves_flags));
        }
    }
}

/// Initialize vDSO manager during kernel boot
/// 
/// From Gemini: Boot-time initialization with embedded vDSO code
pub fn init() -> Result<(), VdsoError> {
    serial::write_str("[VDSO] Initializing vDSO manager\n");
    
    // Check if already initialized
    {
        let manager_guard = VDSO_MANAGER.lock();
        if manager_guard.is_some() {
            return Err(VdsoError::AlreadyInstalled);
        }
    }
    
    // Get memory manager
    let mm = get_memory_manager().map_err(|_| VdsoError::NoMem)?;
    
    // Allocate physical frame for vDSO code
    let code_frame = mm.alloc_frame().ok_or(VdsoError::NoMem)?;
    
    // Map frame temporarily into kernel space to copy vDSO code
    let temp_kva = mm.map_kernel_temp(code_frame).map_err(|_| VdsoError::Map)?;
    
    // Copy embedded vDSO code
    // TODO: In real implementation, this would copy from embedded ELF section
    unsafe {
        // Placeholder: Zero-fill for now (real implementation would copy actual vDSO code)
        core::ptr::write_bytes(temp_kva.as_mut_ptr(), 0, PAGE_SIZE);
        
        // Write vDSO magic signature at start
        let magic_ptr = temp_kva.as_mut_ptr() as *mut u32;
        *magic_ptr = 0x554F4456; // "VDOU" in little-endian
    }
    
    // Unmap temporary mapping
    mm.unmap_kernel_temp(temp_kva).map_err(|_| VdsoError::Map)?;
    
    // Create vDSO code instance
    let vdso_code = VdsoCode::new(code_frame);
    
    // Create and install manager
    let manager = VdsoManager::new(vdso_code);
    {
        let mut manager_guard = VDSO_MANAGER.lock();
        *manager_guard = Some(manager);
    }
    
    serial::write_str("[VDSO] vDSO manager initialized successfully\n");
    Ok(())
}

/// Install vDSO for a process during exec
/// 
/// From ChatGPT: Safe process-specific vDSO installation
pub fn install_for_task(
    task: &mut Task,
    pt: &mut PageTable,
) -> Result<(), VdsoError> {
    // Check if vDSO manager is initialized
    let manager_lock = VDSO_MANAGER.lock();
    let manager = manager_lock.as_ref().ok_or(VdsoError::Uninit)?;
    
    // Get process ID
    let process_id = task.id as u64;
    
    // Check if already installed
    if task.vdso.is_some() {
        return Err(VdsoError::AlreadyInstalled);
    }
    
    // Get memory manager
    let mm = get_memory_manager().map_err(|_| VdsoError::NoMem)?;
    
    // Allocate virtual addresses
    let code_va = VirtPage::new(VDSO_CODE_VA);
    let comm_va = manager.alloc_comm_va();
    
    // Allocate physical frame for communication page
    let comm_frame = mm.alloc_frame().ok_or(VdsoError::NoMem)?;
    
    // Map vDSO code page (shared, read-only + execute)
    let code_flags = PteFlags::new()
        .with_user(true)
        .with_readonly(true)
        .with_executable(true)
        .with_shared(true);
    
    // Map communication page (private, read-write, no execute)
    let comm_flags = PteFlags::new()
        .with_user(true)
        .with_readonly(false)
        .with_executable(false)
        .with_shared(false);
    
    // Multi-AI Hybrid Solution: RAII + commit pattern with automatic rollback
    // From ChatGPT: Exception-safe RAII pattern
    // From Grok: Zero-cost abstractions with early borrow release
    // From Gemini: Clean composition for future transaction patterns
    
    // Map code page with RAII guard
    let code_guard = pt.map_user(code_va, manager.vdso_code.frame(), code_flags)
        .map_err(|_| VdsoError::Map)?;
    
    // Map communication page with RAII guard
    let comm_guard = pt.map_user(comm_va, comm_frame, comm_flags)
        .map_err(|_| VdsoError::Map)?; // code_guard auto-unmaps on error
    
    // Success: commit both mappings (release borrows)
    code_guard.commit();
    comm_guard.commit();
    
    // Map communication page into kernel space for updates
    let comm_kva = mm.map_kernel(comm_frame).map_err(|_| VdsoError::Map)?;
    
    // Initialize vDSO header in communication page
    unsafe {
        let header = &mut *(comm_kva.as_ptr() as *mut VdsoHeader);
        *header = VdsoHeader {
            magic: VDSO_MAGIC,
            abi_version: VDSO_ABI_VERSION,
            flags: VdsoFlags(0), // No special flags for now
            counter_freq_hz: read_counter_frequency(),
            cache_line_size: 64,
            reserved: 0,
            ring_ptr: core::ptr::null(),
            region_table_ptr: core::ptr::null(),
            live_status_ptr: (comm_kva.as_ptr() as u64 + size_of::<VdsoHeader>() as u64) as *const LiveStatus,
            hw_caps_ptr: core::ptr::null(),
        };
        
        // Initialize live status
        let live_status = &mut *((comm_kva.as_ptr() as u64 + size_of::<VdsoHeader>() as u64) as *mut LiveStatus);
        *live_status = LiveStatus {
            npu_utilization: AtomicU32::new(0),
            gpu_utilization: AtomicU32::new(0),
            thermal_state: AtomicU32::new(0),
            power_mw: AtomicU32::new(1000), // 1W default
            fastpath_handle: AtomicU64::new(0),
            cluster_healthy: AtomicU32::new(1),
            raft_leader_id: AtomicU32::new(0),
            ops_completed: AtomicU64::new(0),
            ops_submitted: AtomicU64::new(0),
            avg_latency_ns: AtomicU32::new(0),
            cache_hit_rate: AtomicU32::new(95), // 95% default
        };
    }
    
    // Memory barrier to ensure initialization is visible (architecture-specific)
    unsafe {
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("dmb ishst", options(nostack, nomem, preserves_flags));
        
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mfence", options(nostack, nomem, preserves_flags));
    }
    
    // Create task vDSO state
    let task_vdso = TaskVdso::new(code_va, comm_va, comm_kva, comm_frame, process_id);
    
    // Guards have already been released - mappings are permanent for this process
    
    // Store in task
    task.vdso = Some(task_vdso);
    
    // Update reference count
    manager.vdso_code.add_ref();
    
    serial::write_str("[VDSO] Installed vDSO for process ");
    serial::write_dec(process_id);
    serial::write_str("\n");
    
    Ok(())
}

/// Cleanup vDSO for a process during exit
/// 
/// From Gemini: Robust cleanup with resource tracking
pub fn cleanup_for_task(
    task: &mut Task,
    pt: &mut PageTable,
) -> Result<(), VdsoError> {
    if let Some(task_vdso) = task.vdso.take() {
        let process_id = task_vdso.process_id;
        
        // Get memory manager
        let mm = get_memory_manager().map_err(|_| VdsoError::NoMem)?;
        
        // Unmap pages
        let _ = pt.unmap_user(task_vdso.code_user_va);
        let _ = pt.unmap_user(task_vdso.comm_user_va);
        let _ = mm.unmap_kernel(task_vdso.comm_kva);
        
        // Free communication page frame
        mm.free_frame(task_vdso.comm_frame);
        
        // Update manager state
        {
            let mut manager_guard = VDSO_MANAGER.lock();
            if let Some(manager) = manager_guard.as_mut() {
                manager.unregister_process(process_id);
                manager.vdso_code.remove_ref();
            }
        }
        
        serial::write_str("[VDSO] Cleaned up vDSO for process ");
        serial::write_dec(process_id);
        serial::write_str("\n");
    }
    
    Ok(())
}

/// Get vDSO statistics
pub fn get_stats() -> VdsoStats {
    let manager_guard = VDSO_MANAGER.lock();
    if let Some(manager) = manager_guard.as_ref() {
        VdsoStats {
            processes_created: manager.processes_created.load(Ordering::Relaxed),
            processes_destroyed: manager.processes_destroyed.load(Ordering::Relaxed),
            active_processes: manager.active_processes.len() as u64,
            code_ref_count: manager.vdso_code.ref_count(),
        }
    } else {
        VdsoStats::default()
    }
}

/// vDSO statistics
#[derive(Debug, Default)]
pub struct VdsoStats {
    pub processes_created: u64,
    pub processes_destroyed: u64,
    pub active_processes: u64,
    pub code_ref_count: u64,
}

/// Read counter frequency (architecture-specific)
/// 
/// From Grok: Optimized timer frequency detection
#[inline(always)]
fn read_counter_frequency() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let freq: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, cntfrq_el0",
                out(reg) freq,
                options(nostack, nomem, preserves_flags)
            );
        }
        freq
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        // TSC frequency detection for x86_64
        // For now, return a reasonable default (2.4 GHz)
        2_400_000_000
    }
}

/// Update live status for all processes (called by kernel subsystems)
/// 
/// From Gemini: Live status update mechanism
pub fn update_live_status<F>(update_fn: F)
where
    F: Fn(&mut LiveStatus),
{
    let mut manager_guard = VDSO_MANAGER.lock();
    if let Some(manager) = manager_guard.as_mut() {
        manager.update_global_status(update_fn);
    }
}

/// Update live status for specific process
pub fn update_process_status<F>(process_id: u64, update_fn: F) -> Result<(), VdsoError>
where
    F: Fn(&mut LiveStatus),
{
    let manager_guard = VDSO_MANAGER.lock();
    let manager = manager_guard.as_ref().ok_or(VdsoError::Uninit)?;
    
    if let Some(task_vdso) = manager.active_processes.get(&process_id) {
        // Safety: We own the comm page mapping
        let live_status = unsafe {
            &mut *(task_vdso.comm_kva.as_ptr() as *mut LiveStatus)
        };
        update_fn(live_status);
        
        // Memory barrier (architecture-specific)
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("dmb ishst", options(nostack, nomem, preserves_flags));
            
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("mfence", options(nostack, nomem, preserves_flags));
        }
        
        Ok(())
    } else {
        Err(VdsoError::InvalidTask)
    }
}

// Helper traits for vDSO implementation