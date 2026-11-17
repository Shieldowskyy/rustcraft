#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# ============================================================================
# DEBUG Build script for RustCraft on Sony PlayStation Vita (.vpk)
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

# Auto-install VitaSDK (toolchain + libs) if the Vita linker is missing
if ! command -v arm-vita-eabi-gcc &> /dev/null; then
    echo "⚠ VitaSDK not found (arm-vita-eabi-gcc missing). Installing to \$HOME/vitasdk ..."
    for tool in git curl cmake make xz python3; do
        if ! command -v "$tool" &> /dev/null; then
            echo "❌ Missing required tool: $tool (install it with your package manager, e.g. sudo apt install git curl cmake make xz-utils python3 build-essential)"
            exit 1
        fi
    done
    export VITASDK="${VITASDK:-$HOME/vitasdk}"
    VDPM_DIR="$(mktemp -d)/vdpm"
    git clone --depth 1 https://github.com/vitasdk/vdpm "$VDPM_DIR"
    (cd "$VDPM_DIR" && ./bootstrap-vitasdk.sh)
    export PATH="$VITASDK/bin:$PATH"
    if ! command -v arm-vita-eabi-gcc &> /dev/null; then
        echo "❌ VitaSDK installation failed. See https://vitasdk.org/ for manual setup."
        exit 1
    fi
    echo "✓ VitaSDK installed at $VITASDK"
fi

TARGET="armv7-sony-vita-newlibeabihf"
TITLE_ID="RUSTCRAFT"
TITLE_NAME="RustCraft [DEBUG]"
VPK_NAME="RustCraft_debug.vpk"
DEBUG_ELF_OUT="RustCraft_debug_symbols.elf"

CARGO_FEATURES=""
for arg in "$@"; do
    case "$arg" in
        --profiler)
            CARGO_FEATURES="--features profiler"
            echo "⚡ Performance profiler enabled by default in Vita build."
            ;;
        --help|-h)
            echo "Usage: ./scripts/build_vita_debug.sh [--profiler]"
            echo "  --profiler    Enable advanced Minecraft-style F3 profiler & telemetry by default on Vita"
            exit 0
            ;;
        *)
            ;;
    esac
done

echo "=== Building RustCraft (DEBUG + SYMBOLS) for PlayStation Vita ($TARGET) ==="

# 1. Compile Rust to ELF in DEV (debug) mode with full debug symbols
echo "Compiling Rust project in debug mode for target $TARGET..."

# Wymuszenie pełnych symboli debugowania (DWARF) oraz braku strippowania, niezależnie od Cargo.toml
export CARGO_PROFILE_DEV_DEBUG=2
export CARGO_PROFILE_DEV_STRIP="none"
export CARGO_PROFILE_DEV_OPT_LEVEL=1 # Lekka optymalizacja (1), żeby gra na Vicie nie działała w 2 FPS, przy zachowaniu pełnych symboli

# Flagi kompilatora: dodajemy -C debuginfo=2 oraz eksport symboli dynamicznych dla lepszych backtrace'ów
export RUSTFLAGS="-C target-cpu=cortex-a9 -C debuginfo=2 -C force-frame-pointers=yes -C link-arg=-Wl,--export-dynamic -C link-arg=-Wl,--defsym=__sce_headroom=0x10000 -C link-arg=-lSceIme_stub -C link-arg=-lSceCommonDialog_stub -C link-arg=-lSceAppUtil_stub -C link-arg=-lSceAudio_stub"

# Budujemy bez flagi --release (czyli profil dev)
RUSTC_BOOTSTRAP=1 cargo build --target "$TARGET" -Z build-std=std,panic_abort $CARGO_FEATURES

TARGET_ELF="target/$TARGET/debug/rustcraft.elf"
if [ ! -f "$TARGET_ELF" ]; then
    TARGET_ELF="target/$TARGET/debug/rustcraft"
fi
BUILD_DIR="target/vita_build_debug"

if [ ! -f "$TARGET_ELF" ]; then
    echo "❌ Binary not found at $TARGET_ELF"
    exit 1
fi

# Zapisujemy kopię ELF z pełnymi symbolami obok VPK (vita-elf-create wycina symbole przy tworzeniu eboot.bin!)
cp "$TARGET_ELF" "$DEBUG_ELF_OUT"
echo "✓ Saved unstripped ELF with debug symbols to: $DEBUG_ELF_OUT"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/sce_sys/livearea/contents"

# Copy LiveArea assets
if [ -d "vita/sce_sys" ]; then
    cp -r vita/sce_sys/* "$BUILD_DIR/sce_sys/"
fi

# Copy soundtracks and mods to build directory
if [ -d "soundtracks" ]; then
    echo "Copying soundtracks to $BUILD_DIR/soundtracks..."
    cp -r soundtracks "$BUILD_DIR/"
fi
if [ -d "mods" ]; then
    echo "Copying mods to $BUILD_DIR/mods..."
    cp -r mods "$BUILD_DIR/"
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
ADD_ARGS=()
if [ -f "$BUILD_DIR/sce_sys/livearea/contents/template.xml" ]; then
    ADD_ARGS+=(--add "$BUILD_DIR/sce_sys/livearea/contents/template.xml=sce_sys/livearea/contents/template.xml")
fi
if [ -d "$BUILD_DIR/soundtracks" ]; then
    ADD_ARGS+=(--add "$BUILD_DIR/soundtracks=soundtracks")
fi
if [ -d "$BUILD_DIR/mods" ]; then
    ADD_ARGS+=(--add "$BUILD_DIR/mods=mods")
fi

if command -v vita-pack-vpk &> /dev/null; then
    vita-pack-vpk -s "$BUILD_DIR/sce_sys/param.sfo" -b "$BUILD_DIR/eboot.bin" \
        "${ADD_ARGS[@]}" \
        "$VPK_NAME"
    echo "✓ Debug package created successfully: $VPK_NAME"
elif command -v zip &> /dev/null; then
    cd "$BUILD_DIR"
    zip -r "../../$VPK_NAME" eboot.bin sce_sys/ soundtracks/ mods/
    cd ../..
    echo "✓ Debug packaged successfully: $VPK_NAME (zip/vpk)"
fi

echo ""
echo "=== Done! ==="
echo "Debug VPK: $VPK_NAME"
echo "Symbols ELF: $DEBUG_ELF_OUT (KEEP THIS FILE to decode crash dumps!)"
echo ""
echo "How to get useful crash logs on PS Vita:"
echo " 1. Install 'vitacompanion' or 'PrincessLog' (taiHEN plugin) on your Vita to see stdout/stderr over Wi-Fi/USB."
echo " 2. If the game crashes with an error code (e.g. C2-12828-1), grab the .psp2dmp file from ux0:data/."
echo " 3. Decode the crash dump using VitaSDK:"
echo "    vita-parse-core ux0_crash.psp2dmp $DEBUG_ELF_OUT"
echo "    OR lookup a specific crash address (PC/LR) directly:"
echo "    arm-vita-eabi-addr2line -e $DEBUG_ELF_OUT -f -C -p <ADRES_Z_CRASHU>"