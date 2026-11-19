//! Playing-state HUD overlays, first-person hand, profiler telemetry, F3 debug panel, and thumbnail capture.
//! Mutates `renderer` HUD overlay, `profiler`, `debug_lines`, `debug_right_lines`, and `last_clean_thumbnail`.
//! Uses `capabilities()` and `platform::threading::worker_thread_count()` with zero platform `#[cfg]`.

use std::fmt::Write;

use crate::core::math::Mat4;
use crate::core::profiler::{FrameProfiler, MemoryBreakdown};
use crate::graphics::camera::Camera;
use crate::platform::capabilities;
use crate::world::block::BlockId;

use super::super::{CameraPerspective, InputState, RustCraftApp};

impl RustCraftApp {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_playing_hud_and_debug(
        &mut self,
        dt: f32,
        input: &InputState,
        effective_camera: &Camera,
        hand_speed: f32,
        is_crouching: bool,
        is_sprinting: bool,
        collect_profiling: bool,
        frame_start_time: Option<std::time::Instant>,
        mut cur_profiler: FrameProfiler,
    ) {
        let caps = capabilities();
        let render_ui_start = collect_profiling.then(std::time::Instant::now);

        if self.camera_perspective != CameraPerspective::ThirdPersonFront {
            self.renderer.draw_crosshair();
        }
        self.renderer.draw_break_progress(self.break_progress);

        if !caps.uses_hardware_gxm && self.camera_perspective == CameraPerspective::FirstPerson {
            let hand_start = collect_profiling.then(std::time::Instant::now);
            let hand_block: Option<u8> =
                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                    .map(|s| s.block.to_u8());
            let aspect = (self.width as f32) / (self.height as f32);
            let hand_proj = Mat4::perspective(68.0_f32.to_radians(), aspect, 0.01, 20.0);
            let hand_view_proj = hand_proj.mul(&effective_camera.view_matrix());
            let offhand_block: Option<u8> = self
                .player
                .inventory
                .offhand
                .as_ref()
                .map(|s| s.block.to_u8());
            self.renderer.draw_hand_3d(
                effective_camera,
                &hand_view_proj,
                hand_block,
                self.walk_phase,
                hand_speed,
                self.hand_animation,
            );
            self.renderer.draw_offhand_3d(
                effective_camera,
                &hand_view_proj,
                offhand_block,
                self.walk_phase,
                hand_speed,
                self.player.is_blocking,
            );
            if let Some(h_start) = hand_start {
                cur_profiler.render_hand_ms = h_start.elapsed().as_secs_f32() * 1000.0;
            }
        }

        self.renderer.draw_hotbar(&self.player);
        crate::ui::minimap::draw_world_waypoint_markers(
            &mut self.renderer.color_buffer,
            self.width,
            self.height,
            effective_camera,
            &self.minimap.waypoints,
        );
        crate::ui::minimap::draw_minimap(
            &mut self.renderer.color_buffer,
            self.width,
            self.height,
            &self.world,
            self.camera.position,
            self.camera.yaw,
            &self.mob_manager.mobs,
            &self.minimap,
        );
        self.chat
            .draw(&mut self.renderer.color_buffer, self.width, self.height);

        if input.key_tab && !self.chat.is_open {
            let p_list = if let Some(client) = &self.net_client {
                &client.player_list[..]
            } else {
                &[]
            };
            self.renderer.draw_player_list_tab(
                p_list,
                &self.settings.nickname,
                self.net_client.is_some(),
            );
        }
        if let Some(u_start) = render_ui_start {
            cur_profiler.render_ui_ms = u_start.elapsed().as_secs_f32() * 1000.0;
        }

        if collect_profiling {
            self.fps_display = (1.0 / dt.max(0.001)).round() as u32;
        }

        if caps.uses_hardware_gxm {
            let dbg_start = collect_profiling.then(std::time::Instant::now);
            if self.settings.debug_overlay {
                self.update_and_draw_debug_overlay(dt, is_crouching, is_sprinting);
            }
            if let Some(d_start) = dbg_start {
                cur_profiler.render_ui_ms += d_start.elapsed().as_secs_f32() * 1000.0;
            }
            self.pending_gxm_profiler = collect_profiling.then_some(cur_profiler);
        } else {
            if collect_profiling {
                self.finalize_frame_profiler(dt, frame_start_time, cur_profiler);
            }
            if self.settings.debug_overlay {
                self.update_and_draw_debug_overlay(dt, is_crouching, is_sprinting);
            }
        }

        if !input.is_mouse_captured && !self.chat.is_open {
            self.renderer.draw_capture_prompt(true);
        }

        if !caps.uses_hardware_gxm {
            self.thumbnail_timer += dt;
            if self.thumbnail_timer >= 0.5 {
                self.thumbnail_timer = 0.0;
                self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(
                    self.renderer.pixel_buffer(),
                    self.width,
                    self.height,
                ));
            }
        }
    }

    fn finalize_frame_profiler(
        &mut self,
        dt: f32,
        frame_start_time: Option<std::time::Instant>,
        mut cur_profiler: FrameProfiler,
    ) {
        if let Some(f_start) = frame_start_time {
            cur_profiler.total_frame_ms = f_start.elapsed().as_secs_f32() * 1000.0;
        }
        cur_profiler.fps = (1.0 / dt.max(0.001)).round() as u32;
        self.populate_render_and_memory_stats(&mut cur_profiler);
        self.profiler.smooth_update(&cur_profiler, 0.15);

        if let Some(ref server) = self.profiler_server {
            let snap = self.build_profiler_snapshot(&cur_profiler);
            server.update_snapshot(snap);
        }
    }

    fn update_and_draw_debug_overlay(&mut self, dt: f32, is_crouching: bool, is_sprinting: bool) {
        self.debug_timer += dt;
        if self.debug_timer >= 0.15 {
            self.debug_timer = 0.0;
            self.cpu_time_ms = dt * 1000.0;
            self.fps_display = (1.0 / dt.max(0.001)).round() as u32;

            let mode_str = if self.fly_mode {
                "FLY"
            } else if is_crouching {
                "SNEAK"
            } else if is_sprinting {
                "SPRINT"
            } else {
                "WALK"
            };

            let yaw_deg = (self.camera.yaw.to_degrees() % 360.0 + 360.0) % 360.0;
            let pitch_deg = self.camera.pitch.to_degrees();
            let (facing_str, axis_str) = if (45.0..135.0).contains(&yaw_deg) {
                ("East", "+X")
            } else if (135.0..225.0).contains(&yaw_deg) {
                ("South", "+Z")
            } else if (225.0..315.0).contains(&yaw_deg) {
                ("West", "-X")
            } else {
                ("North", "-Z")
            };

            let px = self.camera.position.x;
            let py = self.camera.position.y;
            let pz = self.camera.position.z;
            let bpos = self.camera.position.floor_to_ivec();
            let cx = bpos.x.div_euclid(16);
            let cz = bpos.z.div_euclid(16);
            let in_cx = bpos.x.rem_euclid(16);
            let in_cz = bpos.z.rem_euclid(16);

            let biome = self.world.biome_name_at(bpos.x, bpos.z);
            let biome_key = match biome {
                "MOUNTAINS" => "biome.mountains",
                "SNOW" => "biome.snow",
                "SNOWY TAIGA" => "biome.snowy_taiga",
                "SNOWY TUNDRA" => "biome.snowy_tundra",
                "ICE SPIKES" => "biome.ice_spikes",
                "TAIGA" => "biome.taiga",
                "DESERT" => "biome.desert",
                "BADLANDS" => "biome.badlands",
                "OCEAN" => "biome.ocean",
                "DEEP OCEAN" => "biome.deep_ocean",
                "MUSHROOM ISLAND" => "biome.mushroom_island",
                "CHERRY GROVE" => "biome.cherry_grove",
                "AUTUMN FOREST" => "biome.autumn_forest",
                "DARK FOREST" => "biome.dark_forest",
                "BIRCH FOREST" => "biome.birch_forest",
                "JUNGLE" => "biome.jungle",
                "SAVANNA" => "biome.savanna",
                "SWAMP" => "biome.swamp",
                "MEADOW" => "biome.meadow",
                "FOREST" => "biome.forest",
                _ => "biome.plains",
            };

            let time_hrs = (self.world.day_fraction() * 24.0 + 6.0) % 24.0;
            let hours = time_hrs.floor() as u32;
            let minutes = (time_hrs.fract() * 60.0) as u32;

            let (ren_name, _, _) = self.renderer.renderer_info();
            self.debug_lines[0].clear();
            let _ = write!(
                &mut self.debug_lines[0],
                "{}",
                crate::core::build_info::get_build_info()
            );

            self.debug_lines[1].clear();
            let _ = write!(
                &mut self.debug_lines[1],
                "FPS: {} ({:.1}ms) | Renderer: {} ({} thr)",
                self.fps_display,
                dt * 1000.0,
                ren_name,
                self.profiler.render_stats.raster_threads
            );

            self.debug_lines[2].clear();
            let _ = write!(
                &mut self.debug_lines[2],
                "Chunks: {} loaded | Subchunks: {} ren, {} cul",
                self.world.chunks.len(),
                self.profiler.render_stats.subchunks_rendered,
                self.profiler.render_stats.subchunks_culled_frustum
            );

            self.debug_lines[3].clear();
            let _ = write!(
                &mut self.debug_lines[3],
                "XYZ: {:.3} / {:.3} / {:.3}",
                px, py, pz
            );

            self.debug_lines[4].clear();
            let _ = write!(
                &mut self.debug_lines[4],
                "Block: {} {} {} [{} {} in chunk {}, {}]",
                bpos.x, bpos.y, bpos.z, in_cx, in_cz, cx, cz
            );

            self.debug_lines[5].clear();
            let _ = write!(
                &mut self.debug_lines[5],
                "Facing: {} ({}) (Yaw: {:.1} / Pitch: {:.1})",
                facing_str, axis_str, yaw_deg, pitch_deg
            );

            self.debug_lines[6].clear();
            let weather_str = match self.world.weather.current_type {
                crate::world::weather::WeatherType::Clear => "Clear",
                crate::world::weather::WeatherType::Rain => "Rain",
                crate::world::weather::WeatherType::Thunder => "Thunder",
            };
            let _ = write!(
                &mut self.debug_lines[6],
                "Biome: {} | Light: {:.0}% ({:02}:{:02}) | Weather: {}",
                crate::core::i18n::t(biome_key),
                self.world.sunlight_factor() * 100.0,
                hours,
                minutes,
                weather_str
            );

            self.debug_lines[7].clear();
            if let Some(target) = self.break_target {
                let target_block = self.world.get_block(target);
                let _ = write!(
                    &mut self.debug_lines[7],
                    "Target: {} [{}, {}, {}]",
                    target_block.display_name().to_uppercase(),
                    target.x,
                    target.y,
                    target.z
                );
            } else {
                let sel_name = BlockId::from_u8(self.renderer.selected_block())
                    .display_name()
                    .to_uppercase();
                let _ = write!(&mut self.debug_lines[7], "Holding: {}", sel_name);
            }

            self.debug_lines[8].clear();
            let _ = write!(
                &mut self.debug_lines[8],
                "Entities: {} mobs, {} drops, {} fx | Mode: {}",
                self.mob_manager.mobs.len(),
                self.items.len(),
                self.particles.particles.len(),
                mode_str
            );

            if self.profiler_enabled {
                while self.debug_right_lines.len() < 14 {
                    self.debug_right_lines.push(String::with_capacity(96));
                }
                let mem = &self.profiler.memory;
                let stats = &self.profiler.render_stats;
                let mesh_tris = self.renderer.mesh_cache_triangles_count();
                self.debug_right_lines[0].clear();
                if capabilities().is_vita {
                    let _ = write!(
                        &mut self.debug_right_lines[0],
                        "Platform: Sony PlayStation Vita (ARM Cortex-A9)"
                    );
                } else {
                    let _ = write!(&mut self.debug_right_lines[0], "Engine: Rust 1.85 (x86_64)");
                }

                self.debug_right_lines[1].clear();
                let _ = write!(&mut self.debug_right_lines[1], "Renderer: {}", stats.renderer_name);
                self.debug_right_lines[2].clear();
                let _ = write!(&mut self.debug_right_lines[2], "GPU: {}", stats.gpu_device);
                self.debug_right_lines[3].clear();
                let _ = write!(&mut self.debug_right_lines[3], "Driver: {}", stats.driver_info);
                self.debug_right_lines[4].clear();
                let _ = write!(
                    &mut self.debug_right_lines[4],
                    "RAM (EST): ~{:.2} MB",
                    MemoryBreakdown::to_mb(mem.total_ram_bytes)
                );
                self.debug_right_lines[5].clear();
                let _ = write!(
                    &mut self.debug_right_lines[5],
                    " - Chunk Voxels: {:.2} MB ({} ch)",
                    MemoryBreakdown::to_mb(mem.chunk_voxels_bytes),
                    self.world.chunks.len()
                );
                self.debug_right_lines[6].clear();
                let _ = write!(
                    &mut self.debug_right_lines[6],
                    " - Mesh Cache:   {:.2} MB ({} tris)",
                    MemoryBreakdown::to_mb(mem.mesh_cache_bytes),
                    mesh_tris
                );
                self.debug_right_lines[7].clear();
                let _ = write!(
                    &mut self.debug_right_lines[7],
                    " - Framebuffers: {:.2} MB (Color+Depth)",
                    MemoryBreakdown::to_mb(mem.framebuffers_bytes)
                );
                self.debug_right_lines[8].clear();
                let _ = write!(
                    &mut self.debug_right_lines[8],
                    " - Textures/Font:{:.2} MB (256x256 RGBA)",
                    MemoryBreakdown::to_mb(mem.textures_and_fonts_bytes)
                );
                self.debug_right_lines[9].clear();
                let _ = write!(
                    &mut self.debug_right_lines[9],
                    " - Entities/Misc:{:.1} KB",
                    MemoryBreakdown::to_kb(mem.entities_bytes + mem.ui_and_misc_bytes)
                );
                self.debug_right_lines[10].clear();
                let _ = write!(
                    &mut self.debug_right_lines[10],
                    "Display: {}x{} ({})",
                    self.width, self.height, stats.pipeline_name
                );
                self.debug_right_lines[11].clear();
                let _ = write!(&mut self.debug_right_lines[11], "Draw Calls: {}", stats.draw_calls);
                self.debug_right_lines[12].clear();
                let _ = write!(
                    &mut self.debug_right_lines[12],
                    "Triangles: {} ren / {} cul",
                    stats.triangles_rendered, stats.triangles_culled
                );
                self.debug_right_lines[13].clear();
                let _ = write!(
                    &mut self.debug_right_lines[13],
                    "Subchunks: {} ren / {} cul",
                    stats.subchunks_rendered, stats.subchunks_culled_frustum
                );
            }
        }

        if self.profiler_enabled {
            self.renderer.draw_advanced_debug_overlay(
                &self.debug_lines,
                &self.debug_right_lines,
                &self.profiler,
            );
        } else {
            self.renderer.draw_debug_panel(&self.debug_lines);
        }
    }
}
