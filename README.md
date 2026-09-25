# RustCraft — Cross-Platform Voxel Sandbox Engine

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS%20%7C%20WASM%20%7C%20PS%20Vita-brightgreen.svg)]()

A high-performance, modular voxel sandbox game and engine written in Rust with first-class support for **five targets**:
Desktop (Linux, Windows, macOS), Sony PlayStation Vita (GXM GPU), and Web browsers (WebAssembly / HTML5 canvas).

RustCraft features a multithreaded CPU software rasterizer (tiled rasterization, sub-pixel precision, per-vertex Ambient Occlusion, distance fog) with zero graphics library dependencies, a hardware-accelerated Vulkan/WGPU backend, and a bare-metal GXM GPU renderer for the PlayStation Vita (SGX543MP4) delivering smooth 60 FPS.

---

## 1. Supported Platforms & Renderers

| Platform | Primary Renderer | Build Script / Command |
|---|---|---|
| **Linux (x86_64)** | Software CPU (Rayon) or Vulkan | `./compile_linux.sh` |
| **Windows (x86_64)** | Software CPU (Rayon) or Vulkan | `./compile-windows.sh` |
| **WebAssembly** | Single-threaded Software CPU | `./build_web.sh` |
| **PlayStation Vita** | Bare-Metal GXM GPU (SGX543MP4) | `./build_vita.sh` |
| **macOS** | Software CPU or Metal / WGPU | `cargo run --release` |

---

## 2. Engine Architecture & Subsystems

RustCraft separates core engine capabilities from gameplay logic, providing human-readable, well-documented traits and extension points under [`src/engine/`](ARCHITECTURE.md):

* **Platform Subsystem (`src/engine/platform.rs`)**:
  [`PlatformBackend`](src/engine/platform.rs) trait abstracting window creation, input polling, event handling, and framebuffer presentation.
* **Application Lifecycle (`src/engine/mod.rs`)**:
  [`EngineApp`](src/engine/mod.rs) trait managing unified simulation loops, rendering triggers, resolution resizing, and frame buffer access.
* **Module & Extension Subsystem (`src/engine/module.rs`)**:
  [`EngineModule`](src/engine/module.rs) and [`ModuleManager`](src/engine/module.rs) providing pluggable lifecycle hooks (`init`, `pre_update`, `post_update`, `render`, `shutdown`).
* **Block & Voxel Registry (`src/engine/registry.rs`)**:
  [`BlockRegistry`](src/engine/registry.rs) and [`BlockProperties`](src/engine/registry.rs) for registering custom blocks with hardness, luminance, transparency, and flammability rules.
* **Modular Typography Subsystem (`src/engine/fonts/`)**:
  [`BitmapFont`](src/engine/fonts/mod.rs) trait with built-in [`RustcraftRegular`](src/engine/fonts/rustcraft_regular.rs) (3x5 pixel font with Polish diacritics and symbols) and font registry.
* **Zero-Overhead Localization (`src/core/i18n.rs`)**:
  In-memory translation engine (`locales/*.json`) with zero runtime file I/O dependencies, dynamic in-game language switching, and Python synchronization pipeline (`scripts/sync_locales.py`).

---

## 3. Requirements

