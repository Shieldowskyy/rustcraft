//! User configuration, graphics options, persistent preferences, and serialization.

use std::path::PathBuf;
use crate::core::i18n::Language;

/// Leaf transparency distance and visual accuracy presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LeafQuality {
    /// Opaque fast leaf texture (no cutout, maximum performance)
    Fast,
    /// Transparent leaf cutout within 16m (~1 chunk)
    Near,
    /// Transparent leaf cutout within 32m (~2 chunks)
    Medium,
    /// Transparent leaf cutout within 64m (~4 chunks)
    Far,
    /// Transparent leaf cutout everywhere
    #[default]
    Fancy,
}

impl LeafQuality {
    pub const ALL: [LeafQuality; 5] = [
        LeafQuality::Fast,
        LeafQuality::Near,
        LeafQuality::Medium,
        LeafQuality::Far,
        LeafQuality::Fancy,
    ];

    pub fn cycle(&mut self, forward: bool) {
        let idx = Self::ALL.iter().position(|q| q == self).unwrap_or(4);
        let next = if forward {
            (idx + 1) % Self::ALL.len()
        } else if idx == 0 {
            Self::ALL.len() - 1
        } else {
            idx - 1
        };
        *self = Self::ALL[next];
    }

    pub fn translation_key(&self) -> &'static str {
        match self {
            LeafQuality::Fast => "settings.leaf_fast",
            LeafQuality::Near => "settings.leaf_near",
            LeafQuality::Medium => "settings.leaf_medium",
            LeafQuality::Far => "settings.leaf_far",
            LeafQuality::Fancy => "settings.leaf_fancy",
        }
    }

    #[inline(always)]
    pub fn is_fancy(&self) -> bool {
        *self != LeafQuality::Fast
    }

    #[inline(always)]
    pub fn max_distance_sq(&self) -> f32 {
        match self {
            LeafQuality::Fast => 0.0,
            LeafQuality::Near => 16.0 * 16.0,
            LeafQuality::Medium => 32.0 * 32.0,
            LeafQuality::Far => 64.0 * 64.0,
            LeafQuality::Fancy => f32::INFINITY,
        }
    }
}

/// Sprint button behavior mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SprintMode {
    #[default]
    Hold,
    Toggle,
}

impl SprintMode {
    pub fn cycle(&mut self) {
        *self = match self {
            SprintMode::Hold => SprintMode::Toggle,
            SprintMode::Toggle => SprintMode::Hold,
        };
    }

    pub fn translation_key(&self) -> &'static str {
        match self {
            SprintMode::Hold => "settings.mode_hold",
            SprintMode::Toggle => "settings.mode_toggle",
        }
    }
}

/// Sneak / crouch button behavior mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SneakMode {
    #[default]
    Hold,
    Toggle,
}

impl SneakMode {
    pub fn cycle(&mut self) {
        *self = match self {
            SneakMode::Hold => SneakMode::Toggle,
            SneakMode::Toggle => SneakMode::Hold,
        };
    }

    pub fn translation_key(&self) -> &'static str {
        match self {
            SneakMode::Hold => "settings.mode_hold",
            SneakMode::Toggle => "settings.mode_toggle",
        }
    }
}

/// Jump behavior mode (standard jump vs auto-stepping blocks).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JumpMode {
    #[default]
    Normal,
    AutoJump,
}

impl JumpMode {
    pub fn cycle(&mut self) {
        *self = match self {
            JumpMode::Normal => JumpMode::AutoJump,
            JumpMode::AutoJump => JumpMode::Normal,
        };
    }

    pub fn translation_key(&self) -> &'static str {
        match self {
            JumpMode::Normal => "settings.mode_normal",
            JumpMode::AutoJump => "settings.mode_autojump",
        }
    }
}

