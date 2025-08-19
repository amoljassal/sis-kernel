//! Kernel modules for tasks, scheduling, syscalls and devices.
//!
//! The kernel modules implement high level functionality on top of
//! the architecture specific code.  These modules are responsible
//! for creating and managing tasks, dispatching system calls,
//! scanning PCI devices and logging via the serial port.

pub mod affinity;
pub mod boot;
pub mod hal;
#[cfg(feature = "ipc")]
pub mod caps;
pub mod current;
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "smp")]
pub mod mm;
pub mod pci;
#[cfg(feature = "scheduler")]
pub mod sched_preempt;
pub mod scheduler;
pub mod serial;
#[cfg(feature = "scheduler")]
pub mod simple_scheduler;
#[cfg(feature = "smp")]
pub mod smp_scheduler;
pub mod spawn;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod task_table;
pub mod validation;
pub mod ai_syscalls;
pub mod memory;
pub mod vdso_manager;
pub mod types;
pub mod vfio;
pub mod vfio_stubs;
#[cfg(feature = "scheduler")]
pub mod waitqueue;
#[cfg(all(feature = "smp", feature = "ipc"))]
pub mod xcpu_ipc;
#[cfg(feature = "smp")]
pub mod xcpu_mbox;

#[cfg(feature = "userland")]
pub mod user;

#[cfg(not(feature = "userland"))]
pub mod vfs;

#[cfg(not(feature = "userland"))]
pub mod initfs;

// AI-Native Kernel Subsystem
pub mod ai;

// Provide stable re-exports for callers
pub use pci::read_id;
pub use pci::{cfg_read32, cfg_write32, find_first_e1000, PciId};
pub use types::Bdf;
