//! VFIO-lite minimal passthrough (Phase 5B)
//!
//! This module provides a capability-based VFIO-like interface for device
//! passthrough with minimal kernel overhead. It maps PCI devices to handles
//! and provides syscalls for userland device inspection.

// Controlled annotation: avoid "shared reference to mutable static" warnings produced by the VFIO
// test scaffolding internals. This does not change codegen and stays inside the `vfio`-gated module.
#![allow(static_mut_refs)]

use crate::kernel::serial;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::idt;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::irqvec;

// VFIO capability handle type with generation
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    next_id: 0x8000, // Start high to avoid conflicts
    next_gen: 1,
};

// VFIO handle state for Phase 5C
#[derive(Debug)]
struct VfioHandleState {
    pub bdf: Bdf,
    #[cfg(feature = "iommu")]
    pub domain_id: Option<crate::arch::x86_64::iommu::DomainId>,
    pub msi_offset: Option<u8>,
    #[cfg(feature = "iommu")]
    pub iova_staging: Option<crate::arch::x86_64::iommu::Iova>,
    pub staging_len: u64,
    pub bus_master_enabled: bool,
    // Phase 5C-B: MSI state
    pub irq_vector: Option<u8>,
    pub irq_count: AtomicU32,
    // MSI runtime:
    pub vector: Option<u8>,
    pub cpu: u8,
    pub armed_epoch: u64,
    pub t_arm_tsc: AtomicU64,
    pub t_trigger_tsc: AtomicU64,
}

// Simple handle state tracking (Phase 5C stub - no proper hash table yet)
static mut HANDLE_STATES: [Option<VfioHandleState>; 16] = [const { None }; 16];
static NEXT_GEN: AtomicU64 = AtomicU64::new(1);

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn tsc() -> u64 {
    // Use ARM64 system timer counter
    unsafe {
        let count: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) count);
        count
    }
}

// Helper types and functions for patch compatibility
use crate::kernel::pci::{cfg_read32, cfg_write32};
use crate::kernel::types::Bdf;

// Handle state manipulation helpers
fn get_state_mut(
    table: &mut [Option<VfioHandleState>; 16],
    h: VfioHandle,
) -> Option<&mut VfioHandleState> {
    // Simple lookup - in real implementation would use proper hash table
    for state in table.iter_mut() {
        if let Some(s) = state.as_mut() {
            // Match by handle - simplified for now
            return Some(s);
        }
    }
    None
}

fn get_state(table: &[Option<VfioHandleState>; 16], h: VfioHandle) -> Option<&VfioHandleState> {
    // Simple lookup - in real implementation would use proper hash table
    for state in table.iter() {
        if let Some(s) = state.as_ref() {
            // Match by handle - simplified for now
            return Some(s);
        }
    }
    None
}

// MSI capability and programming helpers
#[cfg(feature = "vfio")]
fn find_msi_cap(bdf: Bdf) -> Option<u8> {
    crate::kernel::pci::find_msi_capability(bdf)
}

#[cfg(not(feature = "vfio"))]
fn find_msi_cap(_bdf: Bdf) -> Option<u8> {
    None
}

fn program_msi(bdf: Bdf, cap_offset: u8, vector: u8) {
    let msi_addr = 0xFEE00000u32; // Standard MSI address base (BSP)
    let msi_data = vector as u32;

    // Write MSI registers
    cfg_write32(bdf.bus, bdf.dev, bdf.func, cap_offset + 4, msi_addr);
    cfg_write32(bdf.bus, bdf.dev, bdf.func, cap_offset + 8, msi_data);

    // Enable MSI
    let ctrl_reg = cfg_read32(bdf.bus, bdf.dev, bdf.func, cap_offset);
    let new_ctrl = ctrl_reg | 0x0001;
    cfg_write32(bdf.bus, bdf.dev, bdf.func, cap_offset, new_ctrl);
}

fn clear_msi(bdf: Bdf, cap_offset: u8) {
    let ctrl_reg = cfg_read32(bdf.bus, bdf.dev, bdf.func, cap_offset);
    let new_ctrl = ctrl_reg & !0x0001;
    cfg_write32(bdf.bus, bdf.dev, bdf.func, cap_offset, new_ctrl);
}

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
                HANDLE_ALLOC.next_id = 0x8000; // Wrap around
                HANDLE_ALLOC.next_gen = HANDLE_ALLOC.next_gen.wrapping_add(1);
                if HANDLE_ALLOC.next_gen == 0 {
                    HANDLE_ALLOC.next_gen = 1;
                }
            }

            VfioHandle::new(id, gen)
        };

        serial::write_fmt(format_args!(
            "[VFIO] Mapped device {:02x}:{:02x}.{} -> handle 0x{:04x} (gen {})\n",
            bus,
            dev,
            func,
            handle.as_u16(),
            handle.gen
        ))
        .ok();

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
            handle.as_u16(),
            handle.gen
        ))
        .ok();

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

