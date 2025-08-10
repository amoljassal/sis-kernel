//! Userland validation suite (Phase 4.1 · Part C)
//! Build-time selected via `RUSTFLAGS="--cfg selftest_USR_INIT"` (etc.)
//! Success => qemu_exit(0x00); failure => non-zero, test-specific code.
#![allow(dead_code)]

use core::ptr;
use crate::serial;
use crate::arch::x86_64::io::qemu_exit;

use super::vfs;
use super::elfsec;
use super::pid;

// Distinct failure codes per scenario
const FAIL_INIT_VFS: u8       = 0x11;
const FAIL_INIT_ELF: u8       = 0x12;
const FAIL_SPAWN_TWO_VFS: u8  = 0x21;
const FAIL_SPAWN_TWO_ELF: u8  = 0x22;
const FAIL_SPAWN_TWO_PID: u8  = 0x23;
const FAIL_ELF_EDGES_GOOD: u8 = 0x31;
const FAIL_ELF_EDGES_BAD: u8  = 0x32;
const FAIL_VFS_NEG_OPEN: u8   = 0x41;
const FAIL_VFS_NEG_READ: u8   = 0x42;

/// Minimal "spawn" stub for validation: we don't execute the image here,
/// we just validate ELF + allocate a PID to prove the loader ingress is sound.
/// In your full loader, replace this with real mapping + task creation.
fn validate_spawn(path: &str) -> Result<u32, ()> {
    let img = vfs::open_elf_verified(path).ok_or(())?;
    elfsec::validate_elf64(img).map_err(|_| ())?;
    pid::alloc_pid().map_err(|_| () )
}

/// USR_INIT — validate that `/sbin/init` exists and passes ELF checks.
pub fn run_usr_init() {
    serial::write_str("[selftest] USR_INIT: start\n");
    let p = "/sbin/init";
    if vfs::open_elf_verified(p).is_none() {
        serial::write_str("[selftest] USR_INIT: vfs open_elf_verified failed\n");
        unsafe { qemu_exit(FAIL_INIT_VFS); }
    }
    let img = vfs::open_elf_verified(p).unwrap();
    if let Err(_e) = elfsec::validate_elf64(img) {
        serial::write_str("[selftest] USR_INIT: elf validate failed\n");
        unsafe { qemu_exit(FAIL_INIT_ELF); }
    }
    serial::write_str("[selftest] USR_INIT: ok\n");
    unsafe { qemu_exit(0x00); }
}

/// USR_SPAWN_TWO — prove we can "spawn" two distinct processes (PID distinction),
/// using two inline test ELFs added in Part A: /bin/hello and /bin/isoprobe
pub fn run_usr_spawn_two() {
    serial::write_str("[selftest] USR_SPAWN_TWO: start\n");
    let pid1 = match validate_spawn("/bin/hello") { Ok(p) => p, Err(_) => {
        serial::write_str("[selftest] USR_SPAWN_TWO: spawn hello failed\n");
        unsafe { qemu_exit(FAIL_SPAWN_TWO_VFS); } } };
    let pid2 = match validate_spawn("/bin/isoprobe") { Ok(p) => p, Err(_) => {
        serial::write_str("[selftest] USR_SPAWN_TWO: spawn isoprobe failed\n");
        unsafe { qemu_exit(FAIL_SPAWN_TWO_ELF); } } };
    if pid1 == pid2 {
        serial::write_str("[selftest] USR_SPAWN_TWO: pid collision\n");
        unsafe { qemu_exit(FAIL_SPAWN_TWO_PID); }
    }
    serial::write_str("[selftest] USR_SPAWN_TWO: ok pid1 != pid2\n");
    unsafe { qemu_exit(0x00); }
}

/// USR_ELF_EDGES — verify that a *valid* ELF passes, and a *malformed* ELF fails.
pub fn run_usr_elf_edges() {
    serial::write_str("[selftest] USR_ELF_EDGES: start\n");
    // positive: a known-good test ELF
    let good = vfs::open_elf_verified("/bin/hello").ok_or(()).and_then(|img| {
        elfsec::validate_elf64(img).map_err(|_| ()).map(|_| ())
    });
    if good.is_err() {
        serial::write_str("[selftest] USR_ELF_EDGES: good elf rejected\n");
        unsafe { qemu_exit(FAIL_ELF_EDGES_GOOD); }
    }
    // negative: a tiny malformed ELF (bad magic)
    const BAD_ELF: [u8; 64] = [
        0x00, 0x45, 0x4C, 0x46, // not 0x7F 'E' 'L' 'F'
        0x02, 0x01, 0x01, 0x00, // 64-bit, LE, ver 1
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    if elfsec::validate_elf64(&BAD_ELF).is_ok() {
        serial::write_str("[selftest] USR_ELF_EDGES: bad elf accepted\n");
        unsafe { qemu_exit(FAIL_ELF_EDGES_BAD); }
    }
    serial::write_str("[selftest] USR_ELF_EDGES: ok\n");
    unsafe { qemu_exit(0x00); }
}

/// USR_VFS_NEG — negative VFS coverage: non-existent open, bounded read.
pub fn run_usr_vfs_neg() {
    serial::write_str("[selftest] USR_VFS_NEG: start\n");
    if vfs::open("/no/such/file").is_some() {
        serial::write_str("[selftest] USR_VFS_NEG: unexpected open success\n");
        unsafe { qemu_exit(FAIL_VFS_NEG_OPEN); }
    }
    if let Some(mut f) = vfs::open("/bin/hello") {
        // Attempt to read "too much"; File::read() must clamp to available size.
        let mut big = [0u8; 8192];
        let n1 = f.read(&mut big);
        let n2 = f.read(&mut big);
        // second read should be 0 (EOF)
        if n1 == 0 || n2 != 0 {
            serial::write_str("[selftest] USR_VFS_NEG: read clamp/EOF failed\n");
            unsafe { qemu_exit(FAIL_VFS_NEG_READ); }
        }
    } else {
        serial::write_str("[selftest] USR_VFS_NEG: hello missing\n");
        unsafe { qemu_exit(FAIL_VFS_NEG_OPEN); }
    }
    serial::write_str("[selftest] USR_VFS_NEG: ok\n");
    unsafe { qemu_exit(0x00); }
}

/// Entry – compile-time select exactly one scenario.
pub fn run() {
    #[cfg(selftest_USR_INIT)]
    { run_usr_init(); return; }
    #[cfg(selftest_USR_SPAWN_TWO)]
    { run_usr_spawn_two(); return; }
    #[cfg(selftest_USR_ELF_EDGES)]
    { run_usr_elf_edges(); return; }
    #[cfg(selftest_USR_VFS_NEG)]
    { run_usr_vfs_neg(); return; }
    serial::write_str("[selftest] userland: no scenario selected\n");
    unsafe { qemu_exit(0x7F); }
}