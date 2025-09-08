//! VirtIO Console driver for SIS kernel
//!
//! Implements VirtIO console device driver for enhanced I/O performance
//! Provides character-based I/O through VirtIO virtqueues

use crate::driver::{Driver, DriverInfo, DeviceInfo, DeviceId, DriverResult, DriverError};
use crate::virtio::{VirtIOMMIOTransport, VirtIOMMIOOffset, VirtIODeviceType};

/// VirtIO Console feature bits
#[repr(u32)]
pub enum VirtIOConsoleFeatures {
    /// Console has multiple ports
    MultiPort = 1 << 1,
    /// Console supports emergency write
    EmergWrite = 1 << 2,
}

/// VirtIO Console configuration space
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct VirtIOConsoleConfig {
    /// Number of columns
    cols: u16,
    /// Number of rows  
    rows: u16,
    /// Maximum number of ports
    max_nr_ports: u32,
    /// Emergency write character
    emerg_wr: u32,
}

/// VirtIO Console control message types
#[repr(u16)]
pub enum VirtIOConsoleControlType {
    DeviceReady = 0,
    DeviceAdd = 1,
    DeviceRemove = 2,
    PortReady = 3,
    ConsolePort = 4,
    Resize = 5,
    PortOpen = 6,
    PortName = 7,
}

/// VirtIO Console control message
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct VirtIOConsoleControl {
    /// Port ID
    id: u32,
    /// Event type
    event: u16,
    /// Value
    value: u16,
}

/// VirtQueue descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtQueueDesc {
    /// Buffer address
    addr: u64,
    /// Buffer length
    len: u32,
    /// Descriptor flags
    flags: u16,
    /// Next descriptor index
    next: u16,
}

/// VirtQueue available ring
#[repr(C)]
#[derive(Debug)]
struct VirtQueueAvail {
    /// Flags
    flags: u16,
    /// Index
    idx: u16,
    /// Ring of descriptor indices
    ring: [u16; 256],
    /// Used event (optional)
    used_event: u16,
}

/// VirtQueue used ring element
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtQueueUsedElem {
    /// Descriptor index
    id: u32,
    /// Bytes written
    len: u32,
}

/// VirtQueue used ring
#[repr(C)]
#[derive(Debug)]
struct VirtQueueUsed {
    /// Flags
    flags: u16,
    /// Index
    idx: u16,
    /// Ring of used elements
    ring: [VirtQueueUsedElem; 256],
    /// Available event (optional)
    avail_event: u16,
}

/// VirtQueue implementation
struct VirtQueue {
    /// Queue index
    #[allow(dead_code)]
    index: u16,
    /// Queue size
    size: u16,
    /// Descriptor table
    desc_table: *mut VirtQueueDesc,
    /// Available ring
    avail_ring: *mut VirtQueueAvail,
    /// Used ring
    used_ring: *mut VirtQueueUsed,
    /// Next available descriptor
    next_desc: u16,
    /// Last seen used index
    last_used_idx: u16,
}

impl VirtQueue {
    /// Create new virtqueue (simplified for basic console)
    unsafe fn new(index: u16, size: u16, desc_addr: u64) -> Self {
        let desc_table = desc_addr as *mut VirtQueueDesc;
        let avail_ring = (desc_addr + (size as u64 * 16)) as *mut VirtQueueAvail;
        let used_ring = (desc_addr + (size as u64 * 16) + (size as u64 * 2) + 6) as *mut VirtQueueUsed;

        // Initialize descriptor table
        for i in 0..size {
            let desc = &mut *desc_table.add(i as usize);
            desc.addr = 0;
            desc.len = 0;
            desc.flags = 0;
            desc.next = (i + 1) % size;
        }

        // Initialize available ring
        (*avail_ring).flags = 0;
        (*avail_ring).idx = 0;

        // Initialize used ring
        (*used_ring).flags = 0;
        (*used_ring).idx = 0;

        VirtQueue {
            index,
            size,
            desc_table,
            avail_ring,
            used_ring,
            next_desc: 0,
            last_used_idx: 0,
        }
    }

    /// Add buffer to queue
    unsafe fn add_buffer(&mut self, addr: u64, len: u32, flags: u16) -> Result<(), DriverError> {
        if self.next_desc >= self.size {
            return Err(DriverError::ResourceError);
        }

        let desc_idx = self.next_desc;
        let desc = &mut *self.desc_table.add(desc_idx as usize);
        
        desc.addr = addr;
        desc.len = len;
        desc.flags = flags;
        
        // Add to available ring
        let avail_idx = (*self.avail_ring).idx as usize % self.size as usize;
        (*self.avail_ring).ring[avail_idx] = desc_idx;
        
        // Update available index
        (*self.avail_ring).idx = (*self.avail_ring).idx.wrapping_add(1);
        
        self.next_desc = (self.next_desc + 1) % self.size;
        
        Ok(())
    }

