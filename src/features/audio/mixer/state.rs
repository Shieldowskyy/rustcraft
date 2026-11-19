//! Stan głosów polifonicznych (`Voice`), kradzież głosów wg priorytetu oraz linie opóźniające pogłosu.

use std::sync::Arc;

use crate::core::math::Vec3;
use crate::features::audio::sound::{SoundCategory, SoundSample};
use crate::features::audio::spatial::AudioListener;

pub const MAX_VOICES: usize = 64;
pub(super) const REVERB_COMB_DELAYS: [f32; 8] = [
    0.0297, 0.0371, 0.0411, 0.0437, 0.0311, 0.0353, 0.0399, 0.0441,
];
pub(super) const REVERB_ALLPASS_DELAYS: [f32; 4] = [0.0051, 0.0073, 0.0061, 0.0083];
const REVERB_MAX_MIX_FRAMES: usize = 1024;

#[derive(Clone, Copy)]
pub(super) struct SpatialSource {
    pub position: Vec3,
    pub volume: f32,
    pub min_dist: f32,
    pub max_dist: f32,
    pub rolloff: f32,
    pub occlusion: f32,
}

pub(super) struct ReverbDelay {
    buffer: Vec<f32>,
    cursor: usize,
}

impl ReverbDelay {
    pub fn new(sample_rate: u32, delay_seconds: f32) -> Self {
        let delay_samples = ((sample_rate as f32 * delay_seconds).round() as usize).max(1);
        Self {
            buffer: vec![0.0; delay_samples],
            cursor: 0,
        }
    }

    #[inline]
    pub fn read(&self) -> f32 {
        self.buffer[self.cursor]
    }

    #[inline]
    pub fn write_and_advance(&mut self, sample: f32) {
        self.buffer[self.cursor] = sample;
        self.cursor += 1;
        if self.cursor == self.buffer.len() {
            self.cursor = 0;
        }
    }
}

#[derive(Clone)]
pub struct Voice {
    pub active: bool,
    pub sample: Option<Arc<[f32]>>,
    pub sample_rate: u32,
    pub cursor: f64,
    pub step: f64,
    pub left_gain: f32,
    pub right_gain: f32,
    pub target_left: f32,
    pub target_right: f32,
    pub category: SoundCategory,
    pub looping: bool,
    pub age: u64,
    pub fade_out_samples: u32,
    pub priority: u8,
    pub filter_alpha: f32,
    pub filter_state: f32,
    pub channels: u16,
    pub(super) spatial_source: Option<SpatialSource>,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            sample: None,
            sample_rate: 48000,
            cursor: 0.0,
            step: 1.0,
            left_gain: 1.0,
            right_gain: 1.0,
            target_left: 1.0,
            target_right: 1.0,
            category: SoundCategory::Sfx,
            looping: false,
            age: 0,
            fade_out_samples: 0,
            priority: 0,
            filter_alpha: 1.0,
            filter_state: 0.0,
            channels: 1,
            spatial_source: None,
        }
    }
}

pub struct AudioMixer {
    pub(super) voices: [Voice; MAX_VOICES],
    pub(super) category_volumes: [f32; 5],
    pub(super) output_sample_rate: u32,
    pub(super) age_counter: u64,
    pub(super) reverb_combs: [ReverbDelay; 8],
    pub(super) reverb_allpasses: [ReverbDelay; 4],
    pub(super) reverb_send_left: Vec<f32>,
    pub(super) reverb_send_right: Vec<f32>,
    pub(super) reverb_wet: f32,
    pub(super) target_reverb_wet: f32,
    pub(super) reverb_decay_seconds: f32,
    pub(super) target_reverb_decay_seconds: f32,
    pub(super) reverb_tail_samples: u32,
    pub(super) listener: AudioListener,
}

