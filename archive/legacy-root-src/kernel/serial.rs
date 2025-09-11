//! Serial port logging.
//!
//! Uses the `uart_16550` crate to interface with a standard 16550
//! compatible UART.  The kernel uses COM1 at 115200 baud for
//! diagnostics and logging.  Functions are provided to write
//! characters, strings and byte buffers to the serial port.

use spin::Mutex;

#[cfg(target_arch = "x86_64")]
use uart_16550::SerialPort;

// Serial port abstraction
#[cfg(target_arch = "x86_64")]
lazy_static::lazy_static! {
    static ref SERIAL: Mutex<SerialPort> = {
        // The base address for COM1 is 0x3F8.
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// ARM64 serial implementation (simplified for Mac M1)
#[cfg(target_arch = "aarch64")]
lazy_static::lazy_static! {
    static ref SERIAL: Mutex<Arm64Serial> = {
        Mutex::new(Arm64Serial::new())
    };
}

#[cfg(target_arch = "aarch64")]
struct Arm64Serial;

#[cfg(target_arch = "aarch64")]
impl Arm64Serial {
    fn new() -> Self {
        Arm64Serial
    }
    
    fn send(&mut self, byte: u8) {
        // For Mac M1, we can use hypervisor console or memory-mapped UART
        // This is a simplified implementation
        unsafe {
            // Write to a debug register or memory location
            // For now, just a no-op on real hardware
            let _ = byte;
        }
    }
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

/// Convert a nibble (4 bits) to hex character
fn nibble_to_hex(n: u8) -> u8 {
    match n & 0xF {
        0..=9 => b'0' + (n & 0xF),
        10..=15 => b'A' + ((n & 0xF) - 10),
        _ => b'?',
    }
}

/// Write an 8-bit value as hex to the serial port.
pub fn write_hex8(val: u8) {
    let hi = nibble_to_hex(val >> 4);
    let lo = nibble_to_hex(val);
    write_char(hi);
    write_char(lo);
}

/// Write a 16-bit value as hex to the serial port.
pub fn write_hex16(val: u16) {
    write_hex8((val >> 8) as u8);
    write_hex8(val as u8);
}

/// Write a 32-bit value as hex to the serial port.
pub fn write_hex32(val: u32) {
    write_hex16((val >> 16) as u16);
    write_hex16(val as u16);
}

/// Write a 64-bit value as hex to the serial port.
pub fn write_hex64(val: u64) {
    write_hex32((val >> 32) as u32);
    write_hex32(val as u32);
}

// Keep your existing write_hex8/16/32/64; add generic front-doors to kill E0308 spam.

#[inline]
pub fn write_hex<T: Into<u64>>(v: T) {
    write_hex64(v.into());
}

#[inline]
pub fn write_hex0x<T: Into<u64>>(v: T) {
    write_str("0x");
    write_hex64(v.into());
}

#[inline]
pub fn write_dec<T: Into<u64>>(v: T) {
    // minimal decimal writer to avoid alloc; prints u64
    let mut n = v.into();
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if n == 0 {
        write_str("0");
        return;
    }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    unsafe {
        write_buf(&buf[i..]);
    }
}

/// Write a buffer of bytes to the serial port (unsafe version for internal use).
unsafe fn write_bytes(buf: &[u8]) {
    write_buf(buf);
}

/// Write formatted output to the serial port.
pub fn write_fmt(args: core::fmt::Arguments) -> Result<(), core::fmt::Error> {
    use core::fmt::Write;

    // Simple buffer-based formatter
    struct SerialWriter;

    impl Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            write_str(s);
            Ok(())
        }
    }

    let mut writer = SerialWriter;
    writer.write_fmt(args)
}

/// Print formatted arguments (for macro compatibility)
#[inline]
pub fn print(args: core::fmt::Arguments<'_>) -> Result<(), core::fmt::Error> {
    write_fmt(args)
}

/// Write single byte (Multi-AI boot framework)
pub fn write_byte(byte: u8) {
    #[cfg(target_arch = "x86_64")]
    {
        SERIAL.lock().send(byte);
    }
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 UART - for now just use the write_str path
        // This will be enhanced with actual UART byte writing
        crate::arch::aarch64::uart::write_byte(byte);
    }
}

/// Flush serial output (Multi-AI boot framework)
pub fn flush() {
    #[cfg(target_arch = "x86_64")]
    {
        // 16550 UART doesn't need explicit flushing
    }
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 UART flush
        crate::arch::aarch64::uart::flush();
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::kernel::serial::print(core::format_args!($($arg)*)).unwrap()
    };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {
        $crate::kprint!("{}\n", core::format_args!($($arg)*))
    };
}
