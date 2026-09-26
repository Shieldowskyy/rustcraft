use std::collections::{HashMap, HashSet};
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
use rayon::prelude::*;

use crate::core::math::{Mat4, Vec3};
use crate::graphics::atlas::{TextureAtlas, TextureId};
use crate::graphics::camera::Camera;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
use crate::graphics::gpu::VulkanContext;
use crate::graphics::traits::RenderBackend;
use crate::world::mesher::{Triangle, Vertex};
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Z};

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
struct ClipVertAO {
    clip: [f32; 4],
    uv: (f32, f32),
    ao: f32,
    sky_light: f32,
    block_light: f32,
}

#[derive(Clone, Copy)]
struct ScreenVertAO {
    x: f32,
    y: f32,
    inv_w: f32,
    uv_over_w: (f32, f32),
    ao_over_w: f32,
    sky_light_over_w: f32,
    block_light_over_w: f32,
}

#[derive(Clone, Copy)]
struct ScreenTriangle {
    sa: ScreenVertAO,
    sb: ScreenVertAO,
    sc: ScreenVertAO,
    texture: TextureId,
    min_y: i32,
    max_y: i32,
}

const TILE_HEIGHT: usize = 32;

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<u32>,
    pub depth_buffer: Vec<f32>,
    mesh_cache: HashMap<(i32, i32), Vec<Triangle>>,
    pub chunk_spawn_times: HashMap<(i32, i32), f32>,
    pub total_time: f32,
    pub chunk_anim: bool,
    selected_block: u8,
    pub textures: TextureAtlas,
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
    vulkan: Option<VulkanContext>,
    #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
    vulkan: Option<()>,
    loaded_keys: Vec<(i32, i32)>,
    dirty_keys: HashSet<(i32, i32)>,
    #[allow(dead_code)]
    visible_keys: Vec<(i32, i32)>,
    opaque_tris_buf: Vec<ScreenTriangle>,
    transparent_tris_buf: Vec<ScreenTriangle>,
    pub particles: Vec<Particle>,
    pub fog_enabled: bool,
    pub render_distance: i32,
    pub current_daylight: f32,
    pub current_fog_color: [f32; 3],
    pub last_draw_calls: u32,
    pub last_triangles_rendered: u32,
    pub last_triangles_culled: u32,
    pub last_subchunks_rendered: u32,
    pub last_subchunks_culled: u32,
    pub async_mesher: crate::world::AsyncMesher,
    pub pending_mesh_keys: HashSet<(i32, i32)>,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        {
            let logical_threads = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            let worker_threads = if logical_threads <= 4 {
                (logical_threads / 2).max(1)
            } else {
                logical_threads
            };
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(worker_threads)
                .build_global();
        }

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
            textures: TextureAtlas::generate(),
            vulkan: None,
            loaded_keys: Vec::with_capacity(128),
            dirty_keys: HashSet::with_capacity(64),
            visible_keys: Vec::with_capacity(128),
            opaque_tris_buf: Vec::with_capacity(16384),
            transparent_tris_buf: Vec::with_capacity(4096),
            particles: Vec::new(),
            fog_enabled: true,
            render_distance: 6,
            current_daylight: 1.0,
            current_fog_color: [0.75, 0.85, 1.0],
            last_draw_calls: 0,
            last_triangles_rendered: 0,
            last_triangles_culled: 0,
            last_subchunks_rendered: 0,
            last_subchunks_culled: 0,
            async_mesher: crate::world::AsyncMesher::new(),
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

    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
    pub fn new_vulkan(width: usize, height: usize) -> Result<Self, String> {
        let mut renderer = Self::new(width, height);
        renderer.vulkan = Some(VulkanContext::new(width, height, &renderer.textures)?);
        Ok(renderer)
    }

    pub fn backend_name(&self) -> &'static str {
        #[cfg(target_os = "vita")]
        return "vita_gxm";
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if self.vulkan.is_some() { return "vulkan"; }
        #[cfg(not(target_os = "vita"))]
        "software"
    }

    pub fn renderer_info(&self) -> (String, String, String) {
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if let Some(vk) = &self.vulkan {
            let (name, driver, ver) = vk.gpu_info();
            return (ver.to_string(), name.to_string(), driver.to_string());
        }

        #[cfg(target_os = "vita")]
        return (
            "Hardware GXM (Shader Patcher)".to_string(),
            "Sony PlayStation Vita (SGX543MP4+)".to_string(),
            "SCE GXM / CDRAM Triple Buffer".to_string(),
        );

        #[cfg(target_arch = "wasm32")]
        return (
            "Software Rasterizer".to_string(),
            "WebAssembly / HTML5 Canvas".to_string(),
            "Canvas 2D Context".to_string(),
        );

        #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
        (
            "Software Rasterizer".to_string(),
            "CPU Multi-threaded SIMD".to_string(),
            format!("Rayon ({} threads)", rayon::current_num_threads()),
        )
    }

    pub fn pixel_buffer(&self) -> &[u32] {
        &self.color_buffer
    }

    /// Resizes internal color and depth buffers to new resolution dimensions.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.color_buffer.resize(width * height, 0);
        self.depth_buffer.resize(width * height, f32::INFINITY);
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if let Some(vk) = self.vulkan.as_mut() {
            vk.resize(width, height);
        }
    }

    pub fn update_textures(&mut self, fancy_leaves: bool) {
        self.textures = TextureAtlas::generate_with_fancy(fancy_leaves);
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if let Some(vk) = self.vulkan.as_mut() {
            vk.update_textures(&self.textures);
        }
    }

    pub fn clear_mesh_cache(&mut self) {
        self.mesh_cache.clear();
        self.dirty_keys.clear();
        self.chunk_spawn_times.clear();
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if let Some(vk) = self.vulkan.as_mut() {
            vk.clear_chunk_buffers();
        }
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

        let clear_tile = |(tile_idx, rows): (usize, &mut [u32])| {
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
        };

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        self.color_buffer
            .par_chunks_mut(width * TILE_HEIGHT)
            .enumerate()
            .for_each(clear_tile);

        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        self.color_buffer
            .chunks_mut(width * TILE_HEIGHT)
            .enumerate()
            .for_each(clear_tile);

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        self.depth_buffer.par_chunks_mut(width * TILE_HEIGHT).for_each(|chunk| {
            chunk.fill(f32::INFINITY);
        });

        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        self.depth_buffer.chunks_mut(width * TILE_HEIGHT).for_each(|chunk| {
            chunk.fill(f32::INFINITY);
        });
    }

    #[inline(always)]
    pub fn main_menu_button_rects(&self) -> [(i32, i32, i32, i32); 3] {
        crate::ui::main_menu_button_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn save_card_rects(&self) -> [(i32, i32, i32, i32); crate::ui::VISIBLE_SAVE_ROWS] {
        crate::ui::save_card_rects(self.width)
    }

    #[inline(always)]
    pub fn save_bottom_button_rects(&self) -> [(i32, i32, i32, i32); 4] {
        crate::ui::save_bottom_button_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_tab_rects(&self) -> [(i32, i32, i32, i32); 3] {
        crate::ui::settings_tab_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_graphics_rects(&self) -> [(i32, i32, i32, i32); 13] {
        crate::ui::settings_graphics_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_controls_rects(&self) -> [(i32, i32, i32, i32); 6] {
        crate::ui::settings_controls_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_debug_rects(&self) -> [(i32, i32, i32, i32); 5] {
        crate::ui::settings_debug_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_option_rects(&self) -> [(i32, i32, i32, i32); 13] {
        crate::ui::settings_graphics_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn create_world_rects(&self) -> [(i32, i32, i32, i32); 6] {
        crate::ui::create_world_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn pause_menu_button_rects(&self) -> [(i32, i32, i32, i32); 5] {
        crate::ui::pause_menu_button_rects(self.width, self.height)
    }

    pub fn draw_main_menu(&mut self, hovered: Option<usize>) {
        crate::ui::draw_main_menu(&mut self.color_buffer, self.width, self.height, hovered);
    }

    pub fn draw_save_select_menu(
        &mut self,
        selected_slot: u8,
        scroll_offset: usize,
        metadata_cache: &[crate::world::storage::SaveMetadata; crate::ui::MAX_SAVE_SLOTS],
        hovered_card: Option<usize>,
        hovered_btn: Option<usize>,
    ) {
        crate::ui::draw_save_select_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            selected_slot,
            scroll_offset,
            metadata_cache,
            hovered_card,
            hovered_btn,
        );
    }

    pub fn draw_create_world_menu(
        &mut self,
        slot: u8,
        name: &str,
        seed_str: &str,
        gamemode: crate::gameplay::player::GameMode,
        focused_element: usize,
        hovered_element: Option<usize>,
        blink_timer: f32,
        is_selected: bool,
    ) {
        crate::ui::draw_create_world_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            slot,
            name,
            seed_str,
            gamemode,
            focused_element,
            hovered_element,
            blink_timer,
            is_selected,
        );
    }

    pub fn draw_settings_menu(
        &mut self,
        settings: &crate::core::settings::GameSettings,
        active_tab: usize,
        hovered_tab: Option<usize>,
        hovered: Option<usize>,
        server_ip: &str,
    ) {
        crate::ui::draw_settings_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            settings,
            active_tab,
            hovered_tab,
            hovered,
            server_ip,
        );
    }

    pub fn draw_crosshair(&mut self) {
        crate::ui::draw_crosshair(&mut self.color_buffer, self.width, self.height);
    }

    pub fn draw_break_progress(&mut self, progress: f32) {
        crate::ui::draw_break_progress(&mut self.color_buffer, self.width, self.height, progress);
    }

    pub fn draw_underwater_overlay(&mut self, time: f32) {
        crate::ui::draw_underwater_overlay(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_lava_overlay(&mut self, time: f32) {
        crate::ui::draw_lava_overlay(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_hotbar(&mut self, player: &crate::gameplay::Player) {
        crate::ui::draw_hotbar(&mut self.color_buffer, self.width, self.height, player);
    }

    pub fn draw_inventory(
        &mut self,
        player: &crate::gameplay::Player,
        hovered_slot: Option<crate::ui::InventorySlotTarget>,
        crafting_table_open: bool,
        recipes: &crate::gameplay::RecipeBook,
        mouse_pos: Option<(f32, f32)>,
    ) {
        crate::ui::draw_inventory_screen(
            &mut self.color_buffer,
            self.width,
            self.height,
            player,
            hovered_slot,
            crafting_table_open,
            recipes,
            mouse_pos,
        );
    }

    pub fn draw_pause_menu(&mut self, fly_mode: bool, hovered: Option<usize>) {
        crate::ui::draw_pause_menu(&mut self.color_buffer, self.width, self.height, fly_mode, hovered);
    }

    pub fn draw_capture_prompt(&mut self, native: bool) {
        crate::ui::draw_capture_prompt(&mut self.color_buffer, self.width, self.height, native);
    }

    pub fn draw_debug_panel(&mut self, lines: &[String]) {
        crate::ui::draw_debug_panel(&mut self.color_buffer, self.width, self.height, lines);
    }

    pub fn draw_advanced_debug_overlay(
        &mut self,
        left_lines: &[String],
        right_lines: &[String],
        profiler: &crate::core::profiler::FrameProfiler,
    ) {
        crate::ui::draw_advanced_debug_overlay(
            &mut self.color_buffer,
            self.width,
            self.height,
            left_lines,
            right_lines,
            profiler,
        );
    }

    #[inline(always)]
    pub fn death_screen_button_rects(&self) -> [(i32, i32, i32, i32); 2] {
        crate::ui::death_screen_button_rects(self.width, self.height)
    }

    pub fn draw_damage_flash(&mut self, intensity: f32) {
        crate::ui::draw_damage_flash(&mut self.color_buffer, self.width, self.height, intensity);
    }

    pub fn draw_low_health_vignette(&mut self, time: f32) {
        crate::ui::draw_low_health_vignette(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_death_screen(&mut self, hovered: Option<usize>) {
        crate::ui::draw_death_screen(&mut self.color_buffer, self.width, self.height, hovered);
    }

    pub fn render_frame(&mut self, world: &mut World, camera: &Camera) {
        let (sky_top, sky_bottom, fog_col) = world.sky_colors();
        let daylight = world.sunlight_factor();
        self.current_daylight = daylight;
        self.current_fog_color = fog_col;

        let aspect = self.width as f32 / self.height as f32;
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect);
        let view_proj = proj.mul(&view);

        if self.vulkan.is_none() {
            self.clear_sky(camera.pitch, sky_top, sky_bottom);
            self.draw_celestials(camera, &view_proj, world.day_fraction());
        }

        self.loaded_keys.clear();
        self.loaded_keys.extend(world.chunks.keys().copied());
        self.mesh_cache.retain(|k, _| world.chunks.contains_key(k));
        self.chunk_spawn_times.retain(|k, _| world.chunks.contains_key(k));
        self.dirty_keys.clear();

        for key in &self.loaded_keys {
            self.chunk_spawn_times.entry(*key).or_insert(self.total_time);
        }

        // 1. Drain completed meshes from background threads
        for (key, mesh) in self.async_mesher.drain_completed() {
            self.mesh_cache.insert(key, mesh);
            self.pending_mesh_keys.remove(&key);
            self.dirty_keys.insert(key);
        }

        self.pending_mesh_keys.retain(|k| world.chunks.contains_key(k));

        // 2. Submit background meshing jobs for dirty or unmeshed chunks
        let dirty_keys_to_build: Vec<(i32, i32)> = self
            .loaded_keys
            .iter()
            .copied()
            .filter(|key| {
                if self.pending_mesh_keys.contains(key) {
                    return false;
                }
                world
                    .chunks
                    .get(key)
                    .map_or(false, |chunk| chunk.dirty || !self.mesh_cache.contains_key(key))
            })
            .collect();

        for key in dirty_keys_to_build {
            if let Some(chunk) = world.chunks.get_mut(&key) {
                chunk.dirty = false;
            }
            if let Some(job) = world.create_mesh_job(key) {
                self.pending_mesh_keys.insert(key);
                self.async_mesher.submit_job(job);
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        if let Some(vulkan) = self.vulkan.as_mut() {
            let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);
            self.visible_keys.clear();
            if is_occluded {
                let (cam_cx, cam_cz, _, _) = World::world_to_chunk(camera.position.x.floor() as i32, camera.position.z.floor() as i32);
                if self.loaded_keys.contains(&(cam_cx, cam_cz)) {
                    self.visible_keys.push((cam_cx, cam_cz));
                }
            } else {
                for key in &self.loaded_keys {
                    if Self::chunk_visible(*key, &view_proj) {
                        self.visible_keys.push(*key);
                    }
                }
            }

            vulkan.render_world(
                &self.mesh_cache,
                &self.loaded_keys,
                &self.dirty_keys,
                &self.visible_keys,
                &view_proj,
                camera.position,
                camera.pitch,
                self.render_distance,
                self.fog_enabled,
                self.chunk_anim,
                self.total_time,
                &self.chunk_spawn_times,
                daylight,
                world.day_fraction(),
                world.sun_direction(),
                (sky_top, sky_bottom, fog_col),
                &mut self.color_buffer,
                &mut self.depth_buffer,
            );

            self.last_draw_calls = self.visible_keys.len() as u32;
            let total_mesh_tris: usize = self.loaded_keys.iter().filter_map(|k| self.mesh_cache.get(k)).map(|v| v.len()).sum();
            let visible_mesh_tris: usize = self.visible_keys.iter().filter_map(|k| self.mesh_cache.get(k)).map(|v| v.len()).sum();
            self.last_triangles_rendered = visible_mesh_tris as u32;
            self.last_triangles_culled = total_mesh_tris.saturating_sub(visible_mesh_tris) as u32;
            self.last_subchunks_rendered = (self.visible_keys.len() * crate::world::mesher::NUM_SUBCHUNKS) as u32;
            self.last_subchunks_culled = (self.loaded_keys.len().saturating_sub(self.visible_keys.len()) * crate::world::mesher::NUM_SUBCHUNKS) as u32;

            return;
        }

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        let mut visible: Vec<(i32, i32)> = self
            .loaded_keys
            .par_iter()
            .copied()
            .filter(|&key| Self::chunk_visible(key, &view_proj))
            .collect();

        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        let mut visible: Vec<(i32, i32)> = self
            .loaded_keys
            .iter()
            .copied()
            .filter(|&key| Self::chunk_visible(key, &view_proj))
            .collect();

        let cam_x = camera.position.x - CHUNK_SIZE_X as f32 * 0.5;
        let cam_z = camera.position.z - CHUNK_SIZE_Z as f32 * 0.5;

        let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);
        if is_occluded {
            let (cam_cx, cam_cz, _, _) = World::world_to_chunk(camera.position.x.floor() as i32, camera.position.z.floor() as i32);
            visible.retain(|&k| k == (cam_cx, cam_cz));
        }

        visible.sort_unstable_by_key(|a| {
            let dx_a = a.0 as f32 * CHUNK_SIZE_X as f32 - cam_x;
            let dz_a = a.1 as f32 * CHUNK_SIZE_Z as f32 - cam_z;
            (dx_a * dx_a + dz_a * dz_a).to_bits()
        });

        self.opaque_tris_buf.clear();
        self.transparent_tris_buf.clear();

        let width = self.width;
        let height = self.height;
        let mesh_cache = &self.mesh_cache;
        let chunk_spawn_times = &self.chunk_spawn_times;
        let current_time = self.total_time;
        let chunk_anim = self.chunk_anim;

        let process_chunk = |key: &(i32, i32)| {
            let mut local_opaque = Vec::with_capacity(256);
            let mut local_transparent = Vec::with_capacity(64);

            let spawn_time = chunk_spawn_times.get(key).copied().unwrap_or(current_time);
            let elapsed = current_time - spawn_time;
            let y_offset = if chunk_anim && elapsed >= 0.0 && elapsed < 0.35 {
                let t = (elapsed / 0.35).clamp(0.0, 1.0);
                let ease = 1.0 - (1.0 - t).powi(3);
                (1.0 - ease) * -14.0
            } else {
                0.0
            };

            let chunk_vp = if y_offset != 0.0 {
                view_proj.mul(&Mat4::translation(0.0, y_offset, 0.0))
            } else {
                view_proj
            };

            if let Some(tris) = mesh_cache.get(key) {
                for tri in tris {
                    let c0 = to_clip_ao(tri.v0, &chunk_vp);
                    let c1 = to_clip_ao(tri.v1, &chunk_vp);
                    let c2 = to_clip_ao(tri.v2, &chunk_vp);

                    let clipped = clip_near_ao(&[c0, c1, c2]);
                    if clipped.len() < 3 {
                        continue;
                    }

                    let is_transparent = matches!(
                        tri.texture,
                        TextureId::Water | TextureId::Glass
                    );

                    for i in 1..clipped.len() - 1 {
                        let sa = project_to_screen_ao(&clipped[0], width, height);
                        let sb = project_to_screen_ao(&clipped[i], width, height);
                        let sc = project_to_screen_ao(&clipped[i + 1], width, height);

                        let area = edge(sa.x, sa.y, sb.x, sb.y, sc.x, sc.y);
                        if area <= 0.0 {
                            continue;
                        }

                        let min_y = sa.y.min(sb.y).min(sc.y).floor() as i32;
                        let max_y = sa.y.max(sb.y).max(sc.y).ceil() as i32;

                        if min_y >= height as i32 || max_y < 0 {
                            continue;
                        }

                        let stri = ScreenTriangle {
                            sa,
                            sb,
                            sc,
                            texture: tri.texture,
                            min_y,
                            max_y,
                        };

                        if is_transparent {
                            local_transparent.push(stri);
                        } else {
                            local_opaque.push(stri);
                        }
                    }
                }
            }
            (local_opaque, local_transparent)
        };

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        let (opaque_chunks, transparent_chunks): (Vec<Vec<ScreenTriangle>>, Vec<Vec<ScreenTriangle>>) = visible
            .par_iter()
            .map(process_chunk)
            .unzip();

        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        let (opaque_chunks, transparent_chunks): (Vec<Vec<ScreenTriangle>>, Vec<Vec<ScreenTriangle>>) = visible
            .iter()
            .map(process_chunk)
            .unzip();

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        {
            self.opaque_tris_buf = opaque_chunks.into_par_iter().flatten().collect();
            self.transparent_tris_buf = transparent_chunks.into_par_iter().flatten().collect();
        }
        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        {
            self.opaque_tris_buf = opaque_chunks.into_iter().flatten().collect();
            self.transparent_tris_buf = transparent_chunks.into_iter().flatten().collect();
        }

        self.last_draw_calls = visible.len() as u32;
        self.last_triangles_rendered = (self.opaque_tris_buf.len() + self.transparent_tris_buf.len()) as u32;
        let total_mesh_tris: usize = self.loaded_keys.iter().filter_map(|k| self.mesh_cache.get(k)).map(|v| v.len()).sum();
        self.last_triangles_culled = total_mesh_tris.saturating_sub(self.last_triangles_rendered as usize) as u32;
        self.last_subchunks_rendered = (visible.len() * crate::world::mesher::NUM_SUBCHUNKS) as u32;
        self.last_subchunks_culled = (self.loaded_keys.len().saturating_sub(visible.len()) * crate::world::mesher::NUM_SUBCHUNKS) as u32;

        let textures = &self.textures;
        let opaque_slice = self.opaque_tris_buf.as_slice();
        let transparent_slice = self.transparent_tris_buf.as_slice();

        let color_chunks = self.color_buffer.chunks_mut(width * TILE_HEIGHT);
        let depth_chunks = self.depth_buffer.chunks_mut(width * TILE_HEIGHT);

        let fog_dist = if self.fog_enabled {
            (self.render_distance as f32 * 16.0).max(32.0)
        } else {
            0.0
        };
        let daylight = self.current_daylight;
        let fog_color = self.current_fog_color;
        let render_tile = |(tile_idx, (c_slice, d_slice)): (usize, (&mut [u32], &mut [f32]))| {
            let tile_y_start = tile_idx * TILE_HEIGHT;
            let tile_y_end = (tile_y_start + TILE_HEIGHT).min(height);

            for stri in opaque_slice {
                if stri.max_y < tile_y_start as i32 || stri.min_y >= tile_y_end as i32 {
                    continue;
                }
                raster_screen_triangle_ao_tile(
                    stri.sa,
                    stri.sb,
                    stri.sc,
                    stri.texture,
                    0.0,
                    width,
                    tile_y_start,
                    tile_y_end,
                    c_slice,
                    d_slice,
                    textures,
                    fog_dist,
                    daylight,
                    fog_color,
                );
            }

            for stri in transparent_slice.iter().rev() {
                if stri.max_y < tile_y_start as i32 || stri.min_y >= tile_y_end as i32 {
                    continue;
                }
                raster_screen_triangle_ao_tile(
                    stri.sa,
                    stri.sb,
                    stri.sc,
                    stri.texture,
                    0.0,
                    width,
                    tile_y_start,
                    tile_y_end,
                    c_slice,
                    d_slice,
                    textures,
                    fog_dist,
                    daylight,
                    fog_color,
                );
            }
        };

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
        color_chunks
            .zip(depth_chunks)
            .enumerate()
            .collect::<Vec<_>>()
            .into_par_iter()
            .for_each(render_tile);

        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
        color_chunks
            .zip(depth_chunks)
            .enumerate()
            .for_each(render_tile);
    }

    fn chunk_visible(key: (i32, i32), view_proj: &Mat4) -> bool {
        let min_x = key.0 as f32 * CHUNK_SIZE_X as f32;
        let min_y = -32.0;
        let min_z = key.1 as f32 * CHUNK_SIZE_Z as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let max_y = 128.0;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        let corners = [
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, min_y, min_z),
            Vec3::new(min_x, max_y, min_z),
            Vec3::new(max_x, max_y, min_z),
            Vec3::new(min_x, min_y, max_z),
            Vec3::new(max_x, min_y, max_z),
            Vec3::new(min_x, max_y, max_z),
            Vec3::new(max_x, max_y, max_z),
        ];

        let mut out_x_pos = 0u8; let mut out_x_neg = 0u8;
        let mut out_y_pos = 0u8; let mut out_y_neg = 0u8;
        let mut out_z_pos = 0u8; let mut out_z_neg = 0u8;

        for (i, &corner) in corners.iter().enumerate() {
            let clip = view_proj.transform_point(corner);
            if clip[0] > clip[3] { out_x_pos += 1; }
            if clip[0] < -clip[3] { out_x_neg += 1; }
            if clip[1] > clip[3] { out_y_pos += 1; }
            if clip[1] < -clip[3] { out_y_neg += 1; }
            if clip[2] > clip[3] { out_z_pos += 1; }
            if clip[2] < -clip[3] { out_z_neg += 1; }

            let remaining = (corners.len() - 1 - i) as u8;
            let none_can_reach_8 = out_x_pos + remaining < 8
                && out_x_neg + remaining < 8
                && out_y_pos + remaining < 8
                && out_y_neg + remaining < 8
                && out_z_pos + remaining < 8
                && out_z_neg + remaining < 8;
            if none_can_reach_8 {
                return true;
            }
        }

        !(out_x_pos == 8 || out_x_neg == 8 || out_y_pos == 8 || out_y_neg == 8 || out_z_pos == 8 || out_z_neg == 8)
    }

    pub fn subchunk_visible(cx: i32, sub_y: usize, cz: i32, view_proj: &Mat4) -> bool {
        let min_x = cx as f32 * CHUNK_SIZE_X as f32;
        let min_y = (sub_y as i32 * crate::world::SUBCHUNK_SIZE) as f32 - 14.0;
        let min_z = cz as f32 * CHUNK_SIZE_Z as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let max_y = (sub_y as i32 * crate::world::SUBCHUNK_SIZE + crate::world::SUBCHUNK_SIZE) as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        let corners = [
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, min_y, min_z),
            Vec3::new(min_x, max_y, min_z),
            Vec3::new(max_x, max_y, min_z),
            Vec3::new(min_x, min_y, max_z),
            Vec3::new(max_x, min_y, max_z),
            Vec3::new(min_x, max_y, max_z),
            Vec3::new(max_x, max_y, max_z),
        ];

        let mut out_x_pos = 0u8; let mut out_x_neg = 0u8;
        let mut out_y_pos = 0u8; let mut out_y_neg = 0u8;
        let mut out_z_pos = 0u8; let mut out_z_neg = 0u8;

        for (i, &corner) in corners.iter().enumerate() {
            let clip = view_proj.transform_point(corner);
            if clip[0] > clip[3] { out_x_pos += 1; }
            if clip[0] < -clip[3] { out_x_neg += 1; }
            if clip[1] > clip[3] { out_y_pos += 1; }
            if clip[1] < -clip[3] { out_y_neg += 1; }
            if clip[2] > clip[3] { out_z_pos += 1; }
            if clip[2] < -clip[3] { out_z_neg += 1; }

            let remaining = (corners.len() - 1 - i) as u8;
            let none_can_reach_8 = out_x_pos + remaining < 8
                && out_x_neg + remaining < 8
                && out_y_pos + remaining < 8
                && out_y_neg + remaining < 8
                && out_z_pos + remaining < 8
                && out_z_neg + remaining < 8;
            if none_can_reach_8 {
                return true;
            }
        }

        !(out_x_pos == 8 || out_x_neg == 8 || out_y_pos == 8 || out_y_neg == 8 || out_z_pos == 8 || out_z_neg == 8)
    }

    fn raster_triangle(&mut self, tri: &Triangle, view_proj: &Mat4) {
        self.raster_triangle_tinted(tri, 0.0, view_proj);
    }

    pub fn raster_triangle_tinted(&mut self, tri: &Triangle, hurt_tint: f32, view_proj: &Mat4) {
        let width = self.width;
        let height = self.height;
        let textures = &self.textures;

        let fog_dist = if self.fog_enabled {
            (self.render_distance as f32 * 16.0).max(32.0)
        } else {
            0.0
        };
        raster_triangle_tile(
            tri,
            hurt_tint,
            view_proj,
            width,
            height,
            0,
            height,
            &mut self.color_buffer,
            &mut self.depth_buffer,
            textures,
            fog_dist,
            self.current_daylight,
            self.current_fog_color,
        );
    }

    pub fn draw_billboard(
        &mut self,
        pos: Vec3,
        size: f32,
        color: [f32; 3],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let right = camera.right_flat();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let h = size * 0.5;

        let corners = [
            pos - right * h - up * h,
            pos + right * h - up * h,
            pos + right * h + up * h,
            pos - right * h + up * h,
        ];

        let project = |p: Vec3| -> Option<(f32, f32, f32)> {
            let c = view_proj.transform_point(p);
            if c[3] <= 1e-4 { return None; }
            let inv_w = 1.0 / c[3];
            let sx = (c[0] * inv_w * 0.5 + 0.5) * self.width as f32;
            let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * self.height as f32;
            Some((sx, sy, 1.0 / c[3]))
        };

        let pts: Vec<_> = corners.iter().filter_map(|&p| project(p)).collect();
        if pts.len() < 4 { return; }

        let depth_at_pos = {
            let c = view_proj.transform_point(pos);
            if c[3] <= 1e-4 { return; }
            c[3]
        };

        let packed = pack_color(color[0], color[1], color[2]);
        for &tri_idx in &[(0usize, 1usize, 2usize), (0usize, 2usize, 3usize)] {
            let (ai, bi, ci) = tri_idx;
            let (ax, ay, _) = pts[ai];
            let (bx, by, _) = pts[bi];
            let (cx, cy, _) = pts[ci];
            let area = edge(ax, ay, bx, by, cx, cy);
            if area <= 0.0 { continue; }
            let min_x = ax.min(bx).min(cx).floor().max(0.0) as i32;
            let max_x = ax.max(bx).max(cx).ceil().min(self.width as f32 - 1.0) as i32;
            let min_y = ay.min(by).min(cy).floor().max(0.0) as i32;
            let max_y = ay.max(by).max(cy).ceil().min(self.height as f32 - 1.0) as i32;
            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let fx = px as f32 + 0.5;
                    let fy = py as f32 + 0.5;
                    let w0 = edge(bx, by, cx, cy, fx, fy);
                    let w1 = edge(cx, cy, ax, ay, fx, fy);
                    let w2 = edge(ax, ay, bx, by, fx, fy);
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let idx = py as usize * self.width + px as usize;
                        if depth_at_pos < self.depth_buffer[idx] {
                            self.color_buffer[idx] = packed;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_clouds(
        &mut self,
        player_pos: Vec3,
        view_proj: &Mat4,
        time: f32,
    ) {
        let cloud_y = 75.0f32;
        let cloud_thickness = 4.0f32;
        let y_bottom = cloud_y;
        let y_top = cloud_y + cloud_thickness;

        if (player_pos.y - cloud_y).abs() > 100.0 {
            return;
        }

        let cloud_size = 12.0f32;
        let scroll = time * 1.5;

        let is_cloud = |gx: i32, gz: i32| -> bool {
            let hx = gx.wrapping_mul(374761393i32);
            let hz = gz.wrapping_mul(668265263i32);
            let h = (hx ^ hz ^ hx.wrapping_mul(hz >> 3)) as u32;
            let fx = gx as f32 * 0.16;
            let fz = gz as f32 * 0.16;
            let wave = (fx.sin() + fz.cos() + (fx * 0.7 + fz * 0.5).sin()) * 0.35;
            let r = ((h >> 16) % 100) as f32 / 100.0;
            (wave + r * 0.35) > 0.08
        };

        let cx = ((player_pos.x - scroll) / cloud_size).floor() as i32;
        let cz = (player_pos.z / cloud_size).floor() as i32;

        let alpha = 0.80f32;

        for dz in -7..=7 {
            for dx in -7..=7 {
                let gx = cx + dx;
                let gz = cz + dz;

                let x_min = gx as f32 * cloud_size + scroll;
                let x_max = x_min + cloud_size;
                let z_min = gz as f32 * cloud_size;
                let z_max = z_min + cloud_size;

                let center_x = (x_min + x_max) * 0.5;
                let center_z = (z_min + z_max) * 0.5;
                let dist_sq = (center_x - player_pos.x).powi(2) + (center_z - player_pos.z).powi(2);
                if dist_sq > (7.5 * cloud_size).powi(2) {
                    continue;
                }

                if !cloud_column_visible(x_min, x_max, y_bottom, y_top, z_min, z_max, view_proj) {
                    continue;
                }

                if !is_cloud(gx, gz) {
                    continue;
                }

                let p000 = Vec3::new(x_min, y_bottom, z_min);
                let p100 = Vec3::new(x_max, y_bottom, z_min);
                let p001 = Vec3::new(x_min, y_bottom, z_max);
                let p101 = Vec3::new(x_max, y_bottom, z_max);
                let p010 = Vec3::new(x_min, y_top, z_min);
                let p110 = Vec3::new(x_max, y_top, z_min);
                let p011 = Vec3::new(x_min, y_top, z_max);
                let p111 = Vec3::new(x_max, y_top, z_max);

                let draw_quad = |renderer: &mut Renderer,
                                 v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3,
                                 shade: f32, view_proj: &Mat4| {
                    let mut emit_triangle = |ta: Vec3, tb: Vec3, tc: Vec3| {
                        let c0 = to_clip_ao(Vertex::new(ta, (0.0, 0.0), shade), view_proj);
                        let c1 = to_clip_ao(Vertex::new(tb, (0.0, 0.0), shade), view_proj);
                        let c2 = to_clip_ao(Vertex::new(tc, (0.0, 0.0), shade), view_proj);

                        let clipped = clip_near_ao(&[c0, c1, c2]);
                        if clipped.len() < 3 { return; }

                        let width = renderer.width;
                        let height = renderer.height;

                        for i in 1..clipped.len() - 1 {
                            let sa = project_to_screen_ao(&clipped[0], width, height);
                            let sb = project_to_screen_ao(&clipped[i], width, height);
                            let sc = project_to_screen_ao(&clipped[i + 1], width, height);

                            let area = edge(sa.x, sa.y, sb.x, sb.y, sc.x, sc.y);
                            if area <= 0.0 { continue; }

                            let min_x = sa.x.min(sb.x).min(sc.x).floor().max(0.0) as i32;
                            let max_x = sa.x.max(sb.x).max(sc.x).ceil().min(width as f32 - 1.0) as i32;
                            let min_y = sa.y.min(sb.y).min(sc.y).floor().max(0.0) as i32;
                            let max_y = sa.y.max(sb.y).max(sc.y).ceil().min(height as f32 - 1.0) as i32;

                            if min_x > max_x || min_y > max_y { continue; }

                            let inv_area = 1.0 / area;
                            let dx0 = sc.x - sb.x; let dy0 = sc.y - sb.y;
                            let dx1 = sa.x - sc.x; let dy1 = sa.y - sc.y;
                            let dx2 = sb.x - sa.x; let dy2 = sb.y - sa.y;

                            let fx0 = min_x as f32 + 0.5;
                            let fy0 = min_y as f32 + 0.5;
                            let mut w0_row = edge(sb.x, sb.y, sc.x, sc.y, fx0, fy0);
                            let mut w1_row = edge(sc.x, sc.y, sa.x, sa.y, fx0, fy0);
                            let mut w2_row = edge(sa.x, sa.y, sb.x, sb.y, fx0, fy0);

                            for py in min_y..=max_y {
                                let mut w0 = w0_row;
                                let mut w1 = w1_row;
                                let mut w2 = w2_row;
                                let row_offset = py as usize * width;

                                for px in min_x..=max_x {
                                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                                        let l0 = w0 * inv_area;
                                        let l1 = w1 * inv_area;
                                        let l2 = w2 * inv_area;

                                        let inv_w = l0 * sa.inv_w + l1 * sb.inv_w + l2 * sc.inv_w;
                                        if inv_w > 0.0 {
                                            let depth = 1.0 / inv_w;
                                            let idx = row_offset + px as usize;
                                            if depth < renderer.depth_buffer[idx] {
                                                let ao = (l0 * sa.ao_over_w + l1 * sb.ao_over_w + l2 * sc.ao_over_w) * depth;
                                                let base_val = (0.95 * ao).clamp(0.0, 1.0);
                                                let fog_dist = if renderer.fog_enabled {
                                                    (renderer.render_distance as f32 * 16.0).max(32.0)
                                                } else {
                                                    0.0
                                                };
                                                let fog_t = if fog_dist > 0.0 {
                                                    ((depth - fog_dist * 0.45) / (fog_dist * 0.55)).clamp(0.0, 1.0)
                                                } else {
                                                    0.0
                                                };
                                                let cr = base_val * (1.0 - fog_t) + 0.75 * fog_t;
                                                let cg = base_val * (1.0 - fog_t) + 0.85 * fog_t;
                                                let cb = base_val * (1.0 - fog_t) + 1.0 * fog_t;

                                                let (br, bg, bb) = unpack_color(renderer.color_buffer[idx]);
                                                renderer.color_buffer[idx] = pack_color(
                                                    cr * alpha + br * (1.0 - alpha),
                                                    cg * alpha + bg * (1.0 - alpha),
                                                    cb * alpha + bb * (1.0 - alpha),
                                                );
                                            }
                                        }
                                    }
                                    w0 += dy0; w1 += dy1; w2 += dy2;
                                }
                                w0_row -= dx0; w1_row -= dx1; w2_row -= dx2;
                            }
                        }
                    };

                    emit_triangle(v0, v1, v2);
                    emit_triangle(v0, v2, v3);
                };

                if player_pos.y >= y_top {
                    draw_quad(self, p010, p011, p111, p110, 1.00, view_proj);
                }
                if player_pos.y <= y_bottom {
                    draw_quad(self, p001, p000, p100, p101, 0.70, view_proj);
                }
                if player_pos.z <= z_min && !is_cloud(gx, gz - 1) {
                    draw_quad(self, p100, p000, p010, p110, 0.85, view_proj);
                }
                if player_pos.z >= z_max && !is_cloud(gx, gz + 1) {
                    draw_quad(self, p001, p101, p111, p011, 0.85, view_proj);
                }
                if player_pos.x <= x_min && !is_cloud(gx - 1, gz) {
                    draw_quad(self, p000, p001, p011, p010, 0.78, view_proj);
                }
                if player_pos.x >= x_max && !is_cloud(gx + 1, gz) {
                    draw_quad(self, p101, p100, p110, p111, 0.78, view_proj);
                }
            }
        }
    }

    pub fn draw_block_face(
        &mut self,
        pos: Vec3,
        face: crate::world::block::Face,
        _color: [f32; 3],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        use crate::world::block::Face;

        if face == Face::Bottom && pos.y <= 0.0 {
            return;
        }
        let cam_p = camera.position;
        let is_backfacing = match face {
            Face::Top => cam_p.y < pos.y + 1.0,
            Face::Bottom => cam_p.y > pos.y,
            Face::North => cam_p.z > pos.z,
            Face::South => cam_p.z < pos.z + 1.0,
            Face::West => cam_p.x > pos.x,
            Face::East => cam_p.x < pos.x + 1.0,
        };
        if is_backfacing {
            return;
        }

        let (v0, v1, v2, v3) = match face {
            Face::Top => (
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            Face::Bottom => (
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
            ),
            Face::North => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
            ),
            Face::South => (
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
            ),
            Face::West => (
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            Face::East => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
            ),
        };

        let tex_id = match face {
            Face::Top | Face::Bottom => TextureId::TntTop,
            _ => TextureId::TntSide,
        };

        let t1 = Triangle {
            v0: Vertex::new(v0, (0.0, 0.0), 1.0),
            v1: Vertex::new(v1, (1.0, 0.0), 1.0),
            v2: Vertex::new(v2, (1.0, 1.0), 1.0),
            texture: tex_id,
        };
        let t2 = Triangle {
            v0: Vertex::new(v0, (0.0, 0.0), 1.0),
            v1: Vertex::new(v2, (1.0, 1.0), 1.0),
            v2: Vertex::new(v3, (0.0, 1.0), 1.0),
            texture: tex_id,
        };

        self.raster_triangle(&t1, view_proj);
        self.raster_triangle(&t2, view_proj);
    }

    pub fn draw_tnt_cube(&mut self, pos: Vec3, fuse: f32, camera: &Camera, view_proj: &Mat4) {
        use crate::world::block::{BlockId, Face};

        let flash = (fuse * 12.0).sin().abs() * 0.6;
        let white_tint = Vec3::new(flash, flash, flash);

        for face in [Face::Top, Face::Bottom, Face::North, Face::South, Face::East, Face::West] {
            let color_base = BlockId::Tnt.face_color(face);
            let color = [
                (color_base[0] + white_tint.x).min(1.0),
                (color_base[1] + white_tint.y).min(1.0),
                (color_base[2] + white_tint.z).min(1.0),
            ];

            self.draw_block_face(pos, face, color, camera, view_proj);
        }
    }

    pub fn draw_mob(&mut self, mob: &crate::gameplay::mob::Mob, camera: &Camera, view_proj: &Mat4) {
        use crate::gameplay::mob::MobKind;

        let hurt_tint = if mob.hurt_timer > 0.0 { 0.7 } else { 0.0 };
        let death_t = (mob.death_timer / 0.8).clamp(0.0, 1.0);
        let roll_angle = death_t * std::f32::consts::FRAC_PI_2;

        let base_pos = mob.pos;
        let yaw = mob.yaw;
        let leg_swing = mob.walk_phase.sin() * 0.65;
        let camera_pos = camera.position;

        match mob.kind {
            MobKind::Pig => {
                let body_box = MobBox {
                    center: Vec3::new(0.0, 0.45, 0.0),
                    size: Vec3::new(0.65, 0.45, 0.90),
                    yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: base_pos,
                    tex_top: TextureId::PigSkin,
                    tex_bottom: TextureId::PigSkin,
                    tex_side: TextureId::PigSkin,
                    tex_front: TextureId::PigSkin,
                };
                self.draw_mob_box(&body_box, hurt_tint, camera_pos, view_proj);

                let head_pos = base_pos + Vec3::new(yaw.sin() * 0.48, 0.52, yaw.cos() * 0.48);
                let head_yaw = yaw + mob.head_yaw_offset;
                let head_box = MobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.48, 0.48, 0.48),
                    yaw: head_yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: head_pos,
                    tex_top: TextureId::PigSkin,
                    tex_bottom: TextureId::PigSkin,
                    tex_side: TextureId::PigSkin,
                    tex_front: TextureId::PigFace,
                };
                self.draw_mob_box(&head_box, hurt_tint, camera_pos, view_proj);

                let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.28, -0.06, head_yaw.cos() * 0.28);
                let snout_box = MobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.24, 0.18, 0.12),
                    yaw: head_yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: snout_pos,
                    tex_top: TextureId::PigSkin,
                    tex_bottom: TextureId::PigSkin,
                    tex_side: TextureId::PigSkin,
                    tex_front: TextureId::PigSnout,
                };
                self.draw_mob_box(&snout_box, hurt_tint, camera_pos, view_proj);

                let leg_offsets = [
                    (-0.22, -0.28, leg_swing),
                    (0.22, -0.28, -leg_swing),
                    (-0.22, 0.28, -leg_swing),
                    (0.22, 0.28, leg_swing),
                ];

                for &(lx, lz, swing) in &leg_offsets {
                    let rotated_x = lx * yaw.cos() + lz * yaw.sin();
                    let rotated_z = -lx * yaw.sin() + lz * yaw.cos();
                    let leg_origin = base_pos + Vec3::new(rotated_x, 0.30, rotated_z);

                    let leg_box = MobBox {
                        center: Vec3::new(0.0, -0.15, 0.0),
                        size: Vec3::new(0.20, 0.30, 0.20),
                        yaw,
                        pitch: swing,
                        roll: roll_angle,
                        origin: leg_origin,
                        tex_top: TextureId::PigSkin,
                        tex_bottom: TextureId::PigSkin,
                        tex_side: TextureId::PigSkin,
                        tex_front: TextureId::PigSkin,
                    };
                    self.draw_mob_box(&leg_box, hurt_tint, camera_pos, view_proj);
                }
            }
            MobKind::Cow => {
                let body_box = MobBox {
                    center: Vec3::new(0.0, 0.78, 0.0),
                    size: Vec3::new(0.75, 0.65, 1.15),
                    yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: base_pos,
                    tex_top: TextureId::CowSkin,
                    tex_bottom: TextureId::CowSkin,
                    tex_side: TextureId::CowSkin,
                    tex_front: TextureId::CowSkin,
                };
                self.draw_mob_box(&body_box, hurt_tint, camera_pos, view_proj);

                let udder_pos = base_pos + Vec3::new(0.0, 0.44, 0.0);
                let udder_box = MobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.28, 0.12, 0.36),
                    yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: udder_pos,
                    tex_top: TextureId::CowUdder,
                    tex_bottom: TextureId::CowUdder,
                    tex_side: TextureId::CowUdder,
                    tex_front: TextureId::CowUdder,
                };
                self.draw_mob_box(&udder_box, hurt_tint, camera_pos, view_proj);

                let head_pos = base_pos + Vec3::new(yaw.sin() * 0.62, 0.95, yaw.cos() * 0.62);
                let head_yaw = yaw + mob.head_yaw_offset;
                let head_box = MobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.48, 0.48, 0.46),
                    yaw: head_yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: head_pos,
                    tex_top: TextureId::CowSkin,
                    tex_bottom: TextureId::CowSkin,
                    tex_side: TextureId::CowSkin,
                    tex_front: TextureId::CowFace,
                };
                self.draw_mob_box(&head_box, hurt_tint, camera_pos, view_proj);

                let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.26, -0.10, head_yaw.cos() * 0.26);
                let snout_box = MobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.34, 0.22, 0.16),
                    yaw: head_yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: snout_pos,
                    tex_top: TextureId::CowSkin,
                    tex_bottom: TextureId::CowSkin,
                    tex_side: TextureId::CowSkin,
                    tex_front: TextureId::CowSnout,
                };
                self.draw_mob_box(&snout_box, hurt_tint, camera_pos, view_proj);

                for side in [-1.0f32, 1.0f32] {
                    let horn_x = side * 0.28 * head_yaw.cos();
                    let horn_z = -side * 0.28 * head_yaw.sin();
                    let horn_pos = head_pos + Vec3::new(horn_x, 0.26, horn_z);
                    let horn_box = MobBox {
                        center: Vec3::zero(),
                        size: Vec3::new(0.10, 0.18, 0.10),
                        yaw: head_yaw,
                        pitch: 0.0,
                        roll: roll_angle,
                        origin: horn_pos,
                        tex_top: TextureId::CowHorns,
                        tex_bottom: TextureId::CowHorns,
                        tex_side: TextureId::CowHorns,
                        tex_front: TextureId::CowHorns,
                    };
                    self.draw_mob_box(&horn_box, hurt_tint, camera_pos, view_proj);
                }

                let leg_offsets = [
                    (-0.25, -0.38, leg_swing),
                    (0.25, -0.38, -leg_swing),
                    (-0.25, 0.38, -leg_swing),
                    (0.25, 0.38, leg_swing),
                ];

                for &(lx, lz, swing) in &leg_offsets {
                    let rotated_x = lx * yaw.cos() + lz * yaw.sin();
                    let rotated_z = -lx * yaw.sin() + lz * yaw.cos();
                    let leg_origin = base_pos + Vec3::new(rotated_x, 0.50, rotated_z);

                    let leg_box = MobBox {
                        center: Vec3::new(0.0, -0.25, 0.0),
                        size: Vec3::new(0.22, 0.50, 0.22),
                        yaw,
                        pitch: swing,
                        roll: roll_angle,
                        origin: leg_origin,
                        tex_top: TextureId::CowSkin,
                        tex_bottom: TextureId::CowSkin,
                        tex_side: TextureId::CowSkin,
                        tex_front: TextureId::CowSkin,
                    };
                    self.draw_mob_box(&leg_box, hurt_tint, camera_pos, view_proj);
                }
            }
        }
    }

    fn draw_mob_box(&mut self, b: &MobBox, hurt_tint: f32, camera_pos: Vec3, view_proj: &Mat4) {
        let half = Vec3::new(b.size.x * 0.5, b.size.y * 0.5, b.size.z * 0.5);

        let local_corners = [
            b.center + Vec3::new(-half.x, -half.y, -half.z), // 0: LBB
            b.center + Vec3::new(half.x, -half.y, -half.z),  // 1: RBB
            b.center + Vec3::new(half.x, half.y, -half.z),   // 2: RTB
            b.center + Vec3::new(-half.x, half.y, -half.z),  // 3: LTB
            b.center + Vec3::new(-half.x, -half.y, half.z),  // 4: LBF
            b.center + Vec3::new(half.x, -half.y, half.z),   // 5: RBF
            b.center + Vec3::new(half.x, half.y, half.z),    // 6: RTF
            b.center + Vec3::new(-half.x, half.y, half.z),   // 7: LTF
        ];

        let transform_point = |p: Vec3| -> Vec3 {
            let p_pitch = if b.pitch != 0.0 {
                Vec3::new(
                    p.x,
                    p.y * b.pitch.cos() - p.z * b.pitch.sin(),
                    p.y * b.pitch.sin() + p.z * b.pitch.cos(),
                )
            } else {
                p
            };

            let p_roll = if b.roll != 0.0 {
                Vec3::new(
                    p_pitch.x * b.roll.cos() - p_pitch.y * b.roll.sin(),
                    p_pitch.x * b.roll.sin() + p_pitch.y * b.roll.cos(),
                    p_pitch.z,
                )
            } else {
                p_pitch
            };

            let p_yaw = Vec3::new(
                p_roll.x * b.yaw.cos() + p_roll.z * b.yaw.sin(),
                p_roll.y,
                -p_roll.x * b.yaw.sin() + p_roll.z * b.yaw.cos(),
            );

            b.origin + p_yaw
        };

        let world_pts: [Vec3; 8] = [
            transform_point(local_corners[0]),
            transform_point(local_corners[1]),
            transform_point(local_corners[2]),
            transform_point(local_corners[3]),
            transform_point(local_corners[4]),
            transform_point(local_corners[5]),
            transform_point(local_corners[6]),
            transform_point(local_corners[7]),
        ];

        let mut draw_quad = |i0: usize, i1: usize, i2: usize, i3: usize, tex: TextureId, shade: f32| {
            let p0 = world_pts[i0];
            let p1 = world_pts[i1];
            let p2 = world_pts[i2];
            let p3 = world_pts[i3];

            // World-space backface culling (eliminates back-facing quads and internal faces when camera is inside)
            let edge1 = p1 - p0;
            let edge2 = p3 - p0;
            let normal = edge1.cross(edge2);
            let center = (p0 + p1 + p2 + p3) * 0.25;
            let to_face = center - camera_pos;
            if to_face.dot(normal) >= 0.0 {
                return;
            }

            let t1 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p1, (1.0, 1.0), shade),
                v2: Vertex::new(p2, (1.0, 0.0), shade),
                texture: tex,
            };
            let t2 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p2, (1.0, 0.0), shade),
                v2: Vertex::new(p3, (0.0, 0.0), shade),
                texture: tex,
            };
            self.raster_triangle_tinted(&t1, hurt_tint, view_proj);
            self.raster_triangle_tinted(&t2, hurt_tint, view_proj);
        };

        draw_quad(7, 6, 2, 3, b.tex_top, 1.00);
        draw_quad(4, 0, 1, 5, b.tex_bottom, 0.50);
        draw_quad(4, 5, 6, 7, b.tex_front, 0.85);
        draw_quad(1, 0, 3, 2, b.tex_side, 0.85);
        draw_quad(0, 4, 7, 3, b.tex_side, 0.70);
        draw_quad(5, 1, 2, 6, b.tex_side, 0.70);
    }

    pub fn draw_procedural_tool_3d(
        &mut self,
        block: crate::world::block::BlockId,
        pos: Vec3,
        tool_right: Vec3,
        tool_up: Vec3,
        tool_forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        use crate::world::block::BlockId;
        let mat_tex = match block {
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword => TextureId::Planks,
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword => TextureId::Stone,
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword => TextureId::IronOre,
            _ => TextureId::WoodSide,
        };
        let handle_tex = TextureId::WoodSide;

        match block {
            BlockId::Stick => {
                // Slender wooden rod
                self.draw_cube_3d(
                    pos,
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.016 * scale, 0.26 * scale, 0.016 * scale],
                    handle_tex,
                    view_proj,
                );
            }
            BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword => {
                // Pommel
                self.draw_cube_3d(
                    pos - tool_up * (0.16 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.024 * scale, 0.018 * scale, 0.024 * scale],
                    handle_tex,
                    view_proj,
                );
                // Grip
                self.draw_cube_3d(
                    pos - tool_up * (0.08 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.016 * scale, 0.065 * scale, 0.016 * scale],
                    handle_tex,
                    view_proj,
                );
                // Crossguard
                self.draw_cube_3d(
                    pos,
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.085 * scale, 0.016 * scale, 0.026 * scale],
                    mat_tex,
                    view_proj,
                );
                // Blade
                self.draw_cube_3d(
                    pos + tool_up * (0.20 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.038 * scale, 0.180 * scale, 0.010 * scale],
                    mat_tex,
                    view_proj,
                );
                // Tip
                self.draw_cube_3d(
                    pos + tool_up * (0.41 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.020 * scale, 0.040 * scale, 0.008 * scale],
                    mat_tex,
                    view_proj,
                );
            }
            BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe => {
                // Handle
                self.draw_cube_3d(
                    pos,
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.015 * scale, 0.28 * scale, 0.015 * scale],
                    handle_tex,
                    view_proj,
                );
                // Head center mount
                self.draw_cube_3d(
                    pos + tool_up * (0.26 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.032 * scale, 0.032 * scale, 0.032 * scale],
                    mat_tex,
                    view_proj,
                );
                // Head crossbar
                self.draw_cube_3d(
                    pos + tool_up * (0.27 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.130 * scale, 0.024 * scale, 0.024 * scale],
                    mat_tex,
                    view_proj,
                );
                // Left & right pick tips
                self.draw_cube_3d(
                    pos + tool_up * (0.23 * scale) - tool_right * (0.12 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.018 * scale, 0.032 * scale, 0.018 * scale],
                    mat_tex,
                    view_proj,
                );
                self.draw_cube_3d(
                    pos + tool_up * (0.23 * scale) + tool_right * (0.12 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.018 * scale, 0.032 * scale, 0.018 * scale],
                    mat_tex,
                    view_proj,
                );
            }
            BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe => {
                // Handle
                self.draw_cube_3d(
                    pos,
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.015 * scale, 0.28 * scale, 0.015 * scale],
                    handle_tex,
                    view_proj,
                );
                // Axe Head Main
                self.draw_cube_3d(
                    pos + tool_up * (0.22 * scale) + tool_right * (0.045 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.048 * scale, 0.065 * scale, 0.022 * scale],
                    mat_tex,
                    view_proj,
                );
                // Blade edge
                self.draw_cube_3d(
                    pos + tool_up * (0.21 * scale) + tool_right * (0.088 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.014 * scale, 0.078 * scale, 0.012 * scale],
                    mat_tex,
                    view_proj,
                );
                // Poll (rear)
                self.draw_cube_3d(
                    pos + tool_up * (0.25 * scale) - tool_right * (0.024 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.020 * scale, 0.028 * scale, 0.020 * scale],
                    mat_tex,
                    view_proj,
                );
            }
            BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel => {
                // Handle
                self.draw_cube_3d(
                    pos,
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.015 * scale, 0.28 * scale, 0.015 * scale],
                    handle_tex,
                    view_proj,
                );
                // Grip Topper
                self.draw_cube_3d(
                    pos - tool_up * (0.27 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.042 * scale, 0.014 * scale, 0.018 * scale],
                    handle_tex,
                    view_proj,
                );
                // Spade Blade
                self.draw_cube_3d(
                    pos + tool_up * (0.33 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.065 * scale, 0.075 * scale, 0.012 * scale],
                    mat_tex,
                    view_proj,
                );
                // Spade Tip
                self.draw_cube_3d(
                    pos + tool_up * (0.42 * scale),
                    tool_right,
                    tool_up,
                    tool_forward,
                    [0.038 * scale, 0.018 * scale, 0.010 * scale],
                    mat_tex,
                    view_proj,
                );
            }
            _ => {}
        }
    }

    pub fn draw_hand_3d(
        &mut self,
        camera: &Camera,
        view_proj: &Mat4,
        block_id: Option<u8>,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
    ) {
        let forward = camera.forward();
        let right = camera.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        let movement = (speed / 8.5).clamp(0.0, 1.0);
        let swing = walk_phase.sin() * 0.10 * movement;
        let attack = (attack_animation * std::f32::consts::PI).sin();
        let swing = swing + attack * 0.16;
        let bob = (walk_phase * 2.0).cos() * 0.035 * movement - attack.abs() * 0.08;
        let hand_pos = camera.position
            + forward * 0.68
            + right * (0.53 + swing)
            + up * (-0.40 + bob);
        let arm_pos = hand_pos - right * 0.10 - up * 0.13;
        // Always draw arm
        self.draw_cube_3d(arm_pos, right, up, forward, [0.085, 0.085, 0.12], TextureId::Dirt, view_proj);
        // Draw held tool, torch or block
        if let Some(id) = block_id {
            let block = crate::world::block::BlockId::from_u8(id);
            if block == crate::world::block::BlockId::Torch {
                let swing_pitch = attack * 0.45;
                let torch_up = (up * (1.0 - swing_pitch * 0.3) + forward * (0.3 + swing_pitch)).normalized();
                let torch_right = right;
                let torch_forward = torch_up.cross(torch_right).normalized();
                let torch_origin = hand_pos + forward * 0.12 + up * 0.10;
                self.draw_torch_3d(torch_origin, torch_right, torch_up, torch_forward, 1.0, view_proj);
            } else if block.is_tool() {
                // Dynamic tool swing arc
                let swing_pitch = attack * 0.85;
                let tool_up = (up * (1.0 - swing_pitch * 0.5) + forward * (0.85 + swing_pitch) - right * 0.20).normalized();
                let tool_right = (right * 0.95 + up * 0.15).normalized();
                let tool_forward = tool_up.cross(tool_right).normalized();
                let tool_origin = hand_pos + forward * (0.10 + attack * 0.15) + up * (0.12 - attack * 0.12) - right * 0.02;

                self.draw_procedural_tool_3d(block, tool_origin, tool_right, tool_up, tool_forward, 1.0, view_proj);
            } else {
                let top_tex = block.texture_id(crate::world::block::Face::Top);
                let side_tex = block.texture_id(crate::world::block::Face::North);
                let bot_tex = block.texture_id(crate::world::block::Face::Bottom);
                self.draw_multitex_cube_3d(hand_pos, right, up, forward, [0.13, 0.13, 0.13], top_tex, bot_tex, side_tex, view_proj);
            }
        }
    }

    pub fn draw_torch_3d(
        &mut self,
        pos: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        // Torch stick body
        self.draw_cube_3d(
            pos,
            right,
            up,
            forward,
            [0.025 * scale, 0.20 * scale, 0.025 * scale],
            TextureId::Torch,
            view_proj,
        );
        // Torch flame head
        self.draw_cube_3d(
            pos + up * (0.16 * scale),
            right,
            up,
            forward,
            [0.038 * scale, 0.05 * scale, 0.038 * scale],
            TextureId::Torch,
            view_proj,
        );
    }

    pub fn draw_celestials(&mut self, camera: &Camera, view_proj: &Mat4, day_fraction: f32) {
        let angle = day_fraction * std::f32::consts::TAU;
        let sun_dir = Vec3::new(angle.sin(), angle.cos(), -0.2).normalized();
        let moon_dir = -sun_dir;

        // Draw Sun if above or near horizon
        if sun_dir.y > -0.25 {
            let sun_pos = camera.position + sun_dir * 120.0;
            self.draw_celestial_billboard(sun_pos, 22.0, [1.0, 0.95, 0.65], camera, view_proj);
        }

        // Draw Moon if above or near horizon
        if moon_dir.y > -0.25 {
            let moon_pos = camera.position + moon_dir * 120.0;
            self.draw_celestial_billboard(moon_pos, 16.0, [0.92, 0.95, 1.0], camera, view_proj);
        }

        // Draw Stars at night / twilight
        let daylight = self.current_daylight;
        if daylight < 0.65 {
            let star_alpha = ((0.65 - daylight) / 0.50).clamp(0.0, 1.0);
            self.draw_stars(camera, view_proj, star_alpha);
        }
    }

    fn draw_celestial_billboard(
        &mut self,
        pos: Vec3,
        size: f32,
        color: [f32; 3],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let right = camera.right_flat();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let h = size * 0.5;

        let corners = [
            pos - right * h - up * h,
            pos + right * h - up * h,
            pos + right * h + up * h,
            pos - right * h + up * h,
        ];

        let project = |p: Vec3| -> Option<(f32, f32, f32)> {
            let c = view_proj.transform_point(p);
            if c[3] <= 1e-4 { return None; }
            let inv_w = 1.0 / c[3];
            let sx = (c[0] * inv_w * 0.5 + 0.5) * self.width as f32;
            let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * self.height as f32;
            Some((sx, sy, c[3]))
        };

        let pts: Vec<_> = corners.iter().filter_map(|&p| project(p)).collect();
        if pts.len() < 4 { return; }

        let depth_val = 5000.0f32;
        let packed = pack_color(color[0], color[1], color[2]);

        for &tri_idx in &[(0usize, 1usize, 2usize), (0usize, 2usize, 3usize)] {
            let (ai, bi, ci) = tri_idx;
            let (ax, ay, _) = pts[ai];
            let (bx, by, _) = pts[bi];
            let (cx, cy, _) = pts[ci];
            let area = edge(ax, ay, bx, by, cx, cy);
            if area <= 0.0 { continue; }
            let min_x = ax.min(bx).min(cx).floor().max(0.0) as i32;
            let max_x = ax.max(bx).max(cx).ceil().min(self.width as f32 - 1.0) as i32;
            let min_y = ay.min(by).min(cy).floor().max(0.0) as i32;
            let max_y = ay.max(by).max(cy).ceil().min(self.height as f32 - 1.0) as i32;
            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let fx = px as f32 + 0.5;
                    let fy = py as f32 + 0.5;
                    let w0 = edge(bx, by, cx, cy, fx, fy);
                    let w1 = edge(cx, cy, ax, ay, fx, fy);
                    let w2 = edge(ax, ay, bx, by, fx, fy);
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let idx = py as usize * self.width + px as usize;
                        if depth_val < self.depth_buffer[idx] {
                            self.color_buffer[idx] = packed;
                            self.depth_buffer[idx] = depth_val;
                        }
                    }
                }
            }
        }
    }

    fn draw_stars(&mut self, camera: &Camera, view_proj: &Mat4, star_alpha: f32) {
        if star_alpha <= 0.01 { return; }
        const STAR_COUNT: usize = 64;
        let width = self.width;
        let height = self.height;

        for i in 0..STAR_COUNT {
            let fi = i as f32;
            let phi = fi * 2.3999632;
            let y = 0.15 + (fi / STAR_COUNT as f32) * 0.85;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let x = phi.cos() * r;
            let z = phi.sin() * r;

            let star_dir = Vec3::new(x, y, z).normalized();
            let star_world = camera.position + star_dir * 100.0;
            let c = view_proj.transform_point(star_world);
            if c[3] <= 0.1 { continue; }
            let inv_w = 1.0 / c[3];
            let sx = ((c[0] * inv_w * 0.5 + 0.5) * width as f32) as i32;
            let sy = ((1.0 - (c[1] * inv_w * 0.5 + 0.5)) * height as f32) as i32;

            if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                let idx = sy as usize * width + sx as usize;
                if 5000.0 < self.depth_buffer[idx] {
                    let twinkle = 0.7 + 0.3 * ((fi * 1.7 + self.total_time * 2.5).sin() * 0.5 + 0.5);
                    let intensity = star_alpha * twinkle;
                    let star_col = pack_color(intensity * 0.95, intensity * 0.95, intensity);
                    self.color_buffer[idx] = star_col;
                }
            }
        }
    }

    pub fn draw_item_3d(
        &mut self,
        block: crate::world::block::BlockId,
        pos: Vec3,
        yaw: f32,
        view_proj: &Mat4,
    ) {
        if block == crate::world::block::BlockId::Torch {
            let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
            let up = Vec3::new(0.0, 1.0, 0.0);
            let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            self.draw_torch_3d(pos, right, up, forward, 0.85, view_proj);
        } else if block.is_tool() {
            // Spinning tilted 3D tool
            let tilt_angle = 0.38f32; // ~22 degrees
            let base_up = Vec3::new(0.0, 1.0, 0.0);
            let base_right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
            let base_forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

            let tool_up = (base_up * tilt_angle.cos() + base_right * tilt_angle.sin()).normalized();
            let tool_right = (base_right * tilt_angle.cos() - base_up * tilt_angle.sin()).normalized();
            let tool_forward = base_forward;

            self.draw_procedural_tool_3d(block, pos, tool_right, tool_up, tool_forward, 0.85, view_proj);
        } else {
            use crate::world::block::Face;
            let size = 0.28;
            let half = size * 0.5;
            let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
            let up = Vec3::new(0.0, 1.0, 0.0);
            let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

            let corner = |x: f32, y: f32, z: f32| {
                pos + right * (x * half) + up * (y * half) + forward * (z * half)
            };

            let face_data = [
                ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::Top), 1.00),
                ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], block.texture_id(Face::Bottom), 0.55),
                ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], block.texture_id(Face::South), 0.85),
                ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], block.texture_id(Face::North), 0.85),
                ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::West), 0.70),
                ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], block.texture_id(Face::East), 0.70),
            ];

            for (corners, tex, ao) in face_data {
                let t1 = Triangle {
                    v0: Vertex::new(corners[0], (0.0, 1.0), ao),
                    v1: Vertex::new(corners[1], (1.0, 1.0), ao),
                    v2: Vertex::new(corners[2], (1.0, 0.0), ao),
                    texture: tex,
                };
                let t2 = Triangle {
                    v0: Vertex::new(corners[0], (0.0, 1.0), ao),
                    v1: Vertex::new(corners[2], (1.0, 0.0), ao),
                    v2: Vertex::new(corners[3], (0.0, 0.0), ao),
                    texture: tex,
                };
                self.raster_triangle(&t1, view_proj);
                self.raster_triangle(&t2, view_proj);
            }
        }
    }

    pub fn draw_item_shadow(&mut self, pos: Vec3, ground_y: f32, view_proj: &Mat4) {
        let s_rad = 0.22f32;
        let sx0 = pos.x - s_rad;
        let sx1 = pos.x + s_rad;
        let sz0 = pos.z - s_rad;
        let sz1 = pos.z + s_rad;
        let s_c0 = Vec3::new(sx0, ground_y, sz1);
        let s_c1 = Vec3::new(sx1, ground_y, sz1);
        let s_c2 = Vec3::new(sx1, ground_y, sz0);
        let s_c3 = Vec3::new(sx0, ground_y, sz0);

        let t1 = Triangle {
            v0: Vertex::new(s_c0, (0.0, 1.0), 1.0),
            v1: Vertex::new(s_c1, (1.0, 1.0), 1.0),
            v2: Vertex::new(s_c2, (1.0, 0.0), 1.0),
            texture: TextureId::EntityShadow,
        };
        let t2 = Triangle {
            v0: Vertex::new(s_c0, (0.0, 1.0), 1.0),
            v1: Vertex::new(s_c2, (1.0, 0.0), 1.0),
            v2: Vertex::new(s_c3, (0.0, 0.0), 1.0),
            texture: TextureId::EntityShadow,
        };
        self.raster_triangle(&t1, view_proj);
        self.raster_triangle(&t2, view_proj);
    }

    pub fn draw_block_break_cracks(
        &mut self,
        target: crate::core::math::IVec3,
        progress: f32,
        view_proj: &Mat4,
    ) {
        if progress <= 0.0 {
            return;
        }
        let stage = ((progress * 10.0).floor() as usize).min(9);
        let crack_tex = TextureId::destroy_stage(stage);

        let eps = 0.003;
        let x0 = target.x as f32 - eps;
        let x1 = target.x as f32 + 1.0 + eps;
        let y0 = target.y as f32 - eps;
        let y1 = target.y as f32 + 1.0 + eps;
        let z0 = target.z as f32 - eps;
        let z1 = target.z as f32 + 1.0 + eps;

        let faces = [
            // Top (+Y)
            [Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0)],
            // Bottom (-Y)
            [Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1)],
            // Front (+Z)
            [Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1)],
            // Back (-Z)
            [Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0)],
            // Left (-X)
            [Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0)],
            // Right (+X)
            [Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1)],
        ];

        for corners in faces {
            let t1 = Triangle {
                v0: Vertex::new(corners[0], (0.0, 1.0), 1.0),
                v1: Vertex::new(corners[1], (1.0, 1.0), 1.0),
                v2: Vertex::new(corners[2], (1.0, 0.0), 1.0),
                texture: crack_tex,
            };
            let t2 = Triangle {
                v0: Vertex::new(corners[0], (0.0, 1.0), 1.0),
                v1: Vertex::new(corners[2], (1.0, 0.0), 1.0),
                v2: Vertex::new(corners[3], (0.0, 0.0), 1.0),
                texture: crack_tex,
            };
            self.raster_triangle(&t1, view_proj);
            self.raster_triangle(&t2, view_proj);
        }
    }

    fn draw_cube_3d(
        &mut self,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        half: [f32; 3],
        texture: TextureId,
        view_proj: &Mat4,
    ) {
        let corner = |x: f32, y: f32, z: f32| {
            center + right * (x * half[0]) + up * (y * half[1]) + forward * (z * half[2])
        };
        let faces = [
            ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)]),
            ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)]),
            ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)]),
            ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)]),
            ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)]),
            ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)]),
        ];
        for corners in faces {
            let vertices = [
                Vertex::new(corners[0], (0.0, 1.0), 1.0),
                Vertex::new(corners[1], (1.0, 1.0), 1.0),
                Vertex::new(corners[2], (1.0, 0.0), 1.0),
                Vertex::new(corners[3], (0.0, 0.0), 1.0),
            ];
            let first = Triangle { v0: vertices[0], v1: vertices[1], v2: vertices[2], texture };
            let second = Triangle { v0: vertices[0], v1: vertices[2], v2: vertices[3], texture };
            self.raster_triangle(&first, view_proj);
            self.raster_triangle(&second, view_proj);
            self.raster_triangle(&Triangle { v0: first.v2, v1: first.v1, v2: first.v0, texture }, view_proj);
            self.raster_triangle(&Triangle { v0: second.v2, v1: second.v2, v2: second.v0, texture }, view_proj);
        }
    }

    fn draw_multitex_cube_3d(
        &mut self,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        half: [f32; 3],
        top_tex: TextureId,
        bot_tex: TextureId,
        side_tex: TextureId,
        view_proj: &Mat4,
    ) {
        let corner = |x: f32, y: f32, z: f32| {
            center + right * (x * half[0]) + up * (y * half[1]) + forward * (z * half[2])
        };
        let faces = [
            ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], side_tex, 0.85),
            ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], side_tex, 0.85),
            ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], side_tex, 0.70),
            ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], side_tex, 0.70),
            ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], top_tex, 1.00),
            ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], bot_tex, 0.55),
        ];
        for (corners, texture, ao) in faces {
            let vertices = [
                Vertex::new(corners[0], (0.0, 1.0), ao),
                Vertex::new(corners[1], (1.0, 1.0), ao),
                Vertex::new(corners[2], (1.0, 0.0), ao),
                Vertex::new(corners[3], (0.0, 0.0), ao),
            ];
            let first = Triangle { v0: vertices[0], v1: vertices[1], v2: vertices[2], texture };
            let second = Triangle { v0: vertices[0], v1: vertices[2], v2: vertices[3], texture };
            self.raster_triangle(&first, view_proj);
            self.raster_triangle(&second, view_proj);
            self.raster_triangle(&Triangle { v0: first.v2, v1: first.v1, v2: first.v0, texture }, view_proj);
            self.raster_triangle(&Triangle { v0: second.v2, v1: second.v2, v2: second.v0, texture }, view_proj);
        }
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

