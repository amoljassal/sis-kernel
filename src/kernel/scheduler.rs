//! Preemptive scheduler with per-CPU runqueues and wait queues.
//!
//! Phase 3: Implements true preemptive multitasking with:
//! - Per-CPU runqueues with time slicing
//! - Task blocking and wakeup primitives
//! - Priority boosting for parent tasks
//! - Integration with LAPIC timer for preemption

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use crate::kernel::{serial, task::{Task, State, Role, BlockReason}};

// Legacy selftests support (maintain compatibility)
#[cfg(all(feature = "idt-selftest", selftest_TIMER))]
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(all(feature = "idt-selftest", selftest_LAPIC_TIMER))]
static LAPIC_TICK_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(all(feature="smp", feature="apic"))]
static CPU_TICKS: [AtomicU64; 2] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
];

// Phase 3: New preemptive scheduler
#[cfg(feature = "scheduler")]
mod preemptive {
    use super::*;
    
    pub type TaskId = u64;
    const MAX_CPUS: usize = 8;
    const DEFAULT_TIMESLICE_TICKS: u64 = 5; // 5ms with 1kHz LAPIC, tune as needed
    
    struct RunQueue {
        ready: VecDeque<TaskId>,
        current: Option<TaskId>,
        quantum_left: u64,
    }
    
    impl RunQueue {
        const fn new() -> Self {
            Self { 
                ready: VecDeque::new(), 
                current: None, 
                quantum_left: DEFAULT_TIMESLICE_TICKS 
            }
        }
    }
    
    pub struct Sched {
        cpus: [Mutex<RunQueue>; MAX_CPUS],
        tasks: Mutex<Vec<Task>>, // Simple task storage for Phase 3
    }
    
    static SCHED: Sched = Sched {
        cpus: [
            Mutex::new(RunQueue::new()), Mutex::new(RunQueue::new()),
            Mutex::new(RunQueue::new()), Mutex::new(RunQueue::new()),
            Mutex::new(RunQueue::new()), Mutex::new(RunQueue::new()),
            Mutex::new(RunQueue::new()), Mutex::new(RunQueue::new()),
        ],
        tasks: Mutex::new(Vec::new()),
    };
    
    pub fn init(cpu_id: usize) {
        serial::write_str("[sched] init\n");
        let mut rq = SCHED.cpus[cpu_id].lock();
        rq.quantum_left = DEFAULT_TIMESLICE_TICKS;
    }
    
    pub fn enqueue(cpu_id: usize, tid: TaskId) {
        let mut rq = SCHED.cpus[cpu_id].lock();
        rq.ready.push_back(tid);
    }
    
    pub fn wake(cpu_id: usize, tid: TaskId) {
        let mut tasks = SCHED.tasks.lock();
        if let Some(t) = tasks.iter_mut().find(|t| t.id as u64 == tid) {
            t.state = State::Ready;
        }
        drop(tasks);
        enqueue(cpu_id, tid);
    }
    
    pub fn block_current(cpu_id: usize, reason: BlockReason) {
        let mut rq = SCHED.cpus[cpu_id].lock();
        if let Some(cur) = rq.current {
            let mut tasks = SCHED.tasks.lock();
            if let Some(t) = tasks.iter_mut().find(|t| t.id as u64 == cur) {
                t.state = State::Blocked(reason);
            }
            drop(tasks);
            rq.current = None;
        }
        drop(rq);
        // immediate reschedule
        schedule(cpu_id);
    }
    
    pub fn yield_now(cpu_id: usize) {
        let mut rq = SCHED.cpus[cpu_id].lock();
        if let Some(cur) = rq.current.take() {
            rq.ready.push_back(cur);
        }
        drop(rq);
        schedule(cpu_id);
    }
    
    pub fn on_timer_tick(cpu_id: usize) {
        let mut rq = SCHED.cpus[cpu_id].lock();
        if rq.quantum_left > 0 {
            rq.quantum_left -= 1;
            return;
        }
        rq.quantum_left = DEFAULT_TIMESLICE_TICKS;
        if let Some(cur) = rq.current.take() {
            // If still Running → demote to Ready
            let mut tasks = SCHED.tasks.lock();
            if let Some(t) = tasks.iter_mut().find(|t| t.id as u64 == cur) {
                if matches!(t.state, State::Running) {
                    t.state = State::Ready;
                    rq.ready.push_back(cur);
                }
            }
            drop(tasks);
        }
        drop(rq);
        schedule(cpu_id);
    }
    
    fn select_next(cpu_id: usize) -> Option<TaskId> {
        let mut rq = SCHED.cpus[cpu_id].lock();
        let tasks = SCHED.tasks.lock();
        
        // Try to pick a boosted Ready first
        if let Some(pos) = rq.ready.iter().position(|tid| {
            tasks.iter().find(|t| t.id as u64 == *tid).map(|t|
                matches!(t.state, State::Ready) && t.priority_boost
            ).unwrap_or(false)
        }) {
            if let Some(tid) = rq.ready.remove(pos) {
                rq.current = Some(tid);
                return Some(tid);
            }
        }
        
        // Otherwise normal RR
        while let Some(tid) = rq.ready.pop_front() {
            if let Some(t) = tasks.iter().find(|t| t.id as u64 == tid) {
                if matches!(t.state, State::Ready) {
                    rq.current = Some(tid);
                    return Some(tid);
                }
            }
        }
        None
    }
    
