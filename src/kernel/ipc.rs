#![allow(dead_code)]
use super::caps::{CapEntry, CapFlags, CapKind, KernelObject};
use crate::kernel::{serial, task};
use alloc::{boxed::Box, collections::VecDeque, vec, vec::Vec};
use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};
use spin::Mutex;

#[cfg(feature = "scheduler")]
use crate::kernel::waitqueue::WaitQueue;

// errno helpers (negative values)
const EINVAL: i64 = -22;
const EPERM: i64 = -1;
const EAGAIN: i64 = -11;
const EFAULT: i64 = -14;
const ETIMED: i64 = -110;
const EPIPE: i64 = -32;

pub struct IpcChannel {
    q: Mutex<VecDeque<Box<[u8]>>>,
    msg_size: usize,
    max_msgs: usize,
    closed: AtomicBool,

    // Phase 3: wait queues for blocking operations
    #[cfg(feature = "scheduler")]
    send_waiters: WaitQueue, // blocked senders waiting for space
    #[cfg(feature = "scheduler")]
    recv_waiters: WaitQueue, // blocked receivers waiting for messages
}

impl KernelObject for IpcChannel {}

impl IpcChannel {
    fn new(max_msgs: usize, msg_size: usize) -> Self {
        Self {
            q: Mutex::new(VecDeque::with_capacity(max_msgs)),
            msg_size,
            max_msgs,
            closed: AtomicBool::new(false),

            // Phase 3: initialize wait queues
            #[cfg(feature = "scheduler")]
            send_waiters: WaitQueue::new(),
            #[cfg(feature = "scheduler")]
            recv_waiters: WaitQueue::new(),
        }
    }

    fn push(&self, msg: &[u8]) -> Result<(), i64> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(EPIPE);
        }
        if msg.len() > self.msg_size {
            return Err(EINVAL);
        }
        let mut q = self.q.lock();
        if q.len() >= self.max_msgs {
            return Err(EAGAIN);
        }
        let mut b = Vec::with_capacity(self.msg_size);
        b.extend_from_slice(msg);
        q.push_back(b.into_boxed_slice());
        Ok(())
    }

    fn pop_into(&self, dst: &mut [u8]) -> Result<usize, i64> {
        let mut q = self.q.lock();
        if let Some(m) = q.pop_front() {
            let n = core::cmp::min(m.len(), dst.len());
            dst[..n].copy_from_slice(&m[..n]);
            Ok(n)
        } else {
            Err(EAGAIN)
        }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);

        // Phase 3: wake all waiters when channel closes
        #[cfg(feature = "scheduler")]
        {
            self.send_waiters.wake_all(|tid| {
                crate::kernel::scheduler::wake(0, tid);
            });
            self.recv_waiters.wake_all(|tid| {
                crate::kernel::scheduler::wake(0, tid);
            });
        }
    }

    // Phase 3: blocking send that integrates with scheduler wait queues
    #[cfg(feature = "scheduler")]
    fn push_blocking(&self, msg: &[u8], current_tid: u64) -> Result<(), i64> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(EPIPE);
        }
        if msg.len() > self.msg_size {
            return Err(EINVAL);
        }

        let mut q = self.q.lock();
        if q.len() < self.max_msgs {
            // Space available, send immediately
            let mut b = Vec::with_capacity(self.msg_size);
            b.extend_from_slice(msg);
            q.push_back(b.into_boxed_slice());
            drop(q);

            // Wake one receiver if any are waiting
            self.recv_waiters.wake_one(|tid| {
                crate::kernel::scheduler::wake(0, tid);
            });

            Ok(())
        } else {
            // No space, block current task
            drop(q);
            self.send_waiters.push(current_tid);
            crate::kernel::scheduler::block_current(0, task::BlockReason::IpcSend);
            Err(EAGAIN) // This return won't be reached due to context switch
        }
    }

    // Phase 3: blocking recv that integrates with scheduler wait queues
    #[cfg(feature = "scheduler")]
    fn pop_blocking(&self, dst: &mut [u8], current_tid: u64) -> Result<usize, i64> {
        let mut q = self.q.lock();
        if let Some(m) = q.pop_front() {
            // Message available, receive immediately
            let n = core::cmp::min(m.len(), dst.len());
            dst[..n].copy_from_slice(&m[..n]);
            drop(q);

            // Wake one sender if any are waiting
            self.send_waiters.wake_one(|tid| {
                crate::kernel::scheduler::wake(0, tid);
            });

            Ok(n)
        } else if self.closed.load(Ordering::Relaxed) {
            // Channel closed and empty
            Err(EPIPE)
        } else {
            // No messages, block current task
            drop(q);
            self.recv_waiters.push(current_tid);
            crate::kernel::scheduler::block_current(0, task::BlockReason::IpcRecv);
            Err(EAGAIN) // This return won't be reached due to context switch
        }
    }
}

// ===== syscalls =====
// Note: These `unsafe` are boundary shims; they immediately convert
// user ptrs to slices via checked copies.

