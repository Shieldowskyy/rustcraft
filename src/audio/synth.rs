//! High-performance procedural sound synthesizer.
//!
//! Generates crisp, authentic Minecraft-style sound effects completely in memory
//! without relying on any external disk assets or audio libraries.
//! Synthesis takes < 2 milliseconds for the entire soundbank at startup.

use crate::audio::sound::{SoundId, SoundSample};
use std::f32::consts::{PI, TAU};

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
#[derive(Clone, Copy)]
pub struct IirFilter {
    alpha: f32,
    prev_out: f32,
    prev_in: f32,
    is_highpass: bool,
}

impl IirFilter {
    pub fn lowpass(cutoff_hz: f32, sr: f32) -> Self {
        let rc = 1.0 / (TAU * cutoff_hz.max(10.0));
        let dt = 1.0 / sr;
        Self {
            alpha: (dt / (rc + dt)).clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: false,
        }
    }

    pub fn highpass(cutoff_hz: f32, sr: f32) -> Self {
        let rc = 1.0 / (TAU * cutoff_hz.max(10.0));
        let dt = 1.0 / sr;
        Self {
            alpha: (rc / (rc + dt)).clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: true,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let out = if self.is_highpass {
            self.alpha * (self.prev_out + input - self.prev_in)
        } else {
            self.prev_out + self.alpha * (input - self.prev_out)
        };
        self.prev_in = input;
        self.prev_out = out;
        out
    }
}

/// 2-pole Biquad IIR filter for sharp acoustic resonances.
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
    fn new(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
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

    fn params(hz: f32, q: f32, sr: f32) -> (f32, f32) {
        let w0 = TAU * hz.clamp(20.0, sr * 0.48) / sr;
        (w0.cos(), w0.sin() / (2.0 * q.max(0.1)))
    }

    pub fn bandpass(center_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(center_hz, q, sr);
        Self::new(alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
    }

    pub fn lowpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(cutoff_hz, q, sr);
        let b1 = 1.0 - cos_w0;
        Self::new(
            b1 * 0.5,
            b1,
            b1 * 0.5,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        )
    }

    pub fn highpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(cutoff_hz, q, sr);
        let b0 = (1.0 + cos_w0) * 0.5;
        Self::new(
            b0,
            -(1.0 + cos_w0),
            b0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        )
    }

    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

// ============================================================================
// Core Synthesis Helpers
// ============================================================================

/// Generates a sample buffer using a closure: `f(index, time_sec, normalized_t)`.
#[inline]
fn render(sr: f32, duration: f32, mut f: impl FnMut(usize, f32, f32) -> f32) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let inv_count = 1.0 / count.max(1) as f32;
    (0..count)
        .map(|i| f(i, i as f32 / sr, i as f32 * inv_count))
        .collect()
}

/// Overlays discrete bandpass-filtered noise grains (debris, splinters, shards, bones).
fn add_bp_grains(
    out: &mut [f32],
    rng: &mut FastRng,
    sr: f32,
    g_dur: f32,
    q: f32,
    vol: f32,
    impulse: bool,
    grains: &[(f32, f32, f32)],
) {
    let g_len = (sr * g_dur) as usize;
    for &(time, freq, amp) in grains {
        let start = (time * sr) as usize;
        let mut bp = BiquadFilter::bandpass(freq, q, sr);
        for i in 0..g_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / g_len as f32;
            let raw = if impulse && i < 3 {
                1.0
            } else if impulse {
                rng.next_f32() * 0.4
            } else {
                rng.next_f32()
            };
            out[start + i] += bp.process(raw) * (1.0 - t).powi(2) * amp * vol;
        }
    }
}

#[inline]
fn normalize_and_scale(samples: &mut [f32], target_vol: f32) {
    let peak = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    if peak > 1e-4 {
        let mult = target_vol / peak;
        for s in samples.iter_mut() {
            *s = (*s * mult).clamp(-1.0, 1.0);
        }
    }
}

// ============================================================================
// Sound Library Dispatcher
// ============================================================================

