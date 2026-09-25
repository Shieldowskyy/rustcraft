# RustCraft — Cross-Platform Voxel Sandbox Engine

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20FreeBSD%20%7C%20ARM64%20%7C%20ARMv7%20%7C%20ARMv6%20%7C%20RISC--V%20%7C%20x86__32%20%7C%20PS%20Vita%20%7C%20WASM-brightgreen.svg)]()

A high-performance, modular voxel sandbox game and engine written in Rust with first-class support for **multiple platforms and architectures**:
Desktop (Linux x86_64 / x86 32-bit / ARM64 / RISC-V 64, Windows 64-bit / 32-bit, macOS, FreeBSD), Embedded Linux (Raspberry Pi Zero W / ARMv6, Raspberry Pi 2/3 / ARMv7), Sony PlayStation Vita (GXM GPU), and Web browsers (WebAssembly / HTML5 canvas).

RustCraft features a multithreaded CPU software rasterizer (tiled rasterization, sub-pixel precision, per-vertex Ambient Occlusion, distance fog) with zero graphics library dependencies, a hardware-accelerated Vulkan/WGPU backend, and a bare-metal GXM GPU renderer for the PlayStation Vita (SGX543MP4) delivering smooth 60 FPS.

---

## 1. Supported Platforms & Renderers

| Platform / Architecture | Target Triple | Primary Renderer | Build Script / Command |
|---|---|---|---|
| **Linux (x86_64)** | `x86_64-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./compile_linux.sh` |
| **Linux (32-bit x86 / i686)** | `i686-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./build_linux_x86_32.sh` |
| **Linux (ARM64 / AArch64)** | `aarch64-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./build_linux_arm64.sh` |
| **Linux (ARMv7 32-bit HF)** | `armv7-unknown-linux-gnueabihf` | Software CPU (Rayon) / Vulkan | `./build_linux_armv7.sh` |
| **Linux (RISC-V 64-bit)** | `riscv64gc-unknown-linux-gnu` | Software CPU (Rayon) | `./build_linux_riscv64.sh` |
| **Raspberry Pi Zero W (ARMv6)** | `arm-unknown-linux-gnueabihf` | Software CPU (Tuned for ARM11) | `./build_rpi_armv6.sh` |
| **FreeBSD (x86_64)** | `x86_64-unknown-freebsd` | Software CPU / X11 | `./build_freebsd.sh` |
| **Windows (x86_64)** | `x86_64-pc-windows-gnu` | Software CPU / Vulkan | `./compile-windows.sh` |
| **Windows (32-bit x86 / i686)** | `i686-pc-windows-gnu` | Software CPU / Vulkan | `./build_windows_32.sh` |
| **WebAssembly** | `wasm32-unknown-unknown` | Single-threaded Software CPU | `./build_web.sh` |
| **PlayStation Vita** | `armv7-sony-vita-newlibeabihf` | Bare-Metal GXM GPU (SGX543MP4) | `./build_vita.sh` |
| **macOS** | `x86_64-apple-darwin` / `aarch64` | Software CPU / Metal (WGPU) | `cargo run --release` |

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

### Desktop & Embedded Linux (x86_64, ARM64, ARMv7, ARMv6, FreeBSD)
* **Rust**: stable toolchain 1.75+ (`rustup` recommended: https://rustup.rs).
* **Linux / FreeBSD**: X11 development headers (`libx11-dev` on Debian/Ubuntu, `xorgproto` on FreeBSD).
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

### Linux (x86_64)
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

### Linux (32-bit x86 / i686)
```bash
./build_linux_x86_32.sh
# Output: target/i686-unknown-linux-gnu/release/rustcraft
```

### Linux ARM64 (Raspberry Pi 3/4/5 64-bit, Orange Pi, Servers)
```bash
./build_linux_arm64.sh
# Output: target/aarch64-unknown-linux-gnu/release/rustcraft
```

### Linux ARMv7 (Raspberry Pi 2/3 32-bit, BeagleBone, Retro Handhelds)
```bash
./build_linux_armv7.sh
# Output: target/armv7-unknown-linux-gnueabihf/release/rustcraft
```

### Linux RISC-V (64-bit GC / VisionFive 2, StarFive, Allwinner D1 64-bit)
```bash
./build_linux_riscv64.sh
# Output: target/riscv64gc-unknown-linux-gnu/release/rustcraft
```

### Raspberry Pi Zero W / Zero / 1 (ARMv6)
```bash
./build_rpi_armv6.sh
# Cross-compiles for ARMv6KZ (arm1176jzf-s) with VFPv2 hardware float support
# Output: target/arm-unknown-linux-gnueabihf/release/rustcraft
```

### FreeBSD (x86_64)
```bash
./build_freebsd.sh
# Output: target/x86_64-unknown-freebsd/release/rustcraft
```

### Windows (x86_64)
```bash
./compile-windows.sh
# Output: target/x86_64-pc-windows-gnu/release/rustcraft.exe
```

### Windows (32-bit x86 / i686)
```bash
./build_windows_32.sh
# Output: target/i686-pc-windows-gnu/release/rustcraft.exe
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

### Desktop & Embedded Linux (Keyboard & Mouse)

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
├── compile_linux.sh            # Linux x86_64 release & strip script
├── build_linux_x86_32.sh       # Linux 32-bit (i686) build script
├── compile-windows.sh          # MinGW cross-compilation script (x86_64)
├── build_windows_32.sh         # Windows 32-bit (i686) build script
├── build_linux_arm64.sh        # Linux ARM64 (Raspberry Pi 4/5) build script
├── build_linux_armv7.sh        # Linux ARMv7 (Raspberry Pi 2/3 32-bit) build script
├── build_linux_riscv64.sh      # Linux RISC-V 64-bit build script
├── build_rpi_armv6.sh          # Raspberry Pi Zero W (ARMv6) build script
├── build_freebsd.sh            # FreeBSD x86_64 build script
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
