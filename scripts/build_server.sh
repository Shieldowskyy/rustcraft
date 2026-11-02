#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"
source "$SCRIPT_DIR/cross_build.sh"

if [[ "$#" -gt 1 ]]; then
    echo "Usage: $0 [rust-target-triple]" >&2
    exit 2
fi

TARGET="${1:-${RUST_TARGET:-}}"
if [[ -z "$TARGET" ]]; then
    TARGET="$(rustc -vV | sed -n 's/^host: //p')"
fi
if [[ -z "$TARGET" ]]; then
    echo "Error: could not determine the Rust target triple." >&2
    exit 1
fi
if [[ ! "$TARGET" =~ ^[A-Za-z0-9_.-]+$ ]]; then
    echo "Error: invalid Rust target triple: $TARGET" >&2
    exit 2
fi

BINARY_NAME="rustcraft-server"
export BINARY_NAME
export CARGO_PROFILE_RELEASE_STRIP=symbols

if command -v rustup >/dev/null 2>&1 \
    && ! rustup target list --installed | grep -Fxq "$TARGET"; then
    rustup target add "$TARGET"
fi

if [[ "$TARGET" == *windows* ]]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    BUILD_TARGET_DIR="$CARGO_TARGET_DIR"
    if [[ "$BUILD_TARGET_DIR" != /* ]]; then
        BUILD_TARGET_DIR="$PROJECT_ROOT/$BUILD_TARGET_DIR"
    fi
else
    BUILD_TARGET_DIR="$PROJECT_ROOT/target"
fi

if [[ "$TARGET" == "$HOST_TARGET" ]]; then
    echo "==> Building headless server for host target $TARGET..."
    export CARGO_TARGET_DIR="$BUILD_TARGET_DIR"
    cargo build --release --bin rustcraft-server --target "$TARGET"
elif cross_available; then
    echo "==> Cross-compiling headless server with cross for $TARGET..."
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target/cross-$TARGET}"
    if [[ "$CARGO_TARGET_DIR" != /* ]]; then
        export CARGO_TARGET_DIR="$PROJECT_ROOT/$CARGO_TARGET_DIR"
    fi
    BUILD_TARGET_DIR="$CARGO_TARGET_DIR"
    cross build --release --bin rustcraft-server --target "$TARGET"
else
    echo "==> Cross-compiling headless server for $TARGET..."
    export CARGO_TARGET_DIR="$BUILD_TARGET_DIR"
    cargo build --release --bin rustcraft-server --target "$TARGET"
fi

BINARY_PATH="$BUILD_TARGET_DIR/$TARGET/release/$BINARY_NAME"
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: expected server executable not found: $BINARY_PATH" >&2
    exit 1
fi

DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"
PACKAGE_NAME="${SERVER_PACKAGE_NAME:-rustcraft-server-$TARGET}"
if [[ ! "$PACKAGE_NAME" =~ ^[A-Za-z0-9_.-]+$ ]]; then
    echo "Error: invalid server package name: $PACKAGE_NAME" >&2
    exit 2
fi
ARCHIVE="$DIST_DIR/$PACKAGE_NAME.tar.gz"
STAGE_DIR="$(mktemp -d "$DIST_DIR/.rustcraft-server-${TARGET}.XXXXXX")"
trap 'rm -rf "$STAGE_DIR"' EXIT

cp "$BINARY_PATH" "$STAGE_DIR/$BINARY_NAME"
cp "$PROJECT_ROOT/server.properties" "$STAGE_DIR/server.properties"
cp "$PROJECT_ROOT/install_relay_service.sh" "$STAGE_DIR/install_relay_service.sh"
tar -czf "$ARCHIVE" -C "$STAGE_DIR" "$BINARY_NAME" server.properties install_relay_service.sh

echo "Created headless server package for $TARGET: $ARCHIVE"
echo "Package contents:"
tar -tzf "$ARCHIVE"
