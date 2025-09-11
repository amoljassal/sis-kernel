// Intel DMAR (DMA Remapping) ACPI Table Parser
// Phase 5A: Detection and capability logging only

use crate::kernel::serial;
use alloc::vec::Vec;
use core::mem;

/// DMAR ACPI table signature
const DMAR_SIGNATURE: [u8; 4] = *b"DMAR";

/// DMAR capabilities detected from ACPI table
#[derive(Debug, Clone, Copy)]
pub struct DmarCapabilities {
    pub dma_remapping: bool,
    pub interrupt_remapping: bool,
    pub fault_recording: bool,
    pub host_address_width: u8,
}

/// DMAR ACPI table header (simplified)
#[repr(C, packed)]
struct DmarHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
    host_address_width: u8,
    flags: u8,
    _reserved: [u8; 10],
}

/// DMAR remapping structure types
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum DmarStructureType {
    DmaRemappingHardwareUnit = 0,
    ReservedMemoryRegion = 1,
    RootPortAtsCapability = 2,
    RemappingHardwareStaticAffinity = 3,
    AcpiNameSpaceDeviceDeclaration = 4,
}

/// DMAR parser for Intel IOMMU detection
pub struct DmarParser;

impl DmarParser {
    /// Probe for Intel DMAR in ACPI tables
    pub fn probe() -> Result<DmarCapabilities, &'static str> {
        // Phase 5A: simplified ACPI table search
        // In a full implementation, this would:
        // 1. Parse RSDP -> RSDT/XSDT -> DMAR table
        // 2. Walk through all ACPI tables looking for DMAR

        // For Phase 5A, we'll use a simplified approach that checks
        // common ACPI memory regions where DMAR might be located
        Self::search_acpi_region(0xE0000, 0x20000) // BIOS area
            .or_else(|| Self::search_acpi_region(0xF0000, 0x10000)) // Extended BIOS area
            .ok_or("DMAR table not found in ACPI")
    }

    /// Search for DMAR table in a specific memory region
    fn search_acpi_region(start: usize, size: usize) -> Option<DmarCapabilities> {
        // Phase 5A: stub implementation
        // In Phase 5B, this would actually map and scan memory

        serial::write_str("[dmar] Searching ACPI region for DMAR table\n");

        // For QEMU with -device intel-iommu, simulate finding DMAR
        // This is a placeholder that would be replaced with real ACPI parsing
        if cfg!(feature = "qemu-intel-iommu-sim") {
            serial::write_str("[dmar] Simulated Intel DMAR detection (QEMU)\n");
            return Some(DmarCapabilities {
                dma_remapping: true,
                interrupt_remapping: true,
                fault_recording: true,
                host_address_width: 48, // Common value
            });
        }

        serial::write_str("[dmar] No DMAR table found in region\n");
        None
    }

    /// Parse DMAR table structure (Phase 5A stub)
    fn parse_dmar_table(_dmar_ptr: *const DmarHeader) -> Result<DmarCapabilities, &'static str> {
        // Phase 5A: placeholder for real DMAR parsing
        // Phase 5B will implement:
        // 1. Validate ACPI table checksum
        // 2. Parse remapping hardware units
        // 3. Extract capabilities and base addresses

        serial::write_str("[dmar] DMAR table parsing (stub)\n");

        Ok(DmarCapabilities {
            dma_remapping: true,
            interrupt_remapping: false, // Conservative default
            fault_recording: false,
            host_address_width: 39, // Conservative default
        })
    }

    /// Validate ACPI table checksum
    fn validate_checksum(_header: &DmarHeader) -> bool {
        // Phase 5A: always return true (stub)
        // Phase 5B: implement actual ACPI checksum validation
        true
    }

    /// Parse remapping hardware units from DMAR
    fn parse_hardware_units(
        _dmar_ptr: *const DmarHeader,
    ) -> Result<Vec<DmarHardwareUnit>, &'static str> {
        // Phase 5A: return empty vector (stub)
        // Phase 5B: parse actual hardware units
        Ok(Vec::new())
    }
}

/// DMAR Hardware Unit (Phase 5B structure)
#[allow(dead_code)]
struct DmarHardwareUnit {
    segment: u16,
    base_address: u64,
    include_pci_all: bool,
    device_scope: Vec<DmarDeviceScope>,
}

/// DMAR Device Scope (Phase 5B structure)
#[allow(dead_code)]
struct DmarDeviceScope {
    scope_type: u8,
    length: u8,
    enumeration_id: u8,
    start_bus: u8,
    path: Vec<DmarDevicePath>,
}

/// DMAR Device Path (Phase 5B structure)
#[allow(dead_code)]
struct DmarDevicePath {
    device: u8,
    function: u8,
}

// Phase 5A: Feature-gated QEMU simulation support
// This allows testing IOMMU detection without real hardware

#[cfg(feature = "qemu-intel-iommu-sim")]
impl DmarParser {
    /// Simulate Intel IOMMU detection for QEMU testing
    pub fn simulate_qemu_detection() -> DmarCapabilities {
        DmarCapabilities {
            dma_remapping: true,
            interrupt_remapping: true,
            fault_recording: true,
            host_address_width: 48,
        }
    }
}

// Unit tests for DMAR parsing (Phase 5B)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmar_structure_sizes() {
        // Ensure packed structures have correct sizes
        assert_eq!(mem::size_of::<DmarHeader>(), 48);
    }

    #[test]
    fn test_dmar_signature() {
        assert_eq!(DMAR_SIGNATURE, *b"DMAR");
    }
}
