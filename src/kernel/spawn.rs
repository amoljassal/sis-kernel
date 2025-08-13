//! Task spawning for Phase 6B patch compatibility

#[cfg(feature = "scheduler")]
use crate::kernel::simple_scheduler;
use crate::kernel::{
    task::{Role, Task},
    task_table,
};
use core::sync::atomic::{AtomicU64, Ordering};

static NEXT_TID: AtomicU64 = AtomicU64::new(2);

/// Spawn kernel closure (for Phase 6B patch compatibility)
pub unsafe fn spawn_kernel_closure(entry_addr: usize) -> u64 {
    let tid = NEXT_TID.fetch_add(1, Ordering::SeqCst);

    // Create a simple task
    let task = Task {
        id: tid as usize,
        name: "affinity_worker",
        role: Role::Child,
        stack: alloc::vec![0u8; 16384].into_boxed_slice(), // 16KB stack
        context: crate::kernel::task::TaskContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: entry_addr as u64,
            rsp: 0, // Will be set properly
        },
        state: crate::kernel::task::State::Ready,
        priority: 1,
        affinity_core: 0,
        affinity_gpu: None,
        next: None,
        kstack_top: 0,
        cpu_hint: 0,
        #[cfg(feature = "per-task-mm")]
        cr3_root: None,
        #[cfg(feature = "per-task-mm")]
        user_stack_top: 0,
        #[cfg(feature = "per-task-mm")]
        guard_pages: (0, 0),
        #[cfg(feature = "per-task-mm")]
        mm_stats: crate::kernel::task::TaskMmStats::default(),
        #[cfg(feature = "ipc")]
        ctable: crate::kernel::caps::CTable::new(),
        #[cfg(feature = "scheduler")]
        priority_boost: false,
        #[cfg(feature = "affinity")]
        cpu_affinity_mask: 0, // No affinity constraint initially
    };

    // Set up stack pointer
    let stack_top = task.stack.as_ptr() as usize + task.stack.len();
    let mut task_with_stack = task;
    task_with_stack.context.rsp = stack_top as u64;
    task_with_stack.kstack_top = stack_top as u64;

    // Store in task table
    task_table::store(tid, task_with_stack);

    // Enqueue for scheduling
    let task_ref = task_table::get(tid);
    let task_guard = task_ref.lock();
    #[cfg(feature = "scheduler")]
    simple_scheduler::enqueue_task(&*task_guard);

    tid
}
