//! CPU and GPU affinity helpers.
//!
//! This module wraps calls into the optional `thread-affinity` crate
//! (which pins the current thread to a particular CPU core) and
//! provides stubs when that crate is unavailable.  In addition it
//! contains placeholders for GPU affinity via IOMMU/VFIO.

// Previous builds referenced a non-existent `thread-affinity` Cargo feature. Consolidate on `affinity`.
#[cfg(feature = "affinity")]
pub fn set_core_affinity(core_id: usize) -> Result<(), &'static str> {
    // Kernel-space affinity: In a real implementation this would interact with
    // the scheduler to pin the current task to a specific CPU core.
    // For now, we log the affinity request and return success.
    crate::kernel::serial::write_str("[affinity] Setting core affinity to CPU ");
    crate::kernel::serial::write_hex8(core_id as u8);
    crate::kernel::serial::write_str("\n");
    Ok(())
}

#[cfg(not(feature = "affinity"))]
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
