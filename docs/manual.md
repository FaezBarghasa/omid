# Omid User Manual (v2.0.0)

Welcome to the Omid (Object-MIDI) Integration Manual. This guide explains how to integrate Omid v2.0.0 into your physical embedded controllers, DAW integrations, and software synthesizers.

---

## 1. Getting Started

Add Omid to your project's `Cargo.toml`:

```toml
[dependencies]
omid = { version = "2.0.0", default-features = false }
```

Enable the default `std` feature if you are building host-side applications (DAWs, VST/CLAP plugins) that require multi-threaded dispatch and platform drivers:

```toml
[dependencies]
omid = { version = "2.0.0", features = ["std"] }
```

---

## 2. Working with Packets

Every controller interaction is encapsulated in a contiguous 8-byte `OmidPacket`.

### Continuous Controllers with High-Resolution ADC
Omid supports raw 12-bit (e.g. faders) and 16-bit (e.g. keys) ADC value inputs.

```rust
use omid::{OmidPacket, EventType, OmidFlags};

let object_id = 4; // Fader 4
let raw_val: u16 = 3000; // 12-bit reading from ADC

let flags = OmidFlags::new(false, false, false, 0);
let packet = OmidPacket::new_adc12(object_id, EventType::AbsoluteChange, flags, raw_val);

// Normalizing the raw value on the host side
let normalized = packet.payload_as_normalized_f32(12);
println!("Fader value: {}", normalized); // 0.7326
```

### Active Haptic Force Feedback Solenoids
To transmit tactile fader profiles (e.g., simulating detents or kinetic dampening) back to the device:

```rust
use omid::{OmidPacket, ForceProfile};

let object_id = 4; // Fader 4
let profile = ForceProfile::SpringTension;
let intensity = 0.75f32; // Electromagnetic fader resistance

let haptic_packet = OmidPacket::new_haptic(object_id, profile, intensity);

// Send over the DAC monitor out stream (EP3 OUT)...
```

---

## 3. Host System Integration

### Real-Time Parallel Dispatcher
Spawn a pool of affinity-pinned worker threads to process events without mutex synchronization:

```rust
use std::sync::Arc;
use omid::{OmidHostDispatcher, DispatcherStats, OmidPacket, SpscRingBuffer};

fn run_host() {
    let stats = Arc::new(DispatcherStats::default());
    
    // Define SPSC ring buffers for 2 DSP threads
    let q0 = Arc::new(SpscRingBuffer::<OmidPacket, 4096>::new());
    let q1 = Arc::new(SpscRingBuffer::<OmidPacket, 4096>::new());
    let queues = vec![q0, q1];
    
    // Spawns worker pools (Worker 0 pinned to Core 2, Worker 1 to Core 3)
    let dispatcher = OmidHostDispatcher::new(2, queues.clone(), stats.clone());
    
    // Route incoming control packets to queues: Voice ID = Object ID % 2
    let packet = OmidPacket::from_bytes(&[0; 8]);
    dispatcher.dispatch(packet, &queues).unwrap();
    
    // Clean shutdown when done
    dispatcher.shutdown();
}
```

### Unified Audio & Control Transport (UACT)
Ingest interleaved PCM audio and OMID control packets from physical DMA streams:

```rust
use omid::{UactDemuxer, ClockSynchronizer};

fn process_stream(dma_buffer: &[u8]) {
    // 2 Audio Channels + Control
    let mut demuxer = UactDemuxer::<2>::new();
    let sync = ClockSynchronizer::new(192000, 122880000.0); // 122.88 MHz Clock
    
    demuxer.process_bytes(dma_buffer, |frame| {
        // Access synchronized audio channels
        let left_channel = frame.audio[0];
        let right_channel = frame.audio[1];
        
        // Check control packet
        if frame.control.is_keypress() {
            let offset_ticks = frame.control.typed_flags().subsample_offset();
            let sample_offset = sync.sample_offset(offset_ticks);
            println!("Sample-accurate keypress occurred at +{} samples", sample_offset);
        }
    }).unwrap();
}
```

### Wireless IoT Integration (BLE 5 & WiFi)
Integrate Bluetooth 5 or WiFi drivers:

```rust
use omid::{MockHardwareDriver, BleDriver, WifiDriver, OmidDriver, OmidPacket};

fn run_ble_simulation() {
    let hardware = MockHardwareDriver::new();
    let mut driver = BleDriver::new(hardware, true, 256, true);
    
    driver.connect();
    
    // Send a fader packet over BLE characteristic notifications / L2CAP CoC
    let packet = OmidPacket::from_bytes(&[0; 8]);
    driver.submit_control(packet).unwrap();
}
```