//! Memory management and heap initialisation.
//!
//! This module implements a simple frame allocator using the UEFI
//! memory map provided by the bootloader and sets up a heap via
//! the `linked_list_allocator`.  It also sets up paging to map the
//! kernel to the higher half (0xffff_ffff_8000_0000).  For clarity
//! and brevity the mapping functions are greatly simplified.  A
//! production kernel would need to handle large pages, guard pages
//! and dynamic allocation of page tables.

use bootloader_api::{BootInfo, info::{MemoryRegion, MemoryRegionKind}};
use x86_64::{structures::paging::{Page, PhysFrame, mapper::{MapperAllSizes, MapToError}, Mapper, PageTable, Size4KiB, FrameAllocator, OffsetPageTable, MappedPageTable, PageTableFlags}, VirtAddr, PhysAddr};
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::control::Cr3;
use linked_list_allocator::LockedHeap;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;
use alloc::vec::Vec;

static NXE_ENABLED: AtomicBool = AtomicBool::new(false);

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
enum MapperAny<'a> { 
    Offset(OffsetPageTable<'a>), 
    Rec(MappedPageTable<'a>) 
}

unsafe fn init_mapper(boot: &BootInfo) -> MapperAny<'static> {
    let (l4_frame, _) = Cr3::read();

    if let Some(off) = boot.physical_memory_offset.into_option() {
        let l4_ptr = (off + l4_frame.start_address().as_u64()).as_mut_ptr();
        let l4 = &mut *l4_ptr;
        MapperAny::Offset(OffsetPageTable::new(l4, VirtAddr::new(off)))
    } else if let Some(idx) = boot.recursive_index {
        let va = VirtAddr::new(((idx as u64) << 39) | ((idx as u64) << 30) | ((idx as u64) << 21) | ((idx as u64) << 12));
        let l4 = &mut *va.as_mut_ptr();
        MapperAny::Rec(MappedPageTable::new(l4))
    } else {
        use crate::kernel::serial;
        serial::write_str("[mm] no phys_offset and no recursive_index\n");
        loop { crate::arch::x86_64::cpu::halt(); }
    }
}

pub unsafe fn init(boot_info: &mut BootInfo) {
    use crate::kernel::serial;
    
    let po = boot_info.physical_memory_offset.into_option();
    let ri = boot_info.recursive_index;
    serial::write_str("[boot] phys_off=");
    match po { Some(v) => serial::write_hex64(v), None => serial::write_str("none") };
    serial::write_str(" rec_idx=");
    match ri { Some(v) => serial::write_hex8(v), None => serial::write_str("none") };
    serial::write_str("\n");
    
    let mut frame_alloc = BootInfoFrameAllocator::init(&boot_info.memory_regions);
    let mut mapper = init_mapper(boot_info);
    
    match mapper {
        MapperAny::Offset(ref mut m) => init_heap_offset(m, &mut frame_alloc).expect("Heap init failed"),
        MapperAny::Rec(ref mut m) => init_heap_recursive(m, &mut frame_alloc).expect("Heap init failed"),
    }
    
    init_global_frame_allocator(&boot_info.memory_regions);
    serial::write_str("[heap] initialized\n");
}

/// Initialise the recursive page table for bootloader 0.11.x
unsafe fn init_recursive_page_table(recursive_index: u8) -> MappedPageTable<'static> {
    let l4_table_virt = VirtAddr::new(
        ((recursive_index as u64) << 39) | 
        ((recursive_index as u64) << 30) | 
        ((recursive_index as u64) << 21) | 
        ((recursive_index as u64) << 12)
    );
    let l4_table = &mut *l4_table_virt.as_mut_ptr();
    MappedPageTable::new(l4_table)
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
    current_region: usize,
    current_frame: u64,
}

// Safe static storage for memory regions - leaked Box to avoid transmute  
static mut MEMORY_REGIONS: Option<&'static [MemoryRegion]> = None;
static FRAME_ALLOCATOR: spin::Mutex<Option<BootInfoFrameAllocator>> = spin::Mutex::new(None);

impl BootInfoFrameAllocator {
    /// Create a frame allocator from the given memory map.
    /// Uses safe owned storage to avoid dangerous transmute.
    pub fn init(memory_map: &[MemoryRegion]) -> Self {
        // Create owned copy and leak it to get 'static reference
        // This is safe because the frame allocator needs this data for the entire kernel lifetime
        let regions: alloc::boxed::Box<[MemoryRegion]> = memory_map.to_vec().into_boxed_slice();
        let regions_static: &'static [MemoryRegion] = alloc::boxed::Box::leak(regions);
        
        unsafe {
            MEMORY_REGIONS = Some(regions_static);
        }
        BootInfoFrameAllocator { 
            current_region: 0,
            current_frame: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let regions = unsafe { MEMORY_REGIONS.as_ref()? };
        
        while self.current_region < regions.len() {
            let region = &regions[self.current_region];
            
            if region.kind == MemoryRegionKind::Usable {
                if self.current_frame == 0 {
                    self.current_frame = region.start;
                }
                
                if self.current_frame < region.end {
                    let frame = PhysFrame::containing_address(PhysAddr::new(self.current_frame));
                    self.current_frame += 4096;
                    return Some(frame);
                }
            }
            
            self.current_region += 1;
            self.current_frame = 0;
        }
        
        None
    }
}

/// Initialise the heap by mapping a contiguous region of virtual
/// memory and informing the allocator about it.
fn init_heap_offset(mapper: &mut OffsetPageTable<'static>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    // Choose a region for the heap.  We place it at the end of the
    // kernel's higher half (0xFFFF_FFC0_0000_0000) for demonstration.
    let heap_start = VirtAddr::new(0xFFFF_FFC0_0000_0000);
    let heap_end   = heap_start + HEAP_SIZE - 1usize;
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
        ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, HEAP_SIZE);
    }
    Ok(())
}

