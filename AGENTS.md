# AGENTS.md — RustCraft Architecture & AI Rules

## 1. Mandatory Rules for AI Agents
1. **Strict Platform Isolation (`#[cfg]`):** NEVER put platform `#[cfg(...)]` or `cfg!(...)` in `src/features/` or `src/core/`. All OS/console/WASM/GPU-availability branching belongs exclusively in `src/platform/`.
2. **90/10 Self-Documenting Standard:**
   - **90% Code & Types:** Use strong types (`WorldPos`, `ChunkPos`, `LocalPos`, `BlockFace`), enums, and small single-purpose files (**150–350 LOC**, hard split >400–450 LOC) with narrow `pub use` facades in `mod.rs` (`pub(super)`/private internals).
   - **10% Micro-docs:**
     - `//!` (2–3 lines) at the top of key files: responsibility, state mutated, inputs/outputs.
     - `// DLACZEGO:` only for non-obvious invariants (CCW winding, AO diagonal flip, Vita RAM/CDRAM/affinity, WGPU layer limits, WASM/WebGL constraints).
     - Update the `src/` map below whenever files move; do NOT create extra `.md` docs or "what" comments.
3. **Git Checkpoint Discipline:** Never keep more than **1 module** uncommitted. Run `cargo check` on supported targets before every `git commit`.
4. **Never Run `cargo run` in Headless Mode:** Launches an interactive `minifb` window and blocks. Use `cargo check` / `cargo test` / `cargo build`.

---

## 2. Coordinate System & Voxel Invariants
- **Coordinate System:** Right-handed OpenGL-style, column-major `Mat4` ([src/core/math.rs](src/core/math.rs)).
  - `+Y`: Up | `-Z`: North (`BlockFace::North`, initial camera `yaw = -π/2`) | `+Z`: South | `+X`: East | `-X`: West.
- **Chunk Layout:** `16 × 64 × 16` (`CHUNK_WIDTH = 16`, `CHUNK_HEIGHT = 64`, `CHUNK_DEPTH = 16`), 4 subchunks (`16` height each).
  - Flat index: `((x << 10) | (y << 4) | z)`. Negative world coords map via `wx >> 4` and `wx & 15` ([src/core/coords.rs](src/core/coords.rs)).
- **Face Winding & AO:** **CCW** when viewed from outside the cube (`(v1 - v0) × (v2 - v0)` points along outward normal), with dynamic quad diagonal flip (`ao_a + ao_c < ao_b + ao_d`) to prevent AO interpolation artifacts.
- **WGPU Texture Array Limit:** Built-in IDs `0..=221` (222 layers), mod IDs `222..=255` (34 custom layers, `TEXTURE_LAYER_COUNT = 256`). Keep `BUILTIN_TEXTURE_COUNT`, `CUSTOM_TEXTURE_LAYERS`, `layer_from_texture_id`, `texture_id_from_layer`, and `required_limits.max_texture_array_layers = adapter_limits.max_texture_array_layers` in sync. Verify with `cargo test graphics::gpu::tests::test_vulkan_context_texture_array_creation`.
- **Zero-Allocation Hot Paths:** Never allocate per-frame or per-pixel inside `software` rasterizer loops.

---

## 3. Tech Stack & Target Differences (`Cargo.toml`)
- **Shared Core:** `bytemuck`, `flate2` (rust_backend), `lewton`, `opus-pure`, `chacha20poly1305`, `hkdf`, `sha2`, `getrandom`.
- **Priority Targets (100% support & verified on every commit):**
  - **Linux (`x86_64-unknown-linux-gnu`) + `rustcraft-server`:** `minifb`, `wgpu 0.19` (Vulkan/GL), `pollster`, `rayon`, `gilrs`, `sentry`, `x11-dl`.
  - **Windows (`x86_64-pc-windows-gnu`):** same + `windows-sys` (Win32 mouse capture, WaveOut/JobObject); `wgpu` (DX12/Vulkan).
  - **Sony PS Vita (`armv7-sony-vita-newlibeabihf`):** nightly + `-Z build-std=std,panic_abort`, zero external GPU/UI crates; raw VitaSDK FFI (`libgxm` Cg→GXP via `build.rs`, `SceAudioOut`, `SceCtrl`, `ux0:`/`app0:` storage, 180 MB `_newlib_heap_size_user`, core affinity masks `0x2`/`0x4`).
