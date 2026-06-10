# Omid Developer Manual (v2.0.0)

This document provides developer guidelines for maintaining and expanding the Omid library.

---

## 1. Development Environment

Omid is designed to compile out-of-the-box on bare-metal architectures as well as desktop operating systems.

### Prerequisites
- Stable Rust toolchain.
- Target verification (embedded targets): `rustup target add thumbv7em-none-eabihf` (Cortex-M4/M7).

### Cargo Features
- `std` (Default): Enables thread-pool dispatching (`OmidHostDispatcher`), platform driver abstractions, socket-level networking, and string allocations.
- No-Features: Compiles the core serialization, packet types, and UACT parsing code in pure `#![no_std]` environment.

---

## 2. Design Rules & Guidelines

All contributions must strictly follow these constraints:

1. **Strict `no_std` Compliance for Core:** The root of the library and core models (`event`, `packet`, `queue`, `topology`, `uact`) must not import `std` or depend on heap allocation (`alloc`). 
2. **Lock-Free Concurrency:** Mutexes, Semaphores, and other blocking primitives are forbidden in real-time execution files (e.g. `src/dispatcher.rs`). Use `SpscRingBuffer` and core atomic types with appropriate memory ordering (Relaxed, Acquire, Release) for inter-thread synchronization.
3. **Rust 2024 FFI Compliance:** Any platform FFI bindings (such as `pthread_setaffinity_np` for thread pinning) must reside within `unsafe extern "C"` blocks.
4. **Complete Implementation Policy:** Do not use `todo!()`, `unimplemented!()`, or panic stubs. Return `Result` or `Option` types for fallible actions.

---

## 3. Testing

### Run Unit Tests
To test the full suite (including concurrent dispatcher and driver simulations):
```bash
cargo test --all-features
```

To test bare-metal compilation:
```bash
cargo build --no-default-features
```

### Writing Tests
When implementing new drivers or core formats, write comprehensive tests in `src/lib.rs` (under the `tests` block):
- Bitwise serialization fidelity.
- Real-time SPSC queue performance under thread contention.
- Sub-sample Clock synchronization and sample-index offset precision.
- Clamping and overflow safety (e.g. clamping GATT MTUs, asserting queue saturation errors).

Example test template:
```rust
#[test]
#[cfg(feature = "std")]
fn test_custom_driver_flow() {
    let hardware = MockHardwareDriver::new();
    let driver = WifiDriver::new(hardware, true, std::string::String::from("127.0.0.1"), 9000, true);
    driver.connect();
    
    let p = OmidPacket::new_adc16(5, EventType::KeyPress, OmidFlags::new(false, false, false, 0), 65535);
    driver.submit_control(p).unwrap();
    assert_eq!(driver.poll_control().unwrap().payload_as_adc16(), 65535);
}
```