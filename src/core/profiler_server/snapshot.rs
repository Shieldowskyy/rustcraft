//! Telemetry snapshot JSON/CSV serialization and uncompressed 24-bit BMP screenshot encoder.

/// Comprehensive snapshot of a single frame's telemetry.
#[derive(Clone, Debug, Default)]
pub struct ProfilerSnapshot {
    pub timestamp_ms: u64,
    pub fps: u32,
    pub frame_time_ms: f32,
    pub tick_ms: f32,
    pub world_io_ms: f32,
    pub render_terrain_ms: f32,
    pub render_trans_ms: f32,
    pub render_clouds_ms: f32,
    pub render_entities_ms: f32,
    pub render_sky_ms: f32,
    pub render_hand_ms: f32,
    pub render_ui_ms: f32,
    pub tick_physics_ms: f32,
    pub tick_mobs_ms: f32,
    pub tick_entities_ms: f32,
    pub tick_world_ms: f32,
    pub world_gen_ms: f32,
    pub world_mesh_ms: f32,
    pub render_cull_ms: f32,
    pub gpu_sync_ms: f32,
    pub total_ram_bytes: usize,
    pub chunk_voxels_bytes: usize,
    pub mesh_cache_bytes: usize,
    pub textures_and_fonts_bytes: usize,
    pub framebuffers_bytes: usize,
    pub entities_bytes: usize,
    pub ui_and_misc_bytes: usize,
    pub cdram_bytes: usize,
    pub pipeline_name: String,
    pub renderer_name: String,
    pub gpu_device: String,
    pub driver_info: String,
    pub draw_calls: u32,
    pub triangles_rendered: u32,
    pub triangles_culled: u32,
    pub subchunks_rendered: u32,
    pub subchunks_culled_frustum: u32,
    pub raster_threads: usize,
    pub chunks_loaded: usize,
    pub player_pos: [f32; 3],
    pub biome: String,
    pub daylight_factor: f32,
    pub platform: String,
    pub is_paused: bool,
    pub game_state: String,
    pub gamemode: String,
    pub render_distance: i32,
    pub fov_deg: f32,
    pub msaa: u32,
    pub fps_limit: u32,
    pub leaf_quality: String,
    pub clouds: bool,
    pub fog: bool,
    pub particles: bool,
    pub chunk_anim: bool,
    pub underwater_fx: bool,
    pub debug_overlay: bool,
}