- **Secondary Targets (kept clean in `src/platform/`):**
  - **Android (`aarch64-linux-android`):** `cdylib`, `jni-sys`, `rayon`, `wgpu`, `ANativeWindow` + multi-touch virtual controls.
  - **Web / WASM (`wasm32-unknown-unknown`):** `cdylib`, `wasm-bindgen`, `web-sys`, `js-sys`, HTML5 2D Canvas software backend, single-threaded.
  - **macOS (`aarch64-apple-darwin` / `x86_64-apple-darwin`):** `minifb` + `wgpu` (Metal), `rayon`, `gilrs`, `sentry`. *(Note: compiling `ring` C sources from Linux fails without a Darwin SDK; run check/build on macOS or CI `.github/workflows/build-macos.yml`.)*
  - **Xbox 360 (`powerpc-unknown-linux-gnu --features xbox360`):** Legacy code preserved in `src/platform/xbox360.rs` (unmaintained).

---

## 4. Verification Commands (`cargo check` / `cargo test`)
```bash
# Priority #1: Linux + rustcraft-server + tests
cargo check --all-targets
cargo test --lib

# Priority #1: Windows (via mingw-w64)
cargo check --target x86_64-pc-windows-gnu --all-targets

# Priority #1: PS Vita (requires VITASDK=/usr/local/vitasdk & nightly rust-src)
VITASDK=/usr/local/vitasdk RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C target-cpu=cortex-a9" \
  cargo check --target armv7-sony-vita-newlibeabihf -Z build-std=std,panic_abort --bin rustcraft

# Secondary: Android & Web/WASM
cargo check --target aarch64-linux-android --lib
cargo check --target wasm32-unknown-unknown --lib

# Secondary: macOS (run on Darwin host / CI)
cargo check --target aarch64-apple-darwin --bins --lib

# GPU texture array invariant test
cargo test graphics::gpu::tests::test_vulkan_context_texture_array_creation
```
*(Crash dumps live in `crash-reports/crash-*.txt`; read the newest before guessing.)*

---

## 5. Living `src/` Directory Map *(update after every structural commit)*
- `src/lib.rs`, `src/main.rs`, `src/cli.rs`, `src/bin/rustcraft-server.rs` — crate root, entrypoints, CLI, dedicated server/relay.
- `src/core/` — zero-platform-`#[cfg]` foundational types (`coords`, `events`, `math`, `noise`, `i18n`, `logger`, `build_info`, `controls`, `settings`, `crash_reporter`, `profiler`, `profiler_server`).
- `src/platform/` — Platform Abstraction Layer (PAL) & all OS/console/GPU/audio `#[cfg]` backends: `traits` (`capabilities()`), `threading`, `storage`, `clipboard`, `system`, `mouse_capture`, `gamepad`, `desktop`, `macos`, `web`, `android/`, `vita/` (`gxm/` [`ffi_defs`, `programs_fx`, `chunk_pool`, `weather`, `postfx`, ...], `lifecycle`, `display`, `input`, `ime`, `panic_log`), `wgpu/`, `gpu_bridge`, `audio/`, `xbox360.rs` (legacy).
- `src/features/` — zero-platform-`#[cfg]` modular game domains:
  - `blocks/` — `BlockId`, physical/optical/tool/drop properties, UV/texture mappings.
  - `meshing/` — pure CPU chunk/subchunk mesh builder (face culling, AO diagonal flip, water/foliage).
  - `physics/` — AABB player/mob collision, ladder/fluid physics, DDA voxel raycasting.
  - `world/` — chunks, biomes, terrain/caves/structures/villages/trees, lighting, fluids, farming, ruststone, TNT, weather, storage.
  - `entities/` — mobs (AI/spawning/combat/breeding/villager trading), dropped items, arrows, particles.
  - `player/` — `Player`, inventory/hotbar/armor, crafting recipes, quests, achievements.
  - `rendering/` — procedural `atlas`, `camera`, `clouds`, `font`, `lighting`, `player_parts`, `software` rasterizer, `traits`, `wgpu_facade`.
  - `ui/` — `draw`, `widgets`, `hud`, `inventory/`, `minimap/`, `worldmap`, `chat/`, menus (`main_menu`, `pause`, `settings`, `saves`, `multiplayer`, `questbook`, `death`, `loading`, `overlay`).
  - `audio/` — procedural `synth/`, `mixer/`, `engine/`, `spatial`, `sound/` (WAV/OGG/Opus), `soundtrack/`.
  - `net/` — multiplayer `protocol/`, `client/`, `server/`, `relay/`, `discovery`, `server_log`.
  - `modding/` — custom blocks/items/recipes/commands, JSON parser, procedural mod texture synthesis.
  - `autotest/` — automated gameplay smoke-test runner & HUD overlay.
- `src/app/` — top-level game loop & state machine (`init`, `frame`, `progress`, `types`, `helpers`, `profiler`, `gameplay/`, `containers/`, `multiplayer/`, `menus/`).
- `src/{world,gameplay,graphics,ui,audio,net,engine}/` — thin `pub use` compatibility facades delegating to `src/features/` & `src/platform/`.