/// Persistent game options, graphics preferences, and controls.
#[derive(Clone, Debug, PartialEq)]
pub struct GameSettings {
    /// Active interface language.
    pub language: Language,
    /// Chunk render distance radius (e.g. 2, 4, 6, 8, 10, 12, 16; default: 6).
    pub render_distance: i32,
    /// Transparent/fancy leaf rendering vs solid/fast leaves (default: true).
    pub fancy_leaves: bool,
    /// Leaf transparency distance and quality setting.
    pub leaf_quality: LeafQuality,
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

    // Controls options
    /// Sprint trigger mode: Hold vs Toggle.
    pub sprint_mode: SprintMode,
    /// Sneak/crouch trigger mode: Hold vs Toggle.
    pub sneak_mode: SneakMode,
    /// Jump behavior mode: Normal vs AutoJump.
    pub jump_mode: JumpMode,
    /// Invert Y (vertical pitch) camera axis.
    pub invert_y: bool,
    /// Invert Z / horizontal (yaw) camera axis.
    pub invert_z: bool,
    /// Remote HTTP profiler server enabled (default: false).
    pub profiler_server: bool,
    /// HTTP port for the profiler server (default: 8080).
    pub profiler_server_port: u16,
    /// Player display nickname for multiplayer (default: 2 words + 3 digits).
    pub nickname: String,

    // Audio options
    /// Master volume [0.0, 1.0] (default: 0.8).
    pub master_volume: f32,
    /// Sound effects volume [0.0, 1.0] (default: 1.0).
    pub sfx_volume: f32,
    /// Music volume [0.0, 1.0] (default: 0.5).
    pub music_volume: f32,
    /// Ambient atmosphere volume [0.0, 1.0] (default: 0.7).
    pub ambient_volume: f32,
    /// UI sound volume [0.0, 1.0] (default: 1.0).
    pub ui_volume: f32,
    /// Mute all game audio (default: false).
    pub mute_audio: bool,
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
            language: Language::English,
            render_distance: 6,
            fancy_leaves: true,
            leaf_quality: LeafQuality::Fancy,
            fog: true,
            clouds: true,
            particles: true,
            chunk_anim: true,
            fov_deg: 70.0,
            underwater_fx: true,
            debug_overlay: true,
            msaa: default_msaa,
            fps_limit: default_fps_limit,
            sprint_mode: SprintMode::Hold,
            sneak_mode: SneakMode::Hold,
            jump_mode: JumpMode::Normal,
            invert_y: false,
            invert_z: false,
            profiler_server: false,
            profiler_server_port: 8080,
            nickname: generate_default_nickname(),
            master_volume: 0.8,
            sfx_volume: 1.0,
            music_volume: 0.5,
            ambient_volume: 0.7,
            ui_volume: 1.0,
            mute_audio: false,
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

                let language = match bytes.get(24).copied().unwrap_or(0) {
                    1 => Language::Polish,
                    _ => Language::English,
                };

                let leaf_quality = match bytes.get(25).copied() {
                    Some(0) => LeafQuality::Fast,
                    Some(1) => LeafQuality::Near,
                    Some(2) => LeafQuality::Medium,
                    Some(3) => LeafQuality::Far,
                    Some(4) => LeafQuality::Fancy,
                    _ => if fancy_leaves { LeafQuality::Fancy } else { LeafQuality::Fast },
                };

                let sprint_mode = match bytes.get(26).copied().unwrap_or(0) {
                    1 => SprintMode::Toggle,
                    _ => SprintMode::Hold,
                };

                let sneak_mode = match bytes.get(27).copied().unwrap_or(0) {
                    1 => SneakMode::Toggle,
                    _ => SneakMode::Hold,
                };

                let jump_mode = match bytes.get(28).copied().unwrap_or(0) {
                    1 => JumpMode::AutoJump,
                    _ => JumpMode::Normal,
                };

