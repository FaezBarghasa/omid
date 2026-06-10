# Omid (Object-MIDI)

**Omid** (Object-MIDI) is a highly optimized, `no_std` compatible Rust library for handling next-generation MIDI and hardware control events. Built for Specification 1.1.0, it is designed for zero-copy compatibility with DMA controllers and high-speed USB bulk endpoints.

## Features

- **`no_std` Compatible:** Ideal for bare-metal embedded development and microcontrollers.
- **Fixed-Size 8-Byte Packets:** Perfectly aligned (4-byte boundary) for highly efficient, zero-copy hardware transfers.
- **Versatile Event Types:** Out-of-the-box support for Absolute/Relative changes, high-precision Key Presses, Haptic Feedback, and Visual Updates.
- **Flexible Payloads:** Pack payloads as IEEE 754 `f32`, fixed-point `u32`, or split `u16`/`u16` (e.g., for XY pad coordinates).
- **Sub-sample Accuracy:** Flag masking for high-resolution microsecond timer offsets to enable ultra-precise VST/DAW integration.

## Usage

Omid provides an intuitive API to quickly create, serialize, and parse data packets.

### Creating and Reading a Packet

```rust
use omid::{OmidPacket, EventType};

fn main() {
    // Create a new packet representing a fader (Object ID 1) moving to 0.75,
    // while actively being touched by the user.
    let packet = OmidPacket::new_f32(
        1,
        EventType::AbsoluteChange,
        OmidPacket::FLAG_TOUCHED,
        0.75,
    );

    // Verify the event type
    assert_eq!(packet.event(), EventType::AbsoluteChange);

    // Check if the capacitive touch flag is active
    if packet.is_touched() {
        println!("Fader is currently being touched!");
    }

    // Read the payload back as an f32
    let value = packet.payload_as_f32();
    println!("Fader value: {}", value);
}
```

### Serialization for DMA/USB Transfer

Omid packets can be trivially serialized and deserialized to 8-byte arrays in Little-Endian format for transport across hardware boundaries.

```rust
use omid::{OmidPacket, EventType};

fn main() {
    let original_packet = OmidPacket::new_xy(
        42, 
        EventType::AbsoluteChange, 
        0, 
        1024, 
        2048
    );

    // Serialize to an 8-byte array
    let raw_bytes: [u8; 8] = original_packet.to_bytes();

    // Data is now ready to be sent over USB or DMA...

    // Deserialize back into an OmidPacket on the receiving end
    let restored_packet = OmidPacket::from_bytes(&raw_bytes);
    let (x, y) = restored_packet.payload_as_xy();

    assert_eq!(x, 1024);
    assert_eq!(y, 2048);
}
```

## Packet Structure

Each `OmidPacket` strictly adheres to an 8-byte structure in memory:
- **`object_id` (2 bytes):** Unique 16-bit Object ID (0x0000 to 0xFFFF).
- **`event_type` (1 byte):** The functional type of the event (e.g., `0x01` for `AbsoluteChange`).
- **`flags` (1 byte):** Bitfield for state modifiers (Touch State, Raw Data, Direction, Sub-sample timer delta).
- **`payload` (4 bytes):** 32-bit payload space, versatile enough for floats or multi-axis coordinates.