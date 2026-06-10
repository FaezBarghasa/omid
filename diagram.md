# Omid Diagrams

## 1. Packet Structure Layout

```mermaid
classDiagram
    class OmidPacket {
        +u16 object_id
        +u8 event_type
        +u8 flags
        +u32 payload
        +new_f32(id, type, flags, val) OmidPacket
        +new_xy(id, type, flags, x, y) OmidPacket
        +payload_as_f32() f32
        +payload_as_xy() (u16, u16)
        +to_bytes() [u8; 8]
        +from_bytes(bytes: &[u8; 8]) OmidPacket
    }
```

## 2. Hardware to Host Data Flow

This diagram illustrates the zero-copy pipeline from a physical hardware interaction to software consumption.

```mermaid
sequenceDiagram
    participant User
    participant MCU (Hardware)
    participant DMA Controller
    participant USB Endpoint
    participant Host Software (Rust)

    User->>MCU (Hardware): Moves Fader (Object ID: 1)
    MCU (Hardware)->>MCU (Hardware): Construct OmidPacket
    MCU (Hardware)->>DMA Controller: Write [u8; 8] into buffer
    DMA Controller->>USB Endpoint: Flush 8-byte packet
    USB Endpoint->>Host Software (Rust): USB Bulk Read Event
    Host Software (Rust)->>Host Software (Rust): OmidPacket::from_bytes()
    Host Software (Rust)->>User: Update UI / Audio Engine (f32)
```

## 3. Payload Interpretation

The 4-byte payload area is flexible depending on the constructor or getter used.

```mermaid
block-beta
    columns 4
    Title("4-Byte Payload Block")
    space:4
    
    block:F32
        Float32("IEEE 754 f32 (e.g., 0.75)")
    end
    
    space:4
    block:U32
        Unsigned32("Standard u32 (e.g., 100000)")
    end

    space:4
    block:XY
        XAxis("X: u16") YAxis("Y: u16")
    end
```