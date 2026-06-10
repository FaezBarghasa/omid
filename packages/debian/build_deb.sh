#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Debian Package for libomid..."
cd "$PROJECT_ROOT"

# Ensure the library builds in release mode
cargo build --release --all-features

# Define target package structure
PKG_DIR="target/debian/libomid"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/lib"
mkdir -p "$PKG_DIR/usr/include"
mkdir -p "$PKG_DIR/usr/share/doc/libomid"

# Copy binary assets and headers
cp target/release/libomid.so "$PKG_DIR/usr/lib/libomid.so"
cp include/omid.h "$PKG_DIR/usr/include/omid.h"
cp bindings/cpp/omid.hpp "$PKG_DIR/usr/include/omid.hpp"
cp LICENSE-MIT "$PKG_DIR/usr/share/doc/libomid/copyright"

# Create the control configuration file
cat <<EOF > "$PKG_DIR/DEBIAN/control"
Package: libomid
Version: 0.1.0
Section: libs
Priority: optional
Architecture: amd64
Maintainer: Omid Developers <developers@omid.org>
Description: Omid (Object-MIDI) Protocol core library.
 High-performance, zero-copy, highly parallelized, bare-metal friendly
 digital music instrument protocol.
EOF

# Build package
echo "Packaging Debian archive..."
dpkg-deb --build "$PKG_DIR" "target/debian/libomid_0.1.0_amd64.deb"

echo "Debian Package created successfully at target/debian/libomid_0.1.0_amd64.deb"
