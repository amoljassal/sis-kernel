//! SIS Kernel Core Types with Geometric Architecture Compliance
//!
//! Implements Gemini's Newtype pattern for trait bound resolution
//! while maintaining PYRAMID > DIAMOND > HYPERCUBE architectural integrity

use core::cmp::Ordering;
use core::fmt;

/// EdgeId - Geometric identifier for graph edges in AI workloads
/// Uses Newtype pattern to provide Ord implementation for BTreeMap keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeId(pub u64);

impl EdgeId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Ord for EdgeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for EdgeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EdgeId({})", self.0)
    }
}

/// IPBlockVersion - Versioned IP blocks for FPGA/hardware synthesis
/// Implements total ordering for geometric consistency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IPBlockVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl IPBlockVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
}

impl Ord for IPBlockVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl PartialOrd for IPBlockVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for IPBlockVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// F32Ord - Total ordering wrapper for f32 values
/// Enables f32 as BTreeMap keys via bitwise comparison
#[derive(Debug, Clone, Copy)]
pub struct F32Ord(pub f32);

impl F32Ord {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
    
    pub const fn value(self) -> f32 {
        self.0
    }
}

impl PartialEq for F32Ord {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F32Ord {}

impl PartialOrd for F32Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F32Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use bitwise comparison for total ordering
        // Handle NaN and signed zero consistently
        let a_bits = self.0.to_bits();
        let b_bits = other.0.to_bits();
        
        // Convert to signed magnitude for proper ordering
        let a_signed = if (a_bits as i32) < 0 {
            !a_bits
        } else {
            a_bits | 0x80000000
        };
        
        let b_signed = if (b_bits as i32) < 0 {
            !b_bits
        } else {
            b_bits | 0x80000000
        };
        
        a_signed.cmp(&b_signed)
    }
}

/// Kernel-wide facade trait for BTreeMap keys
/// Gemini's architectural pattern for consolidating trait requirements
pub trait KernelKey: Ord + Clone + core::fmt::Debug {}

// Blanket implementation for all types meeting the requirements
impl<T: Ord + Clone + core::fmt::Debug> KernelKey for T {}

/// Kernel result type for consistent error handling
/// Follows Gemini's centralized error vocabulary pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    OutOfMemory,
    InvalidAddress,
    PermissionDenied,
    AlreadyMapped,
    NotMapped,
    InvalidAlignment,
    InitializationFailed,
    NotInitialized,
    DeviceNotFound,
    SchedulingConflict,
    InvalidWorkload,
    TimeoutExpired,
}

pub type KernelResult<T> = Result<T, KernelError>;

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::OutOfMemory => write!(f, "Out of memory"),
            KernelError::InvalidAddress => write!(f, "Invalid address"),
            KernelError::PermissionDenied => write!(f, "Permission denied"),
            KernelError::AlreadyMapped => write!(f, "Already mapped"),
            KernelError::NotMapped => write!(f, "Not mapped"),
            KernelError::InvalidAlignment => write!(f, "Invalid alignment"),
            KernelError::InitializationFailed => write!(f, "Initialization failed"),
            KernelError::NotInitialized => write!(f, "Not initialized"),
            KernelError::DeviceNotFound => write!(f, "Device not found"),
            KernelError::SchedulingConflict => write!(f, "Scheduling conflict"),
            KernelError::InvalidWorkload => write!(f, "Invalid workload"),
            KernelError::TimeoutExpired => write!(f, "Timeout expired"),
        }
    }
}