impl AudioMixer {
    pub fn new(output_sample_rate: u32) -> Self {
        const INIT_VOICE: Voice = Voice {
            active: false,
            sample: None,
            sample_rate: 48000,
            cursor: 0.0,
            step: 1.0,
            left_gain: 1.0,
            right_gain: 1.0,
            target_left: 1.0,
            target_right: 1.0,
            category: SoundCategory::Sfx,
            looping: false,
            age: 0,
            fade_out_samples: 0,
            priority: 0,
            filter_alpha: 1.0,
            filter_state: 0.0,
            channels: 1,
            spatial_source: None,
        };

        let output_sample_rate = output_sample_rate.max(8000);
        Self {
            voices: [INIT_VOICE; MAX_VOICES],
            category_volumes: [1.0, 1.0, 1.0, 1.0, 1.0],
            output_sample_rate,
            age_counter: 0,
            reverb_combs: std::array::from_fn(|i| {
                ReverbDelay::new(output_sample_rate, REVERB_COMB_DELAYS[i])
            }),
            reverb_allpasses: std::array::from_fn(|i| {
                ReverbDelay::new(output_sample_rate, REVERB_ALLPASS_DELAYS[i])
            }),
            reverb_send_left: vec![0.0; REVERB_MAX_MIX_FRAMES],
            reverb_send_right: vec![0.0; REVERB_MAX_MIX_FRAMES],
            reverb_wet: 0.0,
            target_reverb_wet: 0.0,
            reverb_decay_seconds: 0.35,
            target_reverb_decay_seconds: 0.35,
            reverb_tail_samples: 0,
            listener: AudioListener::default(),
        }
    }

    #[inline]
    pub fn set_reverb(&mut self, wet: f32, decay_seconds: f32) {
        self.target_reverb_wet = wet.clamp(0.0, 0.55);
        self.target_reverb_decay_seconds = decay_seconds.clamp(0.35, 3.5);
    }

    #[inline]
    pub fn set_category_volume(&mut self, category: SoundCategory, volume: f32) {
        let idx = match category {
            SoundCategory::Master => 0,
            SoundCategory::Sfx => 1,
            SoundCategory::Music => 2,
            SoundCategory::Ambient => 3,
            SoundCategory::Ui => 4,
        };
        self.category_volumes[idx] = volume.clamp(0.0, 2.0);
    }

    #[inline]
    pub fn category_volume(&self, category: SoundCategory) -> f32 {
        let idx = match category {
            SoundCategory::Master => 0,
            SoundCategory::Sfx => 1,
            SoundCategory::Music => 2,
            SoundCategory::Ambient => 3,
            SoundCategory::Ui => 4,
        };
        self.category_volumes[idx]
    }

    pub fn play_sample(
        &mut self,
        sample: &SoundSample,
        category: SoundCategory,
        volume: f32,
        pitch: f32,
        left_pan_gain: f32,
        right_pan_gain: f32,
        filter_alpha: f32,
        looping: bool,
    ) {
        self.play_sample_inner(
            sample,
            category,
            volume,
            pitch,
            left_pan_gain,
            right_pan_gain,
            filter_alpha,
            looping,
            None,
        );
    }

    pub fn play_spatial_sample(
        &mut self,
        sample: &SoundSample,
        category: SoundCategory,
        volume: f32,
        pitch: f32,
        position: Vec3,
        min_dist: f32,
        max_dist: f32,
        rolloff: f32,
        occlusion: f32,
        looping: bool,
    ) {
        let source = SpatialSource {
            position,
            volume,
            min_dist,
            max_dist,
            rolloff,
            occlusion,
        };
        let result = self
            .listener
            .calculate_spatial(position, min_dist, max_dist, rolloff, occlusion);
        self.play_sample_inner(
            sample,
            category,
            volume,
            pitch,
            result.left_gain,
            result.right_gain,
            result.filter_alpha,
            looping,
            Some(source),
        );
    }

    pub fn update_spatial_listener(&mut self, listener: AudioListener) {
        self.listener = listener;
        for voice in &mut self.voices {
            let Some(source) = voice.spatial_source else {
                continue;
            };
            if !voice.active {
                continue;
            }
            let result = listener.calculate_spatial(
                source.position,
                source.min_dist,
                source.max_dist,
                source.rolloff,
                source.occlusion,
            );
            voice.target_left = (source.volume * result.left_gain).clamp(0.0, 2.0);
            voice.target_right = (source.volume * result.right_gain).clamp(0.0, 2.0);
            voice.filter_alpha = result.filter_alpha.clamp(0.005, 1.0);
        }
    }

