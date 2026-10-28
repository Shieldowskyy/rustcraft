# LLM & Developer Maintenance Guide for RustCraft

This guide is designed for LLMs (and developers) to understand the codebase structure quickly, locate relevant modules, adhere to conventions, and safely add new features.

---

## 1. Fast Directory Reference

| To modify / add: | Primary file/module: | Notes: |
|---|---|---|
| **New Block Type** | `src/world/block.rs`, `src/graphics/atlas.rs` | Add variant to `BlockId` (standard IDs 0..=162, custom modded IDs 163..=255), assign `TextureId`, add texture generator in `atlas.rs`. |
| **New Mob Type** | `src/gameplay/mob.rs` | Add variant to `MobKind` (14 types: Pig, Cow, Chicken, Sheep, Zombie, Skeleton, Creeper, Dolphin, Cod, Salmon, Pufferfish, TropicalFish, Villager, IronGolem), define bounding boxes & AI tick, add draw calls in `software/mod.rs` and `platform/vita_gxm.rs`. |
| **Dynamic Weather** | `src/world/weather.rs` | Manage weather states (Clear, Rain, Thunderstorm), precipitation by biome temperature (rain/snow), storm audio, lightning flash, and `/weather` commands. |
| **Villages & Trading** | `src/world/mod.rs`, `src/gameplay/trading.rs` | Village layout generator, professions, trade offerings, emerald currency, and trading window in `src/ui/container_ui.rs`. |
| **New UI Screen / Element** | `src/ui/` (`hud.rs`, `menu.rs`, `inventory_ui.rs`, `container_ui.rs`, `minimap.rs`) | Use functions in `src/ui/draw.rs` (`fill_rect_alpha`, `draw_text_shadowed`). |
| **Player / Physics / Movement** | `src/gameplay/physics.rs`, `player.rs` | Physics uses AABB sweeps. Maintain step-up, crouching checks, and fluid buoyancy. |
| **World Generation / Biomes** | `src/world/mod.rs` (in `Chunk::generate`) | 20 biomes via noise axes in `get_biome_at()`. Uses `src/core/noise.rs`. Villages, ruins, wells, underwater vegetation (kelp/seagrass). |
| **Procedural Sound / Audio** | `src/audio/synth.rs`, `src/audio/sound.rs` | Synthesize waveform in `synth.rs`, trigger via `AudioEngine::play_sound` or `play_spatial_sound`. |
| **Soundtrack / Music** | `src/audio/soundtrack.rs` | Streaming background music player (OGG/Opus) with category shuffle and UI toast notifications. |
| **Audio Platform Backend** | `src/audio/backend/` | Implement `AudioBackend` trait with ring buffer / hardware output stream. |
| **Crash Reporter & Sentry** | `src/core/crash_reporter.rs`, `src/cli.rs` | Global panic hook, dumps system state/breadcrumbs, optional Sentry upload. |
| **Multiplayer / Networking** | `src/net/` (`protocol.rs`, `server.rs`, `client.rs`) | UDP packet serialization, entity sync, mod exchange, LAN broadcast discovery. |
| **New Graphics Backend** | `src/graphics/traits.rs`, `src/graphics/gpu/` | Implement `RenderBackend` trait. |
| **Platform / OS Input Hook** | `src/platform/` (`android.rs`, `xbox360.rs`, `mouse_capture.rs`) | Keep OS-specific code guarded by `#[cfg(target_os = ...)]`. |

---

## 2. Common Workflows

### How to Add a New Block
1. **Define Block**: Open `src/world/block.rs` and add a new variant to `pub enum BlockId`. Note that IDs `0..=162` are reserved for standard vanilla items and blocks; `163..=255` are mapped to `BlockId::Custom(u8)`.
2. **Configure Properties**: In `src/world/block.rs`, update:
   - `from_u8(v: u8)` and `to_u8(self)`
   - `name(&self)`
   - `is_solid(&self)` / `is_transparent(&self)` / `is_fluid(&self)`
   - `break_time(&self)`
   - `texture_id(&self, face: Face)` -> map to a `TextureId`
3. **Texture Atlas**: If the block has a new texture, open `src/graphics/atlas.rs`:
   - Add variant to `pub enum TextureId`
   - In `TextureAtlas::generate()`, add procedural texture drawing into the texture array slot.
4. **Crafting & Smelting**: Add recipes in `src/gameplay/recipe.rs` and smelting in `src/world/tile_entity.rs`.
5. **Creative Inventory**: If applicable, ensure the block is listed in `src/ui/inventory_ui.rs` in the appropriate category tab.

### How to Add a New Procedural Sound Effect
1. Open `src/audio/sound.rs` and add a variant to `pub enum SoundId`.
2. Open `src/audio/synth.rs` and implement waveform synthesis (e.g. noise bandpass, frequency envelopes, white noise bursts).
3. Trigger from gameplay/app code via `audio.play_spatial_sound(SoundId::..., pos)` for 3D attenuated audio, or `audio.play_sound(SoundId::...)` for UI/global sounds.

### How to Add a New Biome
1. Open `src/world/mod.rs` and add variant to `pub enum Biome`.
2. Map noise axes in `get_biome_at(wx, wz, seed)` (continentalness, temperature, humidity).
3. In `Chunk::generate()`, define surface/subsurface blocks, height variations, and flora/structures (`decorations.push(...)`).
4. Add display name in `World::biome_name_at()`.

### How to Add a New Mob
1. Open `src/gameplay/mob.rs`:
   - Add variant to `MobKind` (e.g. `MobKind::Villager`, `MobKind::IronGolem`).
   - Define its hitbox dimensions and AI behavior in `Mob::update()`.
2. Open `src/graphics/software/mod.rs` and `src/platform/vita_gxm.rs`:
   - In `Renderer::draw_mob()`, match on the new `MobKind` and define the 3D box hierarchy (head, body, limbs) with corresponding rotations and texture IDs.

### How to Work with Crash Reporter & Sentry
1. **Breadcrumbs**: Use `crate::core::logger::record_breadcrumb(category, message)` before risky operations.
2. **Testing**: Run `cargo run --release -- --crash-test` to test panic trapping and crash dump output.
3. **Sentry Target**: Pass `--sentry-dsn <DSN>` or set `SENTRY_DSN` environment variable. Disable with `--no-sentry`.

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
