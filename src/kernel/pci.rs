//! PCI enumeration and VFIO-lite integration (Phase 5B).
//!
//! This module provides PCI configuration space access and device
//! enumeration for VFIO-lite passthrough. It maintains the original
//! GPU scanning functionality while adding the new Phase 5B API.

use crate::arch::x86_64::io::{outl, inl};
use crate::kernel::serial::{self, write_hex32};
use core::fmt::Write;

#[allow(dead_code)]
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
#[allow(dead_code)]
const PCI_CONFIG_DATA:    u16 = 0xCFC;

static mut GPU_COUNT: usize = 0;

// Phase 5B: Clean PCI config space interface
#[inline(always)]
fn cfg_addr(bus: u8, dev: u8, func: u8, off: u8) -> u32 {
    let aligned = (off & !3) as u32;
    (1u32 << 31)
        | ((bus as u32) << 16)
        | ((dev as u32) << 11)
        | ((func as u32) << 8)
        | aligned
}

pub fn cfg_read32(bus: u8, dev: u8, func: u8, off: u8) -> u32 {
    unsafe { outl(0xCF8, cfg_addr(bus, dev, func, off)); inl(0xCFC) }
}

pub fn cfg_write32(bus: u8, dev: u8, func: u8, off: u8, val: u32) {
    unsafe { outl(0xCF8, cfg_addr(bus, dev, func, off)); outl(0xCFC, val); }
}

#[derive(Debug, Clone, Copy)]
pub struct Bdf { pub bus: u8, pub dev: u8, pub func: u8 }

#[derive(Clone, Copy)]
pub struct PciId { pub vendor: u16, pub device: u16 }

pub fn read_id(bdf: Bdf) -> PciId {
    let v = cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x00);
    PciId { vendor: (v & 0xFFFF) as u16, device: ((v >> 16) & 0xFFFF) as u16 }
}

pub fn read_bar0(bdf: Bdf) -> u64 {
    let lo = cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    let is_64 = (lo & 0b110) == 0b100;
    let base_lo = (lo & !0xF) as u64;
    if is_64 {
        let hi = cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x14) as u64;
        (hi << 32) | base_lo
    } else { base_lo }
}

/// Check if BAR is I/O space (bit 0 = 1) vs MMIO (bit 0 = 0)
pub fn bar_is_io_space(bdf: Bdf, bar_idx: u8) -> bool {
    if bar_idx >= 6 { return false; }
    let bar_offset = 0x10 + (bar_idx as u8 * 4);
    let bar_val = cfg_read32(bdf.bus, bdf.dev, bdf.func, bar_offset);
    (bar_val & 1) != 0
}

/// Get BAR size by writing all 1s and reading back
pub fn get_bar_size(bdf: Bdf, bar_idx: u8) -> u32 {
    if bar_idx >= 6 { return 0; }
    let bar_offset = 0x10 + (bar_idx as u8 * 4);
    
    // Save original value
    let original = cfg_read32(bdf.bus, bdf.dev, bdf.func, bar_offset);
    
    // Write all 1s to determine size
    cfg_write32(bdf.bus, bdf.dev, bdf.func, bar_offset, 0xFFFFFFFF);
    let size_mask = cfg_read32(bdf.bus, bdf.dev, bdf.func, bar_offset);
    
    // Restore original value
    cfg_write32(bdf.bus, bdf.dev, bdf.func, bar_offset, original);
    
    // Calculate size (mask off type bits)
    let masked = if (original & 1) != 0 {
        // I/O space - mask off bottom 2 bits
        size_mask & !0x3
    } else {
        // Memory space - mask off bottom 4 bits  
        size_mask & !0xF
    };
    
    if masked == 0 { return 0; }
    (!masked) + 1
}

