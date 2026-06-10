# Omid Architecture (Specification 2.0.0)

## Overview
The core architectural philosophy of Omid (Object-MIDI) is predictability, zero-copy alignment, and massive parallelism. Omid v2.0.0 introduces **Unified Audio & Control Transport (UACT)**, bridging the gap between legacy channel-centric MIDI and synchronous multi-channel digital audio over a single physical clock domain.

---

## 1. Packet Structure Layout
Each control event is represented by an exactly 8-byte `OmidPacket` structured in Little-Endian format:

| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | Byte 5 | Byte 6 | Byte 7 |
|--------|--------|--------|--------|--------|--------|--------|--------|
| `object_id` (LSB) | `object_id` (MSB) | `event_type` | `flags` | `payload` (Byte 0) | `payload` (Byte 1) | `payload` (Byte 2) | `payload` (Byte 3) |

### Component Breakdown
- **Object ID (`u16`)**: Uniquely identifies the physical component (e.g. Fader 1, Pad A, Key 32). Mapped globally via Vendor Prefix registers.
- **Event Type (`u8`)**: Mapped to the `EventType` enum:
  - `0x01` -> `AbsoluteChange` (Continuous faders)
  - `0x02` -> `RelativeDelta` (Endless rotary encoders)
  - `0x03` -> `KeyPress` (Key strike events)
  - `0x04` -> `KeyRelease` (Key release events)
  - `0x05` -> `HapticFeedback` (Bi-directional force feedback fader/key profiles)
  - `0x06` -> `VisualUpdate` (LED meters, OLED screens)
  - `0x07` -> `SystemHandshake` (Topology query and firmware config)
- **Flags (`u8`)**: Bitmask state modifiers:
  - Bit 0 (`TOUCHED`): Touch-capacitance state (active fader grab).
  - Bit 1 (`RAW_DATA`): Raw un-normalized ADC reading indicator.
  - Bit 2 (`DIRECTION`): Rotation increment direction (0 for positive, 1 for negative).
  - Bits 3-7 (`TIMER_DELTA`): 5-bit sub-sample timer offset relative to the master PLL clock domain (122.88 MHz).
- **Payload (`u32`)**: Mapped zero-copy based on the event:
  - `f32`: IEEE 754 float range `0.0..=1.0`.
  - `u16` & `u16`: Dual-axis pad coordinate data.
  - `u16` (Low 16-bits): Raw 12-bit (`0..=4095`) or 16-bit (`0..=65535`) ADC values when `RAW_DATA` is set.
  - `f32` (Haptic Intensity) & `ForceProfile` (stored in `flags` byte) when `event_type == 0x05`.

---

## 2. Unified Audio & Control Transport (UACT)
To eliminate scheduler jitter and multi-driver syncing overhead, Omid packs audio and control data into single unified DMA frames.

```
                    OMID UNIFIED PHYSICAL PIPELINE
┌────────────────────────────────────────────────────────────────────┐
│                  OMID PHYSICAL CLOCK (122.88 MHz)                  │
├─────────────────┬─────────────────┬─────────────────┬──────────────┤
│ Audio Chan 1    │ Audio Chan 2    │ ...             │ OMID Control │
│ (32-bit float)  │ (32-bit float)  │                 │ (64-bit frame)
└─────────────────┴─────────────────┴─────────────────┴──────────────┘
```

A frame with $C$ audio channels contains:
1. **Audio block ($C \times 4$ bytes):** IEEE 754 32-bit floats.
2. **Control block ($8$ bytes):** Standard 8-byte `OmidPacket`.

### Clock Synchronization
Control event offsets are measured in ticks of the master PLL clock (122.88 MHz or 112.896 MHz) relative to the start of the audio sample period. 
$$\text{Sample Offset} = \frac{\text{Timer Delta}}{\text{PLL Clock Frequency}} \times \text{Sample Rate}$$
This achieves sub-microsecond control timestamp accuracy at standard audio rates (e.g. 192kHz).

---

## 3. Parallel Dispatch Pipeline
Modern DAWs and synthesis engines process DSP operations in parallel. Omid host dispatching maps control packets to individual real-time threads using a lock-free Single-Producer Multi-Consumer (SPMC) routing ring:

```
                       INCOMING OMID BULK STREAM
                                   │
                                   ▼
                       [LOCK-FREE SPSC GENERATOR]
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         ▼                         ▼                         ▼
   [WORKER THREAD 1]         [WORKER THREAD 2]         [WORKER THREAD 3]
  (Voice Allocate 1-16)     (Voice Allocate 17-32)    (Fader Engine Feedback)
```

Worker thread indices are allocated using a lock-free hash:
$$\text{Voice ID} = \text{Object ID} \pmod{\text{Thread Count}}$$
This prevents synchronization locks and mutex bottlenecks on core sweeps. Adaptive spin-locking yields and sleeps to prevent pegging cores during idle states.

---

## 4. Platform and Wireless Driver Interface
To bypass standard OS queue delays, Omid specifies direct DMA and hardware-bypass interfaces:
- **Linux:** `io_uring` and raw `usbfs` to submit asynchronous queues directly to physical USB filesystems.
- **Windows:** WinUSB overlapped I/O mapped to I/O Completion Ports (IOCP) alongside page-locked buffers (`VirtualLock`).
- **macOS:** USBDriverKit execution pools with real-time Darwin scheduler (Thread Time Constraint policy) flags.
- **Bluetooth 5:** High-throughput GATT characteristic notifications and L2CAP Connection-Oriented Channels utilizing BLE 2M PHY.
- **WiFi:** Direct TCP/UDP socket streaming utilizing socket-level configuration options like `TCP_NODELAY`.