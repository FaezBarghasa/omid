# Omid (Object-MIDI)

**Omid** (Object-MIDI) is a highly optimized, `no_std` compatible Rust library for handling next-generation MIDI, digital audio, and hardware control events. Built for **Specification 2.0.0 (Global Unified Audio, Control, & Driver Standard)**, it is designed for zero-copy compatibility with DMA controllers, high-speed USB/PCIe endpoints, and wireless IoT hardware interfaces.

## Features

- **`no_std` Core:** Ideal for bare-metal embedded development and microcontrollers (Cortex-M, RISC-V, etc.).
- **Fixed-Size 8-Byte Packets:** Perfectly aligned for highly efficient, zero-copy hardware transfers.
- **High-Resolution ADC Support:** Native support for 12-bit faders (`0..=4095`) and 16-bit keys (`0..=65535`), automatically setting the `RAW_DATA` flag.
- **Haptic Force Feedback:** Bi-directional electromagnetic force feedback profiles (Hammer Strike, Spring Tension, Kinetic Dampening) with dynamic intensity control.
- **Unified Audio & Control Transport (UACT):** Synchronized PCM channels and OMID control signals packed into single clock-locked DMA frames (supporting sample clocks up to 192kHz).
- **Parallel Host Dispatcher:** A lock-free Single-Producer Multi-Consumer (SPMC) routing engine mapping events to affinity-pinned real-time DSP threads with a **configurable callback API**.
- **Real-Time Latency Auditing**: Built-in monotonic timer hooks to measure round-trip time (RTT) from host submission to device response.
- **MIDI 1.0 & 2.0 Translation**: Convert Omid events to/from Note On/Off, CC, Pitch Bend, and pack Omid into 128-bit MIDI 2.0 Sysex8 Universal MIDI Packets (UMP).
- **On-the-Fly Bridge Daemon**: Exposes standard MIDI ports (using `midir`) and translates messages to Omid on the fly.
- **Optimized SPSC Queue**: Cache-line aligned write/read indexes to prevent false sharing, power-of-two capacity masking, and batch `push_many`/`pop_many` methods.
- **IoT & Platform Driver Interfaces**: Real TCP/UDP sockets (`WifiDriver`), `/dev/omid0` usbfs support (`LinuxDriver`), Ble GATT (`BleDriver`), WebUSB browser bindings, and Windows INF configurations.

## Usage

### Creating and Reading high-resolution ADC/Haptic Packets

```rust
use omid::{OmidPacket, EventType, OmidFlags, ForceProfile};

fn main() {
    // Construct a packet containing a 12-bit raw ADC fader value (value: 2048)
    let flags = OmidFlags::new(false, false, false, 0);
    let packet = OmidPacket::new_adc12(1, EventType::AbsoluteChange, flags, 2048);
    
    assert!(packet.is_raw_data());
    assert_eq!(packet.payload_as_adc12(), 2048);
    
    // Normalize to 0.0..=1.0 float dynamically
    let normalized = packet.payload_as_normalized_f32(12);
    println!("Normalized fader: {}", normalized); // 0.5

    // Create a haptic force feedback command (Spring Tension profile, 0.8 intensity)
    let haptic = OmidPacket::new_haptic(1, ForceProfile::SpringTension, 0.8);
    assert_eq!(haptic.haptic_force_profile(), Ok(ForceProfile::SpringTension));
}
```

### Stream Demultiplexing and Clock Sync (UACT)

```rust
use omid::{UactFrame, UactDemuxer, ClockSynchronizer};

fn main() {
    // 2 Audio Channels + Control Packet
    let mut demuxer = UactDemuxer::<2>::new();
    let raw_dma_bytes: [u8; 16] = [0; 16]; // Filled by USB/DMA stream...

    // Zero-allocation stream parsing (optimized ring-buffer parsing)
    demuxer.process_bytes(&raw_dma_bytes, |frame| {
        println!("Received frame with audio: {:?}", frame.audio);
        let subsample_offset = frame.control.typed_flags().subsample_offset();
        
        // Sync with a 122.88 MHz PLL clock at 192 kHz sample rate
        let sync = ClockSynchronizer::new(192000, 122880000.0);
        let sample_offset = sync.sample_offset(subsample_offset);
        println!("Sample-accurate event offset: {} samples", sample_offset);
    }).unwrap();
}
```

### Running the Legacy Bridge Daemon

To launch the CLI bridge on the host and expose Omid devices as standard MIDI ports:
```bash
cargo run --bin bridge --features bridge
```

---

## Packet Structure

Each `OmidPacket` strictly adheres to an 8-byte structure in memory:
- **`object_id` (2 bytes):** Unique 16-bit Object ID (`0x0000` to `0xFFFF`).
- **`event_type` (1 byte):** Functional type of the event (e.g., `0x03` for `KeyPress`, `0x05` for `HapticFeedback`).
- **`flags` (1 byte):** Bitfield for state modifiers (Touch state, Raw data, Direction, Sub-sample timer delta / Force Profile).
- **`payload` (4 bytes):** 32-bit payload space, used for float values, 12/16-bit integers, or XY coordinates.

## Multi-Language FFI Bindings

Omid exports a universal C-API from its dynamic library (`libomid.so` / `omid.dll` / `libomid.dylib`) and includes native wrappers for key languages. This allows VST and hardware driver developers to use Omid in their stack of choice:

- **C++:** Native wrapper class in [bindings/cpp/omid.hpp](file:///home/jrad/RustroverProjects/omid/bindings/cpp/omid.hpp) wrapping the C header.
- **Go:** `cgo` bindings in [bindings/go/omid/omid.go](file:///home/jrad/RustroverProjects/omid/bindings/go/omid/omid.go).
- **Python:** High-level wrapper in [bindings/python/omid.py](file:///home/jrad/RustroverProjects/omid/bindings/python/omid.py) using `ctypes`.
- **TypeScript:** High-performance FFI mapping in [bindings/typescript/omid.ts](file:///home/jrad/RustroverProjects/omid/bindings/typescript/omid.ts) (optimized for Bun/Deno/Node.js).
- **Dart:** `dart:ffi` bindings in [bindings/dart/omid.dart](file:///home/jrad/RustroverProjects/omid/bindings/dart/omid.dart) (Flutter-compatible).
- **C#:** P/Invoke wrappers in [bindings/csharp/Omid.cs](file:///home/jrad/RustroverProjects/omid/bindings/csharp/Omid.cs).
- **Java / Kotlin:** JNA-based bindings in [bindings/java/Omid.java](file:///home/jrad/RustroverProjects/omid/bindings/java/Omid.java) and idiomatic Kotlin wrappers in [bindings/kotlin/Omid.kt](file:///home/jrad/RustroverProjects/omid/bindings/kotlin/Omid.kt).

### Compilation for FFI

To compile Omid as a shared dynamic library and generate the C headers (`include/omid.h`):
```bash
cargo build --release --all-features
```
The compiled library will be in `target/release/` and header files in `include/`.