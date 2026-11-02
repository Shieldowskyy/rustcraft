# Agent Guidelines & Quickstart for RustCraft

Welcome, Agent. This repository contains **RustCraft**, a voxel game and engine written in Rust with desktop, Android, WebAssembly, PS Vita, and Xbox 360 build targets. Runtime support varies by platform.

Refer to [LLM_GUIDE.md](LLM_GUIDE.md) and [ARCHITECTURE.md](ARCHITECTURE.md) for repository documentation.

---

## 1. Top Gotchas & Pitfalls (Read Before Editing)

### A. WGPU Texture Array Limits
- **Location**: [src/graphics/gpu/mod.rs](src/graphics/gpu/mod.rs) and [src/graphics/atlas.rs](src/graphics/atlas.rs).
- **The Issue**: By default, `wgpu::Limits::default().max_texture_array_layers` is hardcoded to **256**.
- **Rule**:
  1. Whenever adding textures, keep `TEXTURE_LAYER_COUNT` and the custom layer range in sync.
  2. `device.request_device` MUST pass `required_limits.max_texture_array_layers = adapter_limits.max_texture_array_layers`.
  3. Keep bidirectional mappings in sync: `layer_from_texture_id` and `texture_id_from_layer`.
  4. Verify with:
     ```bash
     cargo test graphics::gpu::tests::test_vulkan_context_texture_array_creation
     ```

### B. Do NOT Launch Interactive `cargo run` in Headless Mode
- `cargo run` launches an interactive desktop window via `minifb` and blocks waiting for user input.
- **To test changes safely without freezing**:
  ```bash
  # Check compilation:
  cargo check

  # Run headless unit tests (including offscreen GPU tests):
  cargo test

  # Build release executable:
  cargo build --release
  ```

### C. Panics & Crash Reports
- All unhandled panics generate a crash report with breadcrumbs and stacktraces in `crash-reports/`:
  ```bash
  cat $(ls -t crash-reports/crash-*.txt | head -n 1)
  ```
- Before guessing, read the latest crash report to see the exact thread, panic message, and line number.

### D. Renderer Switching & Fallback
- RustCraft supports multiple backends:
  - `--renderer vulkan`: Hardware WGPU Vulkan (default on desktop).
  - `--renderer opengl`: Hardware OpenGL / GLES.
  - `--renderer software`: Multithreaded CPU software rasterizer (fallback if GPU fails).
- Override WGPU backend via environment variable: `WGPU_BACKEND=vulkan|gl|dx12|metal`.

### E. Zero-Allocation in Render Loop
- Never allocate inside the per-frame or per-pixel render loops in `src/graphics/software/`.
- Re-use buffers (scratch triangles, strings, vectors).

---

## 2. Fast Navigation Table

| Component | Files | What to know |
|---|---|---|
| **Blocks** | `src/world/block.rs`, `src/graphics/atlas.rs` | Built-in IDs are 0..=185; mod IDs are 186..=249, matching the 64 custom texture layers. Map to `TextureId`. |
| **Textures** | `src/graphics/atlas.rs`, `src/graphics/gpu/mod.rs` | Update `TextureId`, `generate()`, both layer mappings, and `TEXTURE_LAYER_COUNT`. |
| **Mobs** | `src/gameplay/mob.rs`, `src/graphics/software/mod.rs` | Add variant to `MobKind`, define hitbox & AI tick, add 3D box hierarchy in `draw_mob`. |
| **World Gen** | `src/world/mod.rs`, `src/core/noise.rs` | 20 biomes via noise axes in `get_biome_at()`. |
| **Recipes** | `src/gameplay/recipe.rs`, `src/world/tile_entity.rs` | 2x2 and 3x3 crafting recipes, furnace smelting. |
| **UI** | `src/ui/` (`hud.rs`, `inventory_ui.rs`, `minimap.rs`) | Use `src/ui/draw.rs` primitives (`fill_rect_alpha`, `draw_text_shadowed`). |
| **Audio** | `src/audio/synth.rs`, `src/audio/sound.rs` | Procedural synthesizer. No external WAV/MP3 files needed. |
