#!/usr/bin/env bash
set -e

# Automatyczne dodanie ~/.cargo/bin do PATH jeśli istnieje
export PATH="$HOME/.cargo/bin:$PATH"

# Kolory do terminala
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'


echo -e "${BLUE}=== Budowanie RustCraft dla WebAssembly (WASM) ===${NC}"

# Sprawdzanie czy zainstalowany jest target wasm32
if ! rustc --print target-list | grep -q "wasm32-unknown-unknown"; then
    echo -e "${RED}Błąd: Brak wsparcia dla targetu wasm32-unknown-unknown w kompilatorze.${NC}"
    exit 1
fi

# Sprawdzanie wasm-pack lub wasm-bindgen
if command -v wasm-pack &> /dev/null; then
    echo -e "${GREEN}Używanie wasm-pack...${NC}"
    wasm-pack build --target web --out-dir web/pkg --release
elif command -v wasm-bindgen &> /dev/null; then
    echo -e "${GREEN}Kompilacja przez cargo + wasm-bindgen-cli...${NC}"
    cargo build --target wasm32-unknown-unknown --release --lib
    wasm-bindgen target/wasm32-unknown-unknown/release/rustcraft.wasm --out-dir web/pkg --target web --no-typescript
else
    echo -e "${YELLOW}Nie znaleziono wasm-pack ani wasm-bindgen.${NC}"
    echo -e "Możesz zainstalować narzędzie za pomocą:"
    echo -e "  cargo install wasm-pack"
    echo -e "  lub"
    echo -e "  cargo install wasm-bindgen-cli"
    exit 1
fi

echo -e "${GREEN}✓ Pomyślnie zbudowano paczkę WebAssembly w katalogu web/pkg!${NC}"
echo -e "${BLUE}Aby uruchomić grę w przeglądarce, uruchom lokalny serwer HTTP w katalogu web/:${NC}"
echo -e "  cd web && python3 -m http.server 8080"
echo -e "A następnie otwórz w przeglądarce: http://localhost:8080"
