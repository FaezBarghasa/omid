#!/usr/bin/env bash
set -euo pipefail

# Find script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "============================================================"
echo "          OMID MULTI-OS DRIVER PACKAGING ENGINE             "
echo "============================================================"
echo "Scanning host environment capabilities..."

# Status tracking
BUILD_DEB="SKIPPED"
BUILD_ARCH="SKIPPED"
BUILD_ANDROID="SKIPPED"
BUILD_WINDOWS="SKIPPED"
BUILD_MACOS="SKIPPED"

# 1. Debian
if command -v dpkg-deb &>/dev/null; then
    echo "-> dpkg-deb detected. Building Debian package..."
    if "$SCRIPT_DIR/debian/build_deb.sh"; then
        BUILD_DEB="SUCCESS"
    else
        BUILD_DEB="FAILED"
    fi
else
    echo "-> dpkg-deb not found. Skipping Debian package."
fi

# 2. Arch Linux
if command -v makepkg &>/dev/null; then
    echo "-> makepkg detected. Building Arch package..."
    if "$SCRIPT_DIR/arch/build_arch.sh"; then
        BUILD_ARCH="SUCCESS"
    else
        BUILD_ARCH="FAILED"
    fi
else
    echo "-> makepkg not found (non-Arch host). Skipping Arch Linux package."
fi

# 3. Android
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-/home/jrad/Android/Sdk/ndk/29.0.14206865}"
if [ -d "$ANDROID_NDK_HOME" ] && command -v cargo-ndk &>/dev/null; then
    echo "-> Android NDK & cargo-ndk detected. Building Android library module..."
    if "$SCRIPT_DIR/android/build_android.sh"; then
        BUILD_ANDROID="SUCCESS"
    else
        BUILD_ANDROID="FAILED"
    fi
else
    echo "-> Android SDK/NDK or cargo-ndk missing. Skipping Android package."
fi

# 4. Windows
if command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "-> MinGW cross-compiler detected. Building Windows DLL package..."
    if "$SCRIPT_DIR/windows/build_windows.sh"; then
        BUILD_WINDOWS="SUCCESS"
    else
        BUILD_WINDOWS="FAILED"
    fi
else
    echo "-> MinGW (x86_64-w64-mingw32-gcc) not found. Skipping Windows cross-compilation."
fi

# 5. macOS
if [ "$(uname)" = "Darwin" ] && command -v lipo &>/dev/null; then
    echo "-> macOS host detected. Building macOS Framework..."
    if "$SCRIPT_DIR/macos/build_macos.sh"; then
        BUILD_MACOS="SUCCESS"
    else
        BUILD_MACOS="FAILED"
    fi
else
    echo "-> Non-macOS host or lipo missing. Skipping macOS Framework build."
fi

echo ""
echo "============================================================"
echo "                 DRIVER BUILD SUMMARY                       "
echo "============================================================"
echo "  Debian Linux Package (.deb):     $BUILD_DEB"
echo "  Arch Linux Package (pkg.tar):    $BUILD_ARCH"
echo "  Android JNI & Bindings (.aar):   $BUILD_ANDROID"
echo "  Windows Dynamic Lib (.dll/.lib): $BUILD_WINDOWS"
echo "  macOS Framework (.framework):    $BUILD_MACOS"
echo "============================================================"
echo "Completed OMID driver packaging sweep."
