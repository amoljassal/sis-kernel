//! Capability-Based Security System
//! Inspired by EROS (Extremely Reliable Operating System) and CHERI
//! Provides fine-grained access control and user sovereignty

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// Capability represents an unforgeable reference to a resource
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique capability identifier
    pub id: CapabilityId,
    /// The resource this capability refers to
    pub resource: ResourceDescriptor,
    /// Permissions granted by this capability
    pub permissions: PermissionSet,
    /// Parent capability (for revocation chains)
    pub parent: Option<CapabilityId>,
    /// Generation number for revocation
    pub generation: u64,
    /// Validity constraints
    pub constraints: CapabilityConstraints,
}

/// Unique identifier for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(u64);

impl CapabilityId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Resource that a capability can reference
#[derive(Debug, Clone)]
pub enum ResourceDescriptor {
    /// Memory region
    Memory {
        base: usize,
        size: usize,
        memory_type: MemoryType,
    },
    /// Thread or process
    Thread {
        tid: u64,
        address_space: u64,
    },
    /// IPC endpoint
    Endpoint {
        eid: u64,
        protocol: IPCProtocol,
    },
    /// File or directory
    File {
        inode: u64,
        path: Vec<u8>,
    },
    /// AI model or neural resource
    AIResource {
        model_id: u64,
        resource_type: AIResourceType,
    },
    /// Hardware device
    Device {
        device_id: u64,
        device_type: DeviceType,
    },
    /// Composite capability (contains other capabilities)
    Composite {
        capabilities: Vec<CapabilityId>,
    },
}

/// Memory types for capability-controlled memory
#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    Normal,
    DeviceMemory,
    UnifiedAI,  // For Apple Silicon unified memory
    GPUMemory,  // For discrete GPU memory
}

/// IPC protocols
#[derive(Debug, Clone, Copy)]
pub enum IPCProtocol {
    Synchronous,
    Asynchronous,
    SharedMemory,
    ArrowFormat,  // Apache Arrow for zero-copy
}

/// AI resource types
#[derive(Debug, Clone, Copy)]
pub enum AIResourceType {
    Model,
    Dataset,
    NeuralEngine,
    InferenceQueue,
    TrainingJob,
}

/// Device types
#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Storage,
    Network,
    GPU,
    NeuralEngine,
    Sensor,
}

/// Permission set for capabilities
#[derive(Debug, Clone, Copy)]
pub struct PermissionSet {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub derive: bool,      // Can create child capabilities
    pub grant: bool,       // Can grant to other domains
    pub revoke: bool,      // Can revoke derived capabilities
    pub amplify: bool,     // Can increase permissions (special)
}

impl PermissionSet {
    /// Create an empty permission set
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
            derive: false,
            grant: false,
            revoke: false,
            amplify: false,
        }
    }

    /// Create a read-only permission set
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
            derive: false,
            grant: false,
            revoke: false,
            amplify: false,
        }
    }

    /// Create a full permission set
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
            derive: true,
            grant: true,
            revoke: true,
            amplify: false,  // Amplify is always special
        }
    }
}

/// Constraints on capability validity
#[derive(Debug, Clone)]
pub struct CapabilityConstraints {
    /// Time-based expiration
    pub expires_at: Option<u64>,
    /// Usage count limit
    pub max_uses: Option<u64>,
    /// Current usage count
    pub use_count: u64,
    /// CPU cycles limit for computation
    pub cycle_limit: Option<u64>,
    /// Memory quota
    pub memory_quota: Option<usize>,
}

impl Default for CapabilityConstraints {
    fn default() -> Self {
        Self {
            expires_at: None,
            max_uses: None,
            use_count: 0,
            cycle_limit: None,
            memory_quota: None,
        }
    }
}

/// Capability table for a domain
pub struct CapabilityTable {
    /// Domain identifier
    domain_id: DomainId,
    /// Capabilities owned by this domain
    capabilities: BTreeMap<CapabilityId, Arc<RwLock<Capability>>>,
    /// Capability slots (C-list in EROS terms)
    slots: Vec<Option<CapabilityId>>,
    /// Revocation generation
    revocation_gen: AtomicU64,
}

