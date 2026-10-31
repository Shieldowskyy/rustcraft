# RustCraft Architecture & Modular Design

## 1. Overview
RustCraft is a lightweight, voxel-based game engine and sandbox built with Rust. It supports a multi-threaded CPU software rasterizer, a GPU hardware pipeline (WGPU / Vulkan), a dedicated PlayStation Vita GXM hardware renderer (SGX543MP4), Microsoft Xbox 360 direct VRAM scanout, Android NDK surface blitting, and a WebAssembly/HTML5 port — all sharing the same core game logic through a platform-agnostic architecture.

The codebase is organized into isolated, single-responsibility modules to maximize maintainability, ease cross-platform porting, and allow rapid feature development.

```
                  ┌──────────────────────────────┐
                  │  main.rs / lib.rs / web.rs   │  (App Loop, CLI, WASM entry)
                  └──────────────┬───────────────┘
                                 │
     ┌───────────────────────────┼──────────────────────────┐
     ▼                           ▼                          ▼
┌──────────────┐         ┌──────────────┐           ┌──────────────┐
│  src/core    │         │  src/world   │           │ src/gameplay │
│  - math.rs   │         │  - block.rs  │           │  - player.rs │
│  - noise.rs  │         │  - mesher.rs │           │  - mob.rs    │
│  - settings  │         │  - storage.rs│           │  - physics.rs│
│  - i18n.rs   │         │  - tile_entity           │  - recipe.rs │
│  - profiler  │         └──────────────┘           │  - autotest  │
│  - crash_rep │                 │                  │  - chat.rs   │
│  - logger.rs │                 │                  └──────────────┘
└──────────────┘                 │                         │
     │                           │                         │
     ▼                           ▼                         ▼
┌──────────────┐         ┌──────────────┐           ┌──────────────┐
│  src/audio   │         │ src/graphics │           │   src/net    │
│  - engine.rs │         │  - traits.rs │           │ - protocol.rs│
│  - mixer.rs  │         │  - atlas.rs  │           │ - server.rs  │
│  - synth.rs  │         │  - software/ │           │ - client.rs  │
│  - spatial.rs│         │  - gpu/      │           │ - discovery  │
│  - backends  │         └──────────────┘           └──────────────┘
└──────────────┘                 ▲                         ▲
        │                        │                         │
        ▼                        ▼                         ▼
┌──────────────┐         ┌──────────────┐           ┌──────────────┐
│    src/ui    │◄────────┤ src/platform │           │  src/cli.rs  │
│  - hud.rs    │         │  - mouse_cap │           │  (CLI flags) │
│  - menu.rs   │         │  - gamepad.rs│           └──────────────┘
│  - inv_ui.rs │         │  - android.rs│
│  - settings  │         │  - xbox360.rs│
└──────────────┘         │  - vita/gxm  │
                         └──────────────┘
```

---

## 2. Directory Structure & Module Responsibilities

### `src/core/` (Core Math, Utilities & Diagnostics)
* **`math.rs`**: Foundational 3D linear algebra (`Vec3`, `IVec3`, `Mat4`). Contains vector operations, projection matrices (`perspective`, `look_at`, `ortho`), and matrix multiplications. Zero external dependencies.
* **`noise.rs`**: Deterministic 2D & 3D value noise and fractal octave noise (`fractal_noise2`, `fractal_noise3`). Used for procedural terrain generation and biome mapping.
* **`settings.rs`**: `GameSettings` struct persisted to disk — render distance, FOV, fog, clouds, particles, debug overlay, chunk fade animation, underwater FX, MSAA, FPS cap, leaf quality (`LeafQuality`: Fast/Near/Medium/Far/Fancy), audio volume sliders (Master, SFX, Music, Footsteps, Ambience), mute on focus loss, sprint/sneak/jump behavior modes (`SprintMode`, `SneakMode`, `JumpMode`), and remote profiler server port. Shared across all platforms.
* **`i18n.rs`**: Zero-dependency in-memory localization engine with runtime language switching between `locales/*.json` dictionaries.
* **`crash_reporter.rs`**: Comprehensive crash diagnostics and telemetry subsystem. Registers a global panic hook capturing stack traces, CPU thread state, active world coordinates, loaded chunks, and in-memory log breadcrumbs. Generates formatted crash logs on disk and optionally transmits JSON envelopes to Sentry via `--sentry-dsn` or `SENTRY_DSN` env var.
* **`logger.rs`**: Lock-free circular ring-buffer logger keeping the last 200 system events and log entries in memory as breadcrumbs for crash dumps.
* **`controls.rs`**: Centralized keybinding and action mapping abstraction for keyboard, mouse, and gamepads.
* **`profiler.rs`**: `MemoryBreakdown` struct (total RAM, chunk voxels, mesh cache, textures, framebuffers, entities, CDRAM in bytes) and `RenderStats` struct (pipeline name, renderer name, GPU device, draw calls, subchunks rendered/culled, triangle counts, rasterizer threads).
* **`profiler_server.rs`**: Embedded zero-dependency micro HTTP server. Hosts a live JSON API (`/api/profiler`) and HTML dashboard for real-time frame telemetry (`ProfilerSnapshot`): granular micro-timings for terrain render, transparent pass, clouds, sky, entity render, 3D hand, UI, physics tick, mob AI, world gen, mesh building. Configurable TCP port.