struct MobBox {
    center: Vec3,
    size: Vec3,
    yaw: f32,
    pitch: f32,
    roll: f32,
    origin: Vec3,
    tex_top: TextureId,
    tex_bottom: TextureId,
    tex_side: TextureId,
    tex_front: TextureId,
}

#[inline]
fn cloud_column_visible(
    min_x: f32, max_x: f32,
    min_y: f32, max_y: f32,
    min_z: f32, max_z: f32,
    view_proj: &Mat4,
) -> bool {
    let corners = [
        Vec3::new(min_x, min_y, min_z),
        Vec3::new(max_x, min_y, min_z),
        Vec3::new(min_x, max_y, min_z),
        Vec3::new(max_x, max_y, min_z),
        Vec3::new(min_x, min_y, max_z),
        Vec3::new(max_x, min_y, max_z),
        Vec3::new(min_x, max_y, max_z),
        Vec3::new(max_x, max_y, max_z),
    ];

    let mut out_x_pos = 0; let mut out_x_neg = 0;
    let mut out_y_pos = 0; let mut out_y_neg = 0;
    let mut out_z_pos = 0; let mut out_z_neg = 0;

    for &corner in &corners {
        let clip = view_proj.transform_point(corner);
        if clip[0] > clip[3] { out_x_pos += 1; }
        if clip[0] < -clip[3] { out_x_neg += 1; }
        if clip[1] > clip[3] { out_y_pos += 1; }
        if clip[1] < -clip[3] { out_y_neg += 1; }
        if clip[2] > clip[3] { out_z_pos += 1; }
        if clip[2] < -clip[3] { out_z_neg += 1; }
    }

    !(out_x_pos == 8 || out_x_neg == 8 || out_y_pos == 8 || out_y_neg == 8 || out_z_pos == 8 || out_z_neg == 8)
}

