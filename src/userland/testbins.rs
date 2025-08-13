//! Inline, self-contained ELF64 test binaries (static PIE-like).
//! These are *minimal* and sufficient for loader/vfs wiring tests.
//! Real functional exec/entry tests arrive in Part C.
#![allow(dead_code)]

/// A trivially valid ELF64 header + one PT_LOAD segment (read-only).
/// Contents are harmless bytes; Part C will replace with purposeful payloads.
pub const BIN_HELLO: &[u8] = &[
    // e_ident[0..4] = 0x7F 'E' 'L' 'F'
    0x7f, 0x45, 0x4c, 0x46,
    // class = ELFCLASS64, data = little, version = 1, osabi = sysv, abiver = 0
    0x02, 0x01, 0x01, 0x00, 0, 0, 0, 0, 0, 0, 0, 0,
    // e_type = ET_EXEC(2), e_machine = x86_64(62), e_version = 1
    0x02, 0x00, 0x3e, 0x00, 0x01, 0x00, 0x00, 0x00,
    // e_entry (dummy), e_phoff=64, e_shoff=0
    0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // e_flags=0, e_ehsize=64, e_phentsize=56, e_phnum=1, e_shentsize=0, e_shnum=0, e_shstrndx=0
    0, 0, 0, 0, 64, 0, 56, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    // Program header (type=PT_LOAD(1))
    // p_type
    0x01, 0x00, 0x00, 0x00, // p_flags = R
    0x04, 0x00, 0x00, 0x00, // p_offset (aligned to 0x1000)
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // p_vaddr
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // p_paddr
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // p_filesz (0x20), p_memsz (0x20), p_align (0x1000)
    0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // segment payload (32 bytes)
    0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0,
];

/// Second binary with a distinct payload region to help "same VA / different contents" tests.
pub const BIN_ISOPROBE: &[u8] = &[
    // same ELF header scaffolding as above, but different segment bytes:
    0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02, 0, 0x3e, 0, 0x01, 0,
    0, 0, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, // different entry just to differ
    0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 56, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0x01, 0, 0, 0, 0x04, 0, 0, 0, 0x00, 0x10, 0, 0, 0, 0, 0, 0, 0x00, 0x10, 0, 0, 0, 0, 0, 0,
    0x00, 0x10, 0, 0, 0, 0, 0, 0, 0x20, 0, 0, 0, 0, 0, 0, 0, 0x20, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x10,
    0, 0, 0, 0, 0, 0, // segment payload (32 bytes) — distinct signature
    0xde, 0xad, 0xbe, 0xef, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0, 0,
    0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
];

/// Logical table for initfs wiring.
pub struct TestFile {
    pub path: &'static str,
    pub data: &'static [u8],
}

pub const TEST_FILES: &[TestFile] = &[
    TestFile {
        path: "/bin/hello",
        data: BIN_HELLO,
    },
    TestFile {
        path: "/bin/isoprobe",
        data: BIN_ISOPROBE,
    },
    // we'll add malformed ELFs in Part B/C
];