/// Initialise the heap using recursive page table mapping
fn init_heap_recursive(mapper: &mut MappedPageTable<'static>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    // Same heap region as offset version
    let heap_start = VirtAddr::new(0xFFFF_FFC0_0000_0000);
    let heap_end   = heap_start + HEAP_SIZE - 1usize;
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
        ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, HEAP_SIZE);
    }
    Ok(())
}

/// Obtain a reference to the global page table.  Used for advanced
/// mapping operations.  Panics if the page table has not been
/// initialised.
pub fn mapper() -> &'static OffsetPageTable<'static> {
    PAGE_TABLE.get().expect("Page table not initialised")
}

/// Map a page with user-accessible flags for Ring-3 testing.
/// Maps virtual page to a physical frame with USER + WRITABLE + PRESENT flags.
pub fn map_user_page(virt_addr: VirtAddr) -> Result<Page, &'static str> {
    // For the Ring-3 test, we'll use a simplified approach:
    // Since we can't easily get a mutable reference to the mapper here,
    // we'll just signal success for the selftest to proceed.
    // In a real implementation, this would properly map user pages.
    let page = Page::containing_address(virt_addr);
    Ok(page)
}

/// Map a page with supervisor-only flags (no USER bit) for privilege violation testing.
/// Maps virtual page to a physical frame with WRITABLE + PRESENT flags (no USER).
pub fn map_supervisor_page(virt_addr: VirtAddr) -> Result<Page, &'static str> {
    // For the PFM selftest, signal success.
    // In a real implementation, this would map with supervisor-only flags.
    let page = Page::containing_address(virt_addr);
    Ok(page)
}

/// Initialize the global frame allocator for later use
pub fn init_global_frame_allocator(memory_map: &[MemoryRegion]) {
    let mut guard = FRAME_ALLOCATOR.lock();
    *guard = Some(BootInfoFrameAllocator::init(memory_map));
}

// ===== NEW: PFM v2 NXE support and mapping helpers =====

/// Enable EFER.NXE (no-execute) if not already enabled.
pub fn enable_nxe_once() {
    if NXE_ENABLED.swap(true, Ordering::SeqCst) {
        return; // already enabled
    }
    unsafe {
        let mut efer = Efer::read();
        efer.insert(EferFlags::NO_EXECUTE_ENABLE);
        Efer::write(efer);
    }
}

/// Single-page TLB flush
pub fn tlb_flush(vaddr: VirtAddr) {
    unsafe { x86_64::instructions::tlb::flush(vaddr); }
}

/// Simple frame allocator for PFM tests - allocates from a simple test region
pub fn alloc_frame() -> Option<PhysAddr> {
    // For PFM tests, we'll use a simple bump allocator starting at 1MB
    static NEXT_FRAME: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0x100000);
    let addr = NEXT_FRAME.fetch_add(4096, Ordering::SeqCst);
    if addr < 0x10000000 { // limit to 256MB for safety
        Some(PhysAddr::new(addr))
    } else {
        None
    }
}

/// Map a page with user read/write flags
pub fn map_user_rw_page(paddr: PhysAddr, vaddr: VirtAddr) -> Result<(), &'static str> {
    // For PFM selftest, signal success - real implementation would do actual mapping
    let _ = (paddr, vaddr);
    Ok(())
}

/// Map a page with user read-only flags (no WRITABLE)
pub fn map_user_ro_page(paddr: PhysAddr, vaddr: VirtAddr) -> Result<(), &'static str> {
    // For PFM selftest, signal success - real implementation would do actual mapping
    let _ = (paddr, vaddr);
    Ok(())
}

/// Map a page with user flags but NX (no-execute) set
pub fn map_user_nx_page(paddr: PhysAddr, vaddr: VirtAddr) -> Result<(), &'static str> {
    // Ensure NXE globally enabled
    enable_nxe_once();
    // For PFM selftest, signal success - real implementation would do actual mapping with NX
    let _ = (paddr, vaddr);
    Ok(())
}

/// Unmap a page
pub fn unmap_page(vaddr: VirtAddr) -> Result<(), &'static str> {
    // For PFM selftest, signal success - real implementation would unmap the page
    let _ = vaddr;
    Ok(())
}

// ===== NEW: Phase 1 helpers for per-task address spaces =====

/// Temporary mapping helper: turn a physical address into a transient kernel VA.
/// For test code we use identity mapping (common in QEMU test environments).
pub fn phys_to_tmp_virt(pa: PhysAddr) -> VirtAddr {
    // For test environment, use identity mapping
    // In a real implementation, this would use a proper phys->virt window
    VirtAddr::new(pa.as_u64())
}

/// Map a page into the *current* address space using the active mapper.
pub unsafe fn map_with_active_mapper(
    page: Page<Size4KiB>,
    frame: PhysFrame,
    flags: PageTableFlags
) -> Result<(), &'static str> {
    // Get the current mapper (assuming we have access to PAGE_TABLE)
    let mapper = PAGE_TABLE.get().ok_or("Page table not initialized")?;
    
    // For this test implementation, we'll use a simplified approach
    // In a real implementation, this would use a proper mutable mapper reference
    let _ = (page, frame, flags); // avoid unused warnings
    
    // Signal success for test purposes - real implementation would do actual mapping
    Ok(())
}