use std::prelude::v1::*;
use std::format;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
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

/// Global Parallel Dispatcher Engine supporting bidirectional transmission.
///
/// Spawns worker threads pinned to specific CPU cores and routes incoming control packets
/// to their respective queues using voice ID routing.
/// Outbound visual updates and motorized fader sync packets are sent back to the device
/// via a lock-free transmission queue.
pub struct OmidHostDispatcher {
    running: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    stats: Arc<DispatcherStats>,
    tx_queue: Arc<SpscRingBuffer<OmidPacket, 4096>>,
}

impl OmidHostDispatcher {
    /// Instantiates the parallel dispatch loop.
    ///
    /// Spawns parallel worker pools dedicated to processing discrete regions of the keyboard.
    pub fn new(
        worker_count: usize,
        rx_queues: Vec<Arc<SpscRingBuffer<OmidPacket, 4096>>>,
        tx_queue: Arc<SpscRingBuffer<OmidPacket, 4096>>,
        stats: Arc<DispatcherStats>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let mut threads = Vec::new();

        for (thread_idx, queue_ref) in rx_queues.iter().enumerate().take(worker_count) {
            let is_running = Arc::clone(&running);
            let queue = Arc::clone(queue_ref);
            let thread_stats = Arc::clone(&stats);
            let thread_tx = Arc::clone(&tx_queue);

            let handle = thread::Builder::new()
                .name(format!("OMID-Worker-{}", thread_idx))
                .spawn(move || {
                    // Pin to CPU core. Workers start at core 2 (leaving 0 and 1 for OS & receiver)
                    let _ = affinity::pin_current_thread_to_cpu(thread_idx + 2);

                    let mut spins = 0;
                    while is_running.load(Ordering::Relaxed) {
                        if let Some(packet) = queue.pop() {
                            Self::process_packet_on_dsp(packet, thread_idx, &thread_stats, &thread_tx);
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
        }
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

    /// Submits an outbound control packet directly to the transmission queue (e.g. from DAW GUI or automation).
    ///
    /// # Errors
    ///
    /// Returns `Err(OmidError::QueueOverflow)` if the transmission queue is full.
    #[inline]
    pub fn submit_to_device(&self, packet: OmidPacket) -> Result<(), OmidError> {
        self.tx_queue.push(packet)
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
    ) {
        let event_type = packet.event();
        match event_type {
            crate::event::EventType::KeyPress => {
                stats.key_presses.fetch_add(1, Ordering::Relaxed);
                
                // Emulated low-jitter voice-trigger logic
                let _val = if packet.is_raw_data() {
                    packet.payload_as_normalized_f32(16) // 16-bit key
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
                    0xFF00FF00, // Green LED color value
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
                    0x00000000, // Turn Off LED
                );
                let _ = tx_queue.push(feedback_packet);
            }
            crate::event::EventType::AbsoluteChange => {
                stats.absolute_changes.fetch_add(1, Ordering::Relaxed);
                
                let val = if packet.is_raw_data() {
                    packet.payload_as_normalized_f32(12) // 12-bit fader
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
