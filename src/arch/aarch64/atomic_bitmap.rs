//! Atomic Bitmap Allocator for Neural Engine Memory
//!
//! Lock-free, race-condition-free bitmap allocator implementing
//! recommendations from Multi-AI consultation. Uses atomic operations
//! for thread-safe concurrent allocation/deallocation.

use core::sync::atomic::{AtomicU64, Ordering};
use core::mem::size_of;
use alloc::vec::Vec;

/// Atomic bitmap allocator for Neural Engine memory management
pub struct AtomicBitmapAllocator {
    /// Atomic bitmap words (each u64 represents 64 pages)
    bitmap: &'static [AtomicU64],
    /// Total number of pages managed
    total_pages: usize,
    /// Size of each page in bytes
    page_size: usize,
    /// Base physical address of the managed region
    base_addr: u64,
}

/// Allocation result
#[derive(Debug, Clone, Copy)]
pub struct AllocResult {
    /// Physical address of allocated region
    pub phys_addr: u64,
    /// Size of allocated region in bytes
    pub size_bytes: usize,
    /// Starting page index
    pub start_page: usize,
    /// Number of allocated pages
    pub page_count: usize,
}

/// Allocation errors
#[derive(Debug, Clone, Copy)]
pub enum AllocError {
    /// No contiguous region of requested size available
    OutOfMemory,
    /// Invalid alignment request
    InvalidAlignment,
    /// Invalid size request (zero or too large)
    InvalidSize,
    /// Internal allocator error
    InternalError,
}

/// Bit manipulation utilities for atomic operations
struct BitOps;

impl BitOps {
    /// Find first run of k consecutive zero bits in a u64 word
    #[inline]
    fn find_run_in_word(word: u64, k: u32) -> Option<u32> {
        if k == 0 || k > 64 {
            return None;
        }

        let mask = if k == 64 { !0u64 } else { (1u64 << k) - 1 };
        
        for start in 0..=(64 - k) {
            let shifted_mask = mask << start;
            if (word & shifted_mask) == 0 {
                return Some(start);
            }
        }
        None
    }

    /// Create bitmask for k consecutive bits starting at position start
    #[inline]
    fn create_mask(start: u32, k: u32) -> u64 {
        if k == 0 || start + k > 64 {
            return 0;
        }
        let mask = if k == 64 { !0u64 } else { (1u64 << k) - 1 };
        mask << start
    }

    /// Count leading zeros in a u64 (for alignment calculations)
    #[inline]
    fn leading_zeros(word: u64) -> u32 {
        word.leading_zeros()
    }

    /// Count trailing zeros in a u64
    #[inline]
    fn trailing_zeros(word: u64) -> u32 {
        word.trailing_zeros()
    }
}

impl AtomicBitmapAllocator {
    /// Create new atomic bitmap allocator
    /// 
    /// # Safety
    /// - bitmap must be properly initialized atomic array
    /// - base_addr must be valid physical address
    /// - total_pages must match bitmap capacity
    pub unsafe fn new(
        bitmap: &'static [AtomicU64], 
        total_pages: usize,
        page_size: usize,
        base_addr: u64
    ) -> Result<Self, AllocError> {
        // Validate parameters
        let expected_bitmap_len = (total_pages + 63) / 64;
        if bitmap.len() < expected_bitmap_len {
            return Err(AllocError::InternalError);
        }

        if page_size == 0 || !page_size.is_power_of_two() {
            return Err(AllocError::InvalidAlignment);
        }

        Ok(Self {
            bitmap,
            total_pages,
            page_size,
            base_addr,
        })
    }

