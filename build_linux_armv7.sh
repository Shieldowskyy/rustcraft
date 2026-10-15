#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on Linux ARMv7 (32-bit Hard Float)
# Target: armv7-unknown-linux-gnueabihf
# Compatible with Raspberry Pi 2/3 (32-bit), BeagleBone Black, retro handhelds
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="armv7-unknown-linux-gnueabihf"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on an ARMv7 host
ARCH=$(uname -m)
if [ "$ARCH" = "armv7l" ] || [ "$ARCH" = "armv7" ]; then
    echo "=== Detected native Linux ARMv7 host ($ARCH) ==="
    cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native ARMv7 binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Cross-compiling RustCraft for Linux ARMv7 ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker
if command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="arm-linux-gnueabihf-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="zig cc -target arm-linux-gnueabihf -mcpu=generic"
else
    echo "==> Note: 'arm-linux-gnueabihf-gcc' not found. If linking fails, install it via:"
    echo "    sudo apt install gcc-arm-linux-gnueabihf (Debian/Ubuntu)"
    echo "    sudo dnf install gcc-arm-linux-gnu (Fedora)"
    echo "    sudo pacman -S arm-linux-gnueabihf-gcc (Arch)"
fi

cross_build "$TARGET" "arm-linux-gnueabihf"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Linux ARMv7 binary built."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip symbols
    if command -v arm-linux-gnueabihf-strip &> /dev/null; then
        echo "==> Running 'arm-linux-gnueabihf-strip'..."
        arm-linux-gnueabihf-strip --strip-all "$BINARY_PATH"
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
    fi

    sync_canonical_binary "$TARGET"

    echo "Done! Linux ARMv7 binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
