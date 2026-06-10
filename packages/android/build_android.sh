#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Android Package for libomid..."

# Set Android NDK Home if not set
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-/home/jrad/Android/Sdk/ndk/29.0.14206865}"

if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "ERROR: Android NDK not found at $ANDROID_NDK_HOME"
    echo "Please set ANDROID_NDK_HOME environment variable to your NDK path."
    exit 1
fi

echo "Using Android NDK: $ANDROID_NDK_HOME"

# Install Android targets if not present
echo "Ensuring rustup targets are installed..."
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Ensure cargo-ndk is installed
if ! command -v cargo-ndk &>/dev/null; then
    echo "cargo-ndk is not installed. Installing it via cargo..."
    cargo install cargo-ndk
fi

# Create directory structure
JAVA_DIR="$SCRIPT_DIR/src/main/java/omid"
JNILIBS_DIR="$SCRIPT_DIR/src/main/jniLibs"
mkdir -p "$JAVA_DIR"
mkdir -p "$JNILIBS_DIR"

# Copy Bindings
cp "$PROJECT_ROOT/bindings/java/Omid.java" "$JAVA_DIR/Omid.java"
cp "$PROJECT_ROOT/bindings/kotlin/Omid.kt" "$JAVA_DIR/Omid.kt"

# Compile and place .so files
echo "Compiling native libraries for Android targets..."
cd "$PROJECT_ROOT"

cargo ndk -t arm64-v8a -t armeabi-v7a -t x86 -t x86_64 -o "$JNILIBS_DIR" build --release --all-features

echo "Android Library module built successfully at $SCRIPT_DIR"
echo "You can import this directory directly into Android Studio as a module, or compile it with Gradle."
