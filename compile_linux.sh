#!/usr/bin/env bash
set -e

# Automatyczne załadowanie środowiska Rusta (jeśli istnieje)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
elif [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v cargo &> /dev/null; then
    echo "==> Błąd: Nie znaleziono polecenia 'cargo'. Upewnij się, że Rust jest zainstalowany."
    exit 1
fi

echo "==> Kompilowanie projektu w trybie release..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

BINARY_PATH="target/release/rustcraft"

if [ -f "$BINARY_PATH" ]; then
    echo "--------------------------------------------------"
    echo " Sukces! Plik binarny został zbudowany."
    echo -n " Rozmiar przed odchudzaniem: "
    du -h "$BINARY_PATH" | awk '{print $1}'
    echo "--------------------------------------------------"

    # Usuwanie symboli debugowania (strip)
    if command -v strip &> /dev/null; then
        echo "==> Uruchamianie 'strip' w celu usunięcia symboli debugowania..."
        strip --strip-all "$BINARY_PATH"
        echo -n " Rozmiar po strip: "
        du -h "$BINARY_PATH" | awk '{print $1}'
        echo "--------------------------------------------------"
    else
        echo "==> Wskazówka: Narzędzie 'strip' nie jest dostępne, pomijam ten krok."
    fi

    echo "Gotowe! Twój plik znajduje się w: $BINARY_PATH"
else
    echo "==> Błąd: Nie znaleziono wygenerowanego pliku binarnego."
    exit 1
fi
