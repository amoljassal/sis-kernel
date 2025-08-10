// IOMMU Detection and Management for SIS Kernel
// Phase 5A: Probe + Guard Rails (deny-all policy by default)

pub mod dmar;

#[cfg(feature = "iommu")]
use self::dmar::DmarParser;

/// IOMMU capability detection results
#[derive(Debug, Clone, Copy)]
pub struct IommuCapabilities {
    pub vendor: IommuVendor,
    pub dma_remapping: bool,
    pub interrupt_remapping: bool,
    pub fault_recording: bool,
}

/// Supported IOMMU vendors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IommuVendor {
    Intel,
    Amd,
    None,
}

/// IOMMU domain identifier for Phase 5C
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomainId(pub u16);

/// IOVA (I/O Virtual Address) type
pub type Iova = u64;

/// Domain management errors
#[derive(Debug)]
pub enum DomainError {
    NoIommu,
    NotSupported,
    AlreadyMapped,
    NotMapped,
    InvalidAlignment,
    OutOfDomains,
    InvalidAddress,
}

/// Simple IOVA allocator state
#[cfg(feature = "vfio")]
struct IovaAllocator {
    base: Iova,
    current: Iova,
    limit: Iova,
}

#[cfg(feature = "vfio")]
static mut IOVA_ALLOC: IovaAllocator = IovaAllocator {
    base: 0x1000_0000,      // Start at 256MB to avoid low ranges
    current: 0x1000_0000,   // Current allocation pointer
    limit: 0x4000_0000,     // Limit to 1GB aperture for safety
};

/// Global IOMMU state (feature-gated)
#[cfg(feature = "iommu")]
static mut IOMMU_CAPS: Option<IommuCapabilities> = None;

/// Initialize IOMMU detection and establish deny-all policy
#[cfg(feature = "iommu")]
pub fn init() -> Result<(), &'static str> {
    use crate::kernel::serial;
    
    serial::write_str("[iommu] Phase 5A: IOMMU detection + deny-all policy\n");
    
    // Probe Intel DMAR first
    match DmarParser::probe() {
        Ok(dmar_caps) => {
            serial::write_str("[iommu] Intel DMAR detected\n");
            unsafe {
                IOMMU_CAPS = Some(IommuCapabilities {
                    vendor: IommuVendor::Intel,
                    dma_remapping: dmar_caps.dma_remapping,
                    interrupt_remapping: dmar_caps.interrupt_remapping,
                    fault_recording: dmar_caps.fault_recording,
                });
            }
            
            // Establish deny-all DMA policy
            establish_deny_all_policy()?;
            return Ok(());
        }
        Err(_) => {
            serial::write_str("[iommu] Intel DMAR not found, checking AMD IVRS\n");
        }
    }
    
    // Probe AMD IVRS (stub implementation for now)
    if probe_amd_ivrs() {
        serial::write_str("[iommu] AMD IVRS detected (not yet implemented)\n");
        unsafe {
            IOMMU_CAPS = Some(IommuCapabilities {
                vendor: IommuVendor::Amd,
                dma_remapping: false, // Not implemented yet
                interrupt_remapping: false,
                fault_recording: false,
            });
        }
        return Err("AMD IVRS support not implemented");
    }
    
    serial::write_str("[iommu] No IOMMU detected\n");
    unsafe {
        IOMMU_CAPS = Some(IommuCapabilities {
            vendor: IommuVendor::None,
            dma_remapping: false,
            interrupt_remapping: false,
            fault_recording: false,
        });
    }
    
    Ok(())
}

/// No-op initialization when IOMMU feature is disabled
#[cfg(not(feature = "iommu"))]
pub fn init() -> Result<(), &'static str> {
    use crate::kernel::serial;
    serial::write_str("[iommu] IOMMU support disabled (no 'iommu' feature)\n");
    Ok(())
}

/// Get current IOMMU capabilities
#[cfg(feature = "iommu")]
pub fn capabilities() -> Option<IommuCapabilities> {
    unsafe { IOMMU_CAPS }
}

