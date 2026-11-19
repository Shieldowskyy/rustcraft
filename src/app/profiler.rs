use crate::core::profiler::{ExternalRenderTimings, FrameProfiler, MemoryBreakdown, RenderStats};
use crate::gameplay::player::GameMode;
use crate::platform::capabilities;
use super::{GameState, RustCraftApp};

impl RustCraftApp {
    /// Populates `render_stats` and `memory` breakdown on a sampled `FrameProfiler`.
    pub(crate) fn populate_render_and_memory_stats(&self, cur_profiler: &mut FrameProfiler) {
        let caps = capabilities();
        let (draw_calls, tris_rendered, tris_culled, subchunks_rendered, subchunks_culled) =
            self.renderer.last_render_telemetry();
        let raster_threads = crate::platform::threading::raster_thread_count();
        let (renderer_name, gpu_device, driver_info) = self.renderer.renderer_info();

        cur_profiler.render_stats = RenderStats {
            pipeline_name: self.renderer.backend_name(),
            renderer_name,
            gpu_device,
            driver_info,
            draw_calls,
            subchunks_rendered,
            subchunks_culled_frustum: subchunks_culled,
            subchunks_culled_occlusion: 0,
            subchunks_empty_skipped: 0,
            triangles_rendered: tris_rendered,
            triangles_culled: tris_culled,
            raster_threads,
        };

        let chunk_count = self.world.chunks.len();
        let chunk_voxels_bytes = chunk_count * caps.chunk_voxel_ram_bytes;
        let mesh_cache_bytes = if caps.is_vita {
            self.chunk_slots_info.0 * (18432 * 24 + 18432 * 2)
        } else {
            self.renderer.mesh_cache_triangles_count()
                * std::mem::size_of::<crate::world::Triangle>()
        };

        let textures_and_fonts_bytes = 256 * 256 * 4 + 256 * 1024;
        let framebuffers_bytes = (self.width * self.height * 4) * 2;
        let entities_bytes = self.mob_manager.mobs.len() * 256
            + self.items.len() * 128
            + self.particles.particles.len() * 64;
        let ui_and_misc_bytes = 256 * 1024;
        let total_ram_bytes = chunk_voxels_bytes
            + mesh_cache_bytes
            + textures_and_fonts_bytes
            + framebuffers_bytes
            + entities_bytes
            + ui_and_misc_bytes;

        let cdram_bytes = if caps.is_vita {
            (self.cdram_mb * 1024.0 * 1024.0) as usize
        } else {
            0
        };

        cur_profiler.memory = MemoryBreakdown {
            total_ram_bytes,
            chunk_voxels_bytes,
            mesh_cache_bytes,
            textures_and_fonts_bytes,
            framebuffers_bytes,
            entities_bytes,
            ui_and_misc_bytes,
            cdram_bytes,
        };
    }

    /// Combines CPU tick timings with hardware GXM/external render timings after presentation.
    pub fn finalize_external_render_profiler(
        &mut self,
        real_dt_secs: f32,
        full_frame_work_ms: f32,
        ext: ExternalRenderTimings,
    ) {
        let pending = self.pending_gxm_profiler.take();
        if pending.is_none() && self.profiler_server.is_none() && !(self.profiler_enabled && self.settings.debug_overlay) {
            return;
        }

        let mut cur = pending.unwrap_or_default();
        cur.render_cull_ms = ext.cull_ms;
        cur.world_mesh_ms = ext.chunk_mesh_upload_ms;
        cur.world_io_ms = cur.world_gen_ms + cur.world_mesh_ms;
        cur.render_terrain_ms = ext.terrain_ms;
        cur.render_sky_ms = ext.sky_ms;
        cur.render_trans_ms = ext.trans_ms;
        cur.render_clouds_ms = ext.clouds_ms;
        cur.render_entities_ms = ext.entities_ms;
        cur.render_hand_ms = ext.hand_ms;
        cur.render_ui_ms += ext.hud_and_postfx_ms;
        cur.gpu_sync_ms = ext.gpu_sync_ms;
        cur.total_frame_ms = full_frame_work_ms;
        cur.fps = (1.0 / real_dt_secs.max(0.001)).round() as u32;

        self.fps_display = cur.fps;
        self.populate_render_and_memory_stats(&mut cur);
        self.profiler.smooth_update(&cur, 0.25);

        if let Some(ref server) = self.profiler_server {
            let snap = self.build_profiler_snapshot(&cur);
            server.update_snapshot(snap);
        }
    }