/// Walk PCI capability list to find MSI capability
#[cfg(feature = "vfio")]
pub fn find_msi_capability(bdf: Bdf) -> Option<u8> {
    serial::write_str("[msi] scan bdf=");
    serial::write_hex8(bdf.bus); serial::write_str(":");
    serial::write_hex8(bdf.dev); serial::write_str(".");
    serial::write_hex8(bdf.func); serial::write_str("\n");
    
    // Check if capabilities supported (status register at 0x06)
    let status = (cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x04) >> 16) as u16;
    serial::write_str("[msi] status=0x");
    crate::kernel::serial::write_hex16(status);
    if (status & 0x10) == 0 {
        serial::write_str(" [no-caps]\n");
        return None; // No capabilities
    }
    serial::write_str(" [caps-ok]\n");
    
    // Get capabilities pointer
    let cap_ptr = (cfg_read32(bdf.bus, bdf.dev, bdf.func, 0x34) & 0xFF) as u8;
    serial::write_str("[msi] cap_ptr=0x");
    crate::kernel::serial::write_hex8(cap_ptr);
    serial::write_str("\n");
    if cap_ptr == 0 { 
        serial::write_str("[msi] no cap ptr\n");
        return None; 
    }
    
    let mut ptr = cap_ptr;
    let mut hops = 0;
    
    // Walk capability list with DWORD alignment per PCI spec
    while ptr != 0 && hops < 32 {
        let cap_reg = cfg_read32(bdf.bus, bdf.dev, bdf.func, ptr);
        let id = (cap_reg & 0xFF) as u8;
        serial::write_str("[msi] cap id=0x");
        crate::kernel::serial::write_hex8(id);
        serial::write_str(" @0x");
        crate::kernel::serial::write_hex8(ptr);
        serial::write_str("\n");
        
        if id == 0x05 {
            serial::write_str("[msi] FOUND MSI\n");
            return Some(ptr);
        }
        let next = ((cap_reg >> 8) & 0xFF) as u8;     // next cap pointer
        ptr = next & 0xFC;                            // DWORD alignment per PCI spec
        hops += 1;
    }
    serial::write_str("[msi] no msi\n");
    
    None
}

pub fn find_first_e1000() -> Option<Bdf> {
    // With QEMU `-device e1000` the default is usually 00:03.0
    for dev in 0..32u8 {
        let bdf = Bdf { bus: 0, dev, func: 0 };
        let id = read_id(bdf);
        if id.vendor == 0x8086 {
            // Classic e1000 device IDs often 0x100e under QEMU
            serial::write_fmt(format_args!("[pci] probe 00:{:02x}.0 vendor=0x{:04x} device=0x{:04x}\n",
                dev, id.vendor, id.device)).ok();
            return Some(bdf);
        }
    }
    None
}

// Legacy functions for backward compatibility
fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    cfg_read32(bus, device, function, offset)
}

fn nibble_to_hex(n: u8) -> u8 {
    match n & 0xF {
        0..=9 => b'0' + (n & 0xF),
        10..=15 => b'A' + ((n & 0xF) - 10),
        _ => b'?',
    }
}

fn write_hex8(val: u8) {
    let hi = nibble_to_hex(val >> 4);
    let lo = nibble_to_hex(val);
    serial::write_char(hi);
    serial::write_char(lo);
}

fn write_hex16(val: u16) {
    write_hex8((val >> 8) as u8);
    write_hex8(val as u8);
}

