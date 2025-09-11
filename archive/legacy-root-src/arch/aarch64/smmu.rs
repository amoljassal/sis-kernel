//! ARM System Memory Management Unit (SMMU) v3 Support
//!
//! This module implements SMMU v3 support for DMA isolation and IOVA management,
//! providing hardware-enforced memory protection for devices and AI accelerators.
//!
//! SMMU provides:
//! - Device address translation and isolation
//! - DMA protection boundaries
//! - IOVA (I/O Virtual Address) space management
//! - Stream-based access control
//!
//! Geometric Principle: SMMU creates isolated address spaces like parallel
//! coordinate systems, where each device operates in its own geometric space
//! with controlled mappings to the physical memory space.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::{collections::BTreeMap, vec::Vec, boxed::Box};
use crate::kernel::sync::{SpinLock, RwLock};

/// SMMU v3 register offsets
pub mod regs {
    pub const CR0: usize = 0x0020;           // Control register 0
    pub const CR0ACK: usize = 0x0024;        // Control register 0 acknowledge
    pub const CR1: usize = 0x0028;           // Control register 1
    pub const CR2: usize = 0x002C;           // Control register 2
    pub const STATUSR: usize = 0x0040;       // Status register
    pub const GBPA: usize = 0x0044;          // Global Bypass Attribute
    pub const AGBPA: usize = 0x0048;         // Alternate Global Bypass Attribute
    pub const IRQ_CTRL: usize = 0x0050;      // Interrupt control
    pub const IRQ_CTRLACK: usize = 0x0054;   // Interrupt control acknowledge
    pub const GERROR: usize = 0x0060;        // Global error register
    pub const GERRORN: usize = 0x0064;       // Global error register (next)
    
    // Queue base addresses
    pub const STRTAB_BASE: usize = 0x0080;   // Stream table base
    pub const STRTAB_BASE_CFG: usize = 0x0088; // Stream table config
    pub const CMDQ_BASE: usize = 0x0090;     // Command queue base
    pub const CMDQ_PROD: usize = 0x0098;     // Command queue producer
    pub const CMDQ_CONS: usize = 0x009C;     // Command queue consumer
    pub const EVTQ_BASE: usize = 0x00A0;     // Event queue base
    pub const EVTQ_PROD: usize = 0x00A8;     // Event queue producer
    pub const EVTQ_CONS: usize = 0x00AC;     // Event queue consumer
    pub const EVTQ_IRQ_CFG0: usize = 0x00B0; // Event queue interrupt config
    pub const EVTQ_IRQ_CFG1: usize = 0x00B4;
    pub const EVTQ_IRQ_CFG2: usize = 0x00B8;
    
    // Page request queue (if supported)
    pub const PRIQ_BASE: usize = 0x00C0;     // Page request queue base
    pub const PRIQ_PROD: usize = 0x00C8;     // Page request queue producer
    pub const PRIQ_CONS: usize = 0x00CC;     // Page request queue consumer
}

/// SMMU v3 command opcodes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SMMUCommand {
    Prefetch = 0x01,      // Prefetch command
    CfgiSte = 0x03,       // Invalidate STE cache
    CfgiAll = 0x04,       // Invalidate all caches
    TlbiNhAsid = 0x11,    // TLB invalidate by ASID
    TlbiEL2All = 0x20,    // TLB invalidate EL2 all
    PriResp = 0x21,       // Page request response
    Resume = 0x22,        // Resume command
    Stall = 0x23,         // Stall command
    Sync = 0x46,          // Synchronize command
}

/// Stream ID for device identification
pub type StreamId = u32;

/// Address Space ID for translation context
pub type ASID = u16;

/// I/O Virtual Address
pub type IOVA = u64;

/// Device Physical Address (bus address seen by device)
pub type DPA = u64;

/// Stream Table Entry (STE) configuration
#[derive(Debug, Clone)]
pub struct StreamTableEntry {
    /// Stream ID
    pub stream_id: StreamId,
    
