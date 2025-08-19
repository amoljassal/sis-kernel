//! Hardware Abstraction Layer (HAL) for SIS Kernel
//!
//! Based on Gemini's architecture guidance: This HAL provides a clean
//! separation between platform-agnostic kernel logic and platform-specific
//! implementations.

use core::sync::atomic::Ordering;

/// HAL capabilities that may not be available on all platforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalCapability {
    /// Neural Processing Unit (Apple Neural Engine on M1)
    NeuralEngine,
    /// GPU Compute (Metal on M1, CUDA on NVIDIA)
    GpuCompute,
    /// SIMD Extensions (NEON on ARM64, AVX on x86)
    SimdExtensions,
    /// Hardware Random Number Generator
    HardwareRng,
    /// Virtualization Extensions
    Virtualization,
}

/// Core HAL trait that all architectures must implement
pub trait Hal {
    /// Initialize the architecture
    fn init() -> Result<(), &'static str>;
    
    /// CPU idle (power-efficient wait)
    fn idle();
    
    /// Send Inter-Processor Interrupt
    fn send_ipi(cpu_id: u32, vector: u8);
    
    /// Enable interrupts globally
    fn enable_interrupts();
    
    /// Disable interrupts globally
    fn disable_interrupts();
    
    /// Check if a capability is available
    fn has_capability(cap: HalCapability) -> bool;
    
    /// Get number of CPU cores
    fn cpu_count() -> u32;
    
    /// Get current CPU ID
    fn current_cpu() -> u32;
    
    /// Memory barrier (full fence)
    fn memory_barrier();
    
    /// Timer initialization
    fn timer_init(frequency_hz: u64);
    
    /// Get timer tick count
    fn timer_ticks() -> u64;
}

/// Interrupt controller abstraction
pub trait InterruptController {
    /// Initialize interrupt controller
    fn init() -> Result<(), &'static str>;
    
    /// Enable specific interrupt
    fn enable_irq(irq: u32);
    
    /// Disable specific interrupt
    fn disable_irq(irq: u32);
    
    /// Send End-Of-Interrupt signal
    fn eoi(irq: u32);
    
    /// Set interrupt affinity to specific CPU
    fn set_affinity(irq: u32, cpu_mask: u64);
}

/// Memory management abstraction
pub trait MemoryManagement {
    /// Set up initial page tables
    fn init_paging() -> Result<(), &'static str>;
    
    /// Map a physical page to virtual address
    fn map_page(virt: u64, phys: u64, flags: PageFlags) -> Result<(), &'static str>;
    
    /// Unmap a virtual page
    fn unmap_page(virt: u64) -> Result<(), &'static str>;
    
    /// Flush TLB for specific address
    fn flush_tlb(virt: u64);
    
    /// Flush entire TLB
    fn flush_tlb_all();
}

/// Page flags for memory mapping
#[derive(Debug, Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub executable: bool,
    pub user_accessible: bool,
    pub cacheable: bool,
    pub write_through: bool,
}

impl Default for PageFlags {
    fn default() -> Self {
        Self {
            writable: true,
            executable: false,
            user_accessible: false,
            cacheable: true,
            write_through: false,
        }
    }
}

/// Syscall handling abstraction
pub trait SyscallHandler {
    /// Initialize syscall handling
    fn init() -> Result<(), &'static str>;
    
    /// Handle syscall (called from assembly)
    fn handle_syscall(num: u64, args: &[u64; 6]) -> i64;
}

/// AI acceleration abstraction (for M1 Neural Engine, etc.)
pub trait AiAccelerator {
    /// Check if AI accelerator is available
    fn available() -> bool;
    
    /// Submit AI inference job
    fn submit_inference(input: &[u8], output: &mut [u8]) -> Result<(), &'static str>;
    
    /// Get accelerator performance metrics
    fn get_metrics() -> AiMetrics;
}

/// AI accelerator metrics
#[derive(Debug, Default)]
pub struct AiMetrics {
    pub operations_per_second: u64,
    pub power_milliwatts: u32,
    pub temperature_celsius: u32,
    pub utilization_percent: u8,
}

/// Architecture detection helper
#[cfg(target_arch = "x86_64")]
pub const CURRENT_ARCH: &str = "x86_64";

#[cfg(target_arch = "aarch64")]
pub const CURRENT_ARCH: &str = "aarch64";

/// Get HAL implementation for current architecture
#[cfg(target_arch = "x86_64")]
pub fn get_hal() -> &'static dyn Hal {
    &crate::arch::arch_impl::X86_64_HAL
}

#[cfg(target_arch = "aarch64")]
pub fn get_hal() -> &'static dyn Hal {
    &crate::arch::arch_impl::AARCH64_HAL
}