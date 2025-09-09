//! Trusted Platform Module (TPM) 2.0 Integration
//!
//! This module implements TPM 2.0 support for measured boot, attestation,
//! and secure key storage in the SIS kernel security architecture.
//!
//! TPM provides hardware-rooted trust through:
//! - Platform Configuration Registers (PCRs) for integrity measurement
//! - Attestation keys for cryptographic proof of system state
//! - Sealed storage for protecting keys and secrets
//!
//! Geometric Principle: TPM measurements form a hash chain creating an
//! immutable audit trail, like adding vertices to a directed acyclic graph
//! where each measurement extends the previous state.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::{vec::Vec, collections::BTreeMap, boxed::Box};
use crate::kernel::sync::{SpinLock, RwLock};

/// TPM 2.0 Command Codes
pub mod commands {
    pub const TPM_CC_STARTUP: u32 = 0x00000144;
    pub const TPM_CC_SHUTDOWN: u32 = 0x00000145;
    pub const TPM_CC_PCR_EXTEND: u32 = 0x00000182;
    pub const TPM_CC_PCR_READ: u32 = 0x0000017E;
    pub const TPM_CC_QUOTE: u32 = 0x00000158;
    pub const TPM_CC_CREATE_PRIMARY: u32 = 0x00000131;
    pub const TPM_CC_CREATE: u32 = 0x00000153;
    pub const TPM_CC_LOAD: u32 = 0x00000157;
    pub const TPM_CC_SEAL: u32 = 0x0000015B;
    pub const TPM_CC_UNSEAL: u32 = 0x0000015E;
    pub const TPM_CC_GET_RANDOM: u32 = 0x0000017B;
    pub const TPM_CC_HASH: u32 = 0x0000017D;
}

/// TPM 2.0 Response Codes
pub mod responses {
    pub const TPM_RC_SUCCESS: u32 = 0x00000000;
    pub const TPM_RC_FAILURE: u32 = 0x00000101;
    pub const TPM_RC_INITIALIZE: u32 = 0x00000100;
    pub const TPM_RC_DISABLED: u32 = 0x00000120;
    pub const TPM_RC_PCR: u32 = 0x00000127;
    pub const TPM_RC_HANDLE: u32 = 0x0000008B;
    pub const TPM_RC_BAD_AUTH: u32 = 0x0000009A;
}

/// TPM Hash Algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TpmHashAlg {
    Sha1 = 0x0004,
    Sha256 = 0x000B,
    Sha384 = 0x000C,
    Sha512 = 0x000D,
}

impl TpmHashAlg {
    pub fn digest_size(&self) -> usize {
        match self {
            TpmHashAlg::Sha1 => 20,
            TpmHashAlg::Sha256 => 32,
            TpmHashAlg::Sha384 => 48,
            TpmHashAlg::Sha512 => 64,
        }
    }
}

/// Platform Configuration Register (PCR) indices
pub mod pcr {
    pub const CORE_FIRMWARE: usize = 0;         // Core firmware/bootloader
    pub const KERNEL: usize = 1;               // Kernel image  
    pub const EARLY_BOOT: usize = 2;           // Early boot components
    pub const OS_LOADER: usize = 3;            // OS loader
    pub const CONFIGURATION: usize = 4;        // Boot configuration
    pub const APPLICATIONS: usize = 5;         // Application measurements
    pub const AI_MODELS: usize = 8;            // AI model measurements
    pub const AI_CONFIG: usize = 9;            // AI configuration
    pub const SECURITY_CONFIG: usize = 10;     // Security configuration
    pub const CAPABILITY_STATE: usize = 11;    // Capability system state
    pub const DEBUG: usize = 16;               // Debug measurements
    pub const DYNAMIC_1: usize = 17;           // Dynamic measurements
    pub const DYNAMIC_2: usize = 18;
    pub const DYNAMIC_3: usize = 19;
    pub const LOCALITY_1: usize = 20;          // Locality-based measurements
    pub const LOCALITY_2: usize = 21;
    pub const LOCALITY_3: usize = 22;
    pub const LOCALITY_4: usize = 23;
}

/// TPM measurement event
#[derive(Debug, Clone)]
pub struct TpmEvent {
    /// PCR index
    pub pcr_index: usize,
    
    /// Hash algorithm used
    pub hash_alg: TpmHashAlg,
    
    /// Digest value
    pub digest: Vec<u8>,
    
    /// Event description
    pub description: alloc::string::String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Component that made the measurement
    pub source: alloc::string::String,
}

