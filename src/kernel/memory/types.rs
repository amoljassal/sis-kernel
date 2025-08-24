//! Memory management types for vDSO integration
//!
//! Safe abstractions for physical frames and virtual pages
//! Based on ChatGPT's type-safe memory management patterns

use core::fmt;

/// Physical frame address (4KB aligned)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame(pub u64);

impl PhysFrame {
    /// Create new physical frame (must be 4KB aligned)
    pub const fn new(addr: u64) -> Self {
        assert!(addr & 0xFFF == 0, "Physical frame must be 4KB aligned");
        Self(addr)
    }
    
    /// Get physical address
    pub const fn addr(self) -> u64 {
        self.0
    }
    
    /// Get frame number
    pub const fn number(self) -> u64 {
        self.0 >> 12
    }
    
    /// Add offset to frame
    pub const fn offset(self, offset: u64) -> PhysFrame {
        Self(self.0 + (offset << 12))
    }
}

impl fmt::Display for PhysFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysFrame(0x{:016x})", self.0)
    }
}

/// Virtual page address (4KB aligned)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPage(pub u64);

impl VirtPage {
    /// Create new virtual page (must be 4KB aligned)
    pub const fn new(addr: u64) -> Self {
        assert!(addr & 0xFFF == 0, "Virtual page must be 4KB aligned");
        Self(addr)
    }
    
    /// Get virtual address
    pub const fn addr(self) -> u64 {
        self.0
    }
    
    /// Get page number
    pub const fn number(self) -> u64 {
        self.0 >> 12
    }
    
    /// Add offset to page
    pub const fn offset(self, offset: u64) -> VirtPage {
        Self(self.0 + (offset << 12))
    }
    
    /// Get as pointer
    pub const fn as_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }
    
    /// Get as mut pointer
    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl fmt::Display for VirtPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtPage(0x{:016x})", self.0)
    }
}

/// Page table entry flags for ARM64
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PteFlags(pub u64);

impl PteFlags {
    /// Create new empty flags
    pub const fn new() -> Self {
        Self(0)
    }
    
    /// Set user accessible
    pub const fn with_user(mut self, user: bool) -> Self {
        if user {
            self.0 |= 1 << 6; // AP[1] = 1 for user access
        } else {
            self.0 &= !(1 << 6);
        }
        self
    }
    
    /// Set read-only
    pub const fn with_readonly(mut self, readonly: bool) -> Self {
        if readonly {
            self.0 |= 1 << 7; // AP[2] = 1 for read-only
        } else {
            self.0 &= !(1 << 7);
        }
        self
    }
    
    /// Set executable
    pub const fn with_executable(mut self, executable: bool) -> Self {
        if executable {
            self.0 &= !(1 << 54); // UXN = 0 for executable
        } else {
            self.0 |= 1 << 54; // UXN = 1 for non-executable
        }
        self
    }
    
    /// Set shared
    pub const fn with_shared(mut self, shared: bool) -> Self {
        if shared {
            self.0 |= 3 << 8; // SH[1:0] = 11 for inner shareable
        } else {
            self.0 &= !(3 << 8);
        }
        self
    }
    
    /// Set memory attributes (Normal memory, write-back)
    pub const fn with_normal_memory(mut self) -> Self {
        // AttrIndx = 0 (Normal memory)
        // AF = 1 (Access flag set)
        self.0 |= 1 << 10; // AF
        self
    }
    
    /// Get raw flags value
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Default for PteFlags {
    fn default() -> Self {
        Self::new().with_normal_memory()
    }
}

/// Memory management error types
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    Success = 0,
    OutOfMemory = -12,
    InvalidAddress = -14,
    PermissionDenied = -1,
    AlreadyMapped = -17,
    NotMapped = -2,
    InvalidAlignment = -22,
    InitializationFailed = -23,
    NotInitialized = -24,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::Success => write!(f, "Success"),
            MemoryError::OutOfMemory => write!(f, "Out of memory"),
            MemoryError::InvalidAddress => write!(f, "Invalid address"),
            MemoryError::PermissionDenied => write!(f, "Permission denied"),
            MemoryError::AlreadyMapped => write!(f, "Already mapped"),
            MemoryError::NotMapped => write!(f, "Not mapped"),
            MemoryError::InvalidAlignment => write!(f, "Invalid alignment"),
            MemoryError::InitializationFailed => write!(f, "Initialization failed"),
            MemoryError::NotInitialized => write!(f, "Not initialized"),
        }
    }
}

/// Memory region descriptor
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Start virtual address
    pub start: VirtPage,
    
    /// Size in pages
    pub size: usize,
    
    /// Flags
    pub flags: PteFlags,
}

impl MemoryRegion {
    /// Create new memory region
    pub const fn new(start: VirtPage, size: usize, flags: PteFlags) -> Self {
        Self { start, size, flags }
    }
    
    /// Get end address
    pub fn end(&self) -> VirtPage {
        VirtPage::new(self.start.addr() + (self.size as u64 * 4096))
    }
    
    /// Check if address is within region
    pub fn contains(&self, addr: VirtPage) -> bool {
        addr >= self.start && addr < self.end()
    }
}