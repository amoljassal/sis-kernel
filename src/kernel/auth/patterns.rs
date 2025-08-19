//! Behavioral pattern structures and analysis
//! 
//! Core pattern recognition and evolution tracking

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use super::BehavioralEvent;

/// Cognitive pattern map for behavioral analysis
pub struct CognitivePatternMap {
    /// Pattern frequency counters
    patterns: BTreeMap<PatternKey, PatternFrequency>,
    /// Total pattern observations
    total_observations: AtomicU64,
}

/// Pattern key for indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatternKey(pub u64);

/// Pattern frequency counter
#[derive(Debug)]
pub struct PatternFrequency {
    /// Observation count
    pub count: AtomicU32,
    /// Last seen timestamp
    pub last_seen: AtomicU64,
    /// Confidence score
    pub confidence: AtomicU32,
}

impl CognitivePatternMap {
    /// Create new empty pattern map
    pub const fn new() -> Self {
        Self {
            patterns: BTreeMap::new(),
            total_observations: AtomicU64::new(0),
        }
    }
    
    /// Record pattern observation
    pub fn observe(&mut self, key: PatternKey, timestamp: u64) {
        let entry = self.patterns.entry(key).or_insert_with(|| {
            PatternFrequency {
                count: AtomicU32::new(0),
                last_seen: AtomicU64::new(0),
                confidence: AtomicU32::new(0),
            }
        });
        
        entry.count.fetch_add(1, Ordering::Relaxed);
        entry.last_seen.store(timestamp, Ordering::Relaxed);
        self.total_observations.fetch_add(1, Ordering::Relaxed);
        
        // Update confidence based on frequency
        let count = entry.count.load(Ordering::Relaxed);
        let total = self.total_observations.load(Ordering::Relaxed) as u32;
        let confidence = (count * 100) / total.max(1);
        entry.confidence.store(confidence, Ordering::Relaxed);
    }
    
    /// Get pattern confidence score
    pub fn get_confidence(&self, key: PatternKey) -> Option<u32> {
        self.patterns.get(&key)
            .map(|p| p.confidence.load(Ordering::Relaxed))
    }
}

/// Linguistic signature for language patterns
pub struct LinguisticSignature {
    /// N-gram frequencies
    ngrams: BTreeMap<NgramHash, u32>,
    /// Vocabulary size
    vocab_size: AtomicU32,
    /// Average sentence length
    avg_sentence_length: AtomicU32,
}

/// N-gram hash for linguistic patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NgramHash(pub u32);

impl LinguisticSignature {
    /// Create new empty signature
    pub const fn new() -> Self {
        Self {
            ngrams: BTreeMap::new(),
            vocab_size: AtomicU32::new(0),
            avg_sentence_length: AtomicU32::new(0),
        }
    }
    
    /// Add n-gram observation
    pub fn add_ngram(&mut self, hash: NgramHash) {
        *self.ngrams.entry(hash).or_insert(0) += 1;
        self.vocab_size.store(self.ngrams.len() as u32, Ordering::Relaxed);
    }
    
    /// Calculate similarity score with another signature
    pub fn similarity(&self, other: &Self) -> f32 {
        let mut common = 0u32;
        let mut total = 0u32;
        
        for (hash, &count) in &self.ngrams {
            total += count;
            if let Some(&other_count) = other.ngrams.get(hash) {
                common += count.min(other_count);
            }
        }
        
        if total == 0 {
            return 0.0;
        }
        
        (common as f32) / (total as f32)
    }
}

/// Temporal evolution tracker
pub struct TemporalEvolution {
    /// Evolution rate (0.0 - 1.0)
    evolution_rate: f32,
    /// Sliding windows for different time scales
    windows: [SlidingWindowStats; 4],
    /// Baseline behavior
    baseline: BehavioralBaseline,
}

/// Sliding window statistics
pub struct SlidingWindowStats {
    /// Window duration in microseconds
    duration_us: u64,
    /// Event count in window
    event_count: AtomicU32,
    /// Average interval
    avg_interval: AtomicU64,
}

/// Behavioral baseline for comparison
pub struct BehavioralBaseline {
    /// Average keystroke interval
    avg_keystroke_interval: AtomicU64,
    /// Average mouse velocity
    avg_mouse_velocity: AtomicU32,
    /// Command frequency
    command_frequency: AtomicU32,
}

impl TemporalEvolution {
    /// Create new evolution tracker
    pub const fn new() -> Self {
        Self {
            evolution_rate: 0.1, // 10% adaptation rate
            windows: [
                SlidingWindowStats::new(60_000_000),    // 1 minute
                SlidingWindowStats::new(300_000_000),   // 5 minutes
                SlidingWindowStats::new(900_000_000),   // 15 minutes
                SlidingWindowStats::new(3600_000_000),  // 1 hour
            ],
            baseline: BehavioralBaseline::new(),
        }
    }
    
