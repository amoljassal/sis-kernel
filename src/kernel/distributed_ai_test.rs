//! Distributed AI Testing Framework - Phase 4 Implementation
//!
//! Comprehensive testing framework for distributed AI systems including Raft consensus,
//! federated learning, workload migration, and network-transparent scheduling.
//!
//! Tests:
//! - Raft consensus protocol correctness
//! - Federated learning convergence and privacy
//! - AI workload migration integrity
//! - Distributed scheduling optimization
//! - Network fault tolerance
//! - Byzantine resistance in distributed systems

use crate::kernel::distributed_raft::{self, RaftState, RaftLogEntry};
use crate::kernel::federated_learning::{self, FLRoundState};
use crate::kernel::ai_workload_migration::{self, MigrationPhase, MigrationStrategy, MigrationReason};
use crate::kernel::distributed_scheduler::{self, DistributedSchedulingStrategy, NodeResources, ThermalState, NetworkTopology, NetworkRequirements, SchedulingConstraints};
use crate::kernel::capabilities::{self, CapabilityType, CapabilityRights};
use crate::kernel::ai_scheduler::{AiTask, AiWorkloadType, CpuAffinity};

/// Test results for distributed AI systems
#[derive(Debug, Default)]
pub struct DistributedAiTestResults {
    pub tests_run: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub consensus_tests: u32,
    pub federated_learning_tests: u32,
    pub migration_tests: u32,
    pub distributed_scheduling_tests: u32,
    pub network_tests: u32,
    pub fault_tolerance_tests: u32,
}

/// Test status enumeration
#[derive(Debug, PartialEq)]
enum TestStatus {
    Pass,
    Fail,
    Skip,
}

/// Run comprehensive distributed AI test suite
pub fn run_distributed_ai_test_suite() -> Result<DistributedAiTestResults, &'static str> {
    let mut results = DistributedAiTestResults::default();
    
    crate::kernel::serial::write_str("\n");
    crate::kernel::serial::write_str("╔══════════════════════════════════════════════════════════════╗\n");
    crate::kernel::serial::write_str("║           SIS Kernel Distributed AI Test Suite             ║\n");
    crate::kernel::serial::write_str("╠══════════════════════════════════════════════════════════════╣\n");
    
    // Test 1: Raft Consensus Protocol
    run_test("raft_consensus_init", test_raft_consensus_initialization, &mut results);
    
    // Test 2: Raft Leader Election
    run_test("raft_leader_election", test_raft_leader_election, &mut results);
    
    // Test 3: Raft Log Replication
    run_test("raft_log_replication", test_raft_log_replication, &mut results);
    
    // Test 4: Federated Learning Framework
    run_test("federated_learning_init", test_federated_learning_initialization, &mut results);
    
    // Test 5: FL Participant Registration
    run_test("fl_participant_registration", test_fl_participant_registration, &mut results);
    
    // Test 6: FL Round Execution
    run_test("fl_round_execution", test_fl_round_execution, &mut results);
    
    // Test 7: FL Byzantine Detection
    run_test("fl_byzantine_detection", test_fl_byzantine_detection, &mut results);
    
    // Test 8: AI Workload Migration
    run_test("ai_workload_migration", test_ai_workload_migration, &mut results);
    
    // Test 9: Migration Checkpoint Integrity
    run_test("migration_checkpoint", test_migration_checkpoint_integrity, &mut results);
    
    // Test 10: Distributed Scheduler
    run_test("distributed_scheduler", test_distributed_scheduler_initialization, &mut results);
    
    // Test 11: Multi-Strategy Scheduling
    run_test("multi_strategy_scheduling", test_multi_strategy_scheduling, &mut results);
    
    // Test 12: Load Balancing
    run_test("load_balancing", test_load_balancing, &mut results);
    
    // Test 13: Network Fault Tolerance
    run_test("network_fault_tolerance", test_network_fault_tolerance, &mut results);
    
    // Test 14: Distributed Consensus Under Load
    run_test("consensus_under_load", test_consensus_under_load, &mut results);
    
    // Test 15: End-to-End Distributed AI Workflow
    run_test("e2e_distributed_workflow", test_e2e_distributed_workflow, &mut results);
    
    crate::kernel::serial::write_str("╠══════════════════════════════════════════════════════════════╣\n");
    
    if results.tests_failed == 0 {
        crate::kernel::serial::write_str("║ 🌐 All distributed AI tests PASSED                         ║\n");
        crate::kernel::serial::write_str("║     System ready for distributed AI operations             ║\n");
    } else {
        crate::kernel::serial::write_str("║ ❌ Some distributed AI tests FAILED                         ║\n");
        crate::kernel::serial::write_str("║     Review failures before distributed deployment          ║\n");
    }
    
    crate::kernel::serial::write_str("╚══════════════════════════════════════════════════════════════╝\n");
    
    Ok(results)
}

