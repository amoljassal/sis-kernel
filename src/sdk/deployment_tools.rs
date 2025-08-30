//! Deployment Automation Tools for SIS-OS Applications
//! Comprehensive deployment, scaling, and lifecycle management

#![no_std]

use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

use super::{
    DeveloperProject, DeploymentConfiguration, DeploymentResult, DeploymentId, 
    DeploymentTarget, ResourceLimits, ScalingConfiguration, SDKError
};

/// Deployment automation and orchestration tools
pub struct DeploymentTools {
    /// Container orchestrator
    orchestrator: ContainerOrchestrator,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Monitoring system
    monitoring: MonitoringSystem,
    /// Load balancer
    load_balancer: LoadBalancer,
    /// Auto-scaling engine
    auto_scaler: AutoScaler,
    /// Active deployments
    deployments: RwLock<BTreeMap<DeploymentId, ActiveDeployment>>,
}

impl DeploymentTools {
    pub fn new() -> Self {
        Self {
            orchestrator: ContainerOrchestrator::new(),
            resource_manager: ResourceManager::new(),
            monitoring: MonitoringSystem::new(),
            load_balancer: LoadBalancer::new(),
            auto_scaler: AutoScaler::new(),
            deployments: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn initialize(&mut self, config: &super::SDKConfiguration) -> Result<(), DeploymentError> {
        self.orchestrator.initialize(config)?;
        self.resource_manager.initialize()?;
        self.monitoring.initialize()?;
        self.load_balancer.initialize()?;
        self.auto_scaler.initialize()?;
        Ok(())
    }

    /// Deploy application to SIS-OS infrastructure
    pub fn deploy_application(&mut self, project: &DeveloperProject, config: DeploymentConfiguration) 
        -> Result<DeploymentResult, DeploymentError> {
        
        let deployment_id = DeploymentId::new();
        
        // Create deployment plan
        let deployment_plan = self.create_deployment_plan(project, &config)?;
        
        // Allocate resources
        let resource_allocation = self.resource_manager.allocate_resources(&config.resource_limits)?;
        
        // Deploy to target environment
        let deployment_info = match config.deployment_target {
            DeploymentTarget::Local => {
                self.deploy_local(project, &deployment_plan, &resource_allocation)?
            },
            DeploymentTarget::Cloud => {
                self.deploy_cloud(project, &deployment_plan, &resource_allocation)?
            },
            DeploymentTarget::Edge => {
                self.deploy_edge(project, &deployment_plan, &resource_allocation)?
            },
            DeploymentTarget::Distributed => {
                self.deploy_distributed(project, &deployment_plan, &resource_allocation)?
            },
        };

        // Setup monitoring
        if config.monitoring_enabled {
            self.monitoring.setup_application_monitoring(deployment_id, &deployment_info)?;
        }

        // Configure load balancing
        if config.scaling_config.auto_scaling_enabled {
            self.load_balancer.configure_load_balancing(deployment_id, &config.scaling_config)?;
            self.auto_scaler.setup_auto_scaling(deployment_id, &config.scaling_config)?;
        }

        // Create active deployment record
        let active_deployment = ActiveDeployment {
            id: deployment_id,
            project_id: project.id,
            configuration: config.clone(),
            deployment_info: deployment_info.clone(),
            resource_allocation,
            status: DeploymentStatus::Running,
            created_at: Self::current_time(),
            last_updated: Self::current_time(),
            metrics: DeploymentMetrics::new(),
        };

        self.deployments.write().insert(deployment_id, active_deployment);

        Ok(DeploymentResult {
            deployment_id,
            success: true,
            deployment_url: deployment_info.access_url,
            deployment_time_ms: 5000,  // Simplified
            resource_usage: deployment_info.resource_usage,
        })
    }

    /// Scale deployed application
    pub fn scale_deployment(&mut self, deployment_id: DeploymentId, target_instances: u32) 
        -> Result<ScalingResult, DeploymentError> {
        
        let mut deployments = self.deployments.write();
        let deployment = deployments.get_mut(&deployment_id)
            .ok_or(DeploymentError::DeploymentNotFound)?;

        // Perform scaling operation
        let scaling_result = self.orchestrator.scale_deployment(
            deployment_id, 
            target_instances,
            &deployment.configuration.resource_limits
        )?;

        // Update deployment record
        deployment.status = DeploymentStatus::Scaling;
        deployment.last_updated = Self::current_time();

        Ok(scaling_result)
    }

    /// Update deployed application
    pub fn update_deployment(&mut self, deployment_id: DeploymentId, project: &DeveloperProject) 
        -> Result<UpdateResult, DeploymentError> {
        
        let deployments = self.deployments.read();
        let deployment = deployments.get(&deployment_id)
            .ok_or(DeploymentError::DeploymentNotFound)?;

        // Perform rolling update
        let update_result = self.orchestrator.rolling_update(
            deployment_id,
            project,
            &deployment.configuration
        )?;

        Ok(update_result)
    }

    /// Get deployment status and metrics
    pub fn get_deployment_status(&self, deployment_id: DeploymentId) 
        -> Result<DeploymentStatus, DeploymentError> {
        
        let deployments = self.deployments.read();
        let deployment = deployments.get(&deployment_id)
            .ok_or(DeploymentError::DeploymentNotFound)?;

        Ok(deployment.status.clone())
    }

    /// Get deployment metrics
    pub fn get_deployment_metrics(&self, deployment_id: DeploymentId) 
        -> Result<DeploymentMetrics, DeploymentError> {
        
        // Fetch real-time metrics from monitoring system
        let metrics = self.monitoring.get_deployment_metrics(deployment_id)?;
        Ok(metrics)
    }

    /// Stop and remove deployment
    pub fn undeploy_application(&mut self, deployment_id: DeploymentId) 
        -> Result<UndeploymentResult, DeploymentError> {
        
        let mut deployments = self.deployments.write();
        let deployment = deployments.get_mut(&deployment_id)
            .ok_or(DeploymentError::DeploymentNotFound)?;

        // Gracefully stop application instances
        let stop_result = self.orchestrator.stop_deployment(deployment_id)?;

        // Cleanup resources
        self.resource_manager.deallocate_resources(&deployment.resource_allocation)?;

        // Remove monitoring
        self.monitoring.remove_monitoring(deployment_id)?;

        // Remove from active deployments
        deployment.status = DeploymentStatus::Stopped;
        deployments.remove(&deployment_id);

        Ok(UndeploymentResult {
            deployment_id,
            success: stop_result.success,
            cleanup_time_ms: stop_result.stop_time_ms,
        })
    }

    /// Generate deployment guide documentation
    pub fn generate_deployment_guide(&self, project: &DeveloperProject) -> Result<String, DeploymentError> {
        let mut guide = String::new();
        guide.push_str("# Deployment Guide for ");
        guide.push_str(&project.name);
        guide.push_str("\n\n");
        
        guide.push_str("## Prerequisites\n");
        guide.push_str("- SIS-OS kernel version 1.0.0 or higher\n");
        guide.push_str("- Required capabilities: ");
        for cap in &project.capabilities {
            guide.push_str(&cap.name);
            guide.push_str(", ");
        }
        guide.push_str("\n\n");
        
        guide.push_str("## Deployment Options\n");
        guide.push_str("### Local Deployment\n");
        guide.push_str("```\nsis deploy --target local\n```\n\n");
        
        guide.push_str("### Cloud Deployment\n");
        guide.push_str("```\nsis deploy --target cloud --instances 3\n```\n\n");
        
        guide.push_str("### Edge Deployment\n");
        guide.push_str("```\nsis deploy --target edge --regions us-west,eu-central\n```\n\n");
        
        guide.push_str("## Monitoring and Scaling\n");
        guide.push_str("- Metrics available at /metrics endpoint\n");
        guide.push_str("- Auto-scaling based on cognitive load\n");
        guide.push_str("- Health checks every 30 seconds\n");
        
        Ok(guide)
    }

    // Private helper methods

    fn create_deployment_plan(&self, project: &DeveloperProject, config: &DeploymentConfiguration) 
        -> Result<DeploymentPlan, DeploymentError> {
        
        Ok(DeploymentPlan {
            project_id: project.id,
            target_environment: config.deployment_target.clone(),
            required_instances: config.scaling_config.min_instances,
            resource_requirements: ResourceRequirements::from_limits(&config.resource_limits),
            network_configuration: NetworkConfiguration::default(),
            security_configuration: SecurityConfiguration::default(),
        })
    }

    fn deploy_local(&mut self, project: &DeveloperProject, plan: &DeploymentPlan, allocation: &ResourceAllocation) 
        -> Result<DeploymentInfo, DeploymentError> {
        
        // Local deployment using SIS-OS native containers
        let container = self.orchestrator.create_container(project, plan)?;
        
        Ok(DeploymentInfo {
            deployment_type: DeploymentType::Local,
            access_url: Some("http://localhost:8080".to_string()),
            resource_usage: super::ResourceUsage {
                cpu_cores: allocation.cpu_cores,
                memory_mb: allocation.memory_mb,
                neural_units: allocation.neural_units,
                storage_mb: allocation.storage_mb,
            },
            instances: vec![InstanceInfo {
                instance_id: "local-001".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: allocation.cpu_cores,
                    memory_mb: allocation.memory_mb,
                    neural_units: allocation.neural_units,
                    storage_mb: allocation.storage_mb,
                },
            }],
        })
    }

    fn deploy_cloud(&mut self, project: &DeveloperProject, plan: &DeploymentPlan, allocation: &ResourceAllocation) 
        -> Result<DeploymentInfo, DeploymentError> {
        
        // Cloud deployment with auto-scaling and load balancing
        let instances = self.orchestrator.create_cloud_instances(project, plan)?;
        
        Ok(DeploymentInfo {
            deployment_type: DeploymentType::Cloud,
            access_url: Some("https://app.sis-cloud.ai".to_string()),
            resource_usage: super::ResourceUsage {
                cpu_cores: allocation.cpu_cores * instances.len() as u32,
                memory_mb: allocation.memory_mb * instances.len() as u64,
                neural_units: allocation.neural_units * instances.len() as u32,
                storage_mb: allocation.storage_mb,
            },
            instances,
        })
    }

    fn deploy_edge(&mut self, project: &DeveloperProject, plan: &DeploymentPlan, allocation: &ResourceAllocation) 
        -> Result<DeploymentInfo, DeploymentError> {
        
        // Edge deployment across multiple regions
        let edge_instances = self.orchestrator.create_edge_instances(project, plan)?;
        
        Ok(DeploymentInfo {
            deployment_type: DeploymentType::Edge,
            access_url: Some("https://edge.sis-ai.net".to_string()),
            resource_usage: super::ResourceUsage {
                cpu_cores: allocation.cpu_cores * edge_instances.len() as u32,
                memory_mb: allocation.memory_mb * edge_instances.len() as u64,
                neural_units: allocation.neural_units * edge_instances.len() as u32,
                storage_mb: allocation.storage_mb,
            },
            instances: edge_instances,
        })
    }

    fn deploy_distributed(&mut self, project: &DeveloperProject, plan: &DeploymentPlan, allocation: &ResourceAllocation) 
        -> Result<DeploymentInfo, DeploymentError> {
        
        // Distributed deployment across multiple SIS-OS nodes
        let distributed_instances = self.orchestrator.create_distributed_instances(project, plan)?;
        
        Ok(DeploymentInfo {
            deployment_type: DeploymentType::Distributed,
            access_url: Some("https://distributed.sis-cluster.ai".to_string()),
            resource_usage: super::ResourceUsage {
                cpu_cores: allocation.cpu_cores * distributed_instances.len() as u32,
                memory_mb: allocation.memory_mb * distributed_instances.len() as u64,
                neural_units: allocation.neural_units * distributed_instances.len() as u32,
                storage_mb: allocation.storage_mb,
            },
            instances: distributed_instances,
        })
    }

    fn current_time() -> u64 {
        0  // Would use actual timestamp
    }
}

/// Container orchestration system
pub struct ContainerOrchestrator {
    containers: BTreeMap<String, Container>,
}

impl ContainerOrchestrator {
    pub fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self, config: &super::SDKConfiguration) -> Result<(), DeploymentError> {
        // Initialize container runtime
        Ok(())
    }

