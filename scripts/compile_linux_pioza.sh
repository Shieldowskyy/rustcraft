#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"
OUTPUT_DIR="$PROJECT_ROOT/target/pioza/linux"
BINARY_PATH="$PROJECT_ROOT/target/release/rustcraft"
ZIP_PATH="$OUTPUT_DIR/Linux.zip"

if ! command -v zip >/dev/null 2>&1; then
    echo "Error: 'zip' command not found. Please install zip and try again." >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

VERSION_FILE="$OUTPUT_DIR/version.txt"
if [[ -f "$VERSION_FILE" ]]; then
    PREVIOUS_VERSION="$(<"$VERSION_FILE")"
    if [[ ! "$PREVIOUS_VERSION" =~ ^[0-9]+$ ]]; then
        echo "Error: $VERSION_FILE must contain a numeric version." >&2
        exit 1
    fi
    VERSION=$((PREVIOUS_VERSION + 1))
else
    VERSION=1
fi

echo "==> Building Linux release..."
"$SCRIPT_DIR/compile_linux.sh"

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Linux binary not found at $BINARY_PATH" >&2
    exit 1
fi

WORK_DIR="$(mktemp -d "$OUTPUT_DIR/.pioza-build.XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

printf '%s\n' "$VERSION" > "$WORK_DIR/version.txt"
printf '%s\n' "$VERSION" > "$WORK_DIR/build.txt"

echo "==> Packaging version $VERSION..."
(
    cd "$WORK_DIR"
    zip -q -9 -j "$WORK_DIR/Linux.zip" "$BINARY_PATH" version.txt
)

mv "$WORK_DIR/Linux.zip" "$ZIP_PATH"
mv "$WORK_DIR/version.txt" "$VERSION_FILE"
mv "$WORK_DIR/build.txt" "$OUTPUT_DIR/build.txt"

echo "Done!"
echo "  ZIP:     $ZIP_PATH"
echo "  Version: $VERSION_FILE ($VERSION)"
echo "  Build:   $OUTPUT_DIR/build.txt"