### Desktop (Linux / Windows / macOS)
* **Rust**: stable toolchain 1.75+ (`rustup` recommended: https://rustup.rs).
* **Linux**: X11 development headers (`libx11-dev` on Debian/Ubuntu).
* **Windows / macOS**: No external C libraries required (`minifb` uses native OS windowing APIs).
* **Vulkan (Optional)**: Vulkan loader and GPU with graphic queue support.

### PlayStation Vita
* [VitaSDK](https://vitasdk.org) installed at `/usr/local/vitasdk` or `$HOME/vitasdk`.
* Rust target: `armv7-sony-vita-newlibeabihf`.
* Cg shader compiler: `psp2cgc` (automatically invoked by `build.rs`, with prebuilt GXP fallbacks).
* Packaging utilities: `vita-elf-create`, `vita-make-fself`, `vita-mksfoex`, `vita-pack-vpk`.

### WebAssembly
* `wasm-pack` or `wasm-bindgen-cli`.
* Target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`).
* Local HTTP server (e.g. `python3 -m http.server 8080`).

---

## 4. Building and Running

### Linux / Desktop (Default Software Renderer)
```bash
# Optimized release build with symbol stripping and UPX compression
./compile_linux.sh

# Or run directly via Cargo:
cargo run --release
```

To run with the hardware-accelerated Vulkan renderer:
```bash
cargo run --release -- --renderer vulkan
# or shortcut:
cargo run --release -- --vulkan
```

To launch with a custom world seed:
```bash
cargo run --release -- --seed 8675309
cargo run --release -- --renderer vulkan --seed 8675309
```

### Windows
```bash
./compile-windows.sh
# Output: target/x86_64-pc-windows-gnu/release/rustcraft.exe
```

### PlayStation Vita
```bash
./build_vita.sh
# Output: RustCraft.vpk
```
Install `RustCraft.vpk` on your console using VitaShell (via USB or FTP to `ux0:`), or test locally with the [Vita3K emulator](https://vita3k.org/).

### WebAssembly / HTML5
```bash
./build_web.sh
cd web && python3 -m http.server 8080
# Open http://localhost:8080 in your browser
```

---

## 5. Controls

### Desktop (Keyboard & Mouse)

| Key / Input | Action |
|---|---|
| **Left Click on Window** | Capture / Lock mouse cursor |
| **ESC / P** | Release cursor / Pause Game Menu |
| **W, A, S, D** | Horizontal movement |
| **SPACE** | Jump (Ascend in fly mode) |
| **LEFT SHIFT / C** | Sneak (Descend in fly mode) |
| **LEFT CTRL (Hold)** | Sprint |
| **F** | Toggle Flight Mode |
| **G** | Toggle Survival / Creative Mode |
| **Mouse Motion** | Look / Aim camera |
| **LMB** | Mine / Break block / Attack |
| **RMB** | Place block / Interact / Consume item |
| **1 – 0 / Mouse Scroll** | Select Hotbar Slot (10 slots) |
| **E** | Open Inventory & Crafting UI |
| **Q** | Drop selected item |
| **F5** | Quick-save world |
| **F3** | Toggle Debug Diagnostics Overlay |

### PlayStation Vita / Gamepad

| Gamepad Button | Action |
|---|---|
| **Left Stick** | Movement (WASD) |
| **Right Stick** | Camera Look / Aim |
| **Cross [X] / A** | Jump / Ascend |
| **Circle [O] / B** | Sneak / Descend |
| **Square [ ] / X** | Mine / Break (secondary) |
| **Triangle [^] / Y** | Open Inventory / Crafting |
| **R Trigger (R2/RT)** | Mine / Attack |
| **L Trigger (L2/LT)** | Place block / Consume |
| **L1 / R1** | Cycle Hotbar Slots |
| **L3 (Stick Click)** | Sprint |
| **R3 (Stick Click)** | Toggle Flight Mode |
| **SELECT** | Toggle Survival / Creative Mode |
| **START** | Game Pause Menu |
| **D-Pad UP** | Toggle Debug Overlay |
| **D-Pad LEFT** | Drop item |
| **Front Touchscreen** | Camera look & slot selection |

---

## 6. Procedural Biomes

The world generator procedural synthesizes 8 distinct biomes using multi-octave gradient noise:

| Biome | Characteristics |
|---|---|
| **Plains** | Flat grassy expanses with oak trees and flowers |
| **Forest** | Rolling hills with dense oak foliage |
| **Desert** | Sand dunes, sandstone layers, and cacti |
| **Mountains** | Towering stone peaks reaching past cloud height |
| **Snow** | Frozen tundra with snow layers and pine trees |
| **Cherry Grove** | Vibrant cherry blossom trees with falling petals |
| **Ocean** | Deep sea floor with dynamic underwater lighting |
| **Savanna** | Arid grasslands with acacia-style trees |

---

## 7. Localization Pipeline (i18n)

RustCraft features a built-in zero-dependency localization system supporting runtime language switching.

* **Translation Files (`locales/*.json`)**:
  * `locales/en.json` – Primary source language.
  * `locales/pl.json` – Polish localization.
* **Sync Script (`scripts/sync_locales.py`)**:
  ```bash
  # Synchronize all existing locale files with en.json
  python3 scripts/sync_locales.py

  # Create a brand new language dictionary (e.g. German)
  python3 scripts/sync_locales.py --new de

  # Verify in CI that all translations are complete
  python3 scripts/sync_locales.py --check
  ```

---

## 8. Codebase Structure

```
rustcraft/
├── Cargo.toml                  # Project dependencies & feature flags
├── ARCHITECTURE.md             # Comprehensive engine architecture guide
├── compile_linux.sh            # Linux build & strip script
├── compile-windows.sh          # MinGW cross-compilation script
├── build_vita.sh               # PS Vita VPK packaging script
├── build_web.sh                # WebAssembly build script
├── locales/                    # Embedded i18n JSON dictionaries
│   ├── en.json
│   └── pl.json
├── scripts/                    # Automation & localization tools
│   └── sync_locales.py
├── src/
│   ├── lib.rs                  # Library root & public module exports
│   ├── main.rs                 # Native binary entry point
│   ├── app.rs                  # Core simulation loop & game state machine
│   ├── engine/                 # Modular engine subsystems
│   │   ├── mod.rs              # EngineApp trait & subsystem re-exports
│   │   ├── platform.rs         # PlatformBackend trait & configuration
│   │   ├── module.rs           # EngineModule & ModuleManager
│   │   ├── registry.rs         # BlockRegistry & voxel properties
│   │   └── fonts/              # Modular BitmapFont system (RustcraftRegular)
│   ├── core/                   # Core math, noise, settings, and i18n
│   ├── world/                  # Chunks, terrain gen, meshing, storage
│   ├── gameplay/               # Inventory, crafting recipes, physics, mobs
│   ├── graphics/               # Software rasterizer, Vulkan backend, camera
│   ├── platform/               # Mouse capture, gamepad, PS Vita GXM
│   ├── ui/                     # Pixel-rendered HUD, menus, and inventory
│   └── web.rs                  # WebAssembly entry point & canvas blitter
└── vita/                       # PS Vita shaders, LiveArea assets, param.sfo
```

---

## 9. License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