    /// Configuration valid
    pub valid: bool,
    
    /// Bypass mode (no translation)
    pub bypass: bool,
    
    /// Address Space ID
    pub asid: ASID,
    
    /// Translation table base address
    pub ttb: u64,
    
    /// Translation control register value
    pub tcr: u32,
    
    /// Memory attribute indirection register
    pub mair: u64,
    
    /// Access permissions
    pub permissions: StreamPermissions,
}

/// Stream permissions for DMA operations
#[derive(Debug, Clone, Copy)]
pub struct StreamPermissions {
    /// Read permission
    pub read: bool,
    
    /// Write permission
    pub write: bool,
    
    /// Execute permission (for instruction fetch)
    pub execute: bool,
    
    /// Privileged access allowed
    pub privileged: bool,
    
    /// Secure access allowed
    pub secure: bool,
}

impl Default for StreamPermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
            privileged: false,
            secure: false,
        }
    }
}

/// IOVA allocation region
#[derive(Debug, Clone)]
pub struct IOVARegion {
    /// Start address
    pub start: IOVA,
    
    /// Size in bytes
    pub size: usize,
    
    /// Available for allocation
    pub available: bool,
    
    /// Owner stream ID
    pub owner: Option<StreamId>,
}

/// DMA mapping entry
#[derive(Debug, Clone)]
pub struct DmaMapping {
    /// I/O Virtual Address
    pub iova: IOVA,
    
    /// Physical address
    pub pa: u64,
    
    /// Size in bytes
    pub size: usize,
    
    /// Stream ID that owns this mapping
    pub stream_id: StreamId,
    
    /// Access permissions
    pub permissions: StreamPermissions,
}

/// SMMU device state
pub struct SMMUDevice {
    /// Base address for MMIO registers
    pub base_address: u64,
    
    /// Device available
    pub available: bool,
    
    /// Supported features
    pub features: SMMUFeatures,
    
    /// Command queue
    pub command_queue: CommandQueue,
    
    /// Event queue  
    pub event_queue: EventQueue,
    
    /// Stream table
    pub stream_table: StreamTable,
    
    /// Statistics
    pub stats: SMMUStatistics,
}

/// SMMU features detected at initialization
#[derive(Debug, Default)]
pub struct SMMUFeatures {
    /// Number of stream ID bits
    pub stream_id_bits: u32,
    
    /// Maximum ASID value
    pub max_asid: u32,
    
    /// Translation table format support
    pub tt_format: u32,
    
    /// Page sizes supported (4K, 16K, 64K)
    pub page_sizes: u32,
    
    /// Two-level stream table support
    pub two_level_stream: bool,
    
    /// Page request interface support
    pub pri_support: bool,
    
    /// ATS (Address Translation Service) support
    pub ats_support: bool,
}

/// Command queue for SMMU operations
pub struct CommandQueue {
    /// Base address
    pub base: u64,
    
    /// Queue size (number of entries)
    pub size: usize,
    
    /// Producer index
    pub producer: AtomicU32,
    
    /// Consumer index (read from hardware)
    pub consumer: u32,
}

/// Event queue for SMMU fault reporting
pub struct EventQueue {
    /// Base address
    pub base: u64,
    
    /// Queue size
    pub size: usize,
    
    /// Producer index (written by hardware)
    pub producer: u32,
    
    /// Consumer index
    pub consumer: AtomicU32,
}

/// Stream table management
pub struct StreamTable {
    /// Base address
    pub base: u64,
    
    /// Table entries
    pub entries: RwLock<BTreeMap<StreamId, StreamTableEntry>>,
    
    /// Maximum stream ID
    pub max_stream_id: u32,
}

/// SMMU statistics
#[derive(Debug, Default)]
pub struct SMMUStatistics {
    /// Total DMA mappings created
    pub mappings_created: AtomicU64,
    
