# RustCraft — Cross-Platform Voxel Sandbox Engine

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Lines of Code](https://img.shields.io/badge/lines%20of%20code-71.6k-blue.svg)](#code-statistics--lines-of-code)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20FreeBSD%20%7C%20Android%20%7C%20Xbox%20360%20%7C%20ARM64%20%7C%20ARMv7%20%7C%20ARMv6%20%7C%20RISC--V%20%7C%20x86__32%20%7C%20Win__ARM64%20%7C%20PS%20Vita%20%7C%20WASM-brightgreen.svg)]()

A high-performance, modular voxel sandbox game and engine written in Rust with first-class support for **multiple platforms and architectures**:
Desktop (Linux x86_64 / x86 32-bit / ARM64 / RISC-V 64, Windows x86_64 / 32-bit / ARM64, macOS, FreeBSD), Mobile (Android ARM64 / ARMv7 / x86_64 APK with multi-touch virtual controls), Consoles (Microsoft Xbox 360 Homebrew XEX2, Sony PlayStation Vita GXM GPU), Embedded Linux (Raspberry Pi Zero W / ARMv6, Raspberry Pi 2/3 / ARMv7), and Web browsers (WebAssembly / HTML5 canvas).

RustCraft features a multithreaded CPU software rasterizer (tiled rasterization, sub-pixel precision, per-vertex Ambient Occlusion, distance fog, per-biome post-process color filters) with zero graphics library dependencies, a hardware-accelerated Vulkan/WGPU backend, and a bare-metal GXM GPU renderer for the PlayStation Vita (SGX543MP4) delivering smooth 60 FPS.

---

## 1. Supported Platforms & Renderers

| Platform / Architecture | Target Triple | Primary Renderer | Build Script / Command |
|---|---|---|---|
| **Linux (x86_64)** | `x86_64-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./scripts/compile_linux.sh` (Ubuntu 22.04 recommended for portable builds) |
| **Linux (32-bit x86 / i686)** | `i686-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./scripts/build_linux_x86_32.sh` |
| **Linux (ARM64 / AArch64)** | `aarch64-unknown-linux-gnu` | Software CPU (Rayon) / Vulkan | `./scripts/build_linux_arm64.sh` |
| **Linux (ARMv7 32-bit HF)** | `armv7-unknown-linux-gnueabihf` | Software CPU (Rayon) / Vulkan | `./scripts/build_linux_armv7.sh` |
| **Linux (RISC-V 64-bit)** | `riscv64gc-unknown-linux-gnu` | Software CPU (Rayon) | `./scripts/build_linux_riscv64.sh` |
| **Raspberry Pi Zero W (ARMv6)** | `arm-unknown-linux-gnueabihf` | Software CPU (Tuned for ARM11) | `./scripts/build_rpi_armv6.sh` |
| **FreeBSD (x86_64)** | `x86_64-unknown-freebsd` | Software CPU / X11 | `./scripts/build_freebsd.sh` |
| **Windows (x86_64)** | `x86_64-pc-windows-gnu` | Software CPU / Vulkan | `./scripts/compile-windows.sh` |
| **Windows (32-bit x86 / i686)** | `i686-pc-windows-gnu` | Software CPU / Vulkan | `./scripts/build_windows_32.sh` |
| **Windows (ARM64 / AArch64)** | `aarch64-pc-windows-msvc` | Software CPU / Vulkan | `./scripts/build_windows_arm64.sh` |
| **Android (ARM64 / ARMv7 / x86_64)** | `aarch64-linux-android` | ANativeWindow (Software / Rayon) | `./scripts/build_android.sh` |
| **Microsoft Xbox 360 (Homebrew XEX2)** | `powerpc-unknown-linux-gnu` / Xenon | Direct VRAM Framebuffer (720p HD) | `./scripts/build_xbox360.sh` |
| **WebAssembly** | `wasm32-unknown-unknown` | Single-threaded Software CPU | `./scripts/build_web.sh` |
| **PlayStation Vita** | `armv7-sony-vita-newlibeabihf` | Bare-Metal GXM GPU (SGX543MP4) | `./scripts/build_vita.sh` |
| **macOS (Apple Silicon & Intel)** | `aarch64` / `x86_64-apple-darwin` | Software CPU / Metal (WGPU) | `./scripts/build_macos.sh` |

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
* **Data-Driven Modding Subsystem (`src/engine/modding.rs`)**:
  Zero-dependency JSON mod loader (`mods/`) supporting custom blocks, items, food nutrition, crafting/smelting recipes, loot tables, and chat commands, with automatic client synchronization over multiplayer (Source engine style).
* **Dynamic Weather Subsystem (`src/world/weather.rs`)**:
  Complete atmospheric weather simulation: Clear, Rain, Thunderstorms, and Snow. Features biome-dependent precipitation (rain in temperate zones, falling snow in cold biomes, clear skies in deserts), realistic sky and cloud darkening, procedural precipitation particle rendering, procedural audio synthesis for rain ambiance, distant thunder rumbles, and random terrain lightning strikes, controllable via `/weather` in-game commands.
* **Villages, Villagers & Trading Economy (`src/gameplay/trading.rs`, `src/world/mod.rs`)**:
  Procedural village generation across biomes with houses, paths, lamp posts, wells, and farm plots. Inhabited by Villagers with 8 distinct professions, interactive trading interface, emerald-based currency economy, and trade level progression. Protected by towering Iron Golems.
* **Modular Typography Subsystem (`src/engine/fonts/`, `src/graphics/font.rs`)**:
  [`BitmapFont`](src/engine/fonts/mod.rs) trait supporting classic pixel fonts ([`RustcraftRegular`](src/engine/fonts/rustcraft_regular.rs), [`RustcraftLight`](src/engine/fonts/rustcraft_light.rs)) and high-resolution Unicode font rendering with toggle in Settings.
* **Zero-Overhead Localization (`src/core/i18n.rs`)**:
  In-memory translation engine (`locales/*.json`) with zero runtime file I/O dependencies, dynamic in-game language switching, and Python synchronization pipeline (`scripts/sync_locales.py`).
* **Audio & Music Subsystem (`src/audio/`)**:
  Zero-allocation polyphonic sound engine with a 64-voice pool, priority stealing, lock-free SPSC command ring buffer, 3D spatial audio (constant-power panning, distance attenuation, acoustic occlusion through voxels, air absorption filtering), and 100% procedural waveform synthesis (zero external audio files). Plus built-in streaming soundtrack system (`src/audio/soundtrack.rs`) supporting Ogg/Opus background music, category shuffling, volume controls, and music toast notifications. Standalone Linux, Windows, macOS, FreeBSD, and Xbox 360 builds embed soundtrack files; adjacent on-disk files take precedence when present. The startup logo is embedded on all platforms. Backends for Linux (PulseAudio/ALSA), Windows, Android (AAudio), PS Vita (SceAudioOut), Xbox 360, and WebAssembly.
* **Crash Reporter & Sentry Telemetry (`src/core/crash_reporter.rs`)**:
  Integrated panic handler capturing thread registers, stack backtraces, OS/hardware telemetry, logger ring-buffer breadcrumbs, and active world state. Generates local crash reports and supports direct telemetry upload to Sentry via `--sentry-dsn <DSN>` or `SENTRY_DSN` env var.
* **Multiplayer Networking (`src/net/`)**:
  Client-server voxel synchronization protocol over TCP, entity interpolation, chat messaging, automatic LAN discovery, and an optional self-hosted CG-NAT relay.
* **CLI Argument Parser (`src/cli.rs`)**:
  Parses `--renderer software|vulkan`, `--seed <NUMBER>`, `--profiler`, `--autotest`, `-log`, `--server`, `--relay-server`, `--relay-port <PORT>`, `--port <PORT>`, `--connect <ADDR>`, `--sentry-dsn <DSN>`, `--no-sentry`, `--crash-test`, and `--help` flags with cryptographically-strong random seed generation via `getrandom`.

---

## Multiplayer over CG-NAT

The in-game **Host this world online** action starts a server for the current save and creates an outbound connection to a RustCraft relay. This lets invited players join without opening a port on the host's router. Guests enter the generated `relay://host:port/code` invite in the multiplayer address field.

The relay is a small TCP forwarder and does not store world data. Build and run it on a public VPS:

```bash
cargo build --release --bin rustcraft-server
cp target/release/rustcraft-server .
sudo ./install_relay_service.sh
```

The installer expects the executable beside itself, installs it under `/usr/local/bin`, and enables the service on systemd or OpenRC (including containers where either init system is running). Pass a different port as an optional argument, for example `sudo ./install_relay_service.sh 26658`. Allow inbound TCP port `26657` (or the selected port) in the VPS firewall and, for LXC, its network policy. The game defaults to `relay.rustcraft.dsh.yt:26657`. On desktop, copy `relay.properties.example` to `relay.properties` and set `relay-address` to override it, or set `RUSTCRAFT_RELAY_ADDR`. For Android builds, optionally embed another address with `RUSTCRAFT_RELAY_ADDR=relay.example.com:26657 ./scripts/build_android.sh`; alternatively place `relay.properties` in the app's internal files directory. The relay address is included in each invite, so guests do not need a separate relay configuration.

To host, load a world, open Pause, and choose **Host Online**. RustCraft reports startup and connection errors with an on-screen toast; once connected, the invite is copied where the platform supports clipboard access and is also posted in chat. To join, open **Multiplayer**, paste the invite into **Server Address or Invite** with `Ctrl+V`, then choose **Join Server**. On desktop, `Ctrl+C` in the pause menu copies the active host invite again.

**Security:** Game traffic through the relay is end-to-end encrypted with ChaCha20-Poly1305. The invitation includes a separate random secret that is never sent to the relay; the relay sees only the room ID and routing metadata. Keep the invite private. Direct connections that do not use `relay://` retain the existing unencrypted TCP behavior.

---

## 3. Requirements

### Desktop & Embedded Linux (x86_64, ARM64, ARMv7, ARMv6, FreeBSD)
* **Rust**: stable toolchain 1.75+ (`rustup` recommended: https://rustup.rs).
* **Linux / FreeBSD**: X11 development headers (`libx11-dev` on Debian/Ubuntu, `xorgproto` & `libudev-devd` on FreeBSD).
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
./scripts/compile_linux.sh

# Or run directly via Cargo:
cargo run --release
```

Linux x86_64 release builds use the generic x86-64 CPU baseline rather than
the build computer's CPU features. For compatibility with older distributions,
build on Ubuntu 22.04 or use the Linux x86_64 artifact from CI. The executable
still uses the host's glibc and desktop graphics/audio/udev libraries; it is
not a fully static binary. Sentry HTTPS uses Rustls, avoiding a runtime
dependency on OpenSSL shared libraries.

To run with the hardware-accelerated Vulkan renderer:
```bash
cargo run --release -- --renderer vulkan
# or shortcut:
cargo run --release -- --vulkan
```

To enable the embedded HTTP profiler dashboard:
```bash
cargo run --release -- --profiler
```

### Linux (32-bit x86 / i686)
```bash
./scripts/build_linux_x86_32.sh
# Output: target/i686-unknown-linux-gnu/release/rustcraft
```

### Linux ARM64 (Raspberry Pi 3/4/5 64-bit, Orange Pi, Servers)
```bash
./scripts/build_linux_arm64.sh
# Output: target/aarch64-unknown-linux-gnu/release/rustcraft
```

### Headless Linux ARM64 server + relay
```bash
./scripts/build_linux_arm64_server.sh
# Output: dist/rustcraft-server-linux-arm64.tar.gz
```

For other Rust targets, use the generic server builder. Omit the target triple
to build for the current host:

```bash
./scripts/build_server.sh x86_64-pc-windows-gnu
# Output: dist/rustcraft-server-x86_64-pc-windows-gnu.tar.gz
```

Cross targets require `cross` with Docker/Podman, or a target linker and
libraries configured for Cargo. Each package contains only the server
executable and `server.properties`, not the game client or bundled soundtracks.
Extract it on the target machine and run one service per process:

```bash
tar -xzf rustcraft-server-linux-arm64.tar.gz
./rustcraft-server --relay-server --relay-port 8080
# Or, in a separate process/machine, run a dedicated game server:
./rustcraft-server --server --port 26656 --seed 133742
```

Open the selected TCP port in the host firewall. The dedicated game server
loads and writes `server.properties` in its working directory. The executable
requires the target system's compatible system libraries.

### Linux ARMv7 (Raspberry Pi 2/3 32-bit, BeagleBone, Retro Handhelds)
```bash
./scripts/build_linux_armv7.sh
# Output: target/armv7-unknown-linux-gnueabihf/release/rustcraft
```

### Linux RISC-V (64-bit GC / VisionFive 2, StarFive, Allwinner D1 64-bit)
```bash
./scripts/build_linux_riscv64.sh
# Output: target/riscv64gc-unknown-linux-gnu/release/rustcraft
```

### Raspberry Pi Zero W / Zero / 1 (ARMv6)
```bash
./scripts/build_rpi_armv6.sh
# Cross-compiles for ARMv6KZ (arm1176jzf-s) with VFPv2 hardware float support
# Output: target/arm-unknown-linux-gnueabihf/release/rustcraft
```

### FreeBSD (x86_64)
```bash
./scripts/build_freebsd.sh
# Output: target/x86_64-unknown-freebsd/release/rustcraft
```

### Windows (x86_64)
```bash
./scripts/compile-windows.sh
# Output: target/x86_64-pc-windows-gnu/release/rustcraft.exe
```

### Windows (32-bit x86 / i686)
```bash
./scripts/build_windows_32.sh
# Output: target/i686-pc-windows-gnu/release/rustcraft.exe
```

### Windows ARM64 (Surface Pro ARM, Snapdragon X Elite, WoA)
```bash
./scripts/build_windows_arm64.sh
# Output: target/aarch64-pc-windows-msvc/release/rustcraft.exe
```

### PlayStation Vita
```bash
./scripts/build_vita.sh
# Output: RustCraft.vpk
```
Install `RustCraft.vpk` on your console using VitaShell (via USB or FTP to `ux0:`), or test locally with the [Vita3K emulator](https://vita3k.org/).

### WebAssembly / HTML5
```bash
./scripts/build_web.sh
cd web && python3 -m http.server 8080
# Open http://localhost:8080 in your browser
```

### macOS (Apple Silicon & Intel)
```bash
# Build for host macOS architecture (Apple Silicon M1/M2/M3/M4 or Intel):
./scripts/build_macos.sh

# Or build Universal 2 binary (arm64 + x86_64) packaged as .app bundle:
./scripts/build_macos.sh universal

# Output: dist/RustCraft.app and dist/rustcraft-macos-*.tar.gz
```

### Microsoft Xbox 360 (Homebrew XEX2 / XeLL)
```bash
./scripts/build_xbox360.sh
# Output: RustCraft-Xbox360.zip and target/xbox360/RustCraft/default.xex
```
Extract `RustCraft-Xbox360.zip` to a FAT32 USB drive (`USB:\RustCraft\default.xex`) and launch via XeXMenu, Aurora Dashboard, or DashLaunch on any exploited Xbox 360 console (RGH / JTAG), or boot `rustcraft.elf` directly via XeLL Reloaded.

### Android (ARM64 / ARMv7 / x86_64)
```bash
./scripts/build_android.sh
# Output: RustCraft.apk
```
Install on your Android device or emulator via `adb install -r RustCraft.apk`. Features touch controls, soft keyboard IME, low-latency native AAudio synthesis, and analog gamepad support.

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
| **RMB** | Place block / Interact / Consume food / Use tool (Hoe, Shears, Fishing Rod) |
| **Middle Click (MMB)** | Pick Block into hotbar (Creative Mode) |
| **1 – 0 / Mouse Scroll** | Select Hotbar Slot (10 slots) |
| **E** | Open Inventory & Crafting UI (with Creative category tabs & pagination) |
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
| **L Trigger (L2/LT)** | Place block / Consume / Tool interact |
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

The world generator procedurally synthesizes **20 distinct biomes** using multi-octave gradient noise (temperature, humidity, and continentalness axes):

| Biome | Characteristics |
|---|---|
| **Plains** | Flat grassy expanses with oak trees and flowers |
| **Forest** | Rolling hills with dense oak foliage |
| **Autumn Forest** | Orange-red autumnal oak trees with `AutumnLeaves` |
| **Birch Forest** | Temperate woodland populated with white-barked birch trees |
| **Dark Forest** | Shaded woodland with massive dense dark oak trees and thick interlocking canopies |
| **Taiga** | Cool evergreen coniferous forest with tall pine trees |
| **Snowy Taiga** | Dense pine forest in cold snowy regions |
| **Snowy Tundra** | Frozen tundra with snow layers and pine trees |
| **Ice Spikes** | Rare frozen wonderland with towering packed ice pillars and snow cover |
| **Jungle** | Dense tropical rainforest with giant canopy trees |
| **Badlands** | Arid canyon terrain with banded terracotta/sand/mud strata and rich surface gold |
| **Desert** | Sand dunes, sandstone layers, and cacti; desert wells spawn here |
| **Savanna** | Arid grasslands with acacia-style trees |
| **Swamp** | Waterlogged low terrain with swamp trees and `Mud` blocks |
| **Mushroom Island** | Ultra-rare surreal biome with mud ground and towering red mushroom trees |
| **Meadow** | Elevated plateau bursting with lush grass and solitary trees |
| **Cherry Grove** | Vibrant cherry blossom trees with `CherryLeaves` |
| **Mountains** | Towering stone peaks reaching past cloud height; stone ruins |
| **Ocean** | Deep sea floor with dynamic underwater lighting, kelp forests, seagrass, and coral ruins |
| **Deep Ocean** | Extra-deep ocean floor far from continental shelves with ocean trenches |

Each biome generates fitting procedural structures: **desert wells**, **stone ruins**, **forest log cabins** (with loot chests), **coral ruins** in oceans, and **procedural villages** (Plains, Savanna, Desert) featuring houses with wooden roofs and glass windows, furnaces, crafting tables, irrigated farm plots, community wells, and lamp posts.

---

## 7. Block, Item & Entity System

RustCraft features **163 unique vanilla block and item IDs** (plus extensible modded IDs 163..=255) across building materials, agriculture, mechanisms, 16 colored wools, aquatic wildlife, emerald economy, and functional blocks:

* **Building blocks**: Grass, Dirt, Stone, Cobblestone, MossyCobblestone, Sand, Planks, Wood, Glass, Ice, Snow, Mud, Bedrock, Sandstone, SmoothStone, Bricks, Obsidian, Bookshelf, WoodenSlab, StoneSlab, CobblestoneSlab, SandstoneSlab, BrickSlab, WoodenStairs, StoneStairs, CobblestoneStairs, SandstoneStairs, BrickStairs, Ladder
* **Fluids**: Water, Lava
* **Foliage & Flora**: Leaves, SnowLeaves, CherryLeaves, AutumnLeaves, Cactus, Kelp (underwater plant), Seagrass (ocean floor vegetation), Sapling
* **Agriculture & Crops**: Farmland, FarmlandWet, WheatSeeds, WheatCrop, Wheat, Bread, Carrot, CarrotCrop, Potato, PotatoCrop, BakedPotato, BoneMeal
* **16 Wool Colors**: WhiteWool, OrangeWool, MagentaWool, LightBlueWool, YellowWool, LimeWool, PinkWool, GrayWool, LightGrayWool, CyanWool, PurpleWool, BlueWool, BrownWool, GreenWool, RedWool, BlackWool
* **RustStone (Redstone Mechanisms)**: RustStoneOre, RustStone (dust/wire), RustStoneWireLit (active wire carrying power), RustStoneBlock (constant power source), RustStoneLamp (unlit), RustStoneLampLit (light-emitting when powered), Lever / LeverOn, Button / ButtonPressed, PressurePlate / PressurePlatePressed
* **Ores, Minerals & Materials**: CoalOre, IronOre, GoldOre, DiamondOre, Diamond, EmeraldOre, Emerald, EmeraldBlock, IronOunce, GoldOunce, IronIngot, GoldIngot, CoalBlock, IronBlock, GoldBlock, DiamondBlock, Clay, ClayBall, Brick, Gravel, Flint
* **Tools (5 tiers + utilities)**:
  * Pickaxes, Axes, Shovels, Swords, and Hoes across 5 material tiers: Wooden, Stone, Iron, Gold, Diamond
  * **Shears**: Used for shearing sheep (yielding matching colored wool) and instantly harvesting leaves and wool
  * **Fishing Rod**: Cast bobber into water, wait for bite splashes, and reel in cod, salmon, pufferfish, or treasure
* **Items, Food & Mob Drops**: Stick, Torch, Apple, RawPorkchop, CookedPorkchop, RawBeef, CookedBeef, RawChicken, CookedChicken, Feather, Bone, Gunpowder, RawCod, CookedCod, RawSalmon, CookedSalmon, Pufferfish (causes poison if consumed), TropicalFish, DriedKelp (quick-snack food), Bed
* **Spawn Eggs**: PigSpawnEgg, CowSpawnEgg, ChickenSpawnEgg, SheepSpawnEgg, DolphinSpawnEgg, FishSpawnEgg, VillagerSpawnEgg, IronGolemSpawnEgg
* **Functional blocks**: CraftingTable, Chest, Furnace / FurnaceLit, Door / DoorOpen, TNT

### Living Entities & Mobs (14 Mob Types)

RustCraft includes a complete AI entity simulation with 14 distinct mob kinds (`MobKind`):
* **Passive Wildlife & Village Inhabitants**:
  * **Pig**: Roams grassy biomes, drops raw porkchops.
  * **Cow**: Roams plains and forests, drops raw beef.
  * **Chicken**: Smaller hitbox, pecks the ground, drops raw chicken and feathers.
  * **Sheep**: Features fleece color varieties, grazes on grass blocks (turning them into dirt and regrowing wool), and can be sheared using Shears without damage.
  * **Villager**: Inhabits procedural villages. Features 8 distinct professions (Farmer, Librarian, Armorer, Cleric, Butcher, Mason, Weapon Smith, Tool Smith), full trading dialogue interface, emerald currency exchange, and level progression (Novice to Master). Panics and flees indoors when attacked.
* **Village Guardians**:
  * **Iron Golem**: Massive 100 HP iron-clad protector guarding villagers and players. Swings heavy iron arms to launch hostile monsters high into the air.
* **Hostile Monsters**:
  * **Zombie**: Tracks player and villagers via pathfinding, deals melee damage, burns in daylight.
  * **Skeleton**: Ranged tracker, burns in daylight, drops bones (craftable into BoneMeal).
  * **Reaper**: Silently approaches the player, hisses with an ignition countdown, and explodes creating crater voxel destruction; drops gunpowder.
* **Aquatic Marine Life**:
  * **Dolphin**: Swims smoothly with realistic banking and surfacing, jumps gracefully, and grants the player **Dolphin's Grace** (swimming speed boost) when nearby.
  * **Cod & Salmon**: Ambient schooling fish inhabiting rivers and oceans, drop raw fish.
  * **Pufferfish**: Puffs up defensively when approached by players, drops poisonous pufferfish.
  * **Tropical Fish**: Vibrant multi-colored fish populating warm ocean biomes.

### Dynamic Weather & Atmosphere

* **Weather States**: Cycles naturally between Clear skies, Rain, Thunderstorms, and Snow storms.
* **Biome Temperature Awareness**: Warm biomes experience soothing rainfall; cold/alpine biomes (Taiga, Snowy Tundra, Ice Spikes) experience falling snowflakes; arid biomes (Desert, Badlands) stay clear.
* **Atmospheric Visuals**: Sky and cloud layers dynamically darken during storm fronts. Precipitation particles are rendered with high performance across both CPU software and GPU rasterizers.
* **Procedural Storm Audio**: Continuous procedural rain loops, distant thunder rumbles, and sudden ground-striking lightning bolts accompanied by blinding screen flashes.
* **Commands**: Full control via in-game chat: `/weather clear`, `/weather rain`, `/weather thunder`, `/weather query`.

### Villages, Villagers & Emerald Economy

* **Procedural Architecture**: Spaced villages generated in Plains, Savanna, and Desert biomes with cobblestone/sandstone paths, village houses with glass windows and interior furniture, village wells, lamp posts, and irrigated farm plots with crops.
* **Villager Trading UI**: Right-clicking a villager opens an interactive trading window displaying available trade offers, emerald and item costs, stock limits, and villager trade level progression.

---

## 8. Tile Entities (Interactive Blocks)

* **Chest** (`src/world/tile_entity.rs`): 27-slot item storage, persistent per world position. Forest cabin chests spawn with procedurally generated starter loot (food, tools, materials).
* **Furnace** (`src/world/tile_entity.rs`): Full smelting simulation — separate input, fuel, and output slots. Supports sand→glass, iron ore→iron ingot, raw meat→cooked food, wood→charcoal. Burn time and cook progress tick every simulation frame. Visual state switches between `Furnace` ↔ `FurnaceLit`.

---

## 9. Embedded Performance Profiler

RustCraft includes a built-in **HTTP profiler web dashboard** (enabled with `--profiler` CLI flag):

* **`src/core/profiler.rs`**: Per-frame telemetry structs — `MemoryBreakdown` (chunk voxels, mesh cache, textures, framebuffers, entities, CDRAM) and `RenderStats` (draw calls, subchunks rendered/culled, triangle counts, rasterizer thread count, pipeline name).
* **`src/core/profiler_server.rs`**: Zero-external-dependency micro HTTP server with a JSON API and live HTML dashboard. Tracks granular micro-timings for: terrain render, transparent pass, clouds, sky, entities, 3D hand, UI, physics tick, mob AI, world gen, and mesh building. Port configurable via `GameSettings::profiler_server_port` (default 8080).

---

## 10. Localization Pipeline (i18n)

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

## 11. Codebase Structure

```
rustcraft/
├── Cargo.toml                  # Project dependencies & feature flags
├── ARCHITECTURE.md             # Comprehensive engine architecture guide
├── build.rs                    # Vita shader compilation & cross-platform build config
├── scripts/compile_linux.sh            # Linux x86_64 release & strip script
├── scripts/build_server.sh              # Headless server package for a Rust target
├── scripts/build_linux_arm64_server.sh  # ARM64 server package compatibility entry point
├── scripts/build_linux_x86_32.sh       # Linux 32-bit (i686) build script
├── scripts/compile-windows.sh          # MinGW cross-compilation script (x86_64)
├── scripts/build_windows_32.sh         # Windows 32-bit (i686) build script
├── scripts/build_windows_arm64.sh      # Windows ARM64 (AArch64) build script
├── scripts/build_linux_arm64.sh        # Linux ARM64 (Raspberry Pi 4/5) build script
├── scripts/build_linux_armv7.sh        # Linux ARMv7 (Raspberry Pi 2/3 32-bit) build script
├── scripts/build_linux_riscv64.sh      # Linux RISC-V 64-bit build script
├── scripts/build_rpi_armv6.sh          # Raspberry Pi Zero W (ARMv6) build script
├── scripts/build_freebsd.sh            # FreeBSD x86_64 build script
├── scripts/build_vita.sh               # PS Vita VPK packaging script
├── scripts/build_web.sh                # WebAssembly build script
├── scripts/build_macos.sh              # macOS (Apple Silicon & Intel) build & .app bundle script
├── locales/                    # Embedded i18n JSON dictionaries
│   ├── en.json
│   └── pl.json
├── scripts/                    # Automation & localization tools
│   └── sync_locales.py
├── src/
│   ├── lib.rs                  # Library root & public module exports
│   ├── main.rs                 # Native binary entry point
│   ├── app/                    # Core simulation loop & game state machine
│   │   ├── mod.rs              # App orchestration, frame rendering & input dispatch
│   │   ├── gameplay.rs         # In-game tick, tool interaction, survival/creative loops
│   │   ├── menus.rs            # Menu state machine, pause and settings transitions
│   │   ├── containers.rs       # Chest and furnace container sync & inventory transfers
│   │   └── types.rs            # AppState, GameMode, Screen, and event types
│   ├── cli.rs                  # CLI argument parser & random seed generation
│   ├── web.rs                  # WebAssembly entry point & canvas blitter
│   ├── audio/                  # Audio engine, procedural synthesis, 3D spatial audio
│   │   ├── engine.rs           # AudioEngine lifecycle & sound dispatch
│   │   ├── mixer.rs            # Zero-allocation 64-voice polyphonic mixer
│   │   ├── synth.rs            # Procedural waveform synthesis (blocks, steps, hits, water)
│   │   ├── soundtrack.rs       # Ogg/Opus background music streaming, shuffle & toasts
│   │   ├── spatial.rs          # 3D spatial audio, acoustic occlusion & air absorption
│   │   ├── ring_buffer.rs      # Lock-free SPSC command ring buffer
│   │   ├── sound.rs            # SoundInstance, SoundId & procedural sound effects
│   │   └── backend/            # Multiplatform drivers (Linux, Win, Android, Vita, Xbox 360, WASM)
│   ├── net/                    # Multiplayer networking & LAN discovery
│   │   ├── protocol.rs         # Binary packet serialization & voxel delta sync
│   │   ├── client.rs           # Multiplayer network client & prediction
│   │   ├── server.rs           # Dedicated and embedded server state
│   │   ├── relay.rs            # Self-hosted reverse TCP relay
│   │   └── discovery.rs        # UDP LAN server discovery & broadcast
│   ├── engine/                 # Modular engine subsystems
│   │   ├── mod.rs              # EngineApp trait & subsystem re-exports
│   │   ├── platform.rs         # PlatformBackend trait & configuration
│   │   ├── module.rs           # EngineModule & ModuleManager
│   │   ├── registry.rs         # BlockRegistry & voxel properties
│   │   ├── modding.rs          # Data-driven JSON mod loader & multiplayer auto-download
│   │   └── fonts/              # BitmapFont system (RustcraftRegular + RustcraftLight)
│   ├── core/                   # Core math, noise, settings, profiler, and i18n
│   │   ├── math.rs             # Vec3, IVec3, Mat4 — zero-dependency 3D math
│   │   ├── noise.rs            # Deterministic 2D/3D fractal noise
│   │   ├── settings.rs         # GameSettings with full serialization
│   │   ├── controls.rs         # Centralized keybinding & gamepad mapping engine
│   │   ├── i18n.rs             # In-memory localization engine
│   │   ├── crash_reporter.rs   # Crash dump generation & Sentry telemetry integration
│   │   ├── logger.rs           # In-memory ring-buffer logging & breadcrumbs
│   │   ├── profiler.rs         # MemoryBreakdown & RenderStats telemetry structs
│   │   └── profiler_server.rs  # Embedded HTTP profiler web dashboard
│   ├── world/                  # Chunks, terrain gen, meshing, storage, tile entities
│   │   ├── block.rs            # BlockId enum (163 types) & block properties
│   │   ├── mesher.rs           # Chunk mesh builder with AO & async meshing
│   │   ├── storage.rs          # Binary world save/load (cross-platform paths)
│   │   ├── tile_entity.rs      # Chest (27-slot) & Furnace smelting simulation
│   │   ├── weather.rs          # Dynamic weather simulation (rain, snow, thunder, lightning)
│   │   └── mod.rs              # World, 20-biome generator, villages, structures, fluid/TNT sim
│   ├── gameplay/               # Inventory, crafting, physics, mobs, particles, trading
│   │   ├── player.rs           # Player entity, GameMode, health, fall damage
│   │   ├── inventory.rs        # Item stacking, hotbar (10), main inventory (27)
│   │   ├── item.rs             # Dropped ItemEntity physics
│   │   ├── physics.rs          # AABB collision, ice/lava friction, step-jumping
│   │   ├── mob.rs              # MobManager: 14 mobs (Pig, Cow, Chicken, Sheep, Zombie, Skeleton, Reaper, Dolphin, Fish, Villager, IronGolem)
│   │   ├── trading.rs          # Villager professions, emerald trading economy & XP levels
│   │   ├── particles.rs        # Block-break, hit sparks, explosion, rain/snow particles
│   │   ├── recipe.rs           # 2×2 and 3×3 crafting recipe definitions
│   │   ├── autotest.rs         # Headless self-test runner & autonomous bot AI
│   │   └── chat.rs             # In-game chat log & command dispatch (/weather, /give, /mobs)
│   ├── graphics/               # Rendering backends, camera, font, texture atlas
│   │   ├── traits.rs           # RenderBackend trait
│   │   ├── atlas.rs            # Procedural 128×128 px ABGR8888 texture atlas (170+ textures)
│   │   ├── camera.rs           # First-person camera, FOV, view matrix, chunk fade
│   │   ├── font.rs             # Bitmap font & Unicode font renderer with shadow & measurement
│   │   ├── software/           # Multi-threaded CPU rasterizer (Rayon tiled scanline, weather particles)
│   │   └── gpu/                # WGPU hardware backend (Vulkan / DX12 / Metal / WebGPU)
│   ├── platform/               # OS abstractions
│   │   ├── mouse_capture.rs    # X11 / Win32 raw mouse & cursor trap
│   │   ├── gamepad.rs          # gilrs gamepad (buttons, sticks, triggers)
│   │   ├── android.rs          # Android ANativeWindow, touch overlay & soft keyboard IME
│   │   ├── xbox360.rs          # Xbox 360 libxenon direct 720p framebuffer
│   │   ├── vita.rs             # PS Vita platform layer & SceDisplay
│   │   └── vita_gxm.rs         # Complete GXM GPU renderer (SGX543MP4)
│   └── ui/                     # Pixel-rendered HUD, menus, and inventory
│       ├── draw.rs             # 2D primitives (fill_rect, draw_text, pack_color)
│       ├── menu.rs             # Main menu, logo voxel art, save slot carousel
│       ├── hud.rs              # Crosshair, hotbar, debug panel, damage vignette
│       ├── minimap.rs          # Real-time top-down radar & compass minimap
│       ├── inventory_ui.rs     # Creative picker (tabbed, paginated), 2×2/3×3 crafting, inventory grid
│       ├── container_ui.rs     # Chest (27-slot), Furnace, and Villager Trading interaction UI
│       ├── create_world.rs     # Create world form (name, seed, game mode)
│       ├── save_select.rs      # Save slot selection screen
│       ├── clipboard.rs        # Clipboard helper (wl-paste / xclip / fallback buffer)
│       └── settings_ui.rs      # Settings menu (audio, video, controls, Unicode font, profiler...)
└── vita/                       # PS Vita shaders, LiveArea assets, param.sfo
    ├── shaders/                # Cg vertex + fragment shaders (*.vert.cg / *.frag.cg)
    └── sce_sys/                # Vita LiveArea assets & param.sfo metadata
```

### Code Statistics & Lines of Code

| Subsystem / Directory | Description | Lines of Code |
|---|---|---|
| **`src/graphics/`** | Software Rayon rasterizer, WGPU backend, Camera, Font, Texture atlas (170+ tex) | ~11,420 |
| **`src/world/`** | Voxel storage, 20 biomes, villages, weather system, chunk mesher, tile entities | ~9,730 |
| **`src/ui/`** | HUD, inventory, minimap, trading UI, craft grids, menus, touch controls, settings | ~9,100 |
| **`src/platform/`** | Xbox 360, PS Vita (GXM GPU), Android NDK, Windows, X11 mouse, Gamepad | ~8,260 |
| **`src/app/`** | Core engine loop, state machine, gameplay tick, containers, menus | ~7,440 |
| **`src/audio/`** | 64-voice polyphonic mixer, procedural synthesis, weather storm audio, soundtrack | ~6,770 |
| **`src/gameplay/`** | AutoTest bot AI, physics/collision, inventory, crafting, 14 mobs, trading, chat | ~6,530 |
| **`src/core/`** | Math (Vec3/Mat4), noise, settings, controls, profiler, crash reporter & Sentry, i18n | ~5,650 |
| **`src/net/`** | Multiplayer networking protocol, packet serialization & LAN discovery | ~2,880 |
| **`src/engine/`** | Modular engine traits, modding system, platform backend interfaces, bitmap fonts | ~2,470 |
| **`Root & Entry Points`** | `main.rs`, `web.rs` (WASM), `cli.rs`, `build.rs`, `lib.rs` | ~1,370 |
| **Total Rust Source** | **Complete Rust codebase (zero external engine dependencies)** | **~71,600 LOC** |

---

## 12. License & Legal Disclaimer

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

### Disclaimer
RustCraft is an independent, non-commercial open-source voxel game engine and sandbox recreation developed in Rust. RustCraft is **not** an official Minecraft product and is **not** approved by, affiliated with, endorsed by, or associated with Mojang Studios or Microsoft. All trademarks, service marks, and trade names (including "Minecraft") belong to their respective owners (Mojang Synergies AB / Microsoft Corporation).
