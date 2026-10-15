# RustCraft Porting Guide (Platforms & Graphics APIs)

This guide provides architectural instructions for porting RustCraft to other operating systems and integrating new graphics APIs. It also documents all currently supported ports.

---

## 1. Currently Supported Platforms

### Linux (x86_64) — Primary Platform
* **Status**: Fully supported.
* **Windowing & Mouse**: `minifb` window + native X11 mouse capture (`XGrabPointer`, `XWarpPointer`, `XFixesHideCursor`) in `src/platform/mouse_capture.rs`.
* **Graphics**: Software rasterizer (Rayon multi-threaded) or WGPU/Vulkan GPU backend.
* **Gamepad**: `gilrs` via `src/platform/gamepad.rs`.
* **Build**:
  ```bash
  ./compile_linux.sh   # release + strip + UPX compression
  # or:
  cargo build --release
  ```

### Windows (x86_64) — Fully Supported
* **Status**: Fully supported.
* **Windowing & Mouse**: Handled in `src/platform/mouse_capture.rs` using `windows-sys` (`ClipCursor`, `SetCursorPos`, `ShowCursor`).
* **Graphics**: Software rasterizer works out of the box; GPU backend utilizes Vulkan or DirectX 12 via WGPU.
* **Build**:
  ```bash
  cargo build --release --target x86_64-pc-windows-msvc
  ```

### macOS (Apple Silicon / Intel)
* **Mouse Capture**:
  * Implement `CGAssociateMouseAndMouseCursorPosition(false)` and `CGWarpMouseCursorPosition` in `src/platform/mouse_capture.rs` under `#[cfg(target_os = "macos")]`.
* **Graphics**:
  * Software rasterizer blits directly to window via minifb / winit.
  * GPU pipeline uses Metal through `wgpu::Backends::METAL`.
* **Build**:
  ```bash
  cargo build --release --target aarch64-apple-darwin
  ```

### WebAssembly (Browser) — Implemented
* **Status**: Fully implemented in `src/web.rs` and `src/lib.rs`.
* **Windowing & Input**:
  * `wasm-bindgen` bindings expose `RustCraftApp` to JavaScript.
  * Zero-copy framebuffer blit to HTML5 `<canvas>` via `ImageData`.
  * Pointer Lock API: `element.requestPointerLock()` for mouse capture.
  * Gamepad API support (browser gamepad mapped in `web/app.js`).
* **Threading**: Rayon multi-threading is conditionally disabled (`#[cfg(not(target_arch = "wasm32"))]`). WASM runs single-threaded rasterizer.
* **Graphics**: Software CPU rasterizer (WGPU WebGPU backend possible via WGPU, not currently enabled).
* **Storage**: Filesystem I/O replaced with `localStorage`/`IndexedDB` hooks (or in-memory only in current port).
* **Frontend**: `web/index.html`, `web/app.js`, `web/style.css` — modern CSS UI with Pointer Lock and gamepad support.
* **Build**:
  ```bash
  ./build_web.sh
  ```

### PlayStation Vita — Implemented
* **Status**: Fully implemented. Native GXM hardware renderer on the **PowerVR SGX543MP4** GPU.
* **Target triple**: `armv7-sony-vita-newlibeabihf`
* **Platform layer** (`src/platform/vita.rs`):
  * `sceKernelLoadStartModule` bootstrap — must be called before using `SceDisplay`, `SceCtrl`, `SceTouch`.
  * Framebuffer pixel format: ABGR8888 (not XRGB — required by `SceDisplay`).
  * `sceKernelGetProcessTimeWide()` used as random seed.
  * `SceTouch` panel input mapped to mouse deltas for camera look.
  * Button mapping: ×/○/□/△, triggers (RT/LT), D-Pad, Start → game actions.
* **GXM renderer** (`src/platform/vita_gxm.rs`):
  * Full GXM init: ring buffers (VDM/vertex/fragment, 3 MB total), USSE shader memory.
  * CDRAM triple-buffering: 3× color buffers + depth buffer, async display swap callback.
  * Vertex/index GPU buffers: 2 MB each, uncached (UC) GPU memory.
  * Texture atlas: 50 textures → 128×128 px ABGR8888 single GPU texture.
  * Cg shaders (`vita/shaders/voxel.vert.cg` / `voxel.frag.cg`) compiled to GXP by `psp2cgc`.
  * Backface culling (CW winding), depth test `LESS_EQUAL`.
  * Hardware-rendered: world chunks, dropped items, mobs, 3D hand/tool, water transparency, screen overlays.
  * Chunk fade/slide-in animation: 100% GPU-uniform driven (zero CPU overhead).
  * Performance profiler: 250 ms sampling of CPU, GPU, RAM, CDRAM, chunk slots.
* **Shaders** (`vita/shaders/`):
  * `voxel.vert.cg` — vertex shader: MVP transform, UV pass-through, fog depth.
  * `voxel.frag.cg` — fragment shader: texture sample, AO multiply, fog blend.
  * Compiled at build time via `build.rs` → `psp2cgc -profile sce_vp_psp2 / sce_fp_psp2`.
  * Fallback: `vita/shaders/prebuilt/*.gxp` if VitaSDK not found.