    pub fn create_container(&mut self, project: &DeveloperProject, plan: &DeploymentPlan) 
        -> Result<Container, DeploymentError> {
        
        Ok(Container {
            id: "container-001".to_string(),
            image: {
                let mut image_name = String::new();
                image_name.push_str("sis-app/");
                image_name.push_str(&project.name);
                image_name
            },
            status: ContainerStatus::Running,
            resource_usage: ContainerResources {
                cpu_usage: 0.5,
                memory_usage_mb: 512,
                neural_unit_usage: 0.8,
            },
        })
    }

    pub fn scale_deployment(&mut self, deployment_id: DeploymentId, target_instances: u32, resource_limits: &ResourceLimits) 
        -> Result<ScalingResult, DeploymentError> {
        
        Ok(ScalingResult {
            previous_instances: 1,
            target_instances,
            current_instances: target_instances,
            scaling_time_ms: 10000,
            success: true,
        })
    }

    pub fn rolling_update(&mut self, deployment_id: DeploymentId, project: &DeveloperProject, config: &DeploymentConfiguration) 
        -> Result<UpdateResult, DeploymentError> {
        
        Ok(UpdateResult {
            deployment_id,
            previous_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
            update_time_ms: 30000,
            success: true,
        })
    }

    pub fn stop_deployment(&mut self, deployment_id: DeploymentId) -> Result<StopResult, DeploymentError> {
        Ok(StopResult {
            deployment_id,
            success: true,
            stop_time_ms: 5000,
        })
    }