                let invert_y = bytes.get(29).copied().unwrap_or(0) != 0;
                let invert_z = bytes.get(30).copied().unwrap_or(0) != 0;
                let profiler_server = bytes.get(31).copied().unwrap_or(0) != 0;
                let profiler_server_port = if bytes.len() >= 34 {
                    u16::from_le_bytes(bytes[32..34].try_into().unwrap_or([144, 31]))
                } else {
                    8080
                };

                let nickname = if bytes.len() >= 35 {
                    let nick_len = bytes[34] as usize;
                    if bytes.len() >= 35 + nick_len && nick_len > 0 {
                        String::from_utf8(bytes[35..35 + nick_len].to_vec())
                            .unwrap_or_else(|_| generate_default_nickname())
                    } else {
                        generate_default_nickname()
                    }
                } else {
                    generate_default_nickname()
                };

                let audio_offset = 35 + if bytes.len() >= 35 { bytes[34] as usize } else { 0 };
                let master_volume = bytes.get(audio_offset).map(|&b| b as f32 / 100.0).unwrap_or(0.8);
                let sfx_volume = bytes.get(audio_offset + 1).map(|&b| b as f32 / 100.0).unwrap_or(1.0);
                let music_volume = bytes.get(audio_offset + 2).map(|&b| b as f32 / 100.0).unwrap_or(0.5);
                let ambient_volume = bytes.get(audio_offset + 3).map(|&b| b as f32 / 100.0).unwrap_or(0.7);
                let ui_volume = bytes.get(audio_offset + 4).map(|&b| b as f32 / 100.0).unwrap_or(1.0);
                let mute_audio = bytes.get(audio_offset + 5).copied().unwrap_or(0) != 0;

                crate::core::i18n::set_language(language);