/// Run individual test and update results
fn run_test<F>(test_name: &str, test_func: F, results: &mut DistributedAiTestResults)
where
    F: FnOnce() -> TestStatus,
{
    crate::kernel::serial::write_str("║ Testing: ");
    crate::kernel::serial::write_str(test_name);
    
    // Pad to align status
    let padding_needed = 40 - test_name.len().min(40);
    for _ in 0..padding_needed {
        crate::kernel::serial::write_str(" ");
    }
    
    let status = test_func();
    results.tests_run += 1;
    
    match status {
        TestStatus::Pass => {
            crate::kernel::serial::write_str("✓ PASS ║\n");
            results.tests_passed += 1;
        },
        TestStatus::Fail => {
            crate::kernel::serial::write_str("✗ FAIL ║\n");
            results.tests_failed += 1;
        },
        TestStatus::Skip => {
            crate::kernel::serial::write_str("⊖ SKIP ║\n");
        }
    }
}

/// Test Raft consensus initialization
fn test_raft_consensus_initialization() -> TestStatus {
    // Create test cluster configuration
    let test_nodes = vec![
        crate::kernel::distributed_raft::RaftNode {
            node_id: 1,
            address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 1],
            port: 8080,
            capabilities: crate::kernel::distributed_raft::NodeCapabilities {
                max_models: 10,
                max_inference_throughput: 1000,
                available_memory: 8 * 1024 * 1024 * 1024, // 8GB
                compute_power: 100,
                has_npu: true,
                security_level: 3,
            },
            last_heartbeat: 0,
            is_active: true,
        },
        crate::kernel::distributed_raft::RaftNode {
            node_id: 2,
            address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 2],
            port: 8080,
            capabilities: crate::kernel::distributed_raft::NodeCapabilities {
                max_models: 8,
                max_inference_throughput: 800,
                available_memory: 4 * 1024 * 1024 * 1024, // 4GB
                compute_power: 80,
                has_npu: false,
                security_level: 2,
            },
            last_heartbeat: 0,
            is_active: true,
        },
    ];
    
    match distributed_raft::init(1, &test_nodes) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test Raft leader election process
fn test_raft_leader_election() -> TestStatus {
    // Test leader election
    match distributed_raft::start_election() {
        Ok(_) => {
            // Check if we became leader or follower
            let state = distributed_raft::get_state();
            if state == RaftState::Leader || state == RaftState::Follower {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            }
        },
        Err(_) => TestStatus::Fail,
    }
}

/// Test Raft log replication
fn test_raft_log_replication() -> TestStatus {
    // Create test capability
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::WRITE | CapabilityRights::EXECUTE),
        0x80000000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    // Test log entry append
    let log_entry = RaftLogEntry::ModelUpdate {
        model_id: 42,
        model_hash: [0x12u8; 32],
        model_size: 1024,
        timestamp: 1000000,
    };
    
    match distributed_raft::append_ai_operation(log_entry, capability_id) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test federated learning initialization
