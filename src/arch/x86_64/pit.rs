//! Programmable Interval Timer (PIT) and PIC remapping.
//!
//! This module configures the PIT to generate periodic interrupts at a
//! fixed frequency and remaps the Programmable Interrupt Controller
//! (PIC) to avoid conflicts with CPU exceptions.  It also exposes
//! a function to acknowledge timer interrupts.

use crate::arch::x86_64::io;

const PIT_CHANNEL0_PORT: u16 = 0x40;
const PIT_COMMAND_PORT: u16 = 0x43;
const PIC_MASTER_COMMAND: u16 = 0x20;
const PIC_MASTER_DATA: u16 = 0x21;
const PIC_SLAVE_COMMAND: u16 = 0xA0;
const PIC_SLAVE_DATA: u16 = 0xA1;

/// Remap the PIC so that IRQs are delivered at vector offsets
/// starting from 32.  Without remapping, hardware interrupts could
/// collide with exceptions.
fn pic_remap() {
    unsafe {
        let mask1 = io::inb(PIC_MASTER_DATA);
        let mask2 = io::inb(PIC_SLAVE_DATA);
        io::outb(PIC_MASTER_COMMAND, 0x11);
        io::outb(PIC_SLAVE_COMMAND, 0x11);
        io::outb(PIC_MASTER_DATA, 0x20);
        io::outb(PIC_SLAVE_DATA, 0x28);
        io::outb(PIC_MASTER_DATA, 0x04);
        io::outb(PIC_SLAVE_DATA, 0x02);
        io::outb(PIC_MASTER_DATA, 0x01);
        io::outb(PIC_SLAVE_DATA, 0x01);
        // Restore saved masks
        io::outb(PIC_MASTER_DATA, mask1);
        io::outb(PIC_SLAVE_DATA, mask2);
    }
}

/// Initialise the PIT to generate interrupts at the given frequency in Hz.
pub fn init(frequency: u32) {
    pic_remap();
    let divisor: u16 = (1193180 / frequency) as u16;
    unsafe {
        io::outb(PIT_COMMAND_PORT, 0x36);
        io::outb(PIT_CHANNEL0_PORT, (divisor & 0xFF) as u8);
        io::outb(PIT_CHANNEL0_PORT, (divisor >> 8) as u8);
        // Unmask IRQ0
        let mut mask = io::inb(PIC_MASTER_DATA);
        mask &= !(1 << 0);
        io::outb(PIC_MASTER_DATA, mask);
    }
}

/// Acknowledge the timer interrupt by sending an End Of Interrupt (EOI) to the PIC.
pub fn ack() {
    unsafe {
        io::outb(PIC_MASTER_COMMAND, 0x20);
    }
}
