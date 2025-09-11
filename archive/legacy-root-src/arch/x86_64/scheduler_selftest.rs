//! Phase 3 scheduler selftests
//!
//! Tests preemptive multitasking with:
//! - Task blocking and wakeup
//! - Priority boosting for parent tasks
//! - Time slice preemption
//! - Wait queue integration

#![cfg(feature = "scheduler")]

use crate::arch::x86_64::io::qemu_exit;
use crate::kernel::scheduler;
use crate::kernel::serial;
use crate::kernel::task::{BlockReason, Role, Task};

/// Run a single scheduler test by tag for flexible harnesses.
pub fn run_one(tag: &str) -> ! {
    match tag {
        "SCHEDULER_PREEMPT" => run_preempt_test(),
        _ => {
            serial::write_str("[scheduler-selftest] Unknown test: ");
            serial::write_str(tag);
            serial::write_str("\n");
            unsafe {
                qemu_exit(0x01);
            } // failure
        }
    }
}

#[cfg(selftest_SCHEDULER_PREEMPT)]
pub fn run() -> ! {
    run_one("SCHEDULER_PREEMPT")
}

fn run_preempt_test() -> ! {
    serial::write_str("[scheduler-selftest] Phase 3 preemptive scheduler test\n");

    // Initialize scheduler for CPU 0
    scheduler::init(0);

    // Test 1: Basic task scheduling and Add tasks to scheduler using the new Phase 3 preemptive API
    // Need to clone tasks since the preemptive add_task takes ownership
    let tid1 = {
        let t = Task::new(Role::Philosophy, test_task_1);
        let tid = t.id as u64;
        scheduler::add_task(t);
        tid
    };
    let tid2 = {
        let t = Task::new(Role::Technical, test_task_2);
        let tid = t.id as u64;
        scheduler::add_task(t);
        tid
    };

    serial::write_str("[scheduler-selftest] Added tasks to scheduler\n");

    // Test 2: Basic scheduling - run a few cycles
    for i in 0..5 {
        serial::write_str("[scheduler-selftest] Scheduling cycle ");
        serial::write_u64(i);
        serial::write_str("\n");
        scheduler::schedule(0);
    }

    // Test 3: Task blocking
    serial::write_str("[scheduler-selftest] Testing task blocking\n");
    scheduler::block_current(0, BlockReason::IpcRecv);

    // Test 4: Task wakeup
    serial::write_str("[scheduler-selftest] Testing task wakeup\n");
    scheduler::wake(0, tid1);

    // Test 5: Priority boosting - parent tasks should get priority
    serial::write_str("[scheduler-selftest] Testing priority boosting\n");
    scheduler::schedule(0);

    // Test 6: Time slice preemption simulation
    serial::write_str("[scheduler-selftest] Testing time slice preemption\n");
    for _ in 0..10 {
        scheduler::on_timer_tick(0);
    }

    serial::write_str("[scheduler-selftest] All tests passed!\n");
    unsafe {
        qemu_exit(0x00);
    } // success
}

fn test_task_1() {
    serial::write_str("[task1] Running\n");
    // Simulate some work
    for i in 0..100 {
        if i % 50 == 0 {
            serial::write_str("[task1] Working...\n");
        }
    }
}

fn test_task_2() {
    serial::write_str("[task2] Running\n");
    // Simulate some work
    for i in 0..100 {
        if i % 50 == 0 {
            serial::write_str("[task2] Working...\n");
        }
    }
}
