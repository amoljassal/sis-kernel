#![allow(dead_code)]
#[cfg(not(feature = "vfio"))]
pub mod stubs {
    pub fn vfio_dump_hist() { /* no-op for CI lint */
    }
}

#[cfg(feature = "vfio")]
pub mod stubs {
    pub fn vfio_dump_hist() {
        crate::arch::x86_64::idt::vfio_rt::dump_hist()
    }
}
