//! ARM64 UART Implementation for SIS Kernel
//!
//! Early UART support for ARM64 boot debugging
//! Compatible with m1n1 proxy and eventual hardware UART

/// Write single byte to ARM64 UART
pub fn write_byte(byte: u8) {
    // For now, this is a placeholder
    // In a real implementation, this would write to MMIO UART registers
    // For M1/M2, we rely on m1n1 proxy initially
}

/// Write string to ARM64 UART
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

/// Flush ARM64 UART output
pub fn flush() {
    // Placeholder for UART flush
    // In a real implementation, would wait for transmit buffer empty
}

/// Initialize early UART for ARM64
pub fn init_early_uart() -> Result<(), &'static str> {
    // Early UART initialization for ARM64
    // For M1/M2 with m1n1, no initialization needed initially
    Ok(())
}