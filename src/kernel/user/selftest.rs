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