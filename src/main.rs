#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

/*
 * main.rs – Entry point for the Sovereign Interface System (SIS) kernel.
 *
 * This file ties together the architecture‑specific setup and the core
 * functionality of the kernel.  It makes use of the `bootloader`
 * crate to provide a UEFI compliant entry point and prints a door
 * glyph before transferring control to this function.  The kernel
 * initialises descriptor tables, paging, heap allocation, the
 * interrupt system, scheduler, tasks and PCI/IOMMU stubs.  It then
 * enables interrupts and enters a low‑power loop while the
 * scheduler dispatches tasks on timer interrupts.
 */

extern crate alloc;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

mod arch;
mod kernel;

use arch::x86_64 as arch_x86;
use kernel::{pci, scheduler, serial, syscall, task};

/// Kernel entry point invoked by the bootloader.  The `BootInfo`
/// structure contains a memory map and other information provided
/// by UEFI.  We use this to initialise the heap and map the kernel
/// to the higher half.
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialise serial logging first so we can print messages during
    // early boot.  Use COM1 at 115200 baud via the uart_16550 crate.
    serial::init();
    serial::write_str("Duality kernel (Rust) start\n");

    // Set up the Global Descriptor Table and Task State Segment.
    arch_x86::gdt::init();

    // Set up the Interrupt Descriptor Table and install default
    // handlers for exceptions, timer interrupts and syscalls.
    arch_x86::idt::init();

    // Configure paging and map the kernel into the higher half.  Use
    // the UEFI memory map to mark free pages and initialise the
    // frame allocator.  We then initialise the heap using a simple
    // linked list allocator backed by the available memory.
    unsafe { arch_x86::memory::init(boot_info) };

    // Initialise the syscall table and routing logic.
    syscall::init();

    // Initialise the scheduler and create the parent tasks.
    scheduler::init();
    let philo = task::Task::new(task::Role::Philosophy, philosophy_parent);
    let tech  = task::Task::new(task::Role::Technical, technical_parent);
    scheduler::add_parent(philo);
    scheduler::add_parent(tech);

    // Initialise the Programmable Interval Timer to 100 Hz.  Each
    // timer interrupt will cause the scheduler to advance tasks and
    // perform context switching.
    arch_x86::pit::init(100);

    // Scan the PCI bus for GPUs and set up IOMMU passthrough.  The
    // PCI module logs any display controllers and assigns them to
    // parent tasks based on their index (GPU0 → Philosophy,
    // GPU1 → Technical).  In a future version this will set up
    // VFIO/IOMMU mapping.
    pci::init();

    // Enable interrupts now that all handlers are installed.
    arch_x86::interrupts::enable();
    serial::write_str("Initialisation complete.  Entering idle loop...\n");

    // Idle loop.  The `hlt` instruction reduces power consumption
    // between interrupts.  The scheduler is driven from the timer
    // interrupt handler and will perform context switches as needed.
    loop {
        arch_x86::cpu::halt();
    }
}

/// Parent task for everyday interactions.  It runs as a privileged
/// parent on its own core/GPU and processes directives that do not
/// require technical analysis.  For now it logs a message and
/// loops.  In the future it will create child tasks to handle
/// specific requests.
fn philosophy_parent() {
    serial::write_str("[Philosophy] Hello from the Philosophy parent!\n");
    loop { arch_x86::cpu::pause(); }
}

/// Parent task for technical tasks such as coding, security and
/// analysis.  It logs a message and loops.  Future versions will
/// use GPU2 via IOMMU and spawn child modules for compute tasks.
fn technical_parent() {
    serial::write_str("[Technical] Hello from the Technical parent!\n");
    loop { arch_x86::cpu::pause(); }
}

/// Panic handler.  When a panic occurs we print the panic
/// information to the serial port and then halt the CPU.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::write_str("[panic] Kernel panic: ");
    if let Some(args) = info.message() {
        // We cannot use formatting macros in `no_std` directly.  For now
        // we simply note that a panic occurred.
        let _ = args;
    }
    loop {
        arch_x86::cpu::halt();
    }
}

/// Allocation error handler required when using `alloc` in a
/// `no_std` environment.  We log the failure and halt.
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    serial::write_str("[alloc] Allocation error\n");
    loop {
        arch_x86::cpu::halt();
    }
}