impl ProfilerSnapshot {
    pub fn to_json(&self) -> String {
        format!(
            "{{\
\"timestamp\":{},\
\"fps\":{},\
\"frame_time_ms\":{:.2},\
\"times\":{{\
\"terrain\":{:.2},\
\"trans\":{:.2},\
\"meshing_io\":{:.2},\
\"world_gen\":{:.2},\
\"world_mesh\":{:.2},\
\"tick\":{:.2},\
\"tick_physics\":{:.2},\
\"tick_mobs\":{:.2},\
\"tick_entities\":{:.2},\
\"tick_world\":{:.2},\
\"clouds\":{:.2},\
\"sky\":{:.2},\
\"entities\":{:.2},\
\"hand_3d\":{:.2},\
\"cull\":{:.2},\
\"ui\":{:.2},\
\"gpu_sync\":{:.2},\
\"total\":{:.2}\
}},\
\"memory\":{{\
\"total_mb\":{:.2},\
\"voxels_mb\":{:.2},\
\"mesh_cache_mb\":{:.2},\
\"textures_mb\":{:.2},\
\"framebuffers_mb\":{:.2},\
\"entities_kb\":{:.1},\
\"misc_kb\":{:.1},\
\"cdram_mb\":{:.2}\
}},\
\"render\":{{\
\"pipeline\":\"{}\",\
\"renderer_name\":\"{}\",\
\"gpu_device\":\"{}\",\
\"driver_info\":\"{}\",\
\"draw_calls\":{},\
\"triangles_rendered\":{},\
\"triangles_culled\":{},\
\"subchunks_rendered\":{},\
\"subchunks_culled\":{},\
\"raster_threads\":{}\
}},\
\"world\":{{\
\"chunks_loaded\":{},\
\"player_x\":{:.2},\
\"player_y\":{:.2},\
\"player_z\":{:.2},\
\"biome\":\"{}\",\
\"daylight\":{:.2}\
}},\
\"game\":{{\
\"is_paused\":{},\
\"state\":\"{}\",\
\"gamemode\":\"{}\"\
}},\
\"settings\":{{\
\"render_distance\":{},\
\"fov_deg\":{:.1},\
\"msaa\":{},\
\"fps_limit\":{},\
\"leaf_quality\":\"{}\",\
\"clouds\":{},\
\"fog\":{},\
\"particles\":{},\
\"chunk_anim\":{},\
\"underwater_fx\":{},\
\"debug_overlay\":{}\
}},\
\"platform\":\"{}\"\
}}",
            self.timestamp_ms,
            self.fps,
            self.frame_time_ms,
            self.render_terrain_ms,
            self.render_trans_ms,
            self.world_io_ms,
            self.world_gen_ms,
            self.world_mesh_ms,
            self.tick_ms,
            self.tick_physics_ms,
            self.tick_mobs_ms,
            self.tick_entities_ms,
            self.tick_world_ms,
            self.render_clouds_ms,
            self.render_sky_ms,
            self.render_entities_ms,
            self.render_hand_ms,
            self.render_cull_ms,
            self.render_ui_ms,
            self.gpu_sync_ms,
            self.frame_time_ms,
            self.total_ram_bytes as f32 / (1024.0 * 1024.0),
            self.chunk_voxels_bytes as f32 / (1024.0 * 1024.0),
            self.mesh_cache_bytes as f32 / (1024.0 * 1024.0),
            self.textures_and_fonts_bytes as f32 / (1024.0 * 1024.0),
            self.framebuffers_bytes as f32 / (1024.0 * 1024.0),
            self.entities_bytes as f32 / 1024.0,
            self.ui_and_misc_bytes as f32 / 1024.0,
            self.cdram_bytes as f32 / (1024.0 * 1024.0),
            escape_json(&self.pipeline_name),
            escape_json(&self.renderer_name),
            escape_json(&self.gpu_device),
            escape_json(&self.driver_info),
            self.draw_calls,
            self.triangles_rendered,
            self.triangles_culled,
            self.subchunks_rendered,
            self.subchunks_culled_frustum,
            self.raster_threads,
            self.chunks_loaded,
            self.player_pos[0],
            self.player_pos[1],
            self.player_pos[2],
            escape_json(&self.biome),
            self.daylight_factor,
            self.is_paused,
            escape_json(&self.game_state),
            escape_json(&self.gamemode),
            self.render_distance,
            self.fov_deg,
            self.msaa,
            self.fps_limit,
            escape_json(&self.leaf_quality),
            self.clouds,
            self.fog,
            self.particles,
            self.chunk_anim,
            self.underwater_fx,
            self.debug_overlay,
            escape_json(&self.platform),
        )
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{},{},{},{},\"{}\",{},\"{}\",{},{:.1},{},{}\n",
            self.timestamp_ms,
            self.fps,
            self.frame_time_ms,
            self.render_terrain_ms,
            self.render_trans_ms,
            self.world_io_ms,
            self.tick_ms,
            self.render_clouds_ms + self.render_sky_ms,
            self.render_entities_ms,
            self.render_ui_ms,
            self.total_ram_bytes as f32 / (1024.0 * 1024.0),
            self.chunk_voxels_bytes as f32 / (1024.0 * 1024.0),
            self.mesh_cache_bytes as f32 / (1024.0 * 1024.0),
            self.framebuffers_bytes as f32 / (1024.0 * 1024.0),
            self.cdram_bytes as f32 / (1024.0 * 1024.0),
            self.draw_calls,
            self.triangles_rendered,
            self.triangles_culled,
            self.subchunks_rendered,
            self.chunks_loaded,
            self.biome,
            self.is_paused,
            self.game_state,
            self.render_distance,
            self.fov_deg,
            self.msaa,
            self.fps_limit,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ScreenshotData {
    pub bmp_bytes: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub timestamp_ms: u64,
    pub fps: u32,
    pub player_pos: [f32; 3],
    pub biome: String,
    pub game_state: String,
    pub gamemode: String,
}

impl ScreenshotData {
    pub fn to_json(&self) -> String {
        format!(
            "{{\
\"timestamp_ms\":{},\
\"width\":{},\
\"height\":{},\
\"fps\":{},\
\"player_x\":{:.2},\
\"player_y\":{:.2},\
\"player_z\":{:.2},\
\"biome\":\"{}\",\
\"game_state\":\"{}\",\
\"gamemode\":\"{}\",\
\"size_bytes\":{},\
\"url\":\"/api/screenshot\"\
}}",
            self.timestamp_ms,
            self.width,
            self.height,
            self.fps,
            self.player_pos[0],
            self.player_pos[1],
            self.player_pos[2],
            escape_json(&self.biome),
            escape_json(&self.game_state),
            escape_json(&self.gamemode),
            self.bmp_bytes.len()
        )
    }
}

/// Encodes raw `0x00RRGGBB` framebuffer pixels as an uncompressed 24-bit bottom-up BMP.
pub fn encode_bmp(color_buffer: &[u32], width: usize, height: usize) -> Vec<u8> {
    if width == 0 || height == 0 || color_buffer.len() < width * height {
        return Vec::new();
    }

    let row_bytes = width * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let row_stride = row_bytes + padding;
    let image_size = row_stride * height;
    let file_size = 54 + image_size;

    let mut bmp = vec![0u8; file_size];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
    bmp[6..10].copy_from_slice(&0u32.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());

    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&(width as i32).to_le_bytes());
    bmp[22..26].copy_from_slice(&(height as i32).to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
    bmp[30..34].copy_from_slice(&0u32.to_le_bytes());
    bmp[34..38].copy_from_slice(&(image_size as u32).to_le_bytes());
    bmp[38..42].copy_from_slice(&2835u32.to_le_bytes());
    bmp[42..46].copy_from_slice(&2835u32.to_le_bytes());
    bmp[46..50].copy_from_slice(&0u32.to_le_bytes());
    bmp[50..54].copy_from_slice(&0u32.to_le_bytes());

    for y in 0..height {
        let src_y = height - 1 - y;
        let src_row = &color_buffer[src_y * width..(src_y + 1) * width];
        let dst_row_off = 54 + y * row_stride;
        let dst_row = &mut bmp[dst_row_off..dst_row_off + row_bytes];
        for (x, &pixel) in src_row.iter().enumerate() {
            let out_idx = x * 3;
            dst_row[out_idx] = (pixel & 0xFF) as u8;
            dst_row[out_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            dst_row[out_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
        }
    }
    bmp
}

pub(super) fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', "")
}

