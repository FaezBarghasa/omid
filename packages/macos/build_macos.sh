#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building macOS Framework for libomid..."

# Check if we are on macOS
if [ "$(uname)" != "Darwin" ]; then
    echo "WARNING: This script requires a macOS build host to run compiler and 'lipo' packaging tools natively."
    echo "Creating target build folders and instructions instead..."
fi

# Ensure targets are added
echo "Adding Apple targets..."
rustup target add x86_64-apple-darwin aarch64-apple-darwin || true

cd "$PROJECT_ROOT"

# Compile targets (this will fail on Linux host without macOS cross-compiler, so we catch it gracefully)
echo "Compiling for x86_64-apple-darwin..."
if ! cargo build --release --target x86_64-apple-darwin --all-features; then
    echo "ERROR: Compilation failed. Ensure Xcode Command Line Tools are installed or run this script on macOS."
    exit 1
fi

echo "Compiling for aarch64-apple-darwin..."
if ! cargo build --release --target aarch64-apple-darwin --all-features; then
    echo "ERROR: Compilation failed."
    exit 1
fi

# Create universal library
echo "Creating universal binary using lipo..."
mkdir -p target/universal
lipo -create \
    target/x86_64-apple-darwin/release/libomid.dylib \
    target/aarch64-apple-darwin/release/libomid.dylib \
    -output target/universal/libomid.dylib

# Structure Framework Bundle
FRAMEWORK_DIR="target/Omid.framework"
rm -rf "$FRAMEWORK_DIR"
mkdir -p "$FRAMEWORK_DIR/Versions/A/Headers"
mkdir -p "$FRAMEWORK_DIR/Versions/A/Resources"

# Copy binary & headers
cp target/universal/libomid.dylib "$FRAMEWORK_DIR/Versions/A/Omid"
cp include/omid.h "$FRAMEWORK_DIR/Versions/A/Headers/omid.h"
cp bindings/cpp/omid.hpp "$FRAMEWORK_DIR/Versions/A/Headers/omid.hpp"

# Create symlinks
cd "$FRAMEWORK_DIR"
ln -s A Versions/Current
ln -s Versions/Current/Omid Omid
ln -s Versions/Current/Headers Headers
ln -s Versions/Current/Resources Resources

# Write Info.plist
cat <<EOF > Versions/A/Resources/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>Omid</string>
    <key>CFBundleIdentifier</key>
    <string>org.omid.libomid</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Omid</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>
EOF

echo "macOS Framework bundle created successfully at target/Omid.framework"