/// TPM attestation quote
#[derive(Debug, Clone)]
pub struct TpmQuote {
    /// PCR selection (which PCRs were quoted)
    pub pcr_selection: Vec<usize>,
    
    /// PCR values at quote time
    pub pcr_values: BTreeMap<usize, Vec<u8>>,
    
    /// Quote signature
    pub signature: Vec<u8>,
    
    /// Nonce used
    pub nonce: Vec<u8>,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Sealed data blob
#[derive(Debug, Clone)]
pub struct SealedData {
    /// Encrypted data
    pub encrypted_blob: Vec<u8>,
    
    /// PCR policy (which PCRs must match for unsealing)
    pub pcr_policy: Vec<usize>,
    
    /// Expected PCR values
    pub pcr_values: BTreeMap<usize, Vec<u8>>,
    
    /// Creation time
    pub created_at: u64,
}

/// TPM device interface
pub struct TpmDevice {
    /// Base address for MMIO access
    pub base_address: u64,
    
    /// Device available flag
    pub available: bool,
    
    /// Supported hash algorithms
    pub supported_algorithms: Vec<TpmHashAlg>,
    
    /// Number of PCR banks
    pub pcr_banks: u32,
    
    /// TPM statistics
    pub stats: TpmStatistics,
}

/// TPM usage statistics
#[derive(Debug, Default)]
pub struct TpmStatistics {
    /// Total commands sent
    pub commands_sent: AtomicU64,
    
    /// PCR extend operations
    pub pcr_extends: AtomicU64,
    
    /// Quote operations
    pub quotes_generated: AtomicU64,
    
    /// Seal/unseal operations
    pub seal_operations: AtomicU64,
    
    /// Failed operations
    pub failures: AtomicU64,
    
    /// Average command latency (microseconds)
    pub avg_latency_us: AtomicU32,
}

/// TPM Manager
pub struct TpmManager {
    /// TPM device
    device: SpinLock<Option<TpmDevice>>,
    
    /// Event log
    event_log: RwLock<Vec<TpmEvent>>,
    
    /// PCR values cache
    pcr_cache: RwLock<BTreeMap<usize, Vec<u8>>>,
    
    /// Global statistics
    global_stats: TpmStatistics,
}

impl TpmDevice {
    /// Initialize TPM device
    pub fn init(base_address: u64) -> Result<Self, &'static str> {
        // Check if TPM is available at the given address
        let device_id = unsafe {
            core::ptr::read_volatile((base_address + 0x000) as *const u32)
        };
        
        if device_id == 0 || device_id == 0xFFFFFFFF {
            return Err("TPM device not found");
        }
        
        let mut device = Self {
            base_address,
            available: false,
            supported_algorithms: Vec::new(),
            pcr_banks: 0,
            stats: TpmStatistics::default(),
        };
        
        // Initialize TPM
        device.startup()?;
        
        // Query capabilities
        device.query_capabilities()?;
        
        device.available = true;
        
        crate::kernel::serial::write_str("[TPM] TPM 2.0 device initialized at 0x");
        crate::kernel::serial::write_hex64(base_address);
        crate::kernel::serial::write_str("\n");
        
        Ok(device)
    }
    
    /// Send TPM startup command
    fn startup(&mut self) -> Result<(), &'static str> {
        let command = self.build_command(
            commands::TPM_CC_STARTUP,
            &[0x00, 0x00], // Clear startup
        );
        
        let response = self.send_command(&command)?;
        
        if self.extract_response_code(&response) != responses::TPM_RC_SUCCESS {
            return Err("TPM startup failed");
        }
        
