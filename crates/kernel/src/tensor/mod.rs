//! Zero-copy friendly tensor handle and simple allocator hooks.
//! Phase 1 keeps this minimal; per-graph arenas arrive later.

use core::ptr::NonNull;
use core::alloc::Layout;

#[repr(C, align(64))]
pub struct TensorHeader {
    pub version: u32,
    pub dtype: u32,
    pub dims: [u64; 4],
    pub strides: [u64; 4],
    pub data_offset: u64,
}

#[derive(Copy, Clone)]
pub struct TensorHandle {
    pub ptr: *mut u8,
    pub len: usize,
}

impl TensorHandle {
    #[inline(always)]
    pub fn null() -> Self { Self { ptr: core::ptr::null_mut(), len: 0 } }
    #[inline(always)]
    pub fn is_null(&self) -> bool { self.ptr.is_null() }
}

pub struct TensorAlloc;

impl TensorAlloc {
    /// Allocate an uninitialized tensor buffer of `len` bytes.
    pub unsafe fn alloc_uninit(len: usize, align: usize) -> Option<TensorHandle> {
        let layout = Layout::from_size_align(len, align).ok()?;
        let ptr = alloc::alloc::alloc(layout);
        NonNull::new(ptr).map(|nn| TensorHandle { ptr: nn.as_ptr(), len })
    }

    /// Deallocate a previously allocated buffer.
    pub unsafe fn dealloc(h: TensorHandle, align: usize) {
        if !h.ptr.is_null() {
            if let Ok(layout) = Layout::from_size_align(h.len, align) {
                alloc::alloc::dealloc(h.ptr, layout);
            }
        }
    }
}

