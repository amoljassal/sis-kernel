//! Per-task address spaces (CR3) + isolation selftest.
//! Feature-gated by `per-task-mm`.
//!
//! Minimal mechanism:
//! - Create a new PML4, clone the kernel higher-half from current PML4
//! - Map a user page at a fixed VA
//! - Flip CR3 during "context switch"
//! Validation:
//! - Build two address spaces (A,B), map the same VA to different frames/values
//! - Load CR3=A and read; then CR3=B and read → values differ → isolation proven

use crate::kernel::serial;
use core::ptr::{read_volatile, write_volatile};
use x86_64::structures::paging::mapper::{MapToError, Mapper, MapperFlush};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Page, PageTable, PageTableFlags as Flags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::arch::x86_64::io::qemu_exit;
use crate::arch::x86_64::memory;

const USER_TEST_VA: u64 = 0x0000_0000_4000_0000;

/// Allocate a zeroed page table frame and return its physical address.
fn alloc_zeroed_pagetable() -> PhysFrame {
    let p = memory::alloc_frame().expect("no frame for pml4");
    unsafe {
        // Use identity mapping for test simplicity (common in QEMU)
        let va = memory::phys_to_tmp_virt(p.start_address());
        let pt_ptr = va.as_mut_ptr::<PageTable>();
        core::ptr::write_bytes(pt_ptr, 0, 1);
    }
    p
}

/// Clone higher-half kernel PML4 entries from current CR3 into `dst_pml4`.
unsafe fn clone_kernel_pml4(dst_pml4_pa: PhysFrame) {
    let (curr_pml4, _) = Cr3::read();
    let src_va = memory::phys_to_tmp_virt(curr_pml4.start_address());
    let dst_va = memory::phys_to_tmp_virt(dst_pml4_pa.start_address());
    let src = src_va.as_ptr::<PageTable>();
    let dst = dst_va.as_mut_ptr::<PageTable>();
    // Kernel half is upper 256 entries [256..512)
    for i in 256..512 {
        (&mut *dst)[i] = (&*src)[i].clone();
    }
}

/// Create a new address space (CR3) with kernel half mapped and empty user half.
pub fn create_address_space() -> PhysFrame {
    let pml4 = alloc_zeroed_pagetable();
    unsafe {
        clone_kernel_pml4(pml4);
    }
    pml4
}

/// Destroy address space: for tests, we only free the root (allocator can reclaim; children tables omitted).
pub fn destroy_address_space(_cr3: PhysFrame) {
    // Optional: walk & free. For selftest we can skip to keep code small/safe.
}

/// Map one user page in the given address space at `va`, with the given `frame` and `flags`.
pub fn map_user_in_as(
    cr3: PhysFrame,
    va: VirtAddr,
    frame: PhysFrame,
    flags: Flags,
) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
    // Switch temporarily to `cr3`, do the map with existing mapper, then switch back.
    let (old, old_flags) = Cr3::read();
    unsafe {
        Cr3::write(cr3, x86_64::registers::control::Cr3Flags::empty());
    }
    let r = unsafe {
        memory::map_with_active_mapper(Page::<Size4KiB>::containing_address(va), frame, flags)
    };
    // Flush that page in this AS
    memory::tlb_flush(va);
    unsafe {
        Cr3::write(old, old_flags);
    }
    r
}

/// Write a u64 at user VA inside `cr3`.
fn write_u64_in_as(cr3: PhysFrame, va: VirtAddr, value: u64) {
    let (old, old_flags) = Cr3::read();
    unsafe {
        Cr3::write(cr3, x86_64::registers::control::Cr3Flags::empty());
    }
    unsafe {
        write_volatile(va.as_mut_ptr::<u64>(), value);
    }
    unsafe {
        Cr3::write(old, old_flags);
    }
}

/// Read a u64 at user VA inside `cr3`.
fn read_u64_in_as(cr3: PhysFrame, va: VirtAddr) -> u64 {
    let (old, old_flags) = Cr3::read();
    unsafe {
        Cr3::write(cr3, x86_64::registers::control::Cr3Flags::empty());
    }
    let v = unsafe { read_volatile(va.as_ptr::<u64>()) };
    unsafe {
        Cr3::write(old, old_flags);
    }
    v
}

#[cfg(selftest_AS_PER_TASK_ISOLATION)]
pub fn selftest_isolation() -> ! {
    serial::write_str("[as] create two address spaces\n");
    let as_a = create_address_space();
    let as_b = create_address_space();

    // Map same VA in both, with different frames.
    let f_a = memory::alloc_frame().expect("no frame A");
    let f_b = memory::alloc_frame().expect("no frame B");

    let va = VirtAddr::new(USER_TEST_VA);
    let flags = Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::WRITABLE;
    map_user_in_as(as_a, va, f_a, flags).expect("map A");
    map_user_in_as(as_b, va, f_b, flags).expect("map B");

    serial::write_str("[as] write A=0xaa at 0x0000000040000000\n");
    write_u64_in_as(as_a, va, 0xaa);
    serial::write_str("[as] write B=0xbb at 0x0000000040000000\n");
    write_u64_in_as(as_b, va, 0xbb);

    // Verify: reading under A sees 0xaa; under B sees 0xbb.
    let ra = read_u64_in_as(as_a, va);
    let rb = read_u64_in_as(as_b, va);
    if ra == 0xaa {
        serial::write_str("[as] verify A ok\n");
    } else {
        serial::write_str("[as] verify A FAIL\n");
        unsafe {
            qemu_exit(0x40);
        }
    }
    if rb == 0xbb {
        serial::write_str("[as] verify B ok\n");
    } else {
        serial::write_str("[as] verify B FAIL\n");
        unsafe {
            qemu_exit(0x41);
        }
    }

    serial::write_str("[as] isolation PASS\n");
    unsafe {
        qemu_exit(0x00);
    }
}
