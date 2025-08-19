//! Task table management for Phase 6B patch compatibility

use crate::kernel::task::Task;
use alloc::sync::Arc;
use spin::Mutex;

// Simplified task storage for Phase 6B patch
static mut TASKS: [Option<Arc<Mutex<Task>>>; 256] = [const { None }; 256];

pub struct TaskRef {
    inner: Arc<Mutex<Task>>,
}

impl TaskRef {
    pub fn lock(&self) -> spin::MutexGuard<Task> {
        self.inner.lock()
    }

    pub fn try_lock_for_enqueue(&self) -> Option<spin::MutexGuard<Task>> {
        self.inner.try_lock()
    }
}

/// Get task by TID (simplified for Phase 6B patch)
pub fn get(tid: u64) -> TaskRef {
    unsafe {
        // For simplicity, use TID as direct index (in real implementation would hash)
        let idx = (tid as usize) % 256;
        if let Some(task_ref) = &TASKS[idx] {
            TaskRef {
                inner: task_ref.clone(),
            }
        } else {
            // Create a dummy task if not found (for patch compatibility)
            let dummy_task = Task {
                id: tid as usize,
                name: "dummy",
                role: crate::kernel::task::Role::Child,
                stack: alloc::vec![0u8; 4096].into_boxed_slice(),
                context: crate::kernel::task::TaskContext {
                    r15: 0,
                    r14: 0,
                    r13: 0,
                    r12: 0,
                    rbx: 0,
                    rbp: 0,
                    rip: 0,
                    rsp: 0,
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
                cpu_affinity_mask: 0,
                vdso: None,
            };
            let task_ref = Arc::new(Mutex::new(dummy_task));
            TASKS[idx] = Some(task_ref.clone());
            TaskRef { inner: task_ref }
        }
    }
}

/// Store task in table
pub fn store(tid: u64, task: Task) {
    unsafe {
        let idx = (tid as usize) % 256;
        TASKS[idx] = Some(Arc::new(Mutex::new(task)));
    }
}
