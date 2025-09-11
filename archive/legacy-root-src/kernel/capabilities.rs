//! Capability-Based Security System
//!
//! This module implements a comprehensive capability-based security system
//! for the SIS kernel, providing fine-grained access control without
//! ambient authority.
//!
//! Geometric Principle: Capabilities form a directed acyclic graph (DAG) where
//! each edge represents a privilege derivation, ensuring no privilege escalation
//! cycles while maintaining mathematical provability of security properties.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use alloc::{collections::BTreeMap, vec::Vec, boxed::Box};
use crate::kernel::sync::{SpinLock, RwLock};

/// Capability object types for the SIS kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum CapabilityType {
    // Memory capabilities
    Memory = 0x0001,              // Raw memory access
    Page = 0x0002,                // Page-level memory
    Frame = 0x0003,               // Physical frame
    
    // Process capabilities  
    Process = 0x0100,             // Process management
    Thread = 0x0101,              // Thread control
    Signal = 0x0102,              // Signal sending
    
    // I/O capabilities
    Port = 0x0200,                // I/O port access
    Interrupt = 0x0201,           // Interrupt handling
    DMA = 0x0202,                 // DMA operations
    
    // AI-specific capabilities
    AIModel = 0x0300,             // AI model access
    AIInference = 0x0301,         // AI inference operations
    NeuralEngine = 0x0302,        // Neural Engine access
    AIData = 0x0303,              // AI training data
    
    // Network capabilities
    Network = 0x0400,             // Network access
    Socket = 0x0401,              // Socket operations
    
    // File system capabilities
    File = 0x0500,                // File access
    Directory = 0x0501,           // Directory operations
    
    // System capabilities
    Time = 0x0600,                // Time/clock access
    Random = 0x0601,              // Random number generation
    Debug = 0x0602,               // Debug operations
    
    // Security capabilities
    Crypto = 0x0700,              // Cryptographic operations
    TrustZone = 0x0701,           // TrustZone access
    TPM = 0x0702,                 // TPM operations
    
    // Administrative capabilities
    Root = 0xFFFF,                // Root capability (all privileges)
}

/// Capability permissions (bitmask)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights(pub u32);

impl CapabilityRights {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const EXECUTE: u32 = 1 << 2;
    pub const DERIVE: u32 = 1 << 3;    // Can create derived capabilities
    pub const GRANT: u32 = 1 << 4;     // Can grant to other entities
    pub const REVOKE: u32 = 1 << 5;    // Can revoke derived capabilities
    pub const TRANSFER: u32 = 1 << 6;  // Can transfer ownership
    
    // AI-specific rights
    pub const AI_TRAIN: u32 = 1 << 16;
    pub const AI_INFER: u32 = 1 << 17;
    pub const AI_PROFILE: u32 = 1 << 18;
    pub const AI_SECURE: u32 = 1 << 19; // Secure AI operations
    
    pub const fn new(rights: u32) -> Self {
        Self(rights)
    }
    
    pub const fn has(&self, right: u32) -> bool {
        (self.0 & right) != 0
    }
    
    pub fn grant(&mut self, right: u32) {
        self.0 |= right;
    }
    
    pub fn revoke(&mut self, right: u32) {
        self.0 &= !right;
    }
    
    pub fn intersect(&self, other: CapabilityRights) -> CapabilityRights {
        CapabilityRights(self.0 & other.0)
    }
}

/// Unique capability identifier
pub type CapabilityId = u64;

/// Capability object
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique identifier
    pub id: CapabilityId,
    
    /// Capability type
    pub cap_type: CapabilityType,
    
    /// Rights/permissions
    pub rights: CapabilityRights,
    
    /// Physical address or resource identifier
    pub address: u64,
    
    /// Size in bytes (for memory capabilities)
    pub size: usize,
    
    /// Parent capability (for derivation tracking)
    pub parent: Option<CapabilityId>,
    
    /// Owner process/entity ID
    pub owner: u32,
    
    /// Creation timestamp (for auditing)
    pub created_at: u64,
    
    /// Reference count
    pub ref_count: AtomicU32,
    
    /// Metadata for AI capabilities
    pub ai_metadata: Option<AiCapabilityMetadata>,
}

/// AI-specific capability metadata
#[derive(Debug, Clone)]
pub struct AiCapabilityMetadata {
    /// Model hash for verification
    pub model_hash: [u8; 32],
    
    /// Security level required
    pub security_level: u8,
    
    /// Performance requirements
    pub max_latency_us: u32,
    