### `src/cli.rs` (Command-Line Interface)
* Parses `--renderer software|vulkan`, `--seed <NUMBER>`, `--profiler`, `--autotest`, `-log`, `--server`, `--port <PORT>`, `--connect <ADDR>`, `--sentry-dsn <DSN>`, `--no-sentry`, `--crash-test`, and `--help` flags.
* `generate_random_seed()`: Cryptographically strong 32-bit seed via `getrandom`, falling back to subsecond clock entropy.

### `src/platform/` (Platform & OS Abstractions)
* **`mouse_capture.rs`**: Cross-platform raw mouse input and cursor trapping:
  * **Linux (X11)**: Native `XGrabPointer`, `XWarpPointer`, `XFixesHideCursor` with dynamic library loading (`x11-dl`). Also handles `maximize_window` via Atom-based `_NET_WM_STATE_MAXIMIZED`.
  * **Windows (Win32)**: Native `ClipCursor`, `SetCursorPos`, `ShowCursor` via `windows-sys`.
  * **Fallback**: Delta-based relative mouse tracking for any windowing system without native grab.
* **`gamepad.rs`**: Full gamepad support via `gilrs` — button mapping, analog sticks, RT/LT triggers for block placement/destruction, and complete UI navigation.
* **`android.rs`**: Native Android platform layer — `ANativeWindow` hardware surface blitting with dynamic aspect-ratio geometry (preventing distortion on 20:9 and 21:9 displays), soft keyboard IME integration for chat and naming, multi-touch virtual controls (analog movement, camera drag, action buttons), USB/Bluetooth gamepad support, and Logcat diagnostics.
* **`xbox360.rs`**: Native Microsoft Xbox 360 port via libxenon — direct 720p 60 FPS VRAM scanout, big-endian pixel conversions, hardware wireless/USB controller support, and FAT32 USB/HDD save persistence.
* **`vita.rs`**: PlayStation Vita platform layer — `sceKernelLoadStartModule` bootstrap for SceDisplay/SceCtrl/SceTouch, ABGR8888 framebuffer format, `sceKernelGetProcessTimeWide()` random seed, SceTouch input mapping to mouse deltas and action buttons, and transition from software CPU blitting to the GXM hardware renderer.
* **`vita_gxm.rs`**: Complete PlayStation Vita GXM hardware renderer (see §4 below).