                return Self {
                    language,
                    render_distance,
                    fancy_leaves: leaf_quality.is_fancy(),
                    leaf_quality,
                    fog,
                    clouds,
                    particles,
                    chunk_anim,
                    fov_deg,
                    underwater_fx,
                    debug_overlay,
                    msaa,
                    fps_limit,
                    sprint_mode,
                    sneak_mode,
                    jump_mode,
                    invert_y,
                    invert_z,
                    profiler_server,
                    profiler_server_port,
                    nickname,
                    master_volume,
                    sfx_volume,
                    music_volume,
                    ambient_volume,
                    ui_volume,
                    mute_audio,
                };
            }
        }
        let def = Self::default();
        crate::core::i18n::set_language(def.language);
        def
    }

    pub fn save(&self) -> Result<(), String> {
        crate::core::i18n::set_language(self.language);
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(b"RCST");
        bytes.extend_from_slice(&self.render_distance.to_le_bytes());
        bytes.push(if self.clouds { 1 } else { 0 });
        bytes.push(if self.underwater_fx { 1 } else { 0 });
        bytes.extend_from_slice(&self.fov_deg.to_le_bytes());
        bytes.push(if self.fog { 1 } else { 0 });
        bytes.push(if self.chunk_anim { 1 } else { 0 });
        bytes.push(if self.particles { 1 } else { 0 });
        bytes.push(if self.debug_overlay { 1 } else { 0 });
        bytes.push(if self.leaf_quality.is_fancy() { 1 } else { 0 });
        bytes.push(self.msaa as u8);
        bytes.extend_from_slice(&self.fps_limit.to_le_bytes());
        bytes.push(match self.language {
            Language::Polish => 1,
            Language::English => 0,
        });
        bytes.push(match self.leaf_quality {
            LeafQuality::Fast => 0,
            LeafQuality::Near => 1,
            LeafQuality::Medium => 2,
            LeafQuality::Far => 3,
            LeafQuality::Fancy => 4,
        });
        bytes.push(match self.sprint_mode {
            SprintMode::Hold => 0,
            SprintMode::Toggle => 1,
        });
        bytes.push(match self.sneak_mode {
            SneakMode::Hold => 0,
            SneakMode::Toggle => 1,
        });
        bytes.push(match self.jump_mode {
            JumpMode::Normal => 0,
            JumpMode::AutoJump => 1,
        });
        bytes.push(if self.invert_y { 1 } else { 0 });
        bytes.push(if self.invert_z { 1 } else { 0 });
        bytes.push(if self.profiler_server { 1 } else { 0 });
        bytes.extend_from_slice(&self.profiler_server_port.to_le_bytes());

        let nick_bytes = self.nickname.as_bytes();
        let nick_len = nick_bytes.len().min(32) as u8;
        bytes.push(nick_len);
        bytes.extend_from_slice(&nick_bytes[..nick_len as usize]);

        bytes.push((self.master_volume.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push((self.sfx_volume.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push((self.music_volume.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push((self.ambient_volume.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push((self.ui_volume.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push(if self.mute_audio { 1 } else { 0 });

        std::fs::write(&path, bytes).map_err(|e| e.to_string())
    }

    pub fn cycle_master_volume(&mut self, forward: bool) {
        let step = 0.1;
        self.master_volume = if forward {
            if self.master_volume >= 0.95 { 0.0 } else { (self.master_volume + step).min(1.0) }
        } else {
            if self.master_volume <= 0.05 { 1.0 } else { (self.master_volume - step).max(0.0) }
        };
    }

    pub fn cycle_sfx_volume(&mut self, forward: bool) {
        let step = 0.1;
        self.sfx_volume = if forward {
            if self.sfx_volume >= 0.95 { 0.0 } else { (self.sfx_volume + step).min(1.0) }
        } else {
            if self.sfx_volume <= 0.05 { 1.0 } else { (self.sfx_volume - step).max(0.0) }
        };
    }

    pub fn cycle_music_volume(&mut self, forward: bool) {
        let step = 0.1;
        self.music_volume = if forward {
            if self.music_volume >= 0.95 { 0.0 } else { (self.music_volume + step).min(1.0) }
        } else {
            if self.music_volume <= 0.05 { 1.0 } else { (self.music_volume - step).max(0.0) }
        };
    }

    pub fn toggle_mute_audio(&mut self) {
        self.mute_audio = !self.mute_audio;
    }

    pub const PORT_PRESETS: [u16; 4] = [8080, 8088, 9000, 3000];

    pub fn toggle_profiler_server(&mut self) {
        self.profiler_server = !self.profiler_server;
    }

    pub fn cycle_profiler_server_port(&mut self, forward: bool) {
        let idx = Self::PORT_PRESETS
            .iter()
            .position(|&p| p == self.profiler_server_port)
            .unwrap_or(0);
        let len = Self::PORT_PRESETS.len();
        let next_idx = if forward {
            (idx + 1) % len
        } else {
            (idx + len - 1) % len
        };
        self.profiler_server_port = Self::PORT_PRESETS[next_idx];
    }

    fn settings_path() -> PathBuf {
        crate::world::storage::settings_file_path()
    }

    pub fn cycle_language(&mut self, forward: bool) {
        let all = Language::all();
        let idx = all.iter().position(|&l| l == self.language).unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % all.len()
        } else if idx == 0 {
            all.len() - 1
        } else {
            idx - 1
        };
        self.language = all[next_idx];
        crate::core::i18n::set_language(self.language);
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

    pub fn cycle_leaf_quality(&mut self, forward: bool) {
        self.leaf_quality.cycle(forward);
        self.fancy_leaves = self.leaf_quality.is_fancy();
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

    pub fn cycle_sprint_mode(&mut self) {
        self.sprint_mode.cycle();
    }

    pub fn cycle_sneak_mode(&mut self) {
        self.sneak_mode.cycle();
    }

    pub fn cycle_jump_mode(&mut self) {
        self.jump_mode.cycle();
    }

    pub fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
    }

    pub fn toggle_invert_z(&mut self) {
        self.invert_z = !self.invert_z;
    }
}

/// Generates a default player nickname in the format of 2 random words and 3 digits.
pub fn generate_default_nickname() -> String {
    const ADJECTIVES: &[&str] = &[
        "Swift", "Brave", "Silent", "Iron", "Golden", "Wild", "Frost", "Shadow",
        "Storm", "Thunder", "Dark", "Bright", "Crimson", "Emerald", "Mystic",
        "Cosmic", "Solar", "Lunar", "Ancient", "Epic", "Noble", "Blazing",
        "Silver", "Mighty", "Quick", "Clever", "Pixel", "Crafty", "Diamond",
    ];
    const NOUNS: &[&str] = &[
        "Miner", "Crafter", "Knight", "Hunter", "Wolf", "Falcon", "Bear", "Builder",
        "Explorer", "Warrior", "Ranger", "Voyager", "Runner", "Golem", "Striker",
        "Wizard", "Nomad", "Pioneer", "Champion", "Seeker", "Fox", "Hawk",
        "Panda", "Tiger", "Badger", "Dragon", "Scout", "Pilot",
    ];
    let seed = crate::cli::generate_random_seed();
    let adj = ADJECTIVES[(seed as usize) % ADJECTIVES.len()];
    let noun = NOUNS[((seed >> 8) as usize) % NOUNS.len()];
    let num = 100 + ((seed >> 16) % 900);
    format!("{}{}{}", adj, noun, num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_quality_cycling_and_distances() {
        let mut quality = LeafQuality::Fancy;
        assert_eq!(quality.max_distance_sq(), f32::INFINITY);
        assert!(quality.is_fancy());

        quality.cycle(true);
        assert_eq!(quality, LeafQuality::Fast);
        assert_eq!(quality.max_distance_sq(), 0.0);
        assert!(!quality.is_fancy());

        quality.cycle(true);
        assert_eq!(quality, LeafQuality::Near);
        assert_eq!(quality.max_distance_sq(), 256.0); // 16^2

        quality.cycle(true);
        assert_eq!(quality, LeafQuality::Medium);
        assert_eq!(quality.max_distance_sq(), 1024.0); // 32^2

        quality.cycle(true);
        assert_eq!(quality, LeafQuality::Far);
        assert_eq!(quality.max_distance_sq(), 4096.0); // 64^2

        quality.cycle(false);
        assert_eq!(quality, LeafQuality::Medium);
    }

    #[test]
    fn test_control_modes_cycling() {
        let mut sprint = SprintMode::Hold;
        sprint.cycle();
        assert_eq!(sprint, SprintMode::Toggle);
        sprint.cycle();
        assert_eq!(sprint, SprintMode::Hold);

        let mut sneak = SneakMode::Hold;
        sneak.cycle();
        assert_eq!(sneak, SneakMode::Toggle);

        let mut jump = JumpMode::Normal;
        jump.cycle();
        assert_eq!(jump, JumpMode::AutoJump);
        jump.cycle();
        assert_eq!(jump, JumpMode::Normal);
    }

    #[test]
    fn test_invert_axes_toggle() {
        let mut settings = GameSettings::default();
        assert!(!settings.invert_y);
        assert!(!settings.invert_z);

        settings.toggle_invert_y();
        assert!(settings.invert_y);

        settings.toggle_invert_z();
        assert!(settings.invert_z);
    }

    #[test]
    fn test_profiler_server_settings() {
        let mut settings = GameSettings::default();
        assert!(!settings.profiler_server);
        assert_eq!(settings.profiler_server_port, 8080);

        settings.toggle_profiler_server();
        assert!(settings.profiler_server);

        settings.cycle_profiler_server_port(true);
        assert_eq!(settings.profiler_server_port, 8088);

        settings.cycle_profiler_server_port(false);
        assert_eq!(settings.profiler_server_port, 8080);

        settings.cycle_profiler_server_port(false);
        assert_eq!(settings.profiler_server_port, 3000);
    }
}
