#!/usr/bin/env bash
set -e

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/scripts/cross_build.sh"

export PATH="$HOME/.cargo/bin:$PATH"

TARGET="x86_64-pc-windows-gnu"
BINARY_NAME="rustcraft.exe"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Check if target is installed in rustup
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET" || true
    fi
fi

# Detect cross-linker when not using containerized cross
if ! cross_available; then
    if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"
    elif command -v zig &> /dev/null; then
        export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="zig cc -target x86_64-windows-gnu"
    fi
fi

echo "==> Compiling project for Windows (${TARGET})..."
cross_build "$TARGET"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! .exe binary built successfully."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip debug symbols using MinGW cross-strip tool
    STRIP_TOOL="x86_64-w64-mingw32-strip"
    if command -v "$STRIP_TOOL" &> /dev/null; then
        echo "==> Running '$STRIP_TOOL'..."
        "$STRIP_TOOL" --strip-all "$BINARY_PATH"
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        echo "==> Using default 'strip'..."
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
        echo -n " Size after strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    # UPX compression
    if command -v upx &> /dev/null; then
        echo "==> UPX tool detected. Compressing .exe binary..."
        upx --best --lzma "$BINARY_PATH" 2>/dev/null || true
        echo "--------------------------------------------------"
        echo -n " Size after UPX compression: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    sync_canonical_binary "$TARGET"

    echo "Done! Your executable is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