    /// Total mappings removed
    pub mappings_removed: AtomicU64,
    
    /// TLB invalidations
    pub tlb_invalidations: AtomicU64,
    
    /// Translation faults
    pub translation_faults: AtomicU64,
    
    /// Permission faults
    pub permission_faults: AtomicU64,
    
    /// Commands processed
    pub commands_processed: AtomicU64,
}

/// SMMU Manager
pub struct SMMUManager {
    /// SMMU device
    device: SpinLock<Option<SMMUDevice>>,
    
    /// IOVA allocator
    iova_allocator: RwLock<IOVAAllocator>,
    
    /// DMA mappings
    mappings: RwLock<BTreeMap<(StreamId, IOVA), DmaMapping>>,
    
    /// Global statistics
    global_stats: SMMUStatistics,
}

/// IOVA space allocator
pub struct IOVAAllocator {
    /// Available regions
    regions: Vec<IOVARegion>,
    
    /// Total IOVA space size
    total_size: usize,
    
    /// Next allocation hint
    next_alloc: IOVA,
}

impl SMMUDevice {
    /// Initialize SMMU device
    pub fn init(base_address: u64) -> Result<Self, &'static str> {
        // Read device ID to verify SMMU is present
        let device_id = unsafe {
            core::ptr::read_volatile((base_address + 0x0000) as *const u32)
        };
        
        // Check for SMMU v3 signature
        if (device_id & 0xFFFF) != 0x43B7 {
            return Err("SMMU v3 not found");
        }
        
        // Read feature registers
        let idr0 = unsafe {
            core::ptr::read_volatile((base_address + 0x0000) as *const u32)
        };
        let idr1 = unsafe {
            core::ptr::read_volatile((base_address + 0x0004) as *const u32)
        };
        
        let features = SMMUFeatures {
            stream_id_bits: ((idr1 >> 27) & 0x1F) + 1,
            max_asid: 1 << (((idr1 >> 21) & 0x3F) + 1),
            tt_format: (idr0 >> 2) & 0x3,
            page_sizes: (idr1 >> 28) & 0xF,
            two_level_stream: (idr0 & (1 << 27)) != 0,
            pri_support: (idr0 & (1 << 16)) != 0,
            ats_support: (idr0 & (1 << 17)) != 0,
        };
        
        // Initialize command queue
        let cmd_queue = CommandQueue {
            base: 0,
            size: 4096, // 4K entries
            producer: AtomicU32::new(0),
            consumer: 0,
        };
        
        // Initialize event queue
        let evt_queue = EventQueue {
            base: 0,
            size: 4096,
            producer: 0,
            consumer: AtomicU32::new(0),
        };
        
        // Initialize stream table
        let stream_table = StreamTable {
            base: 0,
            entries: RwLock::new(BTreeMap::new()),
            max_stream_id: (1 << features.stream_id_bits) - 1,
        };
        
        let mut device = Self {
            base_address,
            available: false,
            features,
            command_queue: cmd_queue,
            event_queue: evt_queue,
            stream_table,
            stats: SMMUStatistics::default(),
        };
        
        // Allocate and setup queues
        device.setup_queues()?;
        
        // Initialize stream table
        device.setup_stream_table()?;
        
        // Enable SMMU
        device.enable()?;
        
        device.available = true;
        
        crate::kernel::serial::write_str("[SMMU] SMMU v3 initialized at 0x");
        crate::kernel::serial::write_hex64(base_address);
        crate::kernel::serial::write_str("\n");
        