// Simplified handle creation for patch compatibility
fn create_handle_state(bdf: Bdf) -> Option<VfioHandle> {
    unsafe {
        for i in 0..HANDLE_STATES.len() {
            if HANDLE_STATES[i].is_none() {
                let h = VfioHandle::new(i as u16, 1);
                HANDLE_STATES[i] = Some(VfioHandleState {
                    bdf,
                    #[cfg(feature = "iommu")]
                    domain_id: None,
                    msi_offset: find_msi_cap(bdf),
                    #[cfg(feature = "iommu")]
                    iova_staging: None,
                    staging_len: 0,
                    bus_master_enabled: false,
                    irq_vector: None,
                    irq_count: AtomicU32::new(0),
                    vector: None,
                    cpu: 0, // Simplified for single CPU case
                    armed_epoch: 0,
                    t_arm_tsc: AtomicU64::new(0),
                    t_trigger_tsc: AtomicU64::new(0),
                });
                return Some(h);
            }
        }
    }
    None
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
            }
            0x3C => {
                // Interrupt line - allow but log
                serial::write_str("[VFIO] Config write to interrupt line\n");
            }
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

        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

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
        ))
        .ok();

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
        ))
        .ok();
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

        // TODO: Get BDF from handle - for now use hardcoded 00:02.0
        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

        match iommu::domain_create(bdf.bus, bdf.dev, bdf.func) {
            Ok(domain_id) => {
                serial::write_fmt(format_args!(
                    "[VFIO] Domain {} created for handle 0x{:04x}\n",
                    domain_id.0, handle_val
                ))
                .ok();

                // Store domain state (simplified for Phase 5C-A)
                // TODO Phase 5C: Proper handle state tracking

                Ok(domain_id.0)
            }
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
                ))
                .ok();
                Ok(iova)
            }
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

        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

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
            handle_val,
            cmd_reg & 0xFFFF,
            new_cmd & 0xFFFF
        ))
        .ok();

        // CRITICAL: Update handle state to reflect bus master enabled
        let handle = handle_from_u16(handle_val);
        unsafe {
            let mut table_lock = core::ptr::addr_of_mut!(HANDLE_STATES);
            let table = &mut *table_lock;
            if let Some(state) = get_state_mut(table, handle) {
                state.bus_master_enabled = true;
                serial::write_str("[vfio] handle state updated: bus_master_enabled=true\n");
            }
        }

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
        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

        // Read current command register
        let cmd_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);

        // Disable bus master bit
        let new_cmd = cmd_reg & !0x04;
        crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd);

        serial::write_fmt(format_args!(
            "[vfio] busmaster=OFF for handle 0x{:04x} (cmd_reg: 0x{:04x} -> 0x{:04x})\n",
            handle_val,
            cmd_reg & 0xFFFF,
            new_cmd & 0xFFFF
        ))
        .ok();

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
    serial::write_str("[msi] arm called\n");

    #[cfg(feature = "vfio")]
    {
        // TODO: Get BDF from handle - for now use hardcoded 00:02.0
        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

        serial::write_str("[vfio] arm bdf=00:02.0\n");

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

        // **NEW: Disable INTx before arming MSI (PCI spec recommends MSI-exclusive mode)**
        let current_cmd = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
        let new_cmd = current_cmd | 0x0400; // Set bit 10: Interrupt Disable
        crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd);

        serial::write_fmt(format_args!(
            "[msi] INTx disabled for MSI-exclusive mode (cmd: 0x{:04x} -> 0x{:04x})\n",
            current_cmd & 0xFFFF,
            new_cmd & 0xFFFF
        ))
        .ok();

        // Program MSI registers (use vector 0x5E for Phase 5C-B)
        program_msi_registers(bdf, msi_offset, 0x5E)?;

        serial::write_fmt(format_args!(
            "[VFIO] MSI armed: handle=0x{:04x} vector=0x{:02x} cap@0x{:02x}\n",
            handle_val, 0x5E, msi_offset
        ))
        .ok();

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
        let bdf = crate::kernel::types::Bdf {
            bus: 0,
            dev: 2,
            func: 0,
        };

        // Find MSI capability
        if let Some(msi_offset) = crate::kernel::pci::find_msi_capability(bdf) {
            // Read current message control
            let ctrl_reg = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, msi_offset);

            // Clear MSI Enable bit (bit 0 of Message Control)
            let new_ctrl = ctrl_reg & !0x0001;
            crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, msi_offset, new_ctrl);

            // **NEW: Enhanced disarm with register readback verification**
            let readback = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, msi_offset);
            let is_disabled = (readback & 0x0001) == 0;

            serial::write_fmt(format_args!(
                "[msi] disarmed handle 0x{:04x} (ctrl: 0x{:04x} -> 0x{:04x}) verify={}\n",
                handle_val,
                ctrl_reg & 0xFFFF,
                new_ctrl & 0xFFFF,
                if is_disabled { "OK" } else { "FAIL" }
            ))
            .ok();

            // **NEW: Re-enable INTx if desired (optional, depends on device requirements)**
            let current_cmd = crate::kernel::pci::cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
            let new_cmd = current_cmd & !0x0400; // Clear bit 10: re-enable INTx
            crate::kernel::pci::cfg_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd);

            serial::write_fmt(format_args!(
                "[msi] INTx re-enabled (cmd: 0x{:04x} -> 0x{:04x})\n",
                current_cmd & 0xFFFF,
                new_cmd & 0xFFFF
            ))
            .ok();
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
fn program_msi_registers(
    bdf: crate::kernel::types::Bdf,
    msi_offset: u8,
    vector: u8,
) -> Result<(), VfioError> {
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
    ))
    .ok();

    Ok(())
}