impl CapabilityTable {
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            capabilities: BTreeMap::new(),
            slots: Vec::with_capacity(256),  // Initial 256 slots
            revocation_gen: AtomicU64::new(0),
        }
    }

    /// Insert a capability into the table
    pub fn insert(&mut self, cap: Capability) -> CapabilityId {
        let id = cap.id;
        self.capabilities.insert(id, Arc::new(RwLock::new(cap)));
        
        // Find first empty slot
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(id);
        } else {
            self.slots.push(Some(id));
        }
        
        id
    }

    /// Lookup a capability by ID
    pub fn lookup(&self, id: CapabilityId) -> Option<Arc<RwLock<Capability>>> {
        self.capabilities.get(&id).cloned()
    }

    /// Derive a new capability from an existing one
    pub fn derive(&mut self, parent_id: CapabilityId, new_perms: PermissionSet) 
        -> Result<CapabilityId, CapabilityError> {
        
        let parent = self.lookup(parent_id)
            .ok_or(CapabilityError::InvalidCapability)?;
        
        let parent_cap = parent.read();
        
        // Check derive permission
        if !parent_cap.permissions.derive {
            return Err(CapabilityError::PermissionDenied);
        }
        
        // New permissions must be subset of parent
        if !Self::is_subset(new_perms, parent_cap.permissions) {
            return Err(CapabilityError::PermissionEscalation);
        }
        
        // Create derived capability
        let derived = Capability {
            id: CapabilityId::new(),
            resource: parent_cap.resource.clone(),
            permissions: new_perms,
            parent: Some(parent_id),
            generation: parent_cap.generation,
            constraints: parent_cap.constraints.clone(),
        };
        
        let id = derived.id;
        drop(parent_cap);  // Release read lock
        
        Ok(self.insert(derived))
    }

    /// Check if permissions are a subset
    fn is_subset(subset: PermissionSet, superset: PermissionSet) -> bool {
        (!subset.read || superset.read) &&
        (!subset.write || superset.write) &&
        (!subset.execute || superset.execute) &&
        (!subset.derive || superset.derive) &&
        (!subset.grant || superset.grant) &&
        (!subset.revoke || superset.revoke) &&
        (!subset.amplify || superset.amplify)
    }

    /// Revoke a capability and all its descendants
    pub fn revoke(&mut self, cap_id: CapabilityId) -> Result<(), CapabilityError> {
        let cap = self.lookup(cap_id)
            .ok_or(CapabilityError::InvalidCapability)?;
        
        let mut cap_write = cap.write();
        
        // Increment revocation generation
        cap_write.generation = self.revocation_gen.fetch_add(1, Ordering::SeqCst);
        
        // Mark as revoked by clearing permissions
        cap_write.permissions = PermissionSet::none();
        
        // Recursively revoke children
        let children: Vec<CapabilityId> = self.capabilities
            .iter()
            .filter_map(|(id, c)| {
                let c = c.read();
                if c.parent == Some(cap_id) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        
        drop(cap_write);
        
        for child in children {
            self.revoke(child)?;
        }
        
        Ok(())
    }
}

/// Domain represents an isolated security context
pub struct Domain {
    pub id: DomainId,
    pub name: String,
    pub capability_table: RwLock<CapabilityTable>,
    pub parent: Option<DomainId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomainId(u64);

impl DomainId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Capability-based system calls
pub struct CapabilitySystemCalls;

impl CapabilitySystemCalls {
    /// Invoke a capability
    pub fn invoke(
        domain: &Domain,
        cap_id: CapabilityId,
        operation: Operation,
        args: &[u8],
    ) -> Result<Vec<u8>, CapabilityError> {
        let table = domain.capability_table.read();
        let cap = table.lookup(cap_id)
            .ok_or(CapabilityError::InvalidCapability)?;
        
        let capability = cap.read();
        
        // Check constraints
        Self::check_constraints(&capability)?;
        
        // Check permissions for operation
        Self::check_permissions(&capability, &operation)?;
        
        // Perform operation based on resource type
        match &capability.resource {
            ResourceDescriptor::Memory { base, size, .. } => {
                Self::invoke_memory_cap(*base, *size, operation, args)
            }
            ResourceDescriptor::AIResource { model_id, .. } => {
                Self::invoke_ai_cap(*model_id, operation, args)
            }
            _ => Err(CapabilityError::UnsupportedOperation),
        }
    }

    fn check_constraints(cap: &Capability) -> Result<(), CapabilityError> {
        // Check expiration
        if let Some(expires) = cap.constraints.expires_at {
            // Check against system time
            // if current_time() > expires {
            //     return Err(CapabilityError::Expired);
            // }
        }
        
        // Check usage limit
        if let Some(max) = cap.constraints.max_uses {
            if cap.constraints.use_count >= max {
                return Err(CapabilityError::UsageLimitExceeded);
            }
        }
        
        Ok(())
    }

    fn check_permissions(cap: &Capability, op: &Operation) -> Result<(), CapabilityError> {
        match op {
            Operation::Read => {
                if !cap.permissions.read {
                    return Err(CapabilityError::PermissionDenied);
                }
            }
            Operation::Write => {
                if !cap.permissions.write {
                    return Err(CapabilityError::PermissionDenied);
                }
            }
            Operation::Execute => {
                if !cap.permissions.execute {
                    return Err(CapabilityError::PermissionDenied);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn invoke_memory_cap(
        base: usize,
        size: usize,
        op: Operation,
        args: &[u8],
    ) -> Result<Vec<u8>, CapabilityError> {
        // Memory capability invocation
        match op {
            Operation::Read => {
                // Safe memory read
                Ok(vec![])  // Placeholder
            }
            Operation::Write => {
                // Safe memory write
                Ok(vec![])  // Placeholder
            }
            _ => Err(CapabilityError::UnsupportedOperation),
        }
    }

    fn invoke_ai_cap(
        model_id: u64,
        op: Operation,
        args: &[u8],
    ) -> Result<Vec<u8>, CapabilityError> {
        // AI resource capability invocation
        match op {
            Operation::Execute => {
                // Run inference
                Ok(vec![])  // Placeholder
            }
            _ => Err(CapabilityError::UnsupportedOperation),
        }
    }
}

/// Operations that can be performed on capabilities
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Read,
    Write,
    Execute,
    Derive,
    Grant,
    Revoke,
    Custom(u32),
}

/// Capability system errors
#[derive(Debug)]
pub enum CapabilityError {
    InvalidCapability,
    PermissionDenied,
    PermissionEscalation,
    Expired,
    UsageLimitExceeded,
    UnsupportedOperation,
    RevocationFailed,
}

/// Capability manager for the entire system
pub struct CapabilityManager {
    domains: RwLock<BTreeMap<DomainId, Arc<Domain>>>,
    root_domain: DomainId,
}

impl CapabilityManager {
    pub fn new() -> Self {
        let root_id = DomainId::new();
        let root_domain = Arc::new(Domain {
            id: root_id,
            name: String::from("root"),
            capability_table: RwLock::new(CapabilityTable::new(root_id)),
            parent: None,
        });
        
        let mut domains = BTreeMap::new();
        domains.insert(root_id, root_domain);
        
        Self {
            domains: RwLock::new(domains),
            root_domain: root_id,
        }
    }

    /// Create a new domain
    pub fn create_domain(&self, name: &str, parent: Option<DomainId>) -> DomainId {
        let id = DomainId::new();
        let domain = Arc::new(Domain {
            id,
            name: String::from(name),
            capability_table: RwLock::new(CapabilityTable::new(id)),
            parent,
        });
        
        self.domains.write().insert(id, domain);
        id
    }

    /// Grant a capability from one domain to another
    pub fn grant_capability(
        &self,
        from_domain: DomainId,
        to_domain: DomainId,
        cap_id: CapabilityId,
        new_perms: PermissionSet,
    ) -> Result<CapabilityId, CapabilityError> {
        let domains = self.domains.read();
        
        let from = domains.get(&from_domain)
            .ok_or(CapabilityError::InvalidCapability)?;
        let to = domains.get(&to_domain)
            .ok_or(CapabilityError::InvalidCapability)?;
        
        // Get capability from source domain
        let from_table = from.capability_table.read();
        let cap = from_table.lookup(cap_id)
            .ok_or(CapabilityError::InvalidCapability)?;
        
        let source_cap = cap.read();
        
        // Check grant permission
        if !source_cap.permissions.grant {
            return Err(CapabilityError::PermissionDenied);
        }
        
        // Create new capability for target domain
        let granted = Capability {
            id: CapabilityId::new(),
            resource: source_cap.resource.clone(),
            permissions: new_perms,
            parent: Some(cap_id),
            generation: source_cap.generation,
            constraints: source_cap.constraints.clone(),
        };
        
        drop(source_cap);
        drop(from_table);
        
        // Insert into target domain
        let mut to_table = to.capability_table.write();
        Ok(to_table.insert(granted))
    }
}

/// Initialize the capability system with bootstrap capabilities
pub fn initialize_capability_system() -> CapabilityManager {
    let manager = CapabilityManager::new();
    
    // Create initial capabilities for kernel resources
    // This would be called during kernel initialization
    
    manager
}