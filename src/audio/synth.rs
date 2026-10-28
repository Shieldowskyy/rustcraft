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

/// 2-pole Biquad IIR filter for sharp acoustic resonances (stone cracks, wooden board harmonics).
#[derive(Clone, Copy)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    pub fn bandpass(center_hz: f32, q: f32, sr: f32) -> Self {
        let w0 = 2.0 * PI * center_hz.clamp(20.0, sr * 0.48) / sr;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn lowpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let w0 = 2.0 * PI * cutoff_hz.clamp(20.0, sr * 0.48) / sr;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();
        let b0 = (1.0 - cos_w0) * 0.5;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn highpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let w0 = 2.0 * PI * cutoff_hz.clamp(20.0, sr * 0.48) / sr;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();
        let b0 = (1.0 + cos_w0) * 0.5;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, in_val: f32) -> f32 {
        let out = self.b0 * in_val + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = in_val;
        self.y2 = self.y1;
        self.y1 = out;
        out
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
        SoundId::PigIdle => 801,
        SoundId::PigHurt => 802,
        SoundId::PigDeath => 803,
        SoundId::CowIdle => 804,
        SoundId::CowHurt => 805,
        SoundId::CowDeath => 806,
        SoundId::ChickenIdle => 807,
        SoundId::ChickenHurt => 808,
        SoundId::ChickenDeath => 809,
        SoundId::ZombieIdle => 810,
        SoundId::ZombieHurt => 811,
        SoundId::ZombieDeath => 812,
        SoundId::SkeletonIdle => 813,
        SoundId::SkeletonHurt => 814,
        SoundId::SkeletonDeath => 815,
        SoundId::CreeperHiss => 816,
        SoundId::CreeperHurt => 817,
        SoundId::CreeperDeath => 818,
        SoundId::SheepIdle => 819,
        SoundId::SheepHurt => 820,
        SoundId::SheepDeath => 821,
        SoundId::SheepShear => 822,
        SoundId::VillagerIdle => 831,
        SoundId::VillagerHurt => 832,
        SoundId::VillagerDeath => 833,
        SoundId::VillagerTrade => 834,
        SoundId::VillagerNo => 835,
        SoundId::IronGolemIdle => 836,
        SoundId::IronGolemHurt => 837,
        SoundId::IronGolemDeath => 838,
        SoundId::IronGolemAttack => 839,
        SoundId::FishFlop => 823,
        SoundId::DolphinIdle => 824,
        SoundId::DolphinHurt => 825,
        SoundId::DolphinDeath => 826,
        SoundId::DolphinJump => 827,
        SoundId::FishingCast => 828,
        SoundId::FishingReel => 829,
        SoundId::FishCatch => 830,
        SoundId::AmbientWind => 701,
        SoundId::AmbientCave => 702,
        SoundId::AmbientWater => 703,
        SoundId::AmbientLeaves => 704,
        SoundId::AmbientCaveRocks => 705,
        SoundId::StepWoodCreak => 706,
        SoundId::AmbientDesertSand => 707,
        SoundId::Rain => 708,
        SoundId::ThunderClose => 709,
        SoundId::ThunderDistant => 710,
        SoundId::BowShoot => 711,
        SoundId::BowHit => 712,
        SoundId::ShieldBlock => 713,
        SoundId::ArmorEquip => 714,
    });

    let samples = match id {
        // Footsteps: Short textured bursts
        SoundId::StepGrass => synth_noise_burst(&mut rng, sr, 0.12, 1800.0, 3200.0, 1.2, 0.6),
        SoundId::StepDirt => synth_noise_burst(&mut rng, sr, 0.14, 300.0, 1400.0, 1.8, 0.8),
        SoundId::StepStone => synth_impact(&mut rng, sr, 0.09, 320.0, 1600.0, 0.015, 0.7),
        SoundId::StepWood => synth_wood_step(&mut rng, sr, 0.14, 0.85),
        SoundId::StepWoodCreak => synth_wood_creak(&mut rng, sr, 0.24, 0.70),
        SoundId::StepSand => synth_noise_burst(&mut rng, sr, 0.15, 600.0, 2400.0, 1.0, 0.75),
        SoundId::StepGravel => synth_noise_burst(&mut rng, sr, 0.16, 400.0, 2800.0, 1.4, 0.85),
        SoundId::StepWater => synth_water_splash(&mut rng, sr, 0.22, 0.4),

        // Digging (tactile punchy Minecraft taps)
        SoundId::DigGrass => synth_grass_dig(&mut rng, sr, 0.075, 0.75),
        SoundId::DigDirt => synth_dirt_dig(&mut rng, sr, 0.08, 0.8),
        SoundId::DigStone => synth_stone_dig(&mut rng, sr, 0.07, 0.85),
        SoundId::DigWood => synth_wood_dig(&mut rng, sr, 0.08, 0.8),
        SoundId::DigSand => synth_sand_dig(&mut rng, sr, 0.085, 0.75),

        // Breaking (authentic Minecraft multi-debris crumple / shatter)
        SoundId::BreakGrass => synth_grass_break(&mut rng, sr, 0.28, 0.9),
        SoundId::BreakDirt => synth_dirt_break(&mut rng, sr, 0.30, 0.95),
        SoundId::BreakStone => synth_stone_break(&mut rng, sr, 0.35, 1.0),
        SoundId::BreakWood => synth_wood_break(&mut rng, sr, 0.32, 0.95),
        SoundId::BreakGlass => synth_glass_shatter(&mut rng, sr, 0.38, 0.9),
        SoundId::BreakSand => synth_sand_break(&mut rng, sr, 0.28, 0.85),

        // Placement (solid impact)
        SoundId::PlaceGrass => synth_noise_burst(&mut rng, sr, 0.11, 800.0, 2600.0, 1.8, 0.7),
        SoundId::PlaceDirt => synth_impact(&mut rng, sr, 0.14, 180.0, 900.0, 0.03, 0.85),
        SoundId::PlaceStone => synth_impact(&mut rng, sr, 0.12, 280.0, 1800.0, 0.02, 0.9),
        SoundId::PlaceWood => synth_wood_step(&mut rng, sr, 0.16, 0.90),
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
        SoundId::UiClick => synth_click(sr, 0.04, 1000.0, 0.85),
        SoundId::UiHover => synth_click(sr, 0.02, 1400.0, 0.35),
        SoundId::ChestOpen => synth_chest_creak(sr, 0.28, true, 0.8),
        SoundId::ChestClose => synth_chest_creak(sr, 0.25, false, 0.8),
        SoundId::DoorOpen => synth_door_swing(sr, 0.22, true, 0.85),
        SoundId::DoorClose => synth_door_swing(sr, 0.20, false, 0.85),
        SoundId::Explosion => synth_explosion(&mut rng, sr, 1.4, 1.0),

        // Mob sounds (Pig, Cow, Chicken, Zombie, Skeleton)
        SoundId::PigIdle => synth_pig_idle(&mut rng, sr, 0.28, 0.85),
        SoundId::PigHurt => synth_pig_hurt(&mut rng, sr, 0.28, 0.95),
        SoundId::PigDeath => synth_pig_death(&mut rng, sr, 0.65, 1.0),
        SoundId::CowIdle => synth_cow_idle(&mut rng, sr, 1.05, 0.85),
        SoundId::CowHurt => synth_cow_hurt(&mut rng, sr, 0.36, 0.95),
        SoundId::CowDeath => synth_cow_death(&mut rng, sr, 0.90, 1.0),
        SoundId::ChickenIdle => synth_chicken_idle(&mut rng, sr, 0.28, 0.80),
        SoundId::ChickenHurt => synth_chicken_hurt(&mut rng, sr, 0.22, 0.95),
        SoundId::ChickenDeath => synth_chicken_death(&mut rng, sr, 0.55, 1.0),
        SoundId::ZombieIdle => synth_zombie_idle(&mut rng, sr, 0.85, 0.85),
        SoundId::ZombieHurt => synth_zombie_hurt(&mut rng, sr, 0.35, 0.95),
        SoundId::ZombieDeath => synth_zombie_death(&mut rng, sr, 0.95, 1.0),
        SoundId::SkeletonIdle => synth_skeleton_idle(&mut rng, sr, 0.40, 0.80),
        SoundId::SkeletonHurt => synth_skeleton_hurt(&mut rng, sr, 0.25, 0.95),
        SoundId::SkeletonDeath => synth_skeleton_death(&mut rng, sr, 0.70, 1.0),
        SoundId::CreeperHiss => synth_creeper_hiss(&mut rng, sr, 1.45, 0.90),
        SoundId::CreeperHurt => synth_creeper_hurt(&mut rng, sr, 0.22, 0.90),
        SoundId::CreeperDeath => synth_creeper_death(&mut rng, sr, 0.45, 0.95),
        SoundId::SheepIdle => synth_sheep_idle(&mut rng, sr, 0.75, 0.85),
        SoundId::SheepHurt => synth_sheep_hurt(&mut rng, sr, 0.28, 0.95),
        SoundId::SheepDeath => synth_sheep_death(&mut rng, sr, 0.70, 0.95),
        SoundId::SheepShear => synth_shears(&mut rng, sr, 0.15, 0.90),
        SoundId::VillagerIdle => synth_villager_idle(&mut rng, sr, 0.45, 0.90),
        SoundId::VillagerHurt => synth_villager_hurt(&mut rng, sr, 0.22, 0.95),
        SoundId::VillagerDeath => synth_villager_death(&mut rng, sr, 0.65, 0.95),
        SoundId::VillagerTrade => synth_villager_trade(&mut rng, sr, 0.38, 0.90),
        SoundId::VillagerNo => synth_villager_no(&mut rng, sr, 0.32, 0.90),
        SoundId::IronGolemIdle => synth_iron_golem_idle(&mut rng, sr, 0.40, 0.85),
        SoundId::IronGolemHurt => synth_iron_golem_hurt(&mut rng, sr, 0.35, 0.95),
        SoundId::IronGolemDeath => synth_iron_golem_death(&mut rng, sr, 0.80, 0.95),
        SoundId::IronGolemAttack => synth_iron_golem_attack(&mut rng, sr, 0.30, 0.95),
        SoundId::FishFlop => synth_fish_flop(&mut rng, sr, 0.18, 0.85),
        SoundId::DolphinIdle => synth_dolphin_idle(&mut rng, sr, 0.45, 0.85),
        SoundId::DolphinHurt => synth_dolphin_hurt(&mut rng, sr, 0.28, 0.95),
        SoundId::DolphinDeath => synth_dolphin_death(&mut rng, sr, 0.75, 0.95),
        SoundId::DolphinJump => synth_dolphin_jump(&mut rng, sr, 0.40, 0.90),
        SoundId::FishingCast => synth_fishing_cast(sr, 0.16, 0.85),
        SoundId::FishingReel => synth_fishing_reel(&mut rng, sr, 0.22, 0.85),
        SoundId::FishCatch => synth_fish_catch(&mut rng, sr, 0.35, 0.95),

        // Ambient (loops or long tones)
        SoundId::AmbientWind => synth_wind_ambient(&mut rng, sr, 4.5, 0.45),
        SoundId::AmbientCave => synth_cave_chord(&mut rng, sr, 4.0, 0.45),
        SoundId::AmbientWater => synth_water_ambient(&mut rng, sr, 3.8, 0.50),
        SoundId::AmbientLeaves => synth_leaves_rustle(&mut rng, sr, 3.2, 0.45),
        SoundId::AmbientCaveRocks => synth_cave_rocks(&mut rng, sr, 2.2, 0.65),
        SoundId::AmbientDesertSand => synth_desert_sand(&mut rng, sr, 3.5, 0.45),
        SoundId::Rain => synth_rain_ambient(&mut rng, sr, 3.8, 0.50),
        SoundId::ThunderClose => synth_thunder(&mut rng, sr, true, 0.90),
        SoundId::ThunderDistant => synth_thunder(&mut rng, sr, false, 0.80),
        SoundId::BowShoot => synth_bow_shoot(&mut rng, sr, 0.22, 0.90),
        SoundId::BowHit => synth_bow_hit(&mut rng, sr, 0.12, 0.95),
        SoundId::ShieldBlock => synth_shield_block(&mut rng, sr, 0.28, 0.95),
        SoundId::ArmorEquip => synth_armor_equip(&mut rng, sr, 0.25, 0.90),
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

/// Minecraft-style stone block break: heavy fracture thump + sharp crack + multi-grain debris scatter.
fn synth_stone_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.32)) as usize;
    let mut out = vec![0.0f32; count];
    let mut bp_crack = BiquadFilter::bandpass(2400.0, 2.2, sr);
    let mut bp_rumble = BiquadFilter::bandpass(140.0, 1.5, sr);

    // 1. Initial heavy fracture snap (high impact thump + rock crack)
    let snap_len = (sr * 0.08) as usize;
    let mut phase = 0.0f32;
    for i in 0..snap_len.min(count) {
        let t = i as f32 / sr;
        let freq = 180.0 - t * 1400.0;
        phase += 2.0 * PI * freq.max(50.0) / sr;
        let thump = phase.sin() * (-t * 45.0).exp() * 0.85;
        let raw_noise = rng.next_f32();
        let crack = bp_crack.process(raw_noise) * (-t * 60.0).exp() * 1.4;
        let body = bp_rumble.process(raw_noise) * (-t * 30.0).exp() * 0.6;
        out[i] += (thump + crack + body) * volume;
    }

    // 2. Debris scatter grains (6 discrete rocky chips scattering)
    let grain_times = [0.035, 0.070, 0.115, 0.165, 0.220, 0.280];
    let grain_amps = [0.85, 0.70, 0.55, 0.42, 0.30, 0.18];
    let grain_freqs = [2800.0, 2100.0, 3200.0, 1800.0, 2500.0, 1500.0];

    for g in 0..grain_times.len() {
        let start_idx = (grain_times[g] * sr) as usize;
        let g_len = (sr * 0.035) as usize;
        let amp = grain_amps[g];
        let mut g_bp = BiquadFilter::bandpass(grain_freqs[g], 3.0, sr);
        for i in 0..g_len {
            let idx = start_idx + i;
            if idx >= count { break; }
            let t = i as f32 / g_len as f32;
            let env = (1.0 - t).powi(2);
            let s = g_bp.process(rng.next_f32()) * env * amp;
            out[idx] += s * volume;
        }
    }

    out
}

