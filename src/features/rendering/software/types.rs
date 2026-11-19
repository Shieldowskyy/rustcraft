//! Software & hybrid GPU renderer types (`Renderer`, `Particle`, `ClipVertAO`, `ScreenVertAO`, `ScreenTriangle`).
//! Strictly isolated from platform-specific `#[cfg]` code; delegates hardware GPU calls to `GpuBridge`.

use std::collections::{HashMap, HashSet};

use crate::core::math::Vec3;
use crate::features::meshing::{AsyncMesher, Triangle};
use crate::features::rendering::atlas::{TextureAtlas, TextureId};
use crate::platform::gpu_bridge::GpuBridge;

pub(super) const TILE_HEIGHT: usize = 32;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub color: [f32; 3],
    pub size: f32,
}

#[derive(Clone, Copy)]
pub(super) struct ClipVertAO {
    pub(super) clip: [f32; 4],
    pub(super) uv: (f32, f32),
    pub(super) ao: f32,
    pub(super) sky_light: f32,
    pub(super) block_light: f32,
}

#[derive(Clone, Copy)]
pub(super) struct ScreenVertAO {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) inv_w: f32,
    pub(super) uv_over_w: (f32, f32),
    pub(super) ao_over_w: f32,
    pub(super) sky_light_over_w: f32,
    pub(super) block_light_over_w: f32,
}

#[derive(Clone, Copy)]
pub(super) struct ScreenTriangle {
    pub(super) sa: ScreenVertAO,
    pub(super) sb: ScreenVertAO,
    pub(super) sc: ScreenVertAO,
    pub(super) texture: TextureId,
    pub(super) min_y: i32,
    pub(super) max_y: i32,
}

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<u32>,
    pub depth_buffer: Vec<f32>,
    pub(super) mesh_cache: HashMap<(i32, i32), Vec<Triangle>>,
    pub chunk_spawn_times: HashMap<(i32, i32), f32>,
    pub total_time: f32,
    pub chunk_anim: bool,
    pub(super) selected_block: u8,
    pub textures: TextureAtlas,
    pub(super) gpu: GpuBridge,
    pub(super) loaded_keys: Vec<(i32, i32)>,
    pub(super) dirty_keys: HashSet<(i32, i32)>,
    pub(super) visible_keys: Vec<(i32, i32)>,
    pub(super) opaque_tris_buf: Vec<ScreenTriangle>,
    pub(super) transparent_tris_buf: Vec<ScreenTriangle>,
    pub particles: Vec<Particle>,
    pub fog_enabled: bool,
    pub render_distance: i32,
    pub brightness: f32,
    pub current_daylight: f32,
    pub current_fog_color: [f32; 3],
    pub last_draw_calls: u32,
    pub last_triangles_rendered: u32,
    pub last_triangles_culled: u32,
    pub last_subchunks_rendered: u32,
    pub last_subchunks_culled: u32,
    pub async_mesher: AsyncMesher,
    pub pending_mesh_keys: HashSet<(i32, i32)>,
}