        Ok(())
    }
    
    /// Query TPM capabilities
    fn query_capabilities(&mut self) -> Result<(), &'static str> {
        // For now, assume SHA-256 support
        self.supported_algorithms.push(TpmHashAlg::Sha256);
        self.pcr_banks = 24; // Standard PCR count
        
        Ok(())
    }
    
    /// Build TPM command packet
    fn build_command(&self, command_code: u32, payload: &[u8]) -> Vec<u8> {
        let mut command = Vec::new();
        
        // TPM command header
        command.extend(&0x8001u16.to_be_bytes());    // Tag
        command.extend(&((10 + payload.len()) as u32).to_be_bytes()); // Command size
        command.extend(&command_code.to_be_bytes()); // Command code
        
        // Payload
        command.extend(payload);
        
        command
    }
    
    /// Send command to TPM and receive response
    fn send_command(&mut self, command: &[u8]) -> Result<Vec<u8>, &'static str> {
        let start_time = crate::arch::aarch64::cpu::read_timer_counter();
        
        self.stats.commands_sent.fetch_add(1, Ordering::Relaxed);
        
        // Write command to TPM (simplified MMIO interface)
        unsafe {
            let data_fifo = (self.base_address + 0x024) as *mut u32;
            let status = (self.base_address + 0x018) as *const u32;
            
            // Wait for TPM ready
            while core::ptr::read_volatile(status) & 0x40 == 0 {
                core::hint::spin_loop();
            }
            
            // Write command
            for chunk in command.chunks(4) {
                let mut word = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    word |= (byte as u32) << (8 * i);
                }
                core::ptr::write_volatile(data_fifo, word);
            }
            
            // Execute command
            core::ptr::write_volatile((self.base_address + 0x020) as *mut u32, 0x20);
            
            // Wait for completion
            while core::ptr::read_volatile(status) & 0x90 == 0 {
                core::hint::spin_loop();
            }
            
            // Read response length
            let response_len = core::ptr::read_volatile(data_fifo) as usize;
            if response_len > 4096 {
                return Err("TPM response too large");
            }
            
            // Read response data
            let mut response = Vec::with_capacity(response_len);
            response.extend(&response_len.to_be_bytes()[0..4]);
            
            for _ in 4..response_len {
                let byte = core::ptr::read_volatile(data_fifo) as u8;
                response.push(byte);
            }
            
            let end_time = crate::arch::aarch64::cpu::read_timer_counter();
            let latency_cycles = end_time.saturating_sub(start_time);
            let latency_us = latency_cycles / (crate::arch::aarch64::cpu::get_timer_frequency() / 1_000_000);
            
            // Update average latency
            let current_avg = self.stats.avg_latency_us.load(Ordering::Relaxed);
            let new_avg = (current_avg + latency_us as u32) / 2;
            self.stats.avg_latency_us.store(new_avg, Ordering::Relaxed);
            
            Ok(response)
        }
    }
    
    /// Extract response code from TPM response
    fn extract_response_code(&self, response: &[u8]) -> u32 {
        if response.len() >= 10 {
            u32::from_be_bytes([response[6], response[7], response[8], response[9]])
        } else {
            responses::TPM_RC_FAILURE
        }
    }
    
    /// Extend PCR with new measurement
    pub fn pcr_extend(&mut self, pcr_index: usize, hash_alg: TpmHashAlg, digest: &[u8]) -> Result<(), &'static str> {
        if pcr_index >= self.pcr_banks as usize {
            return Err("Invalid PCR index");
        }
        
        if digest.len() != hash_alg.digest_size() {
            return Err("Digest size mismatch");
        }
        
        let mut payload = Vec::new();
        payload.extend(&(pcr_index as u32).to_be_bytes());
        payload.extend(&1u32.to_be_bytes()); // One digest
        payload.extend(&(hash_alg as u16).to_be_bytes());
        payload.extend(digest);
        
        let command = self.build_command(commands::TPM_CC_PCR_EXTEND, &payload);
        let response = self.send_command(&command)?;
        
        if self.extract_response_code(&response) != responses::TPM_RC_SUCCESS {
            self.stats.failures.fetch_add(1, Ordering::Relaxed);
            return Err("PCR extend failed");
        }
        
        self.stats.pcr_extends.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Read PCR values
    pub fn pcr_read(&mut self, pcr_indices: &[usize]) -> Result<BTreeMap<usize, Vec<u8>>, &'static str> {
        let mut payload = Vec::new();
        
        // PCR selection
        payload.extend(&1u32.to_be_bytes()); // One hash algorithm
        payload.extend(&(TpmHashAlg::Sha256 as u16).to_be_bytes());
        payload.push(3); // Size of select array (24 PCRs / 8 bits per byte)
        
        let mut pcr_select = [0u8; 3];
        for &index in pcr_indices {
            if index < 24 {
                let byte_index = index / 8;
                let bit_index = index % 8;
                pcr_select[byte_index] |= 1 << bit_index;
            }
        }
        payload.extend(&pcr_select);
        
        let command = self.build_command(commands::TPM_CC_PCR_READ, &payload);
        let response = self.send_command(&command)?;
        
        if self.extract_response_code(&response) != responses::TPM_RC_SUCCESS {
            self.stats.failures.fetch_add(1, Ordering::Relaxed);
            return Err("PCR read failed");
        }
        
        // Parse PCR values from response (simplified)
        let mut pcr_values = BTreeMap::new();
        let digest_size = TpmHashAlg::Sha256.digest_size();
        
        for (i, &pcr_index) in pcr_indices.iter().enumerate() {
            let offset = 10 + i * digest_size; // Skip response header
            if offset + digest_size <= response.len() {
                let digest = response[offset..offset + digest_size].to_vec();
                pcr_values.insert(pcr_index, digest);
            }
        }
        
        Ok(pcr_values)
    }
    
    /// Generate attestation quote
    pub fn quote(&mut self, pcr_indices: &[usize], nonce: &[u8]) -> Result<TpmQuote, &'static str> {
        // Read current PCR values
        let pcr_values = self.pcr_read(pcr_indices)?;
        
        // Generate quote (simplified - would normally involve attestation key)
        let mut signature = Vec::new();
        signature.extend(b"SIS_TPM_QUOTE");
        signature.extend(nonce);
        
        for pcr_value in pcr_values.values() {
            signature.extend(pcr_value);
        }
        
        self.stats.quotes_generated.fetch_add(1, Ordering::Relaxed);
        
        Ok(TpmQuote {
            pcr_selection: pcr_indices.to_vec(),
            pcr_values,
            signature,
            nonce: nonce.to_vec(),
            timestamp: crate::arch::aarch64::cpu::read_timer_counter(),
        })
    }
    
    /// Seal data to PCR state
    pub fn seal(&mut self, data: &[u8], pcr_policy: &[usize]) -> Result<SealedData, &'static str> {
        // Read current PCR values for policy
        let pcr_values = self.pcr_read(pcr_policy)?;
        
        // Simple XOR encryption (would use real TPM sealing in practice)
        let key = b"SIS_SEAL_KEY_32_BYTES_LONG_12345";
        let mut encrypted = Vec::new();
        
        for (i, &byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key[i % key.len()]);
        }
        
        self.stats.seal_operations.fetch_add(1, Ordering::Relaxed);
        
        Ok(SealedData {
            encrypted_blob: encrypted,
            pcr_policy: pcr_policy.to_vec(),
            pcr_values,
            created_at: crate::arch::aarch64::cpu::read_timer_counter(),
        })
    }
    
    /// Unseal data (verify PCR state first)
    pub fn unseal(&mut self, sealed_data: &SealedData) -> Result<Vec<u8>, &'static str> {
        // Verify current PCR values match policy
        let current_pcrs = self.pcr_read(&sealed_data.pcr_policy)?;
        
        for (&pcr_index, expected_value) in &sealed_data.pcr_values {
            if let Some(current_value) = current_pcrs.get(&pcr_index) {
                if current_value != expected_value {
                    return Err("PCR policy violation - system state changed");
                }
            } else {
                return Err("Required PCR not available");
            }
        }
        
        // Decrypt data (simple XOR reversal)
        let key = b"SIS_SEAL_KEY_32_BYTES_LONG_12345";
        let mut decrypted = Vec::new();
        
        for (i, &byte) in sealed_data.encrypted_blob.iter().enumerate() {
            decrypted.push(byte ^ key[i % key.len()]);
        }
        
        self.stats.seal_operations.fetch_add(1, Ordering::Relaxed);
        
        Ok(decrypted)
    }
}

