//! High-precision engine performance profiler, memory usage breakdown, and debug metrics.

use crate::ui::draw::pack_color;

/// Detailed breakdown of memory allocation across all game subsystems in bytes.
#[derive(Clone, Debug, Default)]
pub struct MemoryBreakdown {
    pub total_ram_bytes: usize,
    pub chunk_voxels_bytes: usize,
    pub mesh_cache_bytes: usize,
    pub textures_and_fonts_bytes: usize,
    pub framebuffers_bytes: usize,
    pub entities_bytes: usize,
    pub ui_and_misc_bytes: usize,
    pub cdram_bytes: usize,
}

impl MemoryBreakdown {
    #[inline(always)]
    pub fn to_mb(bytes: usize) -> f32 {
        bytes as f32 / (1024.0 * 1024.0)
    }

    #[inline(always)]
    pub fn to_kb(bytes: usize) -> f32 {
        bytes as f32 / 1024.0
    }
}

/// Detailed real-time renderer metrics (draw calls, triangles, culling counts, subchunks, backend info).
#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    pub pipeline_name: &'static str,
    pub renderer_name: String,
    pub gpu_device: String,
    pub driver_info: String,
    pub draw_calls: u32,
    pub subchunks_rendered: u32,
    pub subchunks_culled_frustum: u32,
    pub subchunks_culled_occlusion: u32,
    pub subchunks_empty_skipped: u32,
    pub triangles_rendered: u32,
    pub triangles_culled: u32,
    pub raster_threads: usize,
}

/// A categorized slice of frame execution time for the Minecraft-style debug pie chart.
#[derive(Clone, Copy, Debug)]
pub struct ProfilerSlice {
    pub name: &'static str,
    pub time_ms: f32,
    pub percentage: f32,
    pub color: u32,
}

/// Comprehensive engine profiler capturing microsecond execution times, RAM allocations, and render telemetry.
#[derive(Clone, Debug, Default)]
pub struct FrameProfiler {
    pub tick_ms: f32,
    pub world_io_ms: f32,
    pub render_terrain_ms: f32,
    pub render_trans_ms: f32,
    pub render_clouds_ms: f32,
    pub render_entities_ms: f32,
    pub render_sky_ms: f32,
    pub render_ui_ms: f32,
    pub total_frame_ms: f32,

    pub fps: u32,
    pub memory: MemoryBreakdown,
    pub render_stats: RenderStats,
}

impl FrameProfiler {
    /// Computes colored slices for the Minecraft-style debug pie chart.
    pub fn pie_slices(&self) -> Vec<ProfilerSlice> {
        let raw_slices = [
            ("terrain", self.render_terrain_ms, pack_color(0.20, 0.85, 0.30)),    // Green
            ("trans/water", self.render_trans_ms, pack_color(0.95, 0.85, 0.20)),  // Yellow
            ("meshing/io", self.world_io_ms, pack_color(0.20, 0.60, 0.95)),      // Blue
            ("game/tick", self.tick_ms, pack_color(0.95, 0.55, 0.15)),          // Orange
            ("clouds/sky", self.render_clouds_ms + self.render_sky_ms, pack_color(0.70, 0.35, 0.95)), // Purple
            ("entities", self.render_entities_ms, pack_color(0.95, 0.25, 0.25)), // Red
            ("gui/hud", self.render_ui_ms, pack_color(0.15, 0.85, 0.85)),        // Cyan
        ];

        let sum_ms: f32 = raw_slices.iter().map(|(_, t, _)| *t).sum::<f32>().max(0.001);

        raw_slices
            .into_iter()
            .map(|(name, t, col)| {
                let ms = t.max(0.01);
                let pct = (ms / sum_ms) * 100.0;
                ProfilerSlice {
                    name,
                    time_ms: ms,
                    percentage: pct,
                    color: col,
                }
            })
            .collect()
    }

    /// Exponential moving average update for smooth, jitter-free reading in UI.
    pub fn smooth_update(&mut self, other: &FrameProfiler, alpha: f32) {
        let lerp = |prev: f32, next: f32| prev + (next - prev) * alpha;

        self.tick_ms = lerp(self.tick_ms, other.tick_ms);
        self.world_io_ms = lerp(self.world_io_ms, other.world_io_ms);
        self.render_terrain_ms = lerp(self.render_terrain_ms, other.render_terrain_ms);
        self.render_trans_ms = lerp(self.render_trans_ms, other.render_trans_ms);
        self.render_clouds_ms = lerp(self.render_clouds_ms, other.render_clouds_ms);
        self.render_entities_ms = lerp(self.render_entities_ms, other.render_entities_ms);
        self.render_sky_ms = lerp(self.render_sky_ms, other.render_sky_ms);
        self.render_ui_ms = lerp(self.render_ui_ms, other.render_ui_ms);
        self.total_frame_ms = lerp(self.total_frame_ms, other.total_frame_ms);
        self.fps = other.fps;
        self.memory = other.memory.clone();
        self.render_stats = other.render_stats.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_pie_slices_sum_to_100() {
        let profiler = FrameProfiler {
            tick_ms: 2.0,
            world_io_ms: 1.5,
            render_terrain_ms: 8.0,
            render_trans_ms: 2.5,
            render_clouds_ms: 0.5,
            render_entities_ms: 1.0,
            render_sky_ms: 0.5,
            render_ui_ms: 0.6,
            total_frame_ms: 16.6,
            fps: 60,
            memory: MemoryBreakdown::default(),
            render_stats: RenderStats::default(),
        };

        let slices = profiler.pie_slices();
        assert_eq!(slices.len(), 7);
        let total_pct: f32 = slices.iter().map(|s| s.percentage).sum();
        assert!((total_pct - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_memory_breakdown_conversions() {
        let bytes = 1024 * 1024 * 32; // 32MB
        assert_eq!(MemoryBreakdown::to_mb(bytes), 32.0);
        assert_eq!(MemoryBreakdown::to_kb(1024 * 64), 64.0);
    }
}
