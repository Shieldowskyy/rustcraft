#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on Linux 32-bit x86 (i686)
# Target: i686-unknown-linux-gnu
# Compatible with legacy 32-bit PCs, netbooks (Intel Atom), old laptops
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="i686-unknown-linux-gnu"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on a 32-bit x86 host
ARCH=$(uname -m)
if [ "$ARCH" = "i686" ] || [ "$ARCH" = "i386" ]; then
    echo "=== Detected native Linux 32-bit host ($ARCH) ==="
    cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native 32-bit binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Compiling RustCraft for Linux 32-bit x86 ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker or multilib gcc
if command -v gcc &> /dev/null; then
    export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER="gcc"
    export RUSTFLAGS="-C link-arg=-m32"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER="zig cc -target i386-linux-gnu"
else
    echo "==> Note: gcc with 32-bit multilib support not found. Install via:"
    echo "    sudo apt install gcc-multilib g++-multilib (Debian/Ubuntu)"
    echo "    sudo dnf install glibc-devel.i686 libgcc.i686 (Fedora)"
    echo "    sudo pacman -S lib32-gcc-libs (Arch)"
fi

cross_build "$TARGET" "i386-linux-gnu"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Linux 32-bit x86 binary built."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip symbols
    if command -v strip &> /dev/null; then
        echo "==> Running 'strip'..."
        strip --strip-all "$BINARY_PATH" || true
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    echo "Done! Linux 32-bit binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