pub unsafe fn sys_chan_create(flags: u32, max_msgs: usize, msg_size: usize) -> Result<u32, i64> {
    if max_msgs == 0 || msg_size == 0 || max_msgs > 1024 || msg_size > 4096 {
        return Err(EINVAL);
    }
    let ch = IpcChannel::new(max_msgs, msg_size);
    let boxed = Box::new(ch);
    let obj_ptr = NonNull::from(Box::leak(boxed)) as NonNull<dyn KernelObject>;

    // For Phase 2 demo: create a fake current task with ctable
    // In production this would get the actual current task
    let current = create_demo_task();
    let sender = CapEntry {
        kind: CapKind::IpcSender,
        obj: obj_ptr,
        gen: 1,
        flags: CapFlags::from_bits_truncate(flags),
    };
    let receiver = CapEntry {
        kind: CapKind::IpcReceiver,
        obj: obj_ptr,
        gen: 1,
        flags: CapFlags::from_bits_truncate(flags),
    };
    let send_id = current.ctable.insert(sender);
    let _recv_id = current.ctable.insert(receiver);

    serial::write_str("[ipc] create channel ok\n");
    Ok(send_id)
}

pub unsafe fn sys_send(cap_id: u32, user_ptr: u64, len: usize) -> Result<usize, i64> {
    let task = create_demo_task();
    let cap = task.ctable.get(cap_id).ok_or(EPERM)?;
    if cap.kind != CapKind::IpcSender {
        return Err(EPERM);
    }
    let ch = cap.obj.cast::<IpcChannel>().as_ref();
    let buf = copy_in(user_ptr, len).map_err(|_| EFAULT)?;

    // Phase 3: use blocking send if scheduler feature enabled
    #[cfg(feature = "scheduler")]
    {
        let current_tid = task.id as u64;
        ch.push_blocking(&buf, current_tid)?;
    }
    #[cfg(not(feature = "scheduler"))]
    {
        ch.push(&buf)?;
    }

    Ok(len)
}

pub unsafe fn sys_recv(
    cap_id: u32,
    user_ptr: u64,
    len: usize,
    _timeout_us: u64,
) -> Result<usize, i64> {
    let task = create_demo_task();
    let cap = task.ctable.get(cap_id).ok_or(EPERM)?;
    if cap.kind != CapKind::IpcReceiver {
        return Err(EPERM);
    }
    let ch = cap.obj.cast::<IpcChannel>().as_ref();

    let mut tmp = vec![0u8; len];

    // Phase 3: use blocking recv if scheduler feature enabled
    #[cfg(feature = "scheduler")]
    {
        let current_tid = task.id as u64;
        match ch.pop_blocking(&mut tmp[..], current_tid) {
            Ok(n) => {
                copy_out(user_ptr, &tmp[..n]).map_err(|_| EFAULT)?;
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
    #[cfg(not(feature = "scheduler"))]
    {
        // Non-blocking v1: return EAGAIN when empty
        match ch.pop_into(&mut tmp[..]) {
            Ok(n) => {
                copy_out(user_ptr, &tmp[..n]).map_err(|_| EFAULT)?;
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}

pub unsafe fn sys_close(cap_id: u32) -> Result<(), i64> {
    let task = create_demo_task();
    let cap = task.ctable.get(cap_id).ok_or(EPERM)?;
    let ch = cap.obj.cast::<IpcChannel>().as_ref();
    ch.close();
    let _ = task.ctable.remove(cap_id);
    Ok(())
}

// ===== user copy helpers (v1: copy via kernel buffer) =====
fn copy_in(user_ptr: u64, len: usize) -> Result<Vec<u8>, ()> {
    if len == 0 || len > 4096 {
        return Err(());
    }
    // For v1 we assume user mapping is valid thanks to Phase 1 tests.
    // If fault occurs, page-fault handler returns deterministically (PFM discipline).
    let mut v = vec![0u8; len];
    unsafe {
        core::ptr::copy_nonoverlapping(user_ptr as *const u8, v.as_mut_ptr(), len);
    }
    Ok(v)
}

fn copy_out(user_ptr: u64, src: &[u8]) -> Result<(), ()> {
    if src.is_empty() || src.len() > 4096 {
        return Err(());
    }
    unsafe {
        core::ptr::copy_nonoverlapping(src.as_ptr(), user_ptr as *mut u8, src.len());
    }
    Ok(())
}

// Demo task creation for Phase 2 testing
// In production this would be replaced by proper current task access
fn create_demo_task() -> &'static task::Task {
    use super::caps::CTable;

    // Create a static demo task for Phase 2 testing
    // This is obviously not production code - just for demonstration
    static mut DEMO_TASK: Option<task::Task> = None;

    unsafe {
        if DEMO_TASK.is_none() {
            DEMO_TASK = Some(task::Task {
                id: 1,
                name: "demo_task",
                role: task::Role::Philosophy,
                stack: alloc::vec![0u8; 4096].into_boxed_slice(),
                context: task::TaskContext {
                    r15: 0,
                    r14: 0,
                    r13: 0,
                    r12: 0,
                    rbx: 0,
                    rbp: 0,
                    rip: 0,
                    rsp: 0,
                },
                state: task::State::Ready,
                priority: 1,
                affinity_core: 0,
                affinity_gpu: None,
                next: None,
                kstack_top: 4096, // Demo value
                #[cfg(feature = "per-task-mm")]
                cr3_root: None,
                #[cfg(feature = "per-task-mm")]
                user_stack_top: 0,
                #[cfg(feature = "per-task-mm")]
                guard_pages: (0, 0),
                #[cfg(feature = "per-task-mm")]
                mm_stats: task::TaskMmStats::default(),
                #[cfg(feature = "ipc")]
                ctable: CTable::new(),
                #[cfg(feature = "scheduler")]
                priority_boost: true,
            });
        }
        DEMO_TASK.as_ref().unwrap()
    }
}