        Ok(device)
    }
    
    /// Setup command and event queues
    fn setup_queues(&mut self) -> Result<(), &'static str> {
        // Allocate command queue memory (64 bytes per entry)
        let cmd_queue_size = self.command_queue.size * 64;
        let cmd_queue_mem = alloc::vec![0u8; cmd_queue_size];
        let cmd_queue_addr = cmd_queue_mem.as_ptr() as u64;
        core::mem::forget(cmd_queue_mem);
        
        self.command_queue.base = cmd_queue_addr;
        
        // Configure command queue
        unsafe {
            core::ptr::write_volatile(
                (self.base_address + regs::CMDQ_BASE) as *mut u64,
                cmd_queue_addr | ((self.command_queue.size as u64).trailing_zeros() as u64),
            );
        }
        
        // Allocate event queue memory (32 bytes per entry)
        let evt_queue_size = self.event_queue.size * 32;
        let evt_queue_mem = alloc::vec![0u8; evt_queue_size];
        let evt_queue_addr = evt_queue_mem.as_ptr() as u64;
        core::mem::forget(evt_queue_mem);
        
        self.event_queue.base = evt_queue_addr;
        
        // Configure event queue
        unsafe {
            core::ptr::write_volatile(
                (self.base_address + regs::EVTQ_BASE) as *mut u64,
                evt_queue_addr | ((self.event_queue.size as u64).trailing_zeros() as u64),
            );
        }
        
        Ok(())
    }
    
    /// Setup stream table
    fn setup_stream_table(&mut self) -> Result<(), &'static str> {
        // Allocate stream table (64 bytes per entry)
        let max_streams = 1 << self.features.stream_id_bits.min(16); // Limit to reasonable size
        let table_size = max_streams * 64;
        let table_mem = alloc::vec![0u8; table_size];
        let table_addr = table_mem.as_ptr() as u64;
        core::mem::forget(table_mem);
        
        self.stream_table.base = table_addr;
        
        // Configure stream table
        unsafe {
            core::ptr::write_volatile(
                (self.base_address + regs::STRTAB_BASE) as *mut u64,
                table_addr,
            );
            
            // Configure stream table format (linear, 64-byte entries)
            let strtab_cfg = ((self.features.stream_id_bits - 1) << 16) | 0x0;
            core::ptr::write_volatile(
                (self.base_address + regs::STRTAB_BASE_CFG) as *mut u32,
                strtab_cfg,
            );
        }
        
        Ok(())
    }
    
    /// Enable SMMU
    fn enable(&mut self) -> Result<(), &'static str> {
        unsafe {
            // Set control register to enable SMMU
            let cr0 = 1 << 0; // SMMU enable bit
            core::ptr::write_volatile(
                (self.base_address + regs::CR0) as *mut u32,
                cr0,
            );
            
            // Wait for acknowledgment
            while core::ptr::read_volatile((self.base_address + regs::CR0ACK) as *const u32) & 1 == 0 {
                core::hint::spin_loop();
            }
        }
        
        Ok(())
    }
    
    /// Create stream table entry for device
    pub fn create_stream(&mut self, stream_id: StreamId, asid: ASID) -> Result<(), &'static str> {
        if stream_id > self.stream_table.max_stream_id {
            return Err("Stream ID exceeds maximum");
        }
        
        // Create translation table for this stream
        let tt_size = 4096; // One page for L1 table
        let tt_mem = alloc::vec![0u8; tt_size];
        let tt_addr = tt_mem.as_ptr() as u64;
        core::mem::forget(tt_mem);
        
        let ste = StreamTableEntry {
            stream_id,
            valid: true,
            bypass: false,
            asid,
            ttb: tt_addr,
            tcr: 0x80803520, // 4KB granule, 48-bit VA
            mair: 0x00000000FFul, // Normal memory attributes
            permissions: StreamPermissions::default(),
        };
        
        // Write STE to stream table
        self.write_stream_table_entry(&ste)?;
        
        // Add to local registry
        self.stream_table.entries.write().insert(stream_id, ste);
        
        // Invalidate STE cache
        self.invalidate_ste_cache(stream_id)?;
        
        Ok(())
    }
    
    /// Write stream table entry to hardware
    fn write_stream_table_entry(&mut self, ste: &StreamTableEntry) -> Result<(), &'static str> {
        let entry_addr = self.stream_table.base + (ste.stream_id as u64 * 64);
        
        unsafe {
            // STE is 64 bytes, write as 8 u64 values
            let entry_ptr = entry_addr as *mut u64;
            
            // Word 0: Valid, Config, StreamID
            let word0 = if ste.valid { 1u64 } else { 0u64 } |
                       if ste.bypass { 1u64 << 2 } else { 0u64 } |
                       ((ste.asid as u64) << 48);
            core::ptr::write_volatile(entry_ptr.offset(0), word0);
            
            // Word 1: Translation table base
            core::ptr::write_volatile(entry_ptr.offset(1), ste.ttb);
            
            // Word 2: TCR
            core::ptr::write_volatile(entry_ptr.offset(2), ste.tcr as u64);
            
            // Word 3: MAIR
            core::ptr::write_volatile(entry_ptr.offset(3), ste.mair);
            
            // Remaining words can be zero for basic configuration
            for i in 4..8 {
                core::ptr::write_volatile(entry_ptr.offset(i), 0);
            }
        }
        
        Ok(())
    }
    
    /// Send command to SMMU
    fn send_command(&mut self, command: SMMUCommand, args: &[u64]) -> Result<(), &'static str> {
        let producer = self.command_queue.producer.load(Ordering::Acquire);
        let next_producer = (producer + 1) % (self.command_queue.size as u32);
        
        // Check queue not full
        if next_producer == self.command_queue.consumer {
            return Err("Command queue full");
        }
        
        // Write command entry (64 bytes)
        let entry_addr = self.command_queue.base + (producer as u64 * 64);
        unsafe {
            let entry_ptr = entry_addr as *mut u64;
            
            // Command word 0: opcode
            core::ptr::write_volatile(entry_ptr.offset(0), command as u64);
            
            // Arguments
            for (i, &arg) in args.iter().enumerate() {
                if i < 7 {
                    core::ptr::write_volatile(entry_ptr.offset(i + 1), arg);
                }
            }
            
            // Pad remaining words
            for i in (args.len() + 1)..8 {
                core::ptr::write_volatile(entry_ptr.offset(i), 0);
            }
        }
        
        // Update producer index
        self.command_queue.producer.store(next_producer, Ordering::Release);
        
        // Notify hardware
        unsafe {
            core::ptr::write_volatile(
                (self.base_address + regs::CMDQ_PROD) as *mut u32,
                next_producer,
            );
        }
        
        self.stats.commands_processed.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Invalidate STE cache for stream
    fn invalidate_ste_cache(&mut self, stream_id: StreamId) -> Result<(), &'static str> {
        self.send_command(SMMUCommand::CfgiSte, &[stream_id as u64])
    }
    
    /// Map IOVA to physical address
    pub fn map_pages(&mut self, stream_id: StreamId, iova: IOVA, pa: u64, size: usize, permissions: StreamPermissions) -> Result<(), &'static str> {
        // Validate alignment
        if (iova & 0xFFF) != 0 || (pa & 0xFFF) != 0 || (size & 0xFFF) != 0 {
            return Err("Address or size not page aligned");
        }
        
        // Get stream table entry
        let entries = self.stream_table.entries.read();
        let ste = entries.get(&stream_id).ok_or("Stream not found")?;
        
        // Map pages in translation table (simplified single-level for demo)
        let num_pages = size / 4096;
        let tt_base = ste.ttb as *mut u64;
        
        unsafe {
            for i in 0..num_pages {
                let page_iova = iova + (i * 4096) as u64;
                let page_pa = pa + (i * 4096) as u64;
                let tt_index = (page_iova >> 12) & 0x1FF; // Simplified L1 index
                
                // Create page table entry with permissions
                let mut pte = page_pa | 0x3; // Valid + Table
                if permissions.read { pte |= 1 << 6; }
                if permissions.write { pte |= 1 << 7; }
                
                core::ptr::write_volatile(tt_base.offset(tt_index as isize), pte);
            }
        }
        
        // Invalidate TLB
        self.send_command(SMMUCommand::TlbiNhAsid, &[ste.asid as u64])?;
        
        self.stats.mappings_created.fetch_add(num_pages as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Unmap IOVA range
    pub fn unmap_pages(&mut self, stream_id: StreamId, iova: IOVA, size: usize) -> Result<(), &'static str> {
        // Validate alignment
        if (iova & 0xFFF) != 0 || (size & 0xFFF) != 0 {
            return Err("Address or size not page aligned");
        }
        
        // Get stream table entry
        let entries = self.stream_table.entries.read();
        let ste = entries.get(&stream_id).ok_or("Stream not found")?;
        
        // Unmap pages in translation table
        let num_pages = size / 4096;
        let tt_base = ste.ttb as *mut u64;
        
        unsafe {
            for i in 0..num_pages {
                let page_iova = iova + (i * 4096) as u64;
                let tt_index = (page_iova >> 12) & 0x1FF;
                
                // Clear page table entry
                core::ptr::write_volatile(tt_base.offset(tt_index as isize), 0);
            }
        }
        
        // Invalidate TLB
        self.send_command(SMMUCommand::TlbiNhAsid, &[ste.asid as u64])?;
        
        self.stats.mappings_removed.fetch_add(num_pages as u64, Ordering::Relaxed);
        
        Ok(())
    }
}

