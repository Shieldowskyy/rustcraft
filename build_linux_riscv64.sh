#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on Linux RISC-V 64-bit (riscv64gc)
# Target: riscv64gc-unknown-linux-gnu
# Compatible with VisionFive 2, Milk-V Pioneer/Mars, Lichee Pi 4A, BeagleV
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="riscv64gc-unknown-linux-gnu"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on a RISC-V host
ARCH=$(uname -m)
if [ "$ARCH" = "riscv64" ]; then
    echo "=== Detected native Linux RISC-V 64 host ($ARCH) ==="
    cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native RISC-V 64 binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Cross-compiling RustCraft for Linux RISC-V 64 ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker
if command -v riscv64-linux-gnu-gcc &> /dev/null; then
    export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER="riscv64-linux-gnu-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER="zig cc -target riscv64-linux-gnu"
else
    echo "==> Note: 'riscv64-linux-gnu-gcc' not found. If linking fails, install it via:"
    echo "    sudo apt install gcc-riscv64-linux-gnu (Debian/Ubuntu)"
    echo "    sudo dnf install gcc-riscv64-linux-gnu (Fedora)"
    echo "    sudo pacman -S riscv64-linux-gnu-gcc (Arch)"
fi

cross_build "$TARGET" "riscv64-linux-gnu"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Linux RISC-V 64 binary built."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip symbols
    if command -v riscv64-linux-gnu-strip &> /dev/null; then
        echo "==> Running 'riscv64-linux-gnu-strip'..."
        riscv64-linux-gnu-strip --strip-all "$BINARY_PATH"
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
    fi

    echo "Done! Linux RISC-V 64 binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
