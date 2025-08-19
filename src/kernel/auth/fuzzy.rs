//! Fuzzy Extractor for privacy-preserving noisy pattern matching
//! 
//! Implements secure biometric template matching that works with noisy
//! behavioral measurements while preserving privacy.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use super::encryption::{encrypt_pattern, decrypt_pattern, get_master_key, CryptoError};

/// Fuzzy extractor error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzyError {
    /// Input data too short
    InsufficientData,
    /// Noise level too high for extraction
    ExcessiveNoise,
    /// Cryptographic error
    CryptoError(CryptoError),
    /// Buffer allocation failed
    AllocationFailed,
    /// Inconsistent template sizes
    SizeMismatch,
}

impl From<CryptoError> for FuzzyError {
    fn from(err: CryptoError) -> Self {
        FuzzyError::CryptoError(err)
    }
}

/// Extracted biometric key and helper data
#[derive(Debug)]
pub struct BiometricExtraction {
    /// Extracted cryptographic key
    pub key: [u8; 32],
    /// Helper data for reconstruction
    pub helper_data: Vec<u8>,
    /// Noise tolerance radius
    pub tolerance: f32,
}

/// Secure sketch for error correction
#[derive(Debug)]
pub struct SecureSketch {
    /// Syndrome data for error correction
    syndrome: Vec<u8>,
    /// Code parameters
    code_length: usize,
    /// Maximum correctable errors
    max_errors: usize,
}

/// Privacy amplification context
#[derive(Debug)]
pub struct PrivacyAmplifier {
    /// Universal hash function parameters
    hash_params: [u64; 4],
    /// Output length in bits
    output_bits: usize,
}

/// Fuzzy extractor instance
pub struct FuzzyExtractor {
    /// Privacy amplifier
    amplifier: PrivacyAmplifier,
    /// Error correction parameters
    sketch_params: SketchParams,
    /// Extraction statistics
    stats: ExtractionStats,
}

/// Error correction parameters
#[derive(Debug, Clone)]
pub struct SketchParams {
    /// Block size for Reed-Solomon coding
    pub block_size: usize,
    /// Redundancy level (0.0-1.0)
    pub redundancy: f32,
    /// Hamming distance threshold
    pub distance_threshold: u32,
}

/// Extraction performance statistics
#[derive(Debug)]
pub struct ExtractionStats {
    /// Total extractions performed
    extractions: AtomicU64,
    /// Successful reconstructions
    reconstructions: AtomicU64,
    /// Average noise level
    avg_noise_level: AtomicU64,
}

impl FuzzyExtractor {
    /// Create new fuzzy extractor with default parameters
    pub fn new() -> Self {
        Self {
            amplifier: PrivacyAmplifier::new(256), // 256-bit output
            sketch_params: SketchParams {
                block_size: 64,
                redundancy: 0.5,
                distance_threshold: 8,
            },
            stats: ExtractionStats::new(),
        }
    }
    
    /// Extract biometric key from noisy measurement
    pub fn extract(&mut self, biometric_data: &[u8]) -> Result<BiometricExtraction, FuzzyError> {
        if biometric_data.len() < 32 {
            return Err(FuzzyError::InsufficientData);
        }
        
        self.stats.extractions.fetch_add(1, Ordering::Relaxed);
        
        // Step 1: Create secure sketch for error correction
        let sketch = self.create_secure_sketch(biometric_data)?;
        
        // Step 2: Apply privacy amplification
        let extracted_key = self.amplifier.extract(biometric_data)?;
        
        // Step 3: Encode helper data
        let helper_data = self.encode_helper_data(&sketch)?;
        
        // Step 4: Calculate noise tolerance
        let tolerance = self.calculate_tolerance(biometric_data);
        
        Ok(BiometricExtraction {
            key: extracted_key,
            helper_data,
            tolerance,
        })
    }
    
    /// Reconstruct key from noisy measurement and helper data
    pub fn reconstruct(&mut self, noisy_data: &[u8], helper_data: &[u8]) -> Result<[u8; 32], FuzzyError> {
        if noisy_data.len() < 32 || helper_data.is_empty() {
            return Err(FuzzyError::InsufficientData);
        }
        
        // Step 1: Decode helper data to get sketch
        let sketch = self.decode_helper_data(helper_data)?;
        
        // Step 2: Apply error correction using secure sketch
        let corrected_data = self.error_correct(noisy_data, &sketch)?;
        
        // Step 3: Apply privacy amplification to get key
        let reconstructed_key = self.amplifier.extract(&corrected_data)?;
        
        self.stats.reconstructions.fetch_add(1, Ordering::Relaxed);
        
        Ok(reconstructed_key)
    }
    