fn project_to_screen_ao(v: &ClipVertAO, width: usize, height: usize) -> ScreenVertAO {
    let inv_w = 1.0 / v.clip[3];
    let ndc_x = v.clip[0] * inv_w;
    let ndc_y = v.clip[1] * inv_w;
    let screen_x = (ndc_x * 0.5 + 0.5) * width as f32;
    let screen_y = (1.0 - (ndc_y * 0.5 + 0.5)) * height as f32;
    ScreenVertAO {
        x: screen_x,
        y: screen_y,
        inv_w,
        uv_over_w: (v.uv.0 * inv_w, v.uv.1 * inv_w),
        ao_over_w: v.ao * inv_w,
        sky_light_over_w: v.sky_light * inv_w,
        block_light_over_w: v.block_light * inv_w,
    }
}

fn raster_triangle_tile(
    tri: &Triangle,
    hurt_tint: f32,
    view_proj: &Mat4,
    width: usize,
    height: usize,
    tile_y_start: usize,
    tile_y_end: usize,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    textures: &TextureAtlas,
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
) {
    let c0 = to_clip_ao(tri.v0, view_proj);
    let c1 = to_clip_ao(tri.v1, view_proj);
    let c2 = to_clip_ao(tri.v2, view_proj);

    let clipped = clip_near_ao(&[c0, c1, c2]);
    if clipped.len() < 3 {
        return;
    }

    for i in 1..clipped.len() - 1 {
        let sa = project_to_screen_ao(&clipped[0], width, height);
        let sb = project_to_screen_ao(&clipped[i], width, height);
        let sc = project_to_screen_ao(&clipped[i + 1], width, height);

        raster_screen_triangle_ao_tile(
            sa, sb, sc, tri.texture, hurt_tint, width, tile_y_start, tile_y_end, color_buffer, depth_buffer, textures, fog_dist, daylight, fog_color,
        );
    }
}