/// **NEW: Trigger e1000 MSI via BAR0 MMIO (IMS set → ICS poke)**
/// This function maps BAR0, manipulates the e1000's Interrupt Mask Set (IMS)
/// and Interrupt Cause Set (ICS) registers to trigger an MSI.
#[cfg(feature = "vfio")]
pub fn syscall_msi_trigger_e1000(handle_val: u16) -> Result<(), VfioError> {
    let bdf = crate::kernel::types::Bdf {
        bus: 0,
        dev: 2,
        func: 0,
    };

    // Get BAR0 base address
    let bar0_addr = crate::kernel::pci::read_bar0(bdf);
    if bar0_addr == 0 {
        serial::write_str("[msi-trigger] BAR0 not mapped\\n");
        return Err(VfioError::InvalidDevice);
    }

    serial::write_fmt(format_args!(
        "[msi-trigger] e1000 BAR0 @ 0x{:08x}, manipulating IMS/ICS...\\n",
        bar0_addr
    ))
    .ok();

    // **e1000 Register Offsets (from Intel e1000 datasheet):**
    // IMS (Interrupt Mask Set)   = BAR0 + 0x00D0
    // ICS (Interrupt Cause Set)  = BAR0 + 0x00C8
    // ICR (Interrupt Cause Read) = BAR0 + 0x00C0

    let bar0_ptr = bar0_addr as *mut u32;

    unsafe {
        // Step 1: Set IMS to unmask TXDW (Transmit Descriptor Written Back) - bit 0
        let ims_offset = 0x00D0 / 4; // Convert to u32 offset
        let ims_ptr = bar0_ptr.add(ims_offset);
        core::ptr::write_volatile(ims_ptr, 0x0001); // Enable TXDW interrupt

        serial::write_str("[msi-trigger] IMS set to 0x0001 (TXDW enabled)\\n");

        // Step 2: Trigger interrupt by setting ICS (this will cause MSI if armed)
        let ics_offset = 0x00C8 / 4; // Convert to u32 offset
        let ics_ptr = bar0_ptr.add(ics_offset);
        core::ptr::write_volatile(ics_ptr, 0x0001); // Set TXDW cause bit

        serial::write_str("[msi-trigger] ICS poked with 0x0001 - MSI should fire now\\n");

        // Step 3: Read back ICR to see if interrupt is pending
        let icr_offset = 0x00C0 / 4; // Convert to u32 offset
        let icr_ptr = bar0_ptr.add(icr_offset);
        let icr_value = core::ptr::read_volatile(icr_ptr);

        serial::write_fmt(format_args!(
            "[msi-trigger] ICR readback = 0x{:08x} (bit 0 should be set)\\n",
            icr_value
        ))
        .ok();
    }

    Ok(())
}

