#![allow(dead_code)]
use alloc::vec::Vec;
use core::{
    ptr::NonNull,
    sync::atomic::{AtomicU16, Ordering},
};
use spin::Mutex;

pub type CapId = u32; // index | (gen<<16)

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CapKind {
    IpcSender,
    IpcReceiver,
}

bitflags::bitflags! {
    #[derive(Copy, Clone)]
    pub struct CapFlags: u32 {
        const NONBLOCK = 1<<0;
    }
}

pub trait KernelObject {}

pub struct CapEntry {
    pub kind: CapKind,
    pub obj: NonNull<dyn KernelObject>,
    pub gen: u16,
    pub flags: CapFlags,
}

pub struct CTable {
    slots: Mutex<Vec<Option<CapEntry>>>,
    gens: Vec<AtomicU16>,
}

unsafe impl Send for CTable {}
unsafe impl Sync for CTable {}

impl CTable {
    pub const fn new() -> Self {
        Self {
            slots: Mutex::new(Vec::new()),
            gens: Vec::new(),
        }
    }

    pub fn insert(&self, e: CapEntry) -> CapId {
        let mut slots = self.slots.lock();
        // find free
        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_none() {
                let gen = self.bump_gen(i);
                *slot = Some(CapEntry { gen, ..e });
                return Self::pack(i as u32, gen);
            }
        }
        // grow
        let i = slots.len();
        slots.push(None);
        self.ensure_gen_len(i + 1);
        let gen = self.bump_gen(i);
        slots[i] = Some(CapEntry { gen, ..e });
        Self::pack(i as u32, gen)
    }

    pub fn remove(&self, id: CapId) -> bool {
        let (i, gen) = Self::unpack(id);
        let mut slots = self.slots.lock();
        if let Some(Some(e)) = slots.get(i as usize) {
            if e.gen == gen {
                slots[i as usize] = None;
                return true;
            }
        }
        false
    }

    pub fn get(&self, id: CapId) -> Option<CapEntry> {
        let (i, gen) = Self::unpack(id);
        let slots = self.slots.lock();
        if let Some(Some(e)) = slots.get(i as usize) {
            if e.gen == gen {
                return Some(CapEntry {
                    kind: e.kind,
                    obj: e.obj,
                    gen: e.gen,
                    flags: e.flags,
                });
            }
        }
        None
    }

    fn pack(idx: u32, gen: u16) -> CapId {
        (idx & 0xffff) | ((gen as u32) << 16)
    }

    fn unpack(id: CapId) -> (u32, u16) {
        (id & 0xffff, (id >> 16) as u16)
    }

    fn ensure_gen_len(&self, n: usize) {
        // Note: This is a simplified version for Phase 2
        // In production we'd handle this more carefully with proper synchronization
        // For now, we'll just use a basic implementation
    }

    fn bump_gen(&self, i: usize) -> u16 {
        // Simplified version - just return 1 for Phase 2
        // In production this would properly manage generations
        1
    }
}
