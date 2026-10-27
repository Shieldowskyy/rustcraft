use crate::core::profiler::FrameProfiler;
use crate::gameplay::player::GameMode;
use super::{GameState, RustCraftApp};

impl RustCraftApp {
    /// Constructs a full telemetry and configuration snapshot for the embedded HTTP profiler server.
    pub fn build_profiler_snapshot(&self, cur_profiler: &FrameProfiler) -> crate::core::profiler_server::ProfilerSnapshot {
        let bpos = self.camera.position.floor_to_ivec();
        let biome = self.world.biome_name_at(bpos.x, bpos.z).to_string();
        let is_paused = matches!(self.state, GameState::Paused);
        let game_state_str = match &self.state {
            GameState::MainMenu => "Main Menu",
            GameState::SaveSelect => "Save Select",
            GameState::CreateWorld => "Create World",
            GameState::Settings(_) => "Settings",
            GameState::EditNickname(_) => "Change Nickname",
            GameState::Playing => "Playing",
            GameState::Paused => "Paused",
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
            platform: if cfg!(target_os = "vita") {
                "PS Vita".to_string()
            } else if cfg!(target_arch = "wasm32") {
                "WebAssembly".to_string()
            } else {
                "PC / Desktop".to_string()
            },
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
