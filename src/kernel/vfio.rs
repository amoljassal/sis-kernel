//! VFIO/IOMMU integration stubs.
//!
//! This module provides placeholder functions for interacting with
//! the VFIO subsystem to perform device passthrough.  In a real
//! implementation these functions would query IOMMU groups, open
//! `/dev/vfio/vfio` and individual group devices, map BARs into
//! kernel space and assign them to tasks.  The `vfio-bindings`
//! crate could be used to access the necessary ioctl constants.

#[derive(Debug)]
pub enum VfioError {
    NotSupported,
    IoError,
    InvalidDevice,
}

/// Initialise the VFIO subsystem.  Returns `Ok(())` if VFIO is
/// available and initialised, `Err` otherwise.  This stub simply
/// returns an error when the `vfio` feature is not enabled.
pub fn init() -> Result<(), VfioError> {
    #[cfg(feature = "vfio")]
    {
        // In a real implementation we would open /dev/vfio/vfio and
        // call the VFIO_GET_API_VERSION ioctl to initialise.  The
        // vfio-bindings crate would provide the definitions.  We
        // return Ok to indicate success.
        Ok(())
    }
    #[cfg(not(feature = "vfio"))]
    {
        Err(VfioError::NotSupported)
    }
}

/// Map a PCI device into an IOMMU group and return a handle.  In a
/// production kernel this would involve VFIO ioctls and may
/// allocate address space.  The device is identified by its bus,
/// device and function numbers.
pub fn map_device(_bus: u8, _device: u8, _function: u8) -> Result<usize, VfioError> {
    // Not implemented: return a dummy handle
    Err(VfioError::NotSupported)
}

/// Unmap a previously mapped device handle.  This would close the
/// VFIO file descriptors and free resources.
pub fn unmap_device(_handle: usize) -> Result<(), VfioError> {
    Err(VfioError::NotSupported)
}