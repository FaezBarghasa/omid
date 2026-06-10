# Omid (Object-MIDI)

**Omid** (Object-MIDI) is a highly optimized, `no_std` compatible Rust library for handling next-generation MIDI, digital audio, and hardware control events. Built for **Specification 2.0.0 (Global Unified Audio, Control, & Driver Standard)**, it is designed for zero-copy compatibility with DMA controllers, high-speed USB/PCIe endpoints, and wireless IoT hardware interfaces.

## Features

- **`no_std` Core:** Ideal for bare-metal embedded development and microcontrollers (Cortex-M, RISC-V, etc.).
- **Fixed-Size 8-Byte Packets:** Perfectly aligned for highly efficient, zero-copy hardware transfers.
- **High-Resolution ADC Support:** Native support for 12-bit faders (`0..=4095`) and 16-bit keys (`0..=65535`), automatically setting the `RAW_DATA` flag.
- **Haptic Force Feedback:** Bi-directional electromagnetic force feedback profiles (Hammer Strike, Spring Tension, Kinetic Dampening) with dynamic intensity control.
- **Unified Audio & Control Transport (UACT):** Synchronized PCM channels and OMID control signals packed into single clock-locked DMA frames (supporting sample clocks up to 192kHz).
- **Parallel Host Dispatcher:** A lock-free Single-Producer Multi-Consumer (SPMC) routing engine mapping events to affinity-pinned real-time DSP threads.
- **IoT & Platform Driver Wrappers:** Out-of-the-box simulations for Linux (`io_uring`/`usbfs`), Windows (`WinUSB` IOCP), macOS (`USBDriverKit`), Bluetooth 5 (GATT MTU & L2CAP Connection-Oriented Channels), and WiFi (TCP/UDP socket streaming).

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

    // Zero-allocation stream parsing
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

## Packet Structure

Each `OmidPacket` strictly adheres to an 8-byte structure in memory:
- **`object_id` (2 bytes):** Unique 16-bit Object ID (`0x0000` to `0xFFFF`).
- **`event_type` (1 byte):** Functional type of the event (e.g., `0x03` for `KeyPress`, `0x05` for `HapticFeedback`).
- **`flags` (1 byte):** Bitfield for state modifiers (Touch state, Raw data, Direction, Sub-sample timer delta / Force Profile).
- **`payload` (4 bytes):** 32-bit payload space, used for float values, 12/16-bit integers, or XY coordinates.