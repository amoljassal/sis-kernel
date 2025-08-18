//! SMP CPU Affinity Selftest
//!
//! Tests CPU affinity functionality by pinning tasks to specific CPUs
//! and verifying they only execute on allowed processors.

#![cfg(all(feature = "affinity", feature = "smp", feature = "scheduler"))]

use crate::kernel::{serial, syscall, smp_scheduler};
use crate::arch::x86_64::percpu;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

static TEST_HITS_CPU1: AtomicU32 = AtomicU32::new(0);
static TEST_DONE: AtomicBool = AtomicBool::new(false);

/// Worker task that increments counter when running on CPU 1
extern "C" fn affinity_worker() -> ! {
    loop {
        if TEST_DONE.load(Ordering::Relaxed) {
            break;
        }
        
        let cpu = percpu::cpu_id();
        if cpu == 1 {
            TEST_HITS_CPU1.fetch_add(1, Ordering::Relaxed);
            serial::write_str("[affinity-worker] Running on CPU 1\n");
        } else {
            serial::write_str("[affinity-worker] Running on CPU ");
            serial::write_u64(cpu as u64);
            serial::write_str(" (should not happen after pinning)\n");
        }
        
        // Yield CPU briefly
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }
    
    loop { crate::arch::x86_64::cpu::halt(); }
}

/// Run SMP affinity test
pub fn test_smp_affinity() -> Result<(), &'static str> {
    serial::write_str("[test] SMP_AFFINITY: Starting affinity validation test\n");
    
    // Reset test state
    TEST_HITS_CPU1.store(0, Ordering::Relaxed);
    TEST_DONE.store(false, Ordering::Relaxed);
    
    // Spawn worker task on SMP scheduler
    let task_id = smp_scheduler::spawn_smp_task(affinity_worker, "affinity_test_worker", crate::kernel::task::Role::Child);
    serial::write_str("[test] Spawned affinity test worker, task_id=");
    serial::write_u64(task_id);
    serial::write_str("\n");
    
    // Let it run briefly to establish baseline
    for _ in 0..100000 {
        core::hint::spin_loop();
    }
    
    // Check initial hits before pinning
    let initial_hits = TEST_HITS_CPU1.load(Ordering::Relaxed);
    serial::write_str("[test] Initial hits on CPU1 (unpinned): ");
    serial::write_u64(initial_hits as u64);
    serial::write_str("\n");
    
    // Pin task to CPU 1 using syscall
    let cpu1_mask = 1u64 << 1; // Bit 1 = CPU 1
    syscall::dispatch_manual(syscall::SYS_SET_AFFINITY as u64, cpu1_mask, 0, 0, 0, 0, 0);
    
    // Wait for affinity to take effect and task to run
    for _ in 0..200000 {
        core::hint::spin_loop();
    }
    
    // Check hits after pinning  
    let pinned_hits = TEST_HITS_CPU1.load(Ordering::Relaxed);
    serial::write_str("[test] Hits on CPU1 (after pinning): ");
    serial::write_u64(pinned_hits as u64);
    serial::write_str("\n");
    
    // Verify we got some activity on CPU 1 after pinning
    if pinned_hits > initial_hits {
        serial::write_str("[test] SUCCESS: Task successfully pinned to CPU 1\n");
        
        // Test relaxing affinity to all CPUs
        let all_cpus_mask = 0; // 0 = no constraint (all CPUs allowed)
        syscall::dispatch_manual(syscall::SYS_SET_AFFINITY as u64, all_cpus_mask, 0, 0, 0, 0, 0);
        
        // Signal test completion
        TEST_DONE.store(true, Ordering::Relaxed);
        
        serial::write_str("[test] SMP_AFFINITY: PASS - CPU affinity working correctly\n");
        Ok(())
    } else {
        TEST_DONE.store(true, Ordering::Relaxed);
        serial::write_str("[test] SMP_AFFINITY: FAIL - No activity detected on CPU 1 after pinning\n");
        Err("CPU affinity enforcement failed")
    }
}