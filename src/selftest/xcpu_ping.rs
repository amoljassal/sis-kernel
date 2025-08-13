//! Phase 6C selftest: cross-CPU mailbox ping/ack
#![allow(dead_code)]

#[cfg(feature = "smp")]
pub fn run() {
    use crate::kernel::serial;
    use crate::qemu;
    use crate::arch::x86_64::topology;
    use crate::arch::x86_64::percpu_clean as percpu;
    use crate::kernel::xcpu_mbox;

    serial::write_str("[xcpu] ping test start\n");
    let cpus = topology::online_cpus();
    if cpus.len() < 2 {
        serial::write_str("[xcpu] single CPU, skipping\n");
        unsafe { qemu::exit_skip(0x50) }
    }

    // Pick a partner CPU != this CPU
    let me = percpu::this().lapic_id;
    let peer_apic = cpus.iter().copied().find(|id| *id != me).unwrap_or(me);

    // Agreement: msg hi32=tag, lo32=seq
    const TAG_PING: u32 = 0xC0FFEE01;
    const TAG_ACK:  u32 = 0xC0FFEEA1;

    // Set up a tiny handler in-thread: we poll recv() and ack pings
    let mut acks = 0u32;
    let mut pings_seen = 0u32;

    // Fire 1,000 pings to peer, peer will ack by sending back ACK(seq)
    for seq in 0..1000u32 {
        let msg = ((TAG_PING as u64) << 32) | (seq as u64);
        if let Err(e) = xcpu_mbox::send(peer_apic, msg) {
            serial::write_str("[xcpu] send err="); serial::write_hex64((-e) as u64); serial::write_str("\n");
            unsafe { qemu::exit_fail(0x6C) }
        }
        // Opportunistically drain our inbox and ack any pings to us
        for _ in 0..8 {
            if let Some(m) = xcpu_mbox::try_recv() {
                let tag = (m >> 32) as u32;
                let seq = m as u32;
                if tag == TAG_PING {
                    pings_seen = pings_seen.wrapping_add(1);
                    let ack = ((TAG_ACK as u64) << 32) | (seq as u64);
                    let _ = xcpu_mbox::send(peer_apic, ack);
                } else if tag == TAG_ACK {
                    acks = acks.wrapping_add(1);
                }
            }
        }
    }

    // Busy-wait briefly to collect any stragglers
    for _ in 0..10_000 {
        if acks >= 1000 { break; }
        if let Some(m) = xcpu_mbox::try_recv() {
            let tag = (m >> 32) as u32;
            if tag == TAG_ACK { acks = acks.wrapping_add(1); }
        }
        // small pause (port I/O yield), if you have a udelay use it
        unsafe { core::arch::x86_64::_mm_pause(); }
    }

    serial::write_str("[xcpu] acks="); serial::write_hex(acks);
    serial::write_str(" pings_seen="); serial::write_hex(pings_seen);
    serial::write_str("\n");
    if acks == 1000 {
        unsafe { qemu::exit_ok() }
    } else {
        unsafe { qemu::exit_fail(0x6D) }
    }
}

#[cfg(not(feature = "smp"))]
pub fn run() {
    use crate::kernel::{serial, qemu};
    serial::write_str("[xcpu] smp disabled, skipping\n");
    unsafe { qemu::exit_skip(0x50) }
}