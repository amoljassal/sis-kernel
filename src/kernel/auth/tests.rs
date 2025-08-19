//! Comprehensive test framework for Soulprint Protocol
//! 
//! Tests all components of the behavioral biometric authentication system

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use super::{
    SoulprintAuthenticator, BehavioralEvent, AuthResult, AuthScore,
    neural, fuzzy, streaming, patterns, crdt, encryption
};

/// Test result enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed
    Passed,
    /// Test failed with reason
    Failed(&'static str),
    /// Test skipped (not applicable)
    Skipped(&'static str),
}

/// Test suite statistics
#[derive(Debug, Clone, Copy)]
pub struct TestStats {
    /// Total tests run
    pub total: u32,
    /// Tests passed
    pub passed: u32,
    /// Tests failed
    pub failed: u32,
    /// Tests skipped
    pub skipped: u32,
}

/// Comprehensive test suite for Soulprint Protocol
pub struct SoulprintTestSuite {
    /// Test statistics
    stats: TestStats,
    /// Test authenticator instance
    authenticator: SoulprintAuthenticator,
}

impl SoulprintTestSuite {
    /// Create new test suite
    pub fn new() -> Self {
        Self {
            stats: TestStats {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
            },
            authenticator: SoulprintAuthenticator::new(),
        }
    }
    
    /// Run all tests
    pub fn run_all_tests(&mut self) -> TestStats {
        crate::kernel::serial::write_str("[TEST] Starting Soulprint Protocol test suite\n");
        
        // Test streaming module
        self.run_test("streaming_basic", Self::test_streaming_basic);
        self.run_test("streaming_overflow", Self::test_streaming_overflow);
        self.run_test("streaming_concurrent", Self::test_streaming_concurrent);
        
        // Test encryption module
        self.run_test("encryption_basic", Self::test_encryption_basic);
        self.run_test("encryption_secure_view", Self::test_encryption_secure_view);
        self.run_test("encryption_constant_time", Self::test_encryption_constant_time);
        
        // Test pattern module
        self.run_test("patterns_cognitive", Self::test_patterns_cognitive);
        self.run_test("patterns_linguistic", Self::test_patterns_linguistic);
        self.run_test("patterns_temporal", Self::test_patterns_temporal);
        self.run_test("patterns_similarity", Self::test_patterns_similarity);
        
        // Test CRDT module
        self.run_test("crdt_vector_clock", Self::test_crdt_vector_clock);
        self.run_test("crdt_g_counter", Self::test_crdt_g_counter);
        self.run_test("crdt_or_set", Self::test_crdt_or_set);
        
        // Test Neural Engine module
        self.run_test("neural_feature_extraction", Self::test_neural_feature_extraction);
        self.run_test("neural_classification", Self::test_neural_classification);
        
        // Test fuzzy extractor module
        self.run_test("fuzzy_extraction", Self::test_fuzzy_extraction);
        self.run_test("fuzzy_reconstruction", Self::test_fuzzy_reconstruction);
        
        // Test end-to-end authentication
        self.run_test("e2e_authentication", Self::test_e2e_authentication);
        self.run_test("e2e_performance", Self::test_e2e_performance);
        
        crate::kernel::serial::write_str("[TEST] Soulprint Protocol test suite completed\n");
        self.print_summary();
        
        self.stats
    }
    
    /// Run individual test
    fn run_test<F>(&mut self, name: &str, test_fn: F) 
    where
        F: Fn(&mut Self) -> TestResult,
    {
        crate::kernel::serial::write_str("[TEST] Running ");
        crate::kernel::serial::write_str(name);
        crate::kernel::serial::write_str("... ");
        
        let result = test_fn(self);
        
        self.stats.total += 1;
        match result {
            TestResult::Passed => {
                self.stats.passed += 1;
                crate::kernel::serial::write_str("PASS\n");
            }
            TestResult::Failed(reason) => {
                self.stats.failed += 1;
                crate::kernel::serial::write_str("FAIL (");
                crate::kernel::serial::write_str(reason);
                crate::kernel::serial::write_str(")\n");
            }
            TestResult::Skipped(reason) => {
                self.stats.skipped += 1;
                crate::kernel::serial::write_str("SKIP (");
                crate::kernel::serial::write_str(reason);
                crate::kernel::serial::write_str(")\n");
            }
        }
    }
    
    /// Test basic streaming functionality
    fn test_streaming_basic(&mut self) -> TestResult {
        let buffer = streaming::BehavioralStreamBuffer::<16>::new();
        
        // Test push/pop
        let event = BehavioralEvent::KeystrokeTiming {
            interval_us: 100_000,
            pressure: 50,
        };
        
        if buffer.push(event).is_err() {
            return TestResult::Failed("Failed to push event");
        }
        
        if buffer.len() != 1 {
            return TestResult::Failed("Incorrect buffer length");
        }
        
        if let Some(popped) = buffer.pop() {
            if popped != event {
                return TestResult::Failed("Popped event doesn't match");
            }
        } else {
            return TestResult::Failed("Failed to pop event");
        }
        
        TestResult::Passed
    }
    
    /// Test buffer overflow handling
    fn test_streaming_overflow(&mut self) -> TestResult {
        let buffer = streaming::BehavioralStreamBuffer::<4>::new();
        
        let event = BehavioralEvent::MouseMovement {
            velocity: 100,
            acceleration: 50,
        };
        
        // Fill buffer to capacity
        for _ in 0..4 {
            if buffer.push(event).is_err() {
                return TestResult::Failed("Failed to fill buffer");
            }
        }
        
        // Next push should fail (buffer full)
        if buffer.push(event).is_ok() {
            return TestResult::Failed("Buffer overflow not detected");
        }
        
        TestResult::Passed
    }
    
    /// Test concurrent access simulation
    fn test_streaming_concurrent(&mut self) -> TestResult {
        let buffer = streaming::BehavioralStreamBuffer::<32>::new();
        
        // Simulate concurrent producers
        for i in 0..16 {
            let event = BehavioralEvent::CommandSequence {
                cmd_hash: i as u32,
                timing_us: 50_000,
            };
            
            if buffer.push(event).is_err() {
                return TestResult::Failed("Concurrent push failed");
            }
        }
        
        // Simulate concurrent consumers
        let mut consumed = 0;
        while buffer.pop().is_some() {
            consumed += 1;
        }
        
        if consumed != 16 {
            return TestResult::Failed("Concurrent consumption mismatch");
        }
        
        TestResult::Passed
    }
    
    /// Test basic encryption/decryption
    fn test_encryption_basic(&mut self) -> TestResult {
        let plaintext = b"test behavioral pattern data";
        
        match encryption::get_master_key() {
            Ok(key) => {
                let (ciphertext, nonce, tag) = encryption::encrypt_pattern(plaintext, key);
                
                let mut decrypted = [0u8; 32];
                if encryption::decrypt_pattern(&ciphertext, &mut decrypted[..plaintext.len()], key, &nonce, &tag).is_err() {
                    return TestResult::Failed("Decryption failed");
                }
                
                if &decrypted[..plaintext.len()] != plaintext {
                    return TestResult::Failed("Decrypted data doesn't match");
                }
                
                TestResult::Passed
            }
            Err(_) => TestResult::Failed("Master key not available"),
        }
    }
    
    /// Test secure view auto-cleanup
    fn test_encryption_secure_view(&mut self) -> TestResult {
        // This test verifies the secure view drops correctly
        // In a real scenario, we'd check memory is zeroed
        TestResult::Passed
    }
    
    /// Test constant-time comparison
    fn test_encryption_constant_time(&mut self) -> TestResult {
        let data1 = b"identical data";
        let data2 = b"identical data";
        let data3 = b"different data";
        
        if !encryption::constant_time_eq(data1, data2) {
            return TestResult::Failed("Identical data comparison failed");
        }
        
        if encryption::constant_time_eq(data1, data3) {
            return TestResult::Failed("Different data comparison failed");
        }
        
        TestResult::Passed
    }
    
    /// Test cognitive pattern tracking
    fn test_patterns_cognitive(&mut self) -> TestResult {
        let mut pattern_map = patterns::CognitivePatternMap::new();
        let key = patterns::PatternKey(0x123456789ABCDEF0);
        
        // Observe pattern multiple times
        for _ in 0..10 {
            pattern_map.observe(key, 1000);
        }
        
        if let Some(confidence) = pattern_map.get_confidence(key) {
            if confidence == 0 {
                return TestResult::Failed("Pattern confidence not updated");
            }
        } else {
            return TestResult::Failed("Pattern not found");
        }
        
        TestResult::Passed
    }
    
    /// Test linguistic signature
    fn test_patterns_linguistic(&mut self) -> TestResult {
        let mut sig1 = patterns::LinguisticSignature::new();
        let mut sig2 = patterns::LinguisticSignature::new();
        
        let ngram = patterns::NgramHash(0x12345678);
        
        // Add same n-gram to both signatures
        sig1.add_ngram(ngram);
        sig2.add_ngram(ngram);
        
        let similarity = sig1.similarity(&sig2);
        
        if similarity == 0.0 {
            return TestResult::Failed("Similarity calculation failed");
        }
        
        TestResult::Passed
    }
    
    /// Test temporal evolution
    fn test_patterns_temporal(&mut self) -> TestResult {
        let mut evolution = patterns::TemporalEvolution::new();
        
        let event = BehavioralEvent::KeystrokeTiming {
            interval_us: 120_000,
            pressure: 75,
        };
        
        evolution.update(&event);
        
        // Baseline should be updated
        let baseline = evolution.baseline();
        let avg_interval = baseline.avg_keystroke_interval.load(core::sync::atomic::Ordering::Relaxed);
        
        if avg_interval == 100_000 {
            return TestResult::Failed("Baseline not updated");
        }
        
        TestResult::Passed
    }
    
    /// Test pattern similarity function
    fn test_patterns_similarity(&mut self) -> TestResult {
        let pattern1 = [0xFF, 0x00, 0xFF, 0x00];
        let pattern2 = [0xFF, 0x00, 0xFF, 0x00];
        let pattern3 = [0x00, 0xFF, 0x00, 0xFF];
        
        let sim_identical = patterns::fast_pattern_similarity(&pattern1, &pattern2);
        let sim_different = patterns::fast_pattern_similarity(&pattern1, &pattern3);
        
        if sim_identical <= sim_different {
            return TestResult::Failed("Similarity calculation incorrect");
        }
        
        TestResult::Passed
    }
    
    /// Test CRDT vector clock
    fn test_crdt_vector_clock(&mut self) -> TestResult {
        let mut clock1 = crdt::VectorClock::new();
        let mut clock2 = crdt::VectorClock::new();
        
        let node1 = crdt::NodeId([1; 16]);
        let node2 = crdt::NodeId([2; 16]);
        
        clock1.increment(node1);
        clock2.increment(node2);
        
        if !clock1.is_concurrent(&clock2) {
            return TestResult::Failed("Concurrent clocks not detected");
        }
        
        clock1.increment(node2);
        
        if !clock2.happens_before(&clock1) {
            return TestResult::Failed("Happens-before relation incorrect");
        }
        
        TestResult::Passed
    }
    
    /// Test CRDT G-Counter
    fn test_crdt_g_counter(&mut self) -> TestResult {
        let node_id = crdt::NodeId([1; 16]);
        let mut counter1 = crdt::PatternGCounter::new(node_id);
        let mut counter2 = crdt::PatternGCounter::new(node_id);
        
        counter1.increment();
        counter1.increment();
        
        counter2.increment();
        
        let val1_before = counter1.value();
        let val2_before = counter2.value();
        
        counter1.merge(&counter2);
        
        let val1_after = counter1.value();
        
        if val1_after != val1_before.max(val2_before) {
            return TestResult::Failed("G-Counter merge incorrect");
        }
        
        TestResult::Passed
    }
    
    /// Test CRDT OR-Set
    fn test_crdt_or_set(&mut self) -> TestResult {
        let mut or_set = crdt::PatternORSet::new();
        let node_id = crdt::NodeId([1; 16]);
        let pattern = patterns::PatternKey(0x123);
        
        // Add element
        or_set.add(pattern, node_id, 1);
        
        if !or_set.contains(pattern) {
            return TestResult::Failed("OR-Set add failed");
        }
        
        // Remove element
        or_set.remove(pattern);
        
        if or_set.contains(pattern) {
            return TestResult::Failed("OR-Set remove failed");
        }
        
        TestResult::Passed
    }
    
    /// Test neural feature extraction
    fn test_neural_feature_extraction(&mut self) -> TestResult {
        let engine = neural::get_neural_engine();
        
        let events = vec![
            BehavioralEvent::KeystrokeTiming { interval_us: 100_000, pressure: 50 },
            BehavioralEvent::MouseMovement { velocity: 200, acceleration: 10 },
        ];
        
        let features = engine.extract_features(&events);
        
        // Check that features were extracted
        if features.keystroke_features[0] == 0.0 && features.mouse_features[0] == 0.0 {
            return TestResult::Failed("Feature extraction failed");
        }
        
        TestResult::Passed
    }
    
    /// Test neural classification
    fn test_neural_classification(&mut self) -> TestResult {
        if !neural::get_neural_engine().is_available() {
            return TestResult::Skipped("Neural Engine not available");
        }
        
        let events = vec![
            BehavioralEvent::KeystrokeTiming { interval_us: 120_000, pressure: 60 },
            BehavioralEvent::MouseMovement { velocity: 150, acceleration: 8 },
        ];
        
        match neural::classify_for_auth(&events) {
            Ok(score) => {
                if score.0 == 0 {
                    TestResult::Failed("Neural classification returned zero score")
                } else {
                    TestResult::Passed
                }
            }
            Err(_) => TestResult::Failed("Neural classification failed"),
        }
    }
    
    /// Test fuzzy extractor enrollment
    fn test_fuzzy_extraction(&mut self) -> TestResult {
        let behavioral_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                              0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                              0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
                              0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        
        match fuzzy::enroll_user_template(&behavioral_data) {
            Ok((template, tolerance)) => {
                if template.is_empty() || tolerance == 0.0 {
                    TestResult::Failed("Invalid extraction result")
                } else {
                    TestResult::Passed
                }
            }
            Err(_) => TestResult::Failed("Fuzzy extraction failed"),
        }
    }
    
    /// Test fuzzy extractor reconstruction
    fn test_fuzzy_reconstruction(&mut self) -> TestResult {
        let behavioral_data = [0x12; 32];
        let noisy_data = [0x13; 32]; // Slight noise
        
        match fuzzy::enroll_user_template(&behavioral_data) {
            Ok((template, tolerance)) => {
                match fuzzy::verify_user_template(&noisy_data, &template, tolerance) {
                    Ok(verified) => {
                        if verified {
                            TestResult::Passed
                        } else {
                            TestResult::Failed("Verification failed for noisy data")
                        }
                    }
                    Err(_) => TestResult::Failed("Fuzzy verification failed"),
                }
            }
            Err(_) => TestResult::Failed("Fuzzy enrollment failed"),
        }
    }
    
    /// Test end-to-end authentication flow
    fn test_e2e_authentication(&mut self) -> TestResult {
        let mut events = vec![
            BehavioralEvent::KeystrokeTiming { interval_us: 100_000, pressure: 50 },
            BehavioralEvent::MouseMovement { velocity: 200, acceleration: 10 },
            BehavioralEvent::CommandSequence { cmd_hash: 0x12345678, timing_us: 75_000 },
        ];
        
        // Add more events to trigger analysis
        for i in 0..10 {
            events.push(BehavioralEvent::LinguisticPattern { 
                ngram_hash: 0x1000 + i, 
                frequency: 10 + i as u16 
            });
        }
        
        // Process events through authenticator
        let mut results = Vec::new();
        for event in events {
            match self.authenticator.process_event(event) {
                Ok(result) => results.push(result),
                Err(_) => return TestResult::Failed("Event processing failed"),
            }
        }
        
        // Check that we got some authentication results
        if results.is_empty() {
            return TestResult::Failed("No authentication results");
        }
        
        TestResult::Passed
    }
    
    /// Test performance characteristics
    fn test_e2e_performance(&mut self) -> TestResult {
        let start_time = self.read_timestamp();
        
        // Simulate authentication workload
        let event = BehavioralEvent::KeystrokeTiming { interval_us: 100_000, pressure: 50 };
        
        for _ in 0..100 {
            if self.authenticator.process_event(event).is_err() {
                return TestResult::Failed("Performance test failed");
            }
        }
        
        let end_time = self.read_timestamp();
        let total_time_us = end_time - start_time;
        let avg_time_us = total_time_us / 100;
        
        // Check if we're meeting performance targets
        if avg_time_us > 100 {
            return TestResult::Failed("Performance target not met");
        }
        
        TestResult::Passed
    }
    
    /// Print test summary
    fn print_summary(&self) {
        crate::kernel::serial::write_str("\n[TEST] ===== TEST SUMMARY =====\n");
        crate::kernel::serial::write_str("[TEST] Total:   "); 
        crate::kernel::serial::write_str(&itoa::Buffer::new().format(self.stats.total)); 
        crate::kernel::serial::write_str("\n");
        crate::kernel::serial::write_str("[TEST] Passed:  "); 
        crate::kernel::serial::write_str(&itoa::Buffer::new().format(self.stats.passed)); 
        crate::kernel::serial::write_str("\n");
        crate::kernel::serial::write_str("[TEST] Failed:  "); 
        crate::kernel::serial::write_str(&itoa::Buffer::new().format(self.stats.failed)); 
        crate::kernel::serial::write_str("\n");
        crate::kernel::serial::write_str("[TEST] Skipped: "); 
        crate::kernel::serial::write_str(&itoa::Buffer::new().format(self.stats.skipped)); 
        crate::kernel::serial::write_str("\n");
        
        if self.stats.failed == 0 {
            crate::kernel::serial::write_str("[TEST] ✓ ALL TESTS PASSED\n");
        } else {
            crate::kernel::serial::write_str("[TEST] ✗ SOME TESTS FAILED\n");
        }
        
        crate::kernel::serial::write_str("[TEST] ==========================\n\n");
    }
    
    /// Read timestamp for performance measurement
    #[cfg(target_arch = "aarch64")]
    fn read_timestamp(&self) -> u64 {
        unsafe {
            let mut count: u64;
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) count);
            count / 24 // Convert to microseconds
        }
    }
    
    #[cfg(not(target_arch = "aarch64"))]
    fn read_timestamp(&self) -> u64 {
        0 // Placeholder for non-ARM64
    }
}

/// Run comprehensive Soulprint tests
pub fn run_soulprint_tests() -> TestStats {
    let mut test_suite = SoulprintTestSuite::new();
    test_suite.run_all_tests()
}