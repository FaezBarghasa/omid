# Omid Architecture

## Overview
The core architectural philosophy of Omid (Object-MIDI) is efficiency and predictability. It is explicitly designed for high-performance hardware integrations, bridging the gap between physical microcontrollers and digital audio workstations (DAWs) or synthesizers without introducing parsing overhead.

Omid packets are exactly 8 bytes long. This specific size was chosen because:
1. It aligns perfectly with 32-bit and 64-bit memory architectures.
2. It comfortably fits within standard USB bulk and interrupt transfer endpoint constraints.
3. It is ideal for Direct Memory Access (DMA) controllers, allowing peripherals to write events directly into memory buffers sequentially without CPU intervention.

## Memory Layout

The `OmidPacket` guarantees an exact 8-byte layout in memory, parsed as Little-Endian:

| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | Byte 5 | Byte 6 | Byte 7 |
|--------|--------|--------|--------|--------|--------|--------|--------|
| `object_id` (LSB) | `object_id` (MSB) | `event_type` | `flags` | `payload` (Byte 0) | `payload` (Byte 1) | `payload` (Byte 2) | `payload` (Byte 3) |

### Component Breakdown

- **Object ID (`u16`)**: Uniquely identifies the hardware component (e.g., Fader 1, Knob A, Pad 5). Spans `0x0000` to `0xFFFF`.
- **Event Type (`u8`)**: Defines how the payload should be interpreted. Examples include `AbsoluteChange` (for faders), `RelativeChange` (for endless encoders), or `KeyPress`.
- **Flags (`u8`)**: A bitmask for state modifiers. This avoids the need for secondary messages. E.g., a fader can send its value and its capacitive touch state simultaneously. It is also used to embed microsecond timing offsets for sub-sample accuracy.
- **Payload (4 bytes)**: A generic 32-bit block. Depending on the `event_type`, this is safely transmuted (zero-copy) into an IEEE 754 `f32`, a standard `u32`, or two `u16` values for dual-axis components like XY pads.

## Zero-Copy & `no_std` Design

The library is strictly `#![no_std]`. It does not rely on heap allocation (`alloc`), making it safe for bare-metal embedded contexts (e.g., Cortex-M, RISC-V microcontrollers). 

Serialization and deserialization methods (`to_bytes` and `from_bytes`) do not allocate memory or perform deep copies; they read from and write to contiguous `[u8; 8]` buffers, relying on fixed array layouts to ensure compiler optimizations reduce the operations to direct register transfers where possible.