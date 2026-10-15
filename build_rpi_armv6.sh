#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on Linux ARMv6 (Raspberry Pi Zero / Zero W / Pi 1)
# Target: arm-unknown-linux-gnueabihf with ARMv6KZ VFPv2 tuning
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="arm-unknown-linux-gnueabihf"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on an ARMv6 host
ARCH=$(uname -m)
if [ "$ARCH" = "armv6l" ]; then
    echo "=== Detected native Raspberry Pi ARMv6 host ($ARCH) ==="
    RUSTFLAGS="-C target-cpu=arm1176jzf-s -C link-arg=-latomic" cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native ARMv6 binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Cross-compiling RustCraft for Raspberry Pi Zero W / ARMv6 ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker or cross runner
USE_CROSS_TOOL=0
if command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER="arm-linux-gnueabihf-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER="zig cc -target arm-linux-gnueabihf -mcpu=arm1176jzf-s"
elif command -v cross &> /dev/null; then
    echo "==> Using 'cross' (containerized cross-compilation) for $TARGET..."
    USE_CROSS_TOOL=1
else
    echo "==> Note: Neither 'arm-linux-gnueabihf-gcc', 'zig', nor 'cross' was found."
    echo "    If linking fails, install it via:"
    echo "    sudo apt install gcc-arm-linux-gnueabihf (Debian/Ubuntu)"
    echo "    or install cross: cargo install cross --git https://github.com/cross-rs/cross"
fi

# Set CPU flag specifically for ARM1176JZF-S (Raspberry Pi Zero / Zero W / 1) and link libatomic
# --unresolved-symbols: the cross image's old glibc sysroot lacks symbols (e.g. statx) that the
# newer armhf libudev references; they resolve at runtime on the Raspberry Pi.
export RUSTFLAGS="-C target-cpu=arm1176jzf-s -C link-arg=-latomic -C link-arg=-Wl,--unresolved-symbols=ignore-in-shared-libs ${RUSTFLAGS:-}"
export PKG_CONFIG_ALLOW_CROSS=1
if [ -d "/usr/lib/arm-linux-gnueabihf/pkgconfig" ]; then
    export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig:${PKG_CONFIG_PATH:-}"
fi

cross_build "$TARGET" "arm-linux-gnueabihf"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Raspberry Pi ARMv6 binary built."
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

    echo "Done! Raspberry Pi Zero W binary is located at: $BINARY_PATH"
    echo "To run on RPi Zero W: copy binary to device, ensure X11/FB environment is running, and launch ./rustcraft"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
