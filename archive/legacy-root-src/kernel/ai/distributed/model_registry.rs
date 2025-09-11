//! Polyglot model registry for distributed AI system
//!
//! Implements Gemini's recommendation for architecture-aware model distribution

use alloc::{vec::Vec, collections::BTreeMap};
use crate::kernel::ai::WorkloadType;

/// Model registry for multiple architecture optimizations
#[derive(Debug)]
pub struct ModelRegistry {
    models: BTreeMap<u32, ModelEntry>,
}

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub model_id: u32,
    pub name: alloc::string::String,
    pub formats: Vec<ModelFormat>,
}

#[derive(Debug, Clone)]
pub struct ModelFormat {
    pub arch: ArchType,
    pub format_type: FormatType,
    pub file_path: alloc::string::String,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchType {
    X86_64,
    AArch64,
}

#[derive(Debug, Clone, Copy)]
pub enum FormatType {
    ONNX,
    CoreML,
    TensorRT,
    TensorFlowLite,
}

#[derive(Debug, Clone, Copy)]
pub struct PerformanceMetrics {
    pub latency_us: u64,
    pub throughput_ops_per_sec: u64,
    pub memory_usage_mb: u32,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: BTreeMap::new(),
        }
    }

    pub fn register_model(&mut self, entry: ModelEntry) -> Result<(), &'static str> {
        self.models.insert(entry.model_id, entry);
        Ok(())
    }

    pub fn get_optimal_format(&self, model_id: u32, arch: ArchType, _workload: WorkloadType) -> Option<&ModelFormat> {
        let model = self.models.get(&model_id)?;
        
        // Find best format for architecture
        model.formats.iter()
            .filter(|f| f.arch == arch)
            .min_by_key(|f| f.performance_metrics.latency_us)
    }
}

static mut REGISTRY: Option<ModelRegistry> = None;

pub fn init() -> Result<(), &'static str> {
    unsafe {
        if REGISTRY.is_some() {
            return Ok(());
        }
        REGISTRY = Some(ModelRegistry::new());
        Ok(())
    }
}

pub fn registry() -> Result<&'static mut ModelRegistry, &'static str> {
    unsafe {
        REGISTRY.as_mut().ok_or("Model registry not initialized")
    }
}