/// Minecraft-style wood break: hollow board splitting + fibrous splinter crackles.
fn synth_wood_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.30)) as usize;
    let mut out = vec![0.0f32; count];
    let mut bp_splinter = BiquadFilter::bandpass(1650.0, 1.8, sr);

    // 1. Initial hollow board fracture snap + tone bend
    let snap_len = (sr * 0.10) as usize;
    let mut phase1 = 0.0f32;
    let mut phase2 = 0.0f32;
    for i in 0..snap_len.min(count) {
        let t = i as f32 / sr;
        let freq = (210.0 - t * 800.0).max(90.0);
        phase1 += 2.0 * PI * freq / sr;
        phase2 += 2.0 * PI * (freq * 2.1) / sr;
        let wood_tone = (phase1.sin() * 0.6 + phase2.sin() * 0.4) * (-t * 35.0).exp();
        let splinter = bp_splinter.process(rng.next_f32()) * (-t * 40.0).exp() * 1.2;
        out[i] += (wood_tone + splinter) * volume;
    }

    // 2. Fibrous splinter crackles
    let splinter_times = [0.03, 0.065, 0.11, 0.16, 0.22];
    let splinter_amps = [0.75, 0.60, 0.45, 0.32, 0.20];
    let splinter_freqs = [1900.0, 1400.0, 2200.0, 1700.0, 1200.0];

    for s_idx in 0..splinter_times.len() {
        let start_idx = (splinter_times[s_idx] * sr) as usize;
        let s_len = (sr * 0.03) as usize;
        let amp = splinter_amps[s_idx];
        let mut s_bp = BiquadFilter::bandpass(splinter_freqs[s_idx], 2.5, sr);
        for i in 0..s_len {
            let idx = start_idx + i;
            if idx >= count { break; }
            let t = i as f32 / s_len as f32;
            let env = (1.0 - t).powi(2);
            let s = s_bp.process(rng.next_f32()) * env * amp;
            out[idx] += s * volume;
        }
    }

    out
}

/// Minecraft-style dirt break: heavy low-frequency thump + tumbling soil clod grains.
fn synth_dirt_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.28)) as usize;
    let mut out = vec![0.0f32; count];
    let mut lp = IirFilter::lowpass(850.0, sr);
    let mut hp = IirFilter::highpass(180.0, sr);

    // Initial soft heavy earthy thud
    let thud_len = (sr * 0.09) as usize;
    let mut phase = 0.0f32;
    for i in 0..thud_len.min(count) {
        let t = i as f32 / sr;
        phase += 2.0 * PI * 85.0 / sr;
        let thud = phase.sin() * (-t * 40.0).exp() * 0.9;
        let crunch = lp.process(hp.process(rng.next_f32())) * (-t * 35.0).exp() * 1.1;
        out[i] += (thud + crunch) * volume;
    }

    // Granular soil clod scatter
    let clod_times = [0.035, 0.075, 0.12, 0.17, 0.22];
    let clod_amps = [0.65, 0.50, 0.38, 0.26, 0.15];
    for c in 0..clod_times.len() {
        let start_idx = (clod_times[c] * sr) as usize;
        let c_len = (sr * 0.04) as usize;
        let mut c_lp = IirFilter::lowpass(650.0, sr);
        let mut c_hp = IirFilter::highpass(220.0, sr);
        let amp = clod_amps[c];
        for i in 0..c_len {
            let idx = start_idx + i;
            if idx >= count { break; }
            let t = i as f32 / c_len as f32;
            let env = (1.0 - t).powi(2);
            let s = c_lp.process(c_hp.process(rng.next_f32())) * env * amp;
            out[idx] += s * volume;
        }
    }

    out
}

/// Minecraft-style grass break: leafy tearing crunch + soft soil base.
fn synth_grass_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.26)) as usize;
    let mut out = vec![0.0f32; count];
    let mut hp_leaf = IirFilter::highpass(1400.0, sr);
    let mut lp_leaf = IirFilter::lowpass(4200.0, sr);
    let mut lp_earth = IirFilter::lowpass(400.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let leaf_env = (1.0 - t).powf(1.8) * 0.85;
        let raw = rng.next_f32();
        let leaf_crunch = lp_leaf.process(hp_leaf.process(raw)) * leaf_env * 1.1;
        let earth_thump = lp_earth.process(raw) * (-t * 12.0).exp() * 0.45;
        out[i] = (leaf_crunch + earth_thump) * volume;
    }

    out
}

