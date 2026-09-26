#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on Sony PlayStation Vita (.vpk)
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

if [ -z "$VITASDK" ]; then
    if [ -d "/usr/local/vitasdk" ]; then
        export VITASDK="/usr/local/vitasdk"
    elif [ -d "$HOME/vitasdk" ]; then
        export VITASDK="$HOME/vitasdk"
    fi
fi

if [ -n "$VITASDK" ] && [ -d "$VITASDK/bin" ]; then
    export PATH="$VITASDK/bin:$PATH"
fi

TARGET="armv7-sony-vita-newlibeabihf"
TITLE_ID="RUSTCRAFT"
TITLE_NAME="RustCraft"
VPK_NAME="RustCraft.vpk"

CARGO_FEATURES=""
for arg in "$@"; do
    case "$arg" in
        --profiler)
            CARGO_FEATURES="--features profiler"
            echo "⚡ Performance profiler enabled by default in Vita build."
            ;;
        --help|-h)
            echo "Usage: ./build_vita.sh [--profiler]"
            echo "  --profiler    Enable advanced Minecraft-style F3 profiler & telemetry by default on Vita"
            exit 0
            ;;
        *)
            ;;
    esac
done

echo "=== Building RustCraft for PlayStation Vita ($TARGET) ==="

# 1. Compile Rust to ELF with built-in standard library
echo "Compiling Rust project for target $TARGET..."
export RUSTFLAGS="-C target-cpu=cortex-a9 -C link-arg=-Wl,--defsym=__sce_headroom=0x10000"
RUSTC_BOOTSTRAP=1 cargo build --target "$TARGET" -Z build-std=std,panic_abort --release $CARGO_FEATURES

TARGET_ELF="target/$TARGET/release/rustcraft.elf"
if [ ! -f "$TARGET_ELF" ]; then
    TARGET_ELF="target/$TARGET/release/rustcraft"
fi
BUILD_DIR="target/vita_build"

if [ ! -f "$TARGET_ELF" ]; then
    echo "❌ Binary not found at $TARGET_ELF"
    exit 1
fi

mkdir -p "$BUILD_DIR/sce_sys/livearea/contents"

# Copy LiveArea assets
if [ -d "vita/sce_sys" ]; then
    cp -r vita/sce_sys/* "$BUILD_DIR/sce_sys/"
fi

# 4. Create VELF and EBOOT.BIN
echo "Converting ELF -> VELF -> EBOOT.BIN..."
if command -v vita-elf-create &> /dev/null; then
    vita-elf-create "$TARGET_ELF" "$BUILD_DIR/rustcraft.velf"
    vita-make-fself -c "$BUILD_DIR/rustcraft.velf" "$BUILD_DIR/eboot.bin"
fi

# 5. Generate param.sfo
if command -v vita-mksfoex &> /dev/null; then
    vita-mksfoex -s TITLE_ID="$TITLE_ID" "$TITLE_NAME" "$BUILD_DIR/sce_sys/param.sfo"
fi

# 6. Package into VPK
if command -v vita-pack-vpk &> /dev/null; then
    vita-pack-vpk -s "$BUILD_DIR/sce_sys/param.sfo" -b "$BUILD_DIR/eboot.bin" \
        --add "$BUILD_DIR/sce_sys/livearea/contents/template.xml=sce_sys/livearea/contents/template.xml" \
        "$VPK_NAME"
    echo "✓ Package created successfully: $VPK_NAME"
elif command -v zip &> /dev/null; then
    cd "$BUILD_DIR"
    zip -r "../../$VPK_NAME" eboot.bin sce_sys/
    cd ../..
    echo "✓ Packaged successfully: $VPK_NAME (zip/vpk)"
fi

echo ""
echo "=== Done! ==="
echo "To install on your PS Vita console:"
echo " 1. Transfer $VPK_NAME to the console (via VitaShell: USB or FTP to ux0:)."
echo " 2. In VitaShell, select $VPK_NAME and press Cross [X] to install."
echo " 3. Launch RustCraft directly from the LiveArea menu!"
echo ""
echo "For PC testing you can also use the Vita3K emulator: https://vita3k.org/"
