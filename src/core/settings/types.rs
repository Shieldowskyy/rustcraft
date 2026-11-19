//! Persistent settings value types, RAM budget limits, and default nickname generation.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::i18n::Language;

static MAX_RAM_MB: AtomicU64 = AtomicU64::new(0);

/// Sets the RAM budget in MB (`None` = unlimited).
pub fn set_max_ram_mb(mb: Option<u64>) {
    MAX_RAM_MB.store(mb.unwrap_or(0), Ordering::Relaxed);
}

/// Returns the RAM budget in MB, if limited.
pub fn max_ram_mb() -> Option<u64> {
    match MAX_RAM_MB.load(Ordering::Relaxed) {
        0 => None,
        v => Some(v),
    }
}

/// Largest render distance (in chunks) that fits the RAM budget.
pub fn max_render_distance() -> i32 {
    match max_ram_mb() {
        None => 16,
        Some(mb) => {
            let chunks = mb.saturating_sub(300) as f64;
            (((chunks.sqrt() - 1.0) / 2.0).floor() as i32).clamp(2, 16)
        }
    }
}

/// Delegates OS-specific hard RAM enforcement to `crate::platform::system`.
pub fn apply_hard_ram_limit(mb: u64) -> Result<(), String> {
    crate::platform::system::apply_hard_ram_limit(mb)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LeafQuality {
    Fast,
    Near,
    Medium,
    Far,
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

#[derive(Clone, Debug, PartialEq)]
pub struct GameSettings {
    pub language: Language,
    pub render_distance: i32,
    pub fancy_leaves: bool,
    pub leaf_quality: LeafQuality,
    pub fog: bool,
    pub clouds: bool,
    pub particles: bool,
    pub chunk_anim: bool,
    pub fov_deg: f32,
    pub brightness: f32,
    pub underwater_fx: bool,
    pub debug_overlay: bool,
    pub force_unicode_font: bool,
    pub msaa: u32,
    pub fps_limit: u32,
    pub sprint_mode: SprintMode,
    pub sneak_mode: SneakMode,
    pub jump_mode: JumpMode,
    pub invert_y: bool,
    pub invert_z: bool,
    pub mouse_sensitivity: f32,
    pub gamepad_sensitivity: f32,
    pub vibration: bool,
    pub damage_tilt: bool,
    pub profiler_server: bool,
    pub profiler_server_port: u16,
    pub nickname: String,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub ambient_volume: f32,
    pub ui_volume: f32,
    pub mute_audio: bool,
    pub show_music_toast: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        let caps = crate::platform::capabilities();
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
            brightness: 0.50,
            underwater_fx: true,
            debug_overlay: true,
            force_unicode_font: false,
            msaa: caps.default_msaa as u32,
            fps_limit: caps.default_fps_limit,
            sprint_mode: SprintMode::Hold,
            sneak_mode: SneakMode::Hold,
            jump_mode: JumpMode::Normal,
            invert_y: false,
            invert_z: false,
            mouse_sensitivity: 1.0,
            gamepad_sensitivity: 1.0,
            vibration: true,
            damage_tilt: true,
            profiler_server: false,
            profiler_server_port: 8080,
            nickname: generate_default_nickname(),
            master_volume: 0.5,
            sfx_volume: 1.0,
            music_volume: 0.5,
            ambient_volume: 0.7,
            ui_volume: 1.0,
            mute_audio: false,
            show_music_toast: true,
        }
    }
}

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

