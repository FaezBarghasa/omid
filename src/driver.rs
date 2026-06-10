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
    pub hardware: MockHardwareDriver,
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
    pub hardware: MockHardwareDriver,
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
        let _len = buffer.len();
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
    pub hardware: MockHardwareDriver,
}

impl MacosDriver {
    pub fn new(hardware: MockHardwareDriver) -> Self {
        Self { hardware }
    }

    /// Configures the Darwin Kernel Time Constraint Policy for a real-time audio thread.
    pub fn apply_thread_time_constraint_policy(&self) {
        // Darwin kernel parameters simulation
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

/// IoT Bluetooth 5 and above driver simulation.
pub struct BleDriver {
    pub hardware: MockHardwareDriver,
    connected: Arc<AtomicBool>,
    phy_2m: bool,
    mtu: u16,
    l2cap_coc: bool,
}

impl BleDriver {
    /// Instantiates a new Bluetooth 5+ simulated driver.
    pub fn new(hardware: MockHardwareDriver, phy_2m: bool, mtu: u16, l2cap_coc: bool) -> Self {
        Self {
            hardware,
            connected: Arc::new(AtomicBool::new(false)),
            phy_2m,
            mtu: if mtu > 512 { 512 } else if mtu < 23 { 23 } else { mtu },
            l2cap_coc,
        }
    }

    /// Simulates Bluetooth 5 connection event.
    pub fn connect(&self) {
        self.connected.store(true, Ordering::Relaxed);
    }

    /// Simulates Bluetooth 5 disconnection event.
    pub fn disconnect(&self) {
        self.connected.store(false, Ordering::Relaxed);
    }

    /// Checks if device is connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Updates the PHY configuration (e.g. switching to BLE 2M PHY).
    pub fn set_phy(&mut self, phy_2m: bool) {
        self.phy_2m = phy_2m;
    }

    /// Negotiates the GATT Attribute MTU size. Bluetooth 5 allows up to 512 bytes.
    pub fn negotiate_mtu(&mut self, target_mtu: u16) -> u16 {
        self.mtu = if target_mtu > 512 { 512 } else if target_mtu < 23 { 23 } else { target_mtu };
        self.mtu
    }

    /// Returns the current MTU size.
    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    /// Checks if L2CAP Connection-Oriented Channels are active.
    pub fn is_l2cap_coc_active(&self) -> bool {
        self.l2cap_coc
    }
}

impl OmidDriver for BleDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        if !self.is_connected() {
            return Err("Bluetooth not connected");
        }
        self.hardware.submit_control(packet)
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        if !self.is_connected() {
            return None;
        }
        self.hardware.poll_control()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        if !self.is_connected() {
            return Err("Bluetooth not connected");
        }
        self.hardware.submit_audio(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        if !self.is_connected() {
            return None;
        }
        self.hardware.poll_audio()
    }
}

/// IoT WiFi driver simulation (supporting TCP/UDP socket configurations).
pub struct WifiDriver {
    pub hardware: MockHardwareDriver,
    connected: Arc<AtomicBool>,
    use_tcp: bool,
    ip_address: String,
    port: u16,
    tcp_nodelay: bool,
    simulated_latency_ms: u32,
}

impl WifiDriver {
    /// Instantiates a new WiFi simulated driver.
    pub fn new(
        hardware: MockHardwareDriver,
        use_tcp: bool,
        ip_address: String,
        port: u16,
        tcp_nodelay: bool,
    ) -> Self {
        Self {
            hardware,
            connected: Arc::new(AtomicBool::new(false)),
            use_tcp,
            ip_address,
            port,
            tcp_nodelay,
            simulated_latency_ms: 0,
        }
    }

    /// Connects to the target network socket.
    pub fn connect(&self) {
        self.connected.store(true, Ordering::Relaxed);
    }

    /// Disconnects from the target network socket.
    pub fn disconnect(&self) {
        self.connected.store(false, Ordering::Relaxed);
    }

    /// Checks if socket is connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Sets simulated socket transmission delay.
    pub fn set_latency(&mut self, latency_ms: u32) {
        self.simulated_latency_ms = latency_ms;
    }

    /// Returns the target IP address.
    pub fn ip_address(&self) -> &str {
        &self.ip_address
    }

    /// Returns the target Port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns whether tcp_nodelay option is active.
    pub fn tcp_nodelay(&self) -> bool {
        self.tcp_nodelay
    }

    /// Returns whether TCP is used (otherwise UDP).
    pub fn is_tcp(&self) -> bool {
        self.use_tcp
    }
}

impl OmidDriver for WifiDriver {
    fn submit_control(&self, packet: OmidPacket) -> Result<(), &'static str> {
        if !self.is_connected() {
            return Err("WiFi socket not connected");
        }
        self.hardware.submit_control(packet)
    }

    fn poll_control(&self) -> Option<OmidPacket> {
        if !self.is_connected() {
            return None;
        }
        self.hardware.poll_control()
    }

    fn submit_audio(&self, sample: f32) -> Result<(), &'static str> {
        if !self.is_connected() {
            return Err("WiFi socket not connected");
        }
        self.hardware.submit_audio(sample)
    }

    fn poll_audio(&self) -> Option<f32> {
        if !self.is_connected() {
            return None;
        }
        self.hardware.poll_audio()
    }
}
