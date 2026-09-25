#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on Windows ARM64 (Snapdragon X Elite, Surface Pro ARM, WoA)
# Target: aarch64-pc-windows-msvc / aarch64-pc-windows-gnullvm
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

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
cargo build --release --target "$TARGET"

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
