# Omid Diagrams (v2.0.0)

This document visualizes the Omid Version 2.0.0 architecture using Mermaid diagrams.

---

## 1. Unified OmidPacket Structure

```mermaid
classDiagram
    class OmidPacket {
        +u16 object_id
        +u8 event_type
        +u8 flags
        +u32 payload
        +new_haptic(object_id, profile, intensity) OmidPacket
        +new_adc12(object_id, event_type, flags, val) OmidPacket
        +new_adc16(object_id, event_type, flags, val) OmidPacket
        +haptic_force_profile() Result~ForceProfile, u8~
        +haptic_intensity() f32
        +payload_as_adc12() u16
        +payload_as_adc16() u16
        +payload_as_normalized_f32(adc_bits) f32
    }
    class ForceProfile {
        <<enumeration>>
        Unknown
        HammerStrike
        SpringTension
        KineticDampening
    }
    OmidPacket ..> ForceProfile : uses
```

---

## 2. UACT Streaming Pipeline & Clock Sync

This sequence demonstrates zero-copy audio/control interleaving and sample-accurate synchronization.

```mermaid
sequenceDiagram
    participant Hardware
    participant DMA Buffer
    participant UactDemuxer
    participant ClockSynchronizer
    participant DSP Engine

    Hardware->>DMA Buffer: Write UactFrame (Audio Chan 1..C + 8-byte OmidPacket)
    DMA Buffer->>UactDemuxer: Stream bytes (process_bytes)
    UactDemuxer->>UactDemuxer: Extract Audio and Control (Circular Buffer)
    UactDemuxer->>ClockSynchronizer: Get subsample_offset (timer_delta)
    ClockSynchronizer->>ClockSynchronizer: Convert ticks to fractional sample offset
    UactDemuxer->>DSP Engine: Dispatch Audio & sample-accurate Control Event
```

---

## 3. Lock-Free Host Parallel Dispatcher

Visualizing the routing of control events to isolated, pinned cores without mutex bottlenecks.

```mermaid
graph TD
    IncomingStream[Incoming Omid Control Packets] --> CoreReceiver[Core 1: Core Receiver]
    CoreReceiver --> Dispatcher[Core 2: Host Dispatcher]
    
    subgraph Parallel Worker Pool
        Dispatcher -->|Voice ID = ID % 3 == 0| SpscQ0[(SPSC Ring Buffer 0)]
        Dispatcher -->|Voice ID = ID % 3 == 1| SpscQ1[(SPSC Ring Buffer 1)]
        Dispatcher -->|Voice ID = ID % 3 == 2| SpscQ2[(SPSC Ring Buffer 2)]
        
        SpscQ0 -->|Pin: Core 3| Worker0[DSP Worker Thread 0]
        SpscQ1 -->|Pin: Core 4| Worker1[DSP Worker Thread 1]
        SpscQ2 -->|Pin: Core 5| Worker2[DSP Worker Thread 2]
    end

    Worker0 --> Out[Synthesized Audio Stream]
    Worker1 --> Out
    Worker2 --> Out
```

---

## 4. IoT Connection Pipelines (BLE 5 & WiFi)

```mermaid
flowchart LR
    subgraph IoT Device
        Sensor[Fader / Key ADC] --> construction[OmidPacket Construction]
    end

    subgraph Transport Channel
        construction -->|BLE 5: GATT / L2CAP CoC| BleDrv[BleDriver]
        construction -->|WiFi: TCP/UDP Socket| WifiDrv[WifiDriver]
    end

    subgraph Host Application
        BleDrv -->|OmidDriver| CoreQueue[(SpscRingBuffer)]
        WifiDrv -->|OmidDriver| CoreQueue
        CoreQueue --> HostDsp[Real-time DSP Processing]
    end
```

---

## 5. Bidirectional Loop & Dispatcher Callbacks

Demonstrates how parameter updates sync back and forth between VST GUI/DSP and physical hardware.

```mermaid
sequenceDiagram
    participant Hardware Knob
    participant OmidHostDispatcher
    participant VST DSP Callback
    participant VST GUI / Automation
    participant Driver TX Queue
    participant Motorized Fader / LED

    Hardware Knob->>OmidHostDispatcher: Incoming Control Packet (Knob rotated)
    OmidHostDispatcher->>VST DSP Callback: Trigger Callback Registry (e.g. obj_id = 42)
    VST DSP Callback->>Driver TX Queue: Push response (echo fader LED update)
    VST GUI / Automation->>Driver TX Queue: Submit packet (User slides GUI fader)
    Driver TX Queue->>Motorized Fader / LED: Stream packet to Hardware
```

---

## 6. SpscRingBuffer Cache-Line Memory Layout

Prevents CPU core cache bouncing (false sharing) by isolating indices to different 64-byte boundaries.

```mermaid
graph LR
    subgraph Cache Line 1 (64 Bytes)
        buf[Data Buffer Array]
    end
    subgraph Cache Line 2 (64 Bytes)
        write_idx[write_idx: AtomicUsize]
    end
    subgraph Cache Line 3 (64 Bytes)
        read_idx[read_idx: AtomicUsize]
    end
```