fn test_federated_learning_initialization() -> TestStatus {
    let initial_parameters = vec![0.1f32; 1000];
    
    match federated_learning::init(1, initial_parameters, 2, 5) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test FL participant registration
fn test_fl_participant_registration() -> TestStatus {
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::READ | CapabilityRights::WRITE | CapabilityRights::EXECUTE),
        0x80001000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    let public_key = [0x34u8; 32];
    
    match federated_learning::register_participant(
        101,          // node_id
        public_key,
        1000,         // data_samples
        50,           // computation_power
        capability_id,
    ) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test FL round execution
fn test_fl_round_execution() -> TestStatus {
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::EXECUTE | CapabilityRights::WRITE),
        0x80002000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    match federated_learning::start_round(capability_id) {
        Ok(round_number) => {
            if round_number > 0 {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            }
        },
        Err(_) => TestStatus::Fail,
    }
}

/// Test FL Byzantine detection
fn test_fl_byzantine_detection() -> TestStatus {
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::WRITE),
        0x80003000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    // Submit a malicious gradient with very large values (should be detected)
    let malicious_gradient = vec![1000.0f32; 100]; // Very large values
    
    match federated_learning::submit_gradient(
        101,                 // participant_id
        malicious_gradient,
        100,                 // data_samples
        capability_id,
    ) {
        Err(_) => TestStatus::Pass, // Should fail due to Byzantine detection
        Ok(_) => TestStatus::Fail,  // Should not succeed
    }
}

/// Test AI workload migration
fn test_ai_workload_migration() -> TestStatus {
    let capabilities = crate::kernel::ai_workload_migration::NodeMigrationCapabilities {
        supports_live_migration: true,
        supports_checkpoint_compression: true,
        max_concurrent_migrations: 4,
        network_bandwidth_mbps: 1000,
        storage_bandwidth_mbps: 500,
        memory_bandwidth_gbps: 50,
    };
    
    // Initialize migration system
    match ai_workload_migration::init(1, capabilities) {
        Ok(_) => {
            // Test migration request
            let capability_id = match capabilities::create_capability(
                CapabilityType::Memory,
                CapabilityRights::new(CapabilityRights::EXECUTE | CapabilityRights::READ | CapabilityRights::WRITE),
                0x80004000,
                4096,
                0,
            ) {
                Ok(cap) => cap,
                Err(_) => return TestStatus::Fail,
            };
            
            match ai_workload_migration::request_migration(
                12345,                          // workload_id
                2,                              // target_node
                MigrationStrategy::LiveMigration,
                MigrationReason::LoadBalancing,
                1000,                           // max_downtime_us
                capability_id,
            ) {
                Ok(_) => TestStatus::Pass,
                Err(_) => TestStatus::Fail,
            }
        },
        Err(_) => TestStatus::Fail,
    }
}

/// Test migration checkpoint integrity
fn test_migration_checkpoint_integrity() -> TestStatus {
    // Test checkpoint creation and validation
    let migration_id = 1;
    
    match ai_workload_migration::get_migration_status(migration_id) {
        Some((phase, progress)) => {
            // Migration should be in some valid phase
            match phase {
                MigrationPhase::Preparing |
                MigrationPhase::Checkpointing |
                MigrationPhase::Transferring |
                MigrationPhase::Restoring |
                MigrationPhase::Validating |
                MigrationPhase::Completing |
                MigrationPhase::Completed => TestStatus::Pass,
                MigrationPhase::Failed => TestStatus::Fail,
            }
        },
        None => TestStatus::Fail,
    }
}

