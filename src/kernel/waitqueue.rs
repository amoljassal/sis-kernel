#![cfg(feature = "scheduler")]
use alloc::collections::VecDeque;
use spin::Mutex;

pub type TaskId = u64;

pub struct WaitQueue {
    q: Mutex<VecDeque<TaskId>>,
}

impl WaitQueue {
    pub const fn new() -> Self { 
        Self { q: Mutex::new(VecDeque::new()) } 
    }
    
    pub fn push(&self, tid: TaskId) { 
        self.q.lock().push_back(tid); 
    }
    
    pub fn pop(&self) -> Option<TaskId> { 
        self.q.lock().pop_front() 
    }
    
    pub fn is_empty(&self) -> bool { 
        self.q.lock().is_empty() 
    }
    
    pub fn wake_one<F: Fn(TaskId)>(&self, wake: F) -> bool {
        if let Some(tid) = self.pop() { 
            wake(tid); 
            true 
        } else { 
            false 
        }
    }
    
    pub fn wake_all<F: Fn(TaskId)>(&self, wake: F) -> usize {
        let mut n = 0;
        loop {
            if let Some(tid) = self.pop() { 
                wake(tid); 
                n += 1; 
            } else { 
                break; 
            }
        }
        n
    }
}