/// Minecraft-style glass shattering: instant crystalline snap + resonant chimes + shard tinkles.
fn synth_glass_shatter(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.38)) as usize;
    let mut out = vec![0.0f32; count];
    let mut hp = IirFilter::highpass(2200.0, sr);
    let mut lp = IirFilter::lowpass(9500.0, sr);

    // 1. Initial sharp crystal snap transient
    let snap_len = (sr * 0.015) as usize;
    for i in 0..snap_len.min(count) {
        let t = i as f32 / snap_len as f32;
        let snap = lp.process(hp.process(rng.next_f32())) * (1.0 - t).powi(3) * 1.8;
        out[i] += snap * volume;
    }

    // 2. Resonant metallic chimes
    let chime_len = (sr * 0.32) as usize;
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut p3 = 0.0f32;
    for i in 0..chime_len.min(count) {
        let t = i as f32 / sr;
        p1 += 2.0 * PI * 3150.0 / sr;
        p2 += 2.0 * PI * 5200.0 / sr;
        p3 += 2.0 * PI * 7800.0 / sr;
        let env = (-t * 12.0).exp();
        let chimes = (p1.sin() * 0.4 + p2.sin() * 0.35 + p3.sin() * 0.25) * env * 0.7;
        out[i] += chimes * volume;
    }

    // 3. Shard tinkles scattering
    let tinkle_times = [0.03, 0.07, 0.12, 0.17, 0.23, 0.29, 0.34];
    let tinkle_amps = [0.7, 0.6, 0.5, 0.4, 0.3, 0.22, 0.15];
    let tinkle_freqs = [4200.0, 6800.0, 3900.0, 8100.0, 5400.0, 7200.0, 4800.0];

    for k in 0..tinkle_times.len() {
        let start_idx = (tinkle_times[k] * sr) as usize;
        let t_len = (sr * 0.02) as usize;
        let mut t_bp = BiquadFilter::bandpass(tinkle_freqs[k], 4.0, sr);
        let amp = tinkle_amps[k];
        for i in 0..t_len {
            let idx = start_idx + i;
            if idx >= count { break; }
            let t = i as f32 / t_len as f32;
            let env = (1.0 - t).powi(2);
            let s = t_bp.process(rng.next_f32()) * env * amp;
            out[idx] += s * volume;
        }
    }

    out
}

/// Minecraft-style sand break: soft crumbling collapse + raspy granular friction.
fn synth_sand_break(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.26)) as usize;
    let mut out = vec![0.0f32; count];
    let mut hp = IirFilter::highpass(500.0, sr);
    let mut lp = IirFilter::lowpass(2800.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(1.5);
        let raw = rng.next_f32();
        let friction = lp.process(hp.process(raw)) * env * 1.1;
        out[i] = friction * volume;
    }
    out
}

/// Punchy stone digging / hitting tap (pickaxe/fist impact against rock).
fn synth_stone_dig(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.065)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(2400.0, 2.5, sr);
    let mut phase = 0.0f32;
    for i in 0..count {
        let t = i as f32 / sr;
        phase += 2.0 * PI * 130.0 / sr;
        let thump = phase.sin() * (-t * 60.0).exp() * 0.7;
        let crack = bp.process(rng.next_f32()) * (-t * 80.0).exp() * 1.3;
        out.push((thump + crack) * volume);
    }
    out
}

/// Punchy wood digging / hitting tap (axe/fist hollow knock).
fn synth_wood_dig(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.075)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(1700.0, 2.0, sr);
    let mut phase = 0.0f32;
    for i in 0..count {
        let t = i as f32 / sr;
        phase += 2.0 * PI * 180.0 / sr;
        let knock = phase.sin() * (-t * 45.0).exp() * 0.75;
        let click = bp.process(rng.next_f32()) * (-t * 70.0).exp() * 1.1;
        out.push((knock + click) * volume);
    }
    out
}

/// Punchy dirt digging / hitting tap (shovel/fist earthy thud).
fn synth_dirt_dig(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.070)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(900.0, sr);
    let mut hp = IirFilter::highpass(180.0, sr);
    let mut phase = 0.0f32;
    for i in 0..count {
        let t = i as f32 / sr;
        phase += 2.0 * PI * 95.0 / sr;
        let thud = phase.sin() * (-t * 50.0).exp() * 0.75;
        let crunch = lp.process(hp.process(rng.next_f32())) * (-t * 50.0).exp() * 1.0;
        out.push((thud + crunch) * volume);
    }
    out
}

/// Punchy grass digging / hitting tap (shears/fist leafy rustle).
fn synth_grass_dig(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.070)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(3200.0, sr);
    let mut hp = IirFilter::highpass(1200.0, sr);
    for i in 0..count {
        let t = i as f32 / sr;
        let snap = lp.process(hp.process(rng.next_f32())) * (-t * 45.0).exp() * 1.1;
        out.push(snap * volume);
    }
    out
}

/// Punchy sand digging / hitting tap (shovel/fist friction scrape).
fn synth_sand_dig(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.080)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(2400.0, sr);
    let mut hp = IirFilter::highpass(600.0, sr);
    for i in 0..count {
        let t = i as f32 / sr;
        let scrape = lp.process(hp.process(rng.next_f32())) * (-t * 35.0).exp() * 1.1;
        out.push(scrape * volume);
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

/// UI click or hover impulse with crisp mechanical contact transient and fast resonant body.
fn synth_click(sr: f32, duration: f32, freq: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut phase = 0.0f32;
    let mut click_rng = 0x2A3B4C5Du32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;
        // Fast exponential decay envelope
        let env = (-cur_time * 85.0).exp();
        // Pitch drop mimicking mechanical switch contact
        let f = freq * (1.18 - 0.35 * t);
        phase += 2.0 * PI * f / sr;
        let tone = phase.sin();

        // Sharp transient snap in first ~1.5ms
        let snap_env = (-cur_time * 500.0).exp();
        click_rng = click_rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((click_rng >> 16) as f32 / 32768.0) - 1.0;
        let snap = noise * snap_env * 0.75;

        out.push(((tone * 0.65 + snap) * env * volume).clamp(-1.0, 1.0));
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

/// Tactical and resonant wood step: solid foot strike thump + hollow wooden board cavity resonance + sole friction.
fn synth_wood_step(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.12)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp_cavity = BiquadFilter::bandpass(320.0, 3.2, sr);
    let mut bp_board = BiquadFilter::bandpass(680.0, 3.5, sr);
    let mut hp_tap = IirFilter::highpass(1200.0, sr);
    let mut lp_tap = IirFilter::lowpass(3800.0, sr);

    let mut phase = 0.0f32;
    for i in 0..count {
        let t = i as f32 / sr;
        let norm_t = i as f32 / count as f32;

        // 1. Hollow plank thump (dropping pitch 180 Hz -> 85 Hz)
        let freq = (180.0 - norm_t * 95.0).max(70.0);
        phase += 2.0 * PI * freq / sr;
        let thump = (phase.sin() * 0.7 + (phase * 2.1).sin() * 0.3) * (-t * 38.0).exp();

        // 2. Resonant board acoustic modes excited by impulse
        let raw_imp = if i < 3 { 1.5 } else { rng.next_f32() * 0.3 };
        let cavity = bp_cavity.process(raw_imp) * (-t * 32.0).exp();
        let board = bp_board.process(raw_imp) * (-t * 42.0).exp();

        // 3. Crisp shoe sole tap
        let sole = lp_tap.process(hp_tap.process(rng.next_f32())) * (-t * 65.0).exp();

        let s = thump * 0.65 + cavity * 0.75 + board * 0.45 + sole * 0.40;
        out.push(s * volume);
    }
    out
}

/// Authentic wooden floorboard strain and creak under foot pressure.
/// Utilizes FM synthesis with stick-slip micro-impulses and wooden cavity formants.
fn synth_wood_creak(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.20)) as usize;
    let mut out = Vec::with_capacity(count);

    let mut bp1 = BiquadFilter::bandpass(380.0, 4.2, sr);
    let mut bp2 = BiquadFilter::bandpass(940.0, 3.6, sr);
    let mut bp_high = BiquadFilter::bandpass(1750.0, 4.0, sr);

    let mut car_phase = 0.0f32;
    let mut mod_phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Wood tension pitch bend: begins under sudden weight, groans slightly down
        let base_pitch = 320.0 - (t * 2.0 * PI).sin() * 35.0 - t * 45.0;
        let mod_ratio = 2.82; // Inharmonic wood grain acoustic ratio
        let mod_freq = base_pitch * mod_ratio;

        // Stick-slip micro friction events (wood fibers catching & slipping)
        let slip_trigger = (cur_time * 55.0 * 2.0 * PI).sin() > 0.85;
        let friction_burst = if slip_trigger { rng.next_f32() * 0.35 } else { 0.0 };

        // FM index envelope
        let mod_idx = (1.8 * (1.0 - t).powf(0.8) + friction_burst).max(0.1);

        mod_phase += 2.0 * PI * mod_freq / sr;
        let mod_signal = mod_phase.sin() * mod_idx;

        car_phase += 2.0 * PI * base_pitch / sr;
        let raw_fm = (car_phase + mod_signal).sin();

        // Pass through wood formant resonators
        let body = bp1.process(raw_fm) * 0.70 + bp2.process(raw_fm) * 0.45 + bp_high.process(raw_fm + friction_burst) * 0.25;

        // Natural smooth swell & decay
        let env = (t * PI).sin().powf(0.7) * (1.0 - t * 0.3);

        // Soft analog warmth / saturation
        let saturated = body / (1.0 + body.abs() * 0.4);
        out.push(saturated * env * volume * 1.3);
    }
    out
}