### `src/audio/` (Audio Engine, 3D Spatial Audio & Synthesis)
* **`engine.rs`**: `AudioEngine` lifecycle, channel orchestration, volume scaling (Master, SFX, Music, Footsteps, Ambience), and background audio thread coordination.
* **`mixer.rs`**: High-performance polyphonic mixer featuring a 64-voice pool, priority stealing, graceful voice release, and stereo room reverb with a persistent decay tail.
* **`synth.rs`**: 100% procedural waveform synthesis requiring zero external audio assets. Generates realistic sound synthesis for block digging and breaking (custom acoustic profiles for wood, stone, dirt, sand, glass, gravel), block placement, footsteps with terrain-matched cadence, jump, landing thuds, player damage, explosions, chest open/close, water splash/swimming, UI clicks, rain ambiance, distant thunder rumbles, lightning strikes, villager voices (idle murmur, trade, hurt, yes, no), and Iron Golem sounds (hit, walk, attack).
* **`soundtrack.rs`**: Built-in background soundtrack engine. Decodes and streams custom OGG / Opus audio tracks with non-repeating shuffle, category tagging (Creative, Nether, Ocean, Ambient), volume attenuation, and in-game music toast notifications (`MusicToast`) displayed upon track changes.
* **`spatial.rs`**: 3D spatial audio simulation — constant-power panning, distance attenuation, real-time voxel raycast acoustic occlusion (muffling through solid obstacles), and atmospheric air absorption low-pass filtering. The ambient probe estimates room enclosure, surface reflectivity, and reverb decay from nearby blocks.
* **`ring_buffer.rs`**: Lock-free single-producer single-consumer (SPSC) command ring buffer ensuring non-blocking audio dispatch from the main simulation thread.
* **`sound.rs`**: `SoundInstance`, `SoundId`, and procedural sound trigger definitions.
* **`backend/`**: Modular cross-platform audio drivers:
  * Linux (`desktop.rs`): PulseAudio / PipeWire with dynamic fallback to ALSA.
  * Windows (`desktop.rs`): Native waveOut / DirectSound.
  * Android (`android.rs`): Low-latency native AAudio backend with dynamic `libaaudio.so` loading.
  * PlayStation Vita (`vita.rs`): SceAudioOut hardware audio channels.
  * Xbox 360 (`xbox360.rs`): DirectSound / Xenon audio buffer.
  * WebAssembly (`wasm.rs`): Web Audio API integration via JavaScript AudioContext.
  * Null (`null.rs`): Headless audio fallback for automated testing and CI.

### `src/net/` (Multiplayer Networking)
* **`protocol.rs`**: Compact binary packet serialization over UDP for player positions, yaw/pitch models, voxel updates, combat hits, chat, mod definition exchange (`ServerPacket::ModSync`), and player health.
* **`server.rs`**: Dedicated authoritative multiplayer server, chunk broadcasting, world state simulation, and player synchronization.
* **`client.rs`**: Multiplayer network client with client-side prediction, delta reconciliation, and server packet ingestion.
* **`discovery.rs`**: Zero-configuration LAN server discovery using UDP broadcast beacons.

