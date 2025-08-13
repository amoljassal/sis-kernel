//! Kernel modules for tasks, scheduling, syscalls and devices.
//!
//! The kernel modules implement high level functionality on top of
//! the architecture specific code.  These modules are responsible
//! for creating and managing tasks, dispatching system calls,
//! scanning PCI devices and logging via the serial port.

pub mod affinity;
#[cfg(feature = "ipc")]
pub mod caps;
pub mod current;
#[cfg(feature = "ipc")]
pub mod ipc;
pub mod pci;
pub mod scheduler;
pub mod serial;
#[cfg(feature = "scheduler")]
pub mod simple_scheduler;
#[cfg(feature = "smp")]
pub mod smp_scheduler;
pub mod spawn;
pub mod syscall;
pub mod task;
pub mod task_table;
pub mod types;
pub mod vfio;
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
