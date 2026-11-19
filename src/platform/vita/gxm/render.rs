//! Main per-frame `GxmRenderer::render_frame` lifecycle, hardware clear, HUD blit, and 2D presentation.
//! Orchestrates `sceGxmBeginScene`, terrain/dynamic passes, `sceGxmEndScene`, and triple-buffered flips.

#![allow(dead_code)]

use crate::graphics::camera::Camera;
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Z};

use super::ffi::*;
use super::renderer::GxmRenderer;
use super::mem::DisplayData;
use super::types::FrameRenderStats;

impl GxmRenderer {
    /// Computes perspective projection matrix mapped to the GXM depth range [0, 1].
    pub fn gxm_perspective_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> crate::core::math::Mat4 {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let mut m = [0.0f32; 16];
        m[0] = f / aspect;
        m[5] = f;
        m[10] = far / (near - far);
        m[11] = -1.0;
        m[14] = (far * near) / (near - far);
        crate::core::math::Mat4 { m }
    }

    /// Renders a 3D scene around the player on the PS Vita GPU and composites the 2D HUD (60 FPS).
    pub fn render_frame(
        &mut self,
        camera: &Camera,
        world: &mut World,
        hud_pixels: Option<&[u32]>,
        break_target: Option<crate::core::math::IVec3>,
        break_progress: f32,
        items: &[crate::gameplay::item::ItemEntity],
        mobs: &[crate::gameplay::mob::Mob],
        hand_block: Option<u8>,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
        render_distance: i32,
        fog_enabled: bool,
        clouds_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        particles: &[crate::gameplay::particles::Particle],
        damage_flash: f32,
        is_underwater: bool,
        is_in_lava: bool,
        low_health: bool,
    ) -> u32 {
        let buf_idx = self.back_buf;
        let color_buf_ptr = self.display_bufs[buf_idx].ptr as *mut u32;

        let (sky_top, sky_bottom, fog_col) = world.sky_colors();
        let daylight = world.sunlight_factor().clamp(0.05, 1.0);
        let sun_dir = world.sun_direction();

        unsafe {
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

            // Fullscreen viewport and edge clipping (960x544)
            sceGxmSetDefaultRegionClipAndViewport(self.context, VITA_WIDTH - 1, VITA_HEIGHT - 1);
            sceGxmSetViewportEnable(self.context, SCE_GXM_VIEWPORT_ENABLED);
            // Force clear triangle (z_ndc = 0.0) to write Z = 1.0 (far plane) into the depth buffer
            sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 1.0, 0.0);
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);