### `src/world/` (Voxel World, Meshing, Weather & Storage)
* **`block.rs`**: `BlockId` enum with **163 unique vanilla block/item types** (0..=162, plus modded types `BlockId::Custom(163..=255)`) — Air, terrain blocks (Grass, Dirt, Stone, Cobblestone, MossyCobblestone, Sand, Snow, Ice, Mud, Bedrock, Sandstone, SmoothStone, Bricks, Obsidian, Bookshelf), foliage (Leaves, SnowLeaves, CherryLeaves, AutumnLeaves, Cactus, Kelp, Seagrass, Sapling), fluids (Water, Lava), agriculture (Farmland, FarmlandWet, WheatSeeds, WheatCrop, Wheat, Bread, Carrot, CarrotCrop, Potato, PotatoCrop, BakedPotato, BoneMeal), 16 wool variants (White, Orange, Magenta, Light Blue, Yellow, Lime, Pink, Gray, Light Gray, Cyan, Purple, Blue, Brown, Green, Red, Black), RustStone mechanisms (RustStoneOre, RustStone wire, RustStoneBlock, RustStoneLamp/Lit, Lever/On, Button/Pressed, PressurePlate/Pressed), ores & minerals (CoalOre, IronOre, GoldOre, DiamondOre, Diamond, EmeraldOre, Emerald, EmeraldBlock, IronOunce, GoldOunce, IronIngot, GoldIngot, CoalBlock, IronBlock, GoldBlock, DiamondBlock, Clay, ClayBall, Brick, Gravel, Flint), stairs and slabs (Wood, Stone, Cobblestone, Sandstone, Brick), tools (5 tiers: Wooden/Stone/Iron/Gold/Diamond Pickaxe/Axe/Shovel/Sword/Hoe; Shears; FishingRod), items & food (Stick, Torch, Apple, Raw/Cooked Porkchop, Beef, Chicken, Cod, Salmon; Pufferfish, TropicalFish, DriedKelp, Feather, Bone, Gunpowder, Bed), spawn eggs (Pig, Cow, Chicken, Sheep, Dolphin, Fish, Villager, IronGolem), and functional blocks (CraftingTable, Door/DoorOpen, Glass, Tnt, Chest, Furnace/FurnaceLit, Planks). Block properties include solid, transparent, fluid, luminance, break time, face texture mappings.
* **`weather.rs`**: Dynamic weather simulation engine (`WeatherManager`). Manages transitions between `Clear`, `Rain`, and `Thunderstorm` states. Adapts precipitation to local biome temperature (temperate rainfall vs. high-altitude/cold-biome snowfall). Drives procedural atmospheric sky/cloud darkening, manages procedural rain ambiance and thunder synthesis, and spawns random lightning bolt strikes that illuminate the terrain with a flash overlay and strike ground voxels.
* **`mesher.rs`**: Chunk mesh builder (`build_chunk_mesh`). Extracts exposed faces, computes smooth Ambient Occlusion (AO) per vertex (0..3 corners), produces packed `Triangle` and `Vertex` buffers. Supports sub-chunk meshing (`build_subchunk_mesh`, `NUM_SUBCHUNKS`, `SUBCHUNK_SIZE`) and async background meshing (`AsyncMesher`, `ChunkMeshJob`). Exports mesh data compatible with the GXM GPU vertex format.
* **`storage.rs`**: Save and load world chunks and player position to disk with binary serialization. Supports custom paths for Android internal storage and Xbox 360 USB/HDD.
* **`tile_entity.rs`**: Tile entity data and simulation:
  * `ChestData`: 27-slot item storage with `add_item` stacking logic. `generate_house_loot(seed, pos)` fills cabin and village chests with deterministic starter loot.
  * `FurnaceData`: Full smelting simulation — input, fuel, output slots, `burn_time_remaining`, `burn_time_total`, `cook_progress`. `tick(dt)` advances burning and smelting. Returns `true` when lit/unlit state changes (triggers visual block swap Furnace ↔ FurnaceLit).
  * `smelting_result()`: Recipes: RawPorkchop→CookedPorkchop, RawBeef→CookedBeef, RawChicken→CookedChicken, RawCod→CookedCod, RawSalmon→CookedSalmon, Kelp→DriedKelp, Potato→BakedPotato, Sand→Glass, ClayBall→Brick, Stone→SmoothStone, Cobblestone→Stone, IronOre→IronIngot, GoldOre→GoldIngot, Wood→CoalOre (charcoal).
  * `fuel_burn_time()`: Burn times: Coal (80 s), Wood/Planks/CraftingTable/Chest (15 s), wooden tools (10 s), Stick (5 s), Leaves (2 s).
* **`mod.rs`**: `World` struct (chunks, seed, tile entities, TNT/explosions, fluid simulation, day/night cycle `time_of_day`, sim tick counter). **20-biome procedural generator** using temperature × humidity × continentalness noise axes: DeepOcean, Ocean, Swamp, Mountain, Desert, Savanna, AutumnForest, Forest, CherryGrove, Plains, SnowyTaiga, SnowyTundra, Jungle, Badlands, DarkForest, Taiga, BirchForest, IceSpikes, MushroomIsland, Meadow. Procedural structures: **villages** (Plains, Savanna, Desert) with paths, houses, farms, wells, lamp posts, chests; desert wells; stone ruins; forest log cabins; coral ruins; underwater kelp forests and seagrass fields. Raycasting (`RaycastHit`), acoustic occlusion checks, fluid flow, falling sand/gravel simulation, chunk streaming with async background loading.

