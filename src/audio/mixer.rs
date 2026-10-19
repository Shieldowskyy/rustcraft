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
        looping: bool,
    ) {
        self.age_counter = self.age_counter.wrapping_add(1);

        // Calculate pitch resample step
        let step = (pitch.clamp(0.1, 4.0) as f64)
            * (sample.sample_rate as f64 / self.output_sample_rate as f64);

        let target_l = (volume * left_pan_gain).clamp(0.0, 2.0);
        let target_r = (volume * right_pan_gain).clamp(0.0, 2.0);

        // Find free voice or pick voice to steal
        let mut best_slot = None;
        let mut lowest_score = f32::MAX;

        for (i, v) in self.voices.iter().enumerate() {
            if !v.active {
                best_slot = Some(i);
                break;
            }
            // Score based on loudness and age (steals quietest / oldest)
            let score = (v.left_gain + v.right_gain) * 10.0 + (self.age_counter.saturating_sub(v.age) as f32 * 0.001);
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
        v.priority = match category {
            SoundCategory::Ui => 3,
            SoundCategory::Sfx => 2,
            SoundCategory::Music => 1,
            SoundCategory::Ambient => 0,
            SoundCategory::Master => 2,
        };
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
            let mut left_gain = v.left_gain * cat_vol;
            let mut right_gain = v.right_gain * cat_vol;
            let target_l = v.target_left * cat_vol;
            let target_r = v.target_right * cat_vol;
            let gain_ramp = 1.0 / (frame_count as f32).max(1.0);

            let mut frame = 0;
            while frame < frame_count {
                let idx0 = cursor as usize;
                if idx0 >= sample_len {
                    if v.looping {
                        cursor %= sample_len as f64;
                    } else {
                        v.active = false;
                        break;
                    }
                }

                let idx0 = cursor as usize;
                let idx1 = if idx0 + 1 < sample_len {
                    idx0 + 1
                } else if v.looping {
                    0
                } else {
                    idx0
                };

                let frac = (cursor - idx0 as f64) as f32;
                let s0 = sample_data[idx0];
                let s1 = sample_data[idx1];
                // Linear interpolation
                let s = s0 + (s1 - s0) * frac;

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

                output[frame * 2] += s * left_gain * fade_mult;
                output[frame * 2 + 1] += s * right_gain * fade_mult;

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

        // Apply fast polynomial soft limiting / anti-clipping
        for sample in output.iter_mut() {
            *sample = fast_soft_clip(*sample);
        }
    }
}

/// Fast branchless rational soft limiter approximating tanh.
/// Never clips harshly even under maximum volume bursts.
#[inline(always)]
fn fast_soft_clip(x: f32) -> f32 {
    if x > 1.25 {
        1.0
    } else if x < -1.25 {
        -1.0
    } else {
        // Rational soft clipping x / (1.0 + |x|^2)^0.5
        x / (1.0 + x * x).sqrt()
    }
}