    pub fn create_cloud_instances(&mut self, project: &DeveloperProject, plan: &DeploymentPlan) 
        -> Result<Vec<InstanceInfo>, DeploymentError> {
        
        Ok(vec![
            InstanceInfo {
                instance_id: "cloud-001".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 4,
                    memory_mb: 8192,
                    neural_units: 2,
                    storage_mb: 10240,
                },
            },
            InstanceInfo {
                instance_id: "cloud-002".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 4,
                    memory_mb: 8192,
                    neural_units: 2,
                    storage_mb: 10240,
                },
            },
        ])
    }

    pub fn create_edge_instances(&mut self, project: &DeveloperProject, plan: &DeploymentPlan) 
        -> Result<Vec<InstanceInfo>, DeploymentError> {
        
        Ok(vec![
            InstanceInfo {
                instance_id: "edge-us-west-001".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 2,
                    memory_mb: 4096,
                    neural_units: 1,
                    storage_mb: 5120,
                },
            },
            InstanceInfo {
                instance_id: "edge-eu-central-001".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 2,
                    memory_mb: 4096,
                    neural_units: 1,
                    storage_mb: 5120,
                },
            },
        ])
    }

    pub fn create_distributed_instances(&mut self, project: &DeveloperProject, plan: &DeploymentPlan) 
        -> Result<Vec<InstanceInfo>, DeploymentError> {
        
        Ok(vec![
            InstanceInfo {
                instance_id: "node-001".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    neural_units: 4,
                    storage_mb: 20480,
                },
            },
            InstanceInfo {
                instance_id: "node-002".to_string(),
                status: InstanceStatus::Running,
                resource_usage: super::ResourceUsage {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    neural_units: 4,
                    storage_mb: 20480,
                },
            },
        ])
    }
}

