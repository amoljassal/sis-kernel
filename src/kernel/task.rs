//! Task representation and creation.
//!
//! A task encapsulates a unit of execution within the kernel.  It
//! contains a context (register state), a stack and metadata such as
//! its role and affinity.  The scheduler manages a collection of
//! tasks and performs context switches between them.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::NonNull;

/// Size of each task's stack in bytes.  Stacks are allocated on
/// demand and freed when the task terminates.
pub const STACK_SIZE: usize = 16 * 1024; // 16 KiB

/// The role of a task in the SIS kernel.  Parent tasks process
/// directives in their domain while child tasks are spawned by
/// parents to handle specific requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Philosophy,
    Technical,
    Child,
}

/// Possible states of a task during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// The CPU context saved during a context switch.  The order of
/// registers must match the assembly in `switch_context` and
/// `restore_context`.  We save callee‑saved registers and the
/// instruction pointer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rip: u64,
    pub rsp: u64,
}

/// A task in the SIS kernel.  Tasks are allocated on the heap and
/// referenced via `&'static mut` pointers by the scheduler.
pub struct Task {
    pub id: usize,
    pub name: &'static str,
    pub role: Role,
    pub stack: Box<[u8]>,
    pub context: TaskContext,
    pub state: State,
    pub priority: u8,
    pub affinity_core: usize,
    pub affinity_gpu: Option<usize>,
    pub next: Option<&'static mut Task>,
}

impl Task {
    /// Create a new task with the given role and entry function.
    /// The task is allocated on the heap and returns a static
    /// reference.  If allocation fails the kernel will panic via
    /// the global allocator error handler.
    pub fn new(role: Role, entry: fn()) -> &'static mut Task {
        // Allocate a stack for the task.
        let mut stack = vec![0u8; STACK_SIZE].into_boxed_slice();
        let stack_top = stack.as_ptr() as usize + STACK_SIZE;
        // Build the initial context: set the instruction pointer to
        // the entry function and the stack pointer to the top of the
        // stack.  Other callee‑saved registers are initialised to
        // zero.
        let context = TaskContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: entry as u64,
            rsp: stack_top as u64,
        };
        // Generate a simple ID.  In a real kernel this would be
        // generated atomically.
        static mut COUNTER: usize = 0;
        let id = unsafe { let id = COUNTER; COUNTER += 1; id };
        // Determine the affinity for the task.  Parent tasks
        // receive a fixed GPU assignment (GPU0 for Philosophy,
        // GPU1 for Technical).  Child tasks inherit their parent.
        let (priority, core, gpu) = match role {
            Role::Philosophy => (1, 0, Some(0)),
            Role::Technical  => (1, 0, Some(1)),
            Role::Child      => (1, 0, None),
        };
        let task = Task {
            id,
            name: match role {
                Role::Philosophy => "philosophy_parent",
                Role::Technical  => "technical_parent",
                Role::Child      => "child",
            },
            role,
            stack,
            context,
            state: State::Ready,
            priority,
            affinity_core: core,
            affinity_gpu: gpu,
            next: None,
        };
        // Leak the task onto the heap and return a static reference.
        Box::leak(Box::new(task))
    }

    /// Spawn a new child task dynamically.  This function creates a
    /// new `Task` in the `Child` role and returns a mutable reference
    /// that can be inserted into the scheduler.  Child tasks inherit
    /// the caller's affinity and priority.
    pub fn spawn(entry: fn(), parent_role: Role) -> &'static mut Task {
        let role = Role::Child;
        let mut task = Task::new(role, entry);
        // Inherit affinity from parent role
        match parent_role {
            Role::Philosophy => {
                task.affinity_core = 0;
                task.affinity_gpu = Some(0);
            }
            Role::Technical => {
                task.affinity_core = 0;
                task.affinity_gpu = Some(1);
            }
            Role::Child => {
                task.affinity_core = 0;
                task.affinity_gpu = None;
            }
        }
        task
    }

    /// Deallocate the task and its resources.  This function
    /// reconstructs the `Box<Task>` from the leaked pointer and drops
    /// it, freeing the stack and the task structure.  Call this
    /// only when the task is no longer referenced by the scheduler.
    pub unsafe fn free(task: *mut Task) {
        // Recreate the box so that Rust will drop the stack and the task.
        let _boxed: Box<Task> = Box::from_raw(task);
        // Dropping the box frees memory.
    }
}

#[cfg(test)]
mod tests {
    // To run these tests we need the standard library.  Enable it here.
    extern crate std;
    use super::*;

    #[test]
    fn create_parent_tasks_have_correct_role() {
        // Create a Philosophy parent task and verify its properties.
        let t = Task::new(Role::Philosophy, test_entry);
        assert_eq!(t.role, Role::Philosophy);
        assert_eq!(t.affinity_gpu, Some(0));
        assert_eq!(t.priority, 1);
        // Safety: free the leaked task to avoid memory leak in tests
        unsafe { Task::free(t as *mut Task) };
    }

    #[test]
    fn spawn_child_inherits_parent_affinity() {
        let child = Task::spawn(test_entry, Role::Technical);
        // Child should inherit GPU1 for technical
        assert_eq!(child.affinity_gpu, Some(1));
        unsafe { Task::free(child as *mut Task) };
    }

    fn test_entry() {
        // Dummy entry used for tasks created in tests.
        // Do nothing.
    }
}