//! Pętla miksowania stereo w czasie rzeczywistym, interpolacja liniowa wysokości dźwięku, pogłos i limiter.

use crate::features::audio::sound::SoundCategory;

use super::state::{AudioMixer, MAX_VOICES, REVERB_COMB_DELAYS};

impl AudioMixer {
    pub fn mix_interleaved_stereo(&mut self, output: &mut [f32]) {
        output.fill(0.0);

        let master_vol = self.category_volumes[0];
        if master_vol <= 0.0001 {
            return;
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

        if active_count == 0 && self.reverb_tail_samples == 0 {
            return;
        }

        if self.reverb_send_left.len() < frame_count {
            self.reverb_send_left.resize(frame_count, 0.0);
            self.reverb_send_right.resize(frame_count, 0.0);
        }
        self.reverb_send_left[..frame_count].fill(0.0);
        self.reverb_send_right[..frame_count].fill(0.0);

        let smoothing = 1.0 - (-1.0 / (0.08 * self.output_sample_rate as f32)).exp();
        self.reverb_decay_seconds +=
            (self.target_reverb_decay_seconds - self.reverb_decay_seconds) * smoothing;
        let feedback: [f32; 8] = std::array::from_fn(|i| {
            10.0_f32.powf(-3.0 * REVERB_COMB_DELAYS[i] / self.reverb_decay_seconds)
        });
        let mut has_reverb_input = false;

        for &v_idx in &active_indices[..active_count] {
            let v = &mut self.voices[v_idx];
            let Some(sample_data) = v.sample.as_deref() else {
                continue;
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

                if v.filter_alpha < 0.999 {
                    v.filter_state += v.filter_alpha * (s_l - v.filter_state);
                    s_l = v.filter_state;
                    s_r = v.filter_state;
                }

                left_gain += (target_l - left_gain) * gain_ramp;
                right_gain += (target_r - right_gain) * gain_ramp;

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

                let mixed_l = s_l * eff_l;
                let mixed_r = s_r * eff_r;
                output[frame * 2] += mixed_l;
                output[frame * 2 + 1] += mixed_r;
                if matches!(v.category, SoundCategory::Sfx | SoundCategory::Ambient) {
                    self.reverb_send_left[frame] += mixed_l;
                    self.reverb_send_right[frame] += mixed_r;
                    has_reverb_input |= mixed_l.abs() + mixed_r.abs() > 0.00001;
                }

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

        if has_reverb_input && (self.target_reverb_wet > 0.0001 || self.reverb_wet > 0.0001) {
            self.reverb_tail_samples = self
                .reverb_tail_samples
                .max((self.reverb_decay_seconds * self.output_sample_rate as f32) as u32);
        }

        let mut wet = self.reverb_wet;
        for frame in 0..frame_count {
            wet += (self.target_reverb_wet - wet) * smoothing;
            let send_l = self.reverb_send_left[frame];
            let send_r = self.reverb_send_right[frame];
            let mut reverb_l = 0.0;
            let mut reverb_r = 0.0;

            for i in 0..4 {
                let comb_l = &mut self.reverb_combs[i];
                let delayed_l = comb_l.read();
                comb_l.write_and_advance(send_l + delayed_l * feedback[i]);
                reverb_l += delayed_l;

                let comb_r = &mut self.reverb_combs[i + 4];
                let delayed_r = comb_r.read();
                comb_r.write_and_advance(send_r + delayed_r * feedback[i + 4]);
                reverb_r += delayed_r;
            }

            reverb_l *= 0.25;
            reverb_r *= 0.25;
            for i in 0..2 {
                let allpass_l = &mut self.reverb_allpasses[i];
                let delayed_l = allpass_l.read();
                let output_l = delayed_l - 0.5 * reverb_l;
                allpass_l.write_and_advance(reverb_l + 0.5 * output_l);
                reverb_l = output_l;

                let allpass_r = &mut self.reverb_allpasses[i + 2];
                let delayed_r = allpass_r.read();
                let output_r = delayed_r - 0.5 * reverb_r;
                allpass_r.write_and_advance(reverb_r + 0.5 * output_r);
                reverb_r = output_r;
            }

            output[frame * 2] += reverb_l * wet;
            output[frame * 2 + 1] += reverb_r * wet;
        }
        self.reverb_wet = wet;
        self.reverb_tail_samples = self.reverb_tail_samples.saturating_sub(frame_count as u32);

        for sample in output.iter_mut() {
            *sample = fast_soft_clip(*sample);
        }
    }
}

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
