//! Renderer initialization, resize, texture refresh, sky clear, and `RenderBackend` trait implementation.
//! Uses `platform::{capabilities, gpu_bridge, threading}` for zero platform `#[cfg]`.

use std::collections::{HashMap, HashSet};

use crate::features::meshing::AsyncMesher;
use crate::features::player::Camera;
use crate::features::rendering::atlas::TextureAtlas;
use crate::features::rendering::traits::RenderBackend;
use crate::features::world::World;
use crate::platform::gpu_bridge::GpuBridge;
use crate::platform::{capabilities, threading};

use super::clip::pack_color;
use super::types::{Renderer, TILE_HEIGHT};

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        threading::init_global_thread_pool();

        Renderer {
            width,
            height,
            color_buffer: vec![0; width * height],
            depth_buffer: vec![f32::INFINITY; width * height],
            mesh_cache: HashMap::new(),
            chunk_spawn_times: HashMap::new(),
            total_time: 0.0,
            chunk_anim: true,
            selected_block: 1,
            textures: {
                crate::log_info!("RENDERER", "Generating procedural texture atlas (70+ textures)...");
                let tex = TextureAtlas::generate();
                crate::log_info!("RENDERER", "Texture atlas generated successfully.");
                tex
            },
            gpu: GpuBridge::none(),
            loaded_keys: Vec::with_capacity(128),
            dirty_keys: HashSet::with_capacity(64),
            visible_keys: Vec::with_capacity(128),
            opaque_tris_buf: Vec::with_capacity(16384),
            transparent_tris_buf: Vec::with_capacity(4096),
            particles: Vec::new(),
            fog_enabled: true,
            render_distance: 6,
            brightness: 0.50,
            current_daylight: 1.0,
            current_fog_color: [0.75, 0.85, 1.0],
            last_draw_calls: 0,
            last_triangles_rendered: 0,
            last_triangles_culled: 0,
            last_subchunks_rendered: 0,
            last_subchunks_culled: 0,
            async_mesher: AsyncMesher::new(),
            pending_mesh_keys: HashSet::new(),
        }
    }

    #[inline(always)]
    pub fn mesh_cache_triangles_count(&self) -> usize {
        self.mesh_cache.values().map(|v| v.len()).sum()
    }

    #[inline(always)]
    pub fn last_render_telemetry(&self) -> (u32, u32, u32, u32, u32) {
        (
            self.last_draw_calls,
            self.last_triangles_rendered,
            self.last_triangles_culled,
            self.last_subchunks_rendered,
            self.last_subchunks_culled,
        )
    }

    pub fn new_vulkan(width: usize, height: usize) -> Result<Self, String> {
        let mut renderer = Self::new(width, height);
        renderer.gpu = GpuBridge::new_vulkan(width, height, &renderer.textures)?;
        Ok(renderer)
    }

    pub fn new_opengl(width: usize, height: usize) -> Result<Self, String> {
        let mut renderer = Self::new(width, height);
        renderer.gpu = GpuBridge::new_opengl(width, height, &renderer.textures)?;
        Ok(renderer)
    }

    pub fn backend_name(&self) -> &'static str {
        if let Some(name) = self.gpu.backend_name() {
            return name;
        }
        let caps = capabilities();
        if caps.is_vita {
            "vita_gxm"
        } else if caps.is_android {
            "android_software"
        } else if caps.is_xbox360 {
            "xenon_framebuffer"
        } else {
            "software"
        }
    }

    pub fn renderer_info(&self) -> (String, String, String) {
        if let Some((name, driver, ver)) = self.gpu.gpu_info() {
            return (ver.to_string(), name.to_string(), driver.to_string());
        }
        let caps = capabilities();
        if caps.is_vita {
            (
                "Hardware GXM (Shader Patcher)".to_string(),
                "Sony PlayStation Vita (SGX543MP4+)".to_string(),
                "SCE GXM / CDRAM Triple Buffer".to_string(),
            )
        } else if caps.is_android {
            (
                "Software Rasterizer (Android NDK)".to_string(),
                "ANativeWindow / ARM NEON SIMD".to_string(),
                format!("Rayon ({} threads)", threading::active_raster_threads()),
            )
        } else if caps.is_xbox360 {
            (
                "Direct Linear VRAM Framebuffer (720p HD)".to_string(),
                "Microsoft Xbox 360 (Xenon 3.2GHz PowerPC / ATI Xenos)".to_string(),
                "libxenon Homebrew API / double-buffered scanout".to_string(),
            )
        } else if caps.is_wasm {
            (
                "Software Rasterizer".to_string(),
                "WebAssembly / HTML5 Canvas".to_string(),
                "Canvas 2D Context".to_string(),
            )
        } else {
            (
                "Software Rasterizer".to_string(),
                "CPU Multi-threaded SIMD".to_string(),
                format!("Rayon ({} threads)", threading::active_raster_threads()),
            )
        }
    }

    pub fn pixel_buffer(&self) -> &[u32] {
        &self.color_buffer
    }

    pub fn pixel_buffer_mut(&mut self) -> &mut [u32] {
        &mut self.color_buffer
    }

    /// Resizes internal color and depth buffers to new resolution dimensions.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.color_buffer.resize(width * height, 0);
        self.depth_buffer.resize(width * height, f32::INFINITY);
        self.gpu.resize(width, height);
    }

    pub fn update_textures(&mut self, fancy_leaves: bool) {
        self.textures = TextureAtlas::generate_with_fancy(fancy_leaves);
        self.gpu.update_textures(&self.textures);
    }

    pub fn clear_mesh_cache(&mut self) {
        let count = self.mesh_cache.len();
        self.mesh_cache.clear();
        self.dirty_keys.clear();
        self.pending_mesh_keys.clear();
        self.async_mesher.invalidate_pending();
        self.chunk_spawn_times.clear();
        self.gpu.clear_chunk_buffers();
        crate::log_info!(
            "RENDER",
            "Cleared chunk mesh cache (freed {} meshes) and reset dirty queues",
            count
        );
    }

    pub fn selected_block(&self) -> u8 {
        self.selected_block
    }

    pub fn set_selected_block(&mut self, id: u8) {
        self.selected_block = id;
    }

    pub fn clear(&mut self, camera_pitch: f32) {
        self.clear_sky(camera_pitch, [0.35, 0.55, 0.95], [0.75, 0.85, 1.0]);
    }

    pub fn clear_sky(&mut self, camera_pitch: f32, top: [f32; 3], bottom: [f32; 3]) {
        let width = self.width;
        let height = self.height;
        let max_h = height.max(1) as f32;

        threading::parallel_for_chunks_mut(
            &mut self.color_buffer,
            width * TILE_HEIGHT,
            |tile_idx, rows| {
                let y_start = tile_idx * TILE_HEIGHT;
                for (row_i, row) in rows.chunks_mut(width).enumerate() {
                    let y = y_start + row_i;
                    let t = y as f32 / max_h;
                    let t = (t + camera_pitch * 0.05).clamp(0.0, 1.0);
                    let r = top[0] + (bottom[0] - top[0]) * t;
                    let g = top[1] + (bottom[1] - top[1]) * t;
                    let b = top[2] + (bottom[2] - top[2]) * t;
                    let color = pack_color(r, g, b);
                    row.fill(color);
                }
            },
        );

        threading::parallel_for_chunks_mut(
            &mut self.depth_buffer,
            width * TILE_HEIGHT,
            |_tile_idx, chunk| {
                chunk.fill(f32::INFINITY);
            },
        );
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(f32::INFINITY);
    }
}

impl RenderBackend for Renderer {
    fn backend_name(&self) -> &'static str {
        self.backend_name()
    }

    fn pixel_buffer(&self) -> &[u32] {
        self.pixel_buffer()
    }

    fn depth_buffer(&self) -> &[f32] {
        &self.depth_buffer
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn clear(&mut self, camera_pitch: f32) {
        self.clear(camera_pitch);
    }

    fn render_frame(&mut self, world: &mut World, camera: &Camera) {
        self.render_frame(world, camera);
    }
}