    /// Allocate contiguous pages with specified alignment
    pub fn alloc_pages(&self, num_pages: usize, alignment_pages: usize) -> Result<AllocResult, AllocError> {
        if num_pages == 0 || num_pages > self.total_pages {
            return Err(AllocError::InvalidSize);
        }

        if alignment_pages == 0 || !alignment_pages.is_power_of_two() {
            return Err(AllocError::InvalidAlignment);
        }

        // Try allocation with exponential backoff for contention
        let mut backoff = 1;
        for attempt in 0..1000 {
            if let Some(result) = self.try_alloc_pages(num_pages, alignment_pages) {
                return Ok(result);
            }

            // Exponential backoff with jitter to reduce contention
            for _ in 0..backoff {
                unsafe { core::arch::asm!("yield", options(nomem, nostack, preserves_flags)); }
            }
            backoff = (backoff * 2).min(64);

            // Periodic stronger backoff
            if attempt % 100 == 99 {
                for _ in 0..1000 {
                    unsafe { core::arch::asm!("nop", options(nomem, nostack, preserves_flags)); }
                }
            }
        }

        Err(AllocError::OutOfMemory)
    }

    /// Attempt single allocation (may fail due to races)
    fn try_alloc_pages(&self, num_pages: usize, alignment_pages: usize) -> Option<AllocResult> {
        // Handle single page allocation optimized path
        if num_pages == 1 && alignment_pages == 1 {
            return self.try_alloc_single_page();
        }

        // Handle multi-page allocation
        self.try_alloc_multi_page(num_pages, alignment_pages)
    }

    /// Optimized single page allocation
    fn try_alloc_single_page(&self) -> Option<AllocResult> {
        for word_idx in 0..self.bitmap.len() {
            let current = self.bitmap[word_idx].load(Ordering::Acquire);
            
            // Find first zero bit
            if current != !0u64 {
                let bit_pos = BitOps::trailing_zeros(!current);
                if bit_pos < 64 {
                    let mask = 1u64 << bit_pos;
                    let new_value = current | mask;
                    
                    // Try to claim this bit atomically
                    match self.bitmap[word_idx].compare_exchange_weak(
                        current, new_value, Ordering::AcqRel, Ordering::Relaxed
                    ) {
                        Ok(_) => {
                            let page_idx = word_idx * 64 + bit_pos as usize;
                            if page_idx < self.total_pages {
                                return Some(AllocResult {
                                    phys_addr: self.base_addr + (page_idx * self.page_size) as u64,
                                    size_bytes: self.page_size,
                                    start_page: page_idx,
                                    page_count: 1,
                                });
                            } else {
                                // Allocated beyond bounds, free it
                                self.bitmap[word_idx].fetch_and(!mask, Ordering::AcqRel);
                                return None;
                            }
                        }
                        Err(_) => {
                            // Race occurred, continue to next bit/word
                            continue;
                        }
                    }
                }
            }
        }
        None
    }

    /// Multi-page allocation with alignment
    fn try_alloc_multi_page(&self, num_pages: usize, alignment_pages: usize) -> Option<AllocResult> {
        // First pass: find suitable location without allocation
        let candidate = self.find_aligned_range(num_pages, alignment_pages)?;
        
        // Second pass: attempt atomic allocation
        self.try_claim_range(candidate.start_page, num_pages)
    }

    /// Find aligned range that could fit the allocation
    fn find_aligned_range(&self, num_pages: usize, alignment_pages: usize) -> Option<AllocResult> {
        let alignment_mask = alignment_pages - 1;
        
        for start_page in (0..self.total_pages).step_by(alignment_pages) {
            // Check if we have enough pages remaining
            if start_page + num_pages > self.total_pages {
                break;
            }

            // Check if the range is free (snapshot check, may change)
            if self.is_range_free_snapshot(start_page, num_pages) {
                return Some(AllocResult {
                    phys_addr: self.base_addr + (start_page * self.page_size) as u64,
                    size_bytes: num_pages * self.page_size,
                    start_page,
                    page_count: num_pages,
                });
            }
        }
        None
    }

    /// Check if range appears free (non-atomic snapshot)
    fn is_range_free_snapshot(&self, start_page: usize, num_pages: usize) -> bool {
        let end_page = start_page + num_pages;
        
        for page in start_page..end_page {
            let word_idx = page / 64;
            let bit_idx = page % 64;
            
            if word_idx >= self.bitmap.len() {
                return false;
            }
            
            let word = self.bitmap[word_idx].load(Ordering::Acquire);
            if (word & (1u64 << bit_idx)) != 0 {
                return false; // Page is allocated
            }
        }
        true
    }