impl IOVAAllocator {
    /// Create new IOVA allocator
    pub fn new(base: IOVA, size: usize) -> Self {
        let mut regions = Vec::new();
        regions.push(IOVARegion {
            start: base,
            size,
            available: true,
            owner: None,
        });
        
        Self {
            regions,
            total_size: size,
            next_alloc: base,
        }
    }
    
    /// Allocate IOVA range
    pub fn allocate(&mut self, size: usize, stream_id: StreamId) -> Result<IOVA, &'static str> {
        // Align size to page boundary
        let aligned_size = (size + 0xFFF) & !0xFFF;
        
        // Find suitable region
        for region in &mut self.regions {
            if region.available && region.size >= aligned_size {
                let iova = region.start;
                
                if region.size == aligned_size {
                    // Exact match - mark as unavailable
                    region.available = false;
                    region.owner = Some(stream_id);
                } else {
                    // Split region
                    let remaining = IOVARegion {
                        start: region.start + aligned_size as u64,
                        size: region.size - aligned_size,
                        available: true,
                        owner: None,
                    };
                    
                    region.size = aligned_size;
                    region.available = false;
                    region.owner = Some(stream_id);
                    
                    self.regions.push(remaining);
                }
                
                return Ok(iova);
            }
        }
        
        Err("No suitable IOVA range available")
    }
    
    /// Free IOVA range
    pub fn free(&mut self, iova: IOVA, stream_id: StreamId) -> Result<(), &'static str> {
        // Find and free the region
        for region in &mut self.regions {
            if region.start == iova && region.owner == Some(stream_id) {
                region.available = true;
                region.owner = None;
                
                // TODO: Coalesce adjacent free regions
                return Ok(());
            }
        }
        
        Err("IOVA region not found")
    }
}

