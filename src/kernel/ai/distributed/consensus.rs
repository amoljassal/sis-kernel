//! Raft consensus for AI cluster metadata
//!
//! Implements simplified Raft protocol for managing:
//! - Node health and capabilities
//! - Model metadata and versioning  
//! - Task queue coordination
//! - Network topology information

use crate::kernel::ai::{CognitivePriority, WorkloadType};
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use alloc::{vec::Vec, collections::BTreeMap};

/// Node identifier in the cluster
pub type NodeId = u32;

/// Raft term number
pub type Term = u64;

/// Log index for Raft entries
pub type LogIndex = u64;

/// Raft node state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

/// Cluster metadata entry types
#[derive(Debug, Clone)]
pub enum MetadataEntry {
    /// Node joined cluster
    NodeJoin {
        node_id: NodeId,
        capabilities: NodeCapabilities,
    },
    /// Node left cluster
    NodeLeave {
        node_id: NodeId,
    },
    /// Model version update
    ModelVersion {
        model_id: u32,
        version: u32,
        hash: [u8; 32],
        location: ModelLocation,
    },
    /// Task assignment
    TaskAssignment {
        task_id: u64,
        assigned_node: NodeId,
        priority: CognitivePriority,
        workload_type: WorkloadType,
    },
}

/// Node hardware capabilities
#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub arch: ArchType,
    pub has_npu: bool,
    pub has_gpu: bool,
    pub memory_mb: u32,
    pub cpu_cores: u32,
    pub network_bandwidth_mbps: u32,
    pub ai_tops: u32, // AI performance in TOPS
}

/// CPU architecture type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchType {
    X86_64,
    AArch64,
}

/// Model storage location
#[derive(Debug, Clone)]
pub enum ModelLocation {
    LocalFile(alloc::string::String),
    RemoteUrl(alloc::string::String),
    DistributedShards(Vec<NodeId>),
}

/// Raft log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub term: Term,
    pub index: LogIndex,
    pub entry: MetadataEntry,
    pub timestamp_us: u64,
}

/// Raft consensus state machine for AI cluster
pub struct RaftConsensus {
    /// Current node ID
    node_id: NodeId,
    /// Current Raft state
    state: RaftState,
    /// Current term
    current_term: AtomicU64,
    /// Voted for in current term
    voted_for: core::sync::atomic::AtomicU32,
    /// Log entries
    log: spin::Mutex<Vec<LogEntry>>,
    /// Commit index
    commit_index: AtomicU64,
    /// Last applied index
    last_applied: AtomicU64,
    /// Cluster metadata state machine
    cluster_state: spin::Mutex<ClusterState>,
    /// Leader ID (0 if no leader)
    leader_id: AtomicU32,
}

/// Cluster state maintained by Raft
#[derive(Debug)]
pub struct ClusterState {
    /// Active nodes in cluster
    pub nodes: BTreeMap<NodeId, NodeCapabilities>,
    /// Model registry metadata
    pub models: BTreeMap<u32, ModelMetadata>,
    /// Current task assignments
    pub task_assignments: BTreeMap<u64, NodeId>,
    /// Network topology costs
    pub network_costs: BTreeMap<(NodeId, NodeId), u32>, // microseconds
}

/// Model metadata in cluster
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub model_id: u32,
    pub version: u32,
    pub hash: [u8; 32],
    pub location: ModelLocation,
    pub optimizations: Vec<ArchOptimization>,
    pub size_bytes: u64,
}

/// Architecture-specific optimizations
#[derive(Debug, Clone)]
pub struct ArchOptimization {
    pub arch: ArchType,
    pub format: ModelFormat,
    pub performance_tops: u32,
    pub memory_usage_mb: u32,
}

/// Model format types
#[derive(Debug, Clone, Copy)]
pub enum ModelFormat {
    ONNX,
    CoreML,      // Apple Neural Engine
    TensorRT,    // NVIDIA GPU
    TensorFlowLite,  // Edge devices
}

