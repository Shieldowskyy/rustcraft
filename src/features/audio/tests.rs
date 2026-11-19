//! Testy integracyjne syntezy proceduralnej, dźwięku przestrzennego 3D, miksera polifonicznego, kolejki SPSC i silnika audio.

use super::*;
use crate::core::math::Vec3;
use crate::features::audio::mixer::AudioMixer;
use crate::features::audio::ring_buffer::{AudioCommand, SpscCommandQueue};
use crate::features::audio::synth::synthesize_sound;
use crate::features::blocks::BlockId;

#[test]
fn test_all_sounds_synthesis_validity() {
    let sample_rate = 48000;
    let test_sounds = [
        SoundId::StepGrass,
        SoundId::StepDirt,
        SoundId::StepStone,
        SoundId::StepWood,
        SoundId::StepSand,
        SoundId::StepWater,
        SoundId::DigDirt,
        SoundId::DigStone,
        SoundId::BreakGrass,
        SoundId::BreakStone,
        SoundId::BreakWood,
        SoundId::BreakGlass,
        SoundId::PlaceDirt,
        SoundId::PlaceStone,
        SoundId::PlayerJump,
        SoundId::PlayerLand,
        SoundId::PlayerHurt,
        SoundId::PlayerEat,
        SoundId::ItemPop,
        SoundId::UiClick,
        SoundId::ChestOpen,
        SoundId::ChestClose,
        SoundId::DoorOpen,
        SoundId::DoorClose,
        SoundId::Explosion,
        SoundId::PigIdle,
        SoundId::PigHurt,
        SoundId::PigDeath,
        SoundId::CowIdle,
        SoundId::CowHurt,
        SoundId::CowDeath,
        SoundId::ChickenIdle,
        SoundId::ChickenHurt,
        SoundId::ChickenDeath,
        SoundId::ZombieIdle,
        SoundId::ZombieHurt,
        SoundId::ZombieDeath,
        SoundId::SkeletonIdle,
        SoundId::SkeletonHurt,
        SoundId::SkeletonDeath,
        SoundId::SheepIdle,
        SoundId::SheepHurt,
        SoundId::SheepDeath,
        SoundId::SheepShear,
        SoundId::AmbientWind,
        SoundId::AmbientCave,
        SoundId::AmbientWater,
        SoundId::AmbientLeaves,
        SoundId::AmbientCaveRocks,
        SoundId::StepWoodCreak,
        SoundId::AmbientDesertSand,
        SoundId::Rain,
        SoundId::ThunderClose,
        SoundId::ThunderDistant,
    ];

    for &id in &test_sounds {
        let sample = synthesize_sound(id, sample_rate);
        assert!(!sample.data.is_empty(), "Sample for {:?} must not be empty", id);
        assert!(
            sample.duration_seconds() > 0.0,
            "Sample {:?} duration must be > 0",
            id
        );
        for (idx, &s) in sample.data.iter().enumerate() {
            assert!(!s.is_nan(), "Sample {:?} at {} is NaN", id, idx);
            assert!(!s.is_infinite(), "Sample {:?} at {} is Infinite", id, idx);
            assert!(
                (-2.0..=2.0).contains(&s),
                "Sample {:?} value {} out of range",
                id,
                s
            );
        }
    }
}

#[test]
fn test_spatial_audio_panning_and_attenuation() {
    let listener = AudioListener::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let (right_lg, right_rg) =
        listener.calculate_spatial_gains(Vec3::new(5.0, 0.0, 0.0), 1.0, 30.0, 1.0);
    assert!(right_rg > right_lg);

    let (left_lg, left_rg) =
        listener.calculate_spatial_gains(Vec3::new(-5.0, 0.0, 0.0), 1.0, 30.0, 1.0);
    assert!(left_lg > left_rg);

    let (center_lg, center_rg) =
        listener.calculate_spatial_gains(Vec3::new(0.0, 0.0, 5.0), 1.0, 30.0, 1.0);
    assert!((center_lg - center_rg).abs() < 0.05);

    let (far_lg, far_rg) =
        listener.calculate_spatial_gains(Vec3::new(0.0, 0.0, 50.0), 1.0, 30.0, 1.0);
    assert_eq!(far_lg, 0.0);
    assert_eq!(far_rg, 0.0);
}