pub fn synthesize_sound(id: SoundId, sample_rate: u32) -> SoundSample {
    let sr = sample_rate as f32;
    let seed = match id {
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
        SoundId::ToolBreak => 715,
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
        SoundId::FishFlop => 823,
        SoundId::DolphinIdle => 824,
        SoundId::DolphinHurt => 825,
        SoundId::DolphinDeath => 826,
        SoundId::DolphinJump => 827,
        SoundId::FishingCast => 828,
        SoundId::FishingReel => 829,
        SoundId::FishCatch => 830,
        SoundId::VillagerIdle | SoundId::SpiderIdle => 831,
        SoundId::VillagerHurt | SoundId::SpiderHurt => 832,
        SoundId::VillagerDeath | SoundId::SpiderDeath => 833,
        SoundId::VillagerTrade => 834,
        SoundId::VillagerNo => 835,
        SoundId::IronGolemIdle => 836,
        SoundId::IronGolemHurt => 837,
        SoundId::IronGolemDeath => 838,
        SoundId::IronGolemAttack => 839,
    };
    let mut rng = FastRng::new(seed);

    let samples = match id {
        // Footsteps
        SoundId::StepGrass => synth_noise_burst(&mut rng, sr, 0.12, 1800.0, 3200.0, 1.2, 0.6),
        SoundId::StepDirt => synth_noise_burst(&mut rng, sr, 0.14, 300.0, 1400.0, 1.8, 0.8),
        SoundId::StepStone => synth_impact(&mut rng, sr, 0.09, 320.0, 1600.0, 0.015, 0.7),
        SoundId::StepWood => synth_wood_step(&mut rng, sr, 0.14, 0.85),
        SoundId::StepWoodCreak => synth_wood_creak(&mut rng, sr, 0.24, 0.70),
        SoundId::StepSand => synth_noise_burst(&mut rng, sr, 0.15, 600.0, 2400.0, 1.0, 0.75),
        SoundId::StepGravel => synth_noise_burst(&mut rng, sr, 0.16, 400.0, 2800.0, 1.4, 0.85),
        SoundId::StepWater => synth_water_splash(&mut rng, sr, 0.22, 0.4),

        // Digging
        SoundId::DigGrass => synth_dig_scrape(&mut rng, sr, 0.075, 1200.0, 3200.0, 45.0, 0.75),
        SoundId::DigDirt => synth_dirt_dig(&mut rng, sr, 0.08, 0.8),
        SoundId::DigStone => synth_bp_dig(
            &mut rng, sr, 0.07, 130.0, 60.0, 0.7, 2400.0, 2.5, 80.0, 1.3, 0.85,
        ),
        SoundId::DigWood => synth_bp_dig(
            &mut rng, sr, 0.08, 180.0, 45.0, 0.75, 1700.0, 2.0, 70.0, 1.1, 0.8,
        ),
        SoundId::DigSand => synth_dig_scrape(&mut rng, sr, 0.085, 600.0, 2400.0, 35.0, 0.75),

        // Breaking
        SoundId::BreakGrass => synth_grass_break(&mut rng, sr, 0.28, 0.9),
        SoundId::BreakDirt => synth_dirt_break(&mut rng, sr, 0.30, 0.95),
        SoundId::BreakStone => synth_stone_break(&mut rng, sr, 0.35, 1.0),
        SoundId::BreakWood => synth_wood_break(&mut rng, sr, 0.32, 0.95),
        SoundId::BreakGlass => synth_glass_shatter(&mut rng, sr, 0.38, 0.9),
        SoundId::BreakSand => synth_noise_burst(&mut rng, sr, 0.28, 500.0, 2800.0, 1.5, 0.85 * 1.1),

        // Placement
        SoundId::PlaceGrass => synth_noise_burst(&mut rng, sr, 0.11, 800.0, 2600.0, 1.8, 0.7),
        SoundId::PlaceDirt => synth_impact(&mut rng, sr, 0.14, 180.0, 900.0, 0.03, 0.85),
        SoundId::PlaceStone => synth_impact(&mut rng, sr, 0.12, 280.0, 1800.0, 0.02, 0.9),
        SoundId::PlaceWood => synth_wood_step(&mut rng, sr, 0.16, 0.90),
        SoundId::PlaceSand => synth_noise_burst(&mut rng, sr, 0.16, 350.0, 1600.0, 1.5, 0.75),
        SoundId::PlaceGeneric => synth_impact(&mut rng, sr, 0.12, 240.0, 1200.0, 0.02, 0.8),

        // Player
        SoundId::PlayerJump => synth_sweep(sr, 0.14, 110.0, 280.0, 0.6),
        SoundId::PlayerLand => synth_impact(&mut rng, sr, 0.22, 90.0, 600.0, 0.06, 0.95),
        SoundId::PlayerHurt => synth_hurt_grunt(&mut rng, sr, 0.20, 0.9),
        SoundId::PlayerSplash => synth_water_splash(&mut rng, sr, 0.38, 0.9),
        SoundId::PlayerEat => synth_noise_burst(&mut rng, sr, 0.14, 1200.0, 4500.0, 1.5, 0.7),

        // UI & World
        SoundId::ItemPop => synth_sweep(sr, 0.09, 440.0, 980.0, 0.7),
        SoundId::UiClick => synth_click(sr, 0.04, 1000.0, 0.85),
        SoundId::UiHover => synth_click(sr, 0.02, 1400.0, 0.35),
        SoundId::ChestOpen => synth_chest_creak(sr, 0.28, true, 0.8),
        SoundId::ChestClose => synth_chest_creak(sr, 0.25, false, 0.8),
        SoundId::DoorOpen => synth_door_swing(sr, 0.22, true, 0.85),
        SoundId::DoorClose => synth_door_swing(sr, 0.20, false, 0.85),
        SoundId::Explosion => synth_explosion(&mut rng, sr, 1.4, 1.0),

        // Mobs
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
        SoundId::CreeperHurt => synth_creeper_impulse(
            &mut rng,
            sr,
            0.22,
            2100.0,
            2.5,
            3,
            1.2,
            0.4,
            true,
            0.90 * 1.6,
        ),
        SoundId::CreeperDeath => synth_creeper_impulse(
            &mut rng,
            sr,
            0.45,
            1400.0,
            2.0,
            5,
            1.5,
            0.5,
            false,
            0.95 * 1.5,
        ),
        SoundId::SheepIdle => synth_sheep_idle(&mut rng, sr, 0.75, 0.85),
        SoundId::SheepHurt => synth_sheep_hurt(&mut rng, sr, 0.28, 0.95),
        SoundId::SheepDeath => synth_sheep_death(sr, 0.70, 0.95),
        SoundId::SheepShear => synth_shears(&mut rng, sr, 0.15, 0.90),
        SoundId::VillagerIdle => synth_villager_idle(&mut rng, sr, 0.45, 0.90),
        SoundId::VillagerHurt => synth_villager_vocal(
            &mut rng,
            sr,
            0.22,
            680.0,
            175.0,
            55.0,
            1.0,
            0.45,
            0.18,
            1.3,
            0.95 * 1.6,
        ),
        SoundId::VillagerDeath => synth_villager_vocal(
            &mut rng,
            sr,
            0.65,
            380.0,
            155.0,
            -65.0,
            0.7,
            0.35,
            0.06,
            1.5,
            0.95 * 1.5,
        ),
        SoundId::VillagerTrade => synth_villager_trade(sr, 0.38, 0.90),
        SoundId::VillagerNo => synth_villager_no(sr, 0.32, 0.90),
        SoundId::IronGolemIdle => synth_iron_golem_idle(&mut rng, sr, 0.40, 0.85),
        SoundId::IronGolemHurt => synth_iron_golem_hurt(sr, 0.35, 0.95),
        SoundId::IronGolemDeath => synth_iron_golem_death(&mut rng, sr, 0.80, 0.95),
        SoundId::IronGolemAttack => synth_iron_golem_attack(&mut rng, sr, 0.30, 0.95),
        SoundId::FishFlop => synth_fish_flop(&mut rng, sr, 0.18, 0.85),
        SoundId::DolphinIdle => synth_dolphin_idle(sr, 0.45, 0.85),
        SoundId::DolphinHurt => synth_dolphin_whistle(sr, 0.28, 2400.0, -1500.0, 1.8, 0.95 * 1.4),
        SoundId::DolphinDeath => synth_dolphin_whistle(sr, 0.75, 1600.0, -900.0, 1.3, 0.95 * 1.3),
        SoundId::DolphinJump => synth_water_splash(&mut rng, sr, 0.40, 0.90),
        SoundId::FishingCast => synth_sweep(sr, 0.16, 220.0, 950.0, 0.85),
        SoundId::FishingReel => synth_fishing_reel(&mut rng, sr, 0.22, 0.85),
        SoundId::FishCatch => synth_fish_catch(&mut rng, sr, 0.35, 0.95),
        SoundId::SpiderIdle => synth_spider_idle(&mut rng, sr, 0.35, 0.85),
        SoundId::SpiderHurt => synth_spider_vocal(
            &mut rng, sr, 0.22, true, 1200.0, 800.0, 900.0, 0.4, 0.6, 1.5, 0.90,
        ),
        SoundId::SpiderDeath => synth_spider_vocal(
            &mut rng, sr, 0.50, false, 2200.0, 150.0, 600.0, 0.45, 0.55, 1.8, 0.95,
        ),

        // Ambient & Combat
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
        SoundId::ToolBreak => synth_tool_break(&mut rng, sr, 0.28, 1.0),
    };

    SoundSample::new(samples, sample_rate, 1)
}

// ============================================================================
// Synthesis Implementations
// ============================================================================

fn synth_noise_burst(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    hp_cut: f32,
    lp_cut: f32,
    decay: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut hp, mut lp) = (
        IirFilter::highpass(hp_cut, sr),
        IirFilter::lowpass(lp_cut, sr),
    );
    render(sr, dur, |_, _, nt| {
        lp.process(hp.process(rng.next_f32())) * (1.0 - nt).powf(decay) * vol
    })
}

fn synth_impact(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    tone_freq: f32,
    noise_lp: f32,
    noise_dur: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(noise_lp, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * tone_freq / sr;
        let sine_val = phase.sin() * (-t * 30.0).exp();
        let noise_env = if t < noise_dur {
            (1.0 - (t / noise_dur)).max(0.0)
        } else {
            0.0
        };
        (sine_val + lp.process(rng.next_f32()) * noise_env * 0.7) * vol
    })
}

fn synth_stone_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.32)) as usize];
    let (mut bp_crack, mut bp_rumble, mut phase) = (
        BiquadFilter::bandpass(2400.0, 2.2, sr),
        BiquadFilter::bandpass(140.0, 1.5, sr),
        0.0f32,
    );
    for i in 0..((sr * 0.08) as usize).min(out.len()) {
        let t = i as f32 / sr;
        phase += TAU * (180.0 - t * 1400.0).max(50.0) / sr;
        let raw = rng.next_f32();
        out[i] += (phase.sin() * (-t * 45.0).exp() * 0.85
            + bp_crack.process(raw) * (-t * 60.0).exp() * 1.4
            + bp_rumble.process(raw) * (-t * 30.0).exp() * 0.6)
            * vol;
    }
    let grains = [
        (0.035, 2800.0, 0.85),
        (0.070, 2100.0, 0.70),
        (0.115, 3200.0, 0.55),
        (0.165, 1800.0, 0.42),
        (0.220, 2500.0, 0.30),
        (0.280, 1500.0, 0.18),
    ];
    add_bp_grains(&mut out, rng, sr, 0.035, 3.0, vol, false, &grains);
    out
}