* **Storage**: Save files stored at `ux0:data/RustCraft/rustcraft_world_NN.bin` via `SceIo`.
* **Build**:
  ```bash
  ./build_vita.sh
  # Output: RustCraft.vpk — install via VitaShell
  ```
  Prerequisites: VitaSDK at `/usr/local/vitasdk`, Rust target `armv7-sony-vita-newlibeabihf`,
### Android (ARM64 / ARMv7 / x86_64) — Implemented
* **Status**: Fully implemented. Native JNI + ANativeWindow hardware-accelerated surface rendering.
* **Target triples**: `aarch64-linux-android` (primary), `armv7-linux-androideabi`, `x86_64-linux-android`.
* **Platform layer** (`src/platform/android.rs`):
  * Hardware surface blit via `ANativeWindow_lock` / `ANativeWindow_unlockAndPost` with automatic resolution scaling to fullscreen.
  * Multi-touch virtual controls overlay:
    * Left virtual joystick / D-Pad (forward, backward, left, right movement).
    * Right camera look drag with smooth velocity scaling.
    * On-screen action buttons: `[JUMP]`, `[SNEAK]`, `[MINE]`, `[PLACE]`.
    * Top toolbar: `[PAUSE]`, `[INV]`, `[FLY]`, `[CAM]`, `[CHAT]`.
    * Bottom hotbar direct slot touch selection.
    * Direct touch clicks in all menus, crafting table, chests, furnaces, and inventory screens.
  * Physical hardware support: USB/Bluetooth gamepads and keyboards via Android KeyEvents & MotionEvents.
  * Logging: Native diagnostics piped to Android Logcat (`__android_log_write`) under tag `RustCraftNative`.
* **Storage**: App internal storage (`/data/data/com.rustcraft.game/files/saves`) configured via `set_custom_save_dir`.
* **Android Project & Packaging** (`android/`):
  * Standalone lean Android application (`com.rustcraft.game`).
  * `RustCraftActivity.java` with sticky immersive fullscreen mode, cutout/notch handling, and 60 FPS render loop thread.
  * Adaptive voxel block icon and AndroidManifest configured for gaming.
* **Build**:
  ```bash
  ./build_android.sh
  # Output: RustCraft.apk — install via adb install -r RustCraft.apk
  ```
  Prerequisites: Android SDK & NDK, OpenJDK/Java (`javac`).

### Microsoft Xbox 360 (Xenon PowerPC / ATI Xenos) — Implemented
* **Status**: Fully implemented. Native libxenon Homebrew API with direct 720p linear framebuffer blit.
* **Architecture & Targets**: `powerpc-unknown-linux-gnu` / `powerpc-xenon-elf` (PowerPC Tri-Core 3.2GHz, Big-Endian).
* **Platform layer** (`src/platform/xbox360.rs`):
  * Low-level libxenon FFI bindings (`video_init`, `video_get_vram`, `video_set_mode`, `usb_do_poll`, `controller_read`).
  * Direct 720p (1280x720) 60 FPS hardware framebuffer scanout with double-buffering.
  * Big-Endian pixel packing compatible with the ATI Xenos GPU memory architecture.
  * Hardware Xbox 360 Wireless and USB controller support:
    * Left Stick: analog walk/run movement (WASD) with deadzone filtering.
    * Right Stick: camera look (pitch & yaw) with smooth exponential curve.
    * Right Trigger (RT): mine / attack / UI click.
    * Left Trigger (LT): place block / secondary action.
    * [A]: jump / fly ascend / UI select.
    * [B]: sneak / fly descend / UI cancel.
    * [X]: drop item / quick craft.
    * [Y]: toggle inventory screen.
    * [LB] / [RB]: hotbar previous / next slot.
    * D-Pad: direct slot navigation & menu grid selection.
    * START: pause menu / settings.
    * BACK: toggle flight mode / 3rd person perspective (F5).
    * Left Thumb (L3): sprint toggle.
    * Right Thumb (R3): F3 telemetry profiler overlay.
    * Virtual cursor with smooth analog tracking in crafting/furnace/chest/inventory screens.
  * Storage: Auto-detection of USB (`uda0:/RustCraft/saves`) and internal hard drive (`sda0:/RustCraft/saves`).
  * High-precision PowerPC timebase reader via `mftb` register.
* **Packaging & XEX Generator** (`scripts/make_xex.py`):
  * Produces valid Microsoft Xbox 360 XEX2 executable (`default.xex`).
  * Uncompressed and unencrypted container with Title ID `0x58580864`.
  * Zero-signature for instant execution on exploited consoles (RGH 1.2, RGH 2.0, RGH 3.0, JTAG, BadUpdate) via hypervisor patch bypass.
  * Pre-packaged with Aurora/XeXMenu metadata (`title.meta`), icon (`icon.png`), and NXE panoramic background art (`nxeart.png`).
  * ELF output (`rustcraft.elf`) for direct XeLL Reloaded bootloader launching.
