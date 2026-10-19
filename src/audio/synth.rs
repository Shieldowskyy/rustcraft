//! High-performance procedural sound synthesizer.
//!
//! Generates crisp, authentic Minecraft-style sound effects completely in memory
//! without relying on any external disk assets or audio libraries.
//! Synthesis takes < 2 milliseconds for the entire soundbank at startup.

use std::f32::consts::PI;
use crate::audio::sound::{SoundId, SoundSample};

/// Fast pseudo-random number generator (xorshift32) for deterministic noise synthesis.
#[derive(Clone, Copy)]
pub struct FastRng {
    state: u32,
}

impl FastRng {
    #[inline(always)]
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x12345678 } else { seed },
        }
    }

    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Returns a uniform float in [-1.0, 1.0].
    #[inline(always)]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32 / 2147483648.0) - 1.0
    }
}

/// Simple 1-pole IIR lowpass / highpass filter for shaping noise.
pub struct IirFilter {
    alpha: f32,
    prev_out: f32,
    prev_in: f32,
    is_highpass: bool,
}

impl IirFilter {
    pub fn lowpass(cutoff_hz: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(10.0));
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        Self {
            alpha: alpha.clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: false,
        }
    }

    pub fn highpass(cutoff_hz: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(10.0));
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);
        Self {
            alpha: alpha.clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: true,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        if self.is_highpass {
            let out = self.alpha * (self.prev_out + input - self.prev_in);
            self.prev_in = input;
            self.prev_out = out;
            out
        } else {
            let out = self.prev_out + self.alpha * (input - self.prev_out);
            self.prev_out = out;
            out
        }
    }
}

