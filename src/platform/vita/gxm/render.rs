//! Main per-frame `GxmRenderer::render_frame` orchestrator, async triple-buffered swap, and 2D present.
//! Executes clear -> 4-pass terrain/sky -> dynamic entities -> GPU post-FX/biome tint -> hand -> HUD.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::app::{CameraPerspective, FishingBobber};
use crate::core::math::{IVec3, Mat4, Vec3};
use crate::features::entities::ArrowEntity;
use crate::features::player::ItemStack;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::Mob;
use crate::gameplay::particles::Particle;
use crate::graphics::camera::Camera;
use crate::net::RemotePlayer;
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Z};

use super::ffi::*;
use super::mem::DisplayData;
use super::render_dynamic::WorldEntitiesInput;
use super::renderer::GxmRenderer;
use super::types::{now_us, FrameRenderStats, FrameTimingsUs};

/// Borrowed per-frame inputs needed by `GxmRenderer::render_frame`.
pub struct GxmFrameParams<'a> {
    pub camera: &'a Camera,
    pub player_feet_pos: Vec3,
    pub player_yaw: f32,
    pub player_pitch: f32,
    pub player_crouching: bool,
    pub player_armor: &'a [Option<ItemStack>; 4],
    pub camera_perspective: CameraPerspective,
    pub is_menu_backdrop: bool,
    pub hud_pixels: Option<&'a [u32]>,
    pub hud_width: usize,
    pub hud_height: usize,
    pub break_target: Option<IVec3>,
    pub break_progress: f32,
    pub items: &'a [ItemEntity],
    pub mobs: &'a [Mob],
    pub particles: &'a [Particle],
    pub arrows: &'a [ArrowEntity],
    pub fishing_bobber: Option<&'a FishingBobber>,
    pub remote_players: Option<&'a HashMap<u32, RemotePlayer>>,
    pub hand_block: Option<u8>,
    pub offhand_block: Option<u8>,
    pub is_blocking: bool,
    pub walk_phase: f32,
    pub speed: f32,
    pub attack_animation: f32,
    pub render_distance: i32,
    pub fog_enabled: bool,
    pub clouds_enabled: bool,
    pub chunk_anim: bool,
    pub brightness: f32,
    pub total_time: f32,
    pub biome_tint: [f32; 3],
    pub damage_tilt_timer: f32,
    pub is_underwater: bool,
    pub is_in_lava: bool,
    pub is_burning: bool,
    pub low_health: bool,
}

