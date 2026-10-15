#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

# ============================================================================
# Build script for RustCraft on FreeBSD (x86_64)
# Target: x86_64-unknown-freebsd
# ============================================================================

export PATH="$HOME/.cargo/bin:/usr/local/bin:${PATH:-/usr/bin:/bin}"

TARGET="x86_64-unknown-freebsd"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on FreeBSD
OS=$(uname -s)
if [ "$OS" = "FreeBSD" ]; then
    echo "=== Detected native FreeBSD host ($OS $(uname -m)) ==="
    export PATH="/usr/local/bin:$HOME/.cargo/bin:${PATH:-/usr/bin:/bin}"
    export PKG_CONFIG_PATH="/usr/local/libdata/pkgconfig:/usr/local/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
    export LIBRARY_PATH="/usr/local/lib:${LIBRARY_PATH:-}"
    export RUSTFLAGS="-L /usr/local/lib -C link-arg=-Wl,-rpath,/usr/local/lib ${RUSTFLAGS:-}"
    cargo build --release
    NATIVE_BIN="target/release/${BINARY_NAME}"
    if [ -f "$NATIVE_BIN" ]; then
        if command -v strip &> /dev/null; then
            strip --strip-all "$NATIVE_BIN"
        fi
        echo "✓ Successfully built native FreeBSD binary at: $NATIVE_BIN"
        exit 0
    fi
fi

echo "=== Cross-compiling RustCraft for FreeBSD ($TARGET) ==="

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# Detect cross-linker or containerized cross tool
USE_CROSS_TOOL=0
if command -v x86_64-unknown-freebsd-gcc &> /dev/null; then
    export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="x86_64-unknown-freebsd-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="zig cc -target x86_64-freebsd"
elif command -v cross &> /dev/null; then
    echo "==> Using 'cross' (containerized cross-compilation) for $TARGET..."
    USE_CROSS_TOOL=1
else
    echo "==> Note: Cross-compiler for FreeBSD not detected in PATH."
    echo "    To cross-compile from Linux, you can install 'zig' (zero-config C cross-compiler), 'cross', or freebsd cross-gcc."
fi

export PKG_CONFIG_ALLOW_CROSS=1
# The FreeBSD cross image has an old glibc and can't run host-built build scripts cached in
# target/release, so use a dedicated target dir when building through `cross`.
if cross_available; then
    export CARGO_TARGET_DIR="target/cross-freebsd"
    BINARY_PATH="${CARGO_TARGET_DIR}/${TARGET}/release/${BINARY_NAME}"
fi
export RUSTFLAGS="-L /usr/local/lib -C link-arg=-Wl,-rpath,/usr/local/lib ${RUSTFLAGS:-}"

cross_build "$TARGET"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! FreeBSD binary built."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip symbols
    if command -v x86_64-unknown-freebsd-strip &> /dev/null; then
        echo "==> Running 'x86_64-unknown-freebsd-strip'..."
        x86_64-unknown-freebsd-strip --strip-all "$BINARY_PATH"
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
    fi

    sync_canonical_binary "$TARGET"

    echo "Done! FreeBSD binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
