//! CRDT structures for distributed behavioral pattern synchronization
//! 
//! Implements conflict-free replicated data types for seamless multi-device
//! behavioral pattern sharing and consensus.

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use super::patterns::{PatternKey, NgramHash};

/// Vector clock for causality tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    /// Node ID -> Clock value mapping
    pub clocks: BTreeMap<NodeId, u64>,
}

/// Node identifier in distributed system
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub [u8; 16]); // 128-bit UUID

/// CRDT G-Counter for pattern frequency counting
#[derive(Debug)]
pub struct PatternGCounter {
    /// Per-replica counters
    counters: BTreeMap<NodeId, u64>,
    /// Local node ID
    node_id: NodeId,
}

/// CRDT PN-Counter for signed pattern evolution
#[derive(Debug)]
pub struct PatternPNCounter {
    /// Positive increments
    positive: PatternGCounter,
    /// Negative decrements
    negative: PatternGCounter,
}

/// CRDT OR-Set for pattern presence tracking
#[derive(Debug)]
pub struct PatternORSet {
    /// Elements with their unique tags
    elements: BTreeMap<PatternKey, BTreeMap<NodeId, u64>>,
    /// Removed elements
    removed: BTreeMap<PatternKey, BTreeMap<NodeId, u64>>,
}

/// CRDT LWW-Register for last-writer-wins semantics
#[derive(Debug)]
pub struct PatternLWWRegister<T> {
    /// Current value
    value: Option<T>,
    /// Timestamp of last write
    timestamp: u64,
    /// Writing node
    node_id: NodeId,
}

/// Distributed behavioral signature using CRDTs
pub struct DistributedBehavioralSignature {
    /// Pattern frequencies (G-Counter)
    pattern_frequencies: BTreeMap<PatternKey, PatternGCounter>,
    /// N-gram frequencies (G-Counter)
    ngram_frequencies: BTreeMap<NgramHash, PatternGCounter>,
    /// Pattern evolution scores (PN-Counter)
    evolution_scores: BTreeMap<PatternKey, PatternPNCounter>,
    /// Active patterns (OR-Set)
    active_patterns: PatternORSet,
    /// Signature metadata (LWW-Register)
    metadata: PatternLWWRegister<SignatureMetadata>,
    /// Local node information
    local_node: NodeId,
    /// Logical clock
    clock: AtomicU64,
}

/// Signature metadata
#[derive(Debug, Clone)]
pub struct SignatureMetadata {
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Version number
    pub version: u64,
    /// Quality score (0-100)
    pub quality: u8,
}

impl VectorClock {
    /// Create new empty vector clock
    pub fn new() -> Self {
        Self {
            clocks: BTreeMap::new(),
        }
    }
    
    /// Increment clock for given node
    pub fn increment(&mut self, node_id: NodeId) {
        let counter = self.clocks.entry(node_id).or_insert(0);
        *counter += 1;
    }
    
    /// Update with another vector clock (taking maximum)
    pub fn update(&mut self, other: &VectorClock) {
        for (&node_id, &clock) in &other.clocks {
            let entry = self.clocks.entry(node_id).or_insert(0);
            *entry = (*entry).max(clock);
        }
    }
    
    /// Check if this clock happens-before another
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        
        // Check all nodes in both clocks
        let all_nodes: alloc::collections::BTreeSet<_> = self.clocks.keys()
            .chain(other.clocks.keys())
            .collect();
            
        for &node_id in all_nodes {
            let self_clock = self.clocks.get(&node_id).unwrap_or(&0);
            let other_clock = other.clocks.get(&node_id).unwrap_or(&0);
            
            if self_clock > other_clock {
                return false; // Not happens-before
            } else if self_clock < other_clock {
                strictly_less = true;
            }
        }
        
