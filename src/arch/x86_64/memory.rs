//! Memory management and heap initialisation.
//!
//! This module implements a simple frame allocator using the UEFI
//! memory map provided by the bootloader and sets up a heap via
//! the `linked_list_allocator`.  It also sets up paging to map the
//! kernel to the higher half (0xffff_ffff_8000_0000).  For clarity
//! and brevity the mapping functions are greatly simplified.  A
//! production kernel would need to handle large pages, guard pages
//! and dynamic allocation of page tables.

use bootloader_api::{BootInfo, info::{MemoryRegionKind, MemoryRegion}};
use x86_64::{structures::paging::{Page, PhysFrame, mapper::{MapperAllSizes, MapToError}, Mapper, PageTable, Size4KiB, FrameAllocator, OffsetPageTable, PageTableFlags}, VirtAddr, PhysAddr};
use linked_list_allocator::LockedHeap;
use core::ptr::NonNull;
use spin::Once;

/// Global heap allocator.  This is a `LockedHeap` protected by a spin
/// lock.  It must be initialised before use via [`init_heap`].
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Size of the heap (in bytes).  In a production kernel this would
/// likely be much larger and configurable.
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

static PAGE_TABLE: Once<OffsetPageTable<'static>> = Once::new();

/// Initialise memory management using the provided BootInfo.  This
/// function sets up an offset page table from the physical memory
/// offset, initialises the frame allocator from the memory map and
/// then sets up the heap in virtual memory.
pub unsafe fn init(boot_info: &BootInfo) {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into());
    let mapper = init_offset_page_table(phys_offset);
    let mut frame_alloc = BootInfoFrameAllocator::init(&boot_info.memory_regions);
    init_heap(&mapper, &mut frame_alloc).expect("Heap initialisation failed");
    PAGE_TABLE.call_once(|| mapper);
}

/// Initialise the offset page table.
unsafe fn init_offset_page_table(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let table_ptr: *mut PageTable = virt.as_mut_ptr();
    OffsetPageTable::new(&mut *table_ptr, phys_offset)
}

/// Frame allocator that returns usable frames from the UEFI memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a frame allocator from the given memory map.
    pub fn init(memory_map: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator { memory_map, next: 0 }
    }
}

impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        while self.next < self.memory_map.len() {
            let region = &self.memory_map[self.next];
            let start = region.start_addr();
            let end = region.end_addr();
            let kind = region.kind;
            // Skip non-usable regions
            if kind == MemoryRegionKind::Usable {
                for frame in (start..end).step_by(4096) {
                    let phys_frame = PhysFrame::containing_address(PhysAddr::new(frame));
                    self.next += 1;
                    return Some(phys_frame);
                }
            }
            self.next += 1;
        }
        None
    }
}

/// Initialise the heap by mapping a contiguous region of virtual
/// memory and informing the allocator about it.
fn init_heap(mapper: &OffsetPageTable<'static>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    // Choose a region for the heap.  We place it at the end of the
    // kernel's higher half (0xFFFF_FFC0_0000_0000) for demonstration.
    let heap_start = VirtAddr::new(0xFFFF_FFC0_0000_0000);
    let heap_end   = heap_start + HEAP_SIZE - 1;
    let start_page = Page::containing_address(heap_start);
    let end_page   = Page::containing_address(heap_end);
    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    unsafe {
        ALLOCATOR.lock().init(heap_start.as_u64() as usize, HEAP_SIZE);
    }
    Ok(())
}

/// Obtain a reference to the global page table.  Used for advanced
/// mapping operations.  Panics if the page table has not been
/// initialised.
pub fn mapper() -> &'static OffsetPageTable<'static> {
    PAGE_TABLE.get().expect("Page table not initialised")
}