fn raster_screen_triangle_ao_tile(
    a: ScreenVertAO,
    b: ScreenVertAO,
    c: ScreenVertAO,
    texture: TextureId,
    hurt_tint: f32,
    width: usize,
    tile_y_start: usize,
    tile_y_end: usize,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    textures: &TextureAtlas,
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
) {
    let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
    if area <= 0.0 {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(width as f32 - 1.0) as i32;
    let min_y = (a.y.min(b.y).min(c.y).floor() as i32).max(tile_y_start as i32);
    let max_y = (a.y.max(b.y).max(c.y).ceil() as i32).min(tile_y_end as i32 - 1);

    if min_x > max_x || min_y > max_y {
        return;
    }

    let inv_area = 1.0 / area;
    let tex = textures.get(texture);
    let is_crack = texture.is_destroy_stage();
    let transparent = matches!(texture, TextureId::Water | TextureId::Glass | TextureId::EntityShadow) || is_crack;

    let dx0 = c.x - b.x;
    let dy0 = c.y - b.y;
    let dx1 = a.x - c.x;
    let dy1 = a.y - c.y;
    let dx2 = b.x - a.x;
    let dy2 = b.y - a.y;

    let fx0 = min_x as f32 + 0.5;
    let fy0 = min_y as f32 + 0.5;

    let mut w0_row = edge(b.x, b.y, c.x, c.y, fx0, fy0);
    let mut w1_row = edge(c.x, c.y, a.x, a.y, fx0, fy0);
    let mut w2_row = edge(a.x, a.y, b.x, b.y, fx0, fy0);

    for py in min_y..=max_y {
        let mut w0 = w0_row;
        let mut w1 = w1_row;
        let mut w2 = w2_row;
        let row_offset = (py as usize - tile_y_start) * width;

        let mut px = min_x;
        while px + 3 <= max_x {
            raster_scanline_simd_4x(
                px,
                row_offset,
                w0,
                w1,
                w2,
                dy0,
                dy1,
                dy2,
                inv_area,
                &a,
                &b,
                &c,
                transparent,
                is_crack,
                hurt_tint,
                tex,
                color_buffer,
                depth_buffer,
                fog_dist,
                daylight,
                fog_color,
            );

            px += 4;
            w0 += dy0 * 4.0;
            w1 += dy1 * 4.0;
            w2 += dy2 * 4.0;
        }

        while px <= max_x {
            raster_scanline_pixel(
                px,
                row_offset,
                w0,
                w1,
                w2,
                inv_area,
                &a,
                &b,
                &c,
                transparent,
                is_crack,
                hurt_tint,
                tex,
                color_buffer,
                depth_buffer,
                fog_dist,
                daylight,
                fog_color,
            );

            px += 1;
            w0 += dy0;
            w1 += dy1;
            w2 += dy2;
        }

        w0_row -= dx0;
        w1_row -= dx1;
        w2_row -= dx2;
    }
}

#[inline(always)]
fn raster_scanline_simd_4x(
    px: i32,
    row_offset: usize,
    w0_start: f32,
    w1_start: f32,
    w2_start: f32,
    dy0: f32,
    dy1: f32,
    dy2: f32,
    inv_area: f32,
    a: &ScreenVertAO,
    b: &ScreenVertAO,
    c: &ScreenVertAO,
    transparent: bool,
    is_crack: bool,
    hurt_tint: f32,
    tex: &crate::graphics::atlas::Texture,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
) {
    let w0 = [
        w0_start,
        w0_start + dy0,
        w0_start + dy0 * 2.0,
        w0_start + dy0 * 3.0,
    ];
    let w1 = [
        w1_start,
        w1_start + dy1,
        w1_start + dy1 * 2.0,
        w1_start + dy1 * 3.0,
    ];
    let w2 = [
        w2_start,
        w2_start + dy2,
        w2_start + dy2 * 2.0,
        w2_start + dy2 * 3.0,
    ];

    let mask = [
        w0[0] >= 0.0 && w1[0] >= 0.0 && w2[0] >= 0.0,
        w0[1] >= 0.0 && w1[1] >= 0.0 && w2[1] >= 0.0,
        w0[2] >= 0.0 && w1[2] >= 0.0 && w2[2] >= 0.0,
        w0[3] >= 0.0 && w1[3] >= 0.0 && w2[3] >= 0.0,
    ];

    if !mask[0] && !mask[1] && !mask[2] && !mask[3] {
        return;
    }

    for i in 0..4 {
        if mask[i] {
            let l0 = w0[i] * inv_area;
            let l1 = w1[i] * inv_area;
            let l2 = w2[i] * inv_area;

            let inv_w = l0 * a.inv_w + l1 * b.inv_w + l2 * c.inv_w;
            if inv_w > 0.0 {
                let depth = 1.0 / inv_w;
                let idx = row_offset + (px as usize + i);

                if depth < depth_buffer[idx] {
                    let u = (l0 * a.uv_over_w.0 + l1 * b.uv_over_w.0 + l2 * c.uv_over_w.0) * depth;
                    let v = (l0 * a.uv_over_w.1 + l1 * b.uv_over_w.1 + l2 * c.uv_over_w.1) * depth;

                    let texel = tex.sample(u, v);
                    if texel[0] < 0.0 {
                        continue;
                    }

                    if !transparent {
                        depth_buffer[idx] = depth;
                    }

                    let ao = (l0 * a.ao_over_w + l1 * b.ao_over_w + l2 * c.ao_over_w) * depth;
                    let sky = (l0 * a.sky_light_over_w + l1 * b.sky_light_over_w + l2 * c.sky_light_over_w) * depth;
                    let block = (l0 * a.block_light_over_w + l1 * b.block_light_over_w + l2 * c.block_light_over_w) * depth;

                    let sky_lit = sky * daylight;
                    let lr = (sky_lit + block * 1.35).clamp(0.06, 1.1) * ao;
                    let lg = (sky_lit + block * 1.15).clamp(0.06, 1.0) * ao;
                    let lb = (sky_lit * 0.95 + block * 0.80).clamp(0.06, 1.0) * ao;

                    let mut r = texel[0] * lr;
                    let mut g = texel[1] * lg;
                    let mut bl = texel[2] * lb;

                    if hurt_tint > 0.0 {
                        r = r * (1.0 - hurt_tint) + 0.95 * hurt_tint;
                        g = g * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                        bl = bl * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                    }

                    let fog_t = if fog_dist > 0.0 {
                        ((depth - fog_dist * 0.45) / (fog_dist * 0.55)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let fr = r * (1.0 - fog_t) + fog_color[0] * fog_t;
                    let fg = g * (1.0 - fog_t) + fog_color[1] * fog_t;
                    let fb = bl * (1.0 - fog_t) + fog_color[2] * fog_t;

                    if is_crack {
                        let (br, bg, bb) = unpack_color(color_buffer[idx]);
                        let alpha = 0.85;
                        color_buffer[idx] = pack_color(
                            fr * alpha + br * (1.0 - alpha),
                            fg * alpha + bg * (1.0 - alpha),
                            fb * alpha + bb * (1.0 - alpha),
                        );
                    } else if transparent {
                        let (br, bg, bb) = unpack_color(color_buffer[idx]);
                        let alpha = 0.48;
                        color_buffer[idx] = pack_color(
                            fr * alpha + br * (1.0 - alpha),
                            fg * alpha + bg * (1.0 - alpha),
                            fb * alpha + bb * (1.0 - alpha),
                        );
                    } else {
                        color_buffer[idx] = pack_color(fr, fg, fb);
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn raster_scanline_pixel(
    px: i32,
    row_offset: usize,
    w0: f32,
    w1: f32,
    w2: f32,
    inv_area: f32,
    a: &ScreenVertAO,
    b: &ScreenVertAO,
    c: &ScreenVertAO,
    transparent: bool,
    is_crack: bool,
    hurt_tint: f32,
    tex: &crate::graphics::atlas::Texture,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
) {
    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
        let l0 = w0 * inv_area;
        let l1 = w1 * inv_area;
        let l2 = w2 * inv_area;

        let inv_w = l0 * a.inv_w + l1 * b.inv_w + l2 * c.inv_w;
        if inv_w > 0.0 {
            let depth = 1.0 / inv_w;
            let idx = row_offset + (px as usize);

            if depth < depth_buffer[idx] {
                let u = (l0 * a.uv_over_w.0 + l1 * b.uv_over_w.0 + l2 * c.uv_over_w.0) * depth;
                let v = (l0 * a.uv_over_w.1 + l1 * b.uv_over_w.1 + l2 * c.uv_over_w.1) * depth;

                let texel = tex.sample(u, v);
                if texel[0] < 0.0 {
                    return;
                }

                if !transparent {
                    depth_buffer[idx] = depth;
                }

                let ao = (l0 * a.ao_over_w + l1 * b.ao_over_w + l2 * c.ao_over_w) * depth;
                let sky = (l0 * a.sky_light_over_w + l1 * b.sky_light_over_w + l2 * c.sky_light_over_w) * depth;
                let block = (l0 * a.block_light_over_w + l1 * b.block_light_over_w + l2 * c.block_light_over_w) * depth;

                let sky_lit = sky * daylight;
                let lr = (sky_lit + block * 1.35).clamp(0.06, 1.1) * ao;
                let lg = (sky_lit + block * 1.15).clamp(0.06, 1.0) * ao;
                let lb = (sky_lit * 0.95 + block * 0.80).clamp(0.06, 1.0) * ao;

                let mut r = texel[0] * lr;
                let mut g = texel[1] * lg;
                let mut bl = texel[2] * lb;

                if hurt_tint > 0.0 {
                    r = r * (1.0 - hurt_tint) + 0.95 * hurt_tint;
                    g = g * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                    bl = bl * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                }

                let fog_t = if fog_dist > 0.0 {
                    ((depth - fog_dist * 0.45) / (fog_dist * 0.55)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let fr = r * (1.0 - fog_t) + fog_color[0] * fog_t;
                let fg = g * (1.0 - fog_t) + fog_color[1] * fog_t;
                let fb = bl * (1.0 - fog_t) + fog_color[2] * fog_t;

                if is_crack {
                    let (br, bg, bb) = unpack_color(color_buffer[idx]);
                    let alpha = 0.85;
                    color_buffer[idx] = pack_color(
                        fr * alpha + br * (1.0 - alpha),
                        fg * alpha + bg * (1.0 - alpha),
                        fb * alpha + bb * (1.0 - alpha),
                    );
                } else if transparent {
                    let (br, bg, bb) = unpack_color(color_buffer[idx]);
                    let alpha = 0.48;
                    color_buffer[idx] = pack_color(
                        fr * alpha + br * (1.0 - alpha),
                        fg * alpha + bg * (1.0 - alpha),
                        fb * alpha + bb * (1.0 - alpha),
                    );
                } else {
                    color_buffer[idx] = pack_color(fr, fg, fb);
                }
            }
        }
    }
}

fn to_clip_ao(v: Vertex, view_proj: &Mat4) -> ClipVertAO {
    let c = view_proj.transform_point(v.pos);
    ClipVertAO {
        clip: c,
        uv: v.uv,
        ao: v.ao,
        sky_light: v.sky_light,
        block_light: v.block_light,
    }
}

fn clip_near_ao(verts: &[ClipVertAO; 3]) -> Vec<ClipVertAO> {
    const EPS: f32 = 0.05;
    let inside = |v: &ClipVertAO| v.clip[3] > EPS;

    let mut output: Vec<ClipVertAO> = Vec::with_capacity(4);
    for i in 0..3 {
        let cur = verts[i];
        let next = verts[(i + 1) % 3];
        let cur_in = inside(&cur);
        let next_in = inside(&next);

        if cur_in {
            output.push(cur);
        }
        if cur_in != next_in {
            let t = (EPS - cur.clip[3]) / (next.clip[3] - cur.clip[3]);
            let clip = [
                lerp(cur.clip[0], next.clip[0], t),
                lerp(cur.clip[1], next.clip[1], t),
                lerp(cur.clip[2], next.clip[2], t),
                lerp(cur.clip[3], next.clip[3], t),
            ];
            let uv = (
                lerp(cur.uv.0, next.uv.0, t),
                lerp(cur.uv.1, next.uv.1, t),
            );
            let ao = lerp(cur.ao, next.ao, t);
            let sky_light = lerp(cur.sky_light, next.sky_light, t);
            let block_light = lerp(cur.block_light, next.block_light, t);
            output.push(ClipVertAO { clip, uv, ao, sky_light, block_light });
        }
    }
    output
}

#[inline(always)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

#[inline(always)]
fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

#[inline(always)]
fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    (r, g, b)
}