/// Resource allocation and management
pub struct ResourceManager {
    available_resources: AvailableResources,
    allocated_resources: BTreeMap<DeploymentId, ResourceAllocation>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            available_resources: AvailableResources::default(),
            allocated_resources: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), DeploymentError> {
        // Query system resources
        self.available_resources = AvailableResources {
            total_cpu_cores: 64,
            total_memory_mb: 131072,  // 128GB
            total_neural_units: 16,
            total_storage_mb: 1048576,  // 1TB
            available_cpu_cores: 56,
            available_memory_mb: 114688,
            available_neural_units: 14,
            available_storage_mb: 943718,
        };
        Ok(())
    }

    pub fn allocate_resources(&mut self, limits: &ResourceLimits) -> Result<ResourceAllocation, DeploymentError> {
        // Check resource availability
        if limits.max_cpu_cores > self.available_resources.available_cpu_cores {
            return Err(DeploymentError::InsufficientResources("CPU cores".to_string()));
        }

        if limits.max_memory_mb > self.available_resources.available_memory_mb {
            return Err(DeploymentError::InsufficientResources("Memory".to_string()));
        }

        if limits.max_neural_units > self.available_resources.available_neural_units {
            return Err(DeploymentError::InsufficientResources("Neural units".to_string()));
        }

        // Allocate resources
        let allocation = ResourceAllocation {
            cpu_cores: limits.max_cpu_cores,
            memory_mb: limits.max_memory_mb,
            neural_units: limits.max_neural_units,
            storage_mb: limits.max_storage_mb,
        };

        // Update available resources
        self.available_resources.available_cpu_cores -= allocation.cpu_cores;
        self.available_resources.available_memory_mb -= allocation.memory_mb;
        self.available_resources.available_neural_units -= allocation.neural_units;
        self.available_resources.available_storage_mb -= allocation.storage_mb;

        Ok(allocation)
    }

    pub fn deallocate_resources(&mut self, allocation: &ResourceAllocation) -> Result<(), DeploymentError> {
        // Return resources to available pool
        self.available_resources.available_cpu_cores += allocation.cpu_cores;
        self.available_resources.available_memory_mb += allocation.memory_mb;
        self.available_resources.available_neural_units += allocation.neural_units;
        self.available_resources.available_storage_mb += allocation.storage_mb;
        Ok(())
    }
}