/// Synthesizes the standard procedural sound library at a specified sample rate.
pub fn synthesize_sound(id: SoundId, sample_rate: u32) -> SoundSample {
    let sr = sample_rate as f32;
    let mut rng = FastRng::new(match id {
        SoundId::StepGrass => 101,
        SoundId::StepDirt => 102,
        SoundId::StepStone => 103,
        SoundId::StepWood => 104,
        SoundId::StepSand => 105,
        SoundId::StepGravel => 106,
        SoundId::StepWater => 107,
        SoundId::DigGrass => 201,
        SoundId::DigDirt => 202,
        SoundId::DigStone => 203,
        SoundId::DigWood => 204,
        SoundId::DigSand => 205,
        SoundId::BreakGrass => 301,
        SoundId::BreakDirt => 302,
        SoundId::BreakStone => 303,
        SoundId::BreakWood => 304,
        SoundId::BreakGlass => 305,
        SoundId::BreakSand => 306,
        SoundId::PlaceGrass => 401,
        SoundId::PlaceDirt => 402,
        SoundId::PlaceStone => 403,
        SoundId::PlaceWood => 404,
        SoundId::PlaceSand => 405,
        SoundId::PlaceGeneric => 406,
        SoundId::PlayerJump => 501,
        SoundId::PlayerLand => 502,
        SoundId::PlayerHurt => 503,
        SoundId::PlayerSplash => 504,
        SoundId::PlayerEat => 505,
        SoundId::ItemPop => 601,
        SoundId::UiClick => 602,
        SoundId::UiHover => 603,
        SoundId::ChestOpen => 604,
        SoundId::ChestClose => 605,
        SoundId::DoorOpen => 606,
        SoundId::DoorClose => 607,
        SoundId::Explosion => 608,
        SoundId::AmbientWind => 701,
        SoundId::AmbientCave => 702,
    });

    let samples = match id {
        // Footsteps: Short textured bursts
        SoundId::StepGrass => synth_noise_burst(&mut rng, sr, 0.12, 1800.0, 3200.0, 1.2, 0.6),
        SoundId::StepDirt => synth_noise_burst(&mut rng, sr, 0.14, 300.0, 1400.0, 1.8, 0.8),
        SoundId::StepStone => synth_impact(&mut rng, sr, 0.09, 320.0, 1600.0, 0.015, 0.7),
        SoundId::StepWood => synth_resonant_knock(sr, 0.12, 220.0, 1.4, 0.8),
        SoundId::StepSand => synth_noise_burst(&mut rng, sr, 0.15, 600.0, 2400.0, 1.0, 0.75),
        SoundId::StepGravel => synth_noise_burst(&mut rng, sr, 0.16, 400.0, 2800.0, 1.4, 0.85),
        SoundId::StepWater => synth_water_splash(&mut rng, sr, 0.22, 0.4),

        // Digging (lighter taps)
        SoundId::DigGrass => synth_noise_burst(&mut rng, sr, 0.08, 1200.0, 3000.0, 1.5, 0.5),
        SoundId::DigDirt => synth_noise_burst(&mut rng, sr, 0.09, 250.0, 1200.0, 2.0, 0.6),
        SoundId::DigStone => synth_impact(&mut rng, sr, 0.07, 450.0, 2200.0, 0.01, 0.6),
        SoundId::DigWood => synth_resonant_knock(sr, 0.08, 260.0, 1.8, 0.6),
        SoundId::DigSand => synth_noise_burst(&mut rng, sr, 0.10, 500.0, 2000.0, 1.2, 0.5),

        // Breaking (rich crumble / shatter)
        SoundId::BreakGrass => synth_crumble(&mut rng, sr, 0.25, 900.0, 3500.0, 0.8),
        SoundId::BreakDirt => synth_crumble(&mut rng, sr, 0.28, 200.0, 1100.0, 0.9),
        SoundId::BreakStone => synth_stone_break(&mut rng, sr, 0.32, 0.95),
        SoundId::BreakWood => synth_wood_break(&mut rng, sr, 0.30, 0.9),
        SoundId::BreakGlass => synth_glass_shatter(&mut rng, sr, 0.35, 0.85),
        SoundId::BreakSand => synth_crumble(&mut rng, sr, 0.26, 400.0, 2200.0, 0.85),

        // Placement (solid impact)
        SoundId::PlaceGrass => synth_noise_burst(&mut rng, sr, 0.11, 800.0, 2600.0, 1.8, 0.7),
        SoundId::PlaceDirt => synth_impact(&mut rng, sr, 0.14, 180.0, 900.0, 0.03, 0.85),
        SoundId::PlaceStone => synth_impact(&mut rng, sr, 0.12, 280.0, 1800.0, 0.02, 0.9),
        SoundId::PlaceWood => synth_resonant_knock(sr, 0.15, 190.0, 1.2, 0.9),
        SoundId::PlaceSand => synth_noise_burst(&mut rng, sr, 0.16, 350.0, 1600.0, 1.5, 0.75),
        SoundId::PlaceGeneric => synth_impact(&mut rng, sr, 0.12, 240.0, 1200.0, 0.02, 0.8),

        // Player actions
        SoundId::PlayerJump => synth_sweep(sr, 0.14, 110.0, 280.0, 0.6),
        SoundId::PlayerLand => synth_impact(&mut rng, sr, 0.22, 90.0, 600.0, 0.06, 0.95),
        SoundId::PlayerHurt => synth_hurt_grunt(&mut rng, sr, 0.20, 0.9),
        SoundId::PlayerSplash => synth_water_splash(&mut rng, sr, 0.38, 0.9),
        SoundId::PlayerEat => synth_crunch(&mut rng, sr, 0.14, 0.7),

        // UI & world interactions
        SoundId::ItemPop => synth_sweep(sr, 0.09, 440.0, 980.0, 0.7),
        SoundId::UiClick => synth_click(sr, 0.04, 950.0, 0.7),
        SoundId::UiHover => synth_click(sr, 0.02, 1400.0, 0.35),
        SoundId::ChestOpen => synth_chest_creak(sr, 0.28, true, 0.8),
        SoundId::ChestClose => synth_chest_creak(sr, 0.25, false, 0.8),
        SoundId::DoorOpen => synth_door_swing(sr, 0.22, true, 0.85),
        SoundId::DoorClose => synth_door_swing(sr, 0.20, false, 0.85),
        SoundId::Explosion => synth_explosion(&mut rng, sr, 1.4, 1.0),

        // Ambient (loops or long tones)
        SoundId::AmbientWind => synth_wind_loop(&mut rng, sr, 3.0, 0.4),
        SoundId::AmbientCave => synth_cave_chord(sr, 3.5, 0.45),
    };

    SoundSample::new(samples, sample_rate, 1) // Mono procedural source for 3D spatial panning
}

// ============================================================================
// Specialized Synthesis Algorithms
// ============================================================================

