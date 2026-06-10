#![cfg(feature = "std")]

use std::prelude::v1::*;
use std::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::packet::OmidPacket;
use crate::queue::SpscRingBuffer;

/// Core interface for high-performance OMID driver transport.
pub trait OmidDriver {
    /// Submits a control packet for transmission to the device.
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str>;

    /// Drains a control packet received from the device, if available.
    fn poll_control(&self) -> Option<OmidPacket>;

    /// Submits an audio sample for transmission back to the device's monitor/haptics.
    fn submit_audio(&self, sample: f32) -> Result<(), &'static str>;

    /// Drains an audio sample received from the device's ADCs.
    fn poll_audio(&self) -> Option<f32>;
}

/// Simulation of the physical OMID endpoints (EP0, EP1, EP2, EP3).
pub struct MockHardwareDriver {
    pub ep1_in: Arc<SpscRingBuffer<OmidPacket, 4096>>,     // Bulk IN: Control from Device
    pub ep2_in: Arc<SpscRingBuffer<f32, 16384>>,            // Isoch IN: Audio from Device
    pub ep3_out: Arc<SpscRingBuffer<f32, 16384>>,           // Isoch OUT: Audio/Haptic to Device
    pub haptic_out: Arc<SpscRingBuffer<OmidPacket, 4096>>,  // Haptic feedback packets to Device
}

impl MockHardwareDriver {
    pub fn new() -> Self {
        Self {
            ep1_in: Arc::new(SpscRingBuffer::new()),
            ep2_in: Arc::new(SpscRingBuffer::new()),
            ep3_out: Arc::new(SpscRingBuffer::new()),
            haptic_out: Arc::new(SpscRingBuffer::new()),
        }
    }
}

impl Default for MockHardwareDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl OmidDriver for MockHardwareDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        if packet.event() == crate::event::EventType::HapticFeedback {
            self.haptic_out.push(packet)
        } else {
            self.ep1_in.push(packet)
        }
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        self.ep1_in.pop()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        self.ep3_out.push(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        self.ep2_in.pop()
    }
}

/// Linux-specific raw usbfs & io_uring zero-copy bypass simulation driver.
pub struct LinuxDriver {
    hardware: MockHardwareDriver,
    uring_active: Arc<AtomicBool>,
}

impl LinuxDriver {
    pub fn new(hardware: MockHardwareDriver) -> Self {
        Self {
            hardware,
            uring_active: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Simulates io_uring completion queue processing loop.
    pub fn process_uring_completions(&self) {
        if self.uring_active.load(Ordering::Relaxed) {
            // Emulating io_uring low-latency packet transfers
            std::hint::spin_loop();
        }
    }

    pub fn shutdown(&self) {
        self.uring_active.store(false, Ordering::Relaxed);
    }
}

impl OmidDriver for LinuxDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        self.hardware.submit_control(packet)
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        self.hardware.poll_control()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        self.hardware.submit_audio(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        self.hardware.poll_audio()
    }
}

/// Windows-specific WinUSB Overlapped I/O Completion Port (IOCP) simulation driver.
pub struct WindowsDriver {
    hardware: MockHardwareDriver,
    iocp_thread_count: usize,
}

impl WindowsDriver {
    pub fn new(hardware: MockHardwareDriver, threads: usize) -> Self {
        Self {
            hardware,
            iocp_thread_count: threads,
        }
    }

    /// Simulates virtual locks and overlapped I/O buffers.
    pub fn simulate_locked_dma_buffer(&self, buffer: &mut [u8]) {
        // Simulating WinUSB Kernel-Bypass Direct DMA (KBDD)
        // Locking page in system memory: VirtualLock(buffer)
        let _len = buffer.len();
        // Zero-copy simulation logic: direct write from hardware DMA...
    }

    pub fn thread_count(&self) -> usize {
        self.iocp_thread_count
    }
}

impl OmidDriver for WindowsDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        self.hardware.submit_control(packet)
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        self.hardware.poll_control()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        self.hardware.submit_audio(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        self.hardware.poll_audio()
    }
}

/// macOS USBDriverKit & Real-time Time Constraint Thread Policy simulation driver.
pub struct MacosDriver {
    hardware: MockHardwareDriver,
}

impl MacosDriver {
    pub fn new(hardware: MockHardwareDriver) -> Self {
        Self { hardware }
    }

    /// Configures the Darwin Kernel Time Constraint Policy for a real-time audio thread.
    pub fn apply_thread_time_constraint_policy(&self) {
        // Darwin kernel parameters simulation:
        // - Period: 1000 microseconds
        // - Computation: 500 microseconds
        // - Constraint: 1000 microseconds
        // - Preemptible: True
    }
}

impl OmidDriver for MacosDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        self.hardware.submit_control(packet)
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        self.hardware.poll_control()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        self.hardware.submit_audio(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        self.hardware.poll_audio()
    }
}
