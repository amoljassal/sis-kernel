//! ARM64 Memory Barriers and MMIO Safety
//!
//! Implements proper ARM64 memory ordering and barriers for Neural Engine
//! hardware access. Based on ChatGPT's recommendations for Device-nGnRnE
//! mapping and correct barrier usage.

use core::ptr::{read_volatile, write_volatile};
use core::arch::asm;

/// ARM64 Memory Barrier Types
pub struct MemoryBarriers;

impl MemoryBarriers {
    /// Data Memory Barrier - Inner Shareable (DMB ISH)
    /// Used for ordering memory accesses within the same inner shareable domain
    #[inline(always)]
    pub fn dmb_ish() {
        unsafe {
            asm!("dmb ish", options(nostack, preserves_flags));
        }
    }

    /// Data Memory Barrier - Inner Shareable Store (DMB ISHST)
    /// Used for ordering store operations before MMIO writes
    #[inline(always)]
    pub fn dmb_ishst() {
        unsafe {
            asm!("dmb ishst", options(nostack, preserves_flags));
        }
    }

    /// Data Synchronization Barrier - System (DSB SY)
    /// Strongest barrier - ensures all memory operations complete
    #[inline(always)]
    pub fn dsb_sy() {
        unsafe {
            asm!("dsb sy", options(nostack, preserves_flags));
        }
    }

    /// Data Synchronization Barrier - Store (DSB ST)
    /// Ensures all store operations complete before proceeding
    #[inline(always)]
    pub fn dsb_st() {
        unsafe {
            asm!("dsb st", options(nostack, preserves_flags));
        }
    }

    /// Instruction Synchronization Barrier (ISB)
    /// Flushes instruction pipeline
    #[inline(always)]
    pub fn isb() {
        unsafe {
            asm!("isb", options(nostack, preserves_flags));
        }
    }

    /// Wait for Event (WFE)
    /// Power-efficient wait for hardware events
    #[inline(always)]
    pub fn wfe() {
        unsafe {
            asm!("wfe", options(nostack, preserves_flags));
        }
    }

    /// Yield hint
    /// Suggests scheduler should consider other threads
    #[inline(always)]
    pub fn yield_hint() {
        unsafe {
            asm!("yield", options(nostack, preserves_flags));
        }
    }
}

/// Safe MMIO register accessor with proper barriers and typing
pub struct MmioRegion {
    base_addr: u64,
    size: usize,
}

impl MmioRegion {
    /// Create new MMIO region
    /// 
    /// # Safety
    /// - base_addr must point to valid MMIO region mapped with Device-nGnRnE attributes
    /// - size must not exceed the actual MMIO region size
    pub unsafe fn new(base_addr: u64, size: usize) -> Self {
        Self { base_addr, size }
    }

    /// Read 32-bit register with proper barriers
    #[inline]
    pub fn read_u32(&self, offset: usize) -> u32 {
        assert!(offset + 4 <= self.size, "MMIO read out of bounds");
        
        // Memory barrier before reading device register
        MemoryBarriers::dmb_ish();
        
        let value = unsafe {
            read_volatile((self.base_addr + offset as u64) as *const u32)
        };
        
        // No barrier after read typically needed for Device-nGnRnE
        value
    }

    /// Write 32-bit register with proper barriers
    #[inline]
    pub fn write_u32(&self, offset: usize, value: u32) {
        assert!(offset + 4 <= self.size, "MMIO write out of bounds");
        
        // Ensure all previous stores complete before MMIO write
        MemoryBarriers::dmb_ishst();
        
        unsafe {
            write_volatile((self.base_addr + offset as u64) as *mut u32, value);
        }
        
        // No barrier after write needed for Device-nGnRnE (enforced by hardware)
    }

    /// Read 64-bit register with proper barriers
    #[inline]
    pub fn read_u64(&self, offset: usize) -> u64 {
        assert!(offset + 8 <= self.size, "MMIO read out of bounds");
        
        MemoryBarriers::dmb_ish();
        
        let value = unsafe {
            read_volatile((self.base_addr + offset as u64) as *const u64)
        };
        
        value
    }

    /// Write 64-bit register with proper barriers
    #[inline]
    pub fn write_u64(&self, offset: usize, value: u64) {
        assert!(offset + 8 <= self.size, "MMIO write out of bounds");
        
        MemoryBarriers::dmb_ishst();
        
        unsafe {
            write_volatile((self.base_addr + offset as u64) as *mut u64, value);
        }
    }

