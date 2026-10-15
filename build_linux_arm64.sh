#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on Linux ARM64 (aarch64)
# Compatible with Raspberry Pi 3/4/5 (64-bit), Orange Pi, ARM64 servers/desktops
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="aarch64-unknown-linux-gnu"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on an ARM64 machine
ARCH=$(uname -m)
if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    echo "=== Detected native Linux ARM64 host ($ARCH) ==="
    cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native ARM64 binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Cross-compiling RustCraft for Linux ARM64 ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker
if command -v aarch64-linux-gnu-gcc &> /dev/null; then
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="zig cc -target aarch64-linux-gnu"
else
    echo "==> Note: 'aarch64-linux-gnu-gcc' not found. If linking fails, install it via:"
    echo "    sudo apt install gcc-aarch64-linux-gnu (Debian/Ubuntu)"
    echo "    sudo dnf install gcc-aarch64-linux-gnu (Fedora)"
    echo "    sudo pacman -S aarch64-linux-gnu-gcc (Arch)"
fi

cross_build "$TARGET" "aarch64-linux-gnu"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Linux ARM64 binary built."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip symbols
    if command -v aarch64-linux-gnu-strip &> /dev/null; then
        echo "==> Running 'aarch64-linux-gnu-strip'..."
        aarch64-linux-gnu-strip --strip-all "$BINARY_PATH"
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
    fi

    echo "Done! Linux ARM64 binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
