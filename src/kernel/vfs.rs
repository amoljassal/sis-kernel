//! Tiny VFS facade over embedded initfs (read-only).
use crate::kernel::initfs;

pub struct File<'a> {
    data: &'a [u8],
    off: usize,
}

pub fn open(path: &str) -> Result<File<'static>, &'static str> {
    initfs::find(path).ok_or("ENOENT").map(|e| File { data: e.data, off: 0 })
}

pub fn read(f: &mut File<'_>, buf: &mut [u8]) -> usize {
    let n = core::cmp::min(buf.len(), f.data.len().saturating_sub(f.off));
    buf[..n].copy_from_slice(&f.data[f.off..f.off + n]);
    f.off += n;
    n
}

pub fn list(mut cb: impl FnMut(&str)) {
    initfs::list(|p| cb(p));
}

pub fn exists(path: &str) -> bool { 
    initfs::find(path).is_some() 
}

pub fn available() -> bool { 
    initfs::available() 
}

pub fn size(path: &str) -> Option<usize> {
    initfs::find(path).map(|e| e.data.len())
}