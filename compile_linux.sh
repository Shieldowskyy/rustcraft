#!/usr/bin/env bash
set -e

# Automatically load Rust environment (if present)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
elif [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v cargo &> /dev/null; then
    echo "==> Error: 'cargo' command not found. Please ensure Rust is installed."
    exit 1
fi

echo "==> Compiling project in release mode..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

BINARY_PATH="target/release/rustcraft"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Success! Binary built successfully."
    echo -n " Size before stripping: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Strip debug symbols
    if command -v strip &> /dev/null; then
        echo "==> Running 'strip' to remove debug symbols..."
        strip --strip-all "$BINARY_PATH"
        echo -n " Size after stripping: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    else
        echo "==> Note: 'strip' utility not found, skipping this step."
    fi

    echo "Done! Your binary is located at: $BINARY_PATH"
else
    echo "==> Error: Generated binary not found."
    exit 1
fi
