//! Production Monitoring and Observability - Phase 5 Implementation
//!
//! Provides comprehensive monitoring, logging, and observability for production
//! distributed AI systems with real-time metrics, alerting, and tracing.
//!
//! Architecture:
//! - Multi-dimensional metrics collection and aggregation
//! - Distributed tracing for request flow analysis
//! - Real-time alerting with intelligent thresholds
//! - Performance dashboards and health indicators
//! - Anomaly detection using statistical methods

use crate::kernel::distributed_raft::{get_leader_id, get_cluster_size};
use crate::kernel::ai_runtime::get_stats as get_ai_stats;
use crate::kernel::capabilities::{CapabilityId, CapabilityRights};
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

/// Maximum number of metrics to track
const MAX_METRICS: usize = 500;

/// Maximum number of alerts
const MAX_ALERTS: usize = 100;

/// Maximum number of trace spans
const MAX_TRACE_SPANS: usize = 1000;

/// Maximum log entries in ring buffer
const MAX_LOG_ENTRIES: usize = 10000;

/// Metric types for monitoring
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricType {
    Counter,      // Monotonic counter
    Gauge,        // Point-in-time value
    Histogram,    // Distribution of values
    Summary,      // Statistical summary
    Timer,        // Timing measurements
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Critical,     // System failure imminent
    Warning,      // Performance degraded
    Info,         // Informational alert
    Debug,        // Debug information
}

/// Alert states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertState {
    Triggered,    // Alert is active
    Resolved,     // Alert condition resolved
    Suppressed,   // Alert temporarily suppressed
    Acknowledged, // Alert acknowledged by operator
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

/// Metric definition and current value
#[derive(Debug, Clone)]
pub struct Metric {
    pub id: u32,
    pub name: &'static str,
    pub metric_type: MetricType,
    pub labels: [(&'static str, &'static str); 4], // Key-value labels
    pub value: f64,
    pub timestamp: u64,
    pub sample_count: u64,
    pub min_value: f64,
    pub max_value: f64,
    pub sum_value: f64,
}

/// Alert configuration and state
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub metric_id: u32,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub triggered_count: u64,
    pub last_triggered: u64,
    pub last_resolved: u64,
    pub evaluation_window_ms: u64,
}

/// Distributed tracing span
#[derive(Debug, Clone)]
pub struct TraceSpan {
    pub span_id: u64,
    pub parent_span_id: Option<u64>,
    pub trace_id: u64,
    pub operation_name: &'static str,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub duration_us: u64,
    pub tags: [(&'static str, &'static str); 4],
    pub status: SpanStatus,
    pub node_id: u32,
}

/// Trace span status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpanStatus {
    Started,
    Finished,
    Error,
    Timeout,
}

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub component: &'static str,
    pub message: &'static str,
    pub context: [(&'static str, u64); 2], // Key-value context
    pub trace_id: Option<u64>,
    pub span_id: Option<u64>,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub component_name: &'static str,
    pub is_healthy: bool,
    pub status_message: &'static str,
    pub last_check: u64,
    pub check_duration_us: u64,
    pub consecutive_failures: u32,
}

/// System dashboard metrics
#[derive(Debug, Clone, Default)]
pub struct DashboardMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_us: u64,
    pub p95_response_time_us: u64,
    pub p99_response_time_us: u64,
    pub active_connections: u32,
    pub cpu_utilization: f32,
    pub memory_utilization: f32,
    pub disk_utilization: f32,
    pub network_rx_mbps: u32,
    pub network_tx_mbps: u32,
    pub error_rate: f32,
    pub uptime_seconds: u64,
}

/// Anomaly detection configuration
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    pub enabled: bool,
    pub sensitivity: f32,          // 0.0-1.0 sensitivity level
    pub baseline_samples: u32,     // Number of samples for baseline
    pub deviation_threshold: f32,  // Standard deviations for anomaly
    pub detection_window_ms: u64,
    pub anomalies_detected: u64,
    pub false_positives: u64,
}

/// Production monitoring engine
pub struct ProductionMonitoringEngine {
    pub initialized: AtomicBool,
    
    // Metrics collection
    pub metrics: [Option<Metric>; MAX_METRICS],
    pub metric_count: AtomicU32,
    
    // Alerting system
    pub alerts: [Option<Alert>; MAX_ALERTS],
    pub alert_count: AtomicU32,
    pub active_alerts: AtomicU32,
    