/// Scan the PCI bus for GPUs and log them.  Assign GPUs to parents
/// according to the order they are found.  In a full kernel this
/// function would configure the IOMMU for passthrough.
pub fn init() {
    serial::write_str("Scanning PCI bus for GPUs...\n");
    
    // Quick device discovery pass for diagnostics
    #[cfg(feature = "vfio")]
    {
        serial::write_str("[vfio:list] Discovered PCI devices:\n");
        for bus in 0..=2u8 {  // Scan first few buses for quick diagnostics
            for dev in 0..32u8 {
                let vendor_device = pci_config_read(bus, dev, 0, 0x00);
                if vendor_device == 0xFFFF_FFFF { continue; }
                let vendor = (vendor_device & 0xFFFF) as u16;
                let device = ((vendor_device >> 16) & 0xFFFF) as u16;
                
                let bdf = Bdf { bus, dev, func: 0 };
                
                serial::write_str("  ");
                write_hex8(bus);
                serial::write_char(b':');
                write_hex8(dev);
                serial::write_str(".0 vendor=0x");
                write_hex16(vendor);
                serial::write_str(" device=0x");
                write_hex16(device);
                
                // Show BAR0 info if present
                let bar0 = read_bar0(bdf);
                if bar0 != 0 {
                    serial::write_str(" bar0=0x");
                    write_hex16((bar0 >> 16) as u16);
                    write_hex16((bar0 & 0xFFFF) as u16);
                    
                    let bar_size = get_bar_size(bdf, 0);
                    if bar_size > 0 {
                        serial::write_str(" size=0x");
                        write_hex32(bar_size);
                    }
                    
                    if bar_is_io_space(bdf, 0) {
                        serial::write_str(" (I/O)");
                    } else {
                        serial::write_str(" (MMIO)");
                    }
                }
                
                // Check for MSI capability
                if let Some(msi_offset) = find_msi_capability(bdf) {
                    serial::write_str(" [msi-cap-ok]");
                }
                
                serial::write_str("\n");
            }
        }
    }
    
    // Initialise VFIO subsystem if available
    match crate::kernel::vfio::init() {
        Ok(()) => serial::write_str("[PCI] VFIO initialised\n"),
        Err(e) => serial::write_str("[PCI] VFIO not available, running without passthrough\n"),
    };
    unsafe { GPU_COUNT = 0; }
    for bus in 0..=255u8 {
        for device in 0..32u8 {
            let vendor_device = pci_config_read(bus, device, 0, 0x00);
            if vendor_device == 0xFFFF_FFFF { continue; }
            let header_type = pci_config_read(bus, device, 0, 0x0C);
            let multi_function = ((header_type >> 16) & 0x80) != 0;
            let functions = if multi_function { 8u8 } else { 1u8 };
            for function in 0..functions {
                let id = pci_config_read(bus, device, function, 0x00);
                if id == 0xFFFF_FFFF { continue; }
                let class_code = pci_config_read(bus, device, function, 0x08);
                let class = ((class_code >> 24) & 0xFF) as u8;
                if class == 0x03 {
                    let vendor: u16 = (id & 0xFFFF) as u16;
                    let device_id: u16 = ((id >> 16) & 0xFFFF) as u16;
                    serial::write_str("[PCI] GPU found ");
                    write_hex8(bus);
                    serial::write_char(b':');
                    write_hex8(device);
                    serial::write_char(b'.');
                    write_hex8(function);
                    serial::write_str(" vendor=0x");
                    write_hex16(vendor);
                    serial::write_str(" device=0x");
                    write_hex16(device_id);
                    // Assign GPU to a parent.  GPU0 -> Philosophy, GPU1 -> Technical.
                    unsafe {
                        match GPU_COUNT {
                            0 => serial::write_str(" assigned to Philosophy parent"),
                            1 => serial::write_str(" assigned to Technical parent"),
                            _ => serial::write_str(" unused for now"),
                        }
                    }
                    // Attempt to map the device via VFIO
                    let res = crate::kernel::vfio::map_device(bus, device, function);
                    match res {
                        Ok(handle) => {
                            serial::write_str(" [VFIO handle=0x");
                            write_hex16(((handle.as_u16()) >> 8) as u16);
                            write_hex16((handle.as_u16()) & 0xFF);
                            serial::write_str("]\n");
                        }
                        Err(_) => {
                            serial::write_str(" [VFIO unavailable]\n");
                        }
                    }
                    unsafe { GPU_COUNT += 1; }
                }
            }
        }
    }
}