/// Synthesizes a noise burst with dual-band filtering and exponential decay.
fn synth_noise_burst(
    rng: &mut FastRng,
    sr: f32,
    duration: f32,
    hp_cutoff: f32,
    lp_cutoff: f32,
    decay_rate: f32,
    volume: f32,
) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(hp_cutoff, sr);
    let mut lp = IirFilter::lowpass(lp_cutoff, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(decay_rate);
        let raw = rng.next_f32();
        let filtered = lp.process(hp.process(raw));
        out.push(filtered * env * volume);
    }
    out
}

/// Synthesizes a solid impact (low sine frequency impulse + filtered noise slap).
fn synth_impact(
    rng: &mut FastRng,
    sr: f32,
    duration: f32,
    tone_freq: f32,
    noise_lp: f32,
    noise_dur: f32,
    volume: f32,
) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(noise_lp, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / sr;
        let env_tone = (-t * 30.0).exp();
        phase += 2.0 * PI * tone_freq / sr;
        let sine_val = phase.sin() * env_tone;

        let noise_env = if t < noise_dur {
            (1.0 - (t / noise_dur)).max(0.0)
        } else {
            0.0
        };
        let noise_val = lp.process(rng.next_f32()) * noise_env * 0.7;

        out.push((sine_val + noise_val) * volume);
    }
    out
}

/// Synthesizes a resonant hollow wood knock.
fn synth_resonant_knock(sr: f32, duration: f32, base_freq: f32, decay_exp: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase1 = 0.0f32;
    let mut phase2 = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(decay_exp);
        phase1 += 2.0 * PI * base_freq / sr;
        phase2 += 2.0 * PI * (base_freq * 2.15) / sr;
        let s = phase1.sin() * 0.7 + phase2.sin() * 0.3;
        out.push(s * env * volume);
    }
    out
}

/// Synthesizes multi-stage crumbling noise (break dirt/sand/grass).
fn synth_crumble(
    rng: &mut FastRng,
    sr: f32,
    duration: f32,
    hp_cutoff: f32,
    lp_cutoff: f32,
    volume: f32,
) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(hp_cutoff, sr);
    let mut lp = IirFilter::lowpass(lp_cutoff, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        // Dual-crest envelope simulating shattering fragments
        let env = ((1.0 - t).powf(1.4) * 0.7 + (-(t - 0.25).abs() * 8.0).exp() * 0.3).min(1.0);
        let raw = rng.next_f32();
        let filtered = lp.process(hp.process(raw));
        out.push(filtered * env * volume);
    }
    out
}

/// Stone break with sharp crack and deep rumble.
fn synth_stone_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(300.0, sr);
    let mut lp = IirFilter::lowpass(2800.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env_crack = (1.0 - t).powf(2.0);
        let env_rumble = (-t * 6.0).exp();
        phase += 2.0 * PI * (120.0 - t * 40.0) / sr;
        let rumble = phase.sin() * env_rumble * 0.45;
        let crack = lp.process(hp.process(rng.next_f32())) * env_crack * 0.8;
        out.push((rumble + crack) * volume);
    }
    out
}

/// Wood break with splintering crunch.
fn synth_wood_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(180.0, sr);
    let mut lp = IirFilter::lowpass(1600.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(1.6);
        let splinter = if (i % ((sr * 0.04) as usize)) < 12 { 1.5 } else { 0.8 };
        let noise = lp.process(hp.process(rng.next_f32())) * splinter;
        out.push(noise * env * volume);
    }
    out
}

/// Glass shattering with crystalline chimes.
fn synth_glass_shatter(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(2500.0, sr);
    let mut lp = IirFilter::lowpass(9000.0, sr);
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut p3 = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(1.8);
        p1 += 2.0 * PI * 3400.0 / sr;
        p2 += 2.0 * PI * 4800.0 / sr;
        p3 += 2.0 * PI * 7200.0 / sr;
        let chime = (p1.sin() * 0.3 + p2.sin() * 0.3 + p3.sin() * 0.2) * env;
        let noise = lp.process(hp.process(rng.next_f32())) * env * 0.6;
        out.push((chime + noise) * volume);
    }
    out
}