### `src/gameplay/` (Entities, Player, Trading & Simulation)
* **`player.rs`**: Player entity state, `GameMode` (Survival / Creative), health, hunger and saturation simulation, invulnerability frames, fall damage tracking, and 3rd-person camera perspective.
* **`inventory.rs`**: Item stacking, slot management, hotbar (10 slots) and main inventory (27 slots).
* **`item.rs`**: `ItemEntity` physics for dropped items (floating, bouncing, pickup radius).
* **`physics.rs`**: AABB collision detection (`block_intersects_player`, `move_player_with_collision`), step jumping, crouching offsets, friction (ice, ground, air), underwater buoyancy, and lava drag.
* **`mob.rs`**: `MobManager`, entity state machines for **14 mob kinds** (`MobKind`: Pig, Cow, Chicken, Sheep, Zombie, Skeleton, Creeper, Dolphin, Cod, Salmon, Pufferfish, TropicalFish, Villager, IronGolem), pathfinding, panic/fleeing behavior when damaged, herd dynamics, explosive creeper fuses, sheep grazing/shearing, dolphin speed buff, Iron Golem protection & arm-swing knockback physics, and box models.
* **`trading.rs`**: Villager trading economy (`VillagerTrading`). Implements 8 villager professions (Farmer, Librarian, Armorer, Cleric, Butcher, Mason, Weapon Smith, Tool Smith), item and emerald exchange trades, stock limits, and 5-tier trade experience progression (Novice to Master).
* **`particles.rs`**: Block breaking debris, mob hit sparks, death poofs, explosion smoke, lava flame & smoke particles, water splash drops, rain splashes, and falling snowflakes.
* **`recipe.rs`**: Crafting recipe definitions for 2×2 (inventory) and 3×3 (crafting table) grids, including tools, hoes, shears, fishing rods, colored beds, bread, bricks, sandstone, cobblestone, stairs, slabs, bookshelves, ruststone mechanisms, and emerald blocks.
* **`autotest.rs`**: Automated headless testing suite and autonomous bot AI controller for integration testing, stress testing, and CI validation.
* **`chat.rs`**: In-game chat system, command execution (`/weather clear|rain|thunder`, `/give`, `/mobs`, `/tp`, `/time`), and multiplayer message dispatch.

### `src/graphics/` (Rendering Backends & Visuals)
* **`traits.rs`**: `RenderBackend` trait specifying standard interfaces for any renderer (pixel buffer, clear, render frame, dimensions).
* **`atlas.rs`**: Procedural pixel-art texture generator for blocks and mobs into a unified texture array (**170+ texture definitions**, each 128×128 px ABGR8888).
* **`camera.rs`**: First-person and 3rd-person camera (pitch, yaw, position, flat forward/right vectors, view matrix, dynamic FOV, chunk fade animation uniform).
* **`font.rs`**: Bitmap font renderer for text overlays with shadowing and measurement, plus support for modern high-resolution Unicode font rendering.
* **`software/`**: Multi-threaded CPU software rasterizer utilizing Rayon (tiled parallel scanline rasterization, sub-pixel precision, depth buffering, fog, per-biome post-process color filters, underwater tints, bilinear texture sampling, precipitation particles for rain and snow, lightning flash overlay). Block-break crack overlay, 3D dropped items, lava/underwater fog. Rayon is disabled on WASM targets.
* **`gpu/`**: WGPU hardware backend with compute shaders (SSBO readback / direct swapchain), supporting Vulkan, DirectX 12, Metal, and WebGPU.