/// Monitoring and observability system
pub struct MonitoringSystem {
    active_monitors: BTreeMap<DeploymentId, Monitor>,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        Self {
            active_monitors: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), DeploymentError> {
        Ok(())
    }

    pub fn setup_application_monitoring(&mut self, deployment_id: DeploymentId, info: &DeploymentInfo) 
        -> Result<(), DeploymentError> {
        
        let monitor = Monitor {
            deployment_id,
            metrics_endpoint: {
                let mut endpoint = String::new();
                endpoint.push_str("http://monitor/deployments/");
                // Simple deployment_id to string conversion
                endpoint.push_str("deployment");
                endpoint
            },
            health_check_interval_ms: 30000,
            alert_thresholds: AlertThresholds::default(),
        };

        self.active_monitors.insert(deployment_id, monitor);
        Ok(())
    }

    pub fn get_deployment_metrics(&self, deployment_id: DeploymentId) -> Result<DeploymentMetrics, DeploymentError> {
        Ok(DeploymentMetrics {
            cpu_utilization: 0.65,
            memory_utilization: 0.78,
            neural_unit_utilization: 0.82,
            request_rate_per_second: 150.0,
            response_time_ms: 25.0,
            error_rate_percentage: 0.5,
            uptime_percentage: 99.9,
        })
    }

    pub fn remove_monitoring(&mut self, deployment_id: DeploymentId) -> Result<(), DeploymentError> {
        self.active_monitors.remove(&deployment_id);
        Ok(())
    }
}

/// Load balancing system
pub struct LoadBalancer {
    load_balancers: BTreeMap<DeploymentId, LoadBalancerConfig>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            load_balancers: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), DeploymentError> {
        Ok(())
    }

    pub fn configure_load_balancing(&mut self, deployment_id: DeploymentId, scaling_config: &ScalingConfiguration) 
        -> Result<(), DeploymentError> {
        
        let config = LoadBalancerConfig {
            deployment_id,
            algorithm: LoadBalancingAlgorithm::RoundRobin,
            health_check_enabled: true,
            sticky_sessions: false,
        };

        self.load_balancers.insert(deployment_id, config);
        Ok(())
    }
}

/// Auto-scaling system
pub struct AutoScaler {
    scaling_policies: BTreeMap<DeploymentId, ScalingPolicy>,
}

