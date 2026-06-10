use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A high-performance, lock-free SPSC (Single Producer Single Consumer) Ring Buffer
/// tailored for real-time DSP pipelines.
pub struct SpscRingBuffer<T: Copy, const N: usize> {
    buffer: UnsafeCell<[Option<T>; N]>,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

// Manually implement Send and Sync because UnsafeCell is !Sync by default.
// The SPSC invariants guarantee that push (writer) and pop (reader)
// do not access the same memory location concurrently.
unsafe impl<T: Copy + Send, const N: usize> Send for SpscRingBuffer<T, N> {}
unsafe impl<T: Copy + Send, const N: usize> Sync for SpscRingBuffer<T, N> {}

impl<T: Copy, const N: usize> SpscRingBuffer<T, N> {
    /// Creates a new `SpscRingBuffer`.
    pub fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([None; N]),
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    /// Enqueues an item into the ring buffer. Non-blocking.
    /// Takes a shared reference `&self` to allow thread sharing via `Arc`.
    #[inline(always)]
    pub fn push(&self, item: T) -> Result<(), &'static str> {
        let current_write = self.write_idx.load(Ordering::Relaxed);
        let current_read = self.read_idx.load(Ordering::Acquire);

        if current_write.wrapping_sub(current_read) == N {
            return Err("Queue Overflow - DSP Buffer Saturated");
        }

        unsafe {
            let buffer_ptr = self.buffer.get();
            (*buffer_ptr)[current_write % N] = Some(item);
        }
        self.write_idx.store(current_write.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Enqueues an item into the ring buffer using a mutable reference.
    /// Provided for API compatibility with single-threaded mutable usage.
    #[inline(always)]
    pub fn push_mut(&mut self, item: T) -> Result<(), &'static str> {
        self.push(item)
    }

    /// Dequeues an item from the ring buffer. Non-blocking.
    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        let current_read = self.read_idx.load(Ordering::Relaxed);
        let current_write = self.write_idx.load(Ordering::Acquire);

        if current_read == current_write {
            return None; // Buffer is empty
        }

        let item = unsafe {
            let buffer_ptr = self.buffer.get();
            (*buffer_ptr)[current_read % N]
        };
        self.read_idx.store(current_read.wrapping_add(1), Ordering::Release);
        item
    }
}

impl<T: Copy, const N: usize> Default for SpscRingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
