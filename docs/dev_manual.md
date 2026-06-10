# Omid Developer Manual (v2.0.0)

This document provides developer guidelines for maintaining and expanding the Omid library.

---

## 1. Development Environment

Omid is designed to compile out-of-the-box on bare-metal architectures as well as desktop operating systems.

### Prerequisites
- Stable Rust toolchain.
- Target verification (embedded targets): `rustup target add thumbv7em-none-eabihf` (Cortex-M4/M7) or `thumbv7m-none-eabi`.

### Cargo Features
- `std` (Default): Enables thread-pool dispatching (`OmidHostDispatcher`), platform driver abstractions, socket-level networking, and string allocations.
- `bridge`: Enables standard MIDI port connections via `midir` and compiles the on-the-fly legacy bridge daemon binary (`bridge`).
- No-Features: Compiles the core serialization, packet types, and UACT parsing code in pure `#![no_std]` environment.

---

## 2. Design Rules & Guidelines

All contributions must strictly follow these constraints:

1. **Strict `no_std` Compliance for Core:** The root of the library and core models (`event`, `packet`, `queue`, `topology`, `uact`, `error`, `midi`) must not import `std` or depend on heap allocation (`alloc`). 
2. **Lock-Free Concurrency:** Mutexes, Semaphores, and other blocking primitives are forbidden in real-time execution files (e.g. `src/dispatcher.rs`). Use `SpscRingBuffer` and core atomic types with appropriate memory ordering (Relaxed, Acquire, Release) for inter-thread synchronization.
3. **Power-of-Two Ring Buffers:** The capacity of `SpscRingBuffer` must be a power of two. Bitwise masking (`& (N - 1)`) must be used instead of modulo (`% N`) to avoid CPU division penalties on DSP threads.
4. **Rust 2024 FFI Compliance:** Any platform FFI bindings (such as `pthread_setaffinity_np` for thread pinning) must reside within `unsafe extern "C"` blocks.
5. **Complete Implementation Policy:** Do not use `todo!()`, `unimplemented!()`, or panic stubs. Return `Result` or `Option` types for fallible actions.

---

## 3. Testing & Compilation

### Run Unit Tests
To test the full suite (including concurrent dispatcher, driver simulations, and MIDI translation):
```bash
cargo test --all-features
```

To test bare-metal compilation:
```bash
cargo check --no-default-features
```

### Running the Legacy Bridge
To test the legacy bridge daemon binary with the `midir` bindings:
```bash
cargo run --bin bridge --features bridge
```

### Android Package Generation
To compile shared libraries for Android (`aarch64`, `armv7`, `i686`, `x86_64`) and generate the corresponding Java Native Access (JNA) bindings automatically:
```bash
./packages/android/build_android.sh
```
Requires the Android NDK to be installed and `ANDROID_NDK_HOME` pointing to its root.

---

## 4. FFI and Multi-Language Bindings

Omid exports a universal C-API from `src/ffi.rs` to allow VST and driver developers working in other languages to build on top of the Omid standard.

### Build Artifacts
- **C Header:** Generated automatically in `include/omid.h` by `cbindgen` on compile.
- **Shared Library:** Generated as `target/debug/libomid.so` / `libomid.dylib` / `omid.dll` by Cargo (`crate-type = ["cdylib"]`).

### Exposing New Functions to FFI
Any new FFI functions must be added to `src/ffi.rs` and comply with the following rules:
1. Use `#[unsafe(no_mangle)]` for all FFI exports (mandatory in Rust 2024 edition).
2. Function signatures must use C-compatible types.
3. Explicitly wrap raw pointer dereferences or unsafe memory actions (e.g., `copy_nonoverlapping`) in `unsafe { ... }` blocks.
4. **Safety Sections**: Every `unsafe` FFI function MUST contain a `# Safety` block in its `///` documentation explaining raw pointer requirements to prevent Clippy errors.

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