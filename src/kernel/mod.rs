//! Kernel modules for tasks, scheduling, syscalls and devices.
//!
//! The kernel modules implement high level functionality on top of
//! the architecture specific code.  These modules are responsible
//! for creating and managing tasks, dispatching system calls,
//! scanning PCI devices and logging via the serial port.

pub mod serial;
pub mod task;
pub mod scheduler;
pub mod syscall;
pub mod pci;
pub mod affinity;
pub mod vfio;