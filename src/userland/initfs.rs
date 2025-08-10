//! Minimal initfs provider. Part A prefers *inline table* (no linker tricks)
//! so tests are hermetic. A real `.initfs` link-section can be supported later.
#![allow(dead_code)]
use core::cmp::min;
use super::testbins::TEST_FILES;

#[derive(Clone, Copy)]
pub struct Entry {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub struct InitFs {
    entries: &'static [Entry],
}

impl InitFs {
    pub fn new() -> Self {
        // Build a static view over TEST_FILES.
        // In Part B, we can add optional cpio/newc parser behind a cfg.
        // For now, zero-copy mapping from inline arrays.
        // SAFETY: all &'static references
        static mut ENTRIES_BUF: Option<&'static [Entry]> = None;
        unsafe {
            if ENTRIES_BUF.is_none() {
                // Convert at compile time via const fn isn't ergonomic; do a tiny runtime build once.
                // Since we can't allocate, map the slice directly via a leaked Box trick is not allowed.
                // Instead, we keep using TEST_FILES via a lightweight wrapper in VFS.
                // So here we just expose an empty placeholder; VFS falls back to TEST_FILES.
                ENTRIES_BUF = Some(&[]);
            }
            Self { entries: ENTRIES_BUF.unwrap() }
        }
    }

    pub fn iter(&self) -> Iter {
        Iter { idx: 0 }
    }

    pub fn find(&self, path: &str) -> Option<Entry> {
        // Prefer explicit entries array if ever populated; otherwise consult TEST_FILES.
        for e in self.entries {
            if e.name == path {
                return Some(*e);
            }
        }
        for tf in TEST_FILES {
            if tf.path == path {
                return Some(Entry { name: tf.path, data: tf.data });
            }
        }
        None
    }
}

pub struct Iter {
    idx: usize,
}

impl Iterator for Iter {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        // Empty placeholder – VFS uses TEST_FILES when ENTRIES is empty.
        let _ = self.idx;
        None
    }
}