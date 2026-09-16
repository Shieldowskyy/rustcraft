#!/usr/bin/env bash
set -e

TARGET="x86_64-pc-windows-gnu"
BINARY_NAME="rustcraft.exe"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

if ! command -v cargo &> /dev/null; then
    echo "==> Błąd: Nie znaleziono polecenia 'cargo'. Upewnij się, że Rust jest zainstalowany."
    exit 1
fi

echo "==> Kompilowanie projektu pod Windowsa (${TARGET})..."
cargo build --release --target "$TARGET"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Sukces! Plik .exe został zbudowany."
    echo -n " Rozmiar przed odchudzaniem: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Usuwanie symboli debugowania za pomocą cross-stripa MinGW
    STRIP_TOOL="x86_64-w64-mingw32-strip"
    if command -v "$STRIP_TOOL" &> /dev/null; then
        echo "==> Uruchamianie '$STRIP_TOOL'..."
        "$STRIP_TOOL" --strip-all "$BINARY_PATH"
        echo -n " Rozmiar po strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    elif command -v strip &> /dev/null; then
        echo "==> Używanie domyślnego 'strip'..."
        strip --strip-all "$BINARY_PATH" 2>/dev/null || true
        echo -n " Rozmiar po strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    # Kompresja UPX
    if command -v upx &> /dev/null; then
        echo "==> Wykryto narzędzie UPX. Trwa kompresja pliku .exe..."
        upx --best --lzma "$BINARY_PATH"
        echo "--------------------------------------------------"
        echo -n " Rozmiar po kompresji UPX: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    fi

    echo "Gotowe! Twój plik znajduje się w: $BINARY_PATH"
else
    echo "==> Błąd: Nie znaleziono wygenerowanego pliku binarnego w $BINARY_PATH"
    exit 1
fi
