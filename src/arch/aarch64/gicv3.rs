//! ARM GICv3 (Generic Interrupt Controller v3) support for multi-core
//!
//! This module implements GICv3 support with redistributor initialization
//! for multi-core SMP operation, enabling efficient interrupt routing
//! across CPU cores for AI workload distribution.
//!
//! GICv3 Architecture:
//! - Distributor: Global interrupt configuration
//! - Redistributors: Per-CPU interrupt management
//! - CPU interfaces: Direct CPU interrupt handling

use core::ptr::{read_volatile, write_volatile};
use crate::arch::aarch64::percpu::PerCpu;

/// GICv3 Distributor base address (QEMU virt)
pub const GICD_BASE: usize = 0x0800_0000;

/// GICv3 Redistributor base address (QEMU virt)
pub const GICR_BASE: usize = 0x080A_0000;

/// GICv3 Distributor registers
mod gicd {
    pub const CTLR: usize = 0x0000;      // Distributor Control
    pub const TYPER: usize = 0x0004;     // Interrupt Controller Type
    pub const IIDR: usize = 0x0008;      // Distributor Implementer ID
    pub const IGROUPR: usize = 0x0080;   // Interrupt Group Registers
    pub const ISENABLER: usize = 0x0100; // Interrupt Set-Enable
    pub const ICENABLER: usize = 0x0180; // Interrupt Clear-Enable
    pub const ISPENDR: usize = 0x0200;   // Interrupt Set-Pending
    pub const ICPENDR: usize = 0x0280;   // Interrupt Clear-Pending
    pub const ISACTIVER: usize = 0x0300; // Interrupt Set-Active
    pub const ICACTIVER: usize = 0x0380; // Interrupt Clear-Active
    pub const IPRIORITYR: usize = 0x0400; // Interrupt Priority
    pub const ITARGETSR: usize = 0x0800; // Interrupt Target (GICv2 compat)
    pub const ICFGR: usize = 0x0C00;     // Interrupt Configuration
    pub const IROUTER: usize = 0x6000;   // Interrupt Routing (GICv3)
}

/// GICv3 Redistributor registers
mod gicr {
    pub const CTLR: usize = 0x0000;      // Redistributor Control
    pub const IIDR: usize = 0x0004;      // Redistributor Implementer ID
    pub const TYPER: usize = 0x0008;     // Redistributor Type
    pub const WAKER: usize = 0x0014;     // Redistributor Wake
    
    // SGI/PPI configuration (offset 0x10000)
    pub const SGI_BASE: usize = 0x10000;
    pub const IGROUPR0: usize = 0x10080;   // Group for SGI/PPI
    pub const ISENABLER0: usize = 0x10100; // Enable for SGI/PPI
    pub const ICENABLER0: usize = 0x10180; // Disable for SGI/PPI
    pub const ISPENDR0: usize = 0x10200;   // Set-Pending for SGI/PPI
    pub const ICPENDR0: usize = 0x10280;   // Clear-Pending for SGI/PPI
    pub const IPRIORITYR: usize = 0x10400; // Priority for SGI/PPI
    pub const ICFGR0: usize = 0x10C00;     // Configuration for SGI/PPI
    pub const ICFGR1: usize = 0x10C04;     // Configuration for PPI
}

/// ICC system registers (CPU interface)
mod icc {
    // These are accessed via system register instructions
    pub const CTLR_EL1: &str = "icc_ctlr_el1";
    pub const PMR_EL1: &str = "icc_pmr_el1";
    pub const IAR1_EL1: &str = "icc_iar1_el1";
    pub const EOIR1_EL1: &str = "icc_eoir1_el1";
    pub const BPR1_EL1: &str = "icc_bpr1_el1";
    pub const SGI1R_EL1: &str = "s3_0_c12_c11_5";
    pub const SRE_EL1: &str = "icc_sre_el1";
}

/// Interrupt types
pub const SGI_START: u32 = 0;   // Software Generated Interrupts (0-15)
pub const PPI_START: u32 = 16;  // Private Peripheral Interrupts (16-31)
pub const SPI_START: u32 = 32;  // Shared Peripheral Interrupts (32+)

/// IPI (Inter-Processor Interrupt) definitions
pub mod ipi {
    pub const IPI_RESCHEDULE: u32 = 0;  // Reschedule request
    pub const IPI_CALL_FUNC: u32 = 1;   // Function call on other CPU
    pub const IPI_CPU_STOP: u32 = 2;    // Stop CPU
    pub const IPI_TIMER: u32 = 3;       // Timer sync
    pub const IPI_IRQ_WORK: u32 = 4;    // IRQ work
    pub const IPI_AI_TASK: u32 = 5;     // AI task distribution
}

/// GICv3 global state
pub struct Gicv3 {
    pub distributor_base: usize,
    pub redistributor_base: usize,
    pub cpu_count: u32,
    pub max_interrupts: u32,
}

