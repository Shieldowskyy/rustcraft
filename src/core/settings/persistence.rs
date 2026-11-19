//! Binary serialization and deserialization for `GameSettings` (`RCST` format).

use std::path::PathBuf;

use crate::core::i18n::Language;

use super::{
    generate_default_nickname, GameSettings, JumpMode, LeafQuality, SneakMode, SprintMode,
};

#[inline]
pub fn round_to_5_pct(val: f32) -> f32 {
    let pct = (val * 100.0).round() as i32;
    let rounded = (((pct as f32 / 5.0).round() as i32) * 5).clamp(0, 100);
    rounded as f32 / 100.0
}

impl GameSettings {
    pub const FPS_LIMIT_PRESETS: [u32; 10] = [25, 30, 40, 60, 75, 90, 120, 144, 165, 0];

    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() >= 10 && &bytes[0..4] == b"RCST" {
                let caps = crate::platform::capabilities();
                let default_msaa = caps.default_msaa as u32;
                let default_fps_limit = caps.default_fps_limit;

                let render_distance =
                    i32::from_le_bytes(bytes[4..8].try_into().unwrap_or([6, 0, 0, 0])).clamp(2, 16);
                let clouds = bytes[8] != 0;
                let underwater_fx = bytes.get(9).copied().unwrap_or(1) != 0;
                let fov_deg = if bytes.len() >= 14 {
                    f32::from_le_bytes(bytes[10..14].try_into().unwrap_or([0, 0, 140, 66]))
                        .clamp(50.0, 110.0)
                } else {
                    70.0
                };
                let fog = bytes.get(14).copied().unwrap_or(1) != 0;
                let chunk_anim = bytes.get(15).copied().unwrap_or(1) != 0;
                let particles = bytes.get(16).copied().unwrap_or(1) != 0;
                let debug_overlay = bytes.get(17).copied().unwrap_or(1) != 0;
                let fancy_leaves = bytes.get(18).copied().unwrap_or(1) != 0;
                let msaa = bytes
                    .get(19)
                    .copied()
                    .map(|v| match v {
                        2 => 2,
                        4 => 4,
                        _ => 1,
                    })
                    .unwrap_or(default_msaa);

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
                    _ => {
                        if fancy_leaves {
                            LeafQuality::Fancy
                        } else {
                            LeafQuality::Fast
                        }
                    }
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
                let master_volume = round_to_5_pct(
                    bytes.get(audio_offset).map(|&b| b as f32 / 100.0).unwrap_or(0.5),
                );
                let sfx_volume = round_to_5_pct(
                    bytes.get(audio_offset + 1).map(|&b| b as f32 / 100.0).unwrap_or(1.0),
                );
                let music_volume = round_to_5_pct(
                    bytes.get(audio_offset + 2).map(|&b| b as f32 / 100.0).unwrap_or(0.5),
                );
                let ambient_volume = round_to_5_pct(
                    bytes.get(audio_offset + 3).map(|&b| b as f32 / 100.0).unwrap_or(0.7),
                );
                let ui_volume = round_to_5_pct(
                    bytes.get(audio_offset + 4).map(|&b| b as f32 / 100.0).unwrap_or(1.0),
                );
                let mute_audio = bytes.get(audio_offset + 5).copied().unwrap_or(0) != 0;
                let vibration = bytes.get(audio_offset + 6).copied().unwrap_or(1) != 0;
                let brightness = bytes
                    .get(audio_offset + 7)
                    .copied()
                    .map(|b| (b as f32 / 100.0).clamp(0.0, 1.0))
                    .unwrap_or(0.50);
                let damage_tilt = bytes.get(audio_offset + 8).copied().unwrap_or(1) != 0;
                let mouse_sensitivity = bytes
                    .get(audio_offset + 9)
                    .map(|&b| if b == 0 { 1.0 } else { (b as f32 / 50.0).clamp(0.2, 3.0) })
                    .unwrap_or(1.0);
                let gamepad_sensitivity = bytes
                    .get(audio_offset + 10)
                    .map(|&b| if b == 0 { 1.0 } else { (b as f32 / 50.0).clamp(0.2, 3.0) })
                    .unwrap_or(1.0);
                let force_unicode_font = bytes.get(audio_offset + 11).copied().unwrap_or(0) != 0;
                let show_music_toast = bytes.get(audio_offset + 12).copied().unwrap_or(1) != 0;

                crate::core::i18n::set_language(language);
                crate::graphics::font::set_force_unicode(force_unicode_font);

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
                    brightness,
                    underwater_fx,
                    debug_overlay,
                    force_unicode_font,
                    msaa,
                    fps_limit,
                    sprint_mode,
                    sneak_mode,
                    jump_mode,
                    invert_y,
                    invert_z,
                    mouse_sensitivity,
                    gamepad_sensitivity,
                    vibration,
                    damage_tilt,
                    profiler_server,
                    profiler_server_port,
                    nickname,
                    master_volume,
                    sfx_volume,
                    music_volume,
                    ambient_volume,
                    ui_volume,
                    mute_audio,
                    show_music_toast,
                };
            }
        }
        let def = Self::default();
        crate::core::i18n::set_language(def.language);
        crate::graphics::font::set_force_unicode(def.force_unicode_font);
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
        bytes.push(u8::from(self.clouds));
        bytes.push(u8::from(self.underwater_fx));
        bytes.extend_from_slice(&self.fov_deg.to_le_bytes());
        bytes.push(u8::from(self.fog));
        bytes.push(u8::from(self.chunk_anim));
        bytes.push(u8::from(self.particles));
        bytes.push(u8::from(self.debug_overlay));
        bytes.push(u8::from(self.leaf_quality.is_fancy()));
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
        bytes.push(u8::from(self.invert_y));
        bytes.push(u8::from(self.invert_z));
        bytes.push(u8::from(self.profiler_server));
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
        bytes.push(u8::from(self.mute_audio));
        bytes.push(u8::from(self.vibration));
        bytes.push((self.brightness.clamp(0.0, 1.0) * 100.0) as u8);
        bytes.push(u8::from(self.damage_tilt));
        bytes.push((self.mouse_sensitivity.clamp(0.2, 3.0) * 50.0).round() as u8);
        bytes.push((self.gamepad_sensitivity.clamp(0.2, 3.0) * 50.0).round() as u8);
        bytes.push(u8::from(self.force_unicode_font));
        bytes.push(u8::from(self.show_music_toast));

        crate::graphics::font::set_force_unicode(self.force_unicode_font);
        std::fs::write(&path, bytes).map_err(|e| e.to_string())
    }

    fn settings_path() -> PathBuf {
        crate::world::storage::settings_file_path()
    }
}

