// SIS Kernel Test Runner Binary
// Main entry point for comprehensive test suite execution

use sis_testing::{SISTestSuite, TestSuiteConfig, setup_logging};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    
    log::info!("SIS Kernel Industry-Grade Test Suite");
    log::info!("====================================");
    
    let config = if env::args().any(|arg| arg == "--quick") {
        TestSuiteConfig {
            qemu_nodes: 3,
            test_duration_secs: 300,
            performance_iterations: 1000,
            statistical_confidence: 0.95,
            output_directory: "target/testing".to_string(),
            generate_reports: true,
            parallel_execution: true,
        }
    } else {
        TestSuiteConfig::default()
    };
    
    log::info!("Test Configuration:");
    log::info!("  QEMU Nodes: {}", config.qemu_nodes);
    log::info!("  Duration: {}s", config.test_duration_secs);
    log::info!("  Performance Iterations: {}", config.performance_iterations);
    log::info!("  Statistical Confidence: {:.1}%", config.statistical_confidence * 100.0);
    log::info!("  Output Directory: {}", config.output_directory);
    log::info!("  Parallel Execution: {}", config.parallel_execution);
    
    let test_suite = SISTestSuite::new(config);
    
    match test_suite.execute_comprehensive_validation().await {
        Ok(report) => {
            log::info!("");
            log::info!("=== VALIDATION COMPLETE ===");
            log::info!("Overall Score: {:.1}%", report.overall_score);
            log::info!("Performance Coverage: {:.1}%", report.test_coverage.performance_coverage * 100.0);
            log::info!("Correctness Coverage: {:.1}%", report.test_coverage.correctness_coverage * 100.0);
            log::info!("Security Coverage: {:.1}%", report.test_coverage.security_coverage * 100.0);
            log::info!("Distributed Coverage: {:.1}%", report.test_coverage.distributed_coverage * 100.0);
            log::info!("AI Coverage: {:.1}%", report.test_coverage.ai_coverage * 100.0);
            log::info!("");
            
            log::info!("Validation Results:");
            for result in &report.results {
                let status = if result.passed { "PASS" } else { "FAIL" };
                log::info!("  [{}] {} ({})", status, result.claim, result.measured);
            }
            
            log::info!("");
            log::info!("Reports generated in: target/testing/");
            log::info!("View dashboard: target/testing/dashboard.html");
            
            if report.overall_score >= 90.0 {
                log::info!("SUCCESS: SIS Kernel meets industry standards for production deployment!");
                std::process::exit(0);
            } else {
                log::warn!("WARNING: SIS Kernel requires improvements before production readiness");
                std::process::exit(1);
            }
        }
        Err(e) => {
            log::error!("Validation failed: {}", e);
            std::process::exit(1);
        }
    }
}