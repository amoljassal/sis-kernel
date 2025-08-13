//! Selftest modules

#[cfg(feature = "affinity")]
pub mod smp_affinity;

#[cfg(selftest_IPC_XCPU_PING)]
pub mod xcpu_ping;