### `src/ui/` (2D Interface & HUD)
* **`draw.rs`**: 2D primitives (`fill_rect`, `fill_rect_alpha`, `draw_text_shadowed`, `draw_text_colored_shadowed`, `pack_color`, `unpack_color`).
* **`menu.rs`**: Main menu layout, logo voxel art, save slot carousel, and interactive button hitboxes.
* **`hud.rs`**: Crosshair, block break progress bar, hotbar overlay, underwater/lava wave effect, damage vignette, death screen, debug statistics panel (including current biome name), pause menu, chat overlay, and music track toasts.
* **`minimap.rs`**: Real-time 2D top-down radar and compass minimap displaying terrain surface voxels, player position, and orientation pointer.
* **`inventory_ui.rs`**: Tabbed creative picker with category pagination (Blocks, Tools, Items, All), quick-access hotbar, 2D inventory grid, and crafting table interface (2×2 / 3×3).
* **`container_ui.rs`**: Interactive UI for tile entities and trading — Chest inventory (27-slot drag & drop), Furnace interface (input / fuel / output slots with animated progress bars), and Villager Trading interface (trade offers, emerald pricing, output reward slot, trader experience bar). Handles slot hit testing, mouse drag, and item transfer.
* **`create_world.rs`**: Create World form — name text field, seed text field with random-seed button, game mode toggle (Survival/Creative), create and cancel buttons. Provides `create_world_rects()` for accurate click-zone layout.
* **`save_select.rs`**: Save slot selection screen.
* **`clipboard.rs`**: Cross-platform clipboard helper — tries `wl-paste` (Wayland), falls back to `xclip` (X11), then falls back to an in-process static string buffer.
* **`settings_ui.rs`**: Interactive tabbed settings menu — Video settings (particles, FOV, render distance, clouds, fog, leaf quality, MSAA, FPS cap), Audio settings (Master, SFX, Music, Footsteps, Ambience, Mute on focus loss), Controls (invert-Y/Z, sensitivity), Unicode font toggle, and remote profiler server toggle.

### `src/web.rs` (WebAssembly / HTML5 Port)
* `wasm-bindgen` bindings exposing `RustCraftApp` to JavaScript.
* Zero-copy framebuffer blitting to an HTML5 `<canvas>` element.
* Pointer Lock API integration, Gamepad API support, and a modern CSS frontend (`web/index.html`, `web/app.js`, `web/style.css`).
* Rayon multi-threading is conditionally disabled (`#[cfg(not(target_arch = "wasm32"))]`).

---

## 3. Key Design Patterns & Invariants
1. **Zero Allocations in Hot Paths**: Render loops and physics updates reuse pre-allocated vectors (`opaque_tris_buf`, `dirty_keys`, `loaded_keys`, `debug_lines`).
2. **Abstract Rendering Pipeline**: The game logic only interacts with `World`, `Camera`, and `RenderBackend`. New rendering backends can be added without touching game simulation.
3. **Platform Independence**: Native OS code is strictly isolated inside `src/platform/`. The Vita GXM renderer, WASM bindings, and desktop mouse capture are all additive modules — the core never branches on target OS.
4. **Settings Persistence**: `GameSettings` is serialized to disk and shared across all platforms, with platform-appropriate paths (e.g., `ux0:data/` on Vita).
5. **Shader Pipeline**: On Vita, Cg shaders (`vita/shaders/voxel.vert.cg`, `voxel.frag.cg`) are compiled to GXP format by `psp2cgc` at build time via `build.rs`. The build script auto-detects the VitaSDK toolchain and falls back to `vita/shaders/prebuilt/` if the compiler is absent.
6. **Tile Entity System**: Chest and Furnace block states are stored in `World::chests` and `World::furnaces` (HashMap keyed by `IVec3`) separately from the voxel grid — only the `BlockId` in the chunk array indicates presence, while all interactive data lives in the tile entity maps.
7. **Profiler Feature Flag**: Performance telemetry in `src/core/profiler.rs` and `profiler_server.rs` is gated by the `profiler` Cargo feature and the `--profiler` CLI flag, adding zero overhead in release builds without the flag.

---

## 4. PlayStation Vita Port (`src/platform/vita_gxm.rs`)

RustCraft runs natively on the PlayStation Vita using the **SCE GXM** low-level GPU API targeting the **PowerVR SGX543MP4** GPU.

### GXM Renderer Overview
| Feature | Details |
|---|---|
| GPU | PowerVR SGX543MP4 (4-core) |
| Render target | CDRAM triple-buffered (3× color + depth, async display swap callback) |
| Vertex/index buffers | 2 MB each, uncached GPU memory (UC) |
| Ring buffers | VDM + vertex + fragment, 3 MB total |
| Texture atlas | 140+ textures → 128×128 px ABGR8888, single GPU texture upload |
| Shaders | Cg vertex + fragment (`.cg` → `.gxp` via `psp2cgc` in `build.rs`) |
| Culling | Backface culling (CW winding), depth test `LESS_EQUAL` |
| Display | `SceDisplay` pitch-mode framebuffer swap (960×544) |