/// Atmospheric wind gust with organic multi-band air turbulence (natural rushing szum, no whistle).
fn synth_wind_ambient(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(3.0)) as usize;
    let mut out = Vec::with_capacity(count);

    let mut lp_sub = IirFilter::lowpass(180.0, sr);
    let mut hp_sub = IirFilter::highpass(32.0, sr);
    let mut bp_mid = BiquadFilter::bandpass(380.0, 1.2, sr);
    let mut bp_high = BiquadFilter::bandpass(680.0, 1.4, sr);
    let mut lp_hiss = IirFilter::lowpass(1100.0, sr);

    // Running noise accumulator for pink/brown warm drift
    let mut brown = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;

        // Natural wind gust envelope: gentle rise, subtle secondary gust, smooth fade
        let base_env = (t * PI).sin().powf(1.3);
        let gust_mod = 1.0 + 0.28 * (t * PI * 3.5).sin() * (t * PI).sin();
        let env = (base_env * gust_mod).clamp(0.0, 1.4);

        let raw = rng.next_f32();
        brown = (brown * 0.94 + raw * 0.06).clamp(-1.0, 1.0);

        // 1. Deep atmospheric pressure rumble (szum niskotonowy)
        let sub_rumble = hp_sub.process(lp_sub.process(raw + brown * 0.5)) * 1.6;

        // 2. Mid-frequency wind gust body (szum powietrza)
        let mid_rush = bp_mid.process(raw) * 1.3;

        // 3. Higher breeze rustle (delikatny powiew liści/koron drzew)
        let high_breeze = bp_high.process(lp_hiss.process(raw)) * 0.75;

        let composite = (sub_rumble * 0.65 + mid_rush * 0.85 + high_breeze * 0.50) * env;
        // Soft saturate to prevent harsh peaks
        let saturated = composite.clamp(-1.0, 1.0);
        out.push(saturated * volume * 1.4);
    }
    out
}

/// Rich subterranean cavern drone with deep sub-bass and ominous FM beating.
fn synth_cave_chord(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(3.0)) as usize;
    let mut out = Vec::with_capacity(count);

    let f0 = 55.0;  // A1 sub
    let f1 = 65.41; // C2
    let f2 = 77.78; // Eb2
    let f3 = 98.00; // G2
    let mut p0 = 0.0f32;
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut p3 = 0.0f32;
    let mut lp_cave = IirFilter::lowpass(480.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (t * PI).sin().powf(0.85);

        // Subtle slow frequency drifting (subterranean draft)
        let drift = (i as f32 / sr * 0.3 * 2.0 * PI).sin() * 0.8;
        p0 += 2.0 * PI * (f0 + drift) / sr;
        p1 += 2.0 * PI * (f1 - drift * 0.5) / sr;
        p2 += 2.0 * PI * (f2 + drift * 0.7) / sr;
        p3 += 2.0 * PI * (f3 + drift) / sr;

        // Subterranean hollow breeze noise
        let cavern_draft = lp_cave.process(rng.next_f32()) * 0.15;

        let tone = p0.sin() * 0.35 + p1.sin() * 0.30 + p2.sin() * 0.22 + p3.sin() * 0.18 + cavern_draft;
        out.push(tone * env * volume * 1.3);
    }
    out
}

/// Gentle flowing water current with organic bubbling brook droplet pops.
fn synth_water_ambient(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(3.0)) as usize;
    let mut out = vec![0.0f32; count];

    let mut bp_stream = BiquadFilter::bandpass(420.0, 1.4, sr);
    let mut lp_stream = IirFilter::lowpass(750.0, sr);

    // 1. Continuous stream turbulence with undulating ripples
    for i in 0..count {
        let t = i as f32 / sr;
        let norm_t = i as f32 / count as f32;
        let ripple = 0.75 + 0.25 * (t * 2.8 * 2.0 * PI).sin() * (t * 0.7 * 2.0 * PI).cos();
        let raw_noise = rng.next_f32();
        let filtered = lp_stream.process(bp_stream.process(raw_noise));
        let env = (norm_t * PI).sin().powf(0.4);
        out[i] = filtered * ripple * env * 0.55;
    }

    // 2. Organic FM water droplet / bubble pops
    let bubble_times = [0.35, 0.78, 1.25, 1.72, 2.15, 2.65, 3.10, 3.48];
    let bubble_freqs = [620.0, 840.0, 520.0, 950.0, 710.0, 880.0, 600.0, 760.0];

    for (b_idx, &bt) in bubble_times.iter().enumerate() {
        let start = (bt * sr) as usize;
        let len = (sr * 0.045) as usize;
        let base_f = bubble_freqs[b_idx];
        let mut b_phase = 0.0f32;

        for i in 0..len {
            let idx = start + i;
            if idx >= count { break; }
            let tau = i as f32 / sr;
            // Rising chirp characteristic of water bubbles
            let f = base_f + (base_f * 0.8) * (1.0 - (-tau * 85.0).exp());
            b_phase += 2.0 * PI * f / sr;
            let env = (-tau * 55.0).exp();
            let pop = b_phase.sin() * env * 0.65;
            out[idx] += pop;
        }
    }

    // Apply master volume and soft saturation
    for sample in out.iter_mut() {
        let s = *sample;
        *sample = (s / (1.0 + s.abs() * 0.35)) * volume;
    }

    out
}

/// Tree leaves rustling in the wind: multi-layered foliage flutter and air movement.
fn synth_leaves_rustle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(2.5)) as usize;
    let mut out = Vec::with_capacity(count);

    let mut bp_leaf1 = BiquadFilter::bandpass(2800.0, 1.8, sr);
    let mut bp_leaf2 = BiquadFilter::bandpass(5200.0, 2.2, sr);
    let mut bp_branch = BiquadFilter::bandpass(480.0, 2.8, sr);
    let mut hp_leaf = IirFilter::highpass(1600.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Wind gust envelope: swells, flutters, gently dies down
        let swell = (t * PI).sin().powf(1.2);

        // Foliage micro-flutter (leaves fluttering against each other at 14-22 Hz)
        let flutter = 0.55 + 0.30 * (cur_time * 16.5 * 2.0 * PI).sin()
            + 0.15 * (cur_time * 23.0 * 2.0 * PI).cos();

        let raw = rng.next_f32();
        let high_rustle = bp_leaf1.process(hp_leaf.process(raw)) * 0.65
            + bp_leaf2.process(raw) * 0.40;
        let branch_hum = bp_branch.process(raw) * 0.20;

        let composite = (high_rustle * flutter + branch_hum) * swell;
        out.push(composite * volume * 1.35);
    }
    out
}

/// Falling pebbles and rocks tumbling down cavern walls with echoing acoustics.
fn synth_cave_rocks(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(1.8)) as usize;
    let mut out = vec![0.0f32; count];

    // Accelerating bounce pattern of tumbling pebbles
    let impacts = [
        (0.04f32, 1800.0f32, 0.90f32),
        (0.16f32, 2400.0f32, 0.75f32),
        (0.31f32, 1500.0f32, 0.65f32),
        (0.44f32, 2900.0f32, 0.55f32),
        (0.55f32, 2100.0f32, 0.45f32),
        (0.63f32, 2600.0f32, 0.35f32),
        (0.70f32, 3300.0f32, 0.25f32),
    ];

    for &(it, freq, amp) in &impacts {
        let start = (it * sr) as usize;
        let len = (sr * 0.035) as usize;
        let mut bp = BiquadFilter::bandpass(freq, 3.5, sr);
        let mut phase_thump = 0.0f32;

        for i in 0..len {
            let idx = start + i;
            if idx >= count { break; }
            let t = i as f32 / len as f32;
            let impulse = if i < 3 { 1.6 } else { rng.next_f32() * 0.3 };
            let clack = bp.process(impulse) * (1.0 - t).powi(2);

            phase_thump += 2.0 * PI * (140.0 - t * 40.0) / sr;
            let thump = phase_thump.sin() * (1.0 - t).powi(3) * 0.4;

            out[idx] += (clack + thump) * amp;
        }
    }

    // Cavern echo: multi-tap delayed reverberation
    let echo_delays = [(0.065 * sr) as usize, (0.135 * sr) as usize, (0.220 * sr) as usize];
    let echo_gains = [0.35, 0.22, 0.14];

    for tap in 0..3 {
        let delay = echo_delays[tap];
        let gain = echo_gains[tap];
        let mut lp = IirFilter::lowpass(1400.0, sr);
        for i in delay..count {
            let delayed = out[i - delay];
            let filtered = lp.process(delayed);
            out[i] += filtered * gain;
        }
    }

    // Master gain
    for s in out.iter_mut() {
        *s *= volume * 1.2;
    }

    out
}

/// Shifting sand cascades and parched desert wind across dunes.
fn synth_desert_sand(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(2.8)) as usize;
    let mut out = Vec::with_capacity(count);

    let mut hp_wind = IirFilter::highpass(780.0, sr);
    let mut bp_sand1 = BiquadFilter::bandpass(2400.0, 1.8, sr);
    let mut bp_sand2 = BiquadFilter::bandpass(4600.0, 2.4, sr);
    let mut warm_phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Sand trickle cascade envelope: small start, swells to cascade, settles
        let cascade_env = (t * PI).sin().powf(1.6);

        // Granular silica micro-shimmer (individual grains sliding against each other)
        let grain_shimmer = 0.65 + 0.35 * (cur_time * 42.0 * 2.0 * PI).sin();

        let raw = rng.next_f32();
        let desert_breeze = hp_wind.process(raw) * 0.35;
        let sand_cascade = (bp_sand1.process(raw) * 0.60 + bp_sand2.process(raw) * 0.40) * grain_shimmer;

        // Warm low desert drone harmonic (Mega Drive FM vibe)
        warm_phase += 2.0 * PI * 110.0 / sr;
        let desert_heat = (warm_phase.sin() * 0.08) * (1.0 - t * 0.5);

        let composite = (desert_breeze + sand_cascade + desert_heat) * cascade_env;
        out.push(composite * volume * 1.3);
    }
    out
}

