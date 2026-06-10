#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Arch Linux Package for libomid..."

# Check if makepkg is installed
if ! command -v makepkg &>/dev/null; then
    echo "WARNING: 'makepkg' not found on this system. You need an Arch Linux-based system to run makepkg directly."
    echo "However, the PKGBUILD is successfully created at packages/arch/PKGBUILD."
    exit 0
fi

BUILD_DIR="$PROJECT_ROOT/target/arch-build"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Copy PKGBUILD to the build dir
cp "$SCRIPT_DIR/PKGBUILD" "$BUILD_DIR/PKGBUILD"

# Create a source copy to avoid modifying the active directory during building
mkdir -p "$BUILD_DIR/src"
# Copy excluding target to save time/space
rsync -a --exclude='target' --exclude='.git' "$PROJECT_ROOT/" "$BUILD_DIR/src/omid/"

cd "$BUILD_DIR"
# Run makepkg (nodeps bypasses dependency checks if we are running standard rust outside pacman)
makepkg --nodeps --noprepare -f

# Copy the generated package back to the build dir
mv *.pkg.tar.zst "$PROJECT_ROOT/target/" 2>/dev/null || mv *.pkg.tar.xz "$PROJECT_ROOT/target/" 2>/dev/null || true

echo "Arch package built and placed in target/"