    /// Check for used buffers
    unsafe fn get_used_buffer(&mut self) -> Option<(u32, u32)> {
        if self.last_used_idx == (*self.used_ring).idx {
            return None;
        }

        let used_elem = (*self.used_ring).ring[self.last_used_idx as usize % self.size as usize];
        self.last_used_idx = self.last_used_idx.wrapping_add(1);
        
        Some((used_elem.id, used_elem.len))
    }
}

/// VirtIO Console driver
pub struct VirtIOConsoleDriver {
    transport: Option<VirtIOMMIOTransport>,
    receiveq: Option<VirtQueue>,
    transmitq: Option<VirtQueue>,
    buffer: [u8; 4096],
    initialized: bool,
}

impl VirtIOConsoleDriver {
    /// Create new VirtIO console driver
    pub const fn new() -> Self {
        VirtIOConsoleDriver {
            transport: None,
            receiveq: None,
            transmitq: None,
            buffer: [0; 4096],
            initialized: false,
        }
    }

    /// Initialize virtqueues
    unsafe fn init_virtqueues(&mut self, _device: &DeviceInfo) -> DriverResult<()> {
        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;

        // Select queue 0 (receiveq)
        transport.write_reg(VirtIOMMIOOffset::QueueSel, 0);
        let queue0_size = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
        
        if queue0_size == 0 {
            return Err(DriverError::NotSupported);
        }

        // Allocate memory for queue 0 (simplified - using static addresses)
        let queue0_addr = 0x50000000u64;
        self.receiveq = Some(VirtQueue::new(0, queue0_size as u16, queue0_addr));

        // Set queue 0 addresses
        transport.write_reg(VirtIOMMIOOffset::QueueDescLow, (queue0_addr & 0xFFFFFFFF) as u32);
        transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (queue0_addr >> 32) as u32);
        
        let avail_addr = queue0_addr + (queue0_size as u64 * 16);
        transport.write_reg(VirtIOMMIOOffset::QueueAvailLow, (avail_addr & 0xFFFFFFFF) as u32);
        transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail_addr >> 32) as u32);
        
        let used_addr = avail_addr + (queue0_size as u64 * 2) + 6;
        transport.write_reg(VirtIOMMIOOffset::QueueUsedLow, (used_addr & 0xFFFFFFFF) as u32);
        transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used_addr >> 32) as u32);

        // Set queue size and enable
        transport.write_reg(VirtIOMMIOOffset::QueueNum, queue0_size);
        transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);

        // Select queue 1 (transmitq)
        transport.write_reg(VirtIOMMIOOffset::QueueSel, 1);
        let queue1_size = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
        
        if queue1_size > 0 {
            let queue1_addr = 0x50010000u64;
            self.transmitq = Some(VirtQueue::new(1, queue1_size as u16, queue1_addr));

            // Set queue 1 addresses
            transport.write_reg(VirtIOMMIOOffset::QueueDescLow, (queue1_addr & 0xFFFFFFFF) as u32);
            transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (queue1_addr >> 32) as u32);
            
            let avail_addr = queue1_addr + (queue1_size as u64 * 16);
            transport.write_reg(VirtIOMMIOOffset::QueueAvailLow, (avail_addr & 0xFFFFFFFF) as u32);
            transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail_addr >> 32) as u32);
            
            let used_addr = avail_addr + (queue1_size as u64 * 2) + 6;
            transport.write_reg(VirtIOMMIOOffset::QueueUsedLow, (used_addr & 0xFFFFFFFF) as u32);
            transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used_addr >> 32) as u32);

            // Set queue size and enable
            transport.write_reg(VirtIOMMIOOffset::QueueNum, queue1_size);
            transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);
        }

        Ok(())
    }

    /// Write data using VirtIO console
    pub fn write_data(&mut self, data: &[u8]) -> DriverResult<usize> {
        if !self.initialized {
            return Err(DriverError::InitFailed);
        }

        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;
        let transmitq = self.transmitq.as_mut().ok_or(DriverError::NotSupported)?;

        unsafe {
            // Copy data to buffer
            let len = core::cmp::min(data.len(), self.buffer.len());
            self.buffer[..len].copy_from_slice(&data[..len]);

            // Add buffer to transmit queue
            transmitq.add_buffer(self.buffer.as_ptr() as u64, len as u32, 0)?;

            // Notify device
            transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);

            // Wait for completion (simplified)
            for _ in 0..1000 {
                if let Some((_, written)) = transmitq.get_used_buffer() {
                    return Ok(written as usize);
                }
                core::hint::spin_loop();
            }
            
            Ok(len)
        }
    }

    /// Read data from VirtIO console
    pub fn read_data(&mut self, buffer: &mut [u8]) -> DriverResult<usize> {
        if !self.initialized {
            return Err(DriverError::InitFailed);
        }

        let receiveq = self.receiveq.as_mut().ok_or(DriverError::NotSupported)?;

        unsafe {
            if let Some((_, len)) = receiveq.get_used_buffer() {
                let read_len = core::cmp::min(len as usize, buffer.len());
                buffer[..read_len].copy_from_slice(&self.buffer[..read_len]);
                return Ok(read_len);
            }
        }

        Ok(0)
    }
}

