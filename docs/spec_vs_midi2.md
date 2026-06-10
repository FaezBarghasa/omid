# Omid Protocol 2.0.0 vs MIDI 2.0 Specification

This document clarifies why the Omid (Object-MIDI) Protocol 2.0.0 was developed, its underlying design philosophy, and how it differs from the official MIDI 2.0 specification defined by the MIDI Association.

## 1. Why Omid? The Need for a Unified Transport

In modern electronic music production and performance:
- Hardware controllers contain high-resolution analog-to-digital converters (ADCs) for faders, keys, and pads (typically 12-bit to 16-bit).
- Studio environments require ultra-low-latency, zero-jitter communication.
- Software synthesizers (VST3, CLAP, AU) run on highly parallelized, multithreaded systems.

Traditional systems separate control data (MIDI) and audio data (PCM) at both the hardware driver and transport layers. Even with MIDI 2.0, control messages are sent over a separate USB endpoint or transport channel than the audio monitor signals, leading to phase drift, buffer jitter, and complex synchronization logic.

Omid introduces the **Unified Audio & Control Transport (UACT)**. Under this standard, audio samples and control packets are multiplexed into a single, synchronous byte stream sharing a **single clock domain and physical clock source**.

---

## 2. Core Differences: Omid 2.0.0 vs MIDI 2.0

| Feature | MIDI 2.0 | Omid 2.0.0 (UACT) |
| :--- | :--- | :--- |
| **Transport Layer** | Channel-centric, separated from audio. Uses USB MIDI Class, Network MIDI, etc. | **Unified Audio & Control (UACT)**. Audio and control share the same physical pipeline and clock. |
| **Addressing Model** | 16 Groups $\times$ 16 Channels (up to 256 channels). | Flat **16-bit Object ID space** (up to 65,536 independent physical nodes/keys/faders per stream). |
| **Jitter & Sync** | Jitter-reduction timestamps (16-bit ticks), but subject to buffer alignment differences from the audio stream. | **Deterministic Sample-Accurate Sync**. Sub-sample timer delta is aligned directly against the multiplexed audio samples. |
| **Concurrency** | Serialized message processing queues per device port. | **SPMC (Single Producer Multiple Consumer) Lock-free Routing** mapped to CPU affinity threads. |
| **Haptic Feedback** | Standard MIDI CC/SysEx mappings (arbitrary/non-standardized vendor formats). | Native, dedicated **Haptic Feedback Packet** supporting defined Force Profiles (Spring, Friction, etc.) directly in the protocol. |
| **Hardware Fit** | Designed as an application-level message format. | Designed to be **DMA-friendly** for raw ADC (12/16-bit) and bare-metal (`#![no_std]`) firmware integration. |

---

## 3. How Omid Improves Performance

### Sample-Accurate Event Timestamping
In Omid, a `UactFrame` contains both audio samples and control packets. If a fader moves or a key is pressed, its sub-sample timer offset (representing CPU/MCU clock ticks since the start of the current audio block) is embedded directly inside the control packet. 

Because the host receives the control packet *inside* the same DMA buffer block as the accompanying audio:
1. The host knows the exact audio sample index at which the control event occurred.
2. There is **zero phase drift** between the controller's hardware clock and the host's audio interface clock.

### Flat Object Model
Instead of routing messages through complex hierarchies of Group, Channel, Controller Number, and Parameter Number:
- Every physical fader, capacitive touch strip, or key is assigned a unique `object_id` (0..=65535).
- Every event is a direct transaction on that `object_id`, greatly simplifying host-side lookup tables.
- This maps directly to VST3/CLAP parameter IDs, allowing zero-overhead routing from hardware to plugin parameters.
