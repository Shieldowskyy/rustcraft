#!/usr/bin/env bash
set -e

TARGET="x86_64-pc-windows-gnu"
BINARY_NAME="rustcraft.exe"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

if ! command -v cargo &> /dev/null; then
    echo "==> Error: 'cargo' command not found. Please ensure Rust is installed."
    exit 1
fi

echo "==> Compiling project for Windows (${TARGET})..."
cargo build --release --target "$TARGET"

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
        upx --best --lzma "$BINARY_PATH"
        echo "--------------------------------------------------"
        echo -n " Size after UPX compression: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    echo "Done! Your executable is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found at $BINARY_PATH"
    exit 1
fi
