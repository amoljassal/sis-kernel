//! Global Descriptor Table (GDT) and Task State Segment (TSS).
//!
//! The GDT defines the memory segments used by the CPU.  We set up
//! separate code and data segments for ring 0 and ring 3 to support
//! user space execution.  A TSS is required for handling double
//! faults and for enabling the `syscall`/`sysret` instructions.

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Allocate IST stack for double fault
const IST_STACK_SIZE: usize = 4096 * 5;
static mut DOUBLE_FAULT_STACK: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE];

/// Indexes into the GDT for each segment.
#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub code_ring0: SegmentSelector,
    pub data_ring0: SegmentSelector,
    pub code_ring3: SegmentSelector,
    pub data_ring3: SegmentSelector,
    pub tss: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        let stack_start = VirtAddr::from_ptr(unsafe { DOUBLE_FAULT_STACK.as_ptr() });
        let stack_end = stack_start + IST_STACK_SIZE;
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_ring0 = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_ring0 = gdt.add_entry(Descriptor::kernel_data_segment());
        let code_ring3 = gdt.add_entry(Descriptor::user_code_segment());
        let data_ring3 = gdt.add_entry(Descriptor::user_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { 
            code_ring0, 
            data_ring0, 
            code_ring3, 
            data_ring3, 
            tss: tss_selector 
        })
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_ring0);
        load_tss(GDT.1.tss);
    }
}

/// Returns the selectors after GDT initialisation.  Used by other
/// modules to enter user space via `syscall`/`sysret`.
pub fn selectors() -> &'static Selectors {
    &GDT.1
}