// New simplified MSI arm function for patch compatibility
pub fn syscall_msi_arm_new(h: VfioHandle) -> i32 {
    unsafe {
        let mut table_lock = core::ptr::addr_of_mut!(HANDLE_STATES);
        let table = &mut *table_lock;
        let s = match get_state_mut(table, h) {
            Some(state) => state,
            None => return -1,
        };

        if s.msi_offset.is_none() {
            serial::write_str("[vfio] no MSI cap\n");
            return -2;
        }

        if !s.bus_master_enabled {
            serial::write_str("[vfio] arm requires busmaster\n");
            return -3;
        }

        #[cfg(target_arch = "x86_64")]
        {
            let cpu = 0usize; // Simplified for single CPU case
            let vec = match crate::arch::x86_64::irqvec::alloc_vector(cpu) {
                Some(v) => v,
                None => {
                    serial::write_str("[vfio] no free vectors\n");
                    return -4;
                }
            };

            program_msi(s.bdf, s.msi_offset.unwrap(), vec);
            crate::arch::x86_64::idt::vfio_isr_vector_install(vec);
            s.armed_epoch = NEXT_GEN.fetch_add(1, Ordering::Relaxed);
            let packed = ((s.armed_epoch & 0xFFFF_FFFF) << 16) | (h.as_u16() as u64);
            crate::arch::x86_64::idt::vfio_map_vector(vec, packed);
            s.vector = Some(vec);
            s.t_arm_tsc.store(tsc(), Ordering::Relaxed);
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 stub - VFIO MSI not yet implemented
            serial::write_str("[vfio] ARM64 MSI stub - not implemented\n");
            s.armed_epoch = NEXT_GEN.fetch_add(1, Ordering::Relaxed);
            s.vector = None;
            s.t_arm_tsc.store(tsc(), Ordering::Relaxed);
        }
        0
    }
}

pub fn syscall_msi_disarm_new(h: VfioHandle) -> i32 {
    unsafe {
        let mut table_lock = core::ptr::addr_of_mut!(HANDLE_STATES);
        let table = &mut *table_lock;
        let s = match get_state_mut(table, h) {
            Some(state) => state,
            None => return -1,
        };

        if let Some(cap) = s.msi_offset {
            clear_msi(s.bdf, cap);
        }

        if let Some(vec) = s.vector.take() {
            #[cfg(target_arch = "x86_64")]
            {
                crate::arch::x86_64::idt::vfio_unmap_vector(vec);
                crate::arch::x86_64::irqvec::free_vector(s.cpu as usize, vec);
            }
            #[cfg(target_arch = "aarch64")]
            {
                // ARM64 stub - nothing to free
            }
        }
        0
    }
}

/// Called by ISR: returns (count, t_trigger_snapshot)
pub fn on_irq(h: VfioHandle, epoch: u64, now: u64) -> (u64, u64) {
    unsafe {
        let table_lock = core::ptr::addr_of!(HANDLE_STATES);
        let table = &*table_lock;
        if let Some(s) = get_state(table, h) {
            if s.armed_epoch == epoch {
                let c = s.irq_count.fetch_add(1, Ordering::Relaxed) + 1;
                let t = s.t_trigger_tsc.load(Ordering::Relaxed);
                (c as u64, t)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    }
}

/// Updated trigger function for e1000 that also records timestamp
pub fn syscall_msi_trigger_e1000_new(h: VfioHandle) -> i32 {
    unsafe {
        let mut table_lock = core::ptr::addr_of_mut!(HANDLE_STATES);
        let table = &mut *table_lock;
        let s = match get_state_mut(table, h) {
            Some(state) => state,
            None => return -1,
        };

        // Existing BAR0 IMS/ICS writes (from existing implementation)
        let bdf = s.bdf;
        let bar0_addr = crate::kernel::pci::read_bar0(bdf);
        if bar0_addr == 0 {
            return -2;
        }

        let bar0_ptr = bar0_addr as *mut u32;

        // Set IMS and trigger via ICS
        let ims_offset = 0x00D0 / 4;
        let ims_ptr = bar0_ptr.add(ims_offset);
        core::ptr::write_volatile(ims_ptr, 0x0001);

        let ics_offset = 0x00C8 / 4;
        let ics_ptr = bar0_ptr.add(ics_offset);
        core::ptr::write_volatile(ics_ptr, 0x0001);

        // Record trigger timestamp
        s.t_trigger_tsc.store(tsc(), Ordering::Relaxed);
        0
    }
}

#[inline(always)]
pub fn lookup_by_vector(vec: u8) -> Option<(VfioHandle, u64)> {
    #[cfg(target_arch = "x86_64")]
    let packed = crate::arch::x86_64::idt::vfio_vector_packed_load(vec);
    #[cfg(target_arch = "aarch64")]
    let packed = 0u64; // ARM64 stub - no interrupt vectors loaded yet
    if packed == 0 {
        return None;
    }
    let handle = VfioHandle::new((packed & 0xFFFF) as u16, 1);
    let epoch = (packed >> 16) & 0xFFFF_FFFF;
    Some((handle, epoch))
}