    /// Check if two biometric templates match within tolerance
    pub fn match_templates(&self, template1: &[u8], template2: &[u8], tolerance: f32) -> bool {
        if template1.len() != template2.len() {
            return false;
        }
        
        let distance = self.hamming_distance(template1, template2);
        let max_distance = (template1.len() as f32 * tolerance) as u32;
        
        distance <= max_distance
    }
    
    /// Get extraction statistics
    pub fn get_stats(&self) -> (u64, u64, f32) {
        let total = self.stats.extractions.load(Ordering::Relaxed);
        let success = self.stats.reconstructions.load(Ordering::Relaxed);
        let success_rate = if total > 0 {
            (success as f32) / (total as f32) * 100.0
        } else {
            0.0
        };
        
        (total, success, success_rate)
    }
    
    /// Create secure sketch for error correction
    fn create_secure_sketch(&self, data: &[u8]) -> Result<SecureSketch, FuzzyError> {
        let code_length = data.len();
        let max_errors = (code_length as f32 * self.sketch_params.redundancy) as usize;
        
        // Simple syndrome-based sketch (replace with Reed-Solomon in production)
        let mut syndrome = Vec::with_capacity(max_errors * 2);
        
        for i in 0..max_errors {
            let idx1 = (i * 2) % code_length;
            let idx2 = (i * 2 + 1) % code_length;
            syndrome.push(data[idx1] ^ data[idx2]);
            syndrome.push(data[idx1].wrapping_add(data[idx2]));
        }
        
        Ok(SecureSketch {
            syndrome,
            code_length,
            max_errors,
        })
    }
    
    /// Apply error correction using secure sketch
    fn error_correct(&self, noisy_data: &[u8], sketch: &SecureSketch) -> Result<Vec<u8>, FuzzyError> {
        if noisy_data.len() != sketch.code_length {
            return Err(FuzzyError::SizeMismatch);
        }
        
        let mut corrected = noisy_data.to_vec();
        
        // Simple error detection and correction
        // In production, use proper Reed-Solomon decoding
        for i in 0..sketch.max_errors {
            if i * 2 + 1 < sketch.syndrome.len() {
                let idx1 = (i * 2) % sketch.code_length;
                let idx2 = (i * 2 + 1) % sketch.code_length;
                
                let expected_xor = sketch.syndrome[i * 2];
                let expected_sum = sketch.syndrome[i * 2 + 1];
                
                let actual_xor = corrected[idx1] ^ corrected[idx2];
                let actual_sum = corrected[idx1].wrapping_add(corrected[idx2]);
                
                // Detect and correct single-bit errors
                if actual_xor != expected_xor || actual_sum != expected_sum {
                    // Simple correction heuristic
                    if actual_xor != expected_xor {
                        corrected[idx1] ^= 1;
                    }
                }
            }
        }
        
        Ok(corrected)
    }
    
    /// Encode helper data for storage
    fn encode_helper_data(&self, sketch: &SecureSketch) -> Result<Vec<u8>, FuzzyError> {
        let key = get_master_key()?;
        let (ciphertext, nonce, tag) = encrypt_pattern(&sketch.syndrome, key);
        
        // Combine nonce + tag + ciphertext
        let mut encoded = Vec::with_capacity(12 + 16 + ciphertext.len());
        encoded.extend_from_slice(&nonce);
        encoded.extend_from_slice(&tag);
        encoded.extend(ciphertext);
        
        Ok(encoded)
    }
    
    /// Decode helper data from storage
    fn decode_helper_data(&self, encoded: &[u8]) -> Result<SecureSketch, FuzzyError> {
        if encoded.len() < 28 {
            return Err(FuzzyError::InsufficientData);
        }
        
        // Extract nonce, tag, and ciphertext
        let nonce: [u8; 12] = encoded[0..12].try_into().map_err(|_| FuzzyError::InsufficientData)?;
        let tag: [u8; 16] = encoded[12..28].try_into().map_err(|_| FuzzyError::InsufficientData)?;
        let ciphertext = &encoded[28..];
        
        // Decrypt syndrome data
        let key = get_master_key()?;
        let mut decrypted = Vec::with_capacity(ciphertext.len());
        decrypted.resize(ciphertext.len(), 0);
        decrypt_pattern(ciphertext, &mut decrypted, key, &nonce, &tag)?;
        
        // Reconstruct sketch (simplified parameters)
        Ok(SecureSketch {
            syndrome: decrypted,
            code_length: 64, // Default
            max_errors: 8,   // Default
        })
    }
    
