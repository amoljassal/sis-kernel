//! Canonical tiny integer types & helpers to stop cast churn.
#![allow(dead_code)]

/// Task ID used across the kernel (keeps your existing shape).
pub type Tid = usize;
/// Compact handle type for caps/VFIO/etc.
pub type Handle = u16;

/// PCI Bus/Device/Function are single-byte fields in config space.
pub type PciBus = u8;
pub type PciDev = u8;
pub type PciFn  = u8;

/// Bus/Device/Function triple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bdf {
    pub bus: PciBus,
    pub dev: PciDev,
    pub func: PciFn,
}
impl Bdf {
    #[inline] pub const fn new(bus: PciBus, dev: PciDev, func: PciFn) -> Self { Self { bus, dev, func } }
    #[inline] pub fn as_tuple(self) -> (PciBus, PciDev, PciFn) { (self.bus, self.dev, self.func) }
}

/// Hex helpers: print any integer type by converting once at the call site.
#[inline] pub fn as_u64<T: Into<u64>>(v: T) -> u64 { v.into() }