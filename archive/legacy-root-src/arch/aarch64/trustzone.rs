//! ARM TrustZone Secure World Integration
//!
//! This module implements TrustZone secure world setup and SMC (Secure Monitor Call)
//! handling for the SIS kernel security architecture.
//!
//! TrustZone provides hardware-enforced separation between:
//! - Normal World (Rich OS - SIS Kernel)  
//! - Secure World (Trusted OS - AI security services)
//!
//! Geometric Principle: TrustZone creates a DIAMOND boundary where secure and
//! non-secure worlds have equal capabilities but controlled interaction points.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use crate::kernel::sync::SpinLock;

/// SMC function IDs for TrustZone services
pub mod smc {
    /// Standard SMC calling convention function IDs
    pub const CALL_COUNT: u32 = 0xFF00;
    pub const CALL_UID: u32 = 0xFF01;
    pub const CALL_REVISION: u32 = 0xFF03;
    
    /// SIS-specific secure services
    pub const SIS_AI_VERIFY: u32 = 0xC2000000;        // AI model verification
    pub const SIS_AI_ENCRYPT: u32 = 0xC2000001;       // AI data encryption
    pub const SIS_AI_ATTEST: u32 = 0xC2000002;        // AI computation attestation
    pub const SIS_KEY_DERIVE: u32 = 0xC2000003;       // Key derivation service
    pub const SIS_SECURE_RAND: u32 = 0xC2000004;      // Hardware random numbers
    pub const SIS_TPM_QUOTE: u32 = 0xC2000005;        // TPM attestation quote
    
    /// Return codes
    pub const SUCCESS: i64 = 0;
    pub const NOT_SUPPORTED: i64 = -1;
    pub const INVALID_PARAMS: i64 = -2;
    pub const NO_MEMORY: i64 = -3;
    pub const ALREADY_DONE: i64 = -4;
    pub const NOT_PERMITTED: i64 = -5;
}

/// TrustZone configuration and state
pub struct TrustZone {
    /// Whether TrustZone is available
    pub available: bool,
    /// Secure world entry point
    pub secure_entry: u64,
    /// SMC call statistics
    pub smc_stats: SmcStatistics,
    /// Security level configuration
    pub security_config: SecurityConfig,
}

/// SMC call statistics for monitoring
#[derive(Debug, Default)]
pub struct SmcStatistics {
    /// Total SMC calls made
    pub total_calls: AtomicU64,
    /// AI verification calls
    pub ai_verify_calls: AtomicU64,
    /// Key derivation calls  
    pub key_derive_calls: AtomicU64,
    /// Failed calls
    pub failed_calls: AtomicU64,
    /// Average latency in cycles
    pub avg_latency_cycles: AtomicU32,
}

/// Security configuration
#[derive(Debug, Clone, Copy)]
pub struct SecurityConfig {
    /// Enable AI model verification
    pub ai_model_verification: bool,
    /// Enable secure key storage
    pub secure_key_storage: bool,
    /// Enable hardware attestation
    pub hardware_attestation: bool,
    /// Security level (1-4)
    pub security_level: u8,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            ai_model_verification: true,
            secure_key_storage: true,
            hardware_attestation: true,
            security_level: 3, // High security by default
        }
    }
}

/// SMC call parameters
#[derive(Debug, Clone, Copy)]
pub struct SmcCall {
    pub function_id: u32,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64,
}

/// SMC call result
#[derive(Debug, Clone, Copy)]
pub struct SmcResult {
    pub ret0: i64,
    pub ret1: u64,
    pub ret2: u64,
    pub ret3: u64,
}

/// Global TrustZone instance
static mut TRUSTZONE: TrustZone = TrustZone::new();
static TRUSTZONE_LOCK: SpinLock<()> = SpinLock::new(());

impl TrustZone {
    /// Create new TrustZone instance
    const fn new() -> Self {
        Self {
            available: false,
            secure_entry: 0,
            smc_stats: SmcStatistics {
                total_calls: AtomicU64::new(0),
                ai_verify_calls: AtomicU64::new(0),
                key_derive_calls: AtomicU64::new(0),
                failed_calls: AtomicU64::new(0),
                avg_latency_cycles: AtomicU32::new(0),
            },
            security_config: SecurityConfig {
                ai_model_verification: true,
                secure_key_storage: true,
                hardware_attestation: true,
                security_level: 3,
            },
        }
    }
    
    /// Initialize TrustZone support
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Check if we're running at EL1 (required for SMC)
        let current_el = Self::get_current_el();
        if current_el != 1 {
            return Err("TrustZone requires EL1");
        }
        