    /// Atomically claim a range of pages
    fn try_claim_range(&self, start_page: usize, num_pages: usize) -> Option<AllocResult> {
        // Build list of atomic operations needed
        let mut operations = Vec::with_capacity(8);
        
        let end_page = start_page + num_pages;
        for page in start_page..end_page {
            let word_idx = page / 64;
            let bit_idx = page % 64;
            operations.push((word_idx, bit_idx));
        }

        // Sort operations by word index to avoid deadlocks
        operations.sort_by_key(|&(word_idx, _)| word_idx);

        // Group operations by word for batch processing
        let mut current_word = !0usize;
        let mut current_mask = 0u64;
        let mut word_operations = Vec::with_capacity(operations.len());

        for (word_idx, bit_idx) in operations {
            if word_idx != current_word {
                if current_word != !0usize {
                    word_operations.push((current_word, current_mask));
                }
                current_word = word_idx;
                current_mask = 1u64 << bit_idx;
            } else {
                current_mask |= 1u64 << bit_idx;
            }
        }
        
        if current_word != !0usize {
            word_operations.push((current_word, current_mask));
        }

        // Attempt atomic allocation across all affected words
        if self.atomic_multi_word_alloc(&word_operations) {
            Some(AllocResult {
                phys_addr: self.base_addr + (start_page * self.page_size) as u64,
                size_bytes: num_pages * self.page_size,
                start_page,
                page_count: num_pages,
            })
        } else {
            None
        }
    }

    /// Atomically allocate across multiple words
    fn atomic_multi_word_alloc(&self, operations: &[(usize, u64)]) -> bool {
        // For single word operations, use simple CAS
        if operations.len() == 1 {
            let (word_idx, mask) = operations[0];
            if word_idx >= self.bitmap.len() {
                return false;
            }

            let current = self.bitmap[word_idx].load(Ordering::Acquire);
            if (current & mask) != 0 {
                return false; // Some bits already allocated
            }

            let new_value = current | mask;
            self.bitmap[word_idx].compare_exchange(
                current, new_value, Ordering::AcqRel, Ordering::Relaxed
            ).is_ok()
        } else {
            // Multi-word allocation requires more complex coordination
            self.atomic_multi_word_alloc_complex(operations)
        }
    }

    /// Complex multi-word atomic allocation using retry logic
    fn atomic_multi_word_alloc_complex(&self, operations: &[(usize, u64)]) -> bool {
        const MAX_RETRIES: usize = 10;
        
        for retry in 0..MAX_RETRIES {
            // Phase 1: Check if all bits are available
            let mut snapshots = Vec::with_capacity(operations.len());
            let mut all_available = true;
            
            for &(word_idx, mask) in operations {
                if word_idx >= self.bitmap.len() {
                    all_available = false;
                    break;
                }
                
                let current = self.bitmap[word_idx].load(Ordering::Acquire);
                snapshots.push((word_idx, current));
                
                if (current & mask) != 0 {
                    all_available = false;
                    break;
                }
            }

            if !all_available {
                return false;
            }

            // Phase 2: Attempt atomic update of all words
            let mut success = true;
            let mut updated_words = Vec::new();

            for (idx, &(word_idx, mask)) in operations.iter().enumerate() {
                let (_, snapshot) = snapshots[idx];
                let new_value = snapshot | mask;
                
                match self.bitmap[word_idx].compare_exchange(
                    snapshot, new_value, Ordering::AcqRel, Ordering::Relaxed
                ) {
                    Ok(_) => {
                        updated_words.push((word_idx, mask));
                    }
                    Err(_) => {
                        success = false;
                        break;
                    }
                }
            }

            if success {
                return true;
            } else {
                // Rollback any successful updates
                for (word_idx, mask) in updated_words {
                    self.bitmap[word_idx].fetch_and(!mask, Ordering::AcqRel);
                }

                // Exponential backoff before retry
                for _ in 0..(1 << retry.min(6)) {
                    unsafe { core::arch::asm!("yield", options(nomem, nostack, preserves_flags)); }
                }
            }
        }

        false
    }