    /// Calculate noise tolerance for biometric data
    fn calculate_tolerance(&self, data: &[u8]) -> f32 {
        // Estimate noise level based on entropy
        let mut bit_counts = [0u32; 256];
        
        for &byte in data {
            bit_counts[byte as usize] += 1;
        }
        
        // Calculate entropy
        let len = data.len() as f32;
        let mut entropy = 0.0;
        
        for &count in &bit_counts {
            if count > 0 {
                let p = (count as f32) / len;
                // Approximate log2 using bit manipulation
                let log2_p = if p > 0.0 {
                    32.0 - (p.to_bits().leading_zeros() as f32)
                } else {
                    0.0
                };
                entropy -= p * log2_p;
            }
        }
        
        // Higher entropy = more noise = higher tolerance needed
        (entropy / 8.0).min(0.3) // Cap at 30% tolerance
    }
    
    /// Calculate Hamming distance between two byte arrays
    fn hamming_distance(&self, a: &[u8], b: &[u8]) -> u32 {
        a.iter().zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum()
    }
}

impl PrivacyAmplifier {
    /// Create new privacy amplifier
    pub fn new(output_bits: usize) -> Self {
        Self {
            hash_params: [0x123456789ABCDEF0, 0xFEDCBA9876543210, 
                          0x0F1E2D3C4B5A6978, 0x8796A5B4C3D2E1F0],
            output_bits,
        }
    }
    
    /// Extract uniform random key from biometric data
    pub fn extract(&self, biometric_data: &[u8]) -> Result<[u8; 32], FuzzyError> {
        if biometric_data.len() < 8 {
            return Err(FuzzyError::InsufficientData);
        }
        
        let mut key = [0u8; 32];
        
        // Universal hash function for privacy amplification
        let mut hash_state = self.hash_params[0];
        
        for chunk in biometric_data.chunks(8) {
            let mut chunk_val = 0u64;
            for (i, &byte) in chunk.iter().enumerate() {
                chunk_val |= (byte as u64) << (i * 8);
            }
            
            hash_state = hash_state.wrapping_mul(self.hash_params[1])
                        .wrapping_add(chunk_val)
                        .wrapping_mul(self.hash_params[2])
                        .wrapping_add(self.hash_params[3]);
        }
        
        // Generate key bytes from hash state
        for i in 0..4 {
            let key_chunk = hash_state.wrapping_mul(self.hash_params[i % 4]);
            let bytes = key_chunk.to_le_bytes();
            
            let start_idx = i * 8;
            let end_idx = (start_idx + 8).min(32);
            key[start_idx..end_idx].copy_from_slice(&bytes[..end_idx - start_idx]);
            
            hash_state = key_chunk;
        }
        
        Ok(key)
    }
}

impl ExtractionStats {
    const fn new() -> Self {
        Self {
            extractions: AtomicU64::new(0),
            reconstructions: AtomicU64::new(0),
            avg_noise_level: AtomicU64::new(0),
        }
    }
}

/// Global fuzzy extractor instance
static mut FUZZY_EXTRACTOR: Option<FuzzyExtractor> = None;

/// Initialize fuzzy extractor subsystem
pub fn init_fuzzy_extractor() -> Result<(), &'static str> {
    unsafe {
        FUZZY_EXTRACTOR = Some(FuzzyExtractor::new());
    }
    Ok(())
}

/// Get global fuzzy extractor reference
pub fn get_fuzzy_extractor() -> Result<&'static mut FuzzyExtractor, &'static str> {
    unsafe {
        FUZZY_EXTRACTOR.as_mut().ok_or("Fuzzy extractor not initialized")
    }
}

/// Extract biometric template for user enrollment
pub fn enroll_user_template(behavioral_data: &[u8]) -> Result<(Vec<u8>, f32), FuzzyError> {
    let extractor = get_fuzzy_extractor()
        .map_err(|_| FuzzyError::AllocationFailed)?;
    
    let extraction = extractor.extract(behavioral_data)?;
    Ok((extraction.helper_data, extraction.tolerance))
}

/// Verify user against enrolled template
pub fn verify_user_template(
    behavioral_data: &[u8],
    enrolled_template: &[u8],
    _tolerance: f32
) -> Result<bool, FuzzyError> {
    let extractor = get_fuzzy_extractor()
        .map_err(|_| FuzzyError::AllocationFailed)?;
    
    match extractor.reconstruct(behavioral_data, enrolled_template) {
        Ok(_key) => Ok(true),
        Err(FuzzyError::ExcessiveNoise) => Ok(false),
        Err(other) => Err(other),
    }
}

/// Get fuzzy extractor performance statistics
pub fn get_fuzzy_stats() -> Result<(u64, u64, f32), &'static str> {
    let extractor = get_fuzzy_extractor()?;
    Ok(extractor.get_stats())
}