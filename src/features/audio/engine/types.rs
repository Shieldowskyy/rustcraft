//! Kontekst akustyczny otoczenia (`AmbientContext`), statystyki diagnostyczne (`AudioStats`) i metody sterowania głośnością/muzyką `AudioEngine`.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::core::math::Vec3;
use crate::features::audio::ring_buffer::AudioCommand;
use crate::features::audio::sound::{SoundCategory, SoundSample};

use super::state::AudioEngine;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioStats {
    pub active_voices: usize,
    pub sample_rate: u32,
    pub backend_name: &'static str,
    pub is_active: bool,
    pub commands_processed: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct AmbientContext {
    pub is_underground: bool,
    pub sky_exposed: bool,
    pub near_water: bool,
    pub water_pos: Option<Vec3>,
    pub near_leaves: bool,
    pub leaves_pos: Option<Vec3>,
    pub is_desert: bool,
    pub sand_pos: Option<Vec3>,
    pub rain_intensity: f32,
    pub reverb_wet: f32,
    pub reverb_decay_seconds: f32,
}

impl Default for AmbientContext {
    fn default() -> Self {
        Self {
            is_underground: false,
            sky_exposed: true,
            near_water: false,
            water_pos: None,
            near_leaves: false,
            leaves_pos: None,
            is_desert: false,
            sand_pos: None,
            rain_intensity: 0.0,
            reverb_wet: 0.0,
            reverb_decay_seconds: 0.35,
        }
    }
}

impl AudioEngine {
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_vol = vol.clamp(0.0, 1.0);
        let eff = if self.is_muted { 0.0 } else { self.master_vol };
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Master,
            volume: eff,
        });
    }

    pub fn set_sfx_volume(&mut self, vol: f32) {
        self.sfx_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Sfx,
            volume: self.sfx_vol,
        });
    }

    pub fn set_ui_volume(&mut self, vol: f32) {
        self.ui_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Ui,
            volume: self.ui_vol,
        });
    }

    pub fn set_ambient_volume(&mut self, vol: f32) {
        self.ambient_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Ambient,
            volume: self.ambient_vol,
        });
    }

    pub fn set_music_volume(&mut self, vol: f32) {
        self.music_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Music,
            volume: self.music_vol,
        });
    }

    pub fn play_music(&mut self, sample: Arc<SoundSample>, volume: f32, looping: bool) {
        if self.is_muted {
            return;
        }
        self.dispatch_cmd(AudioCommand::PlayMusicSample {
            sample,
            volume,
            looping,
        });
    }

    pub fn stop_music(&mut self, fade_out_seconds: f32) {
        self.dispatch_cmd(AudioCommand::StopMusic { fade_out_seconds });
    }

    pub fn is_music_playing(&self) -> bool {
        if let Some(sync_mixer) = &self.sync_mixer {
            sync_mixer.is_category_playing(SoundCategory::Music)
        } else {
            self.music_playing.load(Ordering::Relaxed)
        }
    }

    pub fn music_playback_progress(&self) -> Option<f32> {
        if let Some(sync_mixer) = &self.sync_mixer {
            sync_mixer.category_playback_progress(SoundCategory::Music)
        } else if self.is_music_playing() {
            Some((self.music_progress.load(Ordering::Relaxed) as f32 / 10000.0).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        let eff = if self.is_muted { 0.0 } else { self.master_vol };
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Master,
            volume: eff,
        });
    }

    pub fn set_muted(&mut self, muted: bool) {
        if self.is_muted != muted {
            self.toggle_mute();
        }
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    pub fn stats(&self) -> AudioStats {
        AudioStats {
            active_voices: 0,
            sample_rate: self.sample_rate,
            backend_name: self.backend_name,
            is_active: self.is_active,
            commands_processed: 0,
        }
    }
}