/// Pig ambient idle snort / grunt: throaty low nasal burst with AM roughness.
fn synth_pig_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.25)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(480.0, 2.5, sr);
    let mut bp2 = BiquadFilter::bandpass(1100.0, 2.0, sr);
    let mut lp_breath = IirFilter::lowpass(750.0, sr);
    let mut hp_breath = IirFilter::highpass(220.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;
        // Pitch drops slightly: 135 Hz down to 110 Hz
        let freq = 135.0 - t * 25.0;
        phase += 2.0 * PI * freq / sr;

        // Snort roughness modulation (28 Hz)
        let roughness = 0.45 + 0.55 * (cur_time * 28.0 * 2.0 * PI).sin();

        // Buzzy harmonic vocal source
        let vocal_source = (phase.sin() * 0.5 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.2) * roughness;
        let formant = bp1.process(vocal_source) * 0.6 + bp2.process(vocal_source) * 0.4;

        // Breathy nasal turbulence
        let raw_noise = rng.next_f32();
        let breath = hp_breath.process(lp_breath.process(raw_noise)) * roughness * 0.25;

        // Envelope: quick attack (0.02s), exponential decay
        let env = if cur_time < 0.02 {
            cur_time / 0.02
        } else {
            (-t * 5.0).exp()
        };

        out.push((formant * 1.8 + breath) * env * volume);
    }
    out
}

/// Pig hurt squeal: sharp, piercing, high-pitched screech with rapid FM vibrato.
fn synth_pig_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.24)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(900.0, 2.8, sr);
    let mut hp_screech = IirFilter::highpass(1600.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Pitch trajectory: starts at 780 Hz, leaps to 920 Hz, then slides to 640 Hz
        let base_freq = if t < 0.25 {
            780.0 + (t / 0.25) * 140.0
        } else {
            920.0 - ((t - 0.25) / 0.75) * 280.0
        };

        // Fast FM squeal vibrato (38 Hz)
        let vib = (cur_time * 38.0 * 2.0 * PI).sin() * 40.0;
        let freq = (base_freq + vib).max(200.0);
        phase += 2.0 * PI * freq / sr;

        // Saturated harmonics for piercing bite
        let tone = (phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.65 + (phase * 2.0).sin() * 0.35;
        let filtered_tone = bp.process(tone);

        // Breath squeak
        let noise = hp_screech.process(rng.next_f32()) * 0.18;

        // Sharp immediate attack (0.008s), exponential decay
        let env = if cur_time < 0.008 {
            cur_time / 0.008
        } else {
            (-t * 9.5).exp()
        };

        out.push((filtered_tone * 1.6 + noise) * env * volume);
    }
    out
}

/// Pig death cry: agonizing long dying squeal sliding from panic high to deep throat death rattle.
fn synth_pig_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.60)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(950.0, 2.4, sr);
    let mut bp2 = BiquadFilter::bandpass(450.0, 2.0, sr);
    let mut lp_rattle = IirFilter::lowpass(650.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Screams high for first 0.15s, then agonizing downward slide to 220 Hz
        let base_freq = if cur_time < 0.15 {
            920.0 + (cur_time / 0.15) * 60.0
        } else {
            let rel = (cur_time - 0.15) / (duration - 0.15);
            980.0 - rel.powf(0.85) * 740.0
        };

        // Agonized panic vibrato and rattle
        let vib = (cur_time * 26.0 * 2.0 * PI).sin() * 55.0;
        phase += 2.0 * PI * (base_freq + vib).max(120.0) / sr;

        let tone = (phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let formant = bp1.process(tone) * (1.0 - t * 0.5) + bp2.process(tone) * (t * 0.7);

        // Death rattle noise increases towards end
        let rattle = lp_rattle.process(rng.next_f32()) * (t * 0.35);

        let env = if cur_time < 0.01 {
            cur_time / 0.01
        } else {
            (1.0 - t).powf(1.4)
        };

        out.push((formant * 1.5 + rattle) * env * volume);
    }
    out
}

/// Cow ambient idle moo: warm, deep, resonant bovine vocalization ("Mmm-Ooooo").
fn synth_cow_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.95)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(380.0, 2.2, sr);
    let mut bp2 = BiquadFilter::bandpass(780.0, 2.0, sr);
    let mut lp_closed = BiquadFilter::lowpass(320.0, 1.4, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Cow moo pitch: starts ~120 Hz, rises slightly to 132 Hz during open moo, then settles to 105 Hz
        let freq = if t < 0.20 {
            118.0 + (t / 0.20) * 14.0
        } else {
            132.0 - ((t - 0.20) / 0.80) * 27.0
        };

        // Gentle bovine vocal vibrato (4.2 Hz)
        let vib = (cur_time * 4.2 * 2.0 * PI).sin() * 3.5;
        phase += 2.0 * PI * (freq + vib) / sr;

        // Harmonic series (warm buzzy vocal cords)
        let vocal_cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.28
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.05;

        // Transition from closed mouth "Mmm" (first 0.18s) to open mouth "Oooo"
        let open_blend = ((cur_time - 0.12) / 0.18).clamp(0.0, 1.0);
        let closed_sound = lp_closed.process(vocal_cords);
        let open_sound = bp1.process(vocal_cords) * 0.65 + bp2.process(vocal_cords) * 0.35;
        let vocal = closed_sound * (1.0 - open_blend) + open_sound * open_blend * 1.5;

        // Gentle breathiness
        let breath = (rng.next_f32() * 0.05) * open_blend;

        // Envelope: smooth rise over 0.15s, sustained body, gentle fade out
        let env = if cur_time < 0.15 {
            (cur_time / 0.15).powf(0.8)
        } else if t > 0.70 {
            ((1.0 - t) / 0.30).powf(1.5)
        } else {
            1.0
        };

        out.push((vocal * 1.6 + breath) * env * volume);
    }
    out
}

/// Cow hurt: sudden painful low-mid grunt / hurt groan.
fn synth_cow_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.32)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(420.0, 2.0, sr);
    let mut lp = IirFilter::lowpass(600.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Sudden hurt downward pitch sweep: 195 Hz -> 105 Hz
        let freq = 195.0 - t * 90.0;
        phase += 2.0 * PI * freq / sr;

        let tone = (phase.sin() * 1.6).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let filtered = bp.process(tone);
        let grunt_noise = lp.process(rng.next_f32()) * 0.25;

        // Sudden attack (0.015s), fast decay
        let env = if cur_time < 0.015 {
            cur_time / 0.015
        } else {
            (-t * 8.5).exp()
        };

        out.push((filtered * 1.7 + grunt_noise) * env * volume);
    }
    out
}

/// Cow death: deep, mournful dying bovine groan.
fn synth_cow_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.85)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(360.0, 1.8, sr);
    let mut lp_sub = IirFilter::lowpass(180.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Deep descending groan: 145 Hz sliding down to 60 Hz
        let freq = (145.0 - t.powf(0.8) * 85.0).max(50.0);
        let vib = (cur_time * 3.5 * 2.0 * PI).sin() * 4.0;
        phase += 2.0 * PI * (freq + vib) / sr;

        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.1;
        let vocal = bp1.process(tone);
        let death_sub = lp_sub.process(rng.next_f32()) * (0.3 * (1.0 - t));

        let env = if cur_time < 0.02 {
            cur_time / 0.02
        } else {
            (1.0 - t).powf(1.6)
        };

        out.push((vocal * 1.8 + death_sub) * env * volume);
    }
    out
}

/// Chicken idle cluck: authentic double cluck ("bok... bawk!").
fn synth_chicken_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.26)) as usize;
    let mut out = vec![0.0f32; count];

    // Two rapid clucks at t = 0.0 and t = 0.11
    let cluck_starts = [0.0f32, 0.11f32];
    let cluck_freqs = [(740.0, 480.0), (660.0, 390.0)];
    let cluck_durs = [0.07f32, 0.09f32];

    for idx in 0..2 {
        let start_sample = (cluck_starts[idx] * sr) as usize;
        let dur_samples = (cluck_durs[idx] * sr) as usize;
        let (f_start, f_end) = cluck_freqs[idx];
        let mut bp = BiquadFilter::bandpass(620.0, 3.2, sr);
        let mut phase = 0.0f32;

        for s in 0..dur_samples {
            let out_idx = start_sample + s;
            if out_idx >= count { break; }
            let t = s as f32 / dur_samples as f32;
            let cur_time = s as f32 / sr;

            let freq = f_start - t * (f_start - f_end);
            phase += 2.0 * PI * freq / sr;

            // Beak click at start (first 0.006s)
            let click = if cur_time < 0.006 {
                rng.next_f32() * 0.4 * (1.0 - cur_time / 0.006)
            } else {
                0.0
            };

            let tone = phase.sin() * 0.8 + (phase * 2.0).sin() * 0.2;
            let vocal = bp.process(tone);
            let env = (1.0 - t).powf(1.8);

            out[out_idx] += (vocal * 1.5 + click) * env * volume;
        }
    }

    out
}

/// Chicken hurt: sharp, loud distress squawk!
fn synth_chicken_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.20)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(1050.0, 2.5, sr);
    let mut hp_flutter = IirFilter::highpass(1800.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Downward distress chirp: 1200 Hz down to 620 Hz
        let base_freq = 1200.0 - t * 580.0;
        let flutter = (cur_time * 48.0 * 2.0 * PI).sin() * 50.0;
        let freq = (base_freq + flutter).max(250.0);
        phase += 2.0 * PI * freq / sr;

        let tone = (phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.7 + (phase * 2.0).sin() * 0.3;
        let vocal = bp.process(tone);
        let flutter_noise = hp_flutter.process(rng.next_f32()) * 0.22;

        let env = if cur_time < 0.008 {
            cur_time / 0.008
        } else {
            (-t * 11.0).exp()
        };

        out.push((vocal * 1.6 + flutter_noise) * env * volume);
    }
    out
}