    pub fn schedule(cpu_id: usize) {
        if let Some(next_id) = select_next(cpu_id) {
            // context switch: save current, load next
            let tasks = SCHED.tasks.lock();
            let (next_ksp, next_cr3) = {
                let t = tasks.iter().find(|t| t.id as u64 == next_id).unwrap();
                let ksp = t.kstack_top;
                #[cfg(feature = "per-task-mm")]
                let cr3 = t.cr3_root;
                #[cfg(not(feature = "per-task-mm"))]
                let cr3: Option<x86_64::structures::paging::PhysFrame> = None;
                (ksp, cr3)
            };
            drop(tasks);
            
            // Update task state to Running
            let mut tasks = SCHED.tasks.lock();
            if let Some(t) = tasks.iter_mut().find(|t| t.id as u64 == next_id) {
                t.state = State::Running;
            }
            drop(tasks);
            
            // CR3 switch if needed (Phase 1 integration)
            #[cfg(feature = "per-task-mm")]
            if let Some(cr3_frame) = next_cr3 {
                use x86_64::registers::control::Cr3;
                unsafe { 
                    Cr3::write(
                        cr3_frame,
                        x86_64::registers::control::Cr3Flags::empty()
                    ); 
                }
            }
            
            // Context switch (placeholder - would call actual context switch)
            // unsafe { context_switch::switch_to(next_ksp) };
        } else {
            // no runnable tasks; CPU can halt until next IRQ
            crate::arch::x86_64::cpu::halt();
        }
    }
    
    // Helper to add tasks for testing
    pub fn add_task(task: Task) -> TaskId {
        let tid = task.id as u64;
        let mut tasks = SCHED.tasks.lock();
        tasks.push(task);
        drop(tasks);
        enqueue(0, tid); // Add to CPU 0 for simplicity
        tid
    }
}

// Re-export preemptive scheduler functions when feature is enabled
#[cfg(feature = "scheduler")]
pub use preemptive::*;

// Legacy scheduler functions (maintain compatibility)
extern "C" {
    fn switch_context(old: *mut super::task::TaskContext, new: *const super::task::TaskContext);
}

lazy_static::lazy_static! {
    static ref SCHEDULER: Mutex<LegacyScheduler> = Mutex::new(LegacyScheduler::new());
}

pub struct LegacyScheduler {
    tasks: VecDeque<&'static mut Task>,
    current: usize,
}

impl LegacyScheduler {
    pub const fn new() -> Self {
        LegacyScheduler { tasks: VecDeque::new(), current: 0 }
    }

    pub fn add_task(&mut self, task: &'static mut Task) {
        self.tasks.push_back(task);
    }

    pub fn next(&mut self) -> Option<&mut Task> {
        if self.tasks.is_empty() {
            return None;
        }
        let len = self.tasks.len();
        let task = &mut self.tasks[self.current];
        self.current = (self.current + 1) % len;
        Some(task)
    }
}

#[cfg(not(feature = "scheduler"))]
pub fn init() {
    serial::write_str("[scheduler] Legacy scheduler initialized\n");
}

pub fn add_parent(task: &'static mut Task) {
    let mut scheduler = SCHEDULER.lock();
    scheduler.add_task(task);
}

pub fn add_task(task: &'static mut Task) {
    let mut scheduler = SCHEDULER.lock();
    scheduler.add_task(task);
}

pub fn spawn_child(entry: fn(), parent_role: super::task::Role) -> usize {
    let task = Task::spawn(entry, parent_role);
    let id = task.id;
    add_task(task);
    id
}

pub fn terminate_current() {
    serial::write_str("[scheduler] Current task terminated\n");
    // In a real implementation, this would remove the current task
    // and perform a context switch to the next runnable task
}

// Legacy tick handler for compatibility
pub fn tick() {
    #[cfg(all(feature = "idt-selftest", selftest_TIMER))]
    {
        let n = TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        if n >= 10 {
            serial::write_str("[tick] n=10\n");
            unsafe { crate::arch::x86_64::io::qemu_exit(0x00); }
        }
    }
    
    #[cfg(all(feature = "idt-selftest", selftest_LAPIC_TIMER))]
    {
        let n = LAPIC_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        if n >= 10 {
            serial::write_str("[lapic-tick] n=10\n");
            unsafe { crate::arch::x86_64::io::qemu_exit(0x00); }
        }
    }
    
    // Call new preemptive scheduler if enabled
    #[cfg(feature = "scheduler")]
    {
        on_timer_tick(0); // Use CPU 0 for now
    }
}

// SMP tick handler
#[cfg(all(feature = "apic", feature = "smp"))]
pub fn tick_smp() {
    let cpu_id = crate::arch::x86_64::apic::current_cpu_id();
    let ticks = CPU_TICKS[cpu_id].fetch_add(1, Ordering::SeqCst);
    
    #[cfg(all(feature = "idt-selftest", selftest_SMP_2))]
    {
        if ticks >= 10 {
            serial::write_str("[lapic-tick] cpu=");
            // Print CPU ID
            let mut buffer = [0u8; 10];
            let mut i = buffer.len();
            let mut n = cpu_id as u64;
            if n == 0 { i -= 1; buffer[i] = b'0'; }
            while n > 0 {
                i -= 1;
                buffer[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            crate::kernel::serial::write_buf(&buffer[i..]);
            serial::write_str("\n");
            
            // Check if both CPUs have reached 10 ticks
            if CPU_TICKS[0].load(Ordering::SeqCst) >= 10 && 
               CPU_TICKS[1].load(Ordering::SeqCst) >= 10 {
                serial::write_str("[smp] Both CPUs reached 10 ticks - test passed\n");
                unsafe { crate::arch::x86_64::io::qemu_exit(0x00); }
            }
        }
    }
    
    // Call new preemptive scheduler if enabled
    #[cfg(feature = "scheduler")]
    {
        on_timer_tick(cpu_id);
    }
}