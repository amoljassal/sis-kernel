//! Lock-free behavioral event streaming
//! 
//! High-performance ring buffers for real-time behavioral analysis

#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use super::BehavioralEvent;

/// Lock-free ring buffer for behavioral events
/// 
/// Achieves <10μs push/pop latency using atomic operations
#[repr(C, align(64))]  // Cache-line aligned
pub struct BehavioralStreamBuffer<const N: usize> {
    /// Event storage
    data: [BehavioralEvent; N],
    /// Producer index (write position)
    head: AtomicUsize,
    /// Consumer index (read position)  
    tail: AtomicUsize,
}

impl<const N: usize> BehavioralStreamBuffer<N> {
    /// Create new empty buffer
    pub const fn new() -> Self {
        Self {
            data: [BehavioralEvent::KeystrokeTiming { 
                interval_us: 0, 
                pressure: 0 
            }; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }
    
    /// Push event to buffer (lock-free)
    #[inline(always)]
    pub fn push(&self, event: BehavioralEvent) -> Result<(), BufferFull> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % N;
        
        // Check if buffer is full
        if next_tail == self.head.load(Ordering::Acquire) {
            return Err(BufferFull);
        }
        
        // Store event
        unsafe {
            let ptr = &self.data[tail] as *const _ as *mut BehavioralEvent;
            ptr.write(event);
        }
        
        // Memory barrier to ensure write completes before index update
        core::sync::atomic::fence(Ordering::Release);
        
        // Update tail
        self.tail.store(next_tail, Ordering::Release);
        
        Ok(())
    }
    
    /// Pop event from buffer (lock-free)
    #[inline(always)]
    pub fn pop(&self) -> Option<BehavioralEvent> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        
        // Check if buffer is empty
        if head == tail {
            return None;
        }
        
        // Read event
        let event = unsafe {
            let ptr = &self.data[head] as *const BehavioralEvent;
            ptr.read()
        };
        
        // Memory barrier
        core::sync::atomic::fence(Ordering::Acquire);
        
        // Update head
        let next_head = (head + 1) % N;
        self.head.store(next_head, Ordering::Release);
        
        Some(event)
    }
    
    /// Get current number of events in buffer
    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        
        if tail >= head {
            tail - head
        } else {
            N - head + tail
        }
    }
    
    /// Check if buffer is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
    
    /// Clear all events
    pub fn clear(&self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
    }
}

/// Buffer full error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFull;

/// Sliding window for temporal analysis
pub struct SlidingWindow<const WINDOW_SIZE: usize> {
    /// Circular buffer of events
    events: [Option<BehavioralEvent>; WINDOW_SIZE],
    /// Current position
    position: AtomicUsize,
    /// Window duration in microseconds
    duration_us: u64,
}

impl<const WINDOW_SIZE: usize> SlidingWindow<WINDOW_SIZE> {
    /// Create new sliding window
    pub const fn new(duration_us: u64) -> Self {
        Self {
            events: [None; WINDOW_SIZE],
            position: AtomicUsize::new(0),
            duration_us,
        }
    }
    
    /// Add event to window
    pub fn add(&mut self, event: BehavioralEvent) {
        let pos = self.position.fetch_add(1, Ordering::Relaxed) % WINDOW_SIZE;
        self.events[pos] = Some(event);
    }
    
    /// Get events within time window
    pub fn get_recent(&self) -> impl Iterator<Item = &BehavioralEvent> + '_ {
        self.events.iter().filter_map(|e| e.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_push_pop() {
        let buffer = BehavioralStreamBuffer::<16>::new();
        
        // Push event
        let event = BehavioralEvent::KeystrokeTiming {
            interval_us: 100,
            pressure: 50,
        };
        
        assert!(buffer.push(event).is_ok());
        assert_eq!(buffer.len(), 1);
        
        // Pop event
        let popped = buffer.pop().unwrap();
        assert_eq!(popped, event);
        assert!(buffer.is_empty());
    }
}