        // Check if TrustZone is implemented
        let id_aa64pfr0: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, id_aa64pfr0_el1",
                out(reg) id_aa64pfr0,
                options(nomem, nostack)
            );
        }
        
        // Check EL3 implementation (bits 15:12)
        let el3_impl = (id_aa64pfr0 >> 12) & 0xF;
        if el3_impl == 0xF {
            return Err("EL3 not implemented - TrustZone unavailable");
        }
        
        // Test basic SMC functionality
        let test_result = self.call_smc(SmcCall {
            function_id: smc::CALL_COUNT,
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            arg6: 0,
        });
        
        match test_result.ret0 {
            smc::SUCCESS => {
                self.available = true;
                crate::kernel::serial::write_str("[TRUSTZONE] TrustZone available, SMC functional\n");
            }
            smc::NOT_SUPPORTED => {
                crate::kernel::serial::write_str("[TRUSTZONE] TrustZone present but no secure services\n");
                self.available = false;
            }
            _ => {
                crate::kernel::serial::write_str("[TRUSTZONE] TrustZone test failed\n");
                self.available = false;
            }
        }
        
        Ok(())
    }
    
    /// Get current exception level
    fn get_current_el() -> u32 {
        let current_el: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, currentel",
                out(reg) current_el,
                options(nomem, nostack)
            );
        }
        ((current_el >> 2) & 0x3) as u32
    }
    
    /// Make SMC call to secure world
    pub fn call_smc(&self, call: SmcCall) -> SmcResult {
        let start_cycles = crate::arch::aarch64::pmu::Pmu::read_cycle_counter();
        
        self.smc_stats.total_calls.fetch_add(1, Ordering::Relaxed);
        
        let (ret0, ret1, ret2, ret3): (i64, u64, u64, u64);
        
        unsafe {
            core::arch::asm!(
                "smc #0",
                inout("x0") call.function_id as u64 => ret0 as u64,
                inout("x1") call.arg0 => ret1,
                inout("x2") call.arg1 => ret2,
                inout("x3") call.arg2 => ret3,
                in("x4") call.arg3,
                in("x5") call.arg4,
                in("x6") call.arg5,
                in("x7") call.arg6,
                options(nomem, nostack)
            );
        }
        
        let end_cycles = crate::arch::aarch64::pmu::Pmu::read_cycle_counter();
        let latency = end_cycles.saturating_sub(start_cycles) as u32;
        
        // Update average latency (simple moving average)
        let current_avg = self.smc_stats.avg_latency_cycles.load(Ordering::Relaxed);
        let new_avg = (current_avg + latency) / 2;
        self.smc_stats.avg_latency_cycles.store(new_avg, Ordering::Relaxed);
        
        if ret0 < 0 {
            self.smc_stats.failed_calls.fetch_add(1, Ordering::Relaxed);
        }
        
        SmcResult {
            ret0,
            ret1,
            ret2,
            ret3,
        }
    }
    
    /// Verify AI model in secure world
    pub fn verify_ai_model(&self, model_hash: &[u8; 32], model_size: usize) -> Result<bool, &'static str> {
        if !self.available || !self.security_config.ai_model_verification {
            return Err("AI verification not available");
        }
        
        self.smc_stats.ai_verify_calls.fetch_add(1, Ordering::Relaxed);
        
        // Pass hash and size to secure world
        let hash_ptr = model_hash.as_ptr() as u64;
        let result = self.call_smc(SmcCall {
            function_id: smc::SIS_AI_VERIFY,
            arg0: hash_ptr,
            arg1: model_size as u64,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            arg6: 0,
        });
        
        match result.ret0 {
            smc::SUCCESS => Ok(result.ret1 == 1), // ret1 contains verification result
            smc::INVALID_PARAMS => Err("Invalid model parameters"),
            smc::NOT_PERMITTED => Err("Model verification not permitted"),
            _ => Err("AI model verification failed"),
        }
    }
    
    /// Derive secure key for AI operations
    pub fn derive_ai_key(&self, context: &str, key_id: u32) -> Result<[u8; 32], &'static str> {
        if !self.available || !self.security_config.secure_key_storage {
            return Err("Key derivation not available");
        }
        
        self.smc_stats.key_derive_calls.fetch_add(1, Ordering::Relaxed);
        
        // Create context hash
        let context_bytes = context.as_bytes();
        let context_ptr = context_bytes.as_ptr() as u64;
        
        // Allocate secure memory for derived key
        let mut derived_key = [0u8; 32];
        let key_ptr = derived_key.as_mut_ptr() as u64;
        
        let result = self.call_smc(SmcCall {
            function_id: smc::SIS_KEY_DERIVE,
            arg0: key_id as u64,
            arg1: context_ptr,
            arg2: context_bytes.len() as u64,
            arg3: key_ptr,
            arg4: 32, // Key length
            arg5: 0,
            arg6: 0,
        });
        
        match result.ret0 {
            smc::SUCCESS => Ok(derived_key),
            smc::INVALID_PARAMS => Err("Invalid key derivation parameters"),
            smc::NO_MEMORY => Err("Secure memory allocation failed"),
            _ => Err("Key derivation failed"),
        }
    }
    
    /// Get hardware random numbers from secure world
    pub fn get_secure_random(&self, buffer: &mut [u8]) -> Result<(), &'static str> {
        if !self.available {
            return Err("Secure random not available");
        }
        
        let buffer_ptr = buffer.as_mut_ptr() as u64;
        let result = self.call_smc(SmcCall {
            function_id: smc::SIS_SECURE_RAND,
            arg0: buffer_ptr,
            arg1: buffer.len() as u64,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            arg6: 0,
        });
        
        match result.ret0 {
            smc::SUCCESS => Ok(()),
            smc::INVALID_PARAMS => Err("Invalid random buffer parameters"),
            _ => Err("Secure random generation failed"),
        }
    }
    
    /// Get attestation quote for AI computation
    pub fn get_ai_attestation(&self, nonce: &[u8; 16], computation_hash: &[u8; 32]) -> Result<Vec<u8>, &'static str> {
        if !self.available || !self.security_config.hardware_attestation {
            return Err("Hardware attestation not available");
        }
        
        let nonce_ptr = nonce.as_ptr() as u64;
        let hash_ptr = computation_hash.as_ptr() as u64;
        
        // Allocate buffer for attestation quote (typical size ~1KB)
        let mut quote_buffer = alloc::vec![0u8; 1024];
        let quote_ptr = quote_buffer.as_mut_ptr() as u64;
        
        let result = self.call_smc(SmcCall {
            function_id: smc::SIS_TPM_QUOTE,
            arg0: nonce_ptr,
            arg1: hash_ptr,
            arg2: quote_ptr,
            arg3: quote_buffer.len() as u64,
            arg4: 0,
            arg5: 0,
            arg6: 0,
        });
        
        match result.ret0 {
            smc::SUCCESS => {
                let actual_size = result.ret1 as usize;
                quote_buffer.truncate(actual_size);
                Ok(quote_buffer)
            }
            smc::INVALID_PARAMS => Err("Invalid attestation parameters"),
            smc::NO_MEMORY => Err("Attestation buffer too small"),
            _ => Err("Hardware attestation failed"),
        }
    }
    
    /// Get SMC statistics
    pub fn get_statistics(&self) -> &SmcStatistics {
        &self.smc_stats
    }
}