impl SMMUManager {
    /// Create new SMMU manager
    pub const fn new() -> Self {
        Self {
            device: SpinLock::new(None),
            iova_allocator: RwLock::new(IOVAAllocator {
                regions: Vec::new(),
                total_size: 0,
                next_alloc: 0,
            }),
            mappings: RwLock::new(BTreeMap::new()),
            global_stats: SMMUStatistics {
                mappings_created: AtomicU64::new(0),
                mappings_removed: AtomicU64::new(0),
                tlb_invalidations: AtomicU64::new(0),
                translation_faults: AtomicU64::new(0),
                permission_faults: AtomicU64::new(0),
                commands_processed: AtomicU64::new(0),
            },
        }
    }
    
    /// Initialize SMMU manager
    pub fn init(&self, base_address: u64) -> Result<(), &'static str> {
        match SMMUDevice::init(base_address) {
            Ok(device) => {
                *self.device.lock() = Some(device);
                
                // Initialize IOVA allocator (1GB space starting at 0x8000_0000)
                *self.iova_allocator.write() = IOVAAllocator::new(0x8000_0000, 1024 * 1024 * 1024);
                
                Ok(())
            }
            Err(e) => {
                crate::kernel::serial::write_str("[SMMU] SMMU initialization failed: ");
                crate::kernel::serial::write_str(e);
                crate::kernel::serial::write_str("\n");
                Ok(()) // Continue without SMMU
            }
        }
    }
    
    /// Create DMA mapping
    pub fn map_dma(&self, stream_id: StreamId, pa: u64, size: usize, permissions: StreamPermissions) -> Result<IOVA, &'static str> {
        let iova = {
            let mut allocator = self.iova_allocator.write();
            allocator.allocate(size, stream_id)?
        };
        
        // Create hardware mapping
        if let Some(ref mut device) = *self.device.lock() {
            device.map_pages(stream_id, iova, pa, size, permissions)?;
        }
        
        // Record mapping
        let mapping = DmaMapping {
            iova,
            pa,
            size,
            stream_id,
            permissions,
        };
        
        self.mappings.write().insert((stream_id, iova), mapping);
        self.global_stats.mappings_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(iova)
    }
    
    /// Remove DMA mapping
    pub fn unmap_dma(&self, stream_id: StreamId, iova: IOVA) -> Result<(), &'static str> {
        // Remove from hardware
        if let Some(mapping) = self.mappings.write().remove(&(stream_id, iova)) {
            if let Some(ref mut device) = *self.device.lock() {
                device.unmap_pages(stream_id, iova, mapping.size)?;
            }
            
            // Free IOVA space
            let mut allocator = self.iova_allocator.write();
            allocator.free(iova, stream_id)?;
            
            self.global_stats.mappings_removed.fetch_add(1, Ordering::Relaxed);
            
            Ok(())
        } else {
            Err("Mapping not found")
        }
    }
    
    /// Create stream for device
    pub fn create_stream(&self, stream_id: StreamId) -> Result<ASID, &'static str> {
        if let Some(ref mut device) = *self.device.lock() {
            let asid = (stream_id as u16) % 256; // Simple ASID allocation
            device.create_stream(stream_id, asid)?;
            Ok(asid)
        } else {
            Err("SMMU not available")
        }
    }
}