            // 1. Hardware sky clear and depth clearing (GPU Hardware Clear Triangle)
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
                let sky_rgba: [f32; 4] = [sky_top[0], sky_top[1], sky_top[2], 1.0f32];
                sceGxmSetUniformDataF(clear_uniform_buf, self.shaders.param_clear_color, 0, 4, sky_rgba.as_ptr());
            }

            sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
            let clear_draw_rc = sceGxmDraw(
                self.context,
                SCE_GXM_PRIMITIVE_TRIANGLES,
                SCE_GXM_INDEX_FORMAT_U16,
                self.clear_ib_mem.ptr,
                3,
            );

            // 2. Render 4-pass 3D terrain + Early-Z procedural sky (Depth range [0.0, 1.0])
            sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 0.0, 1.0);
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);

            let aspect = VITA_WIDTH as f32 / VITA_HEIGHT as f32;
            let proj = Self::gxm_perspective_matrix(camera.fov_y, aspect, 0.05, 400.0);
            let view = camera.view_matrix();
            let mvp = proj.mul(&view);

            let cam_chunk_x = (camera.position.x / CHUNK_SIZE_X as f32).floor() as i32;
            let cam_chunk_z = (camera.position.z / CHUNK_SIZE_Z as f32).floor() as i32;
            let render_dist = render_distance.clamp(2, 5);

            let mut stats = FrameRenderStats::default();
            if clear_draw_rc == SCE_OK {
                stats.draw_calls += 1;
            } else {
                crate::vita_log!("sceGxmDraw(clear) failed: 0x{:08X}", clear_draw_rc);
            }

            let is_occluded = self.render_terrain(
                buf_idx,
                camera,
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
                fog_enabled,
                chunk_anim,
                total_time,
                &mut stats,
            );

            let _ = (damage_flash, is_underwater, is_in_lava, low_health);
            let max_fog = if fog_enabled { (render_dist as f32 * 16.0).max(32.0) } else { 0.0 };
            let swing = break_progress.max(attack_animation);
            let dyn_input = super::render_dynamic::WorldEntitiesInput {
                camera,
                world,
                mvp: &mvp,
                player_feet_pos: camera.position - crate::core::math::Vec3::new(0.0, crate::features::physics::STANDING_EYE_OFFSET, 0.0),
                player_yaw: camera.yaw,
                player_pitch: camera.pitch,
                walk_phase,
                swing_progress: swing,
                player_crouching: false,
                damage_tilt_timer: 0.0,
                held_block: hand_block,
                player_armor: &[None, None, None, None],
                camera_perspective: crate::app::CameraPerspective::FirstPerson,
                break_target,
                break_progress,
                items,
                mobs,
                particles,
                arrows: &[],
                fishing_bobber: None,
                remote_players: None,
                clouds_enabled,
                fog_enabled,
                max_fog,
                fog_col,
                daylight,
                brightness: 0.0,
                total_time,
                is_occluded,
            };
            let (v_cur, i_cur) = self.render_world_entities(buf_idx, &dyn_input, &mut stats);
            self.render_first_person_hand(
                buf_idx,
                camera,
                hand_block,
                None,
                false,
                walk_phase,
                speed,
                attack_animation,
                daylight,
                0.0,
                v_cur,
                i_cur,
                &mut stats,
            );

            self.last_draw_calls = stats.draw_calls;
            self.last_triangles_rendered = stats.tri_count;
            self.last_triangles_culled = stats.tri_culled;
            self.last_subchunks_rendered = stats.subchunks_rendered;
            self.last_subchunks_culled = stats.subchunks_culled;

            sceGxmEndScene(self.context, core::ptr::null(), core::ptr::null());

            // Synchronize GPU before CPU writes to the framebuffer to prevent HUD glitches
            sceGxmFinish(self.context);

            // 3. Composite 2D HUD overlay (crosshair, hotbar, break gauge, debug text)
            if let Some(hud) = hud_pixels {
                Self::blit_hud_overlay(color_buf_ptr, hud);
            }

            // 4. Asynchronously enqueue display buffer into presentation queue (Triple Buffering)
            sceGxmDisplayQueueAddEntry(
                self.display_sync[self.front_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );

            self.front_buf = buf_idx;
            self.back_buf = (buf_idx + 1) % DISPLAY_BUFFER_COUNT;

            stats.tri_count
        }
    }

    /// Captures the last rendered 3D frame into a CPU pixel buffer and dims it by 50% for pause/menu backgrounds.
    pub fn capture_frame_snapshot(&self, dst: &mut Option<Vec<u32>>) {
        let game_width = VITA_WIDTH as usize;
        let game_height = VITA_HEIGHT as usize;
        let y_offset = 0usize;
        let vec = dst.get_or_insert_with(|| vec![0u32; game_width * game_height]);
        if vec.len() != game_width * game_height {
            vec.resize(game_width * game_height, 0);
        }
        let src_buf = self.display_bufs[self.front_buf].ptr as *const u32;
        unsafe {
            for y in 0..game_height {
                let src_offset = (y + y_offset) * VITA_STRIDE as usize;
                let dst_offset = y * game_width;
                let src_ptr = src_buf.add(src_offset);
                let dst_ptr = vec.as_mut_ptr().add(dst_offset);
                for x in 0..game_width {
                    let abgr = *src_ptr.add(x);
                    let r = abgr & 0xFF;
                    let g = (abgr >> 8) & 0xFF;
                    let b = (abgr >> 16) & 0xFF;
                    // XRGB format with 50% dimming for menu backgrounds
                    let dimmed_r = r >> 1;
                    let dimmed_g = g >> 1;
                    let dimmed_b = b >> 1;
                    *dst_ptr.add(x) = (dimmed_r << 16) | (dimmed_g << 8) | dimmed_b;
                }
            }
        }
    }

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
            if y_offset > 0 {
                let dst_slice = core::slice::from_raw_parts_mut(dst_buf, (VITA_WIDTH * VITA_HEIGHT) as usize);
                dst_slice.fill(0xFF000000);
            }

            for y in 0..copy_height {
                let src_offset = y * game_width;
                let dst_offset = (y + y_offset) * VITA_STRIDE as usize;
                let src_ptr = src_pixels.as_ptr().add(src_offset);
                let dst_ptr = dst_buf.add(dst_offset);
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

    /// Blits 2D HUD pixels onto the rendered 3D scene using strictly 32-bit aligned reads/writes.
    /// Avoids 64-bit `read_unaligned` (`LDRD`/`VLDR`) which causes ARMv7 Alignment Faults on PS Vita.
    fn blit_hud_overlay(dst_buf: *mut u32, src_pixels: &[u32]) {
        let game_width = 960usize;
        let game_height = 540usize.min(src_pixels.len() / game_width);
        let y_offset = 2usize;

        unsafe {
            for y in 0..game_height {
                let src_row = y * game_width;
                let dst_row = (y + y_offset) * VITA_STRIDE as usize;
                let src_ptr = src_pixels.as_ptr().add(src_row);
                let dst_ptr = dst_buf.add(dst_row);

                let mut x = 0;
                while x + 8 <= game_width {
                    let p0 = *src_ptr.add(x);
                    let p1 = *src_ptr.add(x + 1);
                    let p2 = *src_ptr.add(x + 2);
                    let p3 = *src_ptr.add(x + 3);
                    let p4 = *src_ptr.add(x + 4);
                    let p5 = *src_ptr.add(x + 5);
                    let p6 = *src_ptr.add(x + 6);
                    let p7 = *src_ptr.add(x + 7);
                    if (p0 | p1 | p2 | p3 | p4 | p5 | p6 | p7) == 0 {
                        x += 8;
                        continue;
                    }
                    let oct = [p0, p1, p2, p3, p4, p5, p6, p7];
                    for (i, &pixel) in oct.iter().enumerate() {
                        let a = (pixel >> 24) & 0xFF;
                        if a > 128 {
                            let r = (pixel >> 16) & 0xFF;
                            let g = (pixel >> 8) & 0xFF;
                            let b = pixel & 0xFF;
                            *dst_ptr.add(x + i) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                        }
                    }
                    x += 8;
                }
                while x < game_width {
                    let pixel = *src_ptr.add(x);
                    let a = (pixel >> 24) & 0xFF;
                    if a > 128 {
                        let r = (pixel >> 16) & 0xFF;
                        let g = (pixel >> 8) & 0xFF;
                        let b = pixel & 0xFF;
                        *dst_ptr.add(x) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                    }
                    x += 1;
                }
            }
        }
    }

}