    /// Constructs a full telemetry and configuration snapshot for the embedded HTTP profiler server.
    pub fn build_profiler_snapshot(&self, cur_profiler: &FrameProfiler) -> crate::core::profiler_server::ProfilerSnapshot {
        let bpos = self.camera.position.floor_to_ivec();
        let biome = self.world.biome_name_at(bpos.x, bpos.z).to_string();
        let is_paused = matches!(self.state, GameState::Paused | GameState::AchievementTree | GameState::Questbook);
        let game_state_str = match &self.state {
            GameState::MainMenu => "Main Menu",
            GameState::SaveSelect => "Save Select",
            GameState::CreateWorld => "Create World",
            GameState::Settings(_) => "Settings",
            GameState::EditNickname(_) => "Change Nickname",
            GameState::Playing => "Playing",
            GameState::Paused => "Paused",
            GameState::AchievementTree => "Achievement Tree",
            GameState::Questbook => "Questbook",
            GameState::Inventory => "Inventory",
            GameState::Chest { .. } => "Chest",
            GameState::Furnace { .. } => "Furnace",
            GameState::VillagerTrade { .. } => "Villager Trade",
            GameState::DeathScreen => "Death Screen",
            GameState::MultiplayerMenu => "Multiplayer Menu",
            GameState::MultiplayerConnecting { .. } => "Multiplayer Connecting",
            _ => "Loading",
        }.to_string();

        let gamemode_str = match self.player.gamemode {
            GameMode::Survival => "Survival",
            GameMode::Creative => "Creative",
        }.to_string();

        let leaf_quality_str = match self.settings.leaf_quality {
            crate::core::settings::LeafQuality::Fast => "Fast (Opaque)",
            crate::core::settings::LeafQuality::Near => "Near (16m)",
            crate::core::settings::LeafQuality::Medium => "Medium (32m)",
            crate::core::settings::LeafQuality::Far => "Far (64m)",
            crate::core::settings::LeafQuality::Fancy => "Fancy (Cutout)",
        }.to_string();

        crate::core::profiler_server::ProfilerSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            fps: cur_profiler.fps,
            frame_time_ms: cur_profiler.total_frame_ms,
            tick_ms: cur_profiler.tick_ms,
            world_io_ms: cur_profiler.world_io_ms,
            render_terrain_ms: cur_profiler.render_terrain_ms,
            render_trans_ms: cur_profiler.render_trans_ms,
            render_clouds_ms: cur_profiler.render_clouds_ms,
            render_entities_ms: cur_profiler.render_entities_ms,
            render_sky_ms: cur_profiler.render_sky_ms,
            render_hand_ms: cur_profiler.render_hand_ms,
            render_ui_ms: cur_profiler.render_ui_ms,
            tick_physics_ms: cur_profiler.tick_physics_ms,
            tick_mobs_ms: cur_profiler.tick_mobs_ms,
            tick_entities_ms: cur_profiler.tick_entities_ms,
            tick_world_ms: cur_profiler.tick_world_ms,
            world_gen_ms: cur_profiler.world_gen_ms,
            world_mesh_ms: cur_profiler.world_mesh_ms,
            render_cull_ms: cur_profiler.render_cull_ms,
            gpu_sync_ms: cur_profiler.gpu_sync_ms,
            total_ram_bytes: cur_profiler.memory.total_ram_bytes,
            chunk_voxels_bytes: cur_profiler.memory.chunk_voxels_bytes,
            mesh_cache_bytes: cur_profiler.memory.mesh_cache_bytes,
            textures_and_fonts_bytes: cur_profiler.memory.textures_and_fonts_bytes,
            framebuffers_bytes: cur_profiler.memory.framebuffers_bytes,
            entities_bytes: cur_profiler.memory.entities_bytes,
            ui_and_misc_bytes: cur_profiler.memory.ui_and_misc_bytes,
            cdram_bytes: cur_profiler.memory.cdram_bytes,
            pipeline_name: cur_profiler.render_stats.pipeline_name.to_string(),
            renderer_name: cur_profiler.render_stats.renderer_name.clone(),
            gpu_device: cur_profiler.render_stats.gpu_device.clone(),
            driver_info: cur_profiler.render_stats.driver_info.clone(),
            draw_calls: cur_profiler.render_stats.draw_calls,
            triangles_rendered: cur_profiler.render_stats.triangles_rendered,
            triangles_culled: cur_profiler.render_stats.triangles_culled,
            subchunks_rendered: cur_profiler.render_stats.subchunks_rendered,
            subchunks_culled_frustum: cur_profiler.render_stats.subchunks_culled_frustum,
            raster_threads: cur_profiler.render_stats.raster_threads,
            chunks_loaded: self.world.chunks.len(),
            player_pos: [self.camera.position.x, self.camera.position.y, self.camera.position.z],
            biome,
            daylight_factor: self.world.day_fraction(),
            platform: crate::platform::capabilities().display_name.to_string(),
            is_paused,
            game_state: game_state_str,
            gamemode: gamemode_str,
            render_distance: self.settings.render_distance,
            fov_deg: self.settings.fov_deg,
            msaa: self.settings.msaa,
            fps_limit: self.settings.fps_limit,
            leaf_quality: leaf_quality_str,
            clouds: self.settings.clouds,
            fog: self.settings.fog,
            particles: self.settings.particles,
            chunk_anim: self.settings.chunk_anim,
            underwater_fx: self.settings.underwater_fx,
            debug_overlay: self.settings.debug_overlay,
        }
    }
}