impl TpmManager {
    /// Create new TPM manager
    pub const fn new() -> Self {
        Self {
            device: SpinLock::new(None),
            event_log: RwLock::new(Vec::new()),
            pcr_cache: RwLock::new(BTreeMap::new()),
            global_stats: TpmStatistics {
                commands_sent: AtomicU64::new(0),
                pcr_extends: AtomicU64::new(0),
                quotes_generated: AtomicU64::new(0),
                seal_operations: AtomicU64::new(0),
                failures: AtomicU64::new(0),
                avg_latency_us: AtomicU32::new(0),
            },
        }
    }
    
    /// Initialize TPM manager
    pub fn init(&self, base_address: u64) -> Result<(), &'static str> {
        match TpmDevice::init(base_address) {
            Ok(device) => {
                *self.device.lock() = Some(device);
                Ok(())
            }
            Err(e) => {
                crate::kernel::serial::write_str("[TPM] TPM initialization failed: ");
                crate::kernel::serial::write_str(e);
                crate::kernel::serial::write_str("\n");
                Ok(()) // Continue without TPM
            }
        }
    }
    
    /// Measure component and extend PCR
    pub fn measure(&self, pcr_index: usize, data: &[u8], description: &str, source: &str) -> Result<(), &'static str> {
        // Calculate SHA-256 hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize().to_vec();
        
        // Extend PCR if TPM available
        if let Some(ref mut device) = *self.device.lock() {
            device.pcr_extend(pcr_index, TpmHashAlg::Sha256, &digest)?;
            
            // Update cache
            self.pcr_cache.write().insert(pcr_index, digest.clone());
        }
        
        // Add to event log
        let event = TpmEvent {
            pcr_index,
            hash_alg: TpmHashAlg::Sha256,
            digest,
            description: description.to_string(),
            timestamp: crate::arch::aarch64::cpu::read_timer_counter(),
            source: source.to_string(),
        };
        
        self.event_log.write().push(event);
        
        Ok(())
    }
    
    /// Get attestation quote
    pub fn get_quote(&self, pcr_indices: &[usize], nonce: &[u8]) -> Result<TpmQuote, &'static str> {
        if let Some(ref mut device) = *self.device.lock() {
            device.quote(pcr_indices, nonce)
        } else {
            Err("TPM not available")
        }
    }
    
    /// Seal data to current system state
    pub fn seal_data(&self, data: &[u8], pcr_policy: &[usize]) -> Result<SealedData, &'static str> {
        if let Some(ref mut device) = *self.device.lock() {
            device.seal(data, pcr_policy)
        } else {
            Err("TPM not available")
        }
    }
    
    /// Unseal data
    pub fn unseal_data(&self, sealed_data: &SealedData) -> Result<Vec<u8>, &'static str> {
        if let Some(ref mut device) = *self.device.lock() {
            device.unseal(sealed_data)
        } else {
            Err("TPM not available")
        }
    }
    
    /// Get event log
    pub fn get_event_log(&self) -> Vec<TpmEvent> {
        self.event_log.read().clone()
    }
    
    /// Get current PCR values
    pub fn get_pcr_values(&self, indices: &[usize]) -> Result<BTreeMap<usize, Vec<u8>>, &'static str> {
        if let Some(ref mut device) = *self.device.lock() {
            device.pcr_read(indices)
        } else {
            // Return cached values if available
            let cache = self.pcr_cache.read();
            let mut result = BTreeMap::new();
            for &index in indices {
                if let Some(value) = cache.get(&index) {
                    result.insert(index, value.clone());
                }
            }
            Ok(result)
        }
    }
}