### GXM Rendering Features
* **3D world chunks** — all visible chunks drawn in a single `BeginScene` → `draw_chunks` → `EndScene` pass.
* **3D dropped items & mobs** — rendered using the GXM pipeline (no CPU fallback).
* **3D hand / tool** — first-person hand model drawn via GXM with correct depth.
* **Water transparency** — alpha blending pass for transparent blocks.
* **Screen overlays** — damage flash, lava/underwater fog drawn efficiently via GXM uniforms to avoid CDRAM readback stalls.
* **Chunk fade/slide-in animation** — smooth cubic ease-out driven 100% by GPU uniform (`u_chunk_offset`), zero CPU overhead.
* **Performance profiler** — 250 ms periodic sampling of CPU time, GPU time, RAM, CDRAM usage, and chunk slot count, displayed in the debug panel.

### Build Pipeline
```
build.rs  ──psp2cgc──►  vita/shaders/*.gxp   (compiled Cg shaders)
              │
              └─ fallback: vita/shaders/prebuilt/*.gxp  (if no VitaSDK)
```
Package output: `RustCraft.vpk` (installable on Vita via VitaShell).
Build script: `build_vita.sh` (cross-compiles for `armv7-sony-vita-newlibeabihf`, runs `vita-elf-create` + `vita-make-fself`).

### Input Mapping (Vita)
| Vita Control | Action |
|---|---|
| Left stick | Move (WASD equivalent) |
| Right stick | Camera look |
| ×  | Jump |
| ○  | Place block |
| □  | Destroy block |
| R trigger (RT) | Destroy block (alternate) |
| L trigger (LT) | Place block (alternate) |
| Start | Pause menu |
| D-Pad | Hotbar / UI navigation |
| Touch panel | Mouse-delta camera look |

---

## 5. Biome System (`src/world/mod.rs`)

The world generator selects a biome per column using three independent fractal noise axes:
- **Continentalness** (800–1000 block period): Controls ocean vs land vs mountain.
- **Temperature** (1000 block period): Controls cold/hot axis (snowy ↔ desert).
- **Humidity** (1000 block period): Controls dry/wet axis (savanna ↔ swamp/forest).

| Biome | Terrain | Decoration |
|---|---|---|
| Plains | Flat grass, occasional hills | Oak trees, tall grass, flowers |
| Forest | Rolling hills | Dense oak trees |
| Autumn Forest | Rolling hills | Orange-red AutumnLeaves oak trees |
| Birch Forest | Temperate woodland hills | Birch trees with white bark |
| Dark Forest | Shaded forest floor | Thick dark oak canopy trees |
| Taiga | Cool evergreen hills | Coniferous pine trees |
| Snowy Taiga | Frozen snow-covered terrain | Pine trees with snow layers |
| Snowy Tundra | White snow-capped flat | Pine trees and ice lakes |
| Ice Spikes | Rare glacial ice field | Towering packed ice spikes, snow ground |
| Jungle | Tropical rainforest, high humidity | Giant jungle trees, vine canopies |
| Badlands | Arid canyon terrain | Banded terracotta/sand/mud strata, surface gold ore |
| Desert | Sandy flat terrain, base y=25 | Cacti, **desert wells** |
| Savanna | Dry flat plains | Acacia-style wide-crown trees |
| Swamp | Low waterlogged terrain | Swamp trees, Mud blocks |
| Mushroom Island | Rare mud ocean island | Giant red mushroom trees, no monsters |
| Meadow | High-altitude grassy plateau | Solitary trees, flowers |
| Cherry Blossom | Gentle hills | Wide-crowned cherry trees (`CherryLeaves`) |
| Mountains | High stone peaks, scaled by continentalness | Sparse pine trees, **stone ruins** |
| Ocean | Flat sandy seabed, water to y=20 | **Coral ruins** |
| Deep Ocean | Very low flat seabed | Deep oceanic trenches |

Active biome is displayed in the debug panel via `World::biome_name_at(wx, wz)`.

---

## 6. Supported Platforms & Build Targets