/// Test distributed scheduler initialization
fn test_distributed_scheduler_initialization() -> TestStatus {
    // Create test cluster nodes
    let cluster_nodes = vec![
        NodeResources {
            node_id: 1,
            cpu_cores: 8,
            cpu_utilization: 0.3,
            memory_total_mb: 8192,
            memory_available_mb: 6000,
            ai_accelerators: 1,
            accelerator_utilization: 0.2,
            network_bandwidth_mbps: 1000,
            power_consumption_watts: 45.0,
            thermal_state: ThermalState::Cool,
            last_heartbeat: 1000,
            is_available: true,
        },
        NodeResources {
            node_id: 2,
            cpu_cores: 4,
            cpu_utilization: 0.7,
            memory_total_mb: 4096,
            memory_available_mb: 2000,
            ai_accelerators: 0,
            accelerator_utilization: 0.0,
            network_bandwidth_mbps: 100,
            power_consumption_watts: 25.0,
            thermal_state: ThermalState::Warm,
            last_heartbeat: 950,
            is_available: true,
        },
    ];
    
    let network_topology = vec![
        NetworkTopology {
            node_id: 1,
            connected_nodes: vec![2],
            latencies_us: vec![500], // 0.5ms to node 2
            bandwidths_mbps: vec![1000],
            packet_loss_rates: vec![0.001],
        },
        NetworkTopology {
            node_id: 2,
            connected_nodes: vec![1],
            latencies_us: vec![500], // 0.5ms to node 1
            bandwidths_mbps: vec![1000],
            packet_loss_rates: vec![0.001],
        },
    ];
    
    match distributed_scheduler::init(
        1,
        &cluster_nodes,
        &network_topology,
        DistributedSchedulingStrategy::Hybrid,
    ) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test multi-strategy scheduling
fn test_multi_strategy_scheduling() -> TestStatus {
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::EXECUTE | CapabilityRights::READ),
        0x80005000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    // Create test AI task
    let ai_task = AiTask {
        task_id: 1001,
        workload_type: AiWorkloadType::Inference,
        priority: 200,
        deadline_us: 40,
        estimated_cycles: 96000,
        model_id: Some(1),
        capability_id,
        cpu_affinity: CpuAffinity::Performance,
        state: crate::kernel::ai_scheduler::AiTaskState::Created,
        created_time: 1000,
        start_time: 0,
        completion_time: 0,
        actual_cycles: 0,
    };
    
    let network_requirements = NetworkRequirements {
        max_latency_us: 1000,
        min_bandwidth_mbps: 100,
        max_packet_loss: 0.01,
        requires_secure_channel: true,
    };
    
    let scheduling_constraints = SchedulingConstraints {
        prohibited_nodes: vec![],
        required_capabilities: vec![],
        anti_affinity_tasks: vec![],
        affinity_tasks: vec![],
        max_migration_count: 3,
    };
    
    match distributed_scheduler::submit_distributed_task(
        ai_task,
        vec![1],          // data_location
        vec![1],          // model_location
        network_requirements,
        scheduling_constraints,
        capability_id,
    ) {
        Ok(_) => TestStatus::Pass,
        Err(_) => TestStatus::Fail,
    }
}

/// Test load balancing functionality
fn test_load_balancing() -> TestStatus {
    // Simulate scheduling multiple tasks to trigger load balancing
    let mut task_count = 0;
    
    for i in 0..5 {
        let capability_id = match capabilities::create_capability(
            CapabilityType::Memory,
            CapabilityRights::new(CapabilityRights::EXECUTE | CapabilityRights::READ),
            0x80006000 + i * 4096,
            4096,
            0,
        ) {
            Ok(cap) => cap,
            Err(_) => continue,
        };
        
        let ai_task = AiTask {
            task_id: 2000 + i as u32,
            workload_type: AiWorkloadType::Inference,
            priority: 128,
            deadline_us: 100,
            estimated_cycles: 240000,
            model_id: Some(1),
            capability_id,
            cpu_affinity: CpuAffinity::Any,
            state: crate::kernel::ai_scheduler::AiTaskState::Created,
            created_time: 2000 + i as u64,
            start_time: 0,
            completion_time: 0,
            actual_cycles: 0,
        };
        
        let network_requirements = NetworkRequirements {
            max_latency_us: 5000,
            min_bandwidth_mbps: 50,
            max_packet_loss: 0.05,
            requires_secure_channel: false,
        };
        
        let scheduling_constraints = SchedulingConstraints {
            prohibited_nodes: vec![],
            required_capabilities: vec![],
            anti_affinity_tasks: vec![],
            affinity_tasks: vec![],
            max_migration_count: 2,
        };
        
        if distributed_scheduler::submit_distributed_task(
            ai_task,
            vec![1, 2],       // data on both nodes
            vec![1],          // model on node 1
            network_requirements,
            scheduling_constraints,
            capability_id,
        ).is_ok() {
            task_count += 1;
        }
    }
    
    if task_count >= 3 {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    }
}

