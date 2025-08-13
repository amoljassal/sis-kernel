//! CPU and GPU affinity helpers.
//!
//! This module wraps calls into the optional `thread-affinity` crate
//! (which pins the current thread to a particular CPU core) and
//! provides stubs when that crate is unavailable.  In addition it
//! contains placeholders for GPU affinity via IOMMU/VFIO.

#[cfg(feature = "thread-affinity")]
pub fn set_core_affinity(core_id: usize) -> Result<(), &'static str> {
    // Use the thread_affinity crate to pin the current thread to a core.
    thread_affinity::set_thread_affinity(&[core_id]).map_err(|_| "Failed to set thread affinity")
}

#[cfg(not(feature = "thread-affinity"))]
pub fn set_core_affinity(_core_id: usize) -> Result<(), &'static str> {
    // No affinity support compiled in; pretend success.
    Ok(())
}

/// Placeholder for setting GPU affinity via VFIO/IOMMU.  In a full
/// implementation this function would use the `vfio-bindings` crate
/// to bind a device to a specific task or address space.  Here we
/// just log the assignment.
pub fn set_gpu_affinity(_gpu_id: usize) -> Result<(), &'static str> {
    // Not implemented: would interact with VFIO/IOMMU to assign GPU.
    Ok(())
}