/// Chicken death: frantic squawk with fluttering wings dying out.
fn synth_chicken_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.52)) as usize;
    let mut out = vec![0.0f32; count];

    // 1. Initial dying screech (0.0 to 0.32s)
    let cry_len = (sr * 0.32) as usize;
    let mut bp = BiquadFilter::bandpass(980.0, 2.2, sr);
    let mut phase = 0.0f32;
    for i in 0..cry_len.min(count) {
        let t = i as f32 / cry_len as f32;
        let cur_time = i as f32 / sr;
        let freq = (1300.0 - t.powf(0.8) * 920.0).max(200.0);
        phase += 2.0 * PI * freq / sr;

        let tone = (phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.75;
        let vocal = bp.process(tone);
        let env = if cur_time < 0.01 {
            cur_time / 0.01
        } else {
            (1.0 - t).powf(1.8)
        };
        out[i] += vocal * env * volume * 1.4;
    }

    // 2. Wing flapping noise puffs (at t = 0.06, 0.16, 0.26, 0.36)
    let flap_times = [0.06f32, 0.16f32, 0.26f32, 0.36f32];
    let flap_amps = [0.55f32, 0.45f32, 0.35f32, 0.20f32];
    for f in 0..flap_times.len() {
        let start_idx = (flap_times[f] * sr) as usize;
        let f_len = (sr * 0.045) as usize;
        let mut lp_flap = IirFilter::lowpass(800.0, sr);
        let mut hp_flap = IirFilter::highpass(220.0, sr);
        let amp = flap_amps[f];

        for i in 0..f_len {
            let idx = start_idx + i;
            if idx >= count { break; }
            let t = i as f32 / f_len as f32;
            let env = (1.0 - t).powi(2);
            let s = lp_flap.process(hp_flap.process(rng.next_f32())) * env * amp;
            out[idx] += s * volume;
        }
    }

    out
}

/// Zombie idle: deep eerie Minecraft zombie groan/moan.
fn synth_zombie_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.6)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(380.0, 3.0, sr);
    let mut bp2 = BiquadFilter::bandpass(820.0, 2.5, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Fluctuating low groan pitch: 105 to 135 Hz
        let pitch_mod = (cur_time * 6.0 * std::f32::consts::PI).sin() * 12.0;
        let base_freq = 115.0 - t * 25.0 + pitch_mod;
        phase += 2.0 * std::f32::consts::PI * base_freq / sr;

        // Distorted vocal cord buzz + rasp
        let buzz = ((phase.sin() * 2.2).clamp(-1.0, 1.0) + (phase * 2.0).sin() * 0.4) * 0.7;
        let rasp = rng.next_f32() * 0.25;
        let sound = bp1.process(buzz + rasp) * 1.5 + bp2.process(buzz) * 0.8;

        // Swelling envelope
        let env = if t < 0.15 {
            t / 0.15
        } else if t > 0.75 {
            (1.0 - t) / 0.25
        } else {
            1.0
        };

        out.push(sound * env * volume);
    }
    out
}

/// Zombie hurt: aggressive guttural grunt.
fn synth_zombie_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.28)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(450.0, 2.2, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 170.0 - t * 60.0;
        phase += 2.0 * std::f32::consts::PI * freq / sr;

        let tone = (phase.sin() * 2.5).clamp(-1.0, 1.0) * 0.8 + rng.next_f32() * 0.3;
        let sound = bp.process(tone);
        let env = (-t * 6.0).exp();

        out.push(sound * env * volume * 1.4);
    }
    out
}

/// Zombie death: deep descending death groan collapsing into silence.
fn synth_zombie_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.7)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(320.0, 2.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = (145.0 - t.powf(0.8) * 85.0).max(45.0);
        phase += 2.0 * std::f32::consts::PI * freq / sr;

        let tone = (phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.7 + rng.next_f32() * 0.2;
        let sound = bp.process(tone);
        let env = (1.0 - t).powf(1.4);

        out.push(sound * env * volume * 1.3);
    }
    out
}

/// Skeleton idle: dry bone rattling clicks.
fn synth_skeleton_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.35)) as usize;
    let mut out = vec![0.0f32; count];
    let clicks = [0.03f32, 0.09f32, 0.16f32, 0.24f32];

    for &ct in &clicks {
        let start = (ct * sr) as usize;
        let len = (sr * 0.022) as usize;
        let mut bp = BiquadFilter::bandpass(2400.0, 4.0, sr);
        for i in 0..len {
            let idx = start + i;
            if idx >= count { break; }
            let t = i as f32 / len as f32;
            let imp = if i < 3 { 1.0 } else { rng.next_f32() * 0.4 };
            let val = bp.process(imp) * (1.0 - t).powi(2);
            out[idx] += val * volume * 1.5;
        }
    }
    out
}

/// Skeleton hurt: sharp high-frequency bone crack.
fn synth_skeleton_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.20)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(1900.0, 3.5, sr);
    let mut bp2 = BiquadFilter::bandpass(3400.0, 4.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let imp = if i < 4 { 1.2 } else { rng.next_f32() * 0.35 };
        let s = bp1.process(imp) * 0.7 + bp2.process(imp) * 0.6;
        let env = (-t * 14.0).exp();
        out.push(s * env * volume * 1.8);
    }
    out
}

/// Skeleton death: crumbling bones collapsing and scattering.
fn synth_skeleton_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.6)) as usize;
    let mut out = vec![0.0f32; count];
    let drops = [0.02f32, 0.08f32, 0.15f32, 0.22f32, 0.31f32, 0.42f32];

    for (d_idx, &dt) in drops.iter().enumerate() {
        let start = (dt * sr) as usize;
        let len = (sr * 0.035) as usize;
        let center_freq = 1600.0 + (d_idx as f32 * 320.0);
        let mut bp = BiquadFilter::bandpass(center_freq, 3.2, sr);
        let amp = 1.0 - (d_idx as f32 * 0.12);

        for i in 0..len {
            let idx = start + i;
            if idx >= count { break; }
            let t = i as f32 / len as f32;
            let imp = if i < 3 { 1.0 } else { rng.next_f32() * 0.4 };
            let val = bp.process(imp) * (1.0 - t).powi(2);
            out[idx] += val * amp * volume * 1.4;
        }
    }
    out
}

/// Creeper hiss: prolonged sizzling noise with swelling envelope before detonation.
fn synth_creeper_hiss(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(1600.0, sr);
    let mut lp = IirFilter::lowpass(6800.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let swell = t.powf(0.5) * (0.65 + 0.35 * ((t * 32.0).sin() * 0.5 + 0.5));
        let raw = rng.next_f32() * 2.0 - 1.0;
        let s = lp.process(hp.process(raw));
        out.push(s * swell * volume * 1.2);
    }
    out
}

/// Creeper hurt: quick dry snap and sizzle.
fn synth_creeper_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.18)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(2100.0, 2.5, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let imp = if i < 3 { 1.2 } else { rng.next_f32() * 0.4 };
        let s = bp.process(imp);
        let env = (-t * 12.0).exp();
        out.push(s * env * volume * 1.6);
    }
    out
}

/// Creeper death: crunchy descending noise burst.
fn synth_creeper_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.4)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(1400.0, 2.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let imp = if i < 5 { 1.5 } else { rng.next_f32() * 0.5 };
        let s = bp.process(imp);
        let env = (1.0 - t).powf(1.8);
        out.push(s * env * volume * 1.5);
    }
    out
}

/// Sheep idle: warm, resonant "baa-a-a" bleat with vocal vibrato and mouth formants.
fn synth_sheep_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.65)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(540.0, 2.2, sr);
    let mut bp2 = BiquadFilter::bandpass(1180.0, 2.0, sr);
    let mut lp_throat = BiquadFilter::lowpass(420.0, 1.4, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Sheep pitch: starts around 210 Hz, slight rise to 235 Hz then falls to 185 Hz
        let base_freq = if t < 0.25 {
            205.0 + (t / 0.25) * 30.0
        } else {
            235.0 - ((t - 0.25) / 0.75) * 50.0
        };

        // Characteristic rapid ovine vocal flutter / vibrato (6.4 Hz)
        let vib = (cur_time * 6.4 * 2.0 * PI).sin() * 7.5;
        phase += 2.0 * PI * (base_freq + vib) / sr;

        // Rich buzzing vocal harmonics
        let vocal_cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.25
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.06;

        let open_blend = ((cur_time - 0.08) / 0.15).clamp(0.0, 1.0);
        let closed_sound = lp_throat.process(vocal_cords);
        let open_sound = bp1.process(vocal_cords) * 0.65 + bp2.process(vocal_cords) * 0.35;
        let vocal = closed_sound * (1.0 - open_blend) + open_sound * open_blend * 1.4;

        let breath = (rng.next_f32() * 0.04) * open_blend;

        let env = if cur_time < 0.08 {
            (cur_time / 0.08).powf(0.8)
        } else if t > 0.65 {
            ((1.0 - t) / 0.35).powf(1.6)
        } else {
            1.0
        };

        out.push((vocal * 1.5 + breath) * env * volume);
    }
    out
}

/// Sheep hurt: high-pitched, startled distress bleat.
fn synth_sheep_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.25)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(680.0, 2.0, sr);
    let mut lp = IirFilter::lowpass(800.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Sudden downward pitch sweep: 290 Hz -> 210 Hz
        let freq = 290.0 - t * 80.0;
        let vib = (cur_time * 8.5 * 2.0 * PI).sin() * 10.0;
        phase += 2.0 * PI * (freq + vib) / sr;

        let tone = (phase.sin() * 1.5).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let filtered = bp.process(tone);
        let noise = lp.process(rng.next_f32()) * 0.2;

        let env = if cur_time < 0.015 {
            cur_time / 0.015
        } else {
            (-t * 8.0).exp()
        };

        out.push((filtered * 1.6 + noise) * env * volume);
    }
    out
}

