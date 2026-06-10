# Omid User Manual

Welcome to the Omid (Object-MIDI) user manual. This guide will walk you through integrating Omid into your embedded hardware or software project.

## Getting Started

Add Omid to your `Cargo.toml`:

```toml
[dependencies]
omid = "1.1.0"
```

Omid has no dependencies and does not require the Rust standard library, meaning it will compile out-of-the-box for `thumbv6m`, `thumbv7m`, `wasm32`, and all standard desktop targets.

## Working with Packets

Every interaction in Omid is encapsulated in an `OmidPacket`. You construct packets based on the type of data your hardware generates.

### Floating Point Data (e.g., Faders, Mod Wheels)

For continuous controllers, `f32` values normalized between `0.0` and `1.0` are standard.

```rust
use omid::{OmidPacket, EventType};

let fader_id = 12;
let current_value = 0.82;

// Construct a packet with the Touched flag active
let packet = OmidPacket::new_f32(
    fader_id, 
    EventType::AbsoluteChange, 
    OmidPacket::FLAG_TOUCHED, 
    current_value
);

assert_eq!(packet.payload_as_f32(), 0.82);
```

### Split Data (e.g., XY Pads, Joysticks)

If your hardware captures dual-axis data, you can split the 4-byte payload into two `u16` values.

```rust
use omid::{OmidPacket, EventType};

let xy_pad_id = 55;
let x_pos = 4000;
let y_pos = 1200;

let packet = OmidPacket::new_xy(
    xy_pad_id,
    EventType::AbsoluteChange,
    0, // No flags
    x_pos,
    y_pos
);

let (x, y) = packet.payload_as_xy();
```

## Transporting Data

Once a packet is created, use `.to_bytes()` to retrieve the raw `[u8; 8]` array. This array is fully compatible with serial ports, SPI, I2C, and USB Bulk/Interrupt endpoints. On the receiving software side (like a DAW integration), ingest the 8 bytes and use `OmidPacket::from_bytes(&buffer)` to interact with the parsed data cleanly.