    /// Ring doorbell with optimal barrier sequence
    /// This is the critical path for Neural Engine command submission
    #[inline]
    pub fn ring_doorbell(&self, doorbell_offset: usize) {
        assert!(doorbell_offset + 4 <= self.size, "Doorbell offset out of bounds");
        
        // Ensure all command descriptor writes are visible before doorbell
        MemoryBarriers::dsb_st();
        
        unsafe {
            write_volatile((self.base_addr + doorbell_offset as u64) as *mut u32, 1);
        }
        
        // No additional barrier needed - Device-nGnRnE ensures ordering
    }

    /// Poll status register with efficient barriers
    #[inline]
    pub fn poll_status(&self, status_offset: usize, mask: u32, expected: u32, max_iterations: u32) -> bool {
        assert!(status_offset + 4 <= self.size, "Status offset out of bounds");
        
        for iteration in 0..max_iterations {
            // Read with barrier
            MemoryBarriers::dmb_ish();
            let status = unsafe {
                read_volatile((self.base_addr + status_offset as u64) as *const u32)
            };
            
            if (status & mask) == expected {
                return true;
            }
            
            // Efficient waiting with increasing backoff
            if iteration < 10 {
                // Tight loop for very short waits
                for _ in 0..4 {
                    unsafe { asm!("nop", options(nomem, nostack, preserves_flags)); }
                }
            } else if iteration < 100 {
                // Yield to other threads
                MemoryBarriers::yield_hint();
            } else {
                // Longer waits use WFE for power efficiency
                MemoryBarriers::wfe();
            }
        }
        
        false
    }

    /// Batch read multiple registers efficiently
    pub fn batch_read_u32(&self, operations: &[(usize, &mut u32)]) {
        // Single barrier for all reads
        MemoryBarriers::dmb_ish();
        
        for &(offset, ref mut dest) in operations {
            assert!(offset + 4 <= self.size, "Batch read offset out of bounds");
            **dest = unsafe {
                read_volatile((self.base_addr + offset as u64) as *const u32)
            };
        }
    }

    /// Batch write multiple registers efficiently
    pub fn batch_write_u32(&self, operations: &[(usize, u32)]) {
        // Single barrier before all writes
        MemoryBarriers::dmb_ishst();
        
        for &(offset, value) in operations {
            assert!(offset + 4 <= self.size, "Batch write offset out of bounds");
            unsafe {
                write_volatile((self.base_addr + offset as u64) as *mut u32, value);
            }
        }
    }
}

/// Typed register accessors with compile-time safety
pub trait TypedRegister {
    type Value: Copy;
    const OFFSET: usize;
    const NAME: &'static str;
    
    fn read(mmio: &MmioRegion) -> Self::Value;
    fn write(mmio: &MmioRegion, value: Self::Value);
}

/// Neural Engine Control Register
pub struct NEControlReg;
impl TypedRegister for NEControlReg {
    type Value = u32;
    const OFFSET: usize = 0x0000;
    const NAME: &'static str = "NE_CONTROL";
    
    fn read(mmio: &MmioRegion) -> u32 {
        mmio.read_u32(Self::OFFSET)
    }
    
    fn write(mmio: &MmioRegion, value: u32) {
        mmio.write_u32(Self::OFFSET, value);
    }
}

/// Neural Engine Status Register
pub struct NEStatusReg;
impl TypedRegister for NEStatusReg {
    type Value = u32;
    const OFFSET: usize = 0x0004;
    const NAME: &'static str = "NE_STATUS";
    
    fn read(mmio: &MmioRegion) -> u32 {
        mmio.read_u32(Self::OFFSET)
    }
    
    fn write(mmio: &MmioRegion, value: u32) {
        mmio.write_u32(Self::OFFSET, value);
    }
}

/// Neural Engine Doorbell Register
pub struct NEDoorbellReg;
impl TypedRegister for NEDoorbellReg {
    type Value = u32;
    const OFFSET: usize = 0x1024;
    const NAME: &'static str = "NE_DOORBELL";
    
    fn read(mmio: &MmioRegion) -> u32 {
        mmio.read_u32(Self::OFFSET)
    }
    
    fn write(mmio: &MmioRegion, value: u32) {
        mmio.write_u32(Self::OFFSET, value);
    }
}

/// High-performance command submission with minimal barriers
pub struct FastCommandSubmitter {
    mmio: MmioRegion,
    doorbell_offset: usize,
}

impl FastCommandSubmitter {
    /// Create fast command submitter
    pub fn new(mmio: MmioRegion, doorbell_offset: usize) -> Self {
        Self { mmio, doorbell_offset }
    }