/// Per-CPU GIC redistributor state
pub struct GicRedistributor {
    pub base: usize,
    pub cpu_id: u32,
    pub sgi_pending: u32,
}

impl Gicv3 {
    /// Create new GICv3 instance
    pub const fn new() -> Self {
        Self {
            distributor_base: GICD_BASE,
            redistributor_base: GICR_BASE,
            cpu_count: 0,
            max_interrupts: 0,
        }
    }
    
    /// Initialize GICv3 distributor
    pub fn init_distributor(&mut self) -> Result<(), &'static str> {
        unsafe {
            // Read distributor type register
            let typer = self.read_gicd(gicd::TYPER);
            self.cpu_count = ((typer >> 5) & 0x7) + 1;
            self.max_interrupts = ((typer & 0x1F) + 1) * 32;
            
            // Disable distributor during configuration
            self.write_gicd(gicd::CTLR, 0);
            
            // Configure all SPIs as Group 1 non-secure
            for i in (SPI_START..self.max_interrupts).step_by(32) {
                self.write_gicd(gicd::IGROUPR + (i / 8) as usize, 0xFFFFFFFF);
            }
            
            // Set default priority for all interrupts
            for i in (SPI_START..self.max_interrupts).step_by(4) {
                self.write_gicd(gicd::IPRIORITYR + i as usize, 0xA0A0A0A0);
            }
            
            // Configure all SPIs as level-triggered
            for i in (SPI_START..self.max_interrupts).step_by(16) {
                self.write_gicd(gicd::ICFGR + (i / 4) as usize, 0);
            }
            
            // Enable distributor with ARE (Affinity Routing Enable)
            self.write_gicd(gicd::CTLR, 0x30); // ARE_NS | ARE_S
        }
        
        crate::kernel::serial::write_str("[GICv3] Distributor initialized: ");
        crate::kernel::serial::write_u32(self.cpu_count);
        crate::kernel::serial::write_str(" CPUs, ");
        crate::kernel::serial::write_u32(self.max_interrupts);
        crate::kernel::serial::write_str(" interrupts\n");
        
        Ok(())
    }
    
    /// Initialize redistributor for current CPU
    pub fn init_redistributor(&self, cpu_id: u32) -> Result<(), &'static str> {
        let redis_base = self.redistributor_base + (cpu_id as usize * 0x20000);
        
        unsafe {
            // Wake up redistributor
            let mut waker = read_volatile((redis_base + gicr::WAKER) as *const u32);
            waker &= !0x2; // Clear ProcessorSleep bit
            write_volatile((redis_base + gicr::WAKER) as *mut u32, waker);
            
            // Wait for redistributor to wake up
            while (read_volatile((redis_base + gicr::WAKER) as *const u32) & 0x4) != 0 {
                core::hint::spin_loop();
            }
            
            // Configure SGIs and PPIs as Group 1
            write_volatile((redis_base + gicr::IGROUPR0) as *mut u32, 0xFFFFFFFF);
            
            // Set default priority for SGIs and PPIs
            for i in 0..8 {
                write_volatile(
                    (redis_base + gicr::IPRIORITYR + i * 4) as *mut u32,
                    0xA0A0A0A0
                );
            }
            
            // Enable all SGIs
            write_volatile((redis_base + gicr::ISENABLER0) as *mut u32, 0xFFFF);
            
            // Store redistributor base in per-CPU data
            if let Some(percpu) = PerCpu::for_cpu(cpu_id) {
                percpu.gicr_base = redis_base as u64;
            }
        }
        
        Ok(())
    }
    
    /// Initialize CPU interface for current CPU
    pub fn init_cpu_interface(&self) -> Result<(), &'static str> {
        unsafe {
            // Enable system register access
            let mut sre: u64;
            core::arch::asm!(
                "mrs {}, icc_sre_el1",
                out(reg) sre,
                options(nomem, nostack)
            );
            sre |= 0x1; // Enable system register interface
            core::arch::asm!(
                "msr icc_sre_el1, {}",
                in(reg) sre,
                options(nomem, nostack)
            );
            
            // Set priority mask to allow all priorities
            core::arch::asm!(
                "msr icc_pmr_el1, {}",
                in(reg) 0xFF,
                options(nomem, nostack)
            );
            
            // Set binary point (no preemption)
            core::arch::asm!(
                "msr icc_bpr1_el1, {}",
                in(reg) 0,
                options(nomem, nostack)
            );
            
            // Enable Group 1 interrupts
            core::arch::asm!(
                "msr icc_igrpen1_el1, {}",
                in(reg) 1,
                options(nomem, nostack)
            );
            
            // Enable CPU interface
            let ctlr: u64 = 0;
            core::arch::asm!(
                "msr icc_ctlr_el1, {}",
                in(reg) ctlr,
                options(nomem, nostack)
            );
        }
        
        Ok(())
    }
    
    /// Read from distributor register
    #[inline]
    unsafe fn read_gicd(&self, offset: usize) -> u32 {
        read_volatile((self.distributor_base + offset) as *const u32)
    }
    
    /// Write to distributor register
    #[inline]
    unsafe fn write_gicd(&self, offset: usize, value: u32) {
        write_volatile((self.distributor_base + offset) as *mut u32, value);
    }
    
    /// Enable an interrupt
    pub fn enable_interrupt(&self, intid: u32) {
        unsafe {
            let reg = intid / 32;
            let bit = intid % 32;
            self.write_gicd(
                gicd::ISENABLER + (reg * 4) as usize,
                1 << bit
            );
        }
    }
    
    /// Disable an interrupt
    pub fn disable_interrupt(&self, intid: u32) {
        unsafe {
            let reg = intid / 32;
            let bit = intid % 32;
            self.write_gicd(
                gicd::ICENABLER + (reg * 4) as usize,
                1 << bit
            );
        }
    }
    
    /// Set interrupt priority
    pub fn set_priority(&self, intid: u32, priority: u8) {
        unsafe {
            let reg = intid / 4;
            let shift = (intid % 4) * 8;
            let addr = self.distributor_base + gicd::IPRIORITYR + reg as usize;
            let mut val = read_volatile(addr as *const u32);
            val &= !(0xFF << shift);
            val |= (priority as u32) << shift;
            write_volatile(addr as *mut u32, val);
        }
    }
    
    /// Send SGI (Software Generated Interrupt) to specific CPUs
    pub fn send_sgi(&self, sgi_id: u32, target_list: u64, target_affinity: u64) {
        if sgi_id > 15 {
            return; // Invalid SGI ID
        }
        
        unsafe {
            // Format: [55:48]=Aff3 [39:32]=Aff2 [23:16]=Aff1 [15:0]=TargetList [27:24]=SGI
            let value = (target_affinity & 0xFF00_0000_00FF_FFFF) |
                       (target_list & 0xFFFF) |
                       ((sgi_id as u64) << 24);
            
            core::arch::asm!(
                "msr s3_0_c12_c11_5, {}", // ICC_SGI1R_EL1
                in(reg) value,
                options(nomem, nostack)
            );
        }
    }
}