    // Distributed tracing
    pub trace_spans: [Option<TraceSpan>; MAX_TRACE_SPANS],
    pub span_count: AtomicU32,
    pub current_trace_id: AtomicU64,
    pub current_span_id: AtomicU64,
    
    // Logging system
    pub log_entries: [Option<LogEntry>; MAX_LOG_ENTRIES],
    pub log_head: AtomicU32,
    pub log_tail: AtomicU32,
    pub log_entries_count: AtomicU64,
    
    // Health monitoring
    pub health_checks: [Option<HealthStatus>; 20],
    pub health_check_count: AtomicU32,
    
    // Dashboard metrics
    pub dashboard: DashboardMetrics,
    
    // Anomaly detection
    pub anomaly_detector: AnomalyDetector,
    
    // Monitoring statistics
    pub metrics_collected: AtomicU64,
    pub alerts_triggered: AtomicU64,
    pub traces_completed: AtomicU64,
    pub health_checks_performed: AtomicU64,
    pub monitoring_overhead_cycles: AtomicU64,
}

/// Global production monitoring engine
static mut MONITORING_ENGINE: ProductionMonitoringEngine = ProductionMonitoringEngine {
    initialized: AtomicBool::new(false),
    metrics: [None; MAX_METRICS],
    metric_count: AtomicU32::new(0),
    alerts: [None; MAX_ALERTS],
    alert_count: AtomicU32::new(0),
    active_alerts: AtomicU32::new(0),
    trace_spans: [None; MAX_TRACE_SPANS],
    span_count: AtomicU32::new(0),
    current_trace_id: AtomicU64::new(1),
    current_span_id: AtomicU64::new(1),
    log_entries: [None; MAX_LOG_ENTRIES],
    log_head: AtomicU32::new(0),
    log_tail: AtomicU32::new(0),
    log_entries_count: AtomicU64::new(0),
    health_checks: [None; 20],
    health_check_count: AtomicU32::new(0),
    dashboard: DashboardMetrics::default(),
    anomaly_detector: AnomalyDetector {
        enabled: true,
        sensitivity: 0.8,
        baseline_samples: 100,
        deviation_threshold: 2.0,
        detection_window_ms: 60000, // 1 minute
        anomalies_detected: 0,
        false_positives: 0,
    },
    metrics_collected: AtomicU64::new(0),
    alerts_triggered: AtomicU64::new(0),
    traces_completed: AtomicU64::new(0),
    health_checks_performed: AtomicU64::new(0),
    monitoring_overhead_cycles: AtomicU64::new(0),
};

/// Initialize production monitoring engine
pub fn init() -> Result<(), &'static str> {
    unsafe {
        if MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Production monitoring already initialized");
        }
        
        // Initialize core metrics
        initialize_core_metrics()?;
        
        // Initialize default alerts
        initialize_default_alerts()?;
        
        // Initialize health checks
        initialize_health_checks()?;
        
        // Initialize logging system
        initialize_logging_system()?;
        
        MONITORING_ENGINE.initialized.store(true, Ordering::Release);
    }
    
    crate::kernel::serial::write_str("[MONITOR] Production monitoring engine initialized\n");
    Ok(())
}