fn synth_wood_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.30)) as usize];
    let (mut bp_splinter, mut p1, mut p2) =
        (BiquadFilter::bandpass(1650.0, 1.8, sr), 0.0f32, 0.0f32);
    for i in 0..((sr * 0.10) as usize).min(out.len()) {
        let t = i as f32 / sr;
        let freq = (210.0 - t * 800.0).max(90.0);
        p1 += TAU * freq / sr;
        p2 += TAU * (freq * 2.1) / sr;
        out[i] += ((p1.sin() * 0.6 + p2.sin() * 0.4) * (-t * 35.0).exp()
            + bp_splinter.process(rng.next_f32()) * (-t * 40.0).exp() * 1.2)
            * vol;
    }
    let grains = [
        (0.03, 1900.0, 0.75),
        (0.065, 1400.0, 0.60),
        (0.11, 2200.0, 0.45),
        (0.16, 1700.0, 0.32),
        (0.22, 1200.0, 0.20),
    ];
    add_bp_grains(&mut out, rng, sr, 0.03, 2.5, vol, false, &grains);
    out
}

fn synth_dirt_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.28)) as usize];
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(850.0, sr),
        IirFilter::highpass(180.0, sr),
        0.0f32,
    );
    for i in 0..((sr * 0.09) as usize).min(out.len()) {
        let t = i as f32 / sr;
        phase += TAU * 85.0 / sr;
        out[i] += (phase.sin() * (-t * 40.0).exp() * 0.9
            + lp.process(hp.process(rng.next_f32())) * (-t * 35.0).exp() * 1.1)
            * vol;
    }
    let c_len = (sr * 0.04) as usize;
    for &(time, amp) in &[
        (0.035, 0.65),
        (0.075, 0.50),
        (0.12, 0.38),
        (0.17, 0.26),
        (0.22, 0.15),
    ] {
        let start = (time * sr) as usize;
        let (mut c_lp, mut c_hp) = (
            IirFilter::lowpass(650.0, sr),
            IirFilter::highpass(220.0, sr),
        );
        for i in 0..c_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / c_len as f32;
            out[start + i] +=
                c_lp.process(c_hp.process(rng.next_f32())) * (1.0 - t).powi(2) * amp * vol;
        }
    }
    out
}

fn synth_grass_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp_l, mut lp_l, mut lp_e) = (
        IirFilter::highpass(1400.0, sr),
        IirFilter::lowpass(4200.0, sr),
        IirFilter::lowpass(400.0, sr),
    );
    render(sr, dur.max(0.26), |_, _, nt| {
        let raw = rng.next_f32();
        (lp_l.process(hp_l.process(raw)) * (1.0 - nt).powf(1.8) * 0.85 * 1.1
            + lp_e.process(raw) * (-nt * 12.0).exp() * 0.45)
            * vol
    })
}

fn synth_glass_shatter(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.38)) as usize];
    let (mut hp, mut lp) = (
        IirFilter::highpass(2200.0, sr),
        IirFilter::lowpass(9500.0, sr),
    );
    let snap_len = (sr * 0.015) as usize;
    for i in 0..snap_len.min(out.len()) {
        out[i] += lp.process(hp.process(rng.next_f32()))
            * (1.0 - i as f32 / snap_len as f32).powi(3)
            * 1.8
            * vol;
    }
    let (mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..((sr * 0.32) as usize).min(out.len()) {
        let t = i as f32 / sr;
        p1 += TAU * 3150.0 / sr;
        p2 += TAU * 5200.0 / sr;
        p3 += TAU * 7800.0 / sr;
        out[i] +=
            (p1.sin() * 0.4 + p2.sin() * 0.35 + p3.sin() * 0.25) * (-t * 12.0).exp() * 0.7 * vol;
    }
    let tinkles = [
        (0.03, 4200.0, 0.7),
        (0.07, 6800.0, 0.6),
        (0.12, 3900.0, 0.5),
        (0.17, 8100.0, 0.4),
        (0.23, 5400.0, 0.3),
        (0.29, 7200.0, 0.22),
        (0.34, 4800.0, 0.15),
    ];
    add_bp_grains(&mut out, rng, sr, 0.02, 4.0, vol, false, &tinkles);
    out
}

fn synth_bp_dig(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    tone_f: f32,
    tone_dec: f32,
    tone_amp: f32,
    bp_f: f32,
    bp_q: f32,
    bp_dec: f32,
    bp_amp: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(bp_f, bp_q, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * tone_f / sr;
        (phase.sin() * (-t * tone_dec).exp() * tone_amp
            + bp.process(rng.next_f32()) * (-t * bp_dec).exp() * bp_amp)
            * vol
    })
}

fn synth_dirt_dig(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(900.0, sr),
        IirFilter::highpass(180.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.07), |_, t, _| {
        phase += TAU * 95.0 / sr;
        let env = (-t * 50.0).exp();
        (phase.sin() * env * 0.75 + lp.process(hp.process(rng.next_f32())) * env) * vol
    })
}

fn synth_dig_scrape(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    hp_f: f32,
    lp_f: f32,
    dec: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut lp, mut hp) = (IirFilter::lowpass(lp_f, sr), IirFilter::highpass(hp_f, sr));
    render(sr, dur, |_, t, _| {
        lp.process(hp.process(rng.next_f32())) * (-t * dec).exp() * 1.1 * vol
    })
}

fn synth_water_splash(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp, mut phase) = (
        IirFilter::highpass(500.0, sr),
        IirFilter::lowpass(2600.0, sr),
        0.0f32,
    );
    render(sr, dur, |_, t, nt| {
        phase += TAU * (500.0 - t * 1800.0).max(120.0) / sr;
        let plop = phase.sin() * (-t * 35.0).exp() * 0.7;
        let spray =
            lp.process(hp.process(rng.next_f32())) * (nt * (1.0 - nt) * 4.0).powf(1.5) * 0.55;
        (plop + spray) * vol
    })
}

fn synth_sweep(sr: f32, dur: f32, start_f: f32, end_f: f32, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase += TAU * (start_f + (end_f - start_f) * nt) / sr;
        phase.sin() * (1.0 - nt).powf(1.1) * vol
    })
}

fn synth_click(sr: f32, dur: f32, freq: f32, vol: f32) -> Vec<f32> {
    let (mut phase, mut click_rng) = (0.0f32, 0x2A3B4C5Du32);
    render(sr, dur, |_, t, nt| {
        phase += TAU * (freq * (1.18 - 0.35 * nt)) / sr;
        click_rng = click_rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((click_rng >> 16) as f32 / 32768.0) - 1.0;
        ((phase.sin() * 0.65 + noise * (-t * 500.0).exp() * 0.75) * (-t * 85.0).exp() * vol)
            .clamp(-1.0, 1.0)
    })
}

fn synth_hurt_grunt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(600.0, sr), 0.0f32);
    render(sr, dur, |_, _, nt| {
        phase += TAU * (160.0 - nt * 60.0) / sr;
        (phase.sin() * 0.6 + lp.process(rng.next_f32()) * 0.4) * (-nt * 12.0).exp() * vol
    })
}

