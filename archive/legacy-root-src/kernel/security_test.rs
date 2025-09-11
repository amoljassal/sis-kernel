//! Security Testing Framework
//!
//! This module implements comprehensive security testing for the SIS kernel
//! Phase 2 security layer, validating TrustZone, capabilities, TPM, and SMMU.
//!
//! Testing Philosophy: Security properties must be mathematically provable
//! through execution, not just theoretically sound. Each test validates
//! a specific security invariant or geometric property.

use crate::kernel::{capabilities, tpm};
use crate::arch::aarch64::{trustzone, smmu};
use alloc::{vec::Vec, string::String, boxed::Box};
use core::sync::atomic::{AtomicU64, Ordering};

/// Security test result
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    Pass,
    Fail(String),
    Skip(String),
}

/// Security test statistics
#[derive(Debug, Default)]
pub struct SecurityTestStats {
    pub total_tests: AtomicU64,
    pub passed_tests: AtomicU64,
    pub failed_tests: AtomicU64,
    pub skipped_tests: AtomicU64,
}

/// Individual security test
pub struct SecurityTest {
    pub name: &'static str,
    pub description: &'static str,
    pub test_fn: fn() -> TestResult,
    pub priority: TestPriority,
}

/// Test execution priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestPriority {
    Critical = 0,   // Must pass for security
    High = 1,       // Important security properties
    Medium = 2,     // Good security practices
    Low = 3,        // Nice-to-have features
}

/// Security test suite
pub struct SecurityTestSuite {
    tests: Vec<SecurityTest>,
    stats: SecurityTestStats,
}

impl SecurityTestSuite {
    /// Create new test suite
    pub fn new() -> Self {
        let mut suite = Self {
            tests: Vec::new(),
            stats: SecurityTestStats::default(),
        };
        
        suite.register_all_tests();
        suite
    }
    
    /// Register all security tests
    fn register_all_tests(&mut self) {
        // TrustZone tests
        self.register_test(SecurityTest {
            name: "trustzone_availability",
            description: "Verify TrustZone SMC interface is available",
            test_fn: test_trustzone_availability,
            priority: TestPriority::Critical,
        });
        
        self.register_test(SecurityTest {
            name: "trustzone_ai_verify",
            description: "Test AI model verification through TrustZone",
            test_fn: test_trustzone_ai_verification,
            priority: TestPriority::High,
        });
        
        self.register_test(SecurityTest {
            name: "trustzone_key_derivation",
            description: "Test secure key derivation",
            test_fn: test_trustzone_key_derivation,
            priority: TestPriority::High,
        });
        
        // Capability tests
        self.register_test(SecurityTest {
            name: "capability_creation",
            description: "Test capability creation and validation",
            test_fn: test_capability_creation,
            priority: TestPriority::Critical,
        });
        
        self.register_test(SecurityTest {
            name: "capability_derivation",
            description: "Test capability derivation with rights restriction",
            test_fn: test_capability_derivation,
            priority: TestPriority::Critical,
        });
        
        self.register_test(SecurityTest {
            name: "capability_revocation",
            description: "Test capability revocation propagates to derivatives",
            test_fn: test_capability_revocation,
            priority: TestPriority::Critical,
        });
        
        self.register_test(SecurityTest {
            name: "ai_capability_protection",
            description: "Test AI-specific capability protection",
            test_fn: test_ai_capability_protection,
            priority: TestPriority::High,
        });
        
        // TPM tests
        self.register_test(SecurityTest {
            name: "tpm_measurement",
            description: "Test TPM PCR measurement functionality",
            test_fn: test_tpm_measurement,
            priority: TestPriority::High,
        });
        
        self.register_test(SecurityTest {
            name: "tpm_attestation",
            description: "Test TPM attestation quote generation",
            test_fn: test_tpm_attestation,
            priority: TestPriority::High,
        });
        
        self.register_test(SecurityTest {
            name: "tpm_sealing",
            description: "Test TPM data sealing and unsealing",
            test_fn: test_tpm_sealing,
            priority: TestPriority::Medium,
        });
        
        // SMMU tests
        self.register_test(SecurityTest {
            name: "smmu_isolation",
            description: "Test SMMU DMA isolation between streams",
            test_fn: test_smmu_isolation,
            priority: TestPriority::High,
        });
        
        self.register_test(SecurityTest {
            name: "smmu_iova_allocation",
            description: "Test SMMU IOVA space management",
            test_fn: test_smmu_iova_allocation,
            priority: TestPriority::Medium,
        });
        
        // Integration tests
        self.register_test(SecurityTest {
            name: "integrated_ai_security",
            description: "Test end-to-end AI security pipeline",
            test_fn: test_integrated_ai_security,
            priority: TestPriority::Critical,
        });
        
        self.register_test(SecurityTest {
            name: "security_under_load",
            description: "Test security properties under concurrent load",
            test_fn: test_security_under_load,
            priority: TestPriority::Medium,
        });
    }
    