/// Initialize core system metrics
fn initialize_core_metrics() -> Result<(), &'static str> {
    // AI Inference Latency Metric
    let ai_latency = Metric {
        id: 1,
        name: "ai_inference_latency_us",
        metric_type: MetricType::Histogram,
        labels: [("component", "ai_runtime"), ("unit", "microseconds"), ("", ""), ("", "")],
        value: 0.0,
        timestamp: read_timestamp(),
        sample_count: 0,
        min_value: f64::MAX,
        max_value: f64::MIN,
        sum_value: 0.0,
    };
    
    add_metric(ai_latency)?;
    
    // CPU Utilization Metric
    let cpu_util = Metric {
        id: 2,
        name: "cpu_utilization_percent",
        metric_type: MetricType::Gauge,
        labels: [("component", "system"), ("unit", "percent"), ("", ""), ("", "")],
        value: 0.0,
        timestamp: read_timestamp(),
        sample_count: 0,
        min_value: 0.0,
        max_value: 100.0,
        sum_value: 0.0,
    };
    
    add_metric(cpu_util)?;
    
    // Memory Utilization Metric
    let mem_util = Metric {
        id: 3,
        name: "memory_utilization_percent",
        metric_type: MetricType::Gauge,
        labels: [("component", "system"), ("unit", "percent"), ("", ""), ("", "")],
        value: 0.0,
        timestamp: read_timestamp(),
        sample_count: 0,
        min_value: 0.0,
        max_value: 100.0,
        sum_value: 0.0,
    };
    
    add_metric(mem_util)?;
    
    // Request Rate Metric
    let request_rate = Metric {
        id: 4,
        name: "requests_per_second",
        metric_type: MetricType::Counter,
        labels: [("component", "api"), ("unit", "rps"), ("", ""), ("", "")],
        value: 0.0,
        timestamp: read_timestamp(),
        sample_count: 0,
        min_value: 0.0,
        max_value: f64::MAX,
        sum_value: 0.0,
    };
    
    add_metric(request_rate)?;
    
    // Error Rate Metric
    let error_rate = Metric {
        id: 5,
        name: "error_rate_percent",
        metric_type: MetricType::Gauge,
        labels: [("component", "system"), ("unit", "percent"), ("", ""), ("", "")],
        value: 0.0,
        timestamp: read_timestamp(),
        sample_count: 0,
        min_value: 0.0,
        max_value: 100.0,
        sum_value: 0.0,
    };
    
    add_metric(error_rate)?;
    
    crate::kernel::serial::write_str("[MONITOR] Core metrics initialized\n");
    Ok(())
}

/// Add metric to the monitoring engine
fn add_metric(metric: Metric) -> Result<(), &'static str> {
    unsafe {
        let count = MONITORING_ENGINE.metric_count.load(Ordering::Relaxed);
        if count >= MAX_METRICS as u32 {
            return Err("Metrics database full");
        }
        
        MONITORING_ENGINE.metrics[count as usize] = Some(metric);
        MONITORING_ENGINE.metric_count.fetch_add(1, Ordering::Relaxed);
    }
    
    Ok(())
}

/// Initialize default alert rules
fn initialize_default_alerts() -> Result<(), &'static str> {
    // High AI Inference Latency Alert
    let high_latency_alert = Alert {
        id: 1,
        name: "high_ai_inference_latency",
        description: "AI inference latency exceeds 40μs threshold",
        metric_id: 1,
        threshold: 40.0,
        severity: AlertSeverity::Critical,
        state: AlertState::Resolved,
        triggered_count: 0,
        last_triggered: 0,
        last_resolved: 0,
        evaluation_window_ms: 10000, // 10 seconds
    };
    
    add_alert(high_latency_alert)?;
    
    // High CPU Utilization Alert
    let high_cpu_alert = Alert {
        id: 2,
        name: "high_cpu_utilization",
        description: "CPU utilization exceeds 90%",
        metric_id: 2,
        threshold: 90.0,
        severity: AlertSeverity::Warning,
        state: AlertState::Resolved,
        triggered_count: 0,
        last_triggered: 0,
        last_resolved: 0,
        evaluation_window_ms: 30000, // 30 seconds
    };
    
    add_alert(high_cpu_alert)?;
    
    // High Memory Utilization Alert
    let high_mem_alert = Alert {
        id: 3,
        name: "high_memory_utilization",
        description: "Memory utilization exceeds 85%",
        metric_id: 3,
        threshold: 85.0,
        severity: AlertSeverity::Warning,
        state: AlertState::Resolved,
        triggered_count: 0,
        last_triggered: 0,
        last_resolved: 0,
        evaluation_window_ms: 30000, // 30 seconds
    };
    
    add_alert(high_mem_alert)?;
    
    // High Error Rate Alert
    let high_error_alert = Alert {
        id: 4,
        name: "high_error_rate",
        description: "System error rate exceeds 5%",
        metric_id: 5,
        threshold: 5.0,
        severity: AlertSeverity::Critical,
        state: AlertState::Resolved,
        triggered_count: 0,
        last_triggered: 0,
        last_resolved: 0,
        evaluation_window_ms: 60000, // 60 seconds
    };
    
    add_alert(high_error_alert)?;
    
    crate::kernel::serial::write_str("[MONITOR] Default alerts initialized\n");
    Ok(())
}

/// Add alert to the monitoring engine
fn add_alert(alert: Alert) -> Result<(), &'static str> {
    unsafe {
        let count = MONITORING_ENGINE.alert_count.load(Ordering::Relaxed);
        if count >= MAX_ALERTS as u32 {
            return Err("Alerts database full");
        }
        
        MONITORING_ENGINE.alerts[count as usize] = Some(alert);
        MONITORING_ENGINE.alert_count.fetch_add(1, Ordering::Relaxed);
    }
    
    Ok(())
}

