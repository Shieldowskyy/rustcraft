#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on FreeBSD (x86_64)
# Target: x86_64-unknown-freebsd
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="x86_64-unknown-freebsd"
BINARY_NAME="rustcraft"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Detect if we are building natively on FreeBSD
OS=$(uname -s)
if [ "$OS" = "FreeBSD" ]; then
    echo "=== Detected native FreeBSD host ($OS $(uname -m)) ==="
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

# Detect cross-linker
if command -v x86_64-unknown-freebsd-gcc &> /dev/null; then
    export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="x86_64-unknown-freebsd-gcc"
elif command -v zig &> /dev/null; then
    export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="zig cc -target x86_64-freebsd"
elif command -v clang &> /dev/null; then
    export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="clang"
    export RUSTFLAGS="-C link-arg=--target=x86_64-unknown-freebsd"
else
    echo "==> Note: Cross-compiler for FreeBSD not detected in PATH."
    echo "    To cross-compile from Linux, you can install 'zig' (zero-config C cross-compiler) or freebsd cross-gcc."
fi

cargo build --release --target "$TARGET"

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

    echo "Done! FreeBSD binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
