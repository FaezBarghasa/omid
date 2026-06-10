#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Windows Package for libomid..."

# Check if MinGW toolchain is installed
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "WARNING: MinGW cross-compiler (x86_64-w64-mingw32-gcc) not found."
    echo "To cross-compile for Windows from Linux, please install it:"
    echo "  For Debian/Ubuntu: sudo apt-get install mingw-w64"
    echo "  For Arch Linux: sudo pacman -S mingw-w64-gcc"
    echo "Exiting Windows cross-compilation runner."
    exit 0
fi

# Ensure target is installed
echo "Adding Windows target..."
rustup target add x86_64-pc-windows-gnu

cd "$PROJECT_ROOT"

# Build target
echo "Compiling for x86_64-pc-windows-gnu..."
cargo build --release --target x86_64-pc-windows-gnu --all-features

# Package structure
PKG_DIR="target/windows-package"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/bin"
mkdir -p "$PKG_DIR/lib"
mkdir -p "$PKG_DIR/include"

# Copy assets
cp target/x86_64-pc-windows-gnu/release/omid.dll "$PKG_DIR/bin/omid.dll"
cp target/x86_64-pc-windows-gnu/release/omid.dll.a "$PKG_DIR/lib/omid.lib" 2>/dev/null || \
cp target/x86_64-pc-windows-gnu/release/libomid.a "$PKG_DIR/lib/omid.lib" || true

cp include/omid.h "$PKG_DIR/include/omid.h"
cp bindings/cpp/omid.hpp "$PKG_DIR/include/omid.hpp"
cp LICENSE-MIT "$PKG_DIR/LICENSE-MIT"
cp LICENSE-APACHE "$PKG_DIR/LICENSE-APACHE"

# Zip the package if zip is available
if command -v zip &>/dev/null; then
    echo "Creating Windows package ZIP..."
    cd "target"
    zip -r "omid-windows-x64.zip" "windows-package"
    echo "Windows driver package created at target/omid-windows-x64.zip"
else
    echo "Package structured in target/windows-package/ (zip tool not found to compress)"
fi