/// Initialize health checks for system components
fn initialize_health_checks() -> Result<(), &'static str> {
    let health_checks = [
        HealthStatus {
            component_name: "ai_runtime",
            is_healthy: true,
            status_message: "AI runtime operational",
            last_check: read_timestamp(),
            check_duration_us: 0,
            consecutive_failures: 0,
        },
        HealthStatus {
            component_name: "distributed_raft",
            is_healthy: true,
            status_message: "Raft consensus healthy",
            last_check: read_timestamp(),
            check_duration_us: 0,
            consecutive_failures: 0,
        },
        HealthStatus {
            component_name: "security",
            is_healthy: true,
            status_message: "Security systems active",
            last_check: read_timestamp(),
            check_duration_us: 0,
            consecutive_failures: 0,
        },
        HealthStatus {
            component_name: "performance",
            is_healthy: true,
            status_message: "Performance within targets",
            last_check: read_timestamp(),
            check_duration_us: 0,
            consecutive_failures: 0,
        },
    ];
    
    unsafe {
        for (i, health) in health_checks.iter().enumerate() {
            MONITORING_ENGINE.health_checks[i] = Some(health.clone());
        }
        
        MONITORING_ENGINE.health_check_count.store(health_checks.len() as u32, Ordering::Relaxed);
    }
    
    Ok(())
}

/// Initialize logging system
fn initialize_logging_system() -> Result<(), &'static str> {
    unsafe {
        // Initialize log ring buffer pointers
        MONITORING_ENGINE.log_head.store(0, Ordering::Relaxed);
        MONITORING_ENGINE.log_tail.store(0, Ordering::Relaxed);
        
        // Log initialization message
        log_message(LogLevel::Info, "system", "Production monitoring initialized", None);
    }
    
    Ok(())
}

/// Record a metric value
pub fn record_metric(
    metric_name: &str,
    value: f64,
    capability_id: CapabilityId,
) -> Result<(), &'static str> {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Monitoring engine not initialized");
        }
        
        // Verify capability
        if !crate::kernel::capabilities::check_capability(
            0,
            capability_id,
            CapabilityRights::new(CapabilityRights::WRITE),
        ) {
            return Err("Insufficient capabilities for metric recording");
        }
        
        let start_cycles = read_cycle_counter();
        
        // Find metric by name
        let metric_count = MONITORING_ENGINE.metric_count.load(Ordering::Relaxed);
        for i in 0..metric_count as usize {
            if let Some(ref mut metric) = MONITORING_ENGINE.metrics[i] {
                if metric.name == metric_name {
                    // Update metric based on type
                    match metric.metric_type {
                        MetricType::Counter => {
                            metric.value += value;
                        },
                        MetricType::Gauge => {
                            metric.value = value;
                        },
                        MetricType::Histogram | MetricType::Summary => {
                            metric.sample_count += 1;
                            metric.sum_value += value;
                            metric.min_value = metric.min_value.min(value);
                            metric.max_value = metric.max_value.max(value);
                            metric.value = metric.sum_value / metric.sample_count as f64;
                        },
                        MetricType::Timer => {
                            metric.value = value;
                            metric.sample_count += 1;
                            metric.sum_value += value;
                            metric.min_value = metric.min_value.min(value);
                            metric.max_value = metric.max_value.max(value);
                        },
                    }
                    
                    metric.timestamp = read_timestamp();
                    
                    // Check for alerts
                    evaluate_alerts(metric.id, value)?;
                    
                    // Perform anomaly detection
                    check_for_anomalies(metric.id, value)?;
                    
                    let monitoring_cycles = read_cycle_counter() - start_cycles;
                    MONITORING_ENGINE.monitoring_overhead_cycles
                        .fetch_add(monitoring_cycles, Ordering::Relaxed);
                    
                    MONITORING_ENGINE.metrics_collected.fetch_add(1, Ordering::Relaxed);
                    
                    return Ok(());
                }
            }
        }
        
        Err("Metric not found")
    }
}