/// Initialize TrustZone support
pub fn init() -> Result<(), &'static str> {
    let _lock = TRUSTZONE_LOCK.lock();
    
    unsafe {
        TRUSTZONE.init()?;
    }
    
    crate::kernel::serial::write_str("[TRUSTZONE] Security boundaries established\n");
    Ok(())
}

/// Verify AI model using TrustZone
pub fn verify_ai_model(model_hash: &[u8; 32], model_size: usize) -> Result<bool, &'static str> {
    let _lock = TRUSTZONE_LOCK.lock();
    unsafe {
        TRUSTZONE.verify_ai_model(model_hash, model_size)
    }
}

/// Derive secure key for AI operations
pub fn derive_ai_key(context: &str, key_id: u32) -> Result<[u8; 32], &'static str> {
    let _lock = TRUSTZONE_LOCK.lock();
    unsafe {
        TRUSTZONE.derive_ai_key(context, key_id)
    }
}

/// Get secure random bytes
pub fn get_secure_random(buffer: &mut [u8]) -> Result<(), &'static str> {
    let _lock = TRUSTZONE_LOCK.lock();
    unsafe {
        TRUSTZONE.get_secure_random(buffer)
    }
}

/// Get hardware attestation for AI computation
pub fn get_ai_attestation(nonce: &[u8; 16], computation_hash: &[u8; 32]) -> Result<Vec<u8>, &'static str> {
    let _lock = TRUSTZONE_LOCK.lock();
    unsafe {
        TRUSTZONE.get_ai_attestation(nonce, computation_hash)
    }
}

/// Get TrustZone statistics
pub fn get_statistics() -> SmcStatistics {
    let _lock = TRUSTZONE_LOCK.lock();
    unsafe {
        SmcStatistics {
            total_calls: AtomicU64::new(TRUSTZONE.smc_stats.total_calls.load(Ordering::Relaxed)),
            ai_verify_calls: AtomicU64::new(TRUSTZONE.smc_stats.ai_verify_calls.load(Ordering::Relaxed)),
            key_derive_calls: AtomicU64::new(TRUSTZONE.smc_stats.key_derive_calls.load(Ordering::Relaxed)),
            failed_calls: AtomicU64::new(TRUSTZONE.smc_stats.failed_calls.load(Ordering::Relaxed)),
            avg_latency_cycles: AtomicU32::new(TRUSTZONE.smc_stats.avg_latency_cycles.load(Ordering::Relaxed)),
        }
    }
}