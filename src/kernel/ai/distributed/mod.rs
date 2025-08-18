//! Distributed AI coordination based on Gemini recommendations
//!
//! Implements two-tiered consensus architecture:
//! - Control plane: Raft consensus for metadata (cluster state, model versions)
//! - Data plane: Gossip protocol for model weight synchronization

pub mod consensus;
pub mod gossip;
pub mod model_registry;

pub use consensus::*;
pub use gossip::*;
pub use model_registry::*;