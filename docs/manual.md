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

To run the legacy bridge daemon, enable the `bridge` feature:
```toml
[dependencies]
omid = { version = "2.0.0", features = ["bridge"] }
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

### Configurable Dispatcher Callbacks
Register callback handlers for specific object IDs. When a packet is dispatched, the corresponding callback is executed on the real-time worker thread:

```rust
use std::sync::Arc;
use omid::{OmidHostDispatcher, DispatcherStats, OmidPacket, SpscRingBuffer};

let stats = Arc::new(DispatcherStats::default());
let q0 = Arc::new(SpscRingBuffer::<OmidPacket, 4096>::new());
let rx_queues = vec![q0];
let tx_queue = Arc::new(SpscRingBuffer::<OmidPacket, 4096>::new());

let dispatcher = OmidHostDispatcher::new(1, rx_queues, tx_queue, stats);

// Register DSP callback for Fader ID 5
dispatcher.register_callback(5, |packet| {
    let value = packet.payload_as_f32();
    println!("Fader 5 updated: {}", value);
});
```

### Latency Measurement Hooks
Measure physical control round-trip times (RTT) directly:

```rust
// Submit packet and record trigger timestamp
dispatcher.submit_to_device_with_timestamp(packet).unwrap();

// When the device responds, dispatcher automatically updates latency metrics:
let latency_us = dispatcher.last_rtt_micros();
println!("Current control loop RTT: {} microseconds", latency_us);
```

### SpscRingBuffer Batch Operations
Process packets in batches to minimize thread overhead:
```rust
let q = SpscRingBuffer::<i32, 16>::new();
let items = [1, 2, 3, 4, 5];

// Push a slice of events
let pushed = q.push_many(&items);

// Pop multiple events
let mut dest = [0i32; 8];
let popped = q.pop_many(&mut dest);
```

---

## 4. MIDI 1.0 & 2.0 Translation

OMID provides native translation modules to integrate seamlessly with existing legacy gear.

### MIDI 1.0 note/CC translation
```rust
use omid::Midi1Translator;

// Convert 3-byte MIDI 1.0 raw buffer to OMID
let midi_msg = [0x90, 0x3C, 0x64]; // Note On
let packet = Midi1Translator::to_omid(&midi_msg).unwrap();

// Convert OMID back to MIDI 1.0
let mut out_msg = [0u8; 3];
Midi1Translator::to_midi1(packet, &mut out_msg).unwrap();
```

### MIDI 2.0 UMP Sysex8 Packing
Pack OMID packets into standard 128-bit MIDI 2.0 Sysex8 Universal MIDI Packets (UMP) to travel over standard USB MIDI 2.0 ports:

```rust
use omid::Midi2UmpTranslator;

// Pack to 4 x u32 UMP words
let ump_words = Midi2UmpTranslator::pack_to_sysex8(packet, 0x01 /* group */, 0x55 /* stream ID */);

// Unpack back to OMID
let decoded_packet = Midi2UmpTranslator::unpack_from_sysex8(ump_words).unwrap();
```

---

## 5. Running the Legacy Bridge Daemon
To start translating and routing MIDI port data on the host machine:
```bash
cargo run --bin bridge --features bridge
```
The daemon binds to standard system MIDI ports (ALSA/CoreMIDI/Windows MIDI) and translates traffic on-the-fly.