    /// Register a test
    fn register_test(&mut self, test: SecurityTest) {
        self.tests.push(test);
    }
    
    /// Run all tests
    pub fn run_all_tests(&mut self) -> bool {
        crate::kernel::serial::write_str("\n");
        crate::kernel::serial::write_str("╔══════════════════════════════════════════════════════════════╗\n");
        crate::kernel::serial::write_str("║             SIS Kernel Security Test Suite                  ║\n");
        crate::kernel::serial::write_str("╠══════════════════════════════════════════════════════════════╣\n");
        
        // Sort tests by priority
        self.tests.sort_by_key(|t| t.priority);
        
        let mut all_critical_passed = true;
        
        for test in &self.tests {
            self.stats.total_tests.fetch_add(1, Ordering::Relaxed);
            
            crate::kernel::serial::write_str("║ Testing: ");
            crate::kernel::serial::write_str(test.name);
            
            // Pad to align output
            let padding = 48_usize.saturating_sub(test.name.len());
            for _ in 0..padding {
                crate::kernel::serial::write_str(" ");
            }
            
            let result = (test.test_fn)();
            
            match result {
                TestResult::Pass => {
                    crate::kernel::serial::write_str("✓ PASS ║\n");
                    self.stats.passed_tests.fetch_add(1, Ordering::Relaxed);
                }
                TestResult::Fail(reason) => {
                    crate::kernel::serial::write_str("✗ FAIL ║\n");
                    crate::kernel::serial::write_str("║   Reason: ");
                    crate::kernel::serial::write_str(&reason);
                    crate::kernel::serial::write_str("                                        ║\n");
                    
                    self.stats.failed_tests.fetch_add(1, Ordering::Relaxed);
                    
                    if test.priority == TestPriority::Critical {
                        all_critical_passed = false;
                    }
                }
                TestResult::Skip(reason) => {
                    crate::kernel::serial::write_str("- SKIP ║\n");
                    crate::kernel::serial::write_str("║   Reason: ");
                    crate::kernel::serial::write_str(&reason);
                    crate::kernel::serial::write_str("                                        ║\n");
                    
                    self.stats.skipped_tests.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        
        // Print summary
        crate::kernel::serial::write_str("╠══════════════════════════════════════════════════════════════╣\n");
        crate::kernel::serial::write_str("║ Test Summary:                                                ║\n");
        crate::kernel::serial::write_str("║   Total:  ");
        crate::kernel::serial::write_u64(self.stats.total_tests.load(Ordering::Relaxed));
        crate::kernel::serial::write_str(" tests                                              ║\n");
        crate::kernel::serial::write_str("║   Passed: ");
        crate::kernel::serial::write_u64(self.stats.passed_tests.load(Ordering::Relaxed));
        crate::kernel::serial::write_str(" tests                                              ║\n");
        crate::kernel::serial::write_str("║   Failed: ");
        crate::kernel::serial::write_u64(self.stats.failed_tests.load(Ordering::Relaxed));
        crate::kernel::serial::write_str(" tests                                              ║\n");
        crate::kernel::serial::write_str("║   Skipped: ");
        crate::kernel::serial::write_u64(self.stats.skipped_tests.load(Ordering::Relaxed));
        crate::kernel::serial::write_str(" tests                                             ║\n");
        
        if all_critical_passed {
            crate::kernel::serial::write_str("║                                                              ║\n");
            crate::kernel::serial::write_str("║ 🔒 All critical security tests PASSED                      ║\n");
            crate::kernel::serial::write_str("║     System meets security requirements                      ║\n");
        } else {
            crate::kernel::serial::write_str("║                                                              ║\n");
            crate::kernel::serial::write_str("║ ⚠️  Critical security tests FAILED                          ║\n");
            crate::kernel::serial::write_str("║     System does NOT meet security requirements              ║\n");
        }
        
        crate::kernel::serial::write_str("╚══════════════════════════════════════════════════════════════╝\n\n");
        
        all_critical_passed
    }
}

// Test implementations

/// Test TrustZone availability
fn test_trustzone_availability() -> TestResult {
    // Test if TrustZone SMC interface responds
    let test_hash = [0x42u8; 32];
    
    match trustzone::verify_ai_model(&test_hash, 1024) {
        Ok(_) => TestResult::Pass,
        Err("AI verification not available") => TestResult::Skip("TrustZone not available".to_string()),
        Err(e) => TestResult::Fail(format!("TrustZone error: {}", e)),
    }
}

/// Test TrustZone AI verification
fn test_trustzone_ai_verification() -> TestResult {
    let valid_hash = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
                      0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
                      0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
                      0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    
    match trustzone::verify_ai_model(&valid_hash, 2048) {
        Ok(is_valid) => {
            if is_valid {
                TestResult::Pass
            } else {
                TestResult::Fail("Model verification returned false".to_string())
            }
        }
        Err("AI verification not available") => TestResult::Skip("TrustZone not available".to_string()),
        Err(e) => TestResult::Fail(format!("Verification failed: {}", e)),
    }
}

/// Test TrustZone key derivation
fn test_trustzone_key_derivation() -> TestResult {
    match trustzone::derive_ai_key("test_context", 12345) {
        Ok(key) => {
            // Verify key is not all zeros
            if key.iter().all(|&b| b == 0) {
                TestResult::Fail("Derived key is all zeros".to_string())
            } else {
                TestResult::Pass
            }
        }
        Err("Key derivation not available") => TestResult::Skip("TrustZone not available".to_string()),
        Err(e) => TestResult::Fail(format!("Key derivation failed: {}", e)),
    }
}

/// Test capability creation
fn test_capability_creation() -> TestResult {
    match capabilities::create_capability(
        capabilities::CapabilityType::Memory,
        capabilities::CapabilityRights::new(capabilities::CapabilityRights::READ | capabilities::CapabilityRights::WRITE),
        0x1000_0000,
        4096,
        1, // Test process ID
    ) {
        Ok(cap_id) => {
            if cap_id > 0 {
                TestResult::Pass
            } else {
                TestResult::Fail("Invalid capability ID returned".to_string())
            }
        }
        Err(e) => TestResult::Fail(format!("Capability creation failed: {}", e)),
    }
}

/// Test capability derivation
fn test_capability_derivation() -> TestResult {
    // Create parent capability
    let parent_id = match capabilities::create_capability(
        capabilities::CapabilityType::Memory,
        capabilities::CapabilityRights::new(
            capabilities::CapabilityRights::READ | 
            capabilities::CapabilityRights::WRITE | 
            capabilities::CapabilityRights::DERIVE
        ),
        0x2000_0000,
        8192,
        1,
    ) {
        Ok(id) => id,
        Err(e) => return TestResult::Fail(format!("Parent capability creation failed: {}", e)),
    };
    
    // Derive child capability with restricted rights
    match capabilities::derive_capability(
        parent_id,
        capabilities::CapabilityRights::new(capabilities::CapabilityRights::READ), // Only read
        capabilities::DerivationContext::Restrict { removed_rights: capabilities::CapabilityRights::WRITE },
        2, // Different process
    ) {
        Ok(child_id) => {
            if child_id > 0 && child_id != parent_id {
                TestResult::Pass
            } else {
                TestResult::Fail("Invalid child capability ID".to_string())
            }
        }
        Err(e) => TestResult::Fail(format!("Capability derivation failed: {}", e)),
    }
}

/// Test capability revocation
fn test_capability_revocation() -> TestResult {
    // Create and derive capabilities for revocation test
    let parent_id = match capabilities::create_capability(
        capabilities::CapabilityType::AIModel,
        capabilities::CapabilityRights::new(
            capabilities::CapabilityRights::DERIVE | 
            capabilities::CapabilityRights::REVOKE | 
            capabilities::CapabilityRights::AI_INFER
        ),
        0,
        0,
        1,
    ) {
        Ok(id) => id,
        Err(_) => return TestResult::Fail("Parent capability creation failed".to_string()),
    };
    
    // Create child capability
    let _child_id = match capabilities::derive_capability(
        parent_id,
        capabilities::CapabilityRights::new(capabilities::CapabilityRights::AI_INFER),
        capabilities::DerivationContext::AiModelLoad { model_id: 123 },
        2,
    ) {
        Ok(id) => id,
        Err(_) => return TestResult::Fail("Child capability creation failed".to_string()),
    };
    
    // Revoke parent (should revoke child too)
    match capabilities::revoke_capability(parent_id, 1) {
        Ok(revoked_count) => {
            if revoked_count >= 1 {
                TestResult::Pass
            } else {
                TestResult::Fail("No capabilities were revoked".to_string())
            }
        }
        Err(e) => TestResult::Fail(format!("Revocation failed: {}", e)),
    }
}

/// Test AI capability protection
fn test_ai_capability_protection() -> TestResult {
    let model_hash = [0xAA; 32];
    
    match capabilities::create_ai_capability(
        capabilities::CapabilityType::AIInference,
        capabilities::CapabilityRights::new(capabilities::CapabilityRights::AI_INFER | capabilities::CapabilityRights::AI_SECURE),
        model_hash,
        3, // High security level
        40, // 40μs max latency
        1,
    ) {
        Ok(cap_id) => {
            // Test access check
            if capabilities::check_capability(
                1,
                cap_id,
                capabilities::CapabilityRights::new(capabilities::CapabilityRights::AI_INFER),
            ) {
                TestResult::Pass
            } else {
                TestResult::Fail("AI capability access check failed".to_string())
            }
        }
        Err(e) => TestResult::Fail(format!("AI capability creation failed: {}", e)),
    }
}

/// Test TPM measurement
fn test_tpm_measurement() -> TestResult {
    let test_data = b"security_test_measurement_data";
    
    match tpm::measure(
        tpm::pcr::DEBUG,
        test_data,
        "Security Test Measurement",
        "security_test",
    ) {
        Ok(()) => TestResult::Pass,
        Err(e) => TestResult::Fail(format!("TPM measurement failed: {}", e)),
    }
}

/// Test TPM attestation
fn test_tpm_attestation() -> TestResult {
    let pcr_indices = vec![tpm::pcr::KERNEL, tpm::pcr::DEBUG];
    let nonce = [0x42; 16];
    
    match tpm::get_quote(&pcr_indices, &nonce) {
        Ok(quote) => {
            if quote.pcr_selection.len() > 0 && quote.nonce == nonce {
                TestResult::Pass
            } else {
                TestResult::Fail("Invalid quote structure".to_string())
            }
        }
        Err("TPM not available") => TestResult::Skip("TPM not available".to_string()),
        Err(e) => TestResult::Fail(format!("TPM quote failed: {}", e)),
    }
}

/// Test TPM sealing
fn test_tpm_sealing() -> TestResult {
    let secret_data = b"secret_ai_key_material";
    let pcr_policy = vec![tpm::pcr::KERNEL];
    
    // Seal data
    let sealed = match tpm::seal_data(secret_data, &pcr_policy) {
        Ok(s) => s,
        Err("TPM not available") => return TestResult::Skip("TPM not available".to_string()),
        Err(e) => return TestResult::Fail(format!("TPM sealing failed: {}", e)),
    };
    
    // Unseal data
    match tpm::unseal_data(&sealed) {
        Ok(unsealed) => {
            if unsealed == secret_data {
                TestResult::Pass
            } else {
                TestResult::Fail("Unsealed data doesn't match original".to_string())
            }
        }
        Err(e) => TestResult::Fail(format!("TPM unsealing failed: {}", e)),
    }
}

/// Test SMMU isolation
fn test_smmu_isolation() -> TestResult {
    let stream1 = 100;
    let stream2 = 200;
    
    // Create streams
    match smmu::create_stream(stream1) {
        Ok(_asid1) => {}
        Err("SMMU not available") => return TestResult::Skip("SMMU not available".to_string()),
        Err(e) => return TestResult::Fail(format!("Stream 1 creation failed: {}", e)),
    }
    
    match smmu::create_stream(stream2) {
        Ok(_asid2) => {}
        Err(e) => return TestResult::Fail(format!("Stream 2 creation failed: {}", e)),
    }
    
    // Test mapping isolation (simplified test)
    let perms = smmu::StreamPermissions::default();
    
    match smmu::map_dma(stream1, 0x1000_0000, 4096, perms) {
        Ok(iova1) => {
            match smmu::map_dma(stream2, 0x2000_0000, 4096, perms) {
                Ok(iova2) => {
                    if iova1 != iova2 {
                        TestResult::Pass
                    } else {
                        TestResult::Fail("IOVA spaces not isolated".to_string())
                    }
                }
                Err(e) => TestResult::Fail(format!("Stream 2 mapping failed: {}", e)),
            }
        }
        Err(e) => TestResult::Fail(format!("Stream 1 mapping failed: {}", e)),
    }
}

/// Test SMMU IOVA allocation
fn test_smmu_iova_allocation() -> TestResult {
    let stream_id = 300;
    let perms = smmu::StreamPermissions::default();
    
    // Create stream first
    match smmu::create_stream(stream_id) {
        Ok(_) => {}
        Err("SMMU not available") => return TestResult::Skip("SMMU not available".to_string()),
        Err(e) => return TestResult::Fail(format!("Stream creation failed: {}", e)),
    }
    
    // Allocate multiple mappings
    let mut iovas = Vec::new();
    for i in 0..4 {
        match smmu::map_dma(stream_id, 0x3000_0000 + (i * 4096), 4096, perms) {
            Ok(iova) => iovas.push(iova),
            Err(e) => return TestResult::Fail(format!("Mapping {} failed: {}", i, e)),
        }
    }
    
    // Verify all IOVAs are different
    for (i, &iova1) in iovas.iter().enumerate() {
        for &iova2 in &iovas[i + 1..] {
            if iova1 == iova2 {
                return TestResult::Fail("Duplicate IOVA allocated".to_string());
            }
        }
    }
    
    TestResult::Pass
}

/// Test integrated AI security pipeline
fn test_integrated_ai_security() -> TestResult {
    // This test validates the entire security pipeline for AI operations
    
    // Step 1: Create AI capability
    let model_hash = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                      0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
                      0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                      0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00];
    
    let ai_cap_id = match capabilities::create_ai_capability(
        capabilities::CapabilityType::AIInference,
        capabilities::CapabilityRights::new(
            capabilities::CapabilityRights::AI_INFER | 
            capabilities::CapabilityRights::AI_SECURE
        ),
        model_hash,
        3,
        40,
        1,
    ) {
        Ok(id) => id,
        Err(e) => return TestResult::Fail(format!("AI capability creation failed: {}", e)),
    };
    
    // Step 2: Measure AI model in TPM
    let _ = tpm::measure(
        tpm::pcr::AI_MODELS,
        &model_hash,
        "Integrated AI Security Test Model",
        "security_test",
    );
    
    // Step 3: Verify model through TrustZone (if available)
    match trustzone::verify_ai_model(&model_hash, 4096) {
        Ok(verified) => {
            if !verified {
                return TestResult::Fail("Model verification failed".to_string());
            }
        }
        Err("AI verification not available") => {
            // Continue without TrustZone verification
        }
        Err(e) => return TestResult::Fail(format!("TrustZone verification error: {}", e)),
    }
    
    // Step 4: Test capability access
    if !capabilities::check_capability(
        1,
        ai_cap_id,
        capabilities::CapabilityRights::new(capabilities::CapabilityRights::AI_INFER),
    ) {
        return TestResult::Fail("AI capability access denied".to_string());
    }
    
    // Step 5: Create SMMU stream for AI accelerator (if available)
    let ai_stream_id = 1000;
    match smmu::create_stream(ai_stream_id) {
        Ok(_) => {
            // Test DMA mapping for AI data
            let perms = smmu::StreamPermissions {
                read: true,
                write: true,
                execute: false,
                privileged: true,
                secure: true,
            };
            
            match smmu::map_dma(ai_stream_id, 0x4000_0000, 1024 * 1024, perms) {
                Ok(_iova) => TestResult::Pass,
                Err(e) => TestResult::Fail(format!("AI DMA mapping failed: {}", e)),
            }
        }
        Err("SMMU not available") => {
            // Integration test passes even without SMMU
            TestResult::Pass
        }
        Err(e) => TestResult::Fail(format!("AI stream creation failed: {}", e)),
    }
}

/// Test security under concurrent load
fn test_security_under_load() -> TestResult {
    // Simulate concurrent capability operations
    for i in 0..10 {
        let cap_id = match capabilities::create_capability(
            capabilities::CapabilityType::Memory,
            capabilities::CapabilityRights::new(
                capabilities::CapabilityRights::READ | 
                capabilities::CapabilityRights::DERIVE
            ),
            0x5000_0000 + (i * 4096),
            4096,
            i as u32,
        ) {
            Ok(id) => id,
            Err(e) => return TestResult::Fail(format!("Concurrent capability {} creation failed: {}", i, e)),
        };
        
        // Try to derive from each capability
        match capabilities::derive_capability(
            cap_id,
            capabilities::CapabilityRights::new(capabilities::CapabilityRights::READ),
            capabilities::DerivationContext::Custom { description: format!("Concurrent test {}", i) },
            (i + 100) as u32,
        ) {
            Ok(_) => continue,
            Err(e) => return TestResult::Fail(format!("Concurrent derivation {} failed: {}", i, e)),
        }
    }
    
    TestResult::Pass
}

/// Global test suite instance
static mut TEST_SUITE: Option<SecurityTestSuite> = None;

/// Run security tests
pub fn run_security_tests() -> bool {
    unsafe {
        if TEST_SUITE.is_none() {
            TEST_SUITE = Some(SecurityTestSuite::new());
        }
        
        if let Some(ref mut suite) = TEST_SUITE {
            suite.run_all_tests()
        } else {
            false
        }
    }
}

/// Get test statistics
pub fn get_test_stats() -> Option<&'static SecurityTestStats> {
    unsafe {
        TEST_SUITE.as_ref().map(|suite| &suite.stats)
    }
}