    /// Data classification
    pub data_classification: DataClassification,
}

/// Data classification for AI capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    Public,        // No restrictions
    Internal,      // Organization-internal
    Confidential,  // Restricted access
    Secret,        // Highest security
}

/// Capability derivation record
#[derive(Debug, Clone)]
pub struct CapabilityDerivation {
    /// Parent capability
    pub parent_id: CapabilityId,
    
    /// Child capability  
    pub child_id: CapabilityId,
    
    /// Derivation timestamp
    pub timestamp: u64,
    
    /// Derivation context
    pub context: DerivationContext,
}

/// Context for capability derivation
#[derive(Debug, Clone)]
pub enum DerivationContext {
    /// Split memory capability
    Split { offset: usize, new_size: usize },
    
    /// Restrict rights
    Restrict { removed_rights: u32 },
    
    /// AI model loading
    AiModelLoad { model_id: u32 },
    
    /// Process creation
    ProcessCreate { pid: u32 },
    
    /// Custom derivation
    Custom { description: alloc::string::String },
}

/// Capability space for a process or entity
pub struct CapabilitySpace {
    /// Entity ID (PID or similar)
    pub entity_id: u32,
    
    /// Capabilities owned by this entity
    pub capabilities: RwLock<BTreeMap<CapabilityId, Box<Capability>>>,
    
    /// Derivation history
    pub derivations: RwLock<Vec<CapabilityDerivation>>,
    
    /// Statistics
    pub stats: CapabilityStats,
}

/// Capability usage statistics
#[derive(Debug, Default)]
pub struct CapabilityStats {
    /// Total capabilities created
    pub created: AtomicU64,
    
    /// Total capabilities revoked
    pub revoked: AtomicU64,
    
    /// Failed access attempts
    pub access_denied: AtomicU64,
    
    /// AI operations performed
    pub ai_operations: AtomicU64,
}

/// Global capability manager
pub struct CapabilityManager {
    /// Next capability ID
    next_id: AtomicU64,
    
    /// Global capability registry
    registry: RwLock<BTreeMap<CapabilityId, Box<Capability>>>,
    
    /// Per-entity capability spaces
    spaces: RwLock<BTreeMap<u32, Box<CapabilitySpace>>>,
    
    /// Global statistics
    global_stats: CapabilityStats,
}