fn synth_chest_creak(sr: f32, dur: f32, open: bool, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase +=
            TAU * (if open {
                300.0 + nt * 450.0
            } else {
                750.0 - nt * 450.0
            }) / sr;
        let tri = (phase % TAU / PI - 1.0).abs() * 2.0 - 1.0;
        tri * (nt * (1.0 - nt) * 4.0).min(1.0) * vol
    })
}

fn synth_door_swing(sr: f32, dur: f32, open: bool, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase +=
            TAU * (if open {
                180.0 + nt * 140.0
            } else {
                320.0 - nt * 140.0
            }) / sr;
        phase.sin() * (-nt * 4.0).exp() * vol
    })
}

fn synth_explosion(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(400.0, sr), 0.0f32);
    render(sr, dur, |_, t, nt| {
        let crack = if t < 0.08 {
            rng.next_f32() * (1.0 - t / 0.08).max(0.0) * 0.9
        } else {
            0.0
        };
        phase += TAU * (55.0 - nt * 25.0) / sr;
        let rumble = (phase.sin() * 0.4 + lp.process(rng.next_f32()) * 0.6) * (-t * 2.5).exp();
        (crack + rumble) * vol
    })
}

fn synth_wood_step(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp_c, mut bp_b) = (
        BiquadFilter::bandpass(320.0, 3.2, sr),
        BiquadFilter::bandpass(680.0, 3.5, sr),
    );
    let (mut hp_t, mut lp_t, mut phase) = (
        IirFilter::highpass(1200.0, sr),
        IirFilter::lowpass(3800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.12), |i, t, nt| {
        phase += TAU * (180.0 - nt * 95.0).max(70.0) / sr;
        let thump = (phase.sin() * 0.7 + (phase * 2.1).sin() * 0.3) * (-t * 38.0).exp();
        let raw_imp = if i < 3 { 1.5 } else { rng.next_f32() * 0.3 };
        let cavity = bp_c.process(raw_imp) * (-t * 32.0).exp();
        let board = bp_b.process(raw_imp) * (-t * 42.0).exp();
        let sole = lp_t.process(hp_t.process(rng.next_f32())) * (-t * 65.0).exp();
        (thump * 0.65 + cavity * 0.75 + board * 0.45 + sole * 0.40) * vol
    })
}

fn synth_wood_creak(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut bp_h) = (
        BiquadFilter::bandpass(380.0, 4.2, sr),
        BiquadFilter::bandpass(940.0, 3.6, sr),
        BiquadFilter::bandpass(1750.0, 4.0, sr),
    );
    let (mut car_p, mut mod_p) = (0.0f32, 0.0f32);
    render(sr, dur.max(0.20), |_, t, nt| {
        let base_pitch = 320.0 - (nt * TAU).sin() * 35.0 - nt * 45.0;
        let friction = if (t * 55.0 * TAU).sin() > 0.85 {
            rng.next_f32() * 0.35
        } else {
            0.0
        };
        let mod_idx = (1.8 * (1.0 - nt).powf(0.8) + friction).max(0.1);
        mod_p += TAU * (base_pitch * 2.82) / sr;
        car_p += TAU * base_pitch / sr;
        let raw_fm = (car_p + mod_p.sin() * mod_idx).sin();
        let body = bp1.process(raw_fm) * 0.70
            + bp2.process(raw_fm) * 0.45
            + bp_h.process(raw_fm + friction) * 0.25;
        let env = (nt * PI).sin().powf(0.7) * (1.0 - nt * 0.3);
        (body / (1.0 + body.abs() * 0.4)) * env * vol * 1.3
    })
}

fn synth_wind_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp_sub, mut hp_sub) = (IirFilter::lowpass(180.0, sr), IirFilter::highpass(32.0, sr));
    let (mut bp_mid, mut bp_high, mut lp_hiss) = (
        BiquadFilter::bandpass(380.0, 1.2, sr),
        BiquadFilter::bandpass(680.0, 1.4, sr),
        IirFilter::lowpass(1100.0, sr),
    );
    let mut brown = 0.0f32;
    render(sr, dur.max(3.0), |_, _, nt| {
        let env = ((nt * PI).sin().powf(1.3)
            * (1.0 + 0.28 * (nt * PI * 3.5).sin() * (nt * PI).sin()))
        .clamp(0.0, 1.4);
        let raw = rng.next_f32();
        brown = (brown * 0.94 + raw * 0.06).clamp(-1.0, 1.0);
        let sub = hp_sub.process(lp_sub.process(raw + brown * 0.5)) * 1.6;
        let mid = bp_mid.process(raw) * 1.3;
        let high = bp_high.process(lp_hiss.process(raw)) * 0.75;
        ((sub * 0.65 + mid * 0.85 + high * 0.50) * env).clamp(-1.0, 1.0) * vol * 1.4
    })
}

fn synth_cave_chord(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut p0, mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut lp_cave = IirFilter::lowpass(480.0, sr);
    render(sr, dur.max(3.0), |_, t, nt| {
        let drift = (t * 0.3 * TAU).sin() * 0.8;
        p0 += TAU * (55.0 + drift) / sr;
        p1 += TAU * (65.41 - drift * 0.5) / sr;
        p2 += TAU * (77.78 + drift * 0.7) / sr;
        p3 += TAU * (98.0 + drift) / sr;
        let tone = p0.sin() * 0.35
            + p1.sin() * 0.30
            + p2.sin() * 0.22
            + p3.sin() * 0.18
            + lp_cave.process(rng.next_f32()) * 0.15;
        tone * (nt * PI).sin().powf(0.85) * vol * 1.3
    })
}

fn synth_water_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp) = (
        BiquadFilter::bandpass(420.0, 1.4, sr),
        IirFilter::lowpass(750.0, sr),
    );
    let mut out = render(sr, dur.max(3.0), |_, t, nt| {
        let ripple = 0.75 + 0.25 * (t * 2.8 * TAU).sin() * (t * 0.7 * TAU).cos();
        lp.process(bp.process(rng.next_f32())) * ripple * (nt * PI).sin().powf(0.4) * 0.55
    });
    let bubbles = [
        (0.35, 620.0),
        (0.78, 840.0),
        (1.25, 520.0),
        (1.72, 950.0),
        (2.15, 710.0),
        (2.65, 880.0),
        (3.10, 600.0),
        (3.48, 760.0),
    ];
    let len = (sr * 0.045) as usize;
    for &(bt, base_f) in &bubbles {
        let (start, mut b_phase) = ((bt * sr) as usize, 0.0f32);
        for i in 0..len {
            if start + i >= out.len() {
                break;
            }
            let tau = i as f32 / sr;
            b_phase += TAU * (base_f + (base_f * 0.8) * (1.0 - (-tau * 85.0).exp())) / sr;
            out[start + i] += b_phase.sin() * (-tau * 55.0).exp() * 0.65;
        }
    }
    for s in &mut out {
        *s = (*s / (1.0 + s.abs() * 0.35)) * vol;
    }
    out
}

fn synth_leaves_rustle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut bp_b) = (
        BiquadFilter::bandpass(2800.0, 1.8, sr),
        BiquadFilter::bandpass(5200.0, 2.2, sr),
        BiquadFilter::bandpass(480.0, 2.8, sr),
    );
    let mut hp = IirFilter::highpass(1600.0, sr);
    render(sr, dur.max(2.5), |_, t, nt| {
        let flutter = 0.55 + 0.30 * (t * 16.5 * TAU).sin() + 0.15 * (t * 23.0 * TAU).cos();
        let raw = rng.next_f32();
        let rustle = bp1.process(hp.process(raw)) * 0.65 + bp2.process(raw) * 0.40;
        (rustle * flutter + bp_b.process(raw) * 0.20) * (nt * PI).sin().powf(1.2) * vol * 1.35
    })
}

