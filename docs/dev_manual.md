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

---

## 4. FFI and Multi-Language Bindings

Omid exports a universal C-API from `src/ffi.rs` to allow VST and driver developers working in other languages to build on top of the Omid standard.

### Build Artifacts
- **C Header:** Generated automatically in `include/omid.h` by `cbindgen` on compile.
- **Shared Library:** Generated as `target/debug/libomid.so` / `libomid.dylib` / `omid.dll` by Cargo (`crate-type = ["cdylib"]`).

### Exposing New Functions to FFI
Any new FFI functions must be added to `src/ffi.rs` and comply with the following rules:
1. Use `#[unsafe(no_mangle)]` for all FFI exports (mandatory in Rust 2024 edition).
2. Function signatures must use C-compatible types (e.g., `*const u8`, `*mut u16`, primitive types, or `#[repr(C)]` structs like `OmidPacket` and `TopologyDescriptor`).
3. Explicitly wrap raw pointer dereferences or unsafe memory actions (e.g., `copy_nonoverlapping`) in `unsafe { ... }` blocks, even inside `unsafe fn` declarations.
4. Ensure the return value is FFI-safe (avoid Rust-specific enums/structs without `#[repr(C)]` or `#[repr(u8)]`).

### Generating Bindings
The bindings directory structure is organized as:
- `bindings/cpp/`: C++ header wrapper (`omid.hpp`).
- `bindings/go/`: Go cgo package.
- `bindings/python/`: Python `ctypes` wrapper.
- `bindings/typescript/`: TypeScript type mappings & Bun/Deno FFI loaders.
- `bindings/dart/`: Dart `dart:ffi` bindings.
- `bindings/csharp/`: C# P/Invoke mappings.
- `bindings/java/`: Java JNA library.
- `bindings/kotlin/`: Kotlin JNA wrapper.