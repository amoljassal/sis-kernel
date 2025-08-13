#![allow(dead_code)]
#[cfg(not(feature = "vfio"))]
pub mod stubs {
    pub fn vfio_dump_hist() { /* no-op for CI lint */
    }
}

#[cfg(feature = "vfio")]
pub mod stubs {
    pub fn vfio_dump_hist() {
        crate::kernel::vfio::dump_hist()
    }
}