impl RaftConsensus {
    /// Create new Raft consensus instance
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            state: RaftState::Follower,
            current_term: AtomicU64::new(0),
            voted_for: AtomicU32::new(0),
            log: spin::Mutex::new(Vec::new()),
            commit_index: AtomicU64::new(0),
            last_applied: AtomicU64::new(0),
            cluster_state: spin::Mutex::new(ClusterState {
                nodes: BTreeMap::new(),
                models: BTreeMap::new(),
                task_assignments: BTreeMap::new(),
                network_costs: BTreeMap::new(),
            }),
            leader_id: AtomicU32::new(0),
        }
    }

    /// Propose new metadata entry to cluster
    pub fn propose_entry(&self, entry: MetadataEntry) -> Result<LogIndex, &'static str> {
        if self.state != RaftState::Leader {
            return Err("Only leader can propose entries");
        }

        let mut log = self.log.lock();
        let term = self.current_term.load(Ordering::Acquire);
        let index = log.len() as u64 + 1;

        let log_entry = LogEntry {
            term,
            index,
            entry,
            timestamp_us: get_timestamp_us(),
        };

        log.push(log_entry);
        
        // In real implementation, would replicate to followers
        // For now, immediately commit locally
        self.commit_index.store(index, Ordering::Release);
        
        Ok(index)
    }

    /// Apply committed entries to state machine
    pub fn apply_entries(&self) -> Result<(), &'static str> {
        let commit_idx = self.commit_index.load(Ordering::Acquire);
        let mut last_applied = self.last_applied.load(Ordering::Acquire);

        if last_applied >= commit_idx {
            return Ok(());
        }

        let log = self.log.lock();
        let mut state = self.cluster_state.lock();

        while last_applied < commit_idx {
            last_applied += 1;
            
            if let Some(entry) = log.get((last_applied - 1) as usize) {
                self.apply_entry(&mut state, &entry.entry)?;
            }
        }

        self.last_applied.store(last_applied, Ordering::Release);
        Ok(())
    }

    /// Apply single entry to state machine
    fn apply_entry(&self, state: &mut ClusterState, entry: &MetadataEntry) -> Result<(), &'static str> {
        match entry {
            MetadataEntry::NodeJoin { node_id, capabilities } => {
                state.nodes.insert(*node_id, capabilities.clone());
                crate::kernel::serial::write_str("[Raft] Node joined cluster\n");
            }
            MetadataEntry::NodeLeave { node_id } => {
                state.nodes.remove(node_id);
                crate::kernel::serial::write_str("[Raft] Node left cluster\n");
            }
            MetadataEntry::ModelVersion { model_id, version, hash, location } => {
                let metadata = ModelMetadata {
                    model_id: *model_id,
                    version: *version,
                    hash: *hash,
                    location: location.clone(),
                    optimizations: Vec::new(), // Populated by gossip protocol
                    size_bytes: 0, // Will be updated
                };
                state.models.insert(*model_id, metadata);
                crate::kernel::serial::write_str("[Raft] Model version updated\n");
            }
            MetadataEntry::TaskAssignment { task_id, assigned_node, .. } => {
                state.task_assignments.insert(*task_id, *assigned_node);
            }
        }
        Ok(())
    }

    /// Get current cluster state (read-only)
    pub fn get_cluster_state(&self) -> ClusterState {
        self.cluster_state.lock().clone()
    }

    /// Check if node is current leader
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Get current leader ID
    pub fn get_leader_id(&self) -> NodeId {
        self.leader_id.load(Ordering::Acquire)
    }

    /// Calculate cost-based scheduling score for node
    pub fn calculate_node_score(&self, node_id: NodeId, workload: &WorkloadRequirements) -> u32 {
        let state = self.cluster_state.lock();
        
        let Some(node_caps) = state.nodes.get(&node_id) else {
            return 0; // Node not found
        };

        let mut score = 100u32; // Base score

        // Architecture preference bonus
        if workload.preferred_arch == Some(node_caps.arch) {
            score += 20;
        }

        // Hardware acceleration bonus
        match workload.workload_type {
            WorkloadType::Inference if node_caps.has_npu => score += 30,
            WorkloadType::Training if node_caps.has_gpu => score += 25,
            _ => {}
        }

        // Network cost penalty
        if let Some(cost) = state.network_costs.get(&(self.node_id, node_id)) {
            score = score.saturating_sub(cost / 1000); // Convert μs to score penalty
        }

        // Resource availability (simplified)
        let memory_ratio = (workload.memory_mb * 100) / node_caps.memory_mb.max(1);
        if memory_ratio > 80 {
            score = score.saturating_sub(20); // Heavy memory penalty
        }

        score
    }
}

/// Workload requirements for scheduling
#[derive(Debug)]
pub struct WorkloadRequirements {
    pub workload_type: WorkloadType,
    pub priority: CognitivePriority,
    pub preferred_arch: Option<ArchType>,
    pub memory_mb: u32,
    pub estimated_compute_ms: u32,
    pub requires_npu: bool,
    pub requires_gpu: bool,
}

/// Get current timestamp in microseconds
fn get_timestamp_us() -> u64 {
    // Simplified implementation
    // In real kernel, would use high-resolution timer
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1000, Ordering::Relaxed) // Simulate 1ms increments
}

/// Global Raft consensus instance
static mut RAFT_CONSENSUS: Option<RaftConsensus> = None;

/// Initialize Raft consensus
pub fn init_raft_consensus(node_id: NodeId) -> Result<(), &'static str> {
    unsafe {
        if RAFT_CONSENSUS.is_some() {
            return Ok(());
        }
        
        RAFT_CONSENSUS = Some(RaftConsensus::new(node_id));
        Ok(())
    }
}

/// Get global Raft consensus instance
pub fn raft_consensus() -> Result<&'static RaftConsensus, &'static str> {
    unsafe {
        RAFT_CONSENSUS.as_ref().ok_or("Raft consensus not initialized")
    }
}