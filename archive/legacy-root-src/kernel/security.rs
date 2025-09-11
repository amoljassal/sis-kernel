//! Security Layer Integration
//!
//! This module integrates all Phase 2 security components (TrustZone, Capabilities,
//! TPM, SMMU) into a unified security subsystem for the SIS kernel.
//!
//! Following the DIAMOND architectural principle, this creates balanced
//! security boundaries with controlled interaction chokepoints.

use crate::kernel::{capabilities, tpm, security_test};
use crate::arch::aarch64::{trustzone, smmu};
use crate::kernel::serial;

/// Security subsystem initialization state
static mut SECURITY_INITIALIZED: bool = false;

/// Security configuration
#[derive(Debug, Clone, Copy)]
pub struct SecurityConfig {
    /// Enable TrustZone integration
    pub enable_trustzone: bool,
    
    /// Enable TPM integration
    pub enable_tpm: bool,
    
    /// Enable SMMU DMA isolation
    pub enable_smmu: bool,
    
    /// Run security tests during initialization
    pub run_tests: bool,
    
    /// Security level (1-4, 4 is highest)
    pub security_level: u8,
    
    /// Enable AI-specific security features
    pub ai_security: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_trustzone: true,
            enable_tpm: true,
            enable_smmu: true,
            run_tests: true,
            security_level: 3,
            ai_security: true,
        }
    }
}

/// Initialize the complete security subsystem
pub fn init() -> Result<(), &'static str> {
    init_with_config(SecurityConfig::default())
}