    /// Update with new event
    pub fn update(&mut self, event: &super::BehavioralEvent) {
        // Update windows
        for window in &mut self.windows {
            window.add_event();
        }
        
        // Update baseline with exponential moving average
        match event {
            super::BehavioralEvent::KeystrokeTiming { interval_us, .. } => {
                let old = self.baseline.avg_keystroke_interval.load(Ordering::Relaxed);
                let new = ((old as f32 * (1.0 - self.evolution_rate)) + 
                          (*interval_us as f32 * self.evolution_rate)) as u64;
                self.baseline.avg_keystroke_interval.store(new, Ordering::Relaxed);
            }
            super::BehavioralEvent::MouseMovement { velocity, .. } => {
                let old = self.baseline.avg_mouse_velocity.load(Ordering::Relaxed);
                let new = ((old as f32 * (1.0 - self.evolution_rate)) + 
                          (*velocity as f32 * self.evolution_rate)) as u32;
                self.baseline.avg_mouse_velocity.store(new, Ordering::Relaxed);
            }
            _ => {}
        }
    }
    
    /// Check for drift from baseline
    pub fn check_drift(&self, threshold: f32) -> bool {
        // Compare current windows against baseline
        // TODO: Implement statistical drift detection
        false
    }
    
    /// Get reference to behavioral baseline
    pub fn baseline(&self) -> &BehavioralBaseline {
        &self.baseline
    }
}

impl SlidingWindowStats {
    const fn new(duration_us: u64) -> Self {
        Self {
            duration_us,
            event_count: AtomicU32::new(0),
            avg_interval: AtomicU64::new(0),
        }
    }
    
    fn add_event(&self) {
        self.event_count.fetch_add(1, Ordering::Relaxed);
    }
}

impl BehavioralBaseline {
    const fn new() -> Self {
        Self {
            avg_keystroke_interval: AtomicU64::new(100_000), // 100ms default
            avg_mouse_velocity: AtomicU32::new(100),
            command_frequency: AtomicU32::new(10),
        }
    }
}

/// Fast similarity comparison using bit operations
pub fn fast_pattern_similarity(a: &[u8], b: &[u8]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let mut matches = 0u32;
    let mut total = 0u32;
    
    // Process in 8-byte chunks for speed
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();
    
    for (chunk_a, chunk_b) in chunks_a.zip(chunks_b) {
        let a_val = u64::from_le_bytes(chunk_a.try_into().unwrap());
        let b_val = u64::from_le_bytes(chunk_b.try_into().unwrap());
        
        // Count matching bits using XOR and population count
        let diff = a_val ^ b_val;
        matches += (64 - diff.count_ones()) as u32;
        total += 64;
    }
    
    // Process remaining bytes
    for (&byte_a, &byte_b) in remainder_a.iter().zip(remainder_b.iter()) {
        let diff = byte_a ^ byte_b;
        matches += (8 - diff.count_ones()) as u32;
        total += 8;
    }
    
    if total == 0 {
        return 0.0;
    }
    
    (matches as f32) / (total as f32)
}

/// Neural Engine optimized pattern classification
#[cfg(target_arch = "aarch64")]
pub fn classify_pattern_neural(pattern: &[f32]) -> Option<u32> {
    // TODO: Interface with Apple Neural Engine
    // For now, return placeholder classification
    if pattern.len() >= 16 {
        Some(pattern[0] as u32 % 10)
    } else {
        None
    }
}

/// Statistical anomaly detection
pub fn detect_anomaly(evolution: &TemporalEvolution, current: &BehavioralEvent) -> bool {
    let baseline = evolution.baseline();
    match current {
        BehavioralEvent::KeystrokeTiming { interval_us, .. } => {
            let baseline_interval = baseline.avg_keystroke_interval.load(Ordering::Relaxed);
            let deviation = (*interval_us as i64 - baseline_interval as i64).abs();
            let threshold = baseline_interval / 3; // 33% deviation threshold
            deviation > threshold as i64
        }
        BehavioralEvent::MouseMovement { velocity, .. } => {
            let baseline_velocity = baseline.avg_mouse_velocity.load(Ordering::Relaxed);
            let deviation = (*velocity as i32 - baseline_velocity as i32).abs();
            let threshold = baseline_velocity / 4; // 25% deviation threshold
            deviation > threshold as i32
        }
        _ => false,
    }
}

/// Initialize pattern storage
pub fn init_storage() -> Result<(), &'static str> {
    // Pre-allocate pattern storage structures
    // TODO: Initialize with kernel memory allocator
    Ok(())
}