#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# ============================================================================
# Build script for RustCraft on Microsoft Xbox 360 (Homebrew / XEX2 / XeLL)
# Produces default.xex, rustcraft.elf, and installable RustCraft-Xbox360.zip
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

echo "=== Building RustCraft for Microsoft Xbox 360 (Homebrew API) ==="

TITLE_ID="FFFE07D1"
TITLE_NAME="RustCraft"
PACKAGE_NAME="RustCraft-Xbox360.zip"
OUT_DIR="target/xbox360/RustCraft"

mkdir -p "$OUT_DIR/saves"

CARGO_FEATURES="--features xbox360"
for arg in "$@"; do
    case "$arg" in
        --profiler)
            CARGO_FEATURES="--features xbox360,profiler"
            echo "⚡ Performance profiler enabled."
            ;;
        --help|-h)
            echo "Usage: ./scripts/build_xbox360.sh [--profiler]"
            echo "  --profiler    Enable F3 telemetry profiler on Xbox 360"
            exit 0
            ;;
        *)
            ;;
    esac
done

# ----------------------------------------------------------------------------
# 1. Check required tools
# ----------------------------------------------------------------------------
for tool in python3 zip; do
    if ! command -v "$tool" &> /dev/null; then
        echo "❌ Missing required host tool: $tool"
        exit 1
    fi
done

# ----------------------------------------------------------------------------
# 2. Compile RustCraft Core for 32-bit PowerPC Xenon & Native Launcher
# ----------------------------------------------------------------------------
echo "Compiling RustCraft engine library for PowerPC Xenon (32-bit Big-Endian)..."
cargo build --lib --release --target powerpc-unknown-linux-gnu $CARGO_FEATURES
RUST_LIB="target/powerpc-unknown-linux-gnu/release/librustcraft.a"

if [ ! -f "$RUST_LIB" ]; then
    echo "❌ Compilation failed: $RUST_LIB not found."
    exit 1
fi
echo "✓ RustCraft core engine library compiled successfully ($RUST_LIB)."

# Compile native PowerPC Big-Endian Xenon launcher (ISA 2.03) and link with Rust engine
if ! command -v clang &> /dev/null || ! command -v ld.lld &> /dev/null; then
    echo "❌ Missing required toolchain: clang and ld.lld must be installed."
    exit 1
fi

echo "Compiling freestanding PowerPC Xenon launcher and runtime shims (clang)..."
mkdir -p target/xbox360
clang -target powerpc-unknown-linux-gnu -O2 -nostdlib -ffreestanding -c xbox360/launcher.c -o target/xbox360/launcher.o

echo "Linking full RustCraft engine with Xbox 360 runtime (ld.lld)..."
PRIMARY_PAYLOAD="$OUT_DIR/rustcraft.elf"
ld.lld -m elf32ppc -T xbox360/launcher.ld target/xbox360/launcher.o "$RUST_LIB" -o "$PRIMARY_PAYLOAD"
echo "✓ Native PowerPC Xenon executable generated: $PRIMARY_PAYLOAD"

# ----------------------------------------------------------------------------
# 3. Generate XEX2 Package (default.xex) & XeLL ELF (rustcraft.elf & xenon.elf)
# ----------------------------------------------------------------------------
echo "Generating Xbox 360 XEX2 executable (default.xex) with valid PE32 container..."
python3 "$SCRIPT_DIR/make_xex.py" "$PRIMARY_PAYLOAD" \
    -o "$OUT_DIR/default.xex" \
    --title-id "$TITLE_ID" \
    --title-name "$TITLE_NAME"

# Copy as xenon.elf for automatic boot via XeLL Reloaded
cp "$PRIMARY_PAYLOAD" "$OUT_DIR/xenon.elf"
echo "✓ XeLL boot executables ready: $OUT_DIR/rustcraft.elf and $OUT_DIR/xenon.elf"

# ----------------------------------------------------------------------------
# 4. Copy Visual Assets & Metadata
# ----------------------------------------------------------------------------
echo "Copying dashboard assets and metadata..."

if [ -f "xbox360/media/icon.png" ]; then
    cp "xbox360/media/icon.png" "$OUT_DIR/icon.png"
fi

if [ -f "xbox360/media/nxeart.png" ]; then
    cp "xbox360/media/nxeart.png" "$OUT_DIR/nxeart.png"
fi

if [ -f "xbox360/title.meta" ]; then
    cp "xbox360/title.meta" "$OUT_DIR/title.meta"
fi

if [ -f "xbox360/xelllaunch.xex" ]; then
    cp "xbox360/xelllaunch.xex" "$OUT_DIR/xelllaunch.xex"
    echo "✓ Included universal XeLL launcher fallback: $OUT_DIR/xelllaunch.xex"
fi

if [ -f "xbox360/README_XBOX360.md" ]; then
    cp "xbox360/README_XBOX360.md" "$OUT_DIR/README_XBOX360.md"
fi

# ----------------------------------------------------------------------------
# 5. Create Distribution Zip Archive
# ----------------------------------------------------------------------------
echo "Creating release distribution package: $PACKAGE_NAME..."
cp "$PRIMARY_PAYLOAD" "target/xbox360/xenon.elf"
rm -f "$PACKAGE_NAME"
(cd target/xbox360 && zip -r "../../$PACKAGE_NAME" RustCraft/ xenon.elf)

echo ""
echo "============================================================================"
echo "✓ Xbox 360 Homebrew Package successfully generated!"
echo "  Output directory: $OUT_DIR"
echo "  Executables:     $OUT_DIR/default.xex (Aurora / XeXMenu / RGH / JTAG)"
echo "                   $OUT_DIR/rustcraft.elf (XeLL Reloaded boot)"
echo "  Distribution:    $PACKAGE_NAME"
echo "============================================================================"
echo ""
echo "Instrukcja uruchomienia na przerobionej konsoli (RGH / JTAG):"
echo " 1. Rozpakuj $PACKAGE_NAME na pendrive FAT32 (np. USB:\\RustCraft\\default.xex)."
echo " 2. Podłącz pendrive do konsoli Xbox 360."
echo " 3. Uruchom XeXMenu lub Aurora Dashboard, wybierz default.xex i wciśnij [A]."
echo " 4. Możesz również przetestować paczkę na PC w emulatorze Xenia Canary."
echo ""