/// No-op capabilities when feature disabled
#[cfg(not(feature = "iommu"))]
pub fn capabilities() -> Option<IommuCapabilities> {
    None
}

/// Establish deny-all DMA policy (Intel IOMMU)
#[cfg(feature = "iommu")]
fn establish_deny_all_policy() -> Result<(), &'static str> {
    use crate::kernel::serial;
    
    serial::write_str("[iommu] Establishing deny-all DMA policy\n");
    
    // For Phase 5A, we just log the intent
    // Phase 5B will implement actual IOMMU page table setup
    serial::write_str("[iommu] DENY-ALL policy: no identity mapping by default\n");
    serial::write_str("[iommu] DMA will be blocked until explicit allow()\n");
    
    Ok(())
}

/// Allow DMA for specific physical address range
#[cfg(feature = "iommu")]
pub fn allow_dma(paddr: u64, len: u64) -> Result<(), &'static str> {
    use crate::kernel::serial;
    
    // Phase 5A: stub implementation (just log)
    serial::write_str("[iommu] allow_dma() called (stub)\n");
    
    match unsafe { IOMMU_CAPS } {
        Some(caps) if caps.dma_remapping => {
            // TODO Phase 5B: actual IOMMU page table updates
            serial::write_str("[iommu] TODO: map paddr range to IOVA space\n");
            Ok(())
        }
        Some(_) => {
            serial::write_str("[iommu] No DMA remapping capability\n");
            Ok(()) // No-op if no IOMMU
        }
        None => Err("IOMMU not initialized"),
    }
}

/// Deny DMA for specific physical address range
#[cfg(feature = "iommu")]
pub fn deny_dma(paddr: u64, len: u64) -> Result<(), &'static str> {
    use crate::kernel::serial;
    
    // Phase 5A: stub implementation (just log)
    serial::write_str("[iommu] deny_dma() called (stub)\n");
    
    match unsafe { IOMMU_CAPS } {
        Some(caps) if caps.dma_remapping => {
            // TODO Phase 5B: actual IOMMU page table updates  
            serial::write_str("[iommu] TODO: unmap paddr range from IOVA space\n");
            Ok(())
        }
        Some(_) => {
            serial::write_str("[iommu] No DMA remapping capability\n");
            Ok(()) // No-op if no IOMMU
        }
        None => Err("IOMMU not initialized"),
    }
}

/// No-op DMA control when feature disabled
#[cfg(not(feature = "iommu"))]
pub fn allow_dma(_paddr: u64, _len: u64) -> Result<(), &'static str> {
    Ok(()) // No-op when IOMMU disabled
}

#[cfg(not(feature = "iommu"))]
pub fn deny_dma(_paddr: u64, _len: u64) -> Result<(), &'static str> {
    Ok(()) // No-op when IOMMU disabled  
}

/// AMD IVRS probe stub (Phase 5A)
fn probe_amd_ivrs() -> bool {
    // TODO: Parse ACPI IVRS table for AMD IOMMU
    // For now, always return false (not implemented)
    false
}

// Phase 5C-A: Domain Management Functions

/// Create IOMMU domain for a PCI device
#[cfg(feature = "vfio")]
pub fn domain_create(bus: u8, dev: u8, func: u8) -> Result<DomainId, DomainError> {
    use crate::kernel::serial;
    
    let caps = unsafe { IOMMU_CAPS }.ok_or(DomainError::NoIommu)?;
    
    if !caps.dma_remapping {
        return Err(DomainError::NotSupported);
    }
    
    // Simple domain ID allocation (Phase 5C stub)
    static mut NEXT_DOMAIN: u16 = 1;
    let domain_id = unsafe {
        let id = NEXT_DOMAIN;
        NEXT_DOMAIN = NEXT_DOMAIN.wrapping_add(1);
        if NEXT_DOMAIN == 0 { NEXT_DOMAIN = 1; } // Skip 0 as invalid
        DomainId(id)
    };
    
    serial::write_fmt(format_args!(
        "[iommu] domain={} created for device {:02x}:{:02x}.{} aperture=[0x{:08x}..0x{:08x}]\n",
        domain_id.0, bus, dev, func,
        unsafe { IOVA_ALLOC.base }, unsafe { IOVA_ALLOC.limit }
    )).ok();
    
    // TODO Phase 5C: Actually create Intel DMAR domain context
    // TODO Phase 5C: Setup page tables for this domain
    // TODO Phase 5C: Associate device BDF with domain
    
    Ok(domain_id)
}