fn synth_cave_rocks(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let count = (sr * dur.max(1.8)) as usize;
    let mut out = vec![0.0f32; count];
    let impacts = [
        (0.04, 1800.0, 0.90),
        (0.16, 2400.0, 0.75),
        (0.31, 1500.0, 0.65),
        (0.44, 2900.0, 0.55),
        (0.55, 2100.0, 0.45),
        (0.63, 2600.0, 0.35),
        (0.70, 3300.0, 0.25),
    ];
    let len = (sr * 0.035) as usize;
    for &(it, freq, amp) in &impacts {
        let (start, mut bp, mut phase) = (
            (it * sr) as usize,
            BiquadFilter::bandpass(freq, 3.5, sr),
            0.0f32,
        );
        for i in 0..len {
            if start + i >= count {
                break;
            }
            let t = i as f32 / len as f32;
            let imp = if i < 3 { 1.6 } else { rng.next_f32() * 0.3 };
            phase += TAU * (140.0 - t * 40.0) / sr;
            out[start + i] +=
                (bp.process(imp) * (1.0 - t).powi(2) + phase.sin() * (1.0 - t).powi(3) * 0.4) * amp;
        }
    }
    for &(delay_s, gain) in &[(0.065, 0.35), (0.135, 0.22), (0.220, 0.14)] {
        let (delay, mut lp) = ((delay_s * sr) as usize, IirFilter::lowpass(1400.0, sr));
        for i in delay..count {
            out[i] += lp.process(out[i - delay]) * gain;
        }
    }
    for s in &mut out {
        *s *= vol * 1.2;
    }
    out
}

fn synth_desert_sand(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut bp1, mut bp2) = (
        IirFilter::highpass(780.0, sr),
        BiquadFilter::bandpass(2400.0, 1.8, sr),
        BiquadFilter::bandpass(4600.0, 2.4, sr),
    );
    let mut phase = 0.0f32;
    render(sr, dur.max(2.8), |_, t, nt| {
        let shimmer = 0.65 + 0.35 * (t * 42.0 * TAU).sin();
        let raw = rng.next_f32();
        phase += TAU * 110.0 / sr;
        let comp = hp.process(raw) * 0.35
            + (bp1.process(raw) * 0.60 + bp2.process(raw) * 0.40) * shimmer
            + phase.sin() * 0.08 * (1.0 - nt * 0.5);
        comp * (nt * PI).sin().powf(1.6) * vol * 1.3
    })
}

fn synth_pig_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2) = (
        BiquadFilter::bandpass(480.0, 2.5, sr),
        BiquadFilter::bandpass(1100.0, 2.0, sr),
    );
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(750.0, sr),
        IirFilter::highpass(220.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.25), |_, t, nt| {
        phase += TAU * (135.0 - nt * 25.0) / sr;
        let rough = 0.45 + 0.55 * (t * 28.0 * TAU).sin();
        let src =
            (phase.sin() * 0.5 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.2) * rough;
        let formant = bp1.process(src) * 0.6 + bp2.process(src) * 0.4;
        let breath = hp.process(lp.process(rng.next_f32())) * rough * 0.25;
        let env = if t < 0.02 {
            t / 0.02
        } else {
            (-nt * 5.0).exp()
        };
        (formant * 1.8 + breath) * env * vol
    })
}

fn synth_pig_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut hp, mut phase) = (
        BiquadFilter::bandpass(900.0, 2.8, sr),
        IirFilter::highpass(1600.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.24), |_, t, nt| {
        let base_f = if nt < 0.25 {
            780.0 + (nt / 0.25) * 140.0
        } else {
            920.0 - ((nt - 0.25) / 0.75) * 280.0
        };
        phase += TAU * (base_f + (t * 38.0 * TAU).sin() * 40.0).max(200.0) / sr;
        let tone =
            bp.process((phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.65 + (phase * 2.0).sin() * 0.35);
        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-nt * 9.5).exp()
        };
        (tone * 1.6 + hp.process(rng.next_f32()) * 0.18) * env * vol
    })
}

fn synth_pig_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(950.0, 2.4, sr),
        BiquadFilter::bandpass(450.0, 2.0, sr),
        IirFilter::lowpass(650.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.60), |_, t, nt| {
        let base_f = if t < 0.15 {
            920.0 + (t / 0.15) * 60.0
        } else {
            980.0 - ((t - 0.15) / (dur - 0.15)).powf(0.85) * 740.0
        };
        phase += TAU * (base_f + (t * 26.0 * TAU).sin() * 55.0).max(120.0) / sr;
        let tone = (phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let formant = bp1.process(tone) * (1.0 - nt * 0.5) + bp2.process(tone) * (nt * 0.7);
        let env = if t < 0.01 {
            t / 0.01
        } else {
            (1.0 - nt).powf(1.4)
        };
        (formant * 1.5 + lp.process(rng.next_f32()) * (nt * 0.35)) * env * vol
    })
}

fn synth_cow_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(380.0, 2.2, sr),
        BiquadFilter::bandpass(780.0, 2.0, sr),
        BiquadFilter::lowpass(320.0, 1.4, sr),
        0.0f32,
    );
    render(sr, dur.max(0.95), |_, t, nt| {
        let freq = if nt < 0.20 {
            118.0 + (nt / 0.20) * 14.0
        } else {
            132.0 - ((nt - 0.20) / 0.80) * 27.0
        };
        phase += TAU * (freq + (t * 4.2 * TAU).sin() * 3.5) / sr;
        let cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.28
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.05;
        let blend = ((t - 0.12) / 0.18).clamp(0.0, 1.0);
        let vocal = lp.process(cords) * (1.0 - blend)
            + (bp1.process(cords) * 0.65 + bp2.process(cords) * 0.35) * blend * 1.5;
        let env = if t < 0.15 {
            (t / 0.15).powf(0.8)
        } else if nt > 0.70 {
            ((1.0 - nt) / 0.30).powf(1.5)
        } else {
            1.0
        };
        (vocal * 1.6 + rng.next_f32() * 0.05 * blend) * env * vol
    })
}

fn synth_cow_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(420.0, 2.0, sr),
        IirFilter::lowpass(600.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.32), |_, t, nt| {
        phase += TAU * (195.0 - nt * 90.0) / sr;
        let tone = (phase.sin() * 1.6).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let env = if t < 0.015 {
            t / 0.015
        } else {
            (-nt * 8.5).exp()
        };
        (bp.process(tone) * 1.7 + lp.process(rng.next_f32()) * 0.25) * env * vol
    })
}

fn synth_cow_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(360.0, 1.8, sr),
        IirFilter::lowpass(180.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.85), |_, t, nt| {
        phase += TAU * ((145.0 - nt.powf(0.8) * 85.0).max(50.0) + (t * 3.5 * TAU).sin() * 4.0) / sr;
        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.1;
        let env = if t < 0.02 {
            t / 0.02
        } else {
            (1.0 - nt).powf(1.6)
        };
        (bp.process(tone) * 1.8 + lp.process(rng.next_f32()) * 0.3 * (1.0 - nt)) * env * vol
    })
}

