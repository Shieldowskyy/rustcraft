#!/usr/bin/env bash
set -e

# Automatically add ~/.cargo/bin to PATH if present
export PATH="$HOME/.cargo/bin:$PATH"

# Terminal color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Building RustCraft for WebAssembly (WASM) ===${NC}"

# Check if wasm32 target is installed
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "^wasm32-unknown-unknown$"; then
        echo -e "${YELLOW}Installing wasm32-unknown-unknown target via rustup...${NC}"
        rustup target add wasm32-unknown-unknown
    fi
elif ! rustc --print target-list | grep -q "wasm32-unknown-unknown"; then
    echo -e "${RED}Error: wasm32-unknown-unknown target not supported in compiler.${NC}"
    exit 1
fi

# Check for wasm-pack or wasm-bindgen
if command -v wasm-pack &> /dev/null; then
    echo -e "${GREEN}Using wasm-pack...${NC}"
    wasm-pack build --target web --out-dir web/pkg --release
elif command -v wasm-bindgen &> /dev/null; then
    echo -e "${GREEN}Compiling via cargo + wasm-bindgen-cli...${NC}"
    cargo build --target wasm32-unknown-unknown --release --lib
    wasm-bindgen target/wasm32-unknown-unknown/release/rustcraft.wasm --out-dir web/pkg --target web --no-typescript
else
    echo -e "${YELLOW}Neither wasm-pack nor wasm-bindgen found.${NC}"
    echo -e "You can install required tools via:"
    echo -e "  cargo install wasm-pack"
    echo -e "  or"
    echo -e "  cargo install wasm-bindgen-cli"
    exit 1
fi

echo -e "${GREEN}✓ WebAssembly package built successfully in web/pkg!${NC}"
echo -e "${BLUE}To run the game in your browser, launch a local HTTP server in the web/ directory:${NC}"
echo -e "  cd web && python3 -m http.server 8080"
echo -e "And then open in your browser: http://localhost:8080"
