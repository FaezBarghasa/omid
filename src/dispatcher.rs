use std::prelude::v1::*;
use std::format;
use core::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use crate::packet::OmidPacket;
use crate::queue::SpscRingBuffer;
use crate::error::OmidError;
use crate::event::OmidFlags;

#[cfg(target_os = "linux")]
mod affinity {
    use std::mem;

    #[repr(C)]
    struct cpu_set_t {
        bits: [usize; 16],
    }

    unsafe extern "C" {
        fn pthread_self() -> libc_pthread_t;
        fn pthread_setaffinity_np(
            thread: libc_pthread_t,
            cpusetsize: usize,
            cpuset: *const cpu_set_t,
        ) -> std::os::raw::c_int;
    }

    #[allow(non_camel_case_types)]
    type libc_pthread_t = usize;

    pub fn pin_current_thread_to_cpu(cpu_id: usize) -> Result<(), std::os::raw::c_int> {
        let mut cpuset = cpu_set_t { bits: [0; 16] };
        let word = cpu_id / (mem::size_of::<usize>() * 8);
        let bit = cpu_id % (mem::size_of::<usize>() * 8);
        if word < 16 {
            cpuset.bits[word] |= 1 << bit;
            unsafe {
                let thread = pthread_self();
                let res = pthread_setaffinity_np(thread, mem::size_of::<cpu_set_t>(), &cpuset);
                if res == 0 {
                    Ok(())
                } else {
                    Err(res)
                }
            }
        } else {
            Err(-1)
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod affinity {
    pub fn pin_current_thread_to_cpu(_cpu_id: usize) -> Result<(), i32> {
        Ok(())
    }
}

/// Thread-safe statistics tracking for the dispatcher.
#[derive(Debug, Default)]
pub struct DispatcherStats {
    /// Count of KeyPress events processed.
    pub key_presses: AtomicUsize,
    /// Count of KeyRelease events processed.
    pub key_releases: AtomicUsize,
    /// Count of AbsoluteChange (fader) events processed.
    pub absolute_changes: AtomicUsize,
    /// Count of HapticFeedback events processed.
    pub haptic_feedbacks: AtomicUsize,
}

/// Type definition for registered DSP callbacks.
pub type OmidCallback = Box<dyn Fn(OmidPacket) + Send + Sync + 'static>;

/// Global Parallel Dispatcher Engine supporting bidirectional transmission, registered callbacks, and latency hooks.
pub struct OmidHostDispatcher {
    running: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    stats: Arc<DispatcherStats>,
    tx_queue: Arc<SpscRingBuffer<OmidPacket, 4096>>,
    callbacks: Arc<RwLock<HashMap<u16, OmidCallback>>>,
    sent_timestamps: Arc<RwLock<HashMap<u16, Instant>>>,
    last_rtt_us: Arc<AtomicU64>,
}

impl OmidHostDispatcher {
    /// Instantiates the parallel dispatch loop.
    pub fn new(
        worker_count: usize,
        rx_queues: Vec<Arc<SpscRingBuffer<OmidPacket, 4096>>>,
        tx_queue: Arc<SpscRingBuffer<OmidPacket, 4096>>,
        stats: Arc<DispatcherStats>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let mut threads = Vec::new();
        let callbacks = Arc::new(RwLock::new(HashMap::new()));
        let sent_timestamps = Arc::new(RwLock::new(HashMap::new()));
        let last_rtt_us = Arc::new(AtomicU64::new(0));

        for (thread_idx, queue_ref) in rx_queues.iter().enumerate().take(worker_count) {
            let is_running = Arc::clone(&running);
            let queue = Arc::clone(queue_ref);
            let thread_stats = Arc::clone(&stats);
            let thread_tx = Arc::clone(&tx_queue);
            let thread_callbacks = Arc::clone(&callbacks);
            let thread_timestamps = Arc::clone(&sent_timestamps);
            let thread_rtt = Arc::clone(&last_rtt_us);

            let handle = thread::Builder::new()
                .name(format!("OMID-Worker-{}", thread_idx))
                .spawn(move || {
                    let _ = affinity::pin_current_thread_to_cpu(thread_idx + 2);

                    let mut spins = 0;
                    while is_running.load(Ordering::Relaxed) {
                        if let Some(packet) = queue.pop() {
                            Self::process_packet_on_dsp(
                                packet,
                                thread_idx,
                                &thread_stats,
                                &thread_tx,
                                &thread_callbacks,
                                &thread_timestamps,
                                &thread_rtt,
                            );
                            spins = 0;
                        } else {
                            spins += 1;
                            if spins > 1000 {
                                std::thread::yield_now();
                                if spins > 10000 {
                                    std::thread::sleep(std::time::Duration::from_micros(10));
                                    spins = 1000;
                                }
                            } else {
                                std::hint::spin_loop();
                            }
                        }
                    }
                })
                .expect("Failed to spawn real-time dispatch thread");

            threads.push(handle);
        }

        Self {
            running,
            threads,
            stats,
            tx_queue,
            callbacks,
            sent_timestamps,
            last_rtt_us,
        }
    }

    /// Registers a handler callback for a specific object ID.
    pub fn register_callback<F>(&self, object_id: u16, handler: F)
    where
        F: Fn(OmidPacket) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.callbacks.write() {
            guard.insert(object_id, Box::new(handler));
        }
    }

    /// Maps an `OmidPacket` to a standard Open Sound Control (OSC) message byte buffer.
    ///
    /// Padded according to OSC spec (aligned to 4-byte boundaries).
    ///
    /// # Errors
    ///
    /// Returns `OmidError::BufferTooSmall` if the destination buffer is not large enough.
    pub fn map_to_osc(packet: OmidPacket, out_buf: &mut [u8]) -> Result<usize, OmidError> {
        let path = std::format!("/omid/{}\0", packet.object_id);
        let path_len = path.len();
        let padded_path_len = (path_len + 3) & !3; // align to 4 bytes

        let type_tag = ",f\0\0";
        let total_len = padded_path_len + 4 + 4; // path + tag + float

        if out_buf.len() < total_len {
            return Err(OmidError::BufferTooSmall);
        }

        out_buf[..total_len].fill(0);
        out_buf[..path_len].copy_from_slice(path.as_bytes());
        out_buf[padded_path_len..padded_path_len + 4].copy_from_slice(type_tag.as_bytes());

        let val = if packet.is_raw_data() {
            packet.payload_as_normalized_f32(12)
        } else {
            packet.payload_as_f32()
        };
        let val_bytes = val.to_be_bytes();
        out_buf[padded_path_len + 4..padded_path_len + 8].copy_from_slice(&val_bytes);

        Ok(total_len)
    }

    /// Dispatch a packet to the appropriate queue using Voice ID routing:
    /// `Voice ID = Object ID % Thread Count`
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::NoWorkerQueues)` if `queues` is empty.
    /// Returns `Err(OmidError::QueueOverflow)` if the routed queue is saturated.
    #[inline(always)]
    pub fn dispatch(
        &self,
        packet: OmidPacket,
        queues: &[Arc<SpscRingBuffer<OmidPacket, 4096>>],
    ) -> Result<(), OmidError> {
        let worker_count = queues.len();
        if worker_count == 0 {
            return Err(OmidError::NoWorkerQueues);
        }
        let thread_idx = (packet.object_id as usize) % worker_count;
        queues[thread_idx].push(packet)
    }

    /// Submits an outbound control packet directly to the transmission queue.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::QueueOverflow)` if the transmission queue is full.
    #[inline]
    pub fn submit_to_device(&self, packet: OmidPacket) -> Result<(), OmidError> {
        self.tx_queue.push(packet)
    }

    /// Submits an outbound control packet and records the timestamp to track latency.
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::QueueOverflow)` if the transmission queue is full.
    pub fn submit_to_device_with_timestamp(&self, packet: OmidPacket) -> Result<(), OmidError> {
        if let Ok(mut guard) = self.sent_timestamps.write() {
            guard.insert(packet.object_id, Instant::now());
        }
        self.submit_to_device(packet)
    }

    /// Returns the last measured round-trip time (RTT) latency in microseconds.
    #[inline]
    pub fn last_rtt_micros(&self) -> u64 {
        self.last_rtt_us.load(Ordering::Relaxed)
    }

    /// Returns a reference to the outbound transmission queue.
    #[inline]
    pub fn tx_queue(&self) -> &Arc<SpscRingBuffer<OmidPacket, 4096>> {
        &self.tx_queue
    }

    #[inline(always)]
    fn process_packet_on_dsp(
        packet: OmidPacket,
        _worker_id: usize,
        stats: &DispatcherStats,
        tx_queue: &Arc<SpscRingBuffer<OmidPacket, 4096>>,
        callbacks: &RwLock<HashMap<u16, OmidCallback>>,
        sent_timestamps: &RwLock<HashMap<u16, Instant>>,
        last_rtt_us: &AtomicU64,
    ) {
        // Measure latency if this was a response/echo from the device
        if let Ok(mut guard) = sent_timestamps.write() {
            if let Some(sent_time) = guard.remove(&packet.object_id) {
                let rtt = sent_time.elapsed().as_micros() as u64;
                last_rtt_us.store(rtt, Ordering::Relaxed);
            }
        }

        // Execute user DSP callback if registered
        if let Ok(guard) = callbacks.read() {
            if let Some(cb) = guard.get(&packet.object_id) {
                cb(packet);
            }
        }

        let event_type = packet.event();
        match event_type {
            crate::event::EventType::KeyPress => {
                stats.key_presses.fetch_add(1, Ordering::Relaxed);
                let _val = if packet.is_raw_data() {
                    packet.payload_as_normalized_f32(16)
                } else {
                    packet.payload_as_f32()
                };
                let _offset = packet.subsample_offset();

                // Echo VisualUpdate feedback back to the device to illuminate key LEDs
                let feedback_flags = OmidFlags::new(packet.is_touched(), false, false, 0);
                let feedback_packet = OmidPacket::new_u32(
                    packet.object_id,
                    crate::event::EventType::VisualUpdate,
                    feedback_flags,
                    0xFF00FF00, // Green LED
                );
                let _ = tx_queue.push(feedback_packet);
            }
            crate::event::EventType::KeyRelease => {
                stats.key_releases.fetch_add(1, Ordering::Relaxed);
                let _offset = packet.subsample_offset();

                // Turn off key LED on release
                let feedback_flags = OmidFlags::new(false, false, false, 0);
                let feedback_packet = OmidPacket::new_u32(
                    packet.object_id,
                    crate::event::EventType::VisualUpdate,
                    feedback_flags,
                    0x00000000,
                );
                let _ = tx_queue.push(feedback_packet);
            }
            crate::event::EventType::AbsoluteChange => {
                stats.absolute_changes.fetch_add(1, Ordering::Relaxed);
                let val = if packet.is_raw_data() {
                    packet.payload_as_normalized_f32(12)
                } else {
                    packet.payload_as_f32()
                };
                let _offset = packet.subsample_offset();

                // Echo back fader position change to synchronize motorized faders on hardware
                let feedback_flags = OmidFlags::new(packet.is_touched(), false, false, 0);
                let feedback_packet = OmidPacket::new_f32(
                    packet.object_id,
                    crate::event::EventType::AbsoluteChange,
                    feedback_flags,
                    val,
                );
                let _ = tx_queue.push(feedback_packet);
            }
            crate::event::EventType::HapticFeedback => {
                stats.haptic_feedbacks.fetch_add(1, Ordering::Relaxed);
                let _profile = packet.haptic_force_profile();
                let _intensity = packet.haptic_intensity();
            }
            _ => {}
        }
    }

    /// Retrieve references to the statistics tracker.
    #[inline]
    pub fn stats(&self) -> &DispatcherStats {
        &self.stats
    }

    /// Cleanly stops all processing threads.
    pub fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);
        for thread in self.threads {
            let _ = thread.join();
        }
    }
}