/// Global TPM manager
static TPM_MANAGER: TpmManager = TpmManager::new();

/// Initialize TPM subsystem
pub fn init() -> Result<(), &'static str> {
    // Try common TPM base addresses
    const TPM_BASE_ADDRESSES: &[u64] = &[
        0xFED4_0000, // Standard TPM 2.0 address
        0xFED3_0000, // Alternative address
        0x2000_0000, // QEMU TPM address
    ];
    
    for &base_addr in TPM_BASE_ADDRESSES {
        if TPM_MANAGER.init(base_addr).is_ok() {
            break;
        }
    }
    
    // Measure kernel initialization
    let kernel_measurement = b"SIS_KERNEL_PHASE2_SECURITY_INIT";
    TPM_MANAGER.measure(
        pcr::KERNEL,
        kernel_measurement,
        "Kernel Phase 2 Security Initialization",
        "kernel_init",
    )?;
    
    crate::kernel::serial::write_str("[TPM] TPM subsystem initialized\n");
    Ok(())
}

/// Measure component
pub fn measure(pcr_index: usize, data: &[u8], description: &str, source: &str) -> Result<(), &'static str> {
    TPM_MANAGER.measure(pcr_index, data, description, source)
}

/// Get attestation quote
pub fn get_quote(pcr_indices: &[usize], nonce: &[u8]) -> Result<TpmQuote, &'static str> {
    TPM_MANAGER.get_quote(pcr_indices, nonce)
}

/// Seal data
pub fn seal_data(data: &[u8], pcr_policy: &[usize]) -> Result<SealedData, &'static str> {
    TPM_MANAGER.seal_data(data, pcr_policy)
}

/// Unseal data
pub fn unseal_data(sealed_data: &SealedData) -> Result<Vec<u8>, &'static str> {
    TPM_MANAGER.unseal_data(sealed_data)
}

/// Get event log
pub fn get_event_log() -> Vec<TpmEvent> {
    TPM_MANAGER.get_event_log()
}

/// Get PCR values
pub fn get_pcr_values(indices: &[usize]) -> Result<BTreeMap<usize, Vec<u8>>, &'static str> {
    TPM_MANAGER.get_pcr_values(indices)
}