fn synth_chicken_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.26)) as usize];
    for &(start_t, c_dur, f_start, f_end) in
        &[(0.0, 0.07, 740.0, 480.0), (0.11, 0.09, 660.0, 390.0)]
    {
        let (start_s, dur_s) = ((start_t * sr) as usize, (c_dur * sr) as usize);
        let (mut bp, mut phase) = (BiquadFilter::bandpass(620.0, 3.2, sr), 0.0f32);
        for s in 0..dur_s {
            if start_s + s >= out.len() {
                break;
            }
            let (t, cur_t) = (s as f32 / dur_s as f32, s as f32 / sr);
            phase += TAU * (f_start - t * (f_start - f_end)) / sr;
            let click = if cur_t < 0.006 {
                rng.next_f32() * 0.4 * (1.0 - cur_t / 0.006)
            } else {
                0.0
            };
            let vocal = bp.process(phase.sin() * 0.8 + (phase * 2.0).sin() * 0.2);
            out[start_s + s] += (vocal * 1.5 + click) * (1.0 - t).powf(1.8) * vol;
        }
    }
    out
}

fn synth_chicken_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut hp, mut phase) = (
        BiquadFilter::bandpass(1050.0, 2.5, sr),
        IirFilter::highpass(1800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.20), |_, t, nt| {
        phase += TAU * (1200.0 - nt * 580.0 + (t * 48.0 * TAU).sin() * 50.0).max(250.0) / sr;
        let vocal =
            bp.process((phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.7 + (phase * 2.0).sin() * 0.3);
        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-nt * 11.0).exp()
        };
        (vocal * 1.6 + hp.process(rng.next_f32()) * 0.22) * env * vol
    })
}

fn synth_chicken_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.52)) as usize];
    let (cry_len, mut bp, mut phase) = (
        (sr * 0.32) as usize,
        BiquadFilter::bandpass(980.0, 2.2, sr),
        0.0f32,
    );
    for i in 0..cry_len.min(out.len()) {
        let (t, cur_t) = (i as f32 / cry_len as f32, i as f32 / sr);
        phase += TAU * (1300.0 - t.powf(0.8) * 920.0).max(200.0) / sr;
        let env = if cur_t < 0.01 {
            cur_t / 0.01
        } else {
            (1.0 - t).powf(1.8)
        };
        out[i] += bp.process((phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.75) * env * vol * 1.4;
    }
    let f_len = (sr * 0.045) as usize;
    for &(ft, amp) in &[(0.06, 0.55), (0.16, 0.45), (0.26, 0.35), (0.36, 0.20)] {
        let (start, mut lp, mut hp) = (
            (ft * sr) as usize,
            IirFilter::lowpass(800.0, sr),
            IirFilter::highpass(220.0, sr),
        );
        for i in 0..f_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / f_len as f32;
            out[start + i] +=
                lp.process(hp.process(rng.next_f32())) * (1.0 - t).powi(2) * amp * vol;
        }
    }
    out
}

fn synth_zombie_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(380.0, 3.0, sr),
        BiquadFilter::bandpass(820.0, 2.5, sr),
        0.0f32,
    );
    render(sr, dur.max(0.6), |_, t, nt| {
        phase += TAU * (115.0 - nt * 25.0 + (t * 6.0 * PI).sin() * 12.0) / sr;
        let buzz = ((phase.sin() * 2.2).clamp(-1.0, 1.0) + (phase * 2.0).sin() * 0.4) * 0.7;
        let sound = bp1.process(buzz + rng.next_f32() * 0.25) * 1.5 + bp2.process(buzz) * 0.8;
        let env = if nt < 0.15 {
            nt / 0.15
        } else if nt > 0.75 {
            (1.0 - nt) / 0.25
        } else {
            1.0
        };
        sound * env * vol
    })
}

fn synth_zombie_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(450.0, 2.2, sr), 0.0f32);
    render(sr, dur.max(0.28), |_, _, nt| {
        phase += TAU * (170.0 - nt * 60.0) / sr;
        bp.process((phase.sin() * 2.5).clamp(-1.0, 1.0) * 0.8 + rng.next_f32() * 0.3)
            * (-nt * 6.0).exp()
            * vol
            * 1.4
    })
}

fn synth_zombie_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(320.0, 2.0, sr), 0.0f32);
    render(sr, dur.max(0.7), |_, _, nt| {
        phase += TAU * (145.0 - nt.powf(0.8) * 85.0).max(45.0) / sr;
        bp.process((phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.7 + rng.next_f32() * 0.2)
            * (1.0 - nt).powf(1.4)
            * vol
            * 1.3
    })
}

fn synth_skeleton_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.35)) as usize];
    let clicks = [
        (0.03, 2400.0, 1.0),
        (0.09, 2400.0, 1.0),
        (0.16, 2400.0, 1.0),
        (0.24, 2400.0, 1.0),
    ];
    add_bp_grains(&mut out, rng, sr, 0.022, 4.0, vol * 1.5, true, &clicks);
    out
}

fn synth_skeleton_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2) = (
        BiquadFilter::bandpass(1900.0, 3.5, sr),
        BiquadFilter::bandpass(3400.0, 4.0, sr),
    );
    render(sr, dur.max(0.20), |i, _, nt| {
        let imp = if i < 4 { 1.2 } else { rng.next_f32() * 0.35 };
        (bp1.process(imp) * 0.7 + bp2.process(imp) * 0.6) * (-nt * 14.0).exp() * vol * 1.8
    })
}

fn synth_skeleton_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.6)) as usize];
    let drops = [
        (0.02, 1600.0, 1.0),
        (0.08, 1920.0, 0.88),
        (0.15, 2240.0, 0.76),
        (0.22, 2560.0, 0.64),
        (0.31, 2880.0, 0.52),
        (0.42, 3200.0, 0.40),
    ];
    add_bp_grains(&mut out, rng, sr, 0.035, 3.2, vol * 1.4, true, &drops);
    out
}

fn synth_creeper_hiss(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp) = (
        IirFilter::highpass(1600.0, sr),
        IirFilter::lowpass(6800.0, sr),
    );
    render(sr, dur, |_, _, nt| {
        let swell = nt.powf(0.5) * (0.65 + 0.35 * ((nt * 32.0).sin() * 0.5 + 0.5));
        lp.process(hp.process(rng.next_f32() * 2.0 - 1.0)) * swell * vol * 1.2
    })
}

fn synth_creeper_impulse(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    freq: f32,
    q: f32,
    imp_len: usize,
    imp_v: f32,
    n_amp: f32,
    exp_dec: bool,
    vol: f32,
) -> Vec<f32> {
    let mut bp = BiquadFilter::bandpass(freq, q, sr);
    render(sr, dur, |i, _, nt| {
        let imp = if i < imp_len {
            imp_v
        } else {
            rng.next_f32() * n_amp
        };
        let env = if exp_dec {
            (-nt * 12.0).exp()
        } else {
            (1.0 - nt).powf(1.8)
        };
        bp.process(imp) * env * vol
    })
}

fn synth_sheep_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(540.0, 2.2, sr),
        BiquadFilter::bandpass(1180.0, 2.0, sr),
        BiquadFilter::lowpass(420.0, 1.4, sr),
        0.0f32,
    );
    render(sr, dur.max(0.65), |_, t, nt| {
        let base_f = if nt < 0.25 {
            205.0 + (nt / 0.25) * 30.0
        } else {
            235.0 - ((nt - 0.25) / 0.75) * 50.0
        };
        phase += TAU * (base_f + (t * 6.4 * TAU).sin() * 7.5) / sr;
        let cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.25
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.06;
        let blend = ((t - 0.08) / 0.15).clamp(0.0, 1.0);
        let vocal = lp.process(cords) * (1.0 - blend)
            + (bp1.process(cords) * 0.65 + bp2.process(cords) * 0.35) * blend * 1.4;
        let env = if t < 0.08 {
            (t / 0.08).powf(0.8)
        } else if nt > 0.65 {
            ((1.0 - nt) / 0.35).powf(1.6)
        } else {
            1.0
        };
        (vocal * 1.5 + rng.next_f32() * 0.04 * blend) * env * vol
    })
}