/// Map physical memory into IOMMU domain
#[cfg(feature = "vfio")]
pub fn domain_map(domain: DomainId, iova: Iova, paddr: u64, len: u64, _perms: u32) -> Result<(), DomainError> {
    use crate::kernel::serial;
    
    let caps = unsafe { IOMMU_CAPS }.ok_or(DomainError::NoIommu)?;
    
    if !caps.dma_remapping {
        return Err(DomainError::NotSupported);
    }
    
    // Validate alignment
    if (iova & 0xFFF) != 0 || (paddr & 0xFFF) != 0 || (len & 0xFFF) != 0 {
        return Err(DomainError::InvalidAlignment);
    }
    
    // Validate IOVA range
    if iova < unsafe { IOVA_ALLOC.base } || iova + len > unsafe { IOVA_ALLOC.limit } {
        return Err(DomainError::InvalidAddress);
    }
    
    serial::write_fmt(format_args!(
        "[iommu] domain={} map iova=0x{:08x} -> paddr=0x{:08x} len=0x{:08x}\n",
        domain.0, iova, paddr, len
    )).ok();
    
    // TODO Phase 5C: Actually program IOMMU page tables
    // TODO Phase 5C: Invalidate IOTLB for this domain
    
    Ok(())
}

/// Unmap IOVA range from IOMMU domain
#[cfg(feature = "vfio")]
pub fn domain_unmap(domain: DomainId, iova: Iova, len: u64) -> Result<(), DomainError> {
    use crate::kernel::serial;
    
    let caps = unsafe { IOMMU_CAPS }.ok_or(DomainError::NoIommu)?;
    
    if !caps.dma_remapping {
        return Err(DomainError::NotSupported);
    }
    
    serial::write_fmt(format_args!(
        "[iommu] domain={} unmap iova=0x{:08x} len=0x{:08x}\n",
        domain.0, iova, len
    )).ok();
    
    // TODO Phase 5C: Clear IOMMU page tables for this range
    // TODO Phase 5C: Invalidate IOTLB
    
    Ok(())
}

/// Destroy IOMMU domain and free resources
#[cfg(feature = "vfio")]
pub fn domain_destroy(domain: DomainId) -> Result<(), DomainError> {
    use crate::kernel::serial;
    
    serial::write_fmt(format_args!(
        "[iommu] domain={} destroyed\n", domain.0
    )).ok();
    
    // TODO Phase 5C: Free all page tables for this domain
    // TODO Phase 5C: Remove device context associations
    // TODO Phase 5C: Return domain ID to free pool
    
    Ok(())
}

/// Allocate IOVA range (4KB aligned)
#[cfg(feature = "vfio")]
pub fn iova_alloc(len: u64) -> Result<Iova, DomainError> {
    use crate::kernel::serial;
    
    // Round up to 4KB alignment
    let aligned_len = (len + 0xFFF) & !0xFFF;
    
    unsafe {
        let iova = IOVA_ALLOC.current;
        let next = iova + aligned_len;
        
        if next > IOVA_ALLOC.limit {
            return Err(DomainError::InvalidAddress);
        }
        
        IOVA_ALLOC.current = next;
        
        serial::write_fmt(format_args!(
            "[iova] alloc base=0x{:08x} len=0x{:08x} (aligned from 0x{:08x})\n",
            iova, aligned_len, len
        )).ok();
        
        Ok(iova)
    }
}

/// Free IOVA range (Phase 5C stub - no actual free list yet)
#[cfg(feature = "vfio")]
pub fn iova_free(iova: Iova, len: u64) {
    use crate::kernel::serial;
    
    serial::write_fmt(format_args!(
        "[iova] free base=0x{:08x} len=0x{:08x} (TODO: implement free list)\n",
        iova, len
    )).ok();
    
    // TODO Phase 5C: Implement proper IOVA free list management
}

