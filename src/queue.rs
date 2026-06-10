use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::error::OmidError;

#[repr(align(64))]
struct CacheLineIndex {
    idx: AtomicUsize,
}

impl CacheLineIndex {
    #[inline]
    fn new(val: usize) -> Self {
        Self {
            idx: AtomicUsize::new(val),
        }
    }

    #[inline]
    fn load(&self, order: Ordering) -> usize {
        self.idx.load(order)
    }

    #[inline]
    fn store(&self, val: usize, order: Ordering) {
        self.idx.store(val, order);
    }
}

/// A high-performance, lock-free SPSC (Single Producer Single Consumer) Ring Buffer
/// tailored for real-time DSP pipelines.
///
/// Features cache-aligned index fields to prevent false sharing and mask-based indexing.
/// The capacity `N` MUST be a power of two.
pub struct SpscRingBuffer<T: Copy, const N: usize> {
    buffer: UnsafeCell<[Option<T>; N]>,
    write_idx: CacheLineIndex,
    read_idx: CacheLineIndex,
}

// Manually implement Send and Sync because UnsafeCell is !Sync by default.
unsafe impl<T: Copy + Send, const N: usize> Send for SpscRingBuffer<T, N> {}
unsafe impl<T: Copy + Send, const N: usize> Sync for SpscRingBuffer<T, N> {}

impl<T: Copy, const N: usize> SpscRingBuffer<T, N> {
    // Compile-time check ensuring the capacity is a power of two.
    const _CHECK_POWER_OF_TWO: () = {
        assert!(N > 0 && (N & (N - 1)) == 0, "SpscRingBuffer capacity N must be a power of two");
    };

    /// Creates a new, empty `SpscRingBuffer`.
    #[inline]
    pub fn new() -> Self {
        let _ = Self::_CHECK_POWER_OF_TWO;
        Self {
            buffer: UnsafeCell::new([None; N]),
            write_idx: CacheLineIndex::new(0),
            read_idx: CacheLineIndex::new(0),
        }
    }

    /// Enqueues an item into the ring buffer. Non-blocking.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::QueueOverflow)` if the buffer is full.
    #[inline(always)]
    pub fn push(&self, item: T) -> Result<(), OmidError> {
        let current_write = self.write_idx.load(Ordering::Relaxed);
        let current_read = self.read_idx.load(Ordering::Acquire);
        let mask = N - 1;

        if current_write.wrapping_sub(current_read) == N {
            return Err(OmidError::QueueOverflow);
        }

        unsafe {
            let buffer_ptr = self.buffer.get();
            (*buffer_ptr)[current_write & mask] = Some(item);
        }
        self.write_idx.store(current_write.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Enqueues multiple items into the ring buffer. Non-blocking.
    ///
    /// Returns the number of items successfully enqueued.
    #[inline]
    pub fn push_many(&self, items: &[T]) -> usize {
        let current_write = self.write_idx.load(Ordering::Relaxed);
        let current_read = self.read_idx.load(Ordering::Acquire);
        let mask = N - 1;

        let available = N - (current_write.wrapping_sub(current_read));
        let count = core::cmp::min(items.len(), available);
        if count == 0 {
            return 0;
        }

        unsafe {
            let buffer_ptr = self.buffer.get();
            for i in 0..count {
                (*buffer_ptr)[(current_write + i) & mask] = Some(items[i]);
            }
        }

        self.write_idx.store(current_write.wrapping_add(count), Ordering::Release);
        count
    }

    /// Enqueues an item into the ring buffer using a mutable reference.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::QueueOverflow)` if the buffer is full.
    #[inline(always)]
    pub fn push_mut(&mut self, item: T) -> Result<(), OmidError> {
        self.push(item)
    }

    /// Dequeues an item from the ring buffer. Non-blocking.
    ///
    /// Returns `None` if the buffer is empty.
    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        let current_read = self.read_idx.load(Ordering::Relaxed);
        let current_write = self.write_idx.load(Ordering::Acquire);
        let mask = N - 1;

        if current_read == current_write {
            return None; // Buffer is empty
        }

        let item = unsafe {
            let buffer_ptr = self.buffer.get();
            (*buffer_ptr)[current_read & mask]
        };
        self.read_idx.store(current_read.wrapping_add(1), Ordering::Release);
        item
    }

    /// Dequeues multiple items from the ring buffer. Non-blocking.
    ///
    /// Returns the number of items successfully dequeued.
    #[inline]
    pub fn pop_many(&self, dest: &mut [T]) -> usize {
        let current_read = self.read_idx.load(Ordering::Relaxed);
        let current_write = self.write_idx.load(Ordering::Acquire);
        let mask = N - 1;

        let available = current_write.wrapping_sub(current_read);
        let count = core::cmp::min(dest.len(), available);
        if count == 0 {
            return 0;
        }

        unsafe {
            let buffer_ptr = self.buffer.get();
            for i in 0..count {
                if let Some(item) = (*buffer_ptr)[(current_read + i) & mask] {
                    dest[i] = item;
                }
            }
        }

        self.read_idx.store(current_read.wrapping_add(count), Ordering::Release);
        count
    }
}

impl<T: Copy, const N: usize> Default for SpscRingBuffer<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
