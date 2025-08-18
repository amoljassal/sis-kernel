//! Extremely small VFS façade backed by initfs (inline test files for Part A).
#![allow(dead_code)]
use super::elfsec::{self, validate_elf64};
use super::initfs::InitFs;

pub struct File<'a> {
    data: &'a [u8],
    pos: usize,
}

pub fn boot_probe() {
    // Enumerate once to ensure initfs is link/loaded. No-op for now.
    let _fs = InitFs::new();
}

pub fn exists(path: &str) -> bool {
    InitFs::new().find(path).is_some()
}

pub fn size(path: &str) -> Option<usize> {
    InitFs::new().find(path).map(|e| e.data.len())
}

pub fn open(path: &str) -> Option<File<'static>> {
    InitFs::new().find(path).map(|e| File {
        data: e.data,
        pos: 0,
    })
}

pub fn read(f: &mut File<'_>, buf: &mut [u8]) -> usize {
    f.read(buf)
}

pub fn available() -> bool {
    // For the inline test files approach, we always have files available
    true
}

pub fn list(mut cb: impl FnMut(&str)) {
    use super::testbins::TEST_FILES;
    for test_file in TEST_FILES {
        cb(test_file.path);
    }
}

impl<'a> File<'a> {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let remain = self.data.len().saturating_sub(self.pos);
        let n = core::cmp::min(remain, buf.len());
        if n == 0 {
            return 0;
        }
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        n
    }
}

/// Open and *verify* an ELF64 image using the hardening validator.
/// Returns the file bytes *only if* the image passes all checks.
pub fn open_elf_verified(path: &str) -> Option<&'static [u8]> {
    let e = InitFs::new().find(path)?;
    match validate_elf64(e.data) {
        Ok(_meta) => Some(e.data),
        Err(_why) => None,
    }
}
