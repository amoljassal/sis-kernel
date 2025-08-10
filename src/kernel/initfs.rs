//! Minimal cpio/newc reader for embedded initfs.
//! Layout: we embed a single byte blob in a custom section and parse it at boot.
//! For now we support regular files only; directories are implicit by path.

#[repr(C, align(16))]
pub struct InitFsBlob([u8; 0]);

extern "C" {
    // Defined by linker script or by #[link_section]
    static __initfs_start: u8;
    static __initfs_end: u8;
}

#[derive(Clone, Copy)]
pub struct FileEntry<'a> {
    pub path: &'a str,
    pub data: &'a [u8],
}

pub fn available() -> bool {
    // If there is no blob, start == end
    unsafe {
        (&__initfs_end as *const u8 as usize) > (&__initfs_start as *const u8 as usize)
    }
}

pub fn find(path: &str) -> Option<FileEntry<'static>> {
    let mut it = Iter::new();
    while let Some(e) = it.next() {
        if e.path == path {
            return Some(e);
        }
    }
    None
}

pub fn list<F: FnMut(&str)>(mut f: F) {
    let mut it = Iter::new();
    while let Some(e) = it.next() {
        f(e.path);
    }
}

struct Iter {
    cur: usize,
    end: usize,
}

impl Iter {
    fn new() -> Self {
        unsafe {
            let s = &__initfs_start as *const u8 as usize;
            let e = &__initfs_end as *const u8 as usize;
            Iter { cur: s, end: e }
        }
    }
}

fn parse_hex(s: &[u8]) -> Option<usize> {
    let mut v: usize = 0;
    for &b in s {
        let d = match b {
            b'0'..=b'9' => (b - b'0') as usize,
            b'a'..=b'f' => (b - b'a' + 10) as usize,
            b'A'..=b'F' => (b - b'A' + 10) as usize,
            _ => return None,
        };
        v = (v << 4) | d;
    }
    Some(v)
}

fn align4(n: usize) -> usize { (n + 3) & !3 }

impl Iterator for Iter {
    type Item = FileEntry<'static>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end { return None; }
        // newc header is 110 bytes ASCII
        const HDR: usize = 110;
        let p = self.cur as *const u8;
        unsafe {
            // trailer?
            if self.cur + HDR > self.end { self.cur = self.end; return None; }
            if core::slice::from_raw_parts(p, 6) != b"070701" {
                // not a newc header; abort iteration
                self.cur = self.end;
                return None;
            }
            let namesize = parse_hex(core::slice::from_raw_parts(p.add(94), 8))?;
            let filesize = parse_hex(core::slice::from_raw_parts(p.add(54), 8))?;
            let path_start = self.cur + HDR;
            let path_end = path_start + namesize;
            if path_end > self.end { self.cur = self.end; return None; }
            let path_bytes = core::slice::from_raw_parts(path_start as *const u8, namesize);
            // path is NUL-terminated
            let path_bytes = &path_bytes[..path_bytes.len().saturating_sub(1)];
            let path = core::str::from_utf8_unchecked(path_bytes);

            // advance to file data (4-byte aligned)
            let data_start = align4(path_end);
            let data_end = data_start + filesize;
            if data_end > self.end { self.cur = self.end; return None; }
            let data = core::slice::from_raw_parts(data_start as *const u8, filesize);

            // advance cursor to next header (data end aligned)
            self.cur = align4(data_end);

            if path == "TRAILER!!!" {
                self.cur = self.end;
                return None;
            }
            Some(FileEntry { path, data })
        }
    }
}

// Default embedded initfs (optional). If you link a real blob, this gets discarded.
#[link_section = ".initfs"]
#[used]
static INITFS_DEFAULT: [u8; 0] = [];