//! SMP boot integration for ARM64
//!
//! This module integrates all Phase 1 components (SMP, GICv3, PMU) into
//! the boot sequence, following the PYRAMID architectural principle of
//! building complex systems from simple, provable foundations.

use crate::arch::aarch64::{percpu, smp, gicv3, pmu};
use crate::kernel::serial;

/// Initialize SMP subsystem during kernel boot
pub fn init_smp() -> Result<(), &'static str> {
    serial::write_str("\n");
    serial::write_str("╔════════════════════════════════════════════════════════════╗\n");
    serial::write_str("║         SIS Kernel SMP Initialization (Phase 1)           ║\n");
    serial::write_str("╠════════════════════════════════════════════════════════════╣\n");
    
    // Step 1: Initialize per-CPU data for boot CPU
    serial::write_str("║ [1/5] Initializing per-CPU data structures...             ║\n");
    percpu::init_boot_cpu()?;
    
    // Step 2: Initialize GICv3 for interrupt routing
    serial::write_str("║ [2/5] Initializing GICv3 interrupt controller...          ║\n");
    gicv3::init()?;
    
    // Step 3: Initialize PMU for performance monitoring
    serial::write_str("║ [3/5] Initializing PMU performance monitoring...          ║\n");
    let pmu_info = pmu::Pmu::init()?;
    
    // Step 4: Initialize SMP and discover CPUs
    serial::write_str("║ [4/5] Discovering and initializing CPUs...                ║\n");
    smp::init_boot_cpu()?;
    
    // Step 5: Bring up secondary CPUs
    serial::write_str("║ [5/5] Bringing up secondary CPU cores...                  ║\n");
    let secondary_cpus = smp::bring_up_secondary_cpus()?;
    
    serial::write_str("╠════════════════════════════════════════════════════════════╣\n");
    serial::write_str("║ SMP Initialization Complete!                              ║\n");
    serial::write_str("║                                                            ║\n");
    
    // Report status
    serial::write_str("║ Status:                                                    ║\n");
    serial::write_str("║   • CPUs online: ");
    serial::write_u32(smp::online_cpu_count());
    serial::write_str(" / ");
    serial::write_u32(secondary_cpus + 1);
    serial::write_str("                                         ║\n");
    
    serial::write_str("║   • PMU counters: ");
    serial::write_u32(pmu_info.num_counters);
    serial::write_str(" available                              ║\n");
    
    serial::write_str("║   • GICv3: Ready for IPIs                                 ║\n");
    serial::write_str("║   • Performance target: <40μs AI inference                ║\n");
    serial::write_str("╚════════════════════════════════════════════════════════════╝\n\n");
    
    // Perform initial performance test
    test_smp_performance();
    
    Ok(())
}

/// Test SMP performance with PMU
fn test_smp_performance() {
    serial::write_str("[SMP TEST] Running performance validation...\n");
    
    // Start profiling
    let profile = pmu::AiPerfProfile::start();
    
    // Simulate AI workload
    let mut sum = 0u64;
    for i in 0..1000 {
        sum = sum.wrapping_add(i);
        // Simulate memory access pattern
        unsafe {
            core::arch::asm!("dmb ish", options(nomem, nostack, preserves_flags));
        }
    }
    
    // Stop profiling and get metrics
    let metrics = profile.stop();
    
    // Log results
    serial::write_str("[SMP TEST] Performance metrics:\n");
    metrics.log();
    
    // Test IPI functionality
    test_ipi_functionality();
}

/// Test Inter-Processor Interrupt functionality
fn test_ipi_functionality() {
    serial::write_str("[IPI TEST] Testing inter-processor interrupts...\n");
    
    let online_cpus = smp::online_cpu_count();
    if online_cpus > 1 {
        // Send test IPI to CPU 1
        gicv3::send_ipi(1, gicv3::ipi::IPI_RESCHEDULE);
        serial::write_str("[IPI TEST] Sent reschedule IPI to CPU 1\n");
        
        // Send AI task distribution IPI to all CPUs
        for cpu_id in 1..online_cpus {
            gicv3::send_ipi(cpu_id, gicv3::ipi::IPI_AI_TASK);
        }
        serial::write_str("[IPI TEST] Sent AI task IPIs to all secondary CPUs\n");
    } else {
        serial::write_str("[IPI TEST] Only one CPU online, IPI test skipped\n");
    }
}

/// Handle SMP-related interrupts
pub fn handle_smp_interrupt(intid: u32) {
    match intid {
        0..=15 => {
            // Software Generated Interrupt (IPI)
            handle_ipi(intid);
        }
        _ => {
            // Other interrupt types handled elsewhere
        }
    }
}

/// Handle Inter-Processor Interrupt
fn handle_ipi(ipi_type: u32) {
    let cpu_id = percpu::current_cpu_id();
    
    match ipi_type {
        gicv3::ipi::IPI_RESCHEDULE => {
            // Reschedule request
            serial::write_str("[CPU");
            serial::write_u32(cpu_id);
            serial::write_str("] Received reschedule IPI\n");
        }
        gicv3::ipi::IPI_CALL_FUNC => {
            // Function call request
            serial::write_str("[CPU");
            serial::write_u32(cpu_id);
            serial::write_str("] Received function call IPI\n");
        }
        gicv3::ipi::IPI_AI_TASK => {
            // AI task distribution
            serial::write_str("[CPU");
            serial::write_u32(cpu_id);
            serial::write_str("] Received AI task IPI\n");
            
            // Record in per-CPU stats
            let percpu = percpu::PerCpu::current();
            percpu.ai_stats.tasks_executed.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }
        _ => {
            // Unknown IPI type
            serial::write_str("[CPU");
            serial::write_u32(cpu_id);
            serial::write_str("] Received unknown IPI type: ");
            serial::write_u32(ipi_type);
            serial::write_str("\n");
        }
    }
    
    // Mark IPI as handled in per-CPU data
    let percpu = percpu::PerCpu::current();
    percpu.ipi_pending.fetch_and(!(1 << ipi_type), core::sync::atomic::Ordering::Release);
}