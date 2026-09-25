//! User configuration, graphics options, persistent preferences, and serialization.

use std::path::PathBuf;

/// Persistent game options and performance preferences.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameSettings {
    /// Chunk render distance radius (e.g. 2, 4, 6, 8, 10, 12, 16; default: 6).
    pub render_distance: i32,
    /// Transparent/fancy leaf rendering vs solid/fast leaves (default: true).
    pub fancy_leaves: bool,
    /// Distance fog blending enabled (default: true).
    pub fog: bool,
    /// Volumetric clouds enabled (default: true).
    pub clouds: bool,
    /// Cosmetic particle emitter system enabled (default: true).
    pub particles: bool,
    /// Animated chunk emergence effect enabled (default: true).
    pub chunk_anim: bool,
    /// Camera field of view in degrees (default: 70.0).
    pub fov_deg: f32,
    /// Underwater tint and caustic effects enabled (default: true).
    pub underwater_fx: bool,
    /// In-game F3 debug text overlay enabled (default: true).
    pub debug_overlay: bool,
    /// Multi-sample anti-aliasing sample count: 1 (off), 2 (2x), 4 (4x) (default: 4x on Vita, 1x on PC).
    pub msaa: u32,
    /// Frame rate cap: 25, 30, 40, 60, 75, 90, 120, 144, 165, 0 (0 = uncapped; default: 30 on Vita, 0 on PC).
    pub fps_limit: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        #[cfg(target_os = "vita")]
        let default_msaa = 4;
        #[cfg(not(target_os = "vita"))]
        let default_msaa = 1;

        #[cfg(target_os = "vita")]
        let default_fps_limit = 30;
        #[cfg(not(target_os = "vita"))]
        let default_fps_limit = 0;

        Self {
            render_distance: 6,
            fancy_leaves: true,
            fog: true,
            clouds: true,
            particles: true,
            chunk_anim: true,
            fov_deg: 70.0,
            underwater_fx: true,
            debug_overlay: true,
            msaa: default_msaa,
            fps_limit: default_fps_limit,
        }
    }
}

impl GameSettings {
    pub const FPS_LIMIT_PRESETS: [u32; 10] = [25, 30, 40, 60, 75, 90, 120, 144, 165, 0];

    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() >= 10 && &bytes[0..4] == b"RCST" {
                let render_distance = i32::from_le_bytes(bytes[4..8].try_into().unwrap_or([6, 0, 0, 0])).clamp(2, 16);
                let clouds = bytes[8] != 0;
                let underwater_fx = bytes.get(9).copied().unwrap_or(1) != 0;
                let fov_deg = if bytes.len() >= 14 {
                    f32::from_le_bytes(bytes[10..14].try_into().unwrap_or([0, 0, 140, 66])).clamp(50.0, 110.0)
                } else {
                    70.0
                };
                let fog = bytes.get(14).copied().unwrap_or(1) != 0;
                let chunk_anim = bytes.get(15).copied().unwrap_or(1) != 0;
                let particles = bytes.get(16).copied().unwrap_or(1) != 0;
                let debug_overlay = bytes.get(17).copied().unwrap_or(1) != 0;
                let fancy_leaves = bytes.get(18).copied().unwrap_or(1) != 0;
                #[cfg(target_os = "vita")]
                let default_msaa = 4;
                #[cfg(not(target_os = "vita"))]
                let default_msaa = 1;
                let msaa = bytes.get(19).copied().map(|v| match v { 2 => 2, 4 => 4, _ => 1 }).unwrap_or(default_msaa);

                #[cfg(target_os = "vita")]
                let default_fps_limit = 30;
                #[cfg(not(target_os = "vita"))]
                let default_fps_limit = 0;

                let fps_limit = if bytes.len() >= 24 {
                    let val = u32::from_le_bytes(bytes[20..24].try_into().unwrap_or([0, 0, 0, 0]));
                    if Self::FPS_LIMIT_PRESETS.contains(&val) {
                        val
                    } else {
                        default_fps_limit
                    }
                } else {
                    default_fps_limit
                };

                return Self {
                    render_distance,
                    fancy_leaves,
                    fog,
                    clouds,
                    particles,
                    chunk_anim,
                    fov_deg,
                    underwater_fx,
                    debug_overlay,
                    msaa,
                    fps_limit,
                };
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(b"RCST");
        bytes.extend_from_slice(&self.render_distance.to_le_bytes());
        bytes.push(if self.clouds { 1 } else { 0 });
        bytes.push(if self.underwater_fx { 1 } else { 0 });
        bytes.extend_from_slice(&self.fov_deg.to_le_bytes());
        bytes.push(if self.fog { 1 } else { 0 });
        bytes.push(if self.chunk_anim { 1 } else { 0 });
        bytes.push(if self.particles { 1 } else { 0 });
        bytes.push(if self.debug_overlay { 1 } else { 0 });
        bytes.push(if self.fancy_leaves { 1 } else { 0 });
        bytes.push(self.msaa as u8);
        bytes.extend_from_slice(&self.fps_limit.to_le_bytes());
        std::fs::write(&path, bytes).map_err(|e| e.to_string())
    }

    fn settings_path() -> PathBuf {
        crate::world::storage::settings_file_path()
    }

    pub fn cycle_render_distance(&mut self, forward: bool) {
        const CHOICES: [i32; 7] = [2, 4, 6, 8, 10, 12, 16];
        let idx = CHOICES.iter().position(|&d| d == self.render_distance).unwrap_or(2);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.render_distance = CHOICES[next_idx];
    }

    pub fn cycle_fov(&mut self, forward: bool) {
        const CHOICES: [f32; 6] = [60.0, 70.0, 80.0, 90.0, 100.0, 110.0];
        let idx = CHOICES.iter().position(|&f| (f - self.fov_deg).abs() < 1.0).unwrap_or(1);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.fov_deg = CHOICES[next_idx];
    }

    pub fn cycle_msaa(&mut self, forward: bool) {
        const CHOICES: [u32; 3] = [1, 2, 4];
        let idx = CHOICES.iter().position(|&m| m == self.msaa).unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.msaa = CHOICES[next_idx];
    }

    pub fn cycle_fps_limit(&mut self, forward: bool) {
        let idx = Self::FPS_LIMIT_PRESETS.iter().position(|&f| f == self.fps_limit).unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % Self::FPS_LIMIT_PRESETS.len()
        } else if idx == 0 {
            Self::FPS_LIMIT_PRESETS.len() - 1
        } else {
            idx - 1
        };
        self.fps_limit = Self::FPS_LIMIT_PRESETS[next_idx];
    }
}
