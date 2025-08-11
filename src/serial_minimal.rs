//! Minimal serial module for firewall mode
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn init() {
    // Initialize serial - already done in lazy_static
}

pub fn write_str(s: &str) {
    let mut serial = SERIAL1.lock();
    for byte in s.bytes() {
        unsafe {
            serial.send(byte);
        }
    }
}

pub fn write_hex64(val: u64) {
    write_str("0x");
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xF;
        let ch = if nibble < 10 {
            b'0' + nibble as u8
        } else {
            b'A' + (nibble - 10) as u8
        };
        let mut serial = SERIAL1.lock();
        unsafe { serial.send(ch); }
    }
}

pub fn write_hex8(val: u8) {
    write_str("0x");
    for i in (0..2).rev() {
        let nibble = (val >> (i * 4)) & 0xF;
        let ch = if nibble < 10 {
            b'0' + nibble as u8
        } else {
            b'A' + (nibble - 10) as u8
        };
        let mut serial = SERIAL1.lock();
        unsafe { serial.send(ch); }
    }
}