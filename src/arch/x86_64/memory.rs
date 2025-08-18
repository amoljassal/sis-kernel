// src/arch/x86_64/memory.rs
#![allow(dead_code)]

use bootloader_api::BootInfo;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

use crate::kernel::serial;
use core::ptr::NonNull;
use linked_list_allocator::LockedHeap;

/// Global boot-time mapper once initialized.
static mut BOOT_MAPPER: Option<OffsetPageTable<'static>> = None;

/// SAFETY: caller must ensure `phys_offset` corresponds to a valid linear mapping of physical memory.
unsafe fn init_offset_page_table(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    let (frame, _) = Cr3::read();
    let phys = frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let l4: *mut PageTable = virt.as_mut_ptr();
    OffsetPageTable::new(&mut *l4, phys_offset)
}

/// Initialize paging mapper from BootInfo:
/// 1) Prefer physical_memory_offset (bootloader 0.11.x + Mapping::Dynamic)
/// 2) (Optional) If you later enable recursive mapping, handle recursive_index here
/// 3) Fallback: Use identity mapping (phys == virt) for bootloader 0.11.x
pub fn init_boot_mappings(boot_info: &'static BootInfo) {
    serial::write_str("[mem] boot mapping options: pmo=");
    match boot_info.physical_memory_offset.into_option() {
        Some(off) => {
            serial::write_str("some, ri=none\n");
            let off = VirtAddr::new(off);
            unsafe {
                let mapper = init_offset_page_table(off);
                BOOT_MAPPER = Some(mapper);
            }
            serial::write_str("[mem] OffsetPageTable initialized (PhysicalMemory=Dynamic)\n");
            return;
        }
        None => {
            // Keep the log shape stable:
            serial::write_str("none, ri=");
            match boot_info.recursive_index.into_option() {
                Some(ri) => {
                    serial::write_hex8(ri as u8);
                    serial::write_str("\n");
                    // If/when you enable recursive mapping mode, initialize it here.
                    // For now, we fall through to identity mapping.
                    serial::write_str(
                        "[mem] recursive mapping available but using identity fallback\n",
                    );
                }
                None => {
                    serial::write_str("none\n");
                }
            }
        }
    }

    // Fallback: Use identity mapping (works when bootloader doesn't do higher-half)
    serial::write_str("[mem] Using identity mapping fallback (phys==virt)\n");
    let identity_offset = VirtAddr::new(0x0);
    unsafe {
        let mapper = init_offset_page_table(identity_offset);
        BOOT_MAPPER = Some(mapper);
    }
    serial::write_str("[mem] OffsetPageTable initialized (Identity mapping)\n");
}

/// Borrow the mapper (panic if not initialized).
pub fn mapper() -> &'static mut OffsetPageTable<'static> {
    unsafe { BOOT_MAPPER.as_mut().expect("BOOT_MAPPER not initialized") }
}

/// Minimal frame allocator stub wired to BootInfo; replace with your real allocator.
pub struct BootInfoFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Plug in your frame allocator. For tests that don't map pages, returning None is fine.
        None
    }
}

/// Global heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Stub implementations for missing functions - these would be implemented properly in full kernel
pub fn map_user_page(virt_addr: VirtAddr) -> Result<(), &'static str> {
    let _ = virt_addr;
    Ok(()) // Stub for tests
}
