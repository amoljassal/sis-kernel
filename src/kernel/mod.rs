//! Kernel modules for tasks, scheduling, syscalls and devices.
//!
//! The kernel modules implement high level functionality on top of
//! the architecture specific code.  These modules are responsible
//! for creating and managing tasks, dispatching system calls,
//! scanning PCI devices and logging via the serial port.

pub mod serial;
pub mod task;
pub mod scheduler;
#[cfg(feature = "smp")]
pub mod smp_scheduler;
#[cfg(all(feature = "smp", feature = "ipc"))]
pub mod xcpu_ipc;
pub mod syscall;
pub mod pci;
pub mod affinity;
pub mod vfio;
#[cfg(feature = "ipc")]
pub mod caps;
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "scheduler")]
pub mod waitqueue;

#[cfg(feature = "userland")]
pub mod user;

#[cfg(not(feature = "userland"))]
pub mod vfs;

#[cfg(not(feature = "userland"))]
pub mod initfs;