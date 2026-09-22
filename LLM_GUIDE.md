# LLM & Developer Maintenance Guide for RustCraft

This guide is designed for LLMs (and developers) to understand the codebase structure quickly, locate relevant modules, adhere to conventions, and safely add new features.

---

## 1. Fast Directory Reference

| To modify / add: | Primary file/module: | Notes: |
|---|---|---|
| **New Block Type** | `src/world/block.rs`, `src/graphics/atlas.rs` | Add variant to `BlockId`, assign `TextureId`, add texture generator in `atlas.rs`. |
| **New Mob Type** | `src/gameplay/mob.rs` | Add variant to `MobKind`, define bounding boxes & AI tick, add draw call in software renderer. |
| **New UI Screen / Element** | `src/ui/` (`hud.rs`, `menu.rs`, `inventory_ui.rs`) | Use functions in `src/ui/draw.rs` (`fill_rect_alpha`, `draw_text_shadowed`). |
| **Player / Physics / Movement** | `src/gameplay/physics.rs`, `player.rs` | Physics uses AABB sweeps. Maintain step-up and crouching checks. |
| **World Generation / Biomes** | `src/world/mod.rs` (in `Chunk::generate`) | Uses `fractal_noise2` from `src/core/noise.rs`. |
| **New Graphics Backend** | `src/graphics/traits.rs`, `src/graphics/gpu/` | Implement `RenderBackend` trait. |
| **Platform / OS Input Hook** | `src/platform/mouse_capture.rs` | Keep OS-specific code guarded by `#[cfg(target_os = ...)]`. |

---

## 2. Common Workflows

### How to Add a New Block
1. **Define Block**: Open `src/world/block.rs` and add a new variant to `pub enum BlockId`.
2. **Configure Properties**: In `src/world/block.rs`, update:
   - `from_u8(v: u8)` and `to_u8(self)`
   - `name(&self)`
   - `is_solid(&self)` / `is_transparent(&self)` / `is_fluid(&self)`
   - `break_time(&self)`
   - `texture_id(&self, face: Face)` -> map to a `TextureId`
3. **Texture Atlas**: If the block has a new texture, open `src/graphics/atlas.rs`:
   - Add variant to `pub enum TextureId`
   - In `TextureAtlas::generate()`, add procedural texture drawing into the texture array slot.

### How to Add a New Mob
1. Open `src/gameplay/mob.rs`:
   - Add variant to `MobKind` (e.g. `MobKind::Skeleton`).
   - Define its hitbox dimensions and AI behavior in `Mob::update()`.
2. Open `src/graphics/software/mod.rs`:
   - In `Renderer::draw_mob()`, match on the new `MobKind` and define the 3D box hierarchy (head, body, limbs) with corresponding rotations and texture IDs.

### How to Add a New UI Component
1. Open `src/ui/hud.rs` (or create a new file in `src/ui/` and export in `src/ui/mod.rs`).
2. Implement your drawing function:
   ```rust
   pub fn draw_my_widget(buffer: &mut [u32], width: usize, height: usize, value: f32) {
       // Use crate::ui::draw helpers (fill_rect, draw_text_shadowed, etc.)
   }
   ```
3. Expose a helper on `Renderer` in `src/graphics/software/mod.rs` if needed for call-site convenience.

---

## 3. Critical Rules & Invariants
1. **Never allocate inside the render loop**:
   - Re-use buffers (like `String` scratch buffers, triangle scratch vectors).
   - Avoid `vec![]` or `.clone()` calls inside per-frame or per-pixel functions.
2. **Chunk Coordinate Math**:
   - World coordinates `(x, y, z)` map to Chunk `(cx, cz)` via integer division `(x >> 4, z >> 4)`.
   - Local coordinates inside chunk: `(x & 15, y, z & 15)`.
   - Chunk buffer index: `((x << 10) | (y << 4) | z) as usize`.
3. **Software Renderer Threading**:
   - `Rayon` operates on horizontal strips (`TILE_HEIGHT = 32`). Each thread gets exclusive mutable access to its slice of `color_buffer` and `depth_buffer`.
4. **Color Packing**:
   - Colors in buffers are packed 32-bit ARGB/xRGB: `(r << 16) | (g << 8) | b`.
   - Use `pack_color(r, g, b)` and `unpack_color(c)` from `src/ui/draw.rs`.