impl GxmRenderer {
    /// Computes perspective projection matrix mapped to the GXM depth range `[0, 1]`.
    pub fn gxm_perspective_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let mut m = [0.0f32; 16];
        m[0] = f / aspect;
        m[5] = f;
        m[10] = far / (near - far);
        m[11] = -1.0;
        m[14] = (far * near) / (near - far);
        Mat4 { m }
    }

    /// Renders a 3D frame on PS Vita GPU, applies GPU post-FX/HUD, and queues an async triple-buffered swap.
    pub fn render_frame(&mut self, world: &mut World, params: &GxmFrameParams<'_>) -> u32 {
        let buf_idx = self.back_buf;
        let (sky_top, sky_bottom, fog_col) = world.sky_colors();
        let daylight = world.sunlight_factor().clamp(0.05, 1.0);
        let sun_dir = world.sun_direction();

        let aspect = VITA_WIDTH as f32 / VITA_HEIGHT as f32;
        let proj = Self::gxm_perspective_matrix(params.camera.fov_y, aspect, 0.05, 400.0);
        let view = params.camera.view_matrix();
        let mvp = proj.mul(&view);

        let cam_chunk_x = (params.camera.position.x / CHUNK_SIZE_X as f32).floor() as i32;
        let cam_chunk_z = (params.camera.position.z / CHUNK_SIZE_Z as f32).floor() as i32;
        let render_dist = params.render_distance.clamp(2, 5);
        let max_fog = if params.fog_enabled { (render_dist as f32 * 16.0).max(32.0) } else { 0.0 };

        unsafe {
            let mut timings = FrameTimingsUs::default();
            let t_begin = now_us();
            sceGxmBeginScene(
                self.context,
                0,
                self.render_target,
                core::ptr::null(),
                core::ptr::null_mut(),
                self.display_sync[buf_idx],
                &self.color_surfaces[buf_idx],
                &self.depth_surface,
            );
            timings.gpu_sync_us += now_us().saturating_sub(t_begin);

            // 1. Hardware clear (write Z = 1.0 across screen and sky_top base color)
            let t_clear = now_us();
            sceGxmSetDefaultRegionClipAndViewport(self.context, VITA_WIDTH - 1, VITA_HEIGHT - 1);
            sceGxmSetViewportEnable(self.context, SCE_GXM_VIEWPORT_ENABLED);
            sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 1.0, 0.0);
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);
            sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
            sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
            sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetFrontPolygonMode(self.context, SCE_GXM_POLYGON_MODE_TRIANGLE_FILL);
            sceGxmSetBackPolygonMode(self.context, SCE_GXM_POLYGON_MODE_TRIANGLE_FILL);

            sceGxmSetVertexProgram(self.context, self.shaders.clear_vert_prog);
            sceGxmSetFragmentProgram(self.context, self.shaders.clear_frag_prog);
            let mut clear_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
            sceGxmReserveFragmentDefaultUniformBuffer(self.context, &mut clear_uniform_buf);
            if !self.shaders.param_clear_color.is_null() && !clear_uniform_buf.is_null() {
                let sky_rgba: [f32; 4] = [sky_top[0], sky_top[1], sky_top[2], 1.0];
                sceGxmSetUniformDataF(clear_uniform_buf, self.shaders.param_clear_color, 0, 4, sky_rgba.as_ptr());
            }
            sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
            let clear_rc = sceGxmDraw(
                self.context,
                SCE_GXM_PRIMITIVE_TRIANGLES,
                SCE_GXM_INDEX_FORMAT_U16,
                self.clear_ib_mem.ptr,
                3,
            );
            timings.terrain_us += now_us().saturating_sub(t_clear);

            let mut stats = FrameRenderStats::default();
            if clear_rc == SCE_OK {
                stats.draw_calls += 1;
            }

            // 2. Restore world depth range [0.0, 1.0] & draw 4-pass terrain + Early-Z sky
            sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 0.0, 1.0);
            let is_occluded = self.render_terrain(
                buf_idx,
                params.camera,
                world,
                &mvp,
                sky_top,
                sky_bottom,
                fog_col,
                daylight,
                sun_dir,
                cam_chunk_x,
                cam_chunk_z,
                render_dist,
                params.fog_enabled,
                params.chunk_anim,
                params.total_time,
                &mut stats,
                &mut timings,
            );

            // 3. Dynamic 3D entities + environmental post-FX (skipped during static menu backdrops)
            let (mut v_cur, mut i_cur) = (0usize, 0usize);
            if !params.is_menu_backdrop {
                let swing = params.break_progress.max(params.attack_animation);
                let dyn_input = WorldEntitiesInput {
                    camera: params.camera,
                    world,
                    mvp: &mvp,
                    player_feet_pos: params.player_feet_pos,
                    player_yaw: params.player_yaw,
                    player_pitch: params.player_pitch,
                    walk_phase: params.walk_phase,
                    swing_progress: swing,
                    player_crouching: params.player_crouching,
                    damage_tilt_timer: params.damage_tilt_timer,
                    held_block: params.hand_block,
                    player_armor: params.player_armor,
                    camera_perspective: params.camera_perspective,
                    break_target: params.break_target,
                    break_progress: params.break_progress,
                    items: params.items,
                    mobs: params.mobs,
                    particles: params.particles,
                    arrows: params.arrows,
                    fishing_bobber: params.fishing_bobber,
                    remote_players: params.remote_players,
                    clouds_enabled: params.clouds_enabled,
                    fog_enabled: params.fog_enabled,
                    max_fog,
                    fog_col,
                    daylight,
                    brightness: params.brightness,
                    total_time: params.total_time,
                    is_occluded,
                };
                (v_cur, i_cur) = self.render_world_entities(buf_idx, &dyn_input, &mut stats, &mut timings);
                let t_env = now_us();
                self.render_env_postfx(
                    params.is_underwater,
                    params.is_in_lava,
                    params.is_burning,
                    params.total_time,
                    &mut stats,
                );
                timings.hud_and_postfx_us += now_us().saturating_sub(t_env);
            }

            // 4. Atmospheric biome tint multiplier (applies in Playing and 3D menu backdrops)
            let t_tint = now_us();
            self.render_biome_tint(params.biome_tint, &mut stats);
            timings.hud_and_postfx_us += now_us().saturating_sub(t_tint);

            // 5. Vitals post-FX + 1st-person main/offhand (Batch C)
            if !params.is_menu_backdrop {
                let damage_alpha = if params.damage_tilt_timer > 0.0 {
                    ((params.damage_tilt_timer / 0.4) * 0.35).clamp(0.0, 0.55)
                } else {
                    0.0
                };
                let t_vitals = now_us();
                self.render_vitals_postfx(damage_alpha, params.low_health, params.total_time, &mut stats);
                timings.hud_and_postfx_us += now_us().saturating_sub(t_vitals);

                if params.camera_perspective == CameraPerspective::FirstPerson {
                    self.render_first_person_hand(
                        buf_idx,
                        params.camera,
                        params.hand_block,
                        params.offhand_block,
                        params.is_blocking,
                        params.walk_phase,
                        params.speed,
                        params.attack_animation,
                        daylight,
                        params.brightness,
                        v_cur,
                        i_cur,
                        &mut stats,
                        &mut timings,
                    );
                }
            }

            // 6. GPU premultiplied-alpha HUD / UI overlay
            if let Some(hud) = params.hud_pixels {
                let t_hud = now_us();
                self.render_hud_overlay(buf_idx, hud, params.hud_width, params.hud_height, &mut stats);
                timings.hud_and_postfx_us += now_us().saturating_sub(t_hud);
            }

            self.last_draw_calls = stats.draw_calls;
            self.last_triangles_rendered = stats.tri_count;
            self.last_triangles_culled = stats.tri_culled;
            self.last_subchunks_rendered = stats.subchunks_rendered;
            self.last_subchunks_culled = stats.subchunks_culled;

            let t_end = now_us();
            sceGxmEndScene(self.context, core::ptr::null(), core::ptr::null());

            // DLACZEGO: HUD compositing runs on GPU via hud.frag.cg inside the scene, so we queue the
            // display flip asynchronously without stalling the CPU via sceGxmFinish().
            sceGxmDisplayQueueAddEntry(
                self.display_sync[self.front_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );
            timings.gpu_sync_us += now_us().saturating_sub(t_end);
            self.last_timings = timings.to_external();

            self.front_buf = buf_idx;
            self.back_buf = (buf_idx + 1) % DISPLAY_BUFFER_COUNT;
            self.last_was_3d = true;
            self.rendered_3d_frames = self.rendered_3d_frames.saturating_add(1);

            stats.tri_count
        }
    }

    /// Reads a completed 3D display buffer as 960x544 XRGB8888 via burst CDRAM copy + cached swizzle.
    /// When `wait_latest` is false, reads `back_buf` (completed 2 frames ago) with zero GPU stall.
    pub fn capture_completed_xrgb(&self, dst: &mut Vec<u32>, wait_latest: bool) -> bool {
        let src_idx = if wait_latest {
            unsafe {
                sceGxmFinish(self.context);
            }
            self.front_buf
        } else {
            if self.rendered_3d_frames < DISPLAY_BUFFER_COUNT as u32 {
                return false;
            }
            self.back_buf
        };

        let width = VITA_WIDTH as usize;
        let height = VITA_HEIGHT as usize;
        let stride = VITA_STRIDE as usize;
        let total = width * height;
        dst.resize(total, 0);
        let src_buf = self.display_bufs[src_idx].ptr as *const u32;
        unsafe {
            if stride == width {
                core::ptr::copy_nonoverlapping(src_buf, dst.as_mut_ptr(), total);
            } else {
                for y in 0..height {
                    core::ptr::copy_nonoverlapping(src_buf.add(y * stride), dst.as_mut_ptr().add(y * width), width);
                }
            }
        }
        for px in dst.iter_mut() {
            let abgr = *px;
            *px = ((abgr & 0xFF) << 16) | (abgr & 0x0000_FF00) | ((abgr >> 16) & 0xFF);
        }
        true
    }

    /// Samples only a `thumb_size x thumb_size` square thumbnail directly from CDRAM with zero GPU stall.
    pub fn capture_thumbnail_xrgb(&self, thumb_size: usize) -> Option<Vec<u32>> {
        if thumb_size == 0 || self.rendered_3d_frames < DISPLAY_BUFFER_COUNT as u32 {
            return None;
        }
        let width = VITA_WIDTH as usize;
        let height = VITA_HEIGHT as usize;
        let stride = VITA_STRIDE as usize;
        let crop_size = width.min(height);
        let start_x = (width - crop_size) / 2;
        let start_y = (height - crop_size) / 2;
        let src_buf = self.display_bufs[self.back_buf].ptr as *const u32;
        let mut thumb = Vec::with_capacity(thumb_size * thumb_size);

        unsafe {
            for ty in 0..thumb_size {
                let sy = start_y + ty * crop_size / thumb_size;
                let row_ptr = src_buf.add(sy * stride);
                for tx in 0..thumb_size {
                    let sx = start_x + tx * crop_size / thumb_size;
                    let abgr = *row_ptr.add(sx);
                    let xrgb = ((abgr & 0xFF) << 16) | (abgr & 0x0000_FF00) | ((abgr >> 16) & 0xFF);
                    thumb.push(xrgb);
                }
            }
        }
        Some(thumb)
    }

    /// Presents a full-screen CPU 2D framebuffer (main menu, save list, loading screen, splash).
    pub fn present_2d_buffer(&mut self, src_pixels: &[u32], game_width: usize, game_height: usize) {
        let buf_idx = self.back_buf;
        let dst_buf = self.display_bufs[buf_idx].ptr as *mut u32;
        let y_offset = if (VITA_HEIGHT as usize) > game_height {
            (VITA_HEIGHT as usize - game_height) / 2
        } else {
            0
        };
        let copy_width = game_width.min(VITA_WIDTH as usize);
        let copy_height = game_height.min(VITA_HEIGHT as usize);

        unsafe {
            if self.last_was_3d {
                // Flush pending 3D scene writes once before switching to CPU display-buffer blits.
                sceGxmFinish(self.context);
                self.last_was_3d = false;
                self.rendered_3d_frames = 0;
            }
            if y_offset > 0 {
                let dst_slice = core::slice::from_raw_parts_mut(dst_buf, (VITA_WIDTH * VITA_HEIGHT) as usize);
                dst_slice.fill(0xFF000000);
            }
            for y in 0..copy_height {
                let src_ptr = src_pixels.as_ptr().add(y * game_width);
                let dst_ptr = dst_buf.add((y + y_offset) * VITA_STRIDE as usize);
                for px in 0..copy_width {
                    let xrgb = *src_ptr.add(px);
                    let r = (xrgb >> 16) & 0xFF;
                    let g = (xrgb >> 8) & 0xFF;
                    let b = xrgb & 0xFF;
                    *dst_ptr.add(px) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                }
            }

            sceGxmPadHeartbeat(&self.color_surfaces[buf_idx], self.display_sync[buf_idx]);
            sceGxmDisplayQueueAddEntry(
                self.display_sync[self.front_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );

            self.front_buf = buf_idx;
            self.back_buf = (buf_idx + 1) % DISPLAY_BUFFER_COUNT;
        }
    }
}
