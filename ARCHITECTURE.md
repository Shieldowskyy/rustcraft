# RustCraft Architecture & Modular Design

## 1. Overview
RustCraft is a lightweight, voxel-based game engine and sandbox built with Rust. It supports a multi-threaded CPU software rasterizer, a GPU hardware pipeline (WGPU / Vulkan), a dedicated PlayStation Vita GXM hardware renderer (SGX543MP4), and a WebAssembly/HTML5 port — all sharing the same core game logic through a platform-agnostic architecture.

The codebase is organized into isolated, single-responsibility modules to maximize maintainability, ease cross-platform porting, and allow rapid feature development.

```
                  ┌──────────────────────┐
                  │    main.rs / lib.rs  │  (App Loop, Input Glue, WASM entry)
                  └──────────┬───────────┘
                             │
     ┌───────────────────────┼────────────────────────┐
     ▼                       ▼                        ▼
┌──────────────┐      ┌──────────────┐         ┌──────────────┐
│  src/core    │      │  src/world   │         │ src/gameplay │
│  - math.rs   │      │  - block.rs  │         │  - player.rs │
│  - noise.rs  │      │  - mesher.rs │         │  - mob.rs    │
│  - settings.rs│     │  - storage.rs│         │  - phys.rs   │
└──────────────┘      └──────────────┘         └──────────────┘
                             │                        │
     ┌───────────────────────┴────────────────────────┴┐
     ▼                                                 ▼
┌──────────────┐                                ┌──────────────┐
│ src/graphics │                                │    src/ui    │
│  - traits.rs │  (RenderBackend)               │  - menu.rs   │
│  - atlas.rs  │                                │  - hud.rs    │
│  - software/ │  (CPU Rasterizer)              │  - draw.rs   │
│  - gpu/      │  (WGPU / Vulkan SSBO)          │  - settings_ui│
└──────────────┘                                └──────────────┘
     ▲
     │
┌─────────────────────────────────────────────────┐
│                  src/platform                   │
│  - mouse_capture.rs  (X11 / Win32 / Fallback)   │
│  - gamepad.rs        (gilrs gamepad support)    │
│  - vita.rs           (PS Vita display & input)  │
│  - vita_gxm.rs       (PS Vita GXM GPU renderer) │
└─────────────────────────────────────────────────┘
     ▲
     │
┌──────────────┐
│   src/web.rs │  (WASM / HTML5 canvas bindings)
└──────────────┘
```

---

## 2. Directory Structure & Module Responsibilities

### `src/core/` (Core Math & Utilities)
* **`math.rs`**: Foundational 3D linear algebra (`Vec3`, `IVec3`, `Mat4`). Contains vector operations, projection matrices (`perspective`, `look_at`, `ortho`), and matrix multiplications. Zero external dependencies.
* **`noise.rs`**: Deterministic 2D & 3D value noise and fractal octave noise (`fractal_noise2`, `fractal_noise3`). Used for procedural terrain generation and biomes.
* **`settings.rs`**: `GameSettings` struct persisted to disk — render distance, FOV, fog, clouds, particles, debug overlay, chunk fade animation, and underwater FX toggles. Shared across all platforms.

### `src/platform/` (Platform & OS Abstractions)
* **`mouse_capture.rs`**: Cross-platform raw mouse input and cursor trapping:
  * **Linux (X11)**: Native `XGrabPointer`, `XWarpPointer`, `XFixesHideCursor` with dynamic library loading (`x11-dl`).
  * **Windows (Win32)**: Native `ClipCursor`, `SetCursorPos`, `ShowCursor` via `windows-sys`.
  * **Fallback**: Delta-based relative mouse tracking for any windowing system without native grab.
  * **`maximize_window`**: Native X11/Win32 window state manipulation.
* **`gamepad.rs`**: Full gamepad support via `gilrs` — button mapping, analog sticks, RT/LT triggers for block placement/destruction, and complete UI navigation.
* **`vita.rs`**: PlayStation Vita platform layer — `sceKernelLoadStartModule` bootstrap for SceDisplay/SceCtrl/SceTouch, ABGR8888 framebuffer format, `sceKernelGetProcessTimeWide()` random seed, SceTouch input mapping to mouse deltas and action buttons, and transition from software CPU blitting to the GXM hardware renderer.
* **`vita_gxm.rs`**: Complete PlayStation Vita GXM hardware renderer (see §4 below).

### `src/world/` (Voxel World, Meshing & Storage)
* **`block.rs`**: `BlockId` enum (Air, Grass, Dirt, Stone, Sand, Wood, Leaves, Water, Lava, Door, Glass, Tnt, CherryLeaves, etc.), block properties (solid, transparent, fluid, light, break time, face colors, and texture mappings).
* **`mesher.rs`**: Chunk mesh builder (`build_chunk_mesh`). Extracts exposed faces, computes smooth Ambient Occlusion (AO) per vertex (0..3 corners), and produces packed `Triangle` and `Vertex` buffers. Exports mesh data compatible with the GXM GPU vertex format.
* **`storage.rs`**: Save and load world chunks and player position to disk with binary serialization. On Vita, uses `ux0:data/RustCraft/` via `SceIoStat`.
* **`mod.rs`**: Chunk indexing (`((x << 10) | (y << 4) | z)`), infinite/bounded chunk streaming (`World`), material simulation (falling sand, fluid flow), raycasting, and biome detection (`biome_name_at(wx, wz)`).

