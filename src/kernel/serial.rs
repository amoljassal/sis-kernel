//! Serial port logging.
//!
//! Uses the `uart_16550` crate to interface with a standard 16550
//! compatible UART.  The kernel uses COM1 at 115200 baud for
//! diagnostics and logging.  Functions are provided to write
//! characters, strings and byte buffers to the serial port.

use uart_16550::SerialPort;
use spin::Mutex;

// Safe global access to the serial port.  We protect the port with
// a spinlock to allow concurrent writes from different contexts.
lazy_static::lazy_static! {
    static ref SERIAL: Mutex<SerialPort> = {
        // The base address for COM1 is 0x3F8.
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Initialise the serial port.  This function must be called before
/// any writes are performed.  Subsequent calls have no effect.
pub fn init() {
    // The lazy_static ensures that the SerialPort is initialised on
    // first access.  Calling this function forces that initialisation.
    let _ = SERIAL.lock();
    // Immediate feedback to confirm serial is working
    write_str("[SERIAL] Serial port initialized.\n");
}

/// Write a single byte to the serial port.  Blocks until the UART
/// is ready to accept a new character.
pub fn write_char(c: u8) {
    let mut serial = SERIAL.lock();
    serial.send(c);
}

/// Write a string to the serial port.  Converts newline (`\n`) to
/// carriage return + newline (`\r\n`) for compatibility with most
/// terminal emulators.
pub fn write_str(s: &str) {
    for &b in s.as_bytes() {
        if b == b'\n' {
            write_char(b'\r');
        }
        write_char(b);
    }
}

/// Write a buffer of bytes to the serial port.
pub fn write_buf(buf: &[u8]) {
    for &b in buf {
        if b == b'\n' {
            write_char(b'\r');
        }
        write_char(b);
    }
}

/// Write a u64 value as decimal to the serial port.
pub fn write_u64(mut val: u64) {
    if val == 0 {
        write_char(b'0');
        return;
    }
    
    let mut buf = [0u8; 20]; // enough for u64::MAX
    let mut i = buf.len();
    
    while val > 0 {
        i -= 1;
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    
    write_buf(&buf[i..]);
}