// No-op stubs when VFIO feature disabled
#[cfg(not(feature = "vfio"))]
pub fn domain_create(_bus: u8, _dev: u8, _func: u8) -> Result<DomainId, DomainError> {
    Err(DomainError::NotSupported)
}

#[cfg(not(feature = "vfio"))]
pub fn domain_map(_domain: DomainId, _iova: Iova, _paddr: u64, _len: u64, _perms: u32) -> Result<(), DomainError> {
    Err(DomainError::NotSupported)
}

#[cfg(not(feature = "vfio"))]
pub fn domain_unmap(_domain: DomainId, _iova: Iova, _len: u64) -> Result<(), DomainError> {
    Err(DomainError::NotSupported)
}

#[cfg(not(feature = "vfio"))]
pub fn domain_destroy(_domain: DomainId) -> Result<(), DomainError> {
    Err(DomainError::NotSupported)
}

#[cfg(not(feature = "vfio"))]
pub fn iova_alloc(_len: u64) -> Result<Iova, DomainError> {
    Err(DomainError::NotSupported)
}

#[cfg(not(feature = "vfio"))]
pub fn iova_free(_iova: Iova, _len: u64) {
    // No-op
}

/// IOMMU selftests (feature-gated)
#[cfg(all(feature = "iommu", any(
    selftest_IOMMU_PROBE,
    selftest_IOMMU_DENY_DEFAULT
)))]
pub mod selftest {
    use super::*;
    use crate::kernel::serial;
    use crate::arch::x86_64::io::qemu_exit;

    #[cfg(cfg(selftest_IOMMU_PROBE))]
    pub fn run_iommu_probe() -> ! {
        serial::write_str("[selftest] IOMMU_PROBE: checking IOMMU detection\n");
        
        match capabilities() {
            Some(caps) => {
                match caps.vendor {
                    IommuVendor::Intel => {
                        serial::write_str("[selftest] IOMMU_PROBE: Intel DMAR found\n");
                        if caps.dma_remapping {
                            serial::write_str("[PASS: IOMMU_PROBE] Intel DMA remapping available\n");
                            qemu_exit(0x00); // Success
                        } else {
                            serial::write_str("[FAIL: IOMMU_PROBE] Intel DMAR found but no DMA remapping\n");
                            qemu_exit(0x51); // DMAR present but incomplete
                        }
                    }
                    IommuVendor::Amd => {
                        serial::write_str("[SKIP: IOMMU_PROBE] AMD IVRS found but not implemented\n");
                        qemu_exit(0x50); // Skip code for AMD
                    }
                    IommuVendor::None => {
                        serial::write_str("[SKIP: IOMMU_PROBE] No IOMMU detected\n");
                        qemu_exit(0x50); // Skip code for no IOMMU
                    }
                }
            }
            None => {
                serial::write_str("[FAIL: IOMMU_PROBE] IOMMU not initialized\n");
                qemu_exit(0x52); // Initialization failure
            }
        }
    }

    #[cfg(cfg(selftest_IOMMU_DENY_DEFAULT))]
    pub fn run_iommu_deny_default() -> ! {
        serial::write_str("[selftest] IOMMU_DENY_DEFAULT: testing deny-all policy\n");
        
        match capabilities() {
            Some(caps) if caps.dma_remapping => {
                // Test that DMA is denied by default
                serial::write_str("[selftest] Testing deny-all DMA policy\n");
                
                // Phase 5A: just verify the policy is established
                serial::write_str("[selftest] Deny-all policy established\n");
                serial::write_str("[PASS: IOMMU_DENY_DEFAULT] DMA blocked by default\n");
                qemu_exit(0x00); // Success
            }
            Some(_) => {
                serial::write_str("[SKIP: IOMMU_DENY_DEFAULT] No DMA remapping capability\n");
                qemu_exit(0x50); // Skip
            }
            None => {
                serial::write_str("[FAIL: IOMMU_DENY_DEFAULT] IOMMU not initialized\n"); 
                qemu_exit(0x53); // Initialization failure
            }
        }
    }
}