    /// Submit command with single optimized barrier sequence
    /// This is the <25μs critical path
    #[inline(always)]
    pub fn submit_command_fast(&self) {
        // Single barrier to publish all descriptor writes
        unsafe {
            asm!("dsb ishst", options(nostack, preserves_flags));
        }
        
        // Doorbell write
        unsafe {
            write_volatile((self.mmio.base_addr + self.doorbell_offset as u64) as *mut u32, 1);
        }
        
        // Device-nGnRnE mapping ensures ordering without additional barriers
    }

    /// Submit with completion polling (for synchronous operations)
    #[inline]
    pub fn submit_and_wait(&self, status_offset: usize, completion_mask: u32, timeout_iterations: u32) -> bool {
        self.submit_command_fast();
        self.mmio.poll_status(status_offset, completion_mask, completion_mask, timeout_iterations)
    }
}

/// DMA coherency management
pub struct DMACoherency;

impl DMACoherency {
    /// Ensure CPU writes are visible to DMA engine
    #[inline(always)]
    pub fn cpu_to_dma_barrier() {
        // Ensure CPU stores reach point of coherency
        MemoryBarriers::dsb_sy();
    }

    /// Ensure DMA writes are visible to CPU
    #[inline(always)]  
    pub fn dma_to_cpu_barrier() {
        // Invalidate CPU caches if needed (depends on mapping)
        MemoryBarriers::dmb_ish();
    }

    /// Full bidirectional DMA coherency barrier
    #[inline(always)]
    pub fn full_dma_barrier() {
        MemoryBarriers::dsb_sy();
    }
}

/// Cache management for non-coherent paths
pub struct CacheOps;

impl CacheOps {
    /// Clean data cache range (write back to memory)
    /// 
    /// # Safety
    /// - addr must be valid virtual address
    /// - size must not exceed actual memory region
    pub unsafe fn dc_cvau_range(addr: u64, size: usize) {
        let cache_line_size = 64; // ARM64 standard
        let start = addr & !(cache_line_size - 1); // Align to cache line
        let end = (addr + size as u64 + cache_line_size - 1) & !(cache_line_size - 1);
        
        let mut current = start;
        while current < end {
            asm!("dc cvau, {}", in(reg) current, options(nostack, preserves_flags));
            current += cache_line_size;
        }
        
        MemoryBarriers::dsb_sy();
    }

    /// Invalidate data cache range
    /// 
    /// # Safety  
    /// - addr must be valid virtual address
    /// - size must not exceed actual memory region
    /// - Use only when certain no dirty data will be lost
    pub unsafe fn dc_ivau_range(addr: u64, size: usize) {
        let cache_line_size = 64;
        let start = addr & !(cache_line_size - 1);
        let end = (addr + size as u64 + cache_line_size - 1) & !(cache_line_size - 1);
        
        let mut current = start;
        while current < end {
            asm!("dc ivau, {}", in(reg) current, options(nostack, preserves_flags));
            current += cache_line_size;
        }
        
        MemoryBarriers::dsb_sy();
    }

    /// Clean and invalidate data cache range
    pub unsafe fn dc_civac_range(addr: u64, size: usize) {
        let cache_line_size = 64;
        let start = addr & !(cache_line_size - 1);
        let end = (addr + size as u64 + cache_line_size - 1) & !(cache_line_size - 1);
        
        let mut current = start;
        while current < end {
            asm!("dc civac, {}", in(reg) current, options(nostack, preserves_flags));
            current += cache_line_size;
        }
        
        MemoryBarriers::dsb_sy();
    }
}

/// Memory mapping attributes for different region types
#[derive(Debug, Clone, Copy)]
pub enum MemoryAttribute {
    /// Device memory - non-Gathering, non-Reordering, non-Early Write Acknowledgement
    /// Used for MMIO registers
    DeviceNGnRnE,
    /// Device memory - non-Gathering, non-Reordering, Early Write Acknowledgement  
    DeviceNGnRE,
    /// Normal memory - Write-Back cacheable
    NormalWB,
    /// Normal memory - Write-Through cacheable
    NormalWT,
    /// Normal memory - Non-cacheable
    NormalNC,
}

impl MemoryAttribute {
    /// Get MAIR_EL1 encoding for this attribute
    pub fn mair_encoding(self) -> u8 {
        match self {
            MemoryAttribute::DeviceNGnRnE => 0x00,
            MemoryAttribute::DeviceNGnRE => 0x04,
            MemoryAttribute::NormalWB => 0xFF,
            MemoryAttribute::NormalWT => 0xBB,
            MemoryAttribute::NormalNC => 0x44,
        }
    }
    
    /// Check if this attribute requires explicit barriers
    pub fn needs_explicit_barriers(self) -> bool {
        match self {
            MemoryAttribute::DeviceNGnRnE | MemoryAttribute::DeviceNGnRE => false,
            _ => true,
        }
    }
}