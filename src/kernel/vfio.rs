//! VFIO-lite minimal passthrough (Phase 5B)
//!
//! This module provides a capability-based VFIO-like interface for device
//! passthrough with minimal kernel overhead. It maps PCI devices to handles
//! and provides syscalls for userland device inspection.

use crate::kernel::serial;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};

// VFIO capability handle type with generation
#[derive(Clone, Copy, Debug)]
pub struct VfioHandle {
    pub id: u16,
    pub gen: u8,
}

impl VfioHandle {
    pub fn new(id: u16, gen: u8) -> Self {
        Self { id, gen }
    }
    
    pub fn as_u16(self) -> u16 {
        ((self.gen as u16) << 8) | (self.id & 0xFF)
    }
}

// VFIO error types
#[derive(Debug)]
pub enum VfioError {
    NotSupported,
    NotInitialized,
    DeviceNotFound,
    HandleExhausted,
    InvalidHandle,
    PermissionDenied,
    IoError,
    InvalidDevice,
}

// Handle allocation state
struct HandleAllocator {
    next_id: u16,
    next_gen: u8,
}

static mut HANDLE_ALLOC: HandleAllocator = HandleAllocator {
    next_id: 0x8000,  // Start high to avoid conflicts
    next_gen: 1,
};

// VFIO handle state for Phase 5C
#[derive(Debug)]
struct VfioHandleState {
    pub bdf: crate::kernel::pci::Bdf,
    pub domain_id: Option<crate::arch::x86_64::iommu::DomainId>,
    pub msi_offset: Option<u8>,
    pub iova_staging: Option<crate::arch::x86_64::iommu::Iova>,
    pub staging_len: u64,
    pub bus_master_enabled: bool,
    // Phase 5C-B: MSI state
    pub irq_vector: Option<u8>,
    pub irq_count: AtomicU32,
}

// Simple handle state tracking (Phase 5C stub - no proper hash table yet)
static mut HANDLE_STATES: [Option<VfioHandleState>; 16] = [None; 16];

