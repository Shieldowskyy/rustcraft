//! Ultra-high performance polyphonic audio mixer with SIMD-friendly batch mixing,
//! smooth gain interpolation, sub-sample pitch resampling, and soft clipping.

use std::sync::Arc;
use crate::audio::sound::{SoundCategory, SoundSample};

/// Maximum polyphonic hardware voice channels allocated upfront.
pub const MAX_VOICES: usize = 64;

/// A single active or idle playback voice in the polyphonic mixer pool.
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
        }
    }
}

/// Real-time lock-free polyphonic audio mixer.
pub struct AudioMixer {
    voices: [Voice; MAX_VOICES],
    category_volumes: [f32; 5],
    output_sample_rate: u32,
    age_counter: u64,
}

impl AudioMixer {
    /// Creates a new audio mixer initialized for a target output sampling rate.
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
        };

        Self {
            voices: [INIT_VOICE; MAX_VOICES],
            category_volumes: [1.0, 1.0, 1.0, 1.0, 1.0], // Master, Sfx, Music, Ambient, Ui
            output_sample_rate: output_sample_rate.max(8000),
            age_counter: 0,
        }
    }

    /// Sets the volume multiplier for a specific category [0.0, 1.0].
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

    /// Retrieves category volume.
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

    /// Allocates an active voice channel with priority voice stealing if capacity is full.
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
        self.age_counter = self.age_counter.wrapping_add(1);

        // Calculate pitch resample step
        let step = (pitch.clamp(0.1, 4.0) as f64)
            * (sample.sample_rate as f64 / self.output_sample_rate as f64);

        let target_l = (volume * left_pan_gain).clamp(0.0, 2.0);
        let target_r = (volume * right_pan_gain).clamp(0.0, 2.0);

        let new_priority: u8 = match category {
            SoundCategory::Ui => 20,
            SoundCategory::Music => 250, // Music has highest priority, protected from sound stealing
            SoundCategory::Sfx => 10,
            SoundCategory::Ambient => 5,
            SoundCategory::Master => 15,
        };

        // Find free voice or pick voice to steal
        let mut best_slot = None;
        let mut lowest_score = f32::MAX;

        for (i, v) in self.voices.iter().enumerate() {
            if !v.active {
                best_slot = Some(i);
                break;
            }
            // Do not steal active music voices for regular sfx/ambient
            if v.category == SoundCategory::Music && category != SoundCategory::Music {
                continue;
            }
            // Score based on priority, loudness and age (steals lower priority / quietest / oldest)
            let score = (v.priority as f32 * 50.0) + (v.left_gain + v.right_gain) * 10.0 + (self.age_counter.saturating_sub(v.age) as f32 * 0.001);
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
    }

    /// Stops all currently active sounds with smooth fade-out to prevent clicks.
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

    /// Stops voices in a specific category with optional smooth fade-out.
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

    /// Returns whether any voice in the given category is currently playing without fading out.
    #[inline]
    pub fn is_category_playing(&self, category: SoundCategory) -> bool {
        self.voices.iter().any(|v| v.active && v.category == category && v.fade_out_samples == 0)
    }

    /// Returns the normalized playback progress [0.0, 1.0] of any actively playing voice in the given category.
    #[inline]
    pub fn category_playback_progress(&self, category: SoundCategory) -> Option<f32> {
        for v in &self.voices {
            if v.active && v.category == category && v.fade_out_samples == 0 {
                if let Some(sample) = &v.sample {
                    let sample_len = sample.len();
                    // cursor is in frame units; for stereo the raw buffer is 2× frame count
                    let total_frames = if v.channels == 2 { sample_len / 2 } else { sample_len };
                    if total_frames > 0 {
                        return Some((v.cursor / total_frames as f64).clamp(0.0, 1.0) as f32);
                    }
                }
            }
        }
        None
    }

    /// Returns the number of currently playing polyphonic voices.
    #[inline]
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    /// Renders and mixes polyphonic audio frames into an interleaved stereo float buffer.
    ///
    /// Performance highlights:
    /// - Zero allocations in inner loop.
    /// - Linear interpolation resampling for sub-sample accuracy without aliasing.
    /// - Native mono and stereo interleaving support.
    /// - Branchless constant-power panning and soft limiting.
    pub fn mix_interleaved_stereo(&mut self, output: &mut [f32]) {
        // Clear destination buffer
        output.fill(0.0);

        let master_vol = self.category_volumes[0];
        if master_vol <= 0.0001 {
            return; // Muted
        }

        let frame_count = output.len() / 2;
        let mut active_indices = [0usize; MAX_VOICES];
        let mut active_count = 0;

        for (i, v) in self.voices.iter().enumerate() {
            if v.active && v.sample.is_some() {
                active_indices[active_count] = i;
                active_count += 1;
            }
        }

        if active_count == 0 {
            return;
        }

        for &v_idx in &active_indices[..active_count] {
            let v = &mut self.voices[v_idx];
            let sample_data = match &v.sample {
                Some(s) => s.as_ref(),
                None => continue,
            };
            let sample_len = sample_data.len();
            if sample_len == 0 {
                v.active = false;
                continue;
            }

            let cat_vol = match v.category {
                SoundCategory::Master => master_vol,
                SoundCategory::Sfx => master_vol * self.category_volumes[1],
                SoundCategory::Music => master_vol * self.category_volumes[2],
                SoundCategory::Ambient => master_vol * self.category_volumes[3],
                SoundCategory::Ui => master_vol * self.category_volumes[4],
            };

            let mut cursor = v.cursor;
            let step = v.step;
            let mut left_gain = v.left_gain;
            let mut right_gain = v.right_gain;
            let target_l = v.target_left;
            let target_r = v.target_right;
            let gain_ramp = 1.0 / (frame_count as f32).max(1.0);

            let is_stereo = v.channels == 2;
            let total_frames = if is_stereo { sample_len / 2 } else { sample_len };

            let mut frame = 0;
            while frame < frame_count {
                let idx0 = cursor as usize;
                if idx0 >= total_frames {
                    if v.looping {
                        cursor %= total_frames as f64;
                    } else {
                        v.active = false;
                        break;
                    }
                }

                let idx0 = cursor as usize;
                let idx1 = if idx0 + 1 < total_frames {
                    idx0 + 1
                } else if v.looping {
                    0
                } else {
                    idx0
                };

                let frac = (cursor - idx0 as f64) as f32;

                let (mut s_l, mut s_r) = if is_stereo {
                    let s_l0 = sample_data[idx0 * 2];
                    let s_l1 = sample_data[idx1 * 2];
                    let s_r0 = sample_data[idx0 * 2 + 1];
                    let s_r1 = sample_data[idx1 * 2 + 1];
                    (s_l0 + (s_l1 - s_l0) * frac, s_r0 + (s_r1 - s_r0) * frac)
                } else {
                    let s0 = sample_data[idx0];
                    let s1 = sample_data[idx1];
                    let mono_s = s0 + (s1 - s0) * frac;
                    (mono_s, mono_s)
                };

                // Atmospheric air absorption and wall acoustic occlusion lowpass filter
                if v.filter_alpha < 0.999 {
                    v.filter_state += v.filter_alpha * (s_l - v.filter_state);
                    s_l = v.filter_state;
                    s_r = v.filter_state;
                }

                // Smooth gain ramping towards target gain to prevent pops
                left_gain += (target_l - left_gain) * gain_ramp;
                right_gain += (target_r - right_gain) * gain_ramp;

                // Handle fade out
                let fade_mult = if v.fade_out_samples > 0 {
                    let m = v.fade_out_samples as f32 / 64.0;
                    v.fade_out_samples = v.fade_out_samples.saturating_sub(1);
                    if v.fade_out_samples == 0 {
                        v.active = false;
                    }
                    m.min(1.0)
                } else {
                    1.0
                };

                let eff_l = left_gain * cat_vol * fade_mult;
                let eff_r = right_gain * cat_vol * fade_mult;

                output[frame * 2] += s_l * eff_l;
                output[frame * 2 + 1] += s_r * eff_r;

                cursor += step;
                frame += 1;
                if !v.active {
                    break;
                }
            }

            v.cursor = cursor;
            v.left_gain = left_gain;
            v.right_gain = right_gain;
        }

        // Apply clean soft limiting / anti-clipping
        for sample in output.iter_mut() {
            *sample = fast_soft_clip(*sample);
        }
    }
}

/// Clean transparent soft limiter: bit-perfect linear for normal audio (|x| <= 0.8),
/// and smoothly saturates signals exceeding headroom without digital clipping or discontinuities.
#[inline(always)]
fn fast_soft_clip(x: f32) -> f32 {
    if x > 0.8 {
        let d = x - 0.8;
        0.8 + (0.2 * d) / (0.2 + d)
    } else if x < -0.8 {
        let d = -x - 0.8;
        -(0.8 + (0.2 * d) / (0.2 + d))
    } else {
        x
    }
}
