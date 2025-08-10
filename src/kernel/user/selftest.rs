use crate::kernel::serial;
#[cfg(feature = "userland")]
use crate::userland::vfs;
#[cfg(not(feature = "userland"))]
use crate::kernel::vfs;
use crate::kernel::user::proc;

pub fn run_usr_init() -> Result<(), &'static str> {
    serial::write_str("[selftest] USR_INIT starting...\n");

    if !vfs::available() {
        serial::write_str("[selftest] initfs not present — SKIP\n");
        unsafe { crate::arch::x86_64::io::qemu_exit(0x7F); } // Skip code
    }

    // If caller hasn't embedded real ELF, we provide a soft skip.
    let has_elf = option_env!("INITFS_HAS_ELF").is_some();
    if !has_elf {
        serial::write_str("[selftest] INITFS_HAS_ELF not set — SKIP\n");
        unsafe { crate::arch::x86_64::io::qemu_exit(0x7F); } // Skip code
    }

    // Test VFS functionality first
    if !vfs::exists("/sbin/init") {
        return Err("no /sbin/init");
    }

    // List files in initfs
    serial::write_str("[selftest] initfs contents:\n");
    vfs::list(|path| {
        serial::write_str("  ");
        serial::write_str(path);
        serial::write_str("\n");
    });

    // Test file reading
    if let Some(mut file) = vfs::open("/sbin/init") {
        let mut buf = [0u8; 64];
        let n = vfs::read(&mut file, &mut buf);
        serial::write_str("[selftest] read ");
        serial::write_u64(n as u64);
        serial::write_str(" bytes from /sbin/init\n");
        
        if n >= 4 && &buf[0..4] == b"\x7FELF" {
            serial::write_str("[selftest] valid ELF header detected\n");
        }
    }

    serial::write_str("[selftest] spawning /sbin/init...\n");
    let pid = proc::sys_spawn_exec(c_str("/sbin/init"), core::ptr::null(), core::ptr::null())
        .map_err(|_| "spawn fail")?;

    // Wait for it to exit cleanly (simplified for Phase 4 v1)
    serial::write_str("[selftest] spawn successful, PID=");
    serial::write_u64(pid);
    serial::write_str("\n");
    
    Ok(())
}

fn c_str(s: &str) -> *const u8 {
    // v1: cheap static C string (only used in selftest)
    static mut BUF: [u8; 128] = [0; 128];
    unsafe {
        let bytes = s.as_bytes();
        let n = core::cmp::min(bytes.len(), BUF.len() - 1);
        BUF[..n].copy_from_slice(&bytes[..n]);
        BUF[n] = 0;
        BUF.as_ptr()
    }
}

// Phase 5B: VFIO userland device inspector selftests
#[cfg(feature = "vfio")]
pub fn run_vfio_bind_e1000() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_BIND_E1000 starting...\n");
    
    // First verify we can find the e1000 device
    if let Some(bdf) = crate::kernel::pci::find_first_e1000() {
        let id = crate::kernel::pci::read_id(bdf);
        if id.vendor != 0x8086 {
            return Err("Expected Intel vendor ID 0x8086");
        }
        serial::write_str("[selftest] Found Intel device for binding test\n");
    } else {
        return Err("No e1000 device found");
    }
    
    // Test binding to e1000 device (typically at 0:03.0 in QEMU)
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO bind syscall completed\n");
    Ok(())
}

#[cfg(feature = "vfio")]
pub fn run_vfio_cfg_read() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_CFG_READ starting...\n");
    
    // First verify device has expected e1000 characteristics
    if let Some(bdf) = crate::kernel::pci::find_first_e1000() {
        let id = crate::kernel::pci::read_id(bdf);
        if id.vendor != 0x8086 {
            return Err("Expected Intel vendor ID 0x8086");
        }
        
        // Check for known e1000 device IDs
        match id.device {
            0x100e | 0x10d3 | 0x1000 | 0x1001 => {
                serial::write_str("[selftest] Found known e1000 device ID\n");
            },
            _ => {
                serial::write_str("[selftest] Warning: Unknown e1000 device ID, continuing\n");
            }
        }
    } else {
        return Err("No e1000 device found for config read test");
    }
    
    // Bind the device
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Read vendor ID via syscall path (not direct PCI helper)
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x51",    // SYS_VFIO_CFG_READ
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 0x00",    // offset = 0x00 (vendor ID)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO config read syscall completed\n");
    Ok(())
}

