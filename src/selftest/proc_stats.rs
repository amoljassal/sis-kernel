#![cfg(all(feature = "smp", feature = "selftests"))]
//! Tiny /proc‑style dump of per‑CPU stats for validation.
use crate::arch::x86_64::percpu_clean::PerCpu;
use crate::arch::x86_64::topology;
use crate::kernel::serial;

pub fn run() {
    serial::write_str("[proc] cpu stats\n");
    for apic in topology::online_cpus() {
        let idx = crate::arch::x86_64::topology::cpu_index_from_apic(apic);
        // this() is current CPU only; for demo we just print current and indexes.
        if idx == crate::arch::x86_64::topology::cpu_index_this() {
            let pc = PerCpu::this();
            serial::write_str(" cpu");
            serial::write_dec(idx as u64);
            serial::write_str(": ticks=");
            serial::write_dec(pc.ticks.load(core::sync::atomic::Ordering::Relaxed));
            serial::write_str(" ctx_sw=");
            serial::write_dec(pc.ctx_sw.load(core::sync::atomic::Ordering::Relaxed));
            serial::write_str(" ipi_rx_resched=");
            serial::write_dec(
                pc.ipi_rx_resched
                    .load(core::sync::atomic::Ordering::Relaxed),
            );
            serial::write_str(" ipi_rx_tlb=");
            serial::write_dec(pc.ipi_rx_tlb.load(core::sync::atomic::Ordering::Relaxed));
            serial::write_str(" mbox_rx=");
            serial::write_dec(pc.mbox_rx.load(core::sync::atomic::Ordering::Relaxed));
            serial::write_str("\n");
        } else {
            // TODO: add remote read via mailbox snapshot if needed
            serial::write_str(" cpu");
            serial::write_dec(idx as u64);
            serial::write_str(": (remote)\n");
        }
    }
}