impl CapabilityManager {
    /// Create new capability manager
    pub const fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            registry: RwLock::new(BTreeMap::new()),
            spaces: RwLock::new(BTreeMap::new()),
            global_stats: CapabilityStats {
                created: AtomicU64::new(0),
                revoked: AtomicU64::new(0),
                access_denied: AtomicU64::new(0),
                ai_operations: AtomicU64::new(0),
            },
        }
    }
    
    /// Create a new capability
    pub fn create_capability(
        &self,
        cap_type: CapabilityType,
        rights: CapabilityRights,
        address: u64,
        size: usize,
        owner: u32,
    ) -> Result<CapabilityId, &'static str> {
        let cap_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        
        let capability = Box::new(Capability {
            id: cap_id,
            cap_type,
            rights,
            address,
            size,
            parent: None,
            owner,
            created_at: crate::arch::aarch64::cpu::read_timer_counter(),
            ref_count: AtomicU32::new(1),
            ai_metadata: None,
        });
        
        // Add to global registry
        {
            let mut registry = self.registry.write();
            registry.insert(cap_id, capability.clone());
        }
        
        // Add to owner's capability space
        {
            let mut spaces = self.spaces.write();
            let space = spaces.entry(owner).or_insert_with(|| {
                Box::new(CapabilitySpace {
                    entity_id: owner,
                    capabilities: RwLock::new(BTreeMap::new()),
                    derivations: RwLock::new(Vec::new()),
                    stats: CapabilityStats::default(),
                })
            });
            
            space.capabilities.write().insert(cap_id, capability);
            space.stats.created.fetch_add(1, Ordering::Relaxed);
        }
        
        self.global_stats.created.fetch_add(1, Ordering::Relaxed);
        
        Ok(cap_id)
    }
    
    /// Create AI-specific capability with metadata
    pub fn create_ai_capability(
        &self,
        cap_type: CapabilityType,
        rights: CapabilityRights,
        model_hash: [u8; 32],
        security_level: u8,
        max_latency_us: u32,
        owner: u32,
    ) -> Result<CapabilityId, &'static str> {
        let cap_id = self.create_capability(cap_type, rights, 0, 0, owner)?;
        
        // Add AI metadata
        {
            let mut registry = self.registry.write();
            if let Some(capability) = registry.get_mut(&cap_id) {
                capability.ai_metadata = Some(AiCapabilityMetadata {
                    model_hash,
                    security_level,
                    max_latency_us,
                    data_classification: DataClassification::Confidential,
                });
            }
        }
        
        Ok(cap_id)
    }
    
    /// Derive a new capability from an existing one
    pub fn derive_capability(
        &self,
        parent_id: CapabilityId,
        new_rights: CapabilityRights,
        context: DerivationContext,
        new_owner: u32,
    ) -> Result<CapabilityId, &'static str> {
        // Verify parent capability exists and has DERIVE right
        let parent_cap = {
            let registry = self.registry.read();
            match registry.get(&parent_id) {
                Some(cap) => {
                    if !cap.rights.has(CapabilityRights::DERIVE) {
                        return Err("Parent capability lacks DERIVE right");
                    }
                    cap.clone()
                }
                None => return Err("Parent capability not found"),
            }
        };
        
        // Ensure new rights are subset of parent rights
        let restricted_rights = parent_cap.rights.intersect(new_rights);
        
        let child_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        
        let (new_address, new_size) = match context {
            DerivationContext::Split { offset, new_size } => {
                if offset + new_size > parent_cap.size {
                    return Err("Split exceeds parent capability bounds");
                }
                (parent_cap.address + offset as u64, new_size)
            }
            _ => (parent_cap.address, parent_cap.size),
        };
        
        let child_cap = Box::new(Capability {
            id: child_id,
            cap_type: parent_cap.cap_type,
            rights: restricted_rights,
            address: new_address,
            size: new_size,
            parent: Some(parent_id),
            owner: new_owner,
            created_at: crate::arch::aarch64::cpu::read_timer_counter(),
            ref_count: AtomicU32::new(1),
            ai_metadata: parent_cap.ai_metadata.clone(),
        });
        
        // Record derivation
        let derivation = CapabilityDerivation {
            parent_id,
            child_id,
            timestamp: crate::arch::aarch64::cpu::read_timer_counter(),
            context,
        };
        
        // Add to registry and spaces
        {
            let mut registry = self.registry.write();
            registry.insert(child_id, child_cap.clone());
        }
        
        {
            let mut spaces = self.spaces.write();
            let space = spaces.entry(new_owner).or_insert_with(|| {
                Box::new(CapabilitySpace {
                    entity_id: new_owner,
                    capabilities: RwLock::new(BTreeMap::new()),
                    derivations: RwLock::new(Vec::new()),
                    stats: CapabilityStats::default(),
                })
            });
            
            space.capabilities.write().insert(child_id, child_cap);
            space.derivations.write().push(derivation);
            space.stats.created.fetch_add(1, Ordering::Relaxed);
        }
        
        self.global_stats.created.fetch_add(1, Ordering::Relaxed);
        
        Ok(child_id)
    }
    
    /// Check if entity has capability with required rights
    pub fn check_capability(
        &self,
        entity_id: u32,
        cap_id: CapabilityId,
        required_rights: CapabilityRights,
    ) -> bool {
        let spaces = self.spaces.read();
        match spaces.get(&entity_id) {
            Some(space) => {
                let caps = space.capabilities.read();
                match caps.get(&cap_id) {
                    Some(cap) => {
                        let intersection = cap.rights.intersect(required_rights);
                        intersection.0 == required_rights.0
                    }
                    None => {
                        space.stats.access_denied.fetch_add(1, Ordering::Relaxed);
                        false
                    }
                }
            }
            None => {
                self.global_stats.access_denied.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }
    
    /// Revoke a capability and all its derivatives
    pub fn revoke_capability(&self, cap_id: CapabilityId, revoker: u32) -> Result<usize, &'static str> {
        let mut revoked_count = 0;
        
        // Find all derivatives recursively
        let mut to_revoke = vec![cap_id];
        let mut revoked_ids = Vec::new();
        
        while let Some(current_id) = to_revoke.pop() {
            // Check if revoker has permission
            if !self.check_capability(
                revoker,
                current_id,
                CapabilityRights::new(CapabilityRights::REVOKE),
            ) {
                continue;
            }
            
            revoked_ids.push(current_id);
            revoked_count += 1;
            
            // Find derivatives of this capability
            let spaces = self.spaces.read();
            for space in spaces.values() {
                let caps = space.capabilities.read();
                for cap in caps.values() {
                    if let Some(parent) = cap.parent {
                        if parent == current_id {
                            to_revoke.push(cap.id);
                        }
                    }
                }
            }
        }
        
        // Remove all revoked capabilities
        {
            let mut registry = self.registry.write();
            for id in &revoked_ids {
                registry.remove(id);
            }
        }
        
        {
            let mut spaces = self.spaces.write();
            for space in spaces.values_mut() {
                let mut caps = space.capabilities.write();
                for id in &revoked_ids {
                    if caps.remove(id).is_some() {
                        space.stats.revoked.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
        
        self.global_stats.revoked.fetch_add(revoked_count as u64, Ordering::Relaxed);
        
        Ok(revoked_count)
    }
    
    /// Get capability information
    pub fn get_capability(&self, cap_id: CapabilityId) -> Option<Capability> {
        let registry = self.registry.read();
        registry.get(&cap_id).map(|cap| (**cap).clone())
    }
    
    /// Get global statistics
    pub fn get_stats(&self) -> CapabilityStats {
        CapabilityStats {
            created: AtomicU64::new(self.global_stats.created.load(Ordering::Relaxed)),
            revoked: AtomicU64::new(self.global_stats.revoked.load(Ordering::Relaxed)),
            access_denied: AtomicU64::new(self.global_stats.access_denied.load(Ordering::Relaxed)),
            ai_operations: AtomicU64::new(self.global_stats.ai_operations.load(Ordering::Relaxed)),
        }
    }
    
    /// Record AI operation
    pub fn record_ai_operation(&self, entity_id: u32) {
        self.global_stats.ai_operations.fetch_add(1, Ordering::Relaxed);
        
        let spaces = self.spaces.read();
        if let Some(space) = spaces.get(&entity_id) {
            space.stats.ai_operations.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Global capability manager instance
static CAPABILITY_MANAGER: CapabilityManager = CapabilityManager::new();

/// Initialize capability system
pub fn init() -> Result<(), &'static str> {
    // Create root capability for kernel
    let root_rights = CapabilityRights::new(
        CapabilityRights::READ |
        CapabilityRights::WRITE |
        CapabilityRights::EXECUTE |
        CapabilityRights::DERIVE |
        CapabilityRights::GRANT |
        CapabilityRights::REVOKE |
        CapabilityRights::TRANSFER |
        CapabilityRights::AI_TRAIN |
        CapabilityRights::AI_INFER |
        CapabilityRights::AI_PROFILE |
        CapabilityRights::AI_SECURE
    );
    
    CAPABILITY_MANAGER.create_capability(
        CapabilityType::Root,
        root_rights,
        0,
        usize::MAX,
        0, // Kernel entity ID
    )?;
    
    crate::kernel::serial::write_str("[CAPS] Capability system initialized\n");
    Ok(())
}

/// Create new capability
pub fn create_capability(
    cap_type: CapabilityType,
    rights: CapabilityRights,
    address: u64,
    size: usize,
    owner: u32,
) -> Result<CapabilityId, &'static str> {
    CAPABILITY_MANAGER.create_capability(cap_type, rights, address, size, owner)
}

/// Create AI capability
pub fn create_ai_capability(
    cap_type: CapabilityType,
    rights: CapabilityRights,
    model_hash: [u8; 32],
    security_level: u8,
    max_latency_us: u32,
    owner: u32,
) -> Result<CapabilityId, &'static str> {
    CAPABILITY_MANAGER.create_ai_capability(
        cap_type, rights, model_hash, security_level, max_latency_us, owner
    )
}

/// Derive capability
pub fn derive_capability(
    parent_id: CapabilityId,
    new_rights: CapabilityRights,
    context: DerivationContext,
    new_owner: u32,
) -> Result<CapabilityId, &'static str> {
    CAPABILITY_MANAGER.derive_capability(parent_id, new_rights, context, new_owner)
}

/// Check capability access
pub fn check_capability(
    entity_id: u32,
    cap_id: CapabilityId,
    required_rights: CapabilityRights,
) -> bool {
    CAPABILITY_MANAGER.check_capability(entity_id, cap_id, required_rights)
}

/// Revoke capability
pub fn revoke_capability(cap_id: CapabilityId, revoker: u32) -> Result<usize, &'static str> {
    CAPABILITY_MANAGER.revoke_capability(cap_id, revoker)
}

/// Get capability statistics  
pub fn get_stats() -> CapabilityStats {
    CAPABILITY_MANAGER.get_stats()
}