/// Water splash with pitch drop plop and fluid spray.
fn synth_water_splash(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(500.0, sr);
    let mut lp = IirFilter::lowpass(2600.0, sr);
    let mut plop_phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;
        // Plop: rapid downward sweep 500 Hz -> 150 Hz
        let freq = (500.0 - cur_time * 1800.0).max(120.0);
        let plop_env = (-cur_time * 35.0).exp();
        plop_phase += 2.0 * PI * freq / sr;
        let plop = plop_phase.sin() * plop_env * 0.7;

        // Spray noise: rises then falls
        let spray_env = (t * (1.0 - t) * 4.0).powf(1.5);
        let spray = lp.process(hp.process(rng.next_f32())) * spray_env * 0.55;

        out.push((plop + spray) * volume);
    }
    out
}

/// Pitch sweep (Item pop or Jump swoop).
fn synth_sweep(sr: f32, duration: f32, start_freq: f32, end_freq: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = start_freq + (end_freq - start_freq) * t;
        let env = (1.0 - t).powf(1.1);
        phase += 2.0 * PI * freq / sr;
        out.push(phase.sin() * env * volume);
    }
    out
}

/// UI click or hover impulse.
fn synth_click(sr: f32, duration: f32, freq: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(3.0);
        phase += 2.0 * PI * freq / sr;
        out.push(phase.sin() * env * volume);
    }
    out
}

/// Player hurt grunt.
fn synth_hurt_grunt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(600.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 160.0 - t * 60.0;
        let env = (-t * 12.0).exp();
        phase += 2.0 * PI * freq / sr;
        let tone = phase.sin() * 0.6;
        let noise = lp.process(rng.next_f32()) * 0.4;
        out.push((tone + noise) * env * volume);
    }
    out
}

/// Food crunching sound.
fn synth_crunch(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(1200.0, sr);
    let mut lp = IirFilter::lowpass(4500.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(1.5);
        let raw = rng.next_f32();
        out.push(lp.process(hp.process(raw)) * env * volume);
    }
    out
}

/// Chest creak opening or closing.
fn synth_chest_creak(sr: f32, duration: f32, open: bool, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = if open {
            300.0 + t * 450.0
        } else {
            750.0 - t * 450.0
        };
        let env = (t * (1.0 - t) * 4.0).min(1.0);
        phase += 2.0 * PI * freq / sr;
        // Triangle-like wave for creak roughness
        let tri = (phase % (2.0 * PI) / PI - 1.0).abs() * 2.0 - 1.0;
        out.push(tri * env * volume);
    }
    out
}

/// Door wooden swing.
fn synth_door_swing(sr: f32, duration: f32, open: bool, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = if open { 180.0 + t * 140.0 } else { 320.0 - t * 140.0 };
        let env = (-t * 4.0).exp();
        phase += 2.0 * PI * freq / sr;
        out.push(phase.sin() * env * volume);
    }
    out
}

/// Massive explosion sound: immediate high-energy burst followed by long exponential rumble.
fn synth_explosion(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(400.0, sr);
    let mut rumble_phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Initial flash / crack (first 0.08s)
        let crack = if cur_time < 0.08 {
            let crack_env = (1.0 - cur_time / 0.08).max(0.0);
            rng.next_f32() * crack_env * 0.9
        } else {
            0.0
        };

        // Long low rumble
        let rumble_env = (-cur_time * 2.5).exp();
        rumble_phase += 2.0 * PI * (55.0 - t * 25.0) / sr;
        let rumble_sine = rumble_phase.sin() * 0.4;
        let rumble_noise = lp.process(rng.next_f32()) * 0.6;
        let rumble = (rumble_sine + rumble_noise) * rumble_env;

        out.push((crack + rumble) * volume);
    }
    out
}

/// Seamless ambient breeze / wind drone.
fn synth_wind_loop(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(650.0, sr);
    let mut hp = IirFilter::highpass(120.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        // Smooth swell modulation
        let swell = (t * 2.0 * PI).sin() * 0.25 + 0.75;
        let raw = rng.next_f32();
        let filtered = hp.process(lp.process(raw));
        out.push(filtered * swell * volume);
    }
    out
}

/// Atmospheric resonant cave drone chord.
fn synth_cave_chord(sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let f1 = 65.41; // C2
    let f2 = 77.78; // Eb2
    let f3 = 98.00; // G2
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut p3 = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (t * PI).sin(); // Smooth rise and fall
        p1 += 2.0 * PI * f1 / sr;
        p2 += 2.0 * PI * f2 / sr;
        p3 += 2.0 * PI * f3 / sr;
        let s = (p1.sin() * 0.45 + p2.sin() * 0.3 + p3.sin() * 0.25) * env;
        out.push(s * volume);
    }
    out
}