    /// Free allocated pages
    pub fn free_pages(&self, alloc_result: &AllocResult) -> Result<(), AllocError> {
        let start_page = alloc_result.start_page;
        let num_pages = alloc_result.page_count;
        
        // Validate the allocation result
        if start_page + num_pages > self.total_pages {
            return Err(AllocError::InternalError);
        }

        // Free all pages in the range
        for page in start_page..(start_page + num_pages) {
            let word_idx = page / 64;
            let bit_idx = page % 64;
            let mask = 1u64 << bit_idx;
            
            // Atomic clear of the bit
            let previous = self.bitmap[word_idx].fetch_and(!mask, Ordering::AcqRel);
            
            // Verify the bit was actually set (debug check)
            debug_assert!((previous & mask) != 0, "Double free detected");
        }

        Ok(())
    }

    /// Get allocation statistics
    pub fn get_stats(&self) -> BitmapStats {
        let mut allocated_pages = 0;
        let mut total_words_checked = 0;

        for word in self.bitmap.iter() {
            let word_value = word.load(Ordering::Acquire);
            allocated_pages += word_value.count_ones() as usize;
            total_words_checked += 1;
            
            // Limit checking to actual capacity
            if total_words_checked * 64 >= self.total_pages {
                // Adjust for partial last word
                let pages_in_last_word = self.total_pages % 64;
                if pages_in_last_word > 0 {
                    let valid_bits = (1u64 << pages_in_last_word) - 1;
                    let valid_allocated = (word_value & valid_bits).count_ones() as usize;
                    allocated_pages = allocated_pages - word_value.count_ones() as usize + valid_allocated;
                }
                break;
            }
        }

        let free_pages = self.total_pages - allocated_pages;
        let utilization = if self.total_pages > 0 {
            (allocated_pages * 10000 / self.total_pages) as u32 // Fixed-point percentage * 100
        } else {
            0
        };

        BitmapStats {
            total_pages: self.total_pages,
            allocated_pages,
            free_pages,
            utilization_bp: utilization, // basis points (1/100th of percent)
            fragmentation_estimate: self.estimate_fragmentation(),
        }
    }

    /// Estimate memory fragmentation
    fn estimate_fragmentation(&self) -> u32 {
        let mut free_runs = 0;
        let mut total_free = 0;
        let mut current_run_length = 0;

        for word_idx in 0..self.bitmap.len() {
            let word = self.bitmap[word_idx].load(Ordering::Acquire);
            
            // Process each bit in the word
            for bit_idx in 0..64 {
                let page_idx = word_idx * 64 + bit_idx;
                if page_idx >= self.total_pages {
                    break;
                }

                if (word & (1u64 << bit_idx)) == 0 {
                    // Free page
                    current_run_length += 1;
                    total_free += 1;
                } else {
                    // Allocated page - end current run
                    if current_run_length > 0 {
                        free_runs += 1;
                        current_run_length = 0;
                    }
                }
            }
        }

        // Handle final run
        if current_run_length > 0 {
            free_runs += 1;
        }

        // Fragmentation = (number of free runs - 1) / total free pages
        // Higher values indicate more fragmentation
        if total_free > 0 && free_runs > 0 {
            ((free_runs - 1) * 10000 / total_free) as u32
        } else {
            0
        }
    }
}

/// Bitmap allocator statistics
#[derive(Debug, Clone)]
pub struct BitmapStats {
    pub total_pages: usize,
    pub allocated_pages: usize,
    pub free_pages: usize,
    pub utilization_bp: u32,      // Utilization in basis points (1/100th percent)
    pub fragmentation_estimate: u32, // Fragmentation metric
}