//! Memory-mapped I/O utilities for ARM64
//!
//! Provides type-safe MMIO access with proper memory barriers for
//! Neural Engine, GIC, and other ARM64 hardware components.

use core::ptr;

/// ARM64 memory barrier helpers
#[inline(always)]
pub unsafe fn dmb_ish() {
    core::arch::asm!("dmb ish", options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn dmb_ishst() {
    core::arch::asm!("dmb ishst", options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn dsb_st() {
    core::arch::asm!("dsb st", options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn dsb_sy() {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn isb() {
    core::arch::asm!("isb", options(nostack, preserves_flags));
}

#[inline(always)]
pub fn sev() {
    unsafe {
        core::arch::asm!("sev", options(nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn wfe() {
    unsafe {
        core::arch::asm!("wfe", options(nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn yield_now() {
    wfe(); // Efficient idle hint for both real hardware and QEMU TCG
}

/// Cache maintenance operations for DMA coherency
pub unsafe fn dcache_clean_range(start: usize, len: usize) {
    let cache_line_size = 64; // Standard ARM64 cache line
    let mut addr = start & !(cache_line_size - 1);
    let end = start + len;
    
    while addr < end {
        core::arch::asm!("dc cvau, {}", in(reg) addr, options(nostack, preserves_flags));
        addr += cache_line_size;
    }
    
    dsb_st();
}

pub unsafe fn dcache_invalidate_range(start: usize, len: usize) {
    let cache_line_size = 64;
    let mut addr = start & !(cache_line_size - 1);
    let end = start + len;
    
    while addr < end {
        core::arch::asm!("dc ivau, {}", in(reg) addr, options(nostack, preserves_flags));
        addr += cache_line_size;
    }
    
    dsb_sy();
    isb();
}

/// Type-safe MMIO register wrapper
pub struct MmioReg<T> {
    ptr: *mut T,
}

impl<T> MmioReg<T> {
    /// Create MMIO register from physical address
    /// 
    /// # Safety
    /// - `addr` must be a valid mapped MMIO address
    /// - Caller must ensure exclusive access during writes
    pub const unsafe fn new(addr: usize) -> Self {
        Self {
            ptr: addr as *mut T,
        }
    }

    /// Read register value with proper barriers
    pub fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe {
            // Ensure all prior operations complete before read
            dsb_sy();
            let value = ptr::read_volatile(self.ptr);
            // Memory barrier not needed after read for most ARM64 devices
            value
        }
    }

    /// Write register value with proper barriers
    pub fn write(&self, value: T)
    where
        T: Copy,
    {
        unsafe {
            ptr::write_volatile(self.ptr, value);
            // Ensure write completes before subsequent operations
            dsb_st();
        }
    }

    /// Modify register using read-modify-write with barriers
    pub fn modify<F>(&self, f: F)
    where
        T: Copy,
        F: FnOnce(T) -> T,
    {
        let current = self.read();
        let new_value = f(current);
        self.write(new_value);
    }
}

/// Neural Engine MMIO registers (Apple M1/M2)
#[repr(C)]
pub struct NeuralEngineRegs {
    pub control: MmioReg<u32>,
    pub status: MmioReg<u32>,
    pub queue_base_lo: MmioReg<u32>,
    pub queue_base_hi: MmioReg<u32>,
    pub queue_length: MmioReg<u32>,
    pub doorbell: MmioReg<u32>,
    _reserved: [u32; 58], // Pad to 256 bytes
}

impl NeuralEngineRegs {
    /// Initialize Neural Engine registers
    /// 
    /// # Safety
    /// Must be called with valid Neural Engine MMIO base address
    pub unsafe fn new(base_addr: usize) -> &'static mut Self {
        &mut *(base_addr as *mut Self)
    }

    /// Start Neural Engine operation
    pub fn start_operation(&mut self, queue_addr: u64, length: u32) {
        // Configure queue
        self.queue_base_lo.write(queue_addr as u32);
        self.queue_base_hi.write((queue_addr >> 32) as u32);
        self.queue_length.write(length);
        
        // Ensure queue setup completes before starting
        unsafe { dsb_sy(); }
        
        // Start operation
        self.control.write(1); // Enable bit
        
        // Ring doorbell to notify hardware
        self.doorbell.write(1);
    }

    /// Check if operation completed
    pub fn is_complete(&self) -> bool {
        const DONE_BIT: u32 = 1 << 0;
        (self.status.read() & DONE_BIT) != 0
    }

    /// Wait for operation completion with timeout
    pub fn wait_complete(&self, timeout_us: u64) -> bool {
        let start_time = unsafe { read_cycle_counter() };
        let timeout_cycles = timeout_us * get_cpu_frequency_mhz() as u64;
        
        loop {
            if self.is_complete() {
                return true;
            }
            
            let elapsed = unsafe { read_cycle_counter() } - start_time;
            if elapsed > timeout_cycles {
                return false; // Timeout
            }
            
            yield_now(); // Efficient wait
        }
    }
}

/// Read ARM64 cycle counter for timing
unsafe fn read_cycle_counter() -> u64 {
    let mut count: u64;
    core::arch::asm!("mrs {}, cntvct_el0", out(reg) count);
    count
}

/// Get CPU frequency in MHz (simplified)
fn get_cpu_frequency_mhz() -> u32 {
    // On Apple Silicon, typically 24MHz counter
    // In real implementation, would read from device tree or system registers
    24
}