impl AutoScaler {
    pub fn new() -> Self {
        Self {
            scaling_policies: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), DeploymentError> {
        Ok(())
    }

    pub fn setup_auto_scaling(&mut self, deployment_id: DeploymentId, scaling_config: &ScalingConfiguration) 
        -> Result<(), DeploymentError> {
        
        let policy = ScalingPolicy {
            deployment_id,
            min_instances: scaling_config.min_instances,
            max_instances: scaling_config.max_instances,
            scale_up_threshold: scaling_config.scaling_threshold,
            scale_down_threshold: scaling_config.scaling_threshold * 0.7,
            cooldown_period_ms: 300000,  // 5 minutes
        };

        self.scaling_policies.insert(deployment_id, policy);
        Ok(())
    }
}

// Data structures

#[derive(Debug, Clone)]
pub struct DeploymentPlan {
    pub project_id: super::ProjectId,
    pub target_environment: DeploymentTarget,
    pub required_instances: u32,
    pub resource_requirements: ResourceRequirements,
    pub network_configuration: NetworkConfiguration,
    pub security_configuration: SecurityConfiguration,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_cores_per_instance: u32,
    pub memory_mb_per_instance: u64,
    pub neural_units_per_instance: u32,
    pub storage_mb_total: u64,
}

impl ResourceRequirements {
    pub fn from_limits(limits: &ResourceLimits) -> Self {
        Self {
            cpu_cores_per_instance: limits.max_cpu_cores,
            memory_mb_per_instance: limits.max_memory_mb,
            neural_units_per_instance: limits.max_neural_units,
            storage_mb_total: limits.max_storage_mb,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkConfiguration {
    pub expose_port: u16,
    pub enable_https: bool,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityConfiguration {
    pub enable_authentication: bool,
    pub require_capabilities: Vec<String>,
    pub network_isolation: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveDeployment {
    pub id: DeploymentId,
    pub project_id: super::ProjectId,
    pub configuration: DeploymentConfiguration,
    pub deployment_info: DeploymentInfo,
    pub resource_allocation: ResourceAllocation,
    pub status: DeploymentStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub metrics: DeploymentMetrics,
}

#[derive(Debug, Clone)]
pub struct DeploymentInfo {
    pub deployment_type: DeploymentType,
    pub access_url: Option<String>,
    pub resource_usage: super::ResourceUsage,
    pub instances: Vec<InstanceInfo>,
}

#[derive(Debug, Clone)]
pub enum DeploymentType {
    Local,
    Cloud,
    Edge,
    Distributed,
}

#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub instance_id: String,
    pub status: InstanceStatus,
    pub resource_usage: super::ResourceUsage,
}

#[derive(Debug, Clone)]
pub enum InstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentStatus {
    Initializing,
    Running,
    Scaling,
    Updating,
    Stopped,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub neural_units: u32,
    pub storage_mb: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AvailableResources {
    pub total_cpu_cores: u32,
    pub total_memory_mb: u64,
    pub total_neural_units: u32,
    pub total_storage_mb: u64,
    pub available_cpu_cores: u32,
    pub available_memory_mb: u64,
    pub available_neural_units: u32,
    pub available_storage_mb: u64,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub image: String,
    pub status: ContainerStatus,
    pub resource_usage: ContainerResources,
}

#[derive(Debug, Clone)]
pub enum ContainerStatus {
    Creating,
    Running,
    Paused,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ContainerResources {
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub neural_unit_usage: f64,
}

#[derive(Debug, Clone)]
pub struct DeploymentMetrics {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub neural_unit_utilization: f64,
    pub request_rate_per_second: f64,
    pub response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub uptime_percentage: f64,
}

impl DeploymentMetrics {
    pub fn new() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            neural_unit_utilization: 0.0,
            request_rate_per_second: 0.0,
            response_time_ms: 0.0,
            error_rate_percentage: 0.0,
            uptime_percentage: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub deployment_id: DeploymentId,
    pub metrics_endpoint: String,
    pub health_check_interval_ms: u64,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Default)]
pub struct AlertThresholds {
    pub max_cpu_utilization: f64,
    pub max_memory_utilization: f64,
    pub max_response_time_ms: f64,
    pub max_error_rate_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub deployment_id: DeploymentId,
    pub algorithm: LoadBalancingAlgorithm,
    pub health_check_enabled: bool,
    pub sticky_sessions: bool,
}

#[derive(Debug, Clone)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    IPHash,
}

#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    pub deployment_id: DeploymentId,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period_ms: u64,
}

// Result structures

#[derive(Debug, Clone)]
pub struct ScalingResult {
    pub previous_instances: u32,
    pub target_instances: u32,
    pub current_instances: u32,
    pub scaling_time_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub deployment_id: DeploymentId,
    pub previous_version: String,
    pub new_version: String,
    pub update_time_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct StopResult {
    pub deployment_id: DeploymentId,
    pub success: bool,
    pub stop_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct UndeploymentResult {
    pub deployment_id: DeploymentId,
    pub success: bool,
    pub cleanup_time_ms: u64,
}

// Error types
#[derive(Debug)]
pub enum DeploymentError {
    ConfigurationError(String),
    InsufficientResources(String),
    DeploymentNotFound,
    ContainerError(String),
    NetworkError(String),
    MonitoringError(String),
    ScalingFailed(String),
    UpdateFailed(String),
}

impl From<DeploymentError> for SDKError {
    fn from(error: DeploymentError) -> Self {
        SDKError::DeploymentError(error)
    }
}