/// Sheep death: mournful, descending expiring bleat.
fn synth_sheep_death(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.65)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(520.0, 1.8, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        let freq = (220.0 - t.powf(0.7) * 110.0).max(80.0);
        let vib = (cur_time * 5.0 * 2.0 * PI).sin() * 6.0;
        phase += 2.0 * PI * (freq + vib) / sr;

        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3;
        let vocal = bp.process(tone);

        let env = if cur_time < 0.04 {
            cur_time / 0.04
        } else {
            ((1.0 - t).max(0.0)).powf(1.4)
        };

        out.push(vocal * env * volume * 1.5);
    }
    out
}

/// Shears: crisp metallic scissor snip / shearing slice.
fn synth_shears(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.12)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut hp = IirFilter::highpass(2200.0, sr);
    let mut bp_ring1 = BiquadFilter::bandpass(3400.0, 6.0, sr);
    let mut bp_ring2 = BiquadFilter::bandpass(4900.0, 5.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let cur_time = i as f32 / sr;

        // Snip friction noise
        let noise = rng.next_f32() * 2.0 - 1.0;
        let friction = hp.process(noise);

        // Dual metallic blade resonant rings
        let ring1 = bp_ring1.process(noise);
        let ring2 = bp_ring2.process(noise);

        // Quick attack, crisp scissor decay
        let env_friction = if cur_time < 0.02 {
            cur_time / 0.02
        } else {
            (-t * 18.0).exp()
        };
        let env_ring = (-t * 12.0).exp();

        let sample = friction * 0.5 * env_friction + (ring1 + ring2) * 0.8 * env_ring;
        out.push(sample * volume * 1.7);
    }
    out
}

/// Fish flop: wet slapping impact and flopping flutter on dry land.
fn synth_fish_flop(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.12)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = IirFilter::lowpass(1800.0, sr);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let flap = (t * 36.0 * std::f32::consts::PI).sin().abs();
        let noise = rng.next_f32() * 2.0 - 1.0;
        let filtered = lp.process(noise);
        let env = ((1.0 - t).max(0.0)).powf(1.5);
        out.push((filtered * 0.6 + noise * flap * 0.4) * env * volume * 1.4);
    }
    out
}

/// Dolphin idle: cheerful aquatic chirp and clicking whistle.
fn synth_dolphin_idle(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.3)) as usize;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 1800.0 + (t * 6.0 * std::f32::consts::PI).sin() * 500.0 + t * 400.0;
        let phase = i as f32 * freq * 2.0 * std::f32::consts::PI / sr;
        let chirp = phase.sin();
        let click = if (i % (sr as usize / 55)) < 4 { 0.45 } else { 0.0 };
        let env = ((1.0 - t).max(0.0)).powf(1.2);
        out.push((chirp * 0.75 + click * 0.25) * env * volume * 1.3);
    }
    out
}

/// Dolphin hurt: startled high-pitched aquatic squeak.
fn synth_dolphin_hurt(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.2)) as usize;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 2400.0 - t * 1500.0;
        let phase = i as f32 * freq * 2.0 * std::f32::consts::PI / sr;
        let env = ((1.0 - t).max(0.0)).powf(1.8);
        out.push(phase.sin() * env * volume * 1.4);
    }
    out
}

/// Dolphin death: mournful descending whistle fading out.
fn synth_dolphin_death(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.5)) as usize;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 1600.0 - t * 900.0;
        let phase = i as f32 * freq * 2.0 * std::f32::consts::PI / sr;
        let env = ((1.0 - t).max(0.0)).powf(1.3);
        out.push(phase.sin() * env * volume * 1.3);
    }
    out
}

/// Dolphin jump: energetic water splash and surge sound.
fn synth_dolphin_jump(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    synth_water_splash(rng, sr, duration, volume)
}

/// Fishing rod cast: swoosh of rod bending and line launching.
fn synth_fishing_cast(sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    synth_sweep(sr, duration, 220.0, 950.0, volume)
}

/// Fishing reel: mechanical clicks of reel turning.
fn synth_fishing_reel(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.15)) as usize;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let is_click = (i % 280) < 35;
        let click = if is_click { (rng.next_f32() * 2.0 - 1.0) * 0.8 } else { 0.0 };
        let env = (1.0 - i as f32 / count as f32).powf(1.2);
        out.push(click * env * volume);
    }
    out
}

/// Fish catch: big satisfying water splash and pop item catch!
fn synth_fish_catch(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let mut splash = synth_water_splash(rng, sr, duration, volume * 0.8);
    let pop = synth_sweep(sr, 0.12, 380.0, 880.0, volume * 0.9);
    for (i, p) in pop.into_iter().enumerate() {
        if i < splash.len() {
            splash[i] = (splash[i] + p).clamp(-1.0, 1.0);
        }
    }
    splash
}

/// Villager idle: signature Minecraft "Hrmmm" nasal hum.
fn synth_villager_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.40)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(420.0, 2.5, sr);
    let mut bp2 = BiquadFilter::bandpass(1120.0, 3.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 152.0 - t * 14.0 + (t * 8.0 * PI).sin() * 2.0;
        phase += 2.0 * PI * freq / sr;

        let raw = phase.sin() * 0.65
            + (phase * 2.0).sin() * 0.35
            + (phase * 3.0).sin() * 0.20
            + (phase * 4.0).sin() * 0.12;

        let filtered = bp1.process(raw) * 0.70 + bp2.process(raw) * 0.30;
        let env = if t < 0.12 {
            (t / 0.12).powf(0.8)
        } else if t > 0.70 {
            ((1.0 - t) / 0.30).powf(1.4)
        } else {
            1.0
        };
        let breath = (rng.next_f32() * 0.03) * env;
        out.push((filtered * env * 1.5 + breath) * volume);
    }
    out
}

/// Villager hurt: surprised, raspy high-pitch grunt "Hrrk!".
fn synth_villager_hurt(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.20)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(680.0, 2.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 175.0 + t * 55.0;
        phase += 2.0 * PI * freq / sr;

        let raw = (phase.sin() + (phase * 2.0).sin() * 0.45) + (rng.next_f32() * 0.18 - 0.09);
        let filtered = bp.process(raw);
        let env = (1.0 - t).powf(1.3);
        out.push(filtered * env * volume * 1.6);
    }
    out
}

/// Villager death: deeper, decaying nasal groan "Huuu-rgh...".
fn synth_villager_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.60)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(380.0, 2.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 155.0 - t * 65.0;
        phase += 2.0 * PI * freq / sr;

        let raw = phase.sin() * 0.7 + (phase * 2.0).sin() * 0.35 + (rng.next_f32() * 0.06 - 0.03);
        let filtered = bp.process(raw);
        let env = (1.0 - t).powf(1.5);
        out.push(filtered * env * volume * 1.5);
    }
    out
}

/// Villager trade: cheerful rising inflection "Hrmm! Yes!".
fn synth_villager_trade(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.35)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp1 = BiquadFilter::bandpass(520.0, 2.2, sr);
    let mut bp2 = BiquadFilter::bandpass(1280.0, 2.5, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let freq = 140.0 + (t.powf(1.4)) * 75.0;
        phase += 2.0 * PI * freq / sr;

        let raw = phase.sin() * 0.65 + (phase * 2.0).sin() * 0.35 + (phase * 3.0).sin() * 0.18;
        let filtered = bp1.process(raw) * 0.65 + bp2.process(raw) * 0.35;
        let env = if t < 0.10 {
            t / 0.10
        } else if t > 0.75 {
            (1.0 - t) / 0.25
        } else {
            1.0
        };
        let chime = (2.0 * PI * (660.0 + t * 440.0) * (i as f32 / sr)).sin() * 0.12 * env;
        out.push((filtered * env * 1.4 + chime) * volume);
    }
    out
}

/// Villager no: refusing double-tone grunt "Hnn-nn!".
fn synth_villager_no(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.32)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut bp = BiquadFilter::bandpass(460.0, 2.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / count as f32;
        let (freq, sub_t) = if t < 0.48 {
            (160.0, t / 0.48)
        } else {
            (125.0, (t - 0.48) / 0.52)
        };
        phase += 2.0 * PI * freq / sr;

        let raw = phase.sin() * 0.7 + (phase * 2.0).sin() * 0.35;
        let filtered = bp.process(raw);
        let pulse_env = (sub_t * (1.0 - sub_t) * 4.0).clamp(0.0, 1.0);
        out.push(filtered * pulse_env * volume * 1.5);
    }
    out
}

/// Iron Golem idle: heavy mechanical footstep / grinding metal gears.
fn synth_iron_golem_idle(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.35)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = BiquadFilter::lowpass(320.0, 1.2, sr);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(2.0);
        let metal = (2.0 * PI * 180.0 * (i as f32 / sr)).sin() * 0.4
            + (2.0 * PI * 340.0 * (i as f32 / sr)).sin() * 0.25;
        let noise = (rng.next_f32() - 0.5) * 0.5;
        let filtered = lp.process(noise + metal);
        out.push(filtered * env * volume * 1.6);
    }
    out
}

/// Iron Golem hurt: heavy resonant metallic anvil strike / clang!
fn synth_iron_golem_hurt(_rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.30)) as usize;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (-t * 9.0).exp();
        let hit = (2.0 * PI * 420.0 * (i as f32 / sr)).sin() * 0.5
            + (2.0 * PI * 840.0 * (i as f32 / sr)).sin() * 0.35
            + (2.0 * PI * 1260.0 * (i as f32 / sr)).sin() * 0.15;
        out.push(hit * env * volume * 1.5);
    }
    out
}

