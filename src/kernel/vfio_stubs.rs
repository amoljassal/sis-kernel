#![allow(dead_code)]

#[cfg(not(feature = "vfio"))]
pub mod stubs {
    pub fn vfio_dump_hist() { /* no-op for CI lint */
    }
}

#[cfg(feature = "vfio")]
pub mod stubs {
    pub fn vfio_dump_hist() {
        // If you have an actual vfio_dump_hist in vfio module, call it here
        // For now, just a no-op to prevent compilation errors
    }
}