| Platform / Architecture | Target Triple | Renderer | Build Script / Command |
|---|---|---|---|
| Linux (x86_64) | `x86_64-unknown-linux-gnu` | Software CPU / Vulkan | `./compile_linux.sh` |
| Linux (32-bit x86 / i686) | `i686-unknown-linux-gnu` | Software CPU / Vulkan | `./build_linux_x86_32.sh` |
| Linux ARM64 (RPi 4/5) | `aarch64-unknown-linux-gnu` | Software CPU / Vulkan | `./build_linux_arm64.sh` |
| Linux ARMv7 (RPi 2/3 32-bit) | `armv7-unknown-linux-gnueabihf` | Software CPU / Vulkan | `./build_linux_armv7.sh` |
| Linux RISC-V 64-bit | `riscv64gc-unknown-linux-gnu` | Software CPU (Rayon) | `./build_linux_riscv64.sh` |
| Raspberry Pi Zero W (ARMv6) | `arm-unknown-linux-gnueabihf` | Software CPU (Tuned for ARM11) | `./build_rpi_armv6.sh` |
| FreeBSD (x86_64) | `x86_64-unknown-freebsd` | Software CPU / X11 | `./build_freebsd.sh` |
| Windows (x86_64) | `x86_64-pc-windows-gnu` | Software CPU / Vulkan | `./compile-windows.sh` |
| Windows (32-bit x86 / i686) | `i686-pc-windows-gnu` | Software CPU / Vulkan | `./build_windows_32.sh` |
| Windows (ARM64 / AArch64) | `aarch64-pc-windows-msvc` | Software CPU / Vulkan | `./build_windows_arm64.sh` |
| Android (ARM64 / ARMv7 / x86_64) | `aarch64-linux-android` | ANativeWindow (Software / Rayon) | `./build_android.sh` |
| Microsoft Xbox 360 (Homebrew XEX2) | `powerpc-unknown-linux-gnu` / Xenon | Direct VRAM Framebuffer (720p HD) | `./build_xbox360.sh` |
| WebAssembly | `wasm32-unknown-unknown` | Software CPU (Single-threaded) | `./build_web.sh` |
| PlayStation Vita | `armv7-sony-vita-newlibeabihf` | Bare-Metal GXM GPU (SGX543MP4) | `./build_vita.sh` |
| macOS | `x86_64-apple-darwin` / `aarch64` | Software CPU / Metal (WGPU) | `cargo run --release` |

---

## 7. Engine API & Extensibility (`src/engine/`)

RustCraft provides an explicit, documented engine trait layer to decouple game simulation from platforms and enable rapid extension.

### Core Engine Subsystems
- **`PlatformBackend`** (`src/engine/platform.rs`): Abstraction for window creation, raw event polling, mouse capture, timer queries, and framebuffer presentation. Adding support for a new console (e.g. Nintendo Switch `libnx`, Android `NDK`, SDL2) only requires implementing this trait.
- **`BlockRegistry` & `BlockProperties`** (`src/engine/registry.rs`): Extensible voxel metadata (hardness, opacity, luminance, flammability, face texture mapping) allowing new custom blocks to be registered without altering mesher or rendering loops.
- **`Modding Engine`** (`src/engine/modding.rs`): Data-driven JSON mod loader (`mods/`) supporting custom blocks, items, nutrition food, recipes, smelting, mob loot tables, and chat commands, with automatic client synchronization over multiplayer (Source engine style).
- **`EngineModule` & `ModuleManager`** (`src/engine/module.rs`): Pluggable lifecycle hooks (`on_init`, `on_pre_update`, `on_post_update`, `on_shutdown`) enabling custom gameplay systems, mod loaders, and analytics passes.
- **`EngineApp`** (`src/engine/mod.rs`): Top-level engine runtime contract implemented by `RustCraftApp` (`update`, `render`, `resize`, `pixel_buffer`).
- **`BitmapFont` & `FontId`** (`src/engine/fonts/mod.rs`): Trait-based font system with `get_font(FontId)` registry. Built-in fonts: `RustcraftRegular` (3×5 compact, with Polish diacritics and symbols) and `RustcraftLight` (3×5 slender, optimized for profiler telemetry and diagnostic overlays). Adding a new font requires only implementing the `BitmapFont` trait and registering a `FontId` variant.
