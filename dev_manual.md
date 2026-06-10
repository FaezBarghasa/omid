# Omid Developer Manual

This document provides guidelines for contributing to and maintaining the Omid library.

## Development Environment

Omid is built on Rust's `no_std` ecosystem to ensure it works on microcontrollers as well as desktop operating systems.

### Prerequisites
- Rust toolchain (stable)
- Embedded targets (optional, for verification): `rustup target add thumbv7em-none-eabihf` (Cortex-M4)

## Design Rules & Guidelines

To maintain the performance and embedded-friendly nature of this crate, all contributors must strictly adhere to the following rules:

1. **No Heap Allocation:** The crate must remain `#![no_std]`. Do not introduce the `alloc` crate. All packet manipulation must occur on the stack or in static memory.
2. **Strict 8-Byte Alignment:** The `OmidPacket` struct must remain exactly 8 bytes. Use `#[repr(C, packed)]` or explicit layout padding if internal layouts are modified, and assert the size in tests.
3. **Complete Implementations (No Stubs):** Do not commit code with `todo!()`, `unimplemented!()`, or `FIXME` comments that panic at runtime. All functions must be fully implemented, returning `Result` types if operations can fail.
4. **Endianness:** All serialization and deserialization must use Little-Endian format (`to_le_bytes`, `from_le_bytes`). This ensures uniform interpretation regardless of the host's native architecture.

## Testing

Since the library is `no_std`, tests are run using the standard host-based cargo test runner, which is completely valid for memory-layout testing.

Run the test suite:
```bash
cargo test
```

### Writing Tests
When adding new event types or payload decoders, you must provide comprehensive tests covering:
- End-to-end serialization and deserialization.
- Edge cases (e.g., `NaN` or `Infinity` in `f32` payloads).
- Bitwise operations for flags.

Example test pattern:
```rust
#[test]
fn test_custom_payload_roundtrip() {
    let original = OmidPacket::new_u32(10, EventType::VisualUpdate, 0, 0xFF00AA);
    let bytes = original.to_bytes();
    let restored = OmidPacket::from_bytes(&bytes);
    
    assert_eq!(restored.object_id(), 10);
    assert_eq!(restored.event(), EventType::VisualUpdate);
    assert_eq!(restored.payload_as_u32(), 0xFF00AA);
}
```

## Adding New Event Types

When expanding the Specification (1.1.0+), new event types should be added to the `EventType` enum. 

```rust
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    AbsoluteChange = 0x01,
    RelativeChange = 0x02,
    KeyPress = 0x03,
    HapticFeedback = 0x04,
    VisualUpdate = 0x05,
    // Add new events here
}
```
Remember to update the `TryFrom<u8>` implementation for `EventType` to parse the new byte value successfully.