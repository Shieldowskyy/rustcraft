#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on macOS (Apple Silicon & Intel)
# Targets:
#   - aarch64-apple-darwin (Apple Silicon: M1 / M2 / M3 / M4)
#   - x86_64-apple-darwin  (Intel Macs)
#   - universal            (Universal 2 Binary combining both architectures)
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"
source "$SCRIPT_DIR/cross_build.sh"

export PATH="$HOME/.cargo/bin:/usr/local/bin:/opt/homebrew/bin:${PATH:-/usr/bin:/bin}"

# Determine target architecture from argument or host
USER_ARG="${1:-}"
case "$USER_ARG" in
    aarch64|arm64|aarch64-apple-darwin)
        TARGET="aarch64-apple-darwin"
        IS_UNIVERSAL=0
        ;;
    x86_64|amd64|intel|x86_64-apple-darwin)
        TARGET="x86_64-apple-darwin"
        IS_UNIVERSAL=0
        ;;
    universal|all)
        TARGET="universal"
        IS_UNIVERSAL=1
        ;;
    "")
        if [ "$(uname -s)" = "Darwin" ]; then
            HOST_ARCH=$(uname -m)
            if [ "$HOST_ARCH" = "arm64" ]; then
                TARGET="aarch64-apple-darwin"
            else
                TARGET="x86_64-apple-darwin"
            fi
        else
            # Default to modern Apple Silicon when cross-compiling
            TARGET="aarch64-apple-darwin"
        fi
        IS_UNIVERSAL=0
        ;;
    *)
        TARGET="$USER_ARG"
        IS_UNIVERSAL=0
        ;;
esac

BINARY_NAME="rustcraft"

ensure_rustup_target() {
    local t="$1"
    if command -v rustup &> /dev/null; then
        if ! rustup target list --installed | grep -q "$t"; then
            echo "==> Adding target $t via rustup..."
            rustup target add "$t" || true
        fi
    fi
}

build_single_target() {
    local tgt="$1"
    local bin_path="target/${tgt}/release/${BINARY_NAME}"

    echo "=== Compiling RustCraft for macOS (${tgt}) ==="
    ensure_rustup_target "$tgt"

    # Configure cross-linker if compiling from Linux/non-Darwin host
    if [ "$(uname -s)" != "Darwin" ] && ! cross_available; then
        if [[ "$tgt" == aarch64* ]]; then
            if command -v aarch64-apple-darwin-clang &> /dev/null; then
                export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="aarch64-apple-darwin-clang"
            elif command -v oa64-clang &> /dev/null; then
                export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="oa64-clang"
            elif command -v zig &> /dev/null; then
                export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="zig cc -target aarch64-macos"
            fi
        elif [[ "$tgt" == x86_64* ]]; then
            if command -v x86_64-apple-darwin-clang &> /dev/null; then
                export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="x86_64-apple-darwin-clang"
            elif command -v o64-clang &> /dev/null; then
                export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="o64-clang"
            elif command -v zig &> /dev/null; then
                export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="zig cc -target x86_64-macos"
            fi
        fi
    fi

    cross_build "$tgt"

    if [ ! -f "$bin_path" ]; then
        echo "==> Error: Generated binary not found at $bin_path"
        exit 1
    fi

    strip_binary "$bin_path"
    sync_canonical_binary "$tgt"
}

create_app_bundle() {
    local src_bin="$1"
    local bundle_dir="$2"
    local contents_dir="${bundle_dir}/Contents"
    local macos_dir="${contents_dir}/MacOS"
    local res_dir="${contents_dir}/Resources"

    mkdir -p "$macos_dir" "$res_dir"
    cp -f "$src_bin" "${macos_dir}/${BINARY_NAME}"
    chmod +x "${macos_dir}/${BINARY_NAME}"

    # Generate Info.plist
    cat > "${contents_dir}/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>rustcraft</string>
    <key>CFBundleIdentifier</key>
    <string>com.rustcraft.rustcraft</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>RustCraft</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

    # Copy controls.json if present
    if [ -f "$PROJECT_ROOT/controls.json" ]; then
        cp -f "$PROJECT_ROOT/controls.json" "$res_dir/"
    fi
}

mkdir -p "$PROJECT_ROOT/dist"

if [ "$IS_UNIVERSAL" -eq 1 ]; then
    echo "=== Building Universal 2 macOS Binary (aarch64 + x86_64) ==="
    build_single_target "aarch64-apple-darwin"
    build_single_target "x86_64-apple-darwin"

    UNIVERSAL_DIR="$PROJECT_ROOT/target/universal-apple-darwin/release"
    UNIVERSAL_BIN="${UNIVERSAL_DIR}/${BINARY_NAME}"
    mkdir -p "$UNIVERSAL_DIR"

    LIPO_TOOL="lipo"
    if ! command -v lipo &> /dev/null && command -v llvm-lipo &> /dev/null; then
        LIPO_TOOL="llvm-lipo"
    fi

    if command -v "$LIPO_TOOL" &> /dev/null; then
        echo "==> Creating Universal binary with $LIPO_TOOL..."
        "$LIPO_TOOL" -create \
            "$PROJECT_ROOT/target/aarch64-apple-darwin/release/${BINARY_NAME}" \
            "$PROJECT_ROOT/target/x86_64-apple-darwin/release/${BINARY_NAME}" \
            -output "$UNIVERSAL_BIN"
    else
        echo "==> Note: 'lipo' not found on this system. Both aarch64 and x86_64 binaries were built separately."
        UNIVERSAL_BIN="$PROJECT_ROOT/target/aarch64-apple-darwin/release/${BINARY_NAME}"
    fi

    FINAL_BIN="$UNIVERSAL_BIN"
    OUT_NAME="rustcraft-macos-universal"
else
    build_single_target "$TARGET"
    FINAL_BIN="$PROJECT_ROOT/target/${TARGET}/release/${BINARY_NAME}"
    OUT_NAME="rustcraft-macos-${TARGET}"
fi

echo "--------------------------------------------------"
echo " Success! macOS binary ready at: $FINAL_BIN"
echo -n " Binary size: "
du -h "$FINAL_BIN" | awk '{print $1}'
echo "--------------------------------------------------"

# Create App bundle in dist
APP_BUNDLE="$PROJECT_ROOT/dist/RustCraft.app"
create_app_bundle "$FINAL_BIN" "$APP_BUNDLE"

# Package archive for distribution
DIST_ARCHIVE="$PROJECT_ROOT/dist/${OUT_NAME}.tar.gz"
tar -czf "$DIST_ARCHIVE" -C "$PROJECT_ROOT/dist" "RustCraft.app"
echo "==> Distribution App bundle packaged at: $DIST_ARCHIVE"
if command -v zip &> /dev/null; then
    (cd "$PROJECT_ROOT/dist" && zip -q -r "${OUT_NAME}.zip" "RustCraft.app")
    echo "==> Distribution ZIP archive at: $PROJECT_ROOT/dist/${OUT_NAME}.zip"
fi
echo "Done! macOS build completed successfully."