/// Send IPI to specific CPU
pub fn send_ipi(cpu_id: u32, ipi_type: u32) {
    if ipi_type > 15 {
        return; // Invalid IPI
    }
    
    // Get target CPU's MPIDR affinity
    let target_affinity = if let Some(percpu) = PerCpu::for_cpu(cpu_id) {
        percpu.mpidr
    } else {
        return;
    };
    
    // Extract affinity fields
    let aff0 = target_affinity & 0xFF;
    let aff1 = (target_affinity >> 8) & 0xFF;
    let aff2 = (target_affinity >> 16) & 0xFF;
    let aff3 = (target_affinity >> 32) & 0xFF;
    
    // Build target list (1 << Aff0)
    let target_list = 1u64 << aff0;
    
    // Build affinity for SGI
    let affinity = (aff3 << 48) | (aff2 << 32) | (aff1 << 16);
    
    unsafe {
        let value = affinity | target_list | ((ipi_type as u64) << 24);
        core::arch::asm!(
            "msr s3_0_c12_c11_5, {}", // ICC_SGI1R_EL1
            in(reg) value,
            options(nomem, nostack)
        );
    }
}

/// Acknowledge interrupt
pub fn acknowledge_interrupt() -> Option<u32> {
    unsafe {
        let intid: u64;
        core::arch::asm!(
            "mrs {}, icc_iar1_el1",
            out(reg) intid,
            options(nomem, nostack)
        );
        
        let intid = intid as u32;
        if intid == 0x3FF {
            None // Spurious interrupt
        } else {
            Some(intid)
        }
    }
}

/// End of interrupt
pub fn end_of_interrupt(intid: u32) {
    unsafe {
        core::arch::asm!(
            "msr icc_eoir1_el1, {}",
            in(reg) intid as u64,
            options(nomem, nostack)
        );
    }
}

/// Global GICv3 instance
static mut GICV3: Gicv3 = Gicv3::new();

/// Initialize GICv3 for boot CPU
pub fn init() -> Result<(), &'static str> {
    unsafe {
        GICV3.init_distributor()?;
        GICV3.init_redistributor(0)?;
        GICV3.init_cpu_interface()?;
    }
    
    crate::kernel::serial::write_str("[GICv3] Initialization complete\n");
    Ok(())
}

/// Initialize GICv3 for secondary CPU
pub fn init_secondary(cpu_id: u32) -> Result<(), &'static str> {
    unsafe {
        GICV3.init_redistributor(cpu_id)?;
        GICV3.init_cpu_interface()?;
    }
    Ok(())
}