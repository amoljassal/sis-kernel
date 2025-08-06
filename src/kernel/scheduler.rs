//! Task scheduler.
//!
//! Implements a simple round‑robin scheduler with support for
//! parent tasks and priorities.  The scheduler maintains a queue of
//! tasks and selects the next runnable task on each timer tick.  A
//! context switch saves the current task's context and restores
//! the next task's context.

use alloc::collections::VecDeque;
use spin::Mutex;
use crate::kernel::{serial, task::{Task, State}};

// Extern declaration of the context switch routine implemented in
// `arch/x86_64/context_switch.rs`.  See that file for details.
extern "C" {
    fn switch_context(old: *mut super::task::TaskContext, new: *const super::task::TaskContext);
}

lazy_static::lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler {
    tasks: VecDeque<&'static mut Task>,
    current: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler { tasks: VecDeque::new(), current: 0 }
    }

    pub fn add_task(&mut self, task: &'static mut Task) {
        self.tasks.push_back(task);
    }

    /// Remove a task by its ID.  Returns true if the task was found
    /// and removed.  After removal the current index is adjusted if
    /// necessary.
    pub fn remove_task(&mut self, id: usize) -> bool {
        if self.tasks.is_empty() { return false; }
        let pos = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = pos {
            self.tasks.remove(idx);
            if self.current >= self.tasks.len() {
                self.current = 0;
            }
            return true;
        }
        false
    }

    /// Spawn a child task by creating it and adding it to the end of
    /// the queue.  Returns the ID of the new task.
    pub fn spawn_child(&mut self, entry: fn(), parent_role: super::task::Role) -> usize {
        let task = super::task::Task::spawn(entry, parent_role);
        let id = task.id;
        self.add_task(task);
        id
    }

    /// Select the next task in a round‑robin fashion.  Returns
    /// None if no tasks are available.
    fn next_task_index(&self) -> Option<usize> {
        if self.tasks.is_empty() { return None; }
        Some((self.current + 1) % self.tasks.len())
    }

    /// Perform a scheduler tick.  Saves the current task's context
    /// (not yet implemented) and switches to the next task.  For
    /// demonstration the function only logs which task would run.
    pub fn tick(&mut self) {
        if self.tasks.is_empty() { return; }
        // Benchmark start time
        let start_tsc = crate::arch::x86_64::cpu::rdtsc();
        let next_index = self.next_task_index().unwrap();
        let current_task = self.tasks[self.current];
        let next_task = self.tasks[next_index];
        // In a full implementation we would save the CPU context of
        // `current_task.context` and restore `next_task.context` via
        // assembly.  That logic is omitted here for clarity.
        // Skip tasks that are blocked or terminated
        let mut target_index = next_index;
        for _ in 0..self.tasks.len() {
            let t = self.tasks[target_index];
            if matches!(t.state, State::Ready | State::Running) {
                break;
            }
            target_index = (target_index + 1) % self.tasks.len();
        }
        // Perform a context switch if the next task is different
        if target_index != self.current {
            let current_ptr: *mut super::task::TaskContext = &mut current_task.context;
            let next_ptr: *const super::task::TaskContext = &next_task.context;
            // Log the switch for debugging
            serial::write_str("[scheduler] Switching from ");
            serial::write_str(current_task.name);
            serial::write_str(" to ");
            serial::write_str(next_task.name);
            serial::write_str("\n");
            unsafe {
                switch_context(current_ptr, next_ptr);
            }
            // Set core and GPU affinity for the next task
            let _ = super::affinity::set_core_affinity(next_task.affinity_core);
            if let Some(gpu) = next_task.affinity_gpu {
                let _ = super::affinity::set_gpu_affinity(gpu);
            }
            self.current = target_index;
            // Benchmark end time and log delta cycles
            let end_tsc = crate::arch::x86_64::cpu::rdtsc();
            let delta = end_tsc - start_tsc;
            // Print cycles difference in decimal
            serial::write_str("[benchmark] switch cycles: ");
            {
                let mut buf = [0u8; 20];
                let mut i = buf.len();
                let mut n = delta;
                if n == 0 { i -= 1; buf[i] = b'0'; }
                while n > 0 {
                    i -= 1;
                    buf[i] = b'0' + (n % 10) as u8;
                    n /= 10;
                }
                serial::write_buf(&buf[i..]);
            }
            serial::write_str("\n");
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::kernel::task::{Task, Role};

    // Dummy entry for spawned tasks
    fn entry() {}

    #[test]
    fn scheduler_adds_and_removes_tasks() {
        let mut sched = Scheduler::new();
        let t1 = Task::new(Role::Philosophy, entry);
        let t2 = Task::new(Role::Technical, entry);
        sched.add_task(t1);
        sched.add_task(t2);
        assert_eq!(sched.tasks.len(), 2);
        // Remove first task
        assert!(sched.remove_task(t1.id));
        assert_eq!(sched.tasks.len(), 1);
        // Clean up
        unsafe { Task::free(t1 as *mut Task) };
        unsafe { Task::free(t2 as *mut Task) };
    }
}

/// Initialise the scheduler.  Clears any existing tasks.
pub fn init() {
    let mut sched = SCHEDULER.lock();
    *sched = Scheduler::new();
}

/// Add a parent task to the front of the scheduling queue.  Parent
/// tasks have priority over child tasks and are inserted at the
/// beginning of the queue.
pub fn add_parent(task: &'static mut Task) {
    let mut sched = SCHEDULER.lock();
    sched.tasks.push_front(task);
}

/// Add a regular task to the scheduler (append at the end).
pub fn add_task(task: &'static mut Task) {
    let mut sched = SCHEDULER.lock();
    sched.add_task(task);
}

/// Called by the timer interrupt handler to advance the scheduler.
pub fn tick() {
    let mut sched = SCHEDULER.lock();
    sched.tick();
}

/// Spawn a child task on behalf of a user space request.  This
/// function acquires the scheduler lock, spawns the task and
/// returns the new task's ID.  The task is appended to the end of
/// the scheduling queue.
pub fn spawn_child(entry: fn(), parent_role: super::task::Role) -> usize {
    let mut sched = SCHEDULER.lock();
    sched.spawn_child(entry, parent_role)
}

/// Terminate the current task.  This removes it from the queue and
/// frees its resources.  After termination the scheduler will
/// immediately switch to the next available task on the next tick.
pub fn terminate_current() {
    let mut sched = SCHEDULER.lock();
    if sched.tasks.is_empty() { return; }
    let current_task = sched.tasks[sched.current];
    // Mark as terminated
    current_task.state = super::task::State::Terminated;
    let id = current_task.id;
    // Remove from queue
    sched.remove_task(id);
    // Free memory
    unsafe { super::task::Task::free(current_task as *mut super::task::Task) };
}