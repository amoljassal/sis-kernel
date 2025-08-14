//! Selftest modules

#[cfg(feature = "affinity")]
pub mod smp_affinity;

#[cfg(selftest_IPC_XCPU_PING)]
pub mod xcpu_ping;

#[cfg(all(feature="smp", feature="selftests"))]
pub mod proc_stats;

#[cfg(all(feature="scheduler", feature="selftests"))]
pub mod sched_preempt_rr;
#[cfg(all(feature="scheduler", feature="selftests"))]
pub mod sched_fair_meter;