/// Initialize security with custom configuration
pub fn init_with_config(config: SecurityConfig) -> Result<(), &'static str> {
    unsafe {
        if SECURITY_INITIALIZED {
            return Ok(());
        }
    }
    
    serial::write_str("\n");
    serial::write_str("╔══════════════════════════════════════════════════════════════╗\n");
    serial::write_str("║          SIS Kernel Phase 2: Security Layer Init            ║\n");
    serial::write_str("╠══════════════════════════════════════════════════════════════╣\n");
    serial::write_str("║ Initializing comprehensive security architecture...         ║\n");
    
    // Step 1: Initialize capability system (foundational)
    serial::write_str("║ [1/5] Initializing capability-based security system...     ║\n");
    capabilities::init()?;
    
    // Step 2: Initialize TPM for measured boot
    if config.enable_tpm {
        serial::write_str("║ [2/5] Initializing TPM for measured boot & attestation...  ║\n");
        tpm::init()?;
        
        // Measure security initialization
        let security_measurement = format!("SECURITY_INIT_LEVEL_{}", config.security_level);
        tpm::measure(
            tpm::pcr::SECURITY_CONFIG,
            security_measurement.as_bytes(),
            "Security Layer Initialization",
            "security_init",
        )?;
    } else {
        serial::write_str("║ [2/5] TPM disabled - skipping TPM initialization           ║\n");
    }
    
    // Step 3: Initialize TrustZone
    if config.enable_trustzone {
        serial::write_str("║ [3/5] Initializing TrustZone secure world interface...     ║\n");
        match trustzone::init() {
            Ok(()) => {
                serial::write_str("║      TrustZone initialization successful                   ║\n");
            }
            Err(e) => {
                serial::write_str("║      TrustZone initialization failed: ");
                serial::write_str(e);
                serial::write_str("                    ║\n");
                // Continue without TrustZone
            }
        }
    } else {
        serial::write_str("║ [3/5] TrustZone disabled - skipping initialization         ║\n");
    }
    
    // Step 4: Initialize SMMU for DMA isolation
    if config.enable_smmu {
        serial::write_str("║ [4/5] Initializing SMMU for DMA isolation...               ║\n");
        match smmu::init() {
            Ok(()) => {
                serial::write_str("║      SMMU initialization successful                        ║\n");
            }
            Err(e) => {
                serial::write_str("║      SMMU initialization failed: ");
                serial::write_str(e);
                serial::write_str("                       ║\n");
                // Continue without SMMU
            }
        }
    } else {
        serial::write_str("║ [4/5] SMMU disabled - skipping initialization              ║\n");
    }
    
    // Step 5: Run security tests
    if config.run_tests {
        serial::write_str("║ [5/5] Running comprehensive security test suite...         ║\n");
        serial::write_str("╚══════════════════════════════════════════════════════════════╝\n");
        
        let tests_passed = security_test::run_security_tests();
        
        if tests_passed {
            serial::write_str("╔══════════════════════════════════════════════════════════════╗\n");
            serial::write_str("║ ✅ Security Layer Initialization COMPLETE                   ║\n");
            serial::write_str("║                                                              ║\n");
            serial::write_str("║ Security Features Active:                                   ║\n");
            
            if config.enable_trustzone {
                serial::write_str("║  🔐 TrustZone: SMC interface for secure operations          ║\n");
            }
            
            serial::write_str("║  🔑 Capabilities: Fine-grained access control               ║\n");
            
            if config.enable_tpm {
                serial::write_str("║  📊 TPM: Measured boot and attestation                      ║\n");
            }
            
            if config.enable_smmu {
                serial::write_str("║  🛡️  SMMU: DMA isolation and IOVA management                ║\n");
            }
            
            if config.ai_security {
                serial::write_str("║  🤖 AI Security: Model verification and secure inference    ║\n");
            }
            
            serial::write_str("║                                                              ║\n");
            serial::write_str("║ Security Level: ");
            serial::write_u32(config.security_level as u32);
            serial::write_str("/4 (");
            match config.security_level {
                1 => serial::write_str("Basic"),
                2 => serial::write_str("Standard"),
                3 => serial::write_str("High"),
                4 => serial::write_str("Maximum"),
                _ => serial::write_str("Unknown"),
            }
            serial::write_str(")                                  ║\n");
            serial::write_str("║ Ready for Phase 3: AI/ML Runtime implementation            ║\n");
            serial::write_str("╚══════════════════════════════════════════════════════════════╝\n\n");
        } else {
            serial::write_str("╔══════════════════════════════════════════════════════════════╗\n");
            serial::write_str("║ ⚠️  Security Layer Initialization COMPLETED WITH WARNINGS   ║\n");
            serial::write_str("║                                                              ║\n");
            serial::write_str("║ Some security tests failed, but system remains operational. ║\n");
            serial::write_str("║ Review test results above for security implications.        ║\n");
            serial::write_str("╚══════════════════════════════════════════════════════════════╝\n\n");
        }
    } else {
        serial::write_str("║ [5/5] Security tests disabled - skipping validation        ║\n");
        serial::write_str("╚══════════════════════════════════════════════════════════════╝\n\n");
        
        serial::write_str("[SECURITY] Security layer initialized (tests skipped)\n");
    }
    
    unsafe {
        SECURITY_INITIALIZED = true;
    }
    
    Ok(())
}

/// Check if security subsystem is initialized
pub fn is_initialized() -> bool {
    unsafe { SECURITY_INITIALIZED }
}

/// Create secure AI execution context
pub fn create_ai_context(
    model_hash: &[u8; 32],
    max_latency_us: u32,
    security_level: u8,
    process_id: u32,
) -> Result<AiSecurityContext, &'static str> {
    if !is_initialized() {
        return Err("Security subsystem not initialized");
    }
    
    // Create AI-specific capability
    let ai_capability = capabilities::create_ai_capability(
        capabilities::CapabilityType::AIInference,
        capabilities::CapabilityRights::new(
            capabilities::CapabilityRights::AI_INFER | 
            capabilities::CapabilityRights::AI_SECURE
        ),
        *model_hash,
        security_level,
        max_latency_us,
        process_id,
    )?;
    
    // Measure AI model in TPM
    tpm::measure(
        tpm::pcr::AI_MODELS,
        model_hash,
        "AI Model Registration",
        "ai_context",
    )?;
    
    // Verify model through TrustZone if available
    let trustzone_verified = match trustzone::verify_ai_model(model_hash, 4096) {
        Ok(verified) => verified,
        Err(_) => false, // TrustZone not available
    };
    
    // Create SMMU stream for AI accelerator if available
    let stream_id = (process_id * 10) + 1000; // Simple stream ID generation
    let smmu_stream = match smmu::create_stream(stream_id) {
        Ok(asid) => Some((stream_id, asid)),
        Err(_) => None, // SMMU not available
    };
    
    Ok(AiSecurityContext {
        capability_id: ai_capability,
        model_hash: *model_hash,
        security_level,
        max_latency_us,
        process_id,
        trustzone_verified,
        smmu_stream,
        created_at: crate::arch::aarch64::cpu::read_timer_counter(),
    })
}