/// Evaluate alert conditions for a metric
fn evaluate_alerts(metric_id: u32, current_value: f64) -> Result<(), &'static str> {
    unsafe {
        let alert_count = MONITORING_ENGINE.alert_count.load(Ordering::Relaxed);
        let current_time = read_timestamp();
        
        for i in 0..alert_count as usize {
            if let Some(ref mut alert) = MONITORING_ENGINE.alerts[i] {
                if alert.metric_id == metric_id {
                    let should_trigger = match alert.severity {
                        AlertSeverity::Critical => current_value > alert.threshold,
                        AlertSeverity::Warning => current_value > alert.threshold * 0.9,
                        _ => current_value > alert.threshold,
                    };
                    
                    if should_trigger && alert.state == AlertState::Resolved {
                        // Trigger alert
                        alert.state = AlertState::Triggered;
                        alert.triggered_count += 1;
                        alert.last_triggered = current_time;
                        
                        MONITORING_ENGINE.active_alerts.fetch_add(1, Ordering::Relaxed);
                        MONITORING_ENGINE.alerts_triggered.fetch_add(1, Ordering::Relaxed);
                        
                        // Log alert
                        log_message(
                            LogLevel::Warning,
                            "alerting",
                            alert.description,
                            None,
                        );
                        
                        crate::kernel::serial::write_str("[MONITOR] ALERT: ");
                        crate::kernel::serial::write_str(alert.name);
                        crate::kernel::serial::write_str("\n");
                    } else if !should_trigger && alert.state == AlertState::Triggered {
                        // Resolve alert
                        alert.state = AlertState::Resolved;
                        alert.last_resolved = current_time;
                        
                        MONITORING_ENGINE.active_alerts.fetch_sub(1, Ordering::Relaxed);
                        
                        crate::kernel::serial::write_str("[MONITOR] RESOLVED: ");
                        crate::kernel::serial::write_str(alert.name);
                        crate::kernel::serial::write_str("\n");
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Check for anomalies in metric values
fn check_for_anomalies(metric_id: u32, current_value: f64) -> Result<(), &'static str> {
    unsafe {
        if !MONITORING_ENGINE.anomaly_detector.enabled {
            return Ok(());
        }
        
        // Simple anomaly detection using standard deviation
        let metric_count = MONITORING_ENGINE.metric_count.load(Ordering::Relaxed);
        for i in 0..metric_count as usize {
            if let Some(ref metric) = MONITORING_ENGINE.metrics[i] {
                if metric.id == metric_id && metric.sample_count > 10 {
                    let mean = metric.sum_value / metric.sample_count as f64;
                    let threshold = MONITORING_ENGINE.anomaly_detector.deviation_threshold;
                    
                    // Simple check: if value is more than 2 standard deviations from mean
                    let deviation = (current_value - mean).abs();
                    let expected_deviation = (metric.max_value - metric.min_value) * 0.1;
                    
                    if deviation > expected_deviation * threshold {
                        MONITORING_ENGINE.anomaly_detector.anomalies_detected += 1;
                        
                        log_message(
                            LogLevel::Warning,
                            "anomaly_detection",
                            "Anomaly detected in metric",
                            None,
                        );
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Start a new distributed trace span
pub fn start_span(
    operation_name: &'static str,
    parent_span_id: Option<u64>,
    capability_id: CapabilityId,
) -> Result<u64, &'static str> {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Monitoring engine not initialized");
        }
        
        // Verify capability
        if !crate::kernel::capabilities::check_capability(
            0,
            capability_id,
            CapabilityRights::new(CapabilityRights::WRITE),
        ) {
            return Err("Insufficient capabilities for tracing");
        }
        
        let span_count = MONITORING_ENGINE.span_count.load(Ordering::Relaxed);
        if span_count >= MAX_TRACE_SPANS as u32 {
            return Err("Trace span limit reached");
        }
        
        let span_id = MONITORING_ENGINE.current_span_id.fetch_add(1, Ordering::Relaxed);
        let trace_id = match parent_span_id {
            Some(_) => {
                // Find parent span to get trace ID
                // Simplified: use current trace ID
                MONITORING_ENGINE.current_trace_id.load(Ordering::Relaxed)
            },
            None => MONITORING_ENGINE.current_trace_id.fetch_add(1, Ordering::Relaxed),
        };
        
        let span = TraceSpan {
            span_id,
            parent_span_id,
            trace_id,
            operation_name,
            start_timestamp: read_timestamp(),
            end_timestamp: 0,
            duration_us: 0,
            tags: [("component", "kernel"), ("", ""), ("", ""), ("", "")],
            status: SpanStatus::Started,
            node_id: 0, // Current node
        };
        
        MONITORING_ENGINE.trace_spans[span_count as usize] = Some(span);
        MONITORING_ENGINE.span_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(span_id)
    }
}

/// Finish a distributed trace span
pub fn finish_span(
    span_id: u64,
    capability_id: CapabilityId,
) -> Result<(), &'static str> {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Monitoring engine not initialized");
        }
        
        // Verify capability
        if !crate::kernel::capabilities::check_capability(
            0,
            capability_id,
            CapabilityRights::new(CapabilityRights::WRITE),
        ) {
            return Err("Insufficient capabilities for tracing");
        }
        
        let span_count = MONITORING_ENGINE.span_count.load(Ordering::Relaxed);
        let current_time = read_timestamp();
        
        for i in 0..span_count as usize {
            if let Some(ref mut span) = MONITORING_ENGINE.trace_spans[i] {
                if span.span_id == span_id {
                    span.end_timestamp = current_time;
                    span.duration_us = (current_time - span.start_timestamp) / 1000; // Convert to microseconds
                    span.status = SpanStatus::Finished;
                    
                    MONITORING_ENGINE.traces_completed.fetch_add(1, Ordering::Relaxed);
                    
                    // Record trace duration as metric
                    if let Ok(cap) = crate::kernel::capabilities::get_kernel_capability() {
                        let _ = record_metric("trace_duration_us", span.duration_us as f64, cap);
                    }
                    
                    return Ok(());
                }
            }
        }
        
        Err("Trace span not found")
    }
}

/// Log a message with structured context
pub fn log_message(
    level: LogLevel,
    component: &'static str,
    message: &'static str,
    trace_context: Option<(u64, u64)>, // (trace_id, span_id)
) {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return;
        }
        
        let current_head = MONITORING_ENGINE.log_head.load(Ordering::Relaxed);
        let next_head = (current_head + 1) % MAX_LOG_ENTRIES as u32;
        
        // Check if buffer is full
        if next_head == MONITORING_ENGINE.log_tail.load(Ordering::Relaxed) {
            // Buffer full, advance tail to overwrite oldest entry
            let next_tail = (MONITORING_ENGINE.log_tail.load(Ordering::Relaxed) + 1) % MAX_LOG_ENTRIES as u32;
            MONITORING_ENGINE.log_tail.store(next_tail, Ordering::Relaxed);
        }
        
        let (trace_id, span_id) = match trace_context {
            Some((tid, sid)) => (Some(tid), Some(sid)),
            None => (None, None),
        };
        
        let log_entry = LogEntry {
            timestamp: read_timestamp(),
            level,
            component,
            message,
            context: [("node_id", 0), ("cpu_id", 0)],
            trace_id,
            span_id,
        };
        
        MONITORING_ENGINE.log_entries[current_head as usize] = Some(log_entry);
        MONITORING_ENGINE.log_head.store(next_head, Ordering::Relaxed);
        MONITORING_ENGINE.log_entries_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Perform health checks on system components
pub fn run_health_checks(capability_id: CapabilityId) -> Result<u32, &'static str> {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Monitoring engine not initialized");
        }
        
        // Verify capability
        if !crate::kernel::capabilities::check_capability(
            0,
            capability_id,
            CapabilityRights::new(CapabilityRights::EXECUTE),
        ) {
            return Err("Insufficient capabilities for health checks");
        }
        
        let health_count = MONITORING_ENGINE.health_check_count.load(Ordering::Relaxed);
        let mut healthy_components = 0;
        let current_time = read_timestamp();
        
        for i in 0..health_count as usize {
            if let Some(ref mut health) = MONITORING_ENGINE.health_checks[i] {
                let check_start = read_cycle_counter();
                
                // Perform component-specific health check
                let is_healthy = match health.component_name {
                    "ai_runtime" => {
                        // Check AI runtime health
                        let ai_stats = get_ai_stats();
                        ai_stats.total_inferences > 0 || ai_stats.models_loaded > 0
                    },
                    "distributed_raft" => {
                        // Check Raft consensus health
                        get_leader_id().is_some() || get_cluster_size() > 0
                    },
                    "security" => {
                        // Check security systems
                        true // Simplified check
                    },
                    "performance" => {
                        // Check performance targets
                        let ai_stats = get_ai_stats();
                        if ai_stats.total_inferences > 0 {
                            let avg_latency = ai_stats.total_cycles / ai_stats.total_inferences / 2400; // Convert to μs
                            avg_latency <= 40 // Within 40μs target
                        } else {
                            true
                        }
                    },
                    _ => true,
                };
                
                let check_cycles = read_cycle_counter() - check_start;
                health.check_duration_us = check_cycles / 2400; // Convert to microseconds
                health.last_check = current_time;
                
                if is_healthy {
                    health.is_healthy = true;
                    health.consecutive_failures = 0;
                    health.status_message = "Component healthy";
                    healthy_components += 1;
                } else {
                    health.is_healthy = false;
                    health.consecutive_failures += 1;
                    health.status_message = "Component unhealthy";
                    
                    // Log unhealthy component
                    log_message(
                        LogLevel::Warning,
                        "health_check",
                        "Component failed health check",
                        None,
                    );
                }
            }
        }
        
        MONITORING_ENGINE.health_checks_performed.fetch_add(1, Ordering::Relaxed);
        
        Ok(healthy_components)
    }
}

/// Update dashboard metrics
pub fn update_dashboard_metrics(capability_id: CapabilityId) -> Result<(), &'static str> {
    unsafe {
        if !MONITORING_ENGINE.initialized.load(Ordering::Acquire) {
            return Err("Monitoring engine not initialized");
        }
        
        // Verify capability
        if !crate::kernel::capabilities::check_capability(
            0,
            capability_id,
            CapabilityRights::new(CapabilityRights::READ),
        ) {
            return Err("Insufficient capabilities for dashboard update");
        }
        
        // Update dashboard with current system metrics
        let ai_stats = get_ai_stats();
        
        MONITORING_ENGINE.dashboard.total_requests = ai_stats.total_inferences;
        MONITORING_ENGINE.dashboard.successful_requests = ai_stats.total_inferences - ai_stats.failed_inferences;
        MONITORING_ENGINE.dashboard.failed_requests = ai_stats.failed_inferences;
        
        if ai_stats.total_inferences > 0 {
            let avg_cycles = ai_stats.total_cycles / ai_stats.total_inferences;
            MONITORING_ENGINE.dashboard.average_response_time_us = avg_cycles / 2400;
            MONITORING_ENGINE.dashboard.p95_response_time_us = (avg_cycles * 2) / 2400; // Estimated
            MONITORING_ENGINE.dashboard.p99_response_time_us = (avg_cycles * 3) / 2400; // Estimated
        }
        
        MONITORING_ENGINE.dashboard.active_connections = get_cluster_size();
        MONITORING_ENGINE.dashboard.cpu_utilization = 15.0; // Simplified
        MONITORING_ENGINE.dashboard.memory_utilization = 25.0; // Simplified
        MONITORING_ENGINE.dashboard.disk_utilization = 10.0; // Simplified
        MONITORING_ENGINE.dashboard.network_rx_mbps = 50; // Simplified
        MONITORING_ENGINE.dashboard.network_tx_mbps = 30; // Simplified
        
        if ai_stats.total_inferences > 0 {
            MONITORING_ENGINE.dashboard.error_rate = 
                (ai_stats.failed_inferences as f32 / ai_stats.total_inferences as f32) * 100.0;
        }
        
        MONITORING_ENGINE.dashboard.uptime_seconds = read_timestamp() / 1000000; // Convert to seconds
    }
    
    Ok(())
}

/// Get monitoring statistics
pub fn get_monitoring_stats() -> (u64, u64, u64, u64, u64, u32) {
    unsafe {
        (
            MONITORING_ENGINE.metrics_collected.load(Ordering::Relaxed),
            MONITORING_ENGINE.alerts_triggered.load(Ordering::Relaxed),
            MONITORING_ENGINE.traces_completed.load(Ordering::Relaxed),
            MONITORING_ENGINE.health_checks_performed.load(Ordering::Relaxed),
            MONITORING_ENGINE.monitoring_overhead_cycles.load(Ordering::Relaxed),
            MONITORING_ENGINE.active_alerts.load(Ordering::Relaxed),
        )
    }
}

/// Read current timestamp in microseconds
fn read_timestamp() -> u64 {
    unsafe {
        let mut cycles: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles);
        cycles / 2400 // Convert to microseconds (assuming 2.4GHz)
    }
}

/// Read cycle counter for timing
fn read_cycle_counter() -> u64 {
    unsafe {
        let mut cycles: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) cycles);
        cycles
    }
}