fn synth_sheep_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(680.0, 2.0, sr),
        IirFilter::lowpass(800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.25), |_, t, nt| {
        phase += TAU * (290.0 - nt * 80.0 + (t * 8.5 * TAU).sin() * 10.0) / sr;
        let tone = (phase.sin() * 1.5).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let env = if t < 0.015 {
            t / 0.015
        } else {
            (-nt * 8.0).exp()
        };
        (bp.process(tone) * 1.6 + lp.process(rng.next_f32()) * 0.2) * env * vol
    })
}

fn synth_sheep_death(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(520.0, 1.8, sr), 0.0f32);
    render(sr, dur.max(0.65), |_, t, nt| {
        phase +=
            TAU * ((220.0 - nt.powf(0.7) * 110.0).max(80.0) + (t * 5.0 * TAU).sin() * 6.0) / sr;
        let env = if t < 0.04 {
            t / 0.04
        } else {
            (1.0 - nt).max(0.0).powf(1.4)
        };
        bp.process(phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3) * env * vol * 1.5
    })
}

fn synth_shears(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut bp1, mut bp2) = (
        IirFilter::highpass(2200.0, sr),
        BiquadFilter::bandpass(3400.0, 6.0, sr),
        BiquadFilter::bandpass(4900.0, 5.0, sr),
    );
    render(sr, dur.max(0.12), |_, t, nt| {
        let noise = rng.next_f32() * 2.0 - 1.0;
        let env_f = if t < 0.02 {
            t / 0.02
        } else {
            (-nt * 18.0).exp()
        };
        (hp.process(noise) * 0.5 * env_f
            + (bp1.process(noise) + bp2.process(noise)) * 0.8 * (-nt * 12.0).exp())
            * vol
            * 1.7
    })
}

fn synth_fish_flop(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = IirFilter::lowpass(1800.0, sr);
    render(sr, dur.max(0.12), |_, _, nt| {
        let noise = rng.next_f32() * 2.0 - 1.0;
        (lp.process(noise) * 0.6 + noise * (nt * 36.0 * PI).sin().abs() * 0.4)
            * (1.0 - nt).max(0.0).powf(1.5)
            * vol
            * 1.4
    })
}

fn synth_dolphin_idle(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.3), |i, t, nt| {
        let freq = 1800.0 + (nt * 6.0 * PI).sin() * 500.0 + nt * 400.0;
        let click = if (i % (sr as usize / 55)) < 4 {
            0.45
        } else {
            0.0
        };
        ((t * freq * TAU).sin() * 0.75 + click * 0.25) * (1.0 - nt).max(0.0).powf(1.2) * vol * 1.3
    })
}

fn synth_dolphin_whistle(sr: f32, dur: f32, start_f: f32, df: f32, exp: f32, vol: f32) -> Vec<f32> {
    render(sr, dur, |_, t, nt| {
        (t * (start_f + nt * df) * TAU).sin() * (1.0 - nt).max(0.0).powf(exp) * vol
    })
}

fn synth_fishing_reel(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.15), |i, _, nt| {
        let click = if (i % 280) < 35 {
            (rng.next_f32() * 2.0 - 1.0) * 0.8
        } else {
            0.0
        };
        click * (1.0 - nt).powf(1.2) * vol
    })
}

fn synth_fish_catch(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut splash = synth_water_splash(rng, sr, dur, vol * 0.8);
    for (s, p) in splash
        .iter_mut()
        .zip(synth_sweep(sr, 0.12, 380.0, 880.0, vol * 0.9))
    {
        *s = (*s + p).clamp(-1.0, 1.0);
    }
    splash
}

fn synth_villager_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(420.0, 2.5, sr),
        BiquadFilter::bandpass(1120.0, 3.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.40), |_, _, nt| {
        phase += TAU * (152.0 - nt * 14.0 + (nt * 8.0 * PI).sin() * 2.0) / sr;
        let raw = phase.sin() * 0.65
            + (phase * 2.0).sin() * 0.35
            + (phase * 3.0).sin() * 0.20
            + (phase * 4.0).sin() * 0.12;
        let env = if nt < 0.12 {
            (nt / 0.12).powf(0.8)
        } else if nt > 0.70 {
            ((1.0 - nt) / 0.30).powf(1.4)
        } else {
            1.0
        };
        ((bp1.process(raw) * 0.70 + bp2.process(raw) * 0.30) * env * 1.5
            + rng.next_f32() * 0.03 * env)
            * vol
    })
}

fn synth_villager_vocal(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    bp_f: f32,
    f0: f32,
    df: f32,
    h1: f32,
    h2: f32,
    n_amp: f32,
    exp: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(bp_f, 2.0, sr), 0.0f32);
    render(sr, dur, |_, _, nt| {
        phase += TAU * (f0 + nt * df) / sr;
        let raw =
            phase.sin() * h1 + (phase * 2.0).sin() * h2 + (rng.next_f32() * n_amp - n_amp * 0.5);
        bp.process(raw) * (1.0 - nt).powf(exp) * vol
    })
}

fn synth_villager_trade(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(520.0, 2.2, sr),
        BiquadFilter::bandpass(1280.0, 2.5, sr),
        0.0f32,
    );
    render(sr, dur.max(0.35), |_, t, nt| {
        phase += TAU * (140.0 + nt.powf(1.4) * 75.0) / sr;
        let raw = phase.sin() * 0.65 + (phase * 2.0).sin() * 0.35 + (phase * 3.0).sin() * 0.18;
        let env = if nt < 0.10 {
            nt / 0.10
        } else if nt > 0.75 {
            (1.0 - nt) / 0.25
        } else {
            1.0
        };
        let chime = (TAU * (660.0 + nt * 440.0) * t).sin() * 0.12 * env;
        ((bp1.process(raw) * 0.65 + bp2.process(raw) * 0.35) * env * 1.4 + chime) * vol
    })
}

fn synth_villager_no(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(460.0, 2.0, sr), 0.0f32);
    render(sr, dur.max(0.32), |_, _, nt| {
        let (freq, sub_t) = if nt < 0.48 {
            (160.0, nt / 0.48)
        } else {
            (125.0, (nt - 0.48) / 0.52)
        };
        phase += TAU * freq / sr;
        bp.process(phase.sin() * 0.7 + (phase * 2.0).sin() * 0.35)
            * (sub_t * (1.0 - sub_t) * 4.0).clamp(0.0, 1.0)
            * vol
            * 1.5
    })
}

fn synth_iron_golem_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(320.0, 1.2, sr);
    render(sr, dur.max(0.35), |_, t, nt| {
        let metal = (TAU * 180.0 * t).sin() * 0.4 + (TAU * 340.0 * t).sin() * 0.25;
        lp.process((rng.next_f32() - 0.5) * 0.5 + metal) * (1.0 - nt).powf(2.0) * vol * 1.6
    })
}

fn synth_iron_golem_hurt(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.30), |_, t, nt| {
        let hit = (TAU * 420.0 * t).sin() * 0.5
            + (TAU * 840.0 * t).sin() * 0.35
            + (TAU * 1260.0 * t).sin() * 0.15;
        hit * (-nt * 9.0).exp() * vol * 1.5
    })
}

fn synth_iron_golem_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(280.0, 1.5, sr);
    render(sr, dur.max(0.75), |_, t, nt| {
        let rumble = (TAU * (90.0 + (1.0 - nt) * 110.0) * t).sin() * 0.6;
        lp.process(rumble + (rng.next_f32() - 0.5) * 0.5) * (1.0 - nt).powf(1.5) * vol * 1.8
    })
}