/// Iron Golem death: deep metallic shudder and heavy crumbling collapse.
fn synth_iron_golem_death(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.75)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = BiquadFilter::lowpass(280.0, 1.5, sr);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = (1.0 - t).powf(1.5);
        let r_freq = 90.0 + (1.0 - t) * 110.0;
        let rumble = (2.0 * PI * r_freq * (i as f32 / sr)).sin() * 0.6;
        let noise = (rng.next_f32() - 0.5) * 0.5;
        let filtered = lp.process(rumble + noise);
        out.push(filtered * env * volume * 1.8);
    }
    out
}

/// Iron Golem attack: massive upward swing whoosh and heavy punch impact.
fn synth_iron_golem_attack(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(0.25)) as usize;
    let mut out = Vec::with_capacity(count);
    let mut lp = BiquadFilter::lowpass(550.0, 1.0, sr);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let env = if t < 0.25 { t / 0.25 } else { (1.0 - t) / 0.75 };
        let whoosh = lp.process((rng.next_f32() - 0.5) * 0.8);
        let thud = (2.0 * PI * 110.0 * (i as f32 / sr)).sin() * 0.5 * (-t * 8.0).exp();
        out.push((whoosh + thud) * env * volume * 1.6);
    }
    out
}

#[inline]
fn normalize_and_scale(samples: &mut [f32], target_vol: f32) {
    let mut peak: f32 = 0.0;
    for &s in samples.iter() {
        peak = peak.max(s.abs());
    }
    if peak > 1e-4 {
        let mult = target_vol / peak;
        for s in samples.iter_mut() {
            *s = (*s * mult).clamp(-1.0, 1.0);
        }
    }
}

/// Soothing procedural rainfall ambience with continuous organic white/pink noise patter.
fn synth_rain_ambient(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration.max(3.0)) as usize;
    let mut out = vec![0.0f32; count];

    let mut bp_rain = BiquadFilter::bandpass(1400.0, 1.2, sr);
    let mut lp_rain = IirFilter::lowpass(2800.0, sr);
    let mut hp_rain = IirFilter::highpass(300.0, sr);

    for i in 0..count {
        let t = i as f32 / sr;
        let norm_t = i as f32 / count as f32;
        let raw = rng.next_f32();
        let filtered = lp_rain.process(hp_rain.process(bp_rain.process(raw)));
        let gust = 0.80 + 0.20 * (t * 1.5 * 2.0 * PI).sin() * (t * 0.4 * 2.0 * PI).cos();
        let env = (norm_t * PI).sin().powf(0.3);
        out[i] = filtered * gust * env * 0.55;
    }

    // Discrete raindrop impact micro-clicks
    let drop_count = 35;
    for _ in 0..drop_count {
        let start = (rng.next_f32() * 0.9 * duration * sr) as usize;
        let len = (sr * 0.015) as usize;
        let freq = 1800.0 + rng.next_f32() * 1400.0;
        let mut d_bp = BiquadFilter::bandpass(freq, 2.5, sr);
        for i in 0..len {
            let idx = start + i;
            if idx >= count { break; }
            let tau = i as f32 / len as f32;
            let env = (1.0 - tau) * (1.0 - tau);
            out[idx] += d_bp.process(rng.next_f32()) * env * 0.22;
        }
    }

    normalize_and_scale(&mut out, volume);
    out
}

/// Massive explosive thunderclap or deep rumbling horizon thunder.
fn synth_thunder(rng: &mut FastRng, sr: f32, is_close: bool, volume: f32) -> Vec<f32> {
    let duration = if is_close { 4.2 } else { 3.8 };
    let count = (sr * duration) as usize;
    let mut out = vec![0.0f32; count];

    if is_close {
        // 1. Initial sharp supersonic electric crack (0.0 .. 0.15s)
        let crack_len = (sr * 0.15) as usize;
        let mut crack_hp = IirFilter::highpass(3500.0, sr);
        let mut crack_bp = BiquadFilter::bandpass(2400.0, 1.8, sr);
        for i in 0..crack_len {
            let tau = i as f32 / crack_len as f32;
            let env = (1.0 - tau).powf(1.5);
            let noise = rng.next_f32();
            out[i] += (crack_hp.process(noise) * 0.6 + crack_bp.process(noise) * 0.4) * env * 0.85;
        }

        // 2. Heavy acoustic explosion thump (0.02 .. 0.45s)
        let thump_len = (sr * 0.45) as usize;
        let thump_start = (sr * 0.02) as usize;
        let mut phase = 0.0f32;
        for i in 0..thump_len {
            let idx = thump_start + i;
            if idx >= count { break; }
            let t = i as f32 / sr;
            let freq = 130.0 * (-t * 12.0).exp() + 45.0;
            phase += 2.0 * PI * freq / sr;
            let env = (1.0 - i as f32 / thump_len as f32).powf(1.8);
            out[idx] += phase.sin() * env * 0.75;
        }
    }

    // Deep rolling subterranean & atmospheric reverberation rumble (40Hz - 90Hz)
    let rumble_start = if is_close { (sr * 0.1) as usize } else { 0 };
    let mut lp_rumble1 = IirFilter::lowpass(85.0, sr);
    let mut lp_rumble2 = IirFilter::lowpass(120.0, sr);
    let mut bp_sub = BiquadFilter::bandpass(55.0, 1.5, sr);

    for i in rumble_start..count {
        let t = (i - rumble_start) as f32 / sr;
        let norm_t = (i - rumble_start) as f32 / (count - rumble_start) as f32;
        let noise = rng.next_f32();
        let sub = bp_sub.process(noise) * 0.5 + lp_rumble1.process(lp_rumble2.process(noise)) * 0.5;

        // Undulating rolling sound wave characteristic of atmospheric echo
        let roll_mod = 0.65
            + 0.25 * (t * 3.2 * 2.0 * PI).sin()
            + 0.15 * (t * 1.4 * 2.0 * PI + 1.2).sin();
        let decay = (1.0 - norm_t).powf(1.4);
        out[i] += sub * roll_mod * decay * 0.80;
    }

    normalize_and_scale(&mut out, volume);
    out
}

/// Synthesizes Minecraft-style bow string release: fast frequency drop twang + noise whoosh.
fn synth_bow_shoot(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = vec![0.0f32; count];
    let mut phase = 0.0f32;
    let mut hp_whoosh = IirFilter::highpass(1600.0, sr);

    for i in 0..count {
        let t = i as f32 / sr;
        let env_sine = (-t * 22.0).exp();
        // Pitch drop from ~360 Hz to ~140 Hz
        let freq = 140.0 + 220.0 * (-t * 30.0).exp();
        phase += 2.0 * PI * freq / sr;
        let sine = phase.sin() * env_sine * 0.75;

        // Brief rushing whoosh of arrow leaving the bow
        let whoosh_env = if t < 0.08 { (1.0 - t / 0.08) * 0.45 } else { 0.0 };
        let whoosh = hp_whoosh.process(rng.next_f32()) * whoosh_env;

        out[i] = (sine + whoosh) * volume;
    }
    out
}

/// Synthesizes arrow impact: sharp snap and wooden fleche shudder.
fn synth_bow_hit(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = vec![0.0f32; count];
    let mut phase = 0.0f32;
    let mut bp_snap = BiquadFilter::bandpass(2400.0, 3.0, sr);

    for i in 0..count {
        let t = i as f32 / sr;
        let env_snap = (-t * 45.0).exp();
        let snap = bp_snap.process(rng.next_f32()) * env_snap * 0.9;

        // Resonant wood body click
        let freq = 420.0 * (-t * 25.0).exp() + 180.0;
        phase += 2.0 * PI * freq / sr;
        let body = phase.sin() * (-t * 35.0).exp() * 0.6;

        out[i] = (snap + body) * volume;
    }
    out
}

/// Synthesizes shield block: metallic deflection clang + solid wood dissipation.
fn synth_shield_block(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = vec![0.0f32; count];
    let mut bp_metal1 = BiquadFilter::bandpass(1450.0, 5.0, sr);
    let mut bp_metal2 = BiquadFilter::bandpass(2900.0, 6.0, sr);
    let mut phase = 0.0f32;

    for i in 0..count {
        let t = i as f32 / sr;
        let raw = rng.next_f32();
        let metal_env = (-t * 16.0).exp();
        let clang = (bp_metal1.process(raw) * 0.6 + bp_metal2.process(raw) * 0.4) * metal_env * 0.85;

        // Heavy solid wood backing thump
        let wood_env = (-t * 28.0).exp();
        phase += 2.0 * PI * 180.0 / sr;
        let thud = phase.sin() * wood_env * 0.6;

        out[i] = (clang + thud) * volume;
    }
    out
}

/// Synthesizes armor equip: metallic / leather buckle clicks.
fn synth_armor_equip(rng: &mut FastRng, sr: f32, duration: f32, volume: f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let mut out = vec![0.0f32; count];
    let mut bp_clink = BiquadFilter::bandpass(3200.0, 4.0, sr);

    // Three rapid micro-clinks representing buckles buckling
    let clink_times = [0.015f32, 0.075, 0.14];
    for &ct in &clink_times {
        let start_idx = (ct * sr) as usize;
        let clink_len = (0.05 * sr) as usize;
        for j in 0..clink_len {
            let idx = start_idx + j;
            if idx >= count { break; }
            let t = j as f32 / sr;
            let env = (-t * 60.0).exp();
            let clink = bp_clink.process(rng.next_f32()) * env * 0.7;
            out[idx] += clink;
        }
    }
    normalize_and_scale(&mut out, volume);
    out
}