* **Build**:
  ```bash
  ./build_xbox360.sh
  # Output: target/xbox360/RustCraft/default.xex and RustCraft-Xbox360.zip
  ```
  Prerequisites: Python 3, `zip`.

---

## 2. Implementing a New Graphics Backend

All renderers implement the `RenderBackend` trait defined in [`src/graphics/traits.rs`](file:///home/shieldziak/Pobrane/rustcraft/src/graphics/traits.rs):

```rust
pub trait RenderBackend {
    /// Name of the backend ("software", "vulkan", "metal", "dx12", "webgpu", etc.)
    fn backend_name(&self) -> &'static str;

    /// 32-bit pixel buffer for display (xRGB/ARGB)
    fn pixel_buffer(&self) -> &[u32];

    /// Z-buffer
    fn depth_buffer(&self) -> &[f32];

    /// Frame dimensions (width, height)
    fn dimensions(&self) -> (usize, usize);

    /// Clear buffers (e.g. sky gradient)
    fn clear(&mut self, camera_pitch: f32);

    /// Render 3D world frame
    fn render_frame(&mut self, world: &mut World, camera: &Camera);
}
```

### Steps to Implement a New GPU Backend (e.g. Raw OpenGL, Metal, or DirectX 12)
1. **Create Submodule**:
   Create `src/graphics/backend_name/mod.rs` (e.g. `src/graphics/metal/mod.rs`).
2. **Chunk Mesh Upload**:
   Query `world.chunks` or meshed triangles from `crate::world::mesher::build_chunk_mesh`.
   Each chunk mesh contains arrays of `Vertex { pos, uv, ao, tex }`.
3. **Texture Atlas Bind**:
   Upload the texture data generated by `crate::graphics::TextureAtlas::generate()` as a 2D Texture Array (`TEX_SIZE = 16×16`, `TEXTURE_LAYER_COUNT = 50`).
4. **Shader Pipeline**:
   - Transform `v.pos` with `view_proj * pos`.
   - Shade texture texel multiplied by vertex AO (`ao` in range 0.0..1.0).
   - Apply fog: `lerp(color, fog_color, clamp(depth / 180.0, 0.0, 1.0))`.
5. **Implement `RenderBackend`**:
   Implement `RenderBackend for YourRenderer` and add it to `src/cli.rs` and `main.rs` / `app.rs`.

> **Vita note**: The GXM backend does **not** use the `RenderBackend` trait — it is integrated
> directly in `src/platform/vita_gxm.rs` and called from the Vita-specific branch in `src/app.rs`,
> because GXM operates on GPU-native buffers without a CPU-side pixel buffer.

---

## 3. Adding a New Platform

1. **Create** `src/platform/<platform_name>.rs`.
2. **Add** `#[cfg(target_os = "<platform>")]` module entry in `src/platform/mod.rs`.
3. **Implement** the required platform surface:
   - Window creation and event loop (or use `minifb` / `winit` if available).
   - Mouse/pointer capture (implement the three functions from `mouse_capture.rs` contract).
   - Input translation to `InputState` (keyboard, mouse deltas, gamepad if applicable).
4. **Wire up** in `src/app.rs` or `src/main.rs` with a `#[cfg]` branch.
5. **Save path**: return a platform-appropriate path from `src/world/storage.rs::save_path()`.

---

## 4. Storage & Save Compatibility

Chunk data serialization in `src/world/storage.rs` uses a portable little-endian binary format:
* Magic header: `0x52, 0x43, 0x57, 0x31` (`RCW1` — RustCraft World 1)
* Seed (`u32` little-endian)
* Player position (`f32` x, y, z)
* Chunk count & chunk blocks array (`16 × 64 × 16` bytes per chunk)

This format is platform-agnostic and endian-safe across Linux, Windows, macOS, WebAssembly, and PlayStation Vita.

Save file paths per platform:
| Platform | Path |
|---|---|
| Linux / Windows / macOS | `./rustcraft_world_NN.bin` (cwd) |
| WebAssembly | In-memory / IndexedDB (platform-specific hook) |
| PlayStation Vita | `ux0:data/RustCraft/rustcraft_world_NN.bin` |
| Android | `/data/data/com.rustcraft.game/files/saves/` |

---

## 5. Shader Compilation Pipeline (Vita)

```
vita/shaders/voxel.vert.cg  ──psp2cgc──► vita/shaders/voxel.vert.gxp
vita/shaders/voxel.frag.cg  ──psp2cgc──► vita/shaders/voxel.frag.gxp
                                                    │
                           build.rs embeds GXP bytes ──► vita_gxm.rs
```

`build.rs` auto-detects `psp2cgc` in `$VITASDK/bin`. If not found, falls back to
`vita/shaders/prebuilt/*.gxp`. The compiled GXP binaries are embedded at link time
and uploaded to GPU USSE memory on startup via `sceGxmShaderPatcherRegisterProgram`.

To install `psp2cgc` on Linux:
```bash
vdpm install psp2cgc-linux
```