/// Global SMMU manager
static SMMU_MANAGER: SMMUManager = SMMUManager::new();

/// Initialize SMMU subsystem
pub fn init() -> Result<(), &'static str> {
    // Try common SMMU base addresses
    const SMMU_BASE_ADDRESSES: &[u64] = &[
        0x0900_0000, // QEMU virt SMMU
        0x0500_0000, // Alternative QEMU address
        0x2B40_0000, // ARM Fixed Virtual Platform
    ];
    
    for &base_addr in SMMU_BASE_ADDRESSES {
        if SMMU_MANAGER.init(base_addr).is_ok() {
            break;
        }
    }
    
    crate::kernel::serial::write_str("[SMMU] SMMU subsystem initialized\n");
    Ok(())
}

/// Create stream for device
pub fn create_stream(stream_id: StreamId) -> Result<ASID, &'static str> {
    SMMU_MANAGER.create_stream(stream_id)
}

/// Map DMA buffer
pub fn map_dma(stream_id: StreamId, pa: u64, size: usize, permissions: StreamPermissions) -> Result<IOVA, &'static str> {
    SMMU_MANAGER.map_dma(stream_id, pa, size, permissions)
}

/// Unmap DMA buffer
pub fn unmap_dma(stream_id: StreamId, iova: IOVA) -> Result<(), &'static str> {
    SMMU_MANAGER.unmap_dma(stream_id, iova)
}