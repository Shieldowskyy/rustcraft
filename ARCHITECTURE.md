# RustCraft Architecture & Modular Design

## 1. Overview
RustCraft is a lightweight, voxel-based game engine and sandbox built with Rust. It supports both a multi-threaded CPU software rasterizer and a GPU hardware pipeline (WGPU / Vulkan) with seamless runtime switching.

The codebase is organized into isolated, single-responsibility modules to maximize maintainability, ease cross-platform porting, and allow rapid feature development.

```
                  ┌──────────────────────┐
                  │       main.rs        │  (App Loop, Input Glue)
                  └──────────┬───────────┘
                             │
     ┌───────────────────────┼────────────────────────┐
     ▼                       ▼                        ▼
┌──────────────┐      ┌──────────────┐         ┌──────────────┐
│  src/core    │      │  src/world   │         │ src/gameplay │
│  - math.rs   │      │  - block.rs  │         │  - player.rs │
│  - noise.rs  │      │  - mesher.rs │         │  - mob.rs    │
└──────────────┘      │  - storage.rs│         │  - phys.rs   │
                      └──────────────┘         └──────────────┘
                             │                        │
     ┌───────────────────────┴────────────────────────┴┐
     ▼                                                 ▼
┌──────────────┐                                ┌──────────────┐
│ src/graphics │                                │    src/ui    │
│  - traits.rs │  (RenderBackend)               │  - menu.rs   │
│  - atlas.rs  │                                │  - hud.rs    │
│  - software/ │  (CPU Rasterizer)              │  - draw.rs   │
│  - gpu/      │  (WGPU / Vulkan SSBO)          └──────────────┘
└──────────────┘
     ▲
     │
┌──────────────┐
│ src/platform │  (Mouse Capture X11 / Win32 / Fallback)
└──────────────┘
```

---

## 2. Directory Structure & Module Responsibilities

### `src/core/` (Core Math & Utilities)
* **`math.rs`**: Foundational 3D linear algebra (`Vec3`, `IVec3`, `Mat4`). Contains vector operations, projection matrices (`perspective`, `look_at`, `ortho`), and matrix multiplications. Zero external dependencies.
* **`noise.rs`**: Deterministic 2D & 3D value noise and fractal octave noise (`fractal_noise2`, `fractal_noise3`). Used for procedural terrain generation and biomes.

### `src/platform/` (Platform & OS Abstractions)
* **`mouse_capture.rs`**: Cross-platform raw mouse input and cursor trapping:
  * **Linux (X11)**: Native `XGrabPointer`, `XWarpPointer`, `XFixesHideCursor` with dynamic library loading (`x11-dl`).
  * **Windows (Win32)**: Native `ClipCursor`, `SetCursorPos`, `ShowCursor` via `windows-sys`.
  * **Fallback**: Delta-based relative mouse tracking for any windowing system without native grab.
  * **`maximize_window`**: Native X11/Win32 window state manipulation.

### `src/world/` (Voxel World, Meshing & Storage)
* **`block.rs`**: `BlockId` enum (Air, Grass, Dirt, Stone, Sand, Wood, Leaves, Water, Lava, Door, Glass, Tnt, etc.), block properties (solid, transparent, fluid, light, break time, face colors, and texture mappings).
* **`mesher.rs`**: Chunk mesh builder (`build_chunk_mesh`). Extracts exposed faces, computes smooth Ambient Occlusion (AO) per vertex (0..3 corners), and produces packed `Triangle` and `Vertex` buffers.
* **`storage.rs`**: Save and load world chunks and player position to disk with binary serialization.
* **`mod.rs`**: Chunk indexing (`((x << 10) | (y << 4) | z)`), infinite/bounded chunk streaming (`World`), material simulation (falling sand, fluid flow), and raycasting.

### `src/gameplay/` (Entities, Player & Simulation)
* **`player.rs`**: Player entity state, `GameMode` (Survival / Creative), health, invulnerability frames, and fall damage tracking.
* **`inventory.rs`**: Item stacking, slot management, hotbar (10 slots) and main inventory (27 slots).
* **`item.rs`**: `ItemEntity` physics for dropped items (floating, bouncing, pickup radius).
* **`physics.rs`**: AABB collision detection (`block_intersects_player`, `move_player_with_collision`), step jumping, crouching offsets, and friction (ice, ground, air).
* **`mob.rs`**: `MobManager`, entity state machines (Pig, Zombie), pathfinding, melee attacks, and box models.
* **`particles.rs`**: Block breaking debris, mob hit sparks, death poofs, explosion smoke.

### `src/graphics/` (Rendering Backends & Visuals)
* **`traits.rs`**: `RenderBackend` trait specifying standard interfaces for any renderer (pixel buffer, clear, render frame, dimensions).
* **`atlas.rs`**: Procedural pixel-art texture generator for blocks and mobs into a unified texture array.
* **`camera.rs`**: First-person 3D camera (pitch, yaw, position, flat forward/right vectors, view matrix, dynamic FOV).
* **`font.rs`**: Bitmap font renderer for text overlays with shadowing and measurement.
* **`software/`**: Multi-threaded CPU software rasterizer utilizing Rayon (tiled parallel scanline rasterization, sub-pixel precision, depth buffering, fog, underwater tints, and bilinear texture sampling).
* **`gpu/`**: WGPU hardware backend with compute shaders (SSBO readback / direct swapchain), supporting Vulkan, DirectX 12, Metal, and WebGPU.

### `src/ui/` (2D Interface & HUD)
* **`draw.rs`**: 2D primitives (`fill_rect`, `fill_rect_alpha`, `draw_text_shadowed`, `draw_text_colored_shadowed`, `pack_color`, `unpack_color`).
* **`menu.rs`**: Main menu layout, logo voxel art, save slot carousel, and interactive button hitboxes.
* **`hud.rs`**: Crosshair, block break progress bar, hotbar overlay, underwater wave effect, debug statistics panel, pause menu.
* **`inventory_ui.rs`**: 2D inventory grid and item selection interface.

---

## 3. Key Design Patterns & Invariants
1. **Zero Allocations in Hot Paths**: Render loops and physics updates reuse pre-allocated vectors (`opaque_tris_buf`, `dirty_keys`, `loaded_keys`, `debug_lines`).
2. **Abstract Rendering Pipeline**: The game logic only interacts with `World`, `Camera`, and `RenderBackend`. New rendering backends can be added without touching game simulation.
3. **Platform Independence**: Native OS code is strictly isolated inside `src/platform/`.