    fn play_sample_inner(
        &mut self,
        sample: &SoundSample,
        category: SoundCategory,
        volume: f32,
        pitch: f32,
        left_pan_gain: f32,
        right_pan_gain: f32,
        filter_alpha: f32,
        looping: bool,
        spatial_source: Option<SpatialSource>,
    ) {
        self.age_counter = self.age_counter.wrapping_add(1);
        let step = (pitch.clamp(0.1, 4.0) as f64)
            * (sample.sample_rate as f64 / self.output_sample_rate as f64);

        let target_l = (volume * left_pan_gain).clamp(0.0, 2.0);
        let target_r = (volume * right_pan_gain).clamp(0.0, 2.0);

        let new_priority: u8 = match category {
            SoundCategory::Ui => 20,
            SoundCategory::Music => 250,
            SoundCategory::Sfx => 10,
            SoundCategory::Ambient => 5,
            SoundCategory::Master => 15,
        };

        let mut best_slot = None;
        let mut lowest_score = f32::MAX;

        for (i, v) in self.voices.iter().enumerate() {
            if !v.active {
                best_slot = Some(i);
                break;
            }
            if v.category == SoundCategory::Music && category != SoundCategory::Music {
                continue;
            }
            let score = (v.priority as f32 * 50.0)
                + (v.left_gain + v.right_gain) * 10.0
                + (self.age_counter.saturating_sub(v.age) as f32 * 0.001);
            if score < lowest_score {
                lowest_score = score;
                best_slot = Some(i);
            }
        }

        let slot = best_slot.unwrap_or(0);
        let v = &mut self.voices[slot];
        v.active = true;
        v.sample = Some(Arc::clone(&sample.data));
        v.sample_rate = sample.sample_rate;
        v.cursor = 0.0;
        v.step = step;
        v.left_gain = target_l;
        v.right_gain = target_r;
        v.target_left = target_l;
        v.target_right = target_r;
        v.category = category;
        v.looping = looping;
        v.age = self.age_counter;
        v.fade_out_samples = 0;
        v.filter_alpha = filter_alpha.clamp(0.005, 1.0);
        v.filter_state = 0.0;
        v.priority = new_priority;
        v.channels = sample.channels;
        v.spatial_source = spatial_source;
    }

    pub fn stop_all(&mut self, fade_out_seconds: f32) {
        let fade_samples = (fade_out_seconds * self.output_sample_rate as f32) as u32;
        for v in &mut self.voices {
            if v.active {
                if fade_samples == 0 {
                    v.active = false;
                    v.sample = None;
                } else {
                    v.fade_out_samples = fade_samples.max(64);
                }
            }
        }
    }

    pub fn stop_category(&mut self, category: SoundCategory, fade_out_seconds: f32) {
        let fade_samples = (fade_out_seconds * self.output_sample_rate as f32) as u32;
        for v in &mut self.voices {
            if v.active && v.category == category {
                if fade_samples == 0 {
                    v.active = false;
                    v.sample = None;
                } else {
                    v.fade_out_samples = fade_samples.max(64);
                }
            }
        }
    }

    #[inline]
    pub fn is_category_playing(&self, category: SoundCategory) -> bool {
        self.voices
            .iter()
            .any(|v| v.active && v.category == category && v.fade_out_samples == 0)
    }

    #[inline]
    pub fn category_playback_progress(&self, category: SoundCategory) -> Option<f32> {
        for v in &self.voices {
            if v.active && v.category == category && v.fade_out_samples == 0 {
                if let Some(sample) = &v.sample {
                    let sample_len = sample.len();
                    let total_frames = if v.channels == 2 { sample_len / 2 } else { sample_len };
                    if total_frames > 0 {
                        return Some((v.cursor / total_frames as f64).clamp(0.0, 1.0) as f32);
                    }
                }
            }
        }
        None
    }

    #[inline]
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    #[inline]
    pub fn has_pending_audio(&self) -> bool {
        self.active_voice_count() > 0 || self.reverb_tail_samples > 0
    }
}