#[test]
fn test_polyphonic_mixer_rendering() {
    let mut mixer = AudioMixer::new(48000);
    let sample = synthesize_sound(SoundId::UiClick, 48000);

    mixer.play_sample(&sample, SoundCategory::Sfx, 1.0, 1.0, 0.7, 0.7, 1.0, false);
    assert_eq!(mixer.active_voice_count(), 1);

    let mut out = [0.0f32; 512];
    mixer.mix_interleaved_stereo(&mut out);

    let energy: f32 = out.iter().map(|s| s * s).sum();
    assert!(energy > 0.0);

    for &s in &out {
        assert!((-1.0..=1.0).contains(&s), "Sample {} clipped", s);
    }
}

#[test]
fn test_lock_free_spsc_command_queue() {
    let mut q = SpscCommandQueue::new();
    assert!(q.pop().is_none());

    let cmd = AudioCommand::PlaySound {
        sound_id: SoundId::UiClick,
        volume: 1.0,
        pitch: 1.0,
        pan: 0.0,
        looping: false,
        spatial: false,
        position: Vec3::zero(),
        min_dist: 1.0,
        max_dist: 20.0,
        rolloff: 1.0,
        occlusion: 0.0,
    };

    assert!(q.push(cmd));
    let popped = q.pop();
    assert!(popped.is_some());
    assert!(q.pop().is_none());
}

#[test]
fn test_audio_engine_lifecycle_and_controls() {
    let mut engine = AudioEngine::new();
    let stats = engine.stats();
    assert_eq!(stats.sample_rate, 48000);

    engine.set_master_volume(0.5);
    engine.set_sfx_volume(0.8);
    engine.set_ambient_volume(0.6);
    engine.set_ui_volume(0.9);
    engine.set_music_volume(0.4);

    assert!(!engine.is_muted());
    engine.toggle_mute();
    assert!(engine.is_muted());
    engine.toggle_mute();
    assert!(!engine.is_muted());

    engine.play_sound(SoundId::UiClick, 1.0, 1.0);
    engine.play_sound_3d(SoundId::StepGrass, Vec3::new(1.0, 2.0, 3.0), 0.8, 1.0);
    engine.play_sound_3d_varied(SoundId::BreakStone, Vec3::new(2.0, 0.0, 2.0), 0.9);
    engine.play_sound_3d_varied_occluded(SoundId::BreakWood, Vec3::new(3.0, 0.0, 3.0), 0.85, 0.7);

    engine.update(
        0.016,
        Vec3::new(0.0, 50.0, 0.0),
        4.0,
        true,
        false,
        BlockId::Grass,
        AmbientContext::default(),
    );
}

#[test]
fn test_spatial_effects_air_absorption_and_occlusion() {
    let listener = AudioListener::new(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let close_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 2.0), 1.0, 40.0, 1.0, 0.0);
    assert!(close_res.filter_alpha > 0.85);

    let far_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 25.0), 1.0, 40.0, 1.0, 0.0);
    assert!(far_res.filter_alpha < close_res.filter_alpha);

    let occluded_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 4.0), 1.0, 40.0, 1.0, 1.0);
    assert!(occluded_res.filter_alpha < 0.15);
    let unobstructed_same_dist =
        listener.calculate_spatial(Vec3::new(0.0, 0.0, 4.0), 1.0, 40.0, 1.0, 0.0);
    assert!(occluded_res.left_gain < unobstructed_same_dist.left_gain);
}

#[test]
fn test_minecraft_break_and_dig_synthesis() {
    let sample_rate = 44100;

    let break_sounds = [
        SoundId::BreakStone,
        SoundId::BreakWood,
        SoundId::BreakDirt,
        SoundId::BreakGrass,
        SoundId::BreakGlass,
        SoundId::BreakSand,
    ];
    for sound in break_sounds {
        let sample = synthesize_sound(sound, sample_rate);
        assert!(!sample.data.is_empty());
        let energy: f32 = sample.data.iter().map(|s| s * s).sum();
        assert!(energy > 0.05);
    }

    let dig_sounds = [
        SoundId::DigStone,
        SoundId::DigWood,
        SoundId::DigDirt,
        SoundId::DigGrass,
        SoundId::DigSand,
    ];
    for sound in dig_sounds {
        let sample = synthesize_sound(sound, sample_rate);
        assert!(!sample.data.is_empty());
        assert!(sample.data.len() < sample_rate as usize);
    }
}