impl Driver for VirtIOConsoleDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "VirtIO Console",
            version: "1.0.0",
            supported_devices: &[
                DeviceId {
                    vendor_id: 0x1AF4, // Red Hat (VirtIO)
                    device_id: 3,      // Console
                    class: 0x07,       // Communication controller
                    subclass: 0x80,    // Other
                },
            ],
        }
    }

    fn probe(&self, device: &DeviceInfo) -> bool {
        device.id.vendor_id == 0x1AF4 && 
        device.id.device_id == 3 &&
        device.id.class == 0x07
    }

    fn init(&mut self, device: &DeviceInfo) -> DriverResult<()> {
        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Initializing VirtIO console driver\n");
        }

        // Create VirtIO transport
        let transport = VirtIOMMIOTransport::new(
            device.base_addr,
            device.size,
            device.irq,
        )?;

        // Verify this is a console device
        if transport.device_type() != VirtIODeviceType::Console {
            return Err(DriverError::InvalidDevice);
        }

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Device verified as VirtIO console\n");
        }

        // Initialize device with minimal features
        transport.init_device(0)?;

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Device initialization complete\n");
        }

        self.transport = Some(transport);
        
        // Initialize virtqueues
        unsafe {
            self.init_virtqueues(device)?;
        }

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Virtqueues initialized\n");
        }

        Ok(())
    }

    fn start(&mut self) -> DriverResult<()> {
        if let Some(transport) = &self.transport {
            // Mark driver as ready
            transport.driver_ready();
            
            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Driver marked as ready\n");
            }
            
            self.initialized = true;
            
            // Test basic functionality
            if let Ok(written) = self.write_data(b"VirtIO Console initialized!\n") {
                unsafe {
                    crate::uart_print(b"[VIRTIO-CONSOLE] Test write completed, bytes written: ");
                    self.print_number(written as u32);
                    crate::uart_print(b"\n");
                }
            }
            
            Ok(())
        } else {
            Err(DriverError::InitFailed)
        }
    }

    fn stop(&mut self) -> DriverResult<()> {
        if let Some(transport) = &self.transport {
            // Reset device
            transport.reset_device()?;
            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Device reset\n");
            }
        }
        
        self.initialized = false;
        Ok(())
    }

    fn handle_irq(&mut self) -> DriverResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;
        
        // Read and acknowledge interrupts
        let int_status = transport.read_reg(VirtIOMMIOOffset::InterruptStatus);
        if int_status != 0 {
            transport.write_reg(VirtIOMMIOOffset::InterruptACK, int_status);
            
            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Interrupt handled: ");
                self.print_hex(int_status);
                crate::uart_print(b"\n");
            }
        }
        
        Ok(())
    }

    fn read(&mut self, _offset: u64, buffer: &mut [u8]) -> DriverResult<usize> {
        self.read_data(buffer)
    }

    fn write(&mut self, _offset: u64, data: &[u8]) -> DriverResult<usize> {
        self.write_data(data)
    }
}

impl VirtIOConsoleDriver {
    /// Helper to print numbers
    unsafe fn print_number(&self, mut num: u32) {
        if num == 0 {
            crate::uart_print(b"0");
            return;
        }
        
        let mut digits = [0u8; 10];
        let mut i = 0;
        
        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }
        
        while i > 0 {
            i -= 1;
            crate::uart_print(&[digits[i]]);
        }
    }

    /// Helper to print hex numbers
    unsafe fn print_hex(&self, num: u32) {
        crate::uart_print(b"0x");
        for i in (0..8).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            let c = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
            crate::uart_print(&[c]);
        }
    }
}

/// Global VirtIO console driver instance
static mut VIRTIO_CONSOLE_DRIVER: VirtIOConsoleDriver = VirtIOConsoleDriver::new();

/// Get reference to global VirtIO console driver
pub fn get_virtio_console_driver() -> &'static mut VirtIOConsoleDriver {
    unsafe { 
        let driver_ptr = &raw mut VIRTIO_CONSOLE_DRIVER;
        &mut *driver_ptr
    }
}