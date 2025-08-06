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
use spin::Once;

/// Indexes into the GDT for each segment.
#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub code_ring0: SegmentSelector,
    pub data_ring0: SegmentSelector,
    pub code_ring3: SegmentSelector,
    pub data_ring3: SegmentSelector,
    pub tss: SegmentSelector,
}

static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();
static mut TSS: Option<TaskStateSegment> = None;

/// Size of the stack used for double fault handling.
pub const DOUBLE_FAULT_STACK_SIZE: usize = 4096;
static mut DOUBLE_FAULT_STACK: [u8; DOUBLE_FAULT_STACK_SIZE] = [0; DOUBLE_FAULT_STACK_SIZE];

/// Index in the Interrupt Stack Table (IST) used for handling
/// double faults.  See the x86_64 architecture manuals for more
/// details on IST.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub fn init() {
    // Initialise the TSS with a known stack for double faults.  We
    // allocate the double fault stack statically to avoid using the
    // allocator before it is ready.
    let mut tss = TaskStateSegment::new();
    let df_stack_start = VirtAddr::from_ptr(unsafe { DOUBLE_FAULT_STACK.as_ptr().add(DOUBLE_FAULT_STACK_SIZE) });
    tss.interrupt_stack_table[0] = df_stack_start;
    unsafe { TSS = Some(tss); }

    // Build the GDT.  We create code and data segments for ring0 and
    // ring3.  The TSS descriptor is created using the `SegmentSelector::new`
    // method.
    let mut gdt = GlobalDescriptorTable::new();
    let code_ring0 = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_ring0 = gdt.add_entry(Descriptor::kernel_data_segment());
    let code_ring3 = gdt.add_entry(Descriptor::user_code_segment());
    let data_ring3 = gdt.add_entry(Descriptor::user_data_segment());
    let tss_sel = {
        let tss_ref = unsafe { TSS.as_ref().unwrap() };
        gdt.add_entry(Descriptor::tss_segment(tss_ref))
    };
    let selectors = Selectors {
        code_ring0,
        data_ring0,
        code_ring3,
        data_ring3,
        tss: tss_sel,
    };
    // Load the GDT and set the segment registers.  Use unsafe
    // assembly to reload CS, DS and load TSS.
    GDT.call_once(|| {
        gdt.load();
        unsafe {
            use x86_64::instructions::{segmentation, tables};
            segmentation::set_cs(selectors.code_ring0);
            segmentation::load_ds(selectors.data_ring0);
            segmentation::load_es(selectors.data_ring0);
            segmentation::load_fs(selectors.data_ring0);
            segmentation::load_gs(selectors.data_ring0);
            segmentation::load_ss(selectors.data_ring0);
            tables::load_tss(selectors.tss);
        }
        (gdt, selectors)
    });
}

/// Returns the selectors after GDT initialisation.  Used by other
/// modules to enter user space via `syscall`/`sysret`.
pub fn selectors() -> &'static Selectors {
    &GDT.get().unwrap().1
}