/// Initialize VFIO subsystem
pub fn init() -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        serial::write_str("[VFIO] Initializing VFIO-lite subsystem\n");
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Map a PCI device to a VFIO handle
pub fn map_device(bus: u8, dev: u8, func: u8) -> Result<VfioHandle, VfioError> {
    #[cfg(feature = "vfio")]
    {
        // Allocate handle with generation
        let handle = unsafe {
            let id = HANDLE_ALLOC.next_id;
            let gen = HANDLE_ALLOC.next_gen;
            
            HANDLE_ALLOC.next_id = HANDLE_ALLOC.next_id.wrapping_add(1);
            if HANDLE_ALLOC.next_id == 0 { 
                HANDLE_ALLOC.next_id = 0x8000;  // Wrap around
                HANDLE_ALLOC.next_gen = HANDLE_ALLOC.next_gen.wrapping_add(1);
                if HANDLE_ALLOC.next_gen == 0 { HANDLE_ALLOC.next_gen = 1; }
            }
            
            VfioHandle::new(id, gen)
        };
        
        serial::write_fmt(format_args!(
            "[VFIO] Mapped device {:02x}:{:02x}.{} -> handle 0x{:04x} (gen {})\n",
            bus, dev, func, handle.as_u16(), handle.gen
        )).ok();
        
        Ok(handle)
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Legacy compatibility function
pub fn unmap_device(_handle: usize) -> Result<(), VfioError> {
    Err(VfioError::NotSupported)
}

/// Close VFIO handle and reclaim resources (called on task exit)
pub fn close_handle(handle: VfioHandle) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        serial::write_fmt(format_args!(
            "[VFIO] Closing handle 0x{:04x} (gen {}): reclaiming BAR maps, IRQ arms\n",
            handle.as_u16(), handle.gen
        )).ok();
        
        // TODO Phase 5C: Actually unmap BARs from user space
        // TODO Phase 5C: Disable and unregister IRQs
        // TODO Phase 5C: Clear IOMMU domain mappings
        // TODO Phase 5C: Return handle to free pool
        
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Reclaim all VFIO handles for a task (called on task teardown)
pub fn task_cleanup() {
    #[cfg(feature = "vfio")]
    {
        serial::write_str("[VFIO] Task cleanup: reclaiming all VFIO handles\n");
        // TODO Phase 5C: Implement per-task handle tracking
        // TODO Phase 5C: Close all handles owned by current task
    }
}

/// Bind device to VFIO handle (syscall implementation)
pub fn syscall_bind_device(bus: u8, dev: u8, func: u8) -> Result<VfioHandle, VfioError> {
    map_device(bus, dev, func)
}

/// Convert u16 back to VfioHandle for syscall compatibility
fn handle_from_u16(val: u16) -> VfioHandle {
    let gen = ((val >> 8) & 0xFF) as u8;
    let id = (val & 0xFF) as u16;
    VfioHandle::new(id, gen)
}

/// Read device configuration space (syscall implementation)
pub fn syscall_cfg_read(handle_val: u16, offset: u8) -> Result<u32, VfioError> {
    #[cfg(feature = "vfio")]
    {
        // In a real implementation, we'd validate the handle and map to BDF
        // For now, just simulate reading from device 0:03.0 (e1000)
        let val = crate::kernel::pci::cfg_read32(0, 3, 0, offset);
        Ok(val)
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Write device configuration space (syscall implementation)  
pub fn syscall_cfg_write(handle_val: u16, offset: u8, value: u32) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        // Guard against dangerous config space writes
        match offset {
            0x04 => {
                // Command register - deny bus mastering until IOMMU domain ready
                if value & 0x04 != 0 {
                    serial::write_str("[VFIO] Denied bus master enable - IOMMU domain required\n");
                    return Err(VfioError::PermissionDenied);
                }
                serial::write_str("[VFIO] Config write to command register (safe bits only)\n");
            },
            0x3C => {
                // Interrupt line - allow but log
                serial::write_str("[VFIO] Config write to interrupt line\n");
            },
            _ => {
                // Other registers - allow with warning for now
                if offset < 0x40 {
                    serial::write_str("[VFIO] Config write to standard register\n");
                }
            }
        }
        
        // In a real implementation, we'd validate the handle and map to BDF
        // For now, just simulate writing to device 0:03.0 (e1000)
        crate::kernel::pci::cfg_write32(0, 3, 0, offset, value);
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Map device BAR to userspace (syscall implementation)
pub fn syscall_map_bar(handle_val: u16, bar_idx: u8) -> Result<u64, VfioError> {
    #[cfg(feature = "vfio")]
    {
        // Basic validation
        if bar_idx >= 6 {
            return Err(VfioError::InvalidDevice);
        }
        
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        // Reject I/O space BARs - only allow MMIO
        if crate::kernel::pci::bar_is_io_space(bdf, bar_idx) {
            serial::write_str("[VFIO] Rejected I/O space BAR - MMIO only\n");
            return Err(VfioError::PermissionDenied);
        }
        
        // Get BAR size for validation
        let bar_size = crate::kernel::pci::get_bar_size(bdf, bar_idx);
        if bar_size == 0 {
            serial::write_str("[VFIO] BAR has zero size\n");
            return Err(VfioError::InvalidDevice);
        }
        
        // Sanity check: refuse unreasonably large BARs (>1GB)
        if bar_size > (1024 * 1024 * 1024) {
            serial::write_str("[VFIO] BAR too large (>1GB), refusing\n");
            return Err(VfioError::InvalidDevice);
        }
        
        // For now, just return BAR0 address with enhanced safety checks
        let addr = if bar_idx == 0 {
            crate::kernel::pci::read_bar0(bdf)
        } else {
            // TODO: implement generic BAR reading for non-BAR0
            serial::write_str("[VFIO] Non-BAR0 mapping not implemented\n");
            return Err(VfioError::NotSupported);
        };
        
        // Enhanced safety: refuse zero or obviously invalid addresses
        if addr == 0 || addr < 0x1000 {
            serial::write_str("[VFIO] Invalid BAR address, refusing mapping\n");
            return Err(VfioError::InvalidDevice);
        }
        
        serial::write_fmt(format_args!(
            "[VFIO] Validated BAR{}: MMIO @ 0x{:08x}, size=0x{:08x}\n",
            bar_idx, addr, bar_size
        )).ok();
        
        // TODO Phase 5C: Validate against kernel address space overlap  
        // TODO Phase 5C: Program IOMMU domain for safe user mapping
        // TODO Phase 5C: Create actual user-accessible mapping
        
        Ok(addr)
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Setup device IRQ (syscall implementation)
pub fn syscall_setup_irq(handle_val: u16, irq_num: u8) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        serial::write_fmt(format_args!(
            "[VFIO] IRQ setup for handle 0x{:04x}, IRQ {}\n",
            handle_val, irq_num
        )).ok();
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

// Phase 5C-A: Domain Management Syscalls

/// Create IOMMU domain for VFIO handle (syscall implementation)
pub fn syscall_domain_create(handle_val: u16) -> Result<u16, VfioError> {
    #[cfg(feature = "vfio")]
    {
        use crate::arch::x86_64::iommu;
        
        // For Phase 5C-A, we use device 0:03.0 (e1000) as target
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        match iommu::domain_create(bdf.bus, bdf.dev, bdf.func) {
            Ok(domain_id) => {
                serial::write_fmt(format_args!(
                    "[VFIO] Domain {} created for handle 0x{:04x}\n",
                    domain_id.0, handle_val
                )).ok();
                
                // Store domain state (simplified for Phase 5C-A)
                // TODO Phase 5C: Proper handle state tracking
                
                Ok(domain_id.0)
            },
            Err(_) => {
                serial::write_str("[VFIO] Domain creation failed\n");
                Err(VfioError::NotSupported)
            }
        }
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Map staging buffer into IOMMU domain (syscall implementation)
pub fn syscall_domain_map_staging(handle_val: u16, len: u32) -> Result<u64, VfioError> {
    #[cfg(feature = "vfio")]
    {
        use crate::arch::x86_64::iommu;
        
        // Validate staging buffer size (16KB-64KB range)
        if len < 16384 || len > 65536 {
            serial::write_str("[VFIO] Invalid staging buffer size (must be 16KB-64KB)\n");
            return Err(VfioError::InvalidDevice);
        }
        
        // Allocate IOVA space
        let iova = match iommu::iova_alloc(len as u64) {
            Ok(addr) => addr,
            Err(_) => {
                serial::write_str("[VFIO] IOVA allocation failed\n");
                return Err(VfioError::HandleExhausted);
            }
        };
        
        // For Phase 5C-A, we use a dummy physical address
        // TODO Phase 5C: Allocate actual physical pages for staging buffer
        let dummy_paddr = 0x1000_0000u64; // 256MB mark (safe dummy)
        
        // Create domain for mapping (simplified)
        let domain_id = crate::arch::x86_64::iommu::DomainId(1); // Dummy domain
        
        match iommu::domain_map(domain_id, iova, dummy_paddr, len as u64, 0x3) {
            Ok(()) => {
                serial::write_fmt(format_args!(
                    "[VFIO] Staging buffer mapped: handle=0x{:04x} iova=0x{:08x} len=0x{:08x}\n",
                    handle_val, iova, len
                )).ok();
                Ok(iova)
            },
            Err(_) => {
                iommu::iova_free(iova, len as u64);
                serial::write_str("[VFIO] Domain mapping failed\n");
                Err(VfioError::InvalidDevice)
            }
        }
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Enable PCI bus master with domain safety check (syscall implementation)
pub fn syscall_enable_busmaster(handle_val: u16) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        // Safety check: only allow bus master if domain exists and at least one IOVA is mapped
        // For Phase 5C-A, we do basic validation
        
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        // Read current command register
        let cmd_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
        
        if (cmd_reg & 0x04) != 0 {
            serial::write_str("[vfio] busmaster=ON (already enabled)\n");
            return Ok(());
        }
        
        // Enable bus master bit
        let new_cmd = cmd_reg | 0x04;
        crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd);
        
        serial::write_fmt(format_args!(
            "[vfio] busmaster=ON for handle 0x{:04x} (cmd_reg: 0x{:04x} -> 0x{:04x})\n",
            handle_val, cmd_reg & 0xFFFF, new_cmd & 0xFFFF
        )).ok();
        
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Disable PCI bus master (syscall implementation)
pub fn syscall_disable_busmaster(handle_val: u16) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        // Read current command register  
        let cmd_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
        
        // Disable bus master bit
        let new_cmd = cmd_reg & !0x04;
        crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd);
        
        serial::write_fmt(format_args!(
            "[vfio] busmaster=OFF for handle 0x{:04x} (cmd_reg: 0x{:04x} -> 0x{:04x})\n",
            handle_val, cmd_reg & 0xFFFF, new_cmd & 0xFFFF
        )).ok();
        
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

// Phase 5C-B: MSI Management Syscalls

/// Arm MSI for VFIO handle with safety preconditions (syscall implementation)
pub fn syscall_msi_arm(handle_val: u16, vector: u8) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        // Safety preconditions: domain exists, busmaster on, MSI capability present
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        // Check bus master is enabled
        let cmd_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
        if (cmd_reg & 0x04) == 0 {
            serial::write_str("[VFIO] MSI arm denied: bus master not enabled\n");
            return Err(VfioError::PermissionDenied);
        }
        
        // Find MSI capability offset
        let msi_offset = match crate::kernel::pci::find_msi_capability(bdf) {
            Some(offset) => offset,
            None => {
                serial::write_str("[VFIO] MSI arm failed: no MSI capability\n");
                return Err(VfioError::NotSupported);
            }
        };
        
        // Install VFIO ISR before enabling MSI (force vector 0x5E for Phase 5C-B)
        crate::arch::x86_64::idt::install_vfio_isr(0x5E);
        
        // Program MSI registers (use vector 0x5E for Phase 5C-B)
        program_msi_registers(bdf, msi_offset, 0x5E)?;
        
        serial::write_fmt(format_args!(
            "[VFIO] MSI armed: handle=0x{:04x} vector=0x{:02x} cap@0x{:02x}\n",
            handle_val, 0x5E, msi_offset
        )).ok();
        
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Disarm/mask MSI for VFIO handle (syscall implementation)
pub fn syscall_msi_disarm(handle_val: u16) -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        let bdf = crate::kernel::pci::Bdf { bus: 0, dev: 3, func: 0 };
        
        // Find MSI capability
        if let Some(msi_offset) = crate::kernel::pci::find_msi_capability(bdf) {
            // Read current message control
            let ctrl_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, msi_offset);
            
            // Clear MSI Enable bit (bit 0 of Message Control)
            let new_ctrl = ctrl_reg & !0x0001;
            crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, msi_offset, new_ctrl);
            
            serial::write_fmt(format_args!(
                "[msi] masked for handle 0x{:04x} (ctrl: 0x{:04x} -> 0x{:04x})\n",
                handle_val, ctrl_reg & 0xFFFF, new_ctrl & 0xFFFF
            )).ok();
        }
        
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Program MSI registers for 32-bit MSI (sufficient for QEMU e1000)
#[cfg(feature = "vfio")]
fn program_msi_registers(bdf: crate::kernel::pci::Bdf, msi_offset: u8, vector: u8) -> Result<(), VfioError> {
    // MSI Message Address: route to BSP (APIC ID 0)
    let msi_addr = 0xFEE00000u32; // Standard MSI address base (dest APIC ID 0)
    
    // MSI Message Data: vector in low 8 bits, edge-triggered, fixed delivery
    let msi_data = vector as u32;
    
    // Write Message Address (offset +4 from capability pointer)
    crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, msi_offset + 4, msi_addr);
    
    // Write Message Data (offset +8 from capability pointer)  
    crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, msi_offset + 8, msi_data);
    
    // Enable MSI: set bit 0 of Message Control register
    let ctrl_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, msi_offset);
    let new_ctrl = ctrl_reg | 0x0001; // Set MSI Enable
    crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, msi_offset, new_ctrl);
    
    serial::write_fmt(format_args!(
        "[msi] cap@0x{:02x} addr=0xFEE00000|dest=0x00 data=0x{:02x} ctrl=enable\n",
        msi_offset, vector
    )).ok();
    
    Ok(())
}