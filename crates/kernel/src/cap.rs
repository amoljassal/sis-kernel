//! Minimal capability types and stubs for Phase 1.
//! Fast-path checks are constant-time over an opaque handle.

use core::num::NonZeroU64;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CapId(NonZeroU64);

bitflags::bitflags! {
    pub struct CapRights: u32 {
        const READ  = 0b0001;
        const WRITE = 0b0010;
        const RUN   = 0b0100;
        const ADMIN = 0b1000;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CapObjectKind { Graph, Operator, Channel, Tensor, Model }

pub struct Capability {
    pub id: CapId,
    pub kind: CapObjectKind,
    pub rights: CapRights,
}

impl CapId {
    #[inline(always)]
    pub fn new(raw: u64) -> Option<Self> { NonZeroU64::new(raw).map(CapId) }
    #[inline(always)]
    pub fn get(self) -> u64 { self.0.get() }
}

#[inline(always)]
pub fn check(cap: Capability, need: CapRights, kind: CapObjectKind) -> bool {
    cap.kind == kind && cap.rights.contains(need)
}