/// Secure AI execution context
#[derive(Debug, Clone)]
pub struct AiSecurityContext {
    /// Capability ID for this AI context
    pub capability_id: capabilities::CapabilityId,
    
    /// AI model hash
    pub model_hash: [u8; 32],
    
    /// Required security level
    pub security_level: u8,
    
    /// Maximum allowed latency
    pub max_latency_us: u32,
    
    /// Owner process ID
    pub process_id: u32,
    
    /// Whether model was verified by TrustZone
    pub trustzone_verified: bool,
    
    /// SMMU stream information (stream_id, asid)
    pub smmu_stream: Option<(smmu::StreamId, smmu::ASID)>,
    
    /// Creation timestamp
    pub created_at: u64,
}

impl AiSecurityContext {
    /// Check if this context can perform AI inference
    pub fn can_infer(&self) -> bool {
        capabilities::check_capability(
            self.process_id,
            self.capability_id,
            capabilities::CapabilityRights::new(capabilities::CapabilityRights::AI_INFER),
        )
    }
    
    /// Map DMA buffer for AI operations
    pub fn map_ai_buffer(&self, pa: u64, size: usize) -> Result<smmu::IOVA, &'static str> {
        if let Some((stream_id, _asid)) = self.smmu_stream {
            let permissions = smmu::StreamPermissions {
                read: true,
                write: true,
                execute: false,
                privileged: true,
                secure: self.security_level >= 3,
            };
            
            smmu::map_dma(stream_id, pa, size, permissions)
        } else {
            Err("SMMU not available for this context")
        }
    }
    
    /// Unmap DMA buffer
    pub fn unmap_ai_buffer(&self, iova: smmu::IOVA) -> Result<(), &'static str> {
        if let Some((stream_id, _asid)) = self.smmu_stream {
            smmu::unmap_dma(stream_id, iova)
        } else {
            Err("SMMU not available for this context")
        }
    }
    
    /// Get attestation for AI computation
    pub fn get_attestation(&self, nonce: &[u8; 16]) -> Result<Vec<u8>, &'static str> {
        // Create computation hash including model and context
        let mut computation_data = Vec::new();
        computation_data.extend(&self.model_hash);
        computation_data.extend(&self.process_id.to_le_bytes());
        computation_data.extend(&self.security_level.to_le_bytes());
        computation_data.extend(&self.created_at.to_le_bytes());
        
        // Hash the computation context
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&computation_data);
        let computation_hash: [u8; 32] = hasher.finalize().into();
        
        // Get TrustZone attestation if available
        trustzone::get_ai_attestation(nonce, &computation_hash)
    }
    
    /// Validate AI operation timing
    pub fn validate_timing(&self, actual_latency_us: u32) -> bool {
        actual_latency_us <= self.max_latency_us
    }
}

/// Get security statistics
pub fn get_security_stats() -> SecurityStats {
    SecurityStats {
        capabilities: capabilities::get_stats(),
        tpm_events: tpm::get_event_log().len() as u64,
        trustzone_available: trustzone::get_statistics().total_calls.load(core::sync::atomic::Ordering::Relaxed) > 0,
        security_tests_passed: security_test::get_test_stats()
            .map(|stats| stats.failed_tests.load(core::sync::atomic::Ordering::Relaxed) == 0)
            .unwrap_or(false),
    }
}

/// Security subsystem statistics
#[derive(Debug)]
pub struct SecurityStats {
    /// Capability system statistics
    pub capabilities: capabilities::CapabilityStats,
    
    /// Number of TPM events recorded
    pub tpm_events: u64,
    
    /// Whether TrustZone is available
    pub trustzone_available: bool,
    
    /// Whether security tests passed
    pub security_tests_passed: bool,
}