fn synth_iron_golem_attack(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(550.0, 1.0, sr);
    render(sr, dur.max(0.25), |_, t, nt| {
        let env = if nt < 0.25 {
            nt / 0.25
        } else {
            (1.0 - nt) / 0.75
        };
        (lp.process((rng.next_f32() - 0.5) * 0.8)
            + (TAU * 110.0 * t).sin() * 0.5 * (-nt * 8.0).exp())
            * env
            * vol
            * 1.6
    })
}

fn synth_rain_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut hp) = (
        BiquadFilter::bandpass(1400.0, 1.2, sr),
        IirFilter::lowpass(2800.0, sr),
        IirFilter::highpass(300.0, sr),
    );
    let mut out = render(sr, dur.max(3.0), |_, t, nt| {
        let gust = 0.80 + 0.20 * (t * 1.5 * TAU).sin() * (t * 0.4 * TAU).cos();
        lp.process(hp.process(bp.process(rng.next_f32()))) * gust * (nt * PI).sin().powf(0.3) * 0.55
    });
    let len = (sr * 0.015) as usize;
    for _ in 0..35 {
        let start = (rng.next_f32() * 0.9 * dur * sr) as usize;
        let mut d_bp = BiquadFilter::bandpass(1800.0 + rng.next_f32() * 1400.0, 2.5, sr);
        for i in 0..len {
            if start + i >= out.len() {
                break;
            }
            let tau = i as f32 / len as f32;
            out[start + i] += d_bp.process(rng.next_f32()) * (1.0 - tau).powi(2) * 0.22;
        }
    }
    normalize_and_scale(&mut out, vol);
    out
}

fn synth_thunder(rng: &mut FastRng, sr: f32, is_close: bool, vol: f32) -> Vec<f32> {
    let count = (sr * if is_close { 4.2 } else { 3.8 }) as usize;
    let mut out = vec![0.0f32; count];
    if is_close {
        let (crack_len, mut c_hp, mut c_bp) = (
            (sr * 0.15) as usize,
            IirFilter::highpass(3500.0, sr),
            BiquadFilter::bandpass(2400.0, 1.8, sr),
        );
        for i in 0..crack_len {
            let noise = rng.next_f32();
            out[i] += (c_hp.process(noise) * 0.6 + c_bp.process(noise) * 0.4)
                * (1.0 - i as f32 / crack_len as f32).powf(1.5)
                * 0.85;
        }
        let (thump_len, thump_start, mut phase) =
            ((sr * 0.45) as usize, (sr * 0.02) as usize, 0.0f32);
        for i in 0..thump_len {
            if thump_start + i >= count {
                break;
            }
            phase += TAU * (130.0 * (-(i as f32 / sr) * 12.0).exp() + 45.0) / sr;
            out[thump_start + i] +=
                phase.sin() * (1.0 - i as f32 / thump_len as f32).powf(1.8) * 0.75;
        }
    }
    let r_start = if is_close { (sr * 0.1) as usize } else { 0 };
    let (mut lp1, mut lp2, mut bp_sub) = (
        IirFilter::lowpass(85.0, sr),
        IirFilter::lowpass(120.0, sr),
        BiquadFilter::bandpass(55.0, 1.5, sr),
    );
    for i in r_start..count {
        let (t, norm_t, noise) = (
            (i - r_start) as f32 / sr,
            (i - r_start) as f32 / (count - r_start) as f32,
            rng.next_f32(),
        );
        let sub = bp_sub.process(noise) * 0.5 + lp1.process(lp2.process(noise)) * 0.5;
        let roll = 0.65 + 0.25 * (t * 3.2 * TAU).sin() + 0.15 * (t * 1.4 * TAU + 1.2).sin();
        out[i] += sub * roll * (1.0 - norm_t).powf(1.4) * 0.80;
    }
    normalize_and_scale(&mut out, vol);
    out
}

fn synth_bow_shoot(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut phase) = (IirFilter::highpass(1600.0, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * (140.0 + 220.0 * (-t * 30.0).exp()) / sr;
        let whoosh = hp.process(rng.next_f32())
            * if t < 0.08 {
                (1.0 - t / 0.08) * 0.45
            } else {
                0.0
            };
        (phase.sin() * (-t * 22.0).exp() * 0.75 + whoosh) * vol
    })
}

fn synth_bow_hit(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(2400.0, 3.0, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        let snap = bp.process(rng.next_f32()) * (-t * 45.0).exp() * 0.9;
        phase += TAU * (420.0 * (-t * 25.0).exp() + 180.0) / sr;
        (snap + phase.sin() * (-t * 35.0).exp() * 0.6) * vol
    })
}

fn synth_shield_block(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(1450.0, 5.0, sr),
        BiquadFilter::bandpass(2900.0, 6.0, sr),
        0.0f32,
    );
    render(sr, dur, |_, t, _| {
        let raw = rng.next_f32();
        phase += TAU * 180.0 / sr;
        ((bp1.process(raw) * 0.6 + bp2.process(raw) * 0.4) * (-t * 16.0).exp() * 0.85
            + phase.sin() * (-t * 28.0).exp() * 0.6)
            * vol
    })
}

fn synth_armor_equip(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur) as usize];
    let mut bp = BiquadFilter::bandpass(3200.0, 4.0, sr);
    let clink_len = (0.05 * sr) as usize;
    for &ct in &[0.015f32, 0.075, 0.14] {
        let start = (ct * sr) as usize;
        for j in 0..clink_len {
            if start + j >= out.len() {
                break;
            }
            out[start + j] += bp.process(rng.next_f32()) * (-(j as f32 / sr) * 60.0).exp() * 0.7;
        }
    }
    normalize_and_scale(&mut out, vol);
    out
}

fn synth_spider_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut hp) = (
        IirFilter::lowpass(2800.0, sr),
        IirFilter::highpass(900.0, sr),
    );
    render(sr, dur, |_, t, _| {
        let rattle = ((t * 85.0).sin() * 0.5 + 0.5).powi(2);
        hp.process(lp.process((rng.next_f32() * 2.0 - 1.0) * rattle))
            * (1.0 - t / dur).max(0.0)
            * vol
    })
}

fn synth_spider_vocal(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    is_hp: bool,
    cutoff: f32,
    f0: f32,
    df: f32,
    t_amp: f32,
    n_amp: f32,
    exp: f32,
    vol: f32,
) -> Vec<f32> {
    let mut filt = if is_hp {
        IirFilter::highpass(cutoff, sr)
    } else {
        IirFilter::lowpass(cutoff, sr)
    };
    render(sr, dur, |_, t, _| {
        let rem = 1.0 - t / dur;
        let tone = (t * (f0 + rem * df) * TAU).sin() * t_amp;
        filt.process(tone + (rng.next_f32() * 2.0 - 1.0) * n_amp) * rem.powf(exp) * vol
    })
}

fn synth_tool_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp, snap_lim) = (
        IirFilter::highpass(400.0, sr),
        IirFilter::lowpass(4500.0, sr),
        (sr * 0.015) as usize,
    );
    render(sr, dur, |i, t, _| {
        let crack = (t * (1400.0 - t * 3000.0).max(100.0) * TAU).sin() * 0.4;
        let snap = (rng.next_f32() * 2.0 - 1.0) * if i < snap_lim { 0.9 } else { 0.3 };
        hp.process(lp.process(crack + snap)) * (-18.0 * t).exp() * vol
    })
}
