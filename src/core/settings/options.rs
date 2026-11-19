//! Interactive cycling and toggling operations for `GameSettings` UI controls.

use crate::core::i18n::Language;

use super::{max_render_distance, GameSettings};

#[inline]
fn cycle_volume_5pct(current: f32, forward: bool) -> f32 {
    let pct = (current * 100.0).round() as i32;
    let rounded = (((pct as f32 / 5.0).round() as i32) * 5).clamp(0, 100);
    let next = if forward {
        if rounded >= 100 { 0 } else { (rounded + 5).min(100) }
    } else if rounded <= 0 {
        100
    } else {
        (rounded - 5).max(0)
    };
    next as f32 / 100.0
}

#[inline]
fn cycle_sensitivity(value: &mut f32, forward: bool) {
    let step = 0.1;
    if forward {
        *value += step;
        if *value > 3.05 {
            *value = 0.2;
        }
    } else {
        *value -= step;
        if *value < 0.15 {
            *value = 3.0;
        }
    }
    *value = (*value * 10.0).round() / 10.0;
}

impl GameSettings {
    pub const PORT_PRESETS: [u16; 4] = [8080, 8088, 9000, 3000];
    pub const BRIGHTNESS_LEVELS: [f32; 5] = [0.0, 0.25, 0.50, 0.75, 1.0];

    pub fn cycle_mouse_sensitivity(&mut self, forward: bool) {
        cycle_sensitivity(&mut self.mouse_sensitivity, forward);
    }

    pub fn cycle_gamepad_sensitivity(&mut self, forward: bool) {
        cycle_sensitivity(&mut self.gamepad_sensitivity, forward);
    }

    pub fn toggle_damage_tilt(&mut self) {
        self.damage_tilt = !self.damage_tilt;
    }

    pub fn cycle_master_volume(&mut self, forward: bool) {
        self.master_volume = cycle_volume_5pct(self.master_volume, forward);
    }

    pub fn cycle_sfx_volume(&mut self, forward: bool) {
        self.sfx_volume = cycle_volume_5pct(self.sfx_volume, forward);
    }

    pub fn cycle_music_volume(&mut self, forward: bool) {
        self.music_volume = cycle_volume_5pct(self.music_volume, forward);
    }

    pub fn cycle_ambient_volume(&mut self, forward: bool) {
        self.ambient_volume = cycle_volume_5pct(self.ambient_volume, forward);
    }

    pub fn cycle_ui_volume(&mut self, forward: bool) {
        self.ui_volume = cycle_volume_5pct(self.ui_volume, forward);
    }

    pub fn toggle_mute_audio(&mut self) {
        self.mute_audio = !self.mute_audio;
    }

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
        const ALL: [i32; 7] = [2, 4, 6, 8, 10, 12, 16];
        let max = max_render_distance();
        let choices: Vec<i32> = ALL.iter().copied().filter(|&d| d <= max).collect();
        let idx = choices.iter().position(|&d| d == self.render_distance).unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % choices.len()
        } else if idx == 0 {
            choices.len() - 1
        } else {
            idx - 1
        };
        self.render_distance = choices[next_idx];
    }

    /// Lowers settings that exceed the `--max-ram` budget.
    pub fn apply_ram_limit(&mut self) {
        let max = max_render_distance();
        if self.render_distance > max {
            crate::log_info!(
                "SETTINGS",
                "Render distance {} -> {} (RAM limit)",
                self.render_distance,
                max
            );
            self.render_distance = max;
        }
    }

    pub fn cycle_leaf_quality(&mut self, forward: bool) {
        self.leaf_quality.cycle(forward);
        self.fancy_leaves = self.leaf_quality.is_fancy();
    }

    pub fn cycle_fov(&mut self, forward: bool) {
        const CHOICES: [f32; 6] = [60.0, 70.0, 80.0, 90.0, 100.0, 110.0];
        let idx = CHOICES
            .iter()
            .position(|&f| (f - self.fov_deg).abs() < 1.0)
            .unwrap_or(1);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.fov_deg = CHOICES[next_idx];
    }

    pub fn cycle_brightness(&mut self, forward: bool) {
        let mut idx = 2;
        let mut min_diff = f32::MAX;
        for (i, &lvl) in Self::BRIGHTNESS_LEVELS.iter().enumerate() {
            let diff = (self.brightness - lvl).abs();
            if diff < min_diff {
                min_diff = diff;
                idx = i;
            }
        }
        let next_idx = if forward {
            (idx + 1) % Self::BRIGHTNESS_LEVELS.len()
        } else if idx == 0 {
            Self::BRIGHTNESS_LEVELS.len() - 1
        } else {
            idx - 1
        };
        self.brightness = Self::BRIGHTNESS_LEVELS[next_idx];
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
        let idx = Self::FPS_LIMIT_PRESETS
            .iter()
            .position(|&f| f == self.fps_limit)
            .unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % Self::FPS_LIMIT_PRESETS.len()
        } else if idx == 0 {
            Self::FPS_LIMIT_PRESETS.len() - 1
        } else {
            idx - 1
        };
        self.fps_limit = Self::FPS_LIMIT_PRESETS[next_idx];
    }

    pub fn cycle_sprint_mode(&mut self, _forward: bool) {
        self.sprint_mode.cycle();
    }

    pub fn cycle_sneak_mode(&mut self, _forward: bool) {
        self.sneak_mode.cycle();
    }

    pub fn cycle_jump_mode(&mut self, _forward: bool) {
        self.jump_mode.cycle();
    }

    pub fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
    }

    pub fn toggle_invert_z(&mut self) {
        self.invert_z = !self.invert_z;
    }

    pub fn toggle_vibration(&mut self) {
        self.vibration = !self.vibration;
    }
}

