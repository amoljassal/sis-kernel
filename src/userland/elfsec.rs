//! ELF64 validation + hardening (bounds, architecture, and W^X policy).
//! Pure `no_std`, zero-allocation, intended to guard loader ingress.
//! This is a *validator*, not a full parser/mapper.
use core::mem::size_of;

#[derive(Debug, Clone, Copy)]
pub struct ElfMeta {
    pub entry: u64,
    pub phoff: u64,
    pub phentsz: u16,
    pub phnum: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    NotElf64Le,
    BadTypeOrMachine,
    BadHeaderBounds,
    BadPhBounds,
    PhEntryTooSmall,
    SegmentOverflow,
    BadFileszMemsz,
    BadAlign,
    WAndX, // violates W^X: PF_W and PF_X both set
}

const EI_NIDENT: usize = 16;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 0x1;
const PF_W: u32 = 0x2;
const PF_R: u32 = 0x4;

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[inline(always)]
fn read_hdr(image: &[u8]) -> Result<&Elf64Ehdr, ElfError> {
    if image.len() < size_of::<Elf64Ehdr>() { return Err(ElfError::TooSmall); }
    // SAFETY: we only read, alignment for repr(C) u64/u32/u16 fields is fine on &[u8]
    let hdr = unsafe { &*(image.as_ptr() as *const Elf64Ehdr) };
    Ok(hdr)
}

#[inline(always)]
fn slice_ok(image: &[u8], off: u64, len: u64) -> bool {
    let end = off.saturating_add(len);
    (end as usize) <= image.len() && (off as usize) <= image.len()
}

/// Validate an ELF64/x86_64 little-endian file and enforce **W^X** (no segment has both W and X).
/// Also checks all basic bounds/size/alignment constraints appropriate for a static/PIE-like loader.
pub fn validate_elf64(image: &[u8]) -> Result<ElfMeta, ElfError> {
    let hdr = read_hdr(image)?;
    // Magic
    let id = &hdr.e_ident;
    if id[0] != 0x7f || id[1] != b'E' || id[2] != b'L' || id[3] != b'F' {
        return Err(ElfError::BadMagic);
    }
    // Class=64, Data=little, Version=1
    if id[4] != 2 || id[5] != 1 || id[6] != 1 {
        return Err(ElfError::NotElf64Le);
    }
    // x86_64 (EM=62), type = ET_EXEC(2) or ET_DYN(3)
    let et = hdr.e_type;
    let em = hdr.e_machine;
    if !((et == 2 || et == 3) && em == 62) {
        return Err(ElfError::BadTypeOrMachine);
    }
    // phoff/num bounds
    let phoff = hdr.e_phoff;
    let phentsz = hdr.e_phentsize;
    let phnum = hdr.e_phnum;
    if (phentsz as usize) < size_of::<Elf64Phdr>() {
        return Err(ElfError::PhEntryTooSmall);
    }
    let ph_table_len = (phnum as u64) * (phentsz as u64);
    if !slice_ok(image, phoff, ph_table_len) {
        return Err(ElfError::BadPhBounds);
    }
    // Iterate PHDRs
    for i in 0..phnum {
        let off = phoff + (i as u64) * (phentsz as u64);
        // SAFETY: we validated table range above
        let ph = unsafe { &*(image.as_ptr().add(off as usize) as *const Elf64Phdr) };
        if ph.p_type == PT_LOAD {
            // filesz <= memsz
            if ph.p_filesz > ph.p_memsz {
                return Err(ElfError::BadFileszMemsz);
            }
            // bounds: file slice exists
            if ph.p_filesz != 0 && !slice_ok(image, ph.p_offset, ph.p_filesz) {
                return Err(ElfError::SegmentOverflow);
            }
            // alignment: page-aligned vaddr and p_align multiple of 0x1000 (or 0)
            if ph.p_align != 0 && ph.p_align & 0xfff != 0 {
                return Err(ElfError::BadAlign);
            }
            if ph.p_vaddr & 0xfff != 0 {
                return Err(ElfError::BadAlign);
            }
            // **W^X**: PF_W and PF_X must not both be set
            let pf = ph.p_flags;
            let w = (pf & PF_W) != 0;
            let x = (pf & PF_X) != 0;
            if w && x {
                return Err(ElfError::WAndX);
            }
        }
    }
    Ok(ElfMeta {
        entry: hdr.e_entry,
        phoff: hdr.e_phoff,
        phentsz: hdr.e_phentsize,
        phnum: hdr.e_phnum,
    })
}