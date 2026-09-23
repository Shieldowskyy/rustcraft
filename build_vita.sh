#!/usr/bin/env bash
set -e

# ============================================================================
# Skrypt budujący RustCraft dla Sony PlayStation Vita (.vpk)
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

echo "=== Budowanie RustCraft dla PlayStation Vita ($TARGET) ==="

# 1. Kompilacja Rust do pliku ELF z wbudowaną biblioteką standardową
echo "Kompilacja projektu Rust dla targetu $TARGET..."
export RUSTFLAGS="-C link-arg=-Wl,--defsym=__sce_headroom=0x10000"
RUSTC_BOOTSTRAP=1 cargo build --target "$TARGET" -Z build-std=std,panic_abort --release

TARGET_ELF="target/$TARGET/release/rustcraft.elf"
if [ ! -f "$TARGET_ELF" ]; then
    TARGET_ELF="target/$TARGET/release/rustcraft"
fi
BUILD_DIR="target/vita_build"

if [ ! -f "$TARGET_ELF" ]; then
    echo "❌ Nie znaleziono pliku binarnego $TARGET_ELF"
    exit 1
fi

mkdir -p "$BUILD_DIR/sce_sys/livearea/contents"

# Kopiowanie zasobów LiveArea
if [ -d "vita/sce_sys" ]; then
    cp -r vita/sce_sys/* "$BUILD_DIR/sce_sys/"
fi

# 4. Tworzenie VELF i EBOOT.BIN
echo "Konwersja ELF -> VELF -> EBOOT.BIN..."
if command -v vita-elf-create &> /dev/null; then
    vita-elf-create "$TARGET_ELF" "$BUILD_DIR/rustcraft.velf"
    vita-make-fself -c "$BUILD_DIR/rustcraft.velf" "$BUILD_DIR/eboot.bin"
fi

# 5. Generowanie param.sfo
if command -v vita-mksfoex &> /dev/null; then
    vita-mksfoex -s TITLE_ID="$TITLE_ID" "$TITLE_NAME" "$BUILD_DIR/sce_sys/param.sfo"
fi

# 6. Pakowanie do VPK
if command -v vita-pack-vpk &> /dev/null; then
    vita-pack-vpk -s "$BUILD_DIR/sce_sys/param.sfo" -b "$BUILD_DIR/eboot.bin" \
        --add "$BUILD_DIR/sce_sys/livearea/contents/template.xml=sce_sys/livearea/contents/template.xml" \
        "$VPK_NAME"
    echo "✓ Pomyślnie utworzono paczkę: $VPK_NAME"
elif command -v zip &> /dev/null; then
    cd "$BUILD_DIR"
    zip -r "../../$VPK_NAME" eboot.bin sce_sys/
    cd ../..
    echo "✓ Pomyślnie spakowano: $VPK_NAME (zip/vpk)"
fi

echo ""
echo "=== Gotowe! ==="
echo "Aby zainstalować grę na konsoli PS Vita:"
echo " 1. Prześlij plik $VPK_NAME na konsolę (przez VitaShell: USB lub FTP do ux0:)."
echo " 2. W VitaShell zaznacz $VPK_NAME i wciśnij Krzyżyk [X], aby zainstalować grę."
echo " 3. Uruchom RustCraft bezpośrednio z menu LiveArea!"
echo ""
echo "Do testowania na PC możesz także użyć emulatora Vita3K: https://vita3k.org/"
