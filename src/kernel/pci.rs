//! PCI enumeration and IOMMU stubs.
//!
//! This module scans the PCI bus to locate GPU devices (class code
//! 0x03) and logs their vendor and device IDs.  It assigns the
//! first GPU to the Philosophy parent and the second GPU to the
//! Technical parent.  In the future this module will configure the
//! IOMMU and use VFIO to pass through GPUs to user space.

use crate::arch::x86_64::io;
use crate::kernel::serial;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA:    u16 = 0xCFC;

static mut GPU_COUNT: usize = 0;

fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 0x8000_0000
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    unsafe {
        io::outl(PCI_CONFIG_ADDRESS, address);
        io::inl(PCI_CONFIG_DATA)
    }
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
                            write_hex16(((handle as u16) >> 8) as u16);
                            write_hex16((handle as u16) & 0xFF);
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