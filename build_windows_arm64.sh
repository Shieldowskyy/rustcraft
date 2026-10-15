#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on Windows ARM64 (Snapdragon X Elite, Surface Pro ARM, WoA)
# Target: aarch64-pc-windows-msvc / aarch64-pc-windows-gnullvm
# ============================================================================

# Automatically load Rust environment (if present)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
elif [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

TARGET="${RUST_TARGET:-aarch64-pc-windows-msvc}"
BINARY_NAME="rustcraft.exe"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET" || true
    fi
fi

echo "=== Compiling RustCraft for Windows ARM64 (${TARGET}) ==="

if ! [[ "$(uname -s)" =~ (MINGW|MSYS|CYGWIN|Windows_NT) ]] && [ "$TARGET" = "aarch64-pc-windows-msvc" ]; then
    if command -v cargo-xwin &> /dev/null; then
        echo "==> Using cargo-xwin for Windows ARM64 MSVC cross-compilation..."
        cargo xwin build --release --target "$TARGET"
    else
        echo "==> Note: Cross-compiling for Windows MSVC (aarch64-pc-windows-msvc) from Linux"
        echo "    requires 'cargo-xwin' (for MSVC CRT & Windows SDK headers)."
        echo "    Install via: cargo install cargo-xwin"
        echo "    Or cross-compile via LLVM: RUST_TARGET=aarch64-pc-windows-gnullvm ./build_windows_arm64.sh"
        cargo build --release --target "$TARGET"
    fi
else
    cargo build --release --target "$TARGET"
fi

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Windows ARM64 .exe built."
    echo -n " Size: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Optional strip if llvm-strip is available
    if command -v llvm-strip &> /dev/null; then
        echo "==> Running llvm-strip..."
        llvm-strip "$BINARY_PATH" 2>/dev/null || true
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    echo "Done! Windows ARM64 executable is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