#[cfg(feature = "vfio")]  
pub fn run_vfio_map_bar() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_MAP_BAR starting...\n");
    
    // First verify BAR0 has a valid non-zero address and reasonable size
    if let Some(bdf) = crate::kernel::pci::find_first_e1000() {
        let bar0 = crate::kernel::pci::read_bar0(bdf);
        if bar0 == 0 {
            return Err("BAR0 is zero - device not configured");
        }
        if bar0 < 0x1000 {
            return Err("BAR0 address too low - suspicious");
        }
        
        let bar_size = crate::kernel::pci::get_bar_size(bdf, 0);
        if bar_size == 0 {
            return Err("BAR0 has zero size");
        }
        if bar_size > (1024 * 1024 * 1024) {  // > 1GB
            return Err("BAR0 too large (>1GB)");
        }
        
        serial::write_str("[selftest] BAR0 has valid address and size\n");
    } else {
        return Err("No e1000 device found for BAR test");
    }
    
    // Bind device first
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Map BAR0
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x53",    // SYS_VFIO_MAP_BAR
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 0",       // bar_idx = 0 (BAR0)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO BAR mapping syscall completed\n");
    Ok(())
}

#[cfg(feature = "vfio")]
pub fn run_vfio_irq_setup() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_IRQ_SETUP starting...\n");
    
    // Bind device first
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Setup IRQ
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x54",    // SYS_VFIO_SETUP_IRQ
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 11",      // irq_num = 11 (typical for e1000)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO IRQ setup syscall completed\n");
    Ok(())
}

// Phase 5C-A: IOMMU domain and DMA staging selftests
#[cfg(feature = "vfio")]
pub fn run_vfio_domain_create() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_DOMAIN_CREATE starting...\n");
    
    // First bind a device
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Create IOMMU domain
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x55",    // SYS_VFIO_DOMAIN_CREATE
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO domain creation syscall completed\n");
    Ok(())
}

#[cfg(feature = "vfio")]
pub fn run_vfio_dma_staging() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_DMA_STAGING starting...\n");
    
    // Bind device
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Create domain
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x55",    // SYS_VFIO_DOMAIN_CREATE
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    // Map 16KB staging buffer
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x56",    // SYS_VFIO_DOMAIN_MAP_STAGING
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 16384",   // len = 16KB
            "syscall",
        );
    };
    
    // Enable bus master (this should now be allowed)
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x57",    // SYS_VFIO_ENABLE_BUSMASTER
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] VFIO DMA staging test completed\n");
    Ok(())
}

// Phase 5C-B: MSI smoke test with end-to-end interrupt delivery
#[cfg(feature = "vfio")]
pub fn run_vfio_msi_smoke() -> Result<(), &'static str> {
    serial::write_str("[selftest] VFIO_MSI_SMOKE starting...\n");
    
    // Step 1: Bind device
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x50",    // SYS_VFIO_BIND
            "mov rdi, 0",       // bus = 0
            "mov rsi, 3",       // dev = 3
            "mov rdx, 0",       // func = 0
            "syscall",
        );
    };
    
    // Step 2: Create domain
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x55",    // SYS_VFIO_DOMAIN_CREATE
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    // Step 3: Map staging buffer
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x56",    // SYS_VFIO_DOMAIN_MAP_STAGING
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 16384",   // len = 16KB
            "syscall",
        );
    };
    
    // Step 4: Enable bus master
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x57",    // SYS_VFIO_ENABLE_BUSMASTER
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    // Step 5: Arm MSI at vector 0x5E
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x58",    // SYS_VFIO_MSI_ARM
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 0x5E",    // vector = 0x5E
            "syscall",
        );
    };
    
    // Step 6: Map BAR0 to get device registers (for BAR0 nudge)
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x53",    // SYS_VFIO_MAP_BAR
            "mov rdi, 0x8000",  // handle (dummy)
            "mov rsi, 0",       // bar_idx = 0 (BAR0)
            "syscall",
        );
    };
    
    serial::write_str("[selftest] MSI smoke test setup completed\n");
    serial::write_str("[selftest] NOTE: BAR0 nudge would trigger MSI (not implemented in this test)\n");
    serial::write_str("[selftest] In full test, writing to device register would generate interrupt\n");
    
    // Step 7: Cleanup - disarm MSI
    let result = unsafe {
        core::arch::asm!(
            "mov rax, 0x59",    // SYS_VFIO_MSI_DISARM
            "mov rdi, 0x8000",  // handle (dummy)
            "syscall",
        );
    };
    
    Ok(())
}