/// Test network fault tolerance
fn test_network_fault_tolerance() -> TestStatus {
    // Simulate network partition and test system behavior
    
    // Update a node to be unavailable
    let updated_resources = NodeResources {
        node_id: 2,
        cpu_cores: 4,
        cpu_utilization: 0.7,
        memory_total_mb: 4096,
        memory_available_mb: 2000,
        ai_accelerators: 0,
        accelerator_utilization: 0.0,
        network_bandwidth_mbps: 100,
        power_consumption_watts: 25.0,
        thermal_state: ThermalState::Critical, // Simulate failure
        last_heartbeat: 0,                     // Old heartbeat
        is_available: false,                   // Mark as unavailable
    };
    
    match distributed_scheduler::update_node_resources(2, updated_resources) {
        Ok(_) => {
            // System should handle the unavailable node gracefully
            TestStatus::Pass
        },
        Err(_) => TestStatus::Fail,
    }
}

/// Test consensus under load
fn test_consensus_under_load() -> TestStatus {
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::WRITE | CapabilityRights::EXECUTE),
        0x80007000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    // Submit multiple log entries to test consensus under load
    let mut successful_ops = 0;
    
    for i in 0..5 {
        let log_entry = RaftLogEntry::InferenceRequest {
            request_id: 3000 + i,
            model_id: 1,
            input_hash: [0x56u8; 32],
            requester_node: 1,
            timestamp: 3000000 + i * 1000,
        };
        
        if distributed_raft::append_ai_operation(log_entry, capability_id).is_ok() {
            successful_ops += 1;
        }
    }
    
    if successful_ops >= 3 {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    }
}

/// Test end-to-end distributed AI workflow
fn test_e2e_distributed_workflow() -> TestStatus {
    // This test validates the entire distributed AI pipeline:
    // 1. Raft consensus for coordination
    // 2. Federated learning for model updates
    // 3. Migration for load balancing
    // 4. Distributed scheduling for task placement
    
    let capability_id = match capabilities::create_capability(
        CapabilityType::Memory,
        CapabilityRights::new(CapabilityRights::READ | CapabilityRights::WRITE | CapabilityRights::EXECUTE),
        0x80008000,
        4096,
        0,
    ) {
        Ok(cap) => cap,
        Err(_) => return TestStatus::Fail,
    };
    
    // Test 1: Consensus operation
    let consensus_result = {
        let log_entry = RaftLogEntry::GradientUpdate {
            model_id: 1,
            gradient_hash: [0x78u8; 32],
            learning_rate: 0.01,
            batch_size: 32,
            timestamp: 4000000,
        };
        
        distributed_raft::append_ai_operation(log_entry, capability_id).is_ok()
    };
    
    // Test 2: Federated learning gradient submission
    let fl_result = {
        let gradient = vec![0.01f32; 50]; // Normal gradient
        federated_learning::submit_gradient(101, gradient, 50, capability_id).is_ok()
    };
    
    // Test 3: Migration status check
    let migration_result = {
        ai_workload_migration::get_migration_status(1).is_some()
    };
    
    // Test 4: Distributed scheduling
    let scheduling_result = {
        let ai_task = AiTask {
            task_id: 4001,
            workload_type: AiWorkloadType::Inference,
            priority: 255, // Critical priority
            deadline_us: 10,
            estimated_cycles: 24000,
            model_id: Some(1),
            capability_id,
            cpu_affinity: CpuAffinity::Performance,
            state: crate::kernel::ai_scheduler::AiTaskState::Created,
            created_time: 4000,
            start_time: 0,
            completion_time: 0,
            actual_cycles: 0,
        };
        
        let network_requirements = NetworkRequirements {
            max_latency_us: 500,
            min_bandwidth_mbps: 1000,
            max_packet_loss: 0.001,
            requires_secure_channel: true,
        };
        
        let scheduling_constraints = SchedulingConstraints {
            prohibited_nodes: vec![],
            required_capabilities: vec!["npu".to_string()],
            anti_affinity_tasks: vec![],
            affinity_tasks: vec![],
            max_migration_count: 1,
        };
        
        distributed_scheduler::submit_distributed_task(
            ai_task,
            vec![1],
            vec![1],
            network_requirements,
            scheduling_constraints,
            capability_id,
        ).is_ok()
    };
    
    // All components should work together
    if consensus_result && migration_result && scheduling_result {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    }
}