### `src/gameplay/` (Entities, Player & Simulation)
* **`player.rs`**: Player entity state, `GameMode` (Survival / Creative), health, invulnerability frames, and fall damage tracking.
* **`inventory.rs`**: Item stacking, slot management, hotbar (10 slots) and main inventory (27 slots).
* **`item.rs`**: `ItemEntity` physics for dropped items (floating, bouncing, pickup radius).
* **`physics.rs`**: AABB collision detection (`block_intersects_player`, `move_player_with_collision`), step jumping, crouching offsets, friction (ice, ground, air), and lava drag.
* **`mob.rs`**: `MobManager`, entity state machines (Pig, Zombie), pathfinding, melee attacks, and box models.
* **`particles.rs`**: Block breaking debris, mob hit sparks, death poofs, explosion smoke, lava flame & smoke particles.
* **`recipe.rs`**: Crafting recipe definitions for 2×2 (inventory) and 3×3 (crafting table) grids.

### `src/graphics/` (Rendering Backends & Visuals)
* **`traits.rs`**: `RenderBackend` trait specifying standard interfaces for any renderer (pixel buffer, clear, render frame, dimensions).
* **`atlas.rs`**: Procedural pixel-art texture generator for blocks and mobs into a unified texture array. Exposes `texture_for(TextureId)` used by the GXM backend to upload a single 128×128 px ABGR8888 GPU texture atlas. `TEXTURE_LAYER_COUNT` is 50.
* **`camera.rs`**: First-person 3D camera (pitch, yaw, position, flat forward/right vectors, view matrix, dynamic FOV, and cubic ease-out chunk fade animation uniform).
* **`font.rs`**: Bitmap font renderer for text overlays with shadowing and measurement.
* **`software/`**: Multi-threaded CPU software rasterizer utilizing Rayon (tiled parallel scanline rasterization, sub-pixel precision, depth buffering, fog, underwater tints, bilinear texture sampling). Block-break crack overlay, 3D dropped items, lava/underwater fog. Rayon is disabled on WASM targets.
* **`gpu/`**: WGPU hardware backend with compute shaders (SSBO readback / direct swapchain), supporting Vulkan, DirectX 12, Metal, and WebGPU.

### `src/ui/` (2D Interface & HUD)
* **`draw.rs`**: 2D primitives (`fill_rect`, `fill_rect_alpha`, `draw_text_shadowed`, `draw_text_colored_shadowed`, `pack_color`, `unpack_color`). On Vita, screen overlays are rendered via GXM's CDRAM-efficient path to avoid readback stalls.
* **`menu.rs`**: Main menu layout, logo voxel art, save slot carousel, and interactive button hitboxes.
* **`hud.rs`**: Crosshair, block break progress bar, hotbar overlay, underwater/lava wave effect, damage vignette, death screen, debug statistics panel (including current biome name), pause menu.
* **`inventory_ui.rs`**: Full Minecraft-style creative mode block picker (scrollable grid of all blocks), quick-access hotbar, 2D inventory grid, and crafting table interface (2×2 / 3×3).
* **`settings_ui.rs`**: Interactive settings menu with toggles for particles, debug overlay, chunk animation, FOV slider, render distance, clouds, fog, and underwater FX.
* **`save_select.rs`**: Save slot selection screen.

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
| Texture atlas | 50 blocks → 128×128 px ABGR8888, single GPU texture upload |
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

The world generator selects a biome per column using `fractal_noise2` on a low-frequency biome map:

| Biome | Terrain | Decoration |
|---|---|---|
| Plains | Flat grass, occasional hills | Oak trees, tall grass |
| Forest | Rolling hills | Dense oak trees |
| Desert | Sandy flat terrain | Cacti |
| Mountains | High stone peaks | Sparse pine trees |
| Snowy | White snow-capped | Spruce trees |
| Cherry Blossom | Gentle hills | Wide-crowned cherry trees with pink (`CherryLeaves`) foliage |
| Ocean | Flat sandy seabed, water to y=20 | No decoration |
| Savanna | Dry flat plains | Acacia-style wide-crown trees |

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
| WebAssembly | `wasm32-unknown-unknown` | Software CPU (Single-threaded) | `./build_web.sh` |
| PlayStation Vita | `armv7-sony-vita-newlibeabihf` | Bare-Metal GXM GPU (SGX543MP4) | `./build_vita.sh` |
| macOS | `x86_64-apple-darwin` / `aarch64` | Software CPU / Metal (WGPU) | `cargo run --release` |

---

## 7. Engine API & Extensibility (`src/engine/`)

RustCraft provides an explicit, documented engine trait layer to decouple game simulation from platforms and enable rapid extension.

### Core Engine Subsystems
- **`PlatformBackend`** (`src/engine/platform.rs`): Abstraction for window creation, raw event polling, mouse capture, timer queries, and framebuffer presentation. Adding support for a new console (e.g. Nintendo Switch `libnx`, Android `NDK`, SDL2) only requires implementing this trait.
- **`BlockRegistry` & `BlockProperties`** (`src/engine/registry.rs`): Extensible voxel metadata (hardness, opacity, luminance, flammability, face texture mapping) allowing new custom blocks to be registered without altering mesher or rendering loops.
- **`EngineModule` & `ModuleManager`** (`src/engine/module.rs`): Pluggable lifecycle hooks (`on_init`, `on_pre_update`, `on_post_update`, `on_shutdown`) enabling custom gameplay systems, mod loaders, and analytics passes.
- **`EngineApp`** (`src/engine/mod.rs`): Top-level engine runtime contract implemented by `RustCraftApp` (`update`, `render`, `resize`, `pixel_buffer`).