        strictly_less
    }
    
    /// Check if clocks are concurrent (neither happens-before the other)
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl PatternGCounter {
    /// Create new G-Counter
    pub fn new(node_id: NodeId) -> Self {
        Self {
            counters: BTreeMap::new(),
            node_id,
        }
    }
    
    /// Increment local counter
    pub fn increment(&mut self) {
        let counter = self.counters.entry(self.node_id).or_insert(0);
        *counter += 1;
    }
    
    /// Merge with another G-Counter
    pub fn merge(&mut self, other: &PatternGCounter) {
        for (&node_id, &count) in &other.counters {
            let entry = self.counters.entry(node_id).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
    
    /// Get current total value
    pub fn value(&self) -> u64 {
        self.counters.values().sum()
    }
}

impl PatternPNCounter {
    /// Create new PN-Counter
    pub fn new(node_id: NodeId) -> Self {
        Self {
            positive: PatternGCounter::new(node_id),
            negative: PatternGCounter::new(node_id),
        }
    }
    
    /// Increment counter
    pub fn increment(&mut self) {
        self.positive.increment();
    }
    
    /// Decrement counter
    pub fn decrement(&mut self) {
        self.negative.increment();
    }
    
    /// Merge with another PN-Counter
    pub fn merge(&mut self, other: &PatternPNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
    
    /// Get current signed value
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }
}

impl PatternORSet {
    /// Create new OR-Set
    pub fn new() -> Self {
        Self {
            elements: BTreeMap::new(),
            removed: BTreeMap::new(),
        }
    }
    
    /// Add element with unique tag
    pub fn add(&mut self, pattern: PatternKey, node_id: NodeId, tag: u64) {
        self.elements.entry(pattern)
            .or_insert_with(BTreeMap::new)
            .insert(node_id, tag);
    }
    
    /// Remove element (mark all current tags as removed)
    pub fn remove(&mut self, pattern: PatternKey) {
        if let Some(tags) = self.elements.get(&pattern) {
            let removed_tags = self.removed.entry(pattern)
                .or_insert_with(BTreeMap::new);
            for (&node_id, &tag) in tags {
                removed_tags.insert(node_id, tag);
            }
        }
    }
    
    /// Check if element is present
    pub fn contains(&self, pattern: PatternKey) -> bool {
        if let Some(element_tags) = self.elements.get(&pattern) {
            if let Some(removed_tags) = self.removed.get(&pattern) {
                // Element present if any tag in elements is not in removed
                for (&node_id, &element_tag) in element_tags {
                    if let Some(&removed_tag) = removed_tags.get(&node_id) {
                        if element_tag > removed_tag {
                            return true; // Newer add after remove
                        }
                    } else {
                        return true; // Add with no corresponding remove
                    }
                }
                false
            } else {
                !element_tags.is_empty() // Has elements, no removes
            }
        } else {
            false // No elements
        }
    }
    
    /// Merge with another OR-Set
    pub fn merge(&mut self, other: &PatternORSet) {
        // Merge elements (take maximum tag for each node)
        for (&pattern, other_tags) in &other.elements {
            let element_tags = self.elements.entry(pattern)
                .or_insert_with(BTreeMap::new);
            for (&node_id, &tag) in other_tags {
                let entry = element_tags.entry(node_id).or_insert(0);
                *entry = (*entry).max(tag);
            }
        }
        
        // Merge removed (take maximum tag for each node)
        for (&pattern, other_removed) in &other.removed {
            let removed_tags = self.removed.entry(pattern)
                .or_insert_with(BTreeMap::new);
            for (&node_id, &tag) in other_removed {
                let entry = removed_tags.entry(node_id).or_insert(0);
                *entry = (*entry).max(tag);
            }
        }
    }
    
    /// Get all present elements
    pub fn elements(&self) -> impl Iterator<Item = PatternKey> + '_ {
        self.elements.keys()
            .filter(move |&&pattern| self.contains(pattern))
            .copied()
    }
}

impl<T: Clone> PatternLWWRegister<T> {
    /// Create new LWW-Register
    pub fn new(node_id: NodeId) -> Self {
        Self {
            value: None,
            timestamp: 0,
            node_id,
        }
    }
    
    /// Set value with current timestamp
    pub fn set(&mut self, value: T, timestamp: u64) {
        if timestamp > self.timestamp || 
           (timestamp == self.timestamp && self.node_id > self.node_id) {
            self.value = Some(value);
            self.timestamp = timestamp;
        }
    }
    
    /// Get current value
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
    
    /// Merge with another LWW-Register
    pub fn merge(&mut self, other: &PatternLWWRegister<T>) {
        if other.timestamp > self.timestamp ||
           (other.timestamp == self.timestamp && other.node_id > self.node_id) {
            if let Some(ref value) = other.value {
                self.value = Some(value.clone());
                self.timestamp = other.timestamp;
                self.node_id = other.node_id;
            }
        }
    }
}

impl DistributedBehavioralSignature {
    /// Create new distributed signature
    pub fn new(node_id: NodeId) -> Self {
        Self {
            pattern_frequencies: BTreeMap::new(),
            ngram_frequencies: BTreeMap::new(),
            evolution_scores: BTreeMap::new(),
            active_patterns: PatternORSet::new(),
            metadata: PatternLWWRegister::new(node_id),
            local_node: node_id,
            clock: AtomicU64::new(0),
        }
    }
    
    /// Record pattern observation
    pub fn observe_pattern(&mut self, pattern: PatternKey) {
        // Increment frequency counter
        let counter = self.pattern_frequencies.entry(pattern)
            .or_insert_with(|| PatternGCounter::new(self.local_node));
        counter.increment();
        
        // Add to active patterns with unique tag
        let tag = self.clock.fetch_add(1, Ordering::Relaxed) + 1;
        self.active_patterns.add(pattern, self.local_node, tag);
    }
    
    /// Record n-gram observation
    pub fn observe_ngram(&mut self, ngram: NgramHash) {
        let counter = self.ngram_frequencies.entry(ngram)
            .or_insert_with(|| PatternGCounter::new(self.local_node));
        counter.increment();
    }
    
    /// Evolve pattern (positive evolution)
    pub fn evolve_pattern(&mut self, pattern: PatternKey) {
        let counter = self.evolution_scores.entry(pattern)
            .or_insert_with(|| PatternPNCounter::new(self.local_node));
        counter.increment();
    }
    
    /// Regress pattern (negative evolution)
    pub fn regress_pattern(&mut self, pattern: PatternKey) {
        let counter = self.evolution_scores.entry(pattern)
            .or_insert_with(|| PatternPNCounter::new(self.local_node));
        counter.decrement();
    }
    
    /// Merge with another distributed signature
    pub fn merge(&mut self, other: &DistributedBehavioralSignature) {
        // Merge pattern frequencies
        for (&pattern, other_counter) in &other.pattern_frequencies {
            let counter = self.pattern_frequencies.entry(pattern)
                .or_insert_with(|| PatternGCounter::new(self.local_node));
            counter.merge(other_counter);
        }
        
        // Merge n-gram frequencies
        for (&ngram, other_counter) in &other.ngram_frequencies {
            let counter = self.ngram_frequencies.entry(ngram)
                .or_insert_with(|| PatternGCounter::new(self.local_node));
            counter.merge(other_counter);
        }
        
        // Merge evolution scores
        for (&pattern, other_counter) in &other.evolution_scores {
            let counter = self.evolution_scores.entry(pattern)
                .or_insert_with(|| PatternPNCounter::new(self.local_node));
            counter.merge(other_counter);
        }
        
        // Merge active patterns
        self.active_patterns.merge(&other.active_patterns);
        
        // Merge metadata
        self.metadata.merge(&other.metadata);
    }
    
    /// Get pattern frequency
    pub fn get_pattern_frequency(&self, pattern: PatternKey) -> u64 {
        self.pattern_frequencies.get(&pattern)
            .map(|c| c.value())
            .unwrap_or(0)
    }
    
    /// Check if pattern is active
    pub fn is_pattern_active(&self, pattern: PatternKey) -> bool {
        self.active_patterns.contains(pattern)
    }
    
    /// Get evolution score for pattern
    pub fn get_evolution_score(&self, pattern: PatternKey) -> i64 {
        self.evolution_scores.get(&pattern)
            .map(|c| c.value())
            .unwrap_or(0)
    }
    
    /// Update metadata
    pub fn update_metadata(&mut self, metadata: SignatureMetadata) {
        let timestamp = self.clock.fetch_add(1, Ordering::Relaxed) + 1;
        self.metadata.set(metadata, timestamp);
    }
}

/// Generate node ID from system information
pub fn generate_node_id() -> NodeId {
    // TODO: Use hardware identifiers (MAC address, CPU serial, etc.)
    // For now, use placeholder
    NodeId([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])
}

/// Initialize CRDT storage
pub fn init_crdt_storage() -> Result<(), &'static str> {
    // Pre-allocate CRDT data structures
    // TODO: Initialize with kernel memory allocator
    Ok(())
}