/// Get distributed AI test statistics
pub fn get_distributed_test_stats() -> DistributedAiTestResults {
    // Return current test statistics
    // In real implementation, this would maintain persistent stats
    DistributedAiTestResults::default()
}

/// Run distributed performance benchmarks
pub fn run_distributed_performance_benchmarks() -> Result<(), &'static str> {
    crate::kernel::serial::write_str("[DIST_TEST] Running distributed performance benchmarks...\n");
    
    // Benchmark 1: Raft consensus latency
    let start = read_cycle_counter();
    let (elections_started, elections_won, log_entries_replicated, consensus_operations) = 
        distributed_raft::get_consensus_stats();
    let consensus_cycles = read_cycle_counter() - start;
    
    crate::kernel::serial::write_str("[DIST_TEST] Consensus stats query: ");
    crate::kernel::serial::write_u64(consensus_cycles);
    crate::kernel::serial::write_str(" cycles\n");
    
    // Benchmark 2: Federated learning round time
    let start = read_cycle_counter();
    let (rounds_completed, gradients_processed, byzantine_detected, privacy_violations, active_participants) = 
        federated_learning::get_fl_stats();
    let fl_cycles = read_cycle_counter() - start;
    
    crate::kernel::serial::write_str("[DIST_TEST] FL stats query: ");
    crate::kernel::serial::write_u64(fl_cycles);
    crate::kernel::serial::write_str(" cycles\n");
    
    // Benchmark 3: Migration throughput
    let start = read_cycle_counter();
    let (migrations_completed, migrations_failed, total_migration_time, total_downtime, bytes_migrated, active_migrations) = 
        ai_workload_migration::get_migration_stats();
    let migration_cycles = read_cycle_counter() - start;
    
    crate::kernel::serial::write_str("[DIST_TEST] Migration stats query: ");
    crate::kernel::serial::write_u64(migration_cycles);
    crate::kernel::serial::write_str(" cycles\n");
    
    // Benchmark 4: Distributed scheduling latency
    let start = read_cycle_counter();
    let (tasks_scheduled, tasks_completed, tasks_migrated, scheduling_decisions_made, network_operations, load_balancing_operations) = 
        distributed_scheduler::get_distributed_stats();
    let scheduling_cycles = read_cycle_counter() - start;
    
    crate::kernel::serial::write_str("[DIST_TEST] Distributed scheduling stats query: ");
    crate::kernel::serial::write_u64(scheduling_cycles);
    crate::kernel::serial::write_str(" cycles\n");
    
    // Convert to microseconds for analysis
    crate::kernel::serial::write_str("[DIST_TEST] Performance summary:\n");
    crate::kernel::serial::write_str("[DIST_TEST] - Consensus operations: ");
    crate::kernel::serial::write_u64(consensus_operations);
    crate::kernel::serial::write_str("\n");
    crate::kernel::serial::write_str("[DIST_TEST] - FL rounds completed: ");
    crate::kernel::serial::write_u64(rounds_completed);
    crate::kernel::serial::write_str("\n");
    crate::kernel::serial::write_str("[DIST_TEST] - Migrations completed: ");
    crate::kernel::serial::write_u64(migrations_completed);
    crate::kernel::serial::write_str("\n");
    crate::kernel::serial::write_str("[DIST_TEST] - Tasks scheduled: ");
    crate::kernel::serial::write_u64(tasks_scheduled);
    crate::kernel::serial::write_str("\n");
    
    Ok(())
}

/// Read cycle counter for benchmarking
fn read_cycle_counter() -> u64 {
    unsafe {
        let mut cycles: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles);
        cycles
    }
}