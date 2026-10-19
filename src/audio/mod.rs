//! Cross-platform high-performance audio engine for RustCraft.
//!
//! Features:
//! - Universal hardware abstraction: Linux (PulseAudio/PipeWire, ALSA), Windows (WaveOut/WASAPI),
//!   WASM (HTML5 Web Audio), Sony PS Vita (SceAudioOut), Xbox 360 (Xenon DMA), Android (AAudio),
//!   and zero-cost Null sink for headless servers and unit tests.
//! - Lock-free SPSC thread architecture: zero audio latency, zero mutex contention, zero stalling
//!   on game simulation or chunk generation threads.
//! - Ultra-fast polyphonic mixing: pre-allocated 64-voice pool, priority voice stealing,
//!   linear interpolation sub-sample pitch resampling, and branchless anti-clipping limiter.
//! - Realistic 3D Spatial Audio: distance-squared fast culling, constant-power stereo panning,
//!   and listener orientation tracking.
//! - Full procedural sound synthesis: authentic zero-asset in-memory waveforms generated in < 2ms.
//! - Integrated footsteps cadence, block break/place feedback, UI clicks, and ambient atmosphere.

pub mod backend;
pub mod engine;
pub mod mixer;
pub mod ring_buffer;
pub mod sound;
pub mod spatial;
pub mod synth;

pub use engine::{AudioEngine, AudioStats};
pub use sound::{SoundCategory, SoundId, SoundSample};
pub use spatial::AudioListener;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec3;
    use crate::audio::ring_buffer::{AudioCommand, SpscCommandQueue};
    use crate::audio::mixer::AudioMixer;
    use crate::audio::synth::synthesize_sound;

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
            SoundId::AmbientWind,
            SoundId::AmbientCave,
        ];

        for &id in &test_sounds {
            let sample = synthesize_sound(id, sample_rate);
            assert!(!sample.data.is_empty(), "Sample for {:?} must not be empty", id);
            assert!(sample.duration_seconds() > 0.0, "Sample {:?} duration must be > 0", id);
            for (idx, &s) in sample.data.iter().enumerate() {
                assert!(!s.is_nan(), "Sample {:?} at {} is NaN", id, idx);
                assert!(!s.is_infinite(), "Sample {:?} at {} is Infinite", id, idx);
                assert!(s >= -2.0 && s <= 2.0, "Sample {:?} value {} out of range", id, s);
            }
        }
    }

    #[test]
    fn test_spatial_audio_panning_and_attenuation() {
        let listener = AudioListener::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0), // right ear points along +X
            Vec3::new(0.0, 1.0, 0.0),
        );

        // Sound straight to the right (+X)
        let (right_lg, right_rg) = listener.calculate_spatial_gains(Vec3::new(5.0, 0.0, 0.0), 1.0, 30.0, 1.0);
        assert!(right_rg > right_lg, "Sound to the right should have right gain > left gain");

        // Sound straight to the left (-X)
        let (left_lg, left_rg) = listener.calculate_spatial_gains(Vec3::new(-5.0, 0.0, 0.0), 1.0, 30.0, 1.0);
        assert!(left_lg > left_rg, "Sound to the left should have left gain > right gain");

        // Sound directly centered ahead (+Z)
        let (center_lg, center_rg) = listener.calculate_spatial_gains(Vec3::new(0.0, 0.0, 5.0), 1.0, 30.0, 1.0);
        assert!((center_lg - center_rg).abs() < 0.05, "Centered sound should have balanced left and right gains");

        // Sound beyond max distance
        let (far_lg, far_rg) = listener.calculate_spatial_gains(Vec3::new(0.0, 0.0, 50.0), 1.0, 30.0, 1.0);
        assert_eq!(far_lg, 0.0);
        assert_eq!(far_rg, 0.0);
    }

    #[test]
    fn test_polyphonic_mixer_rendering() {
        let mut mixer = AudioMixer::new(48000);
        let sample = synthesize_sound(SoundId::UiClick, 48000);

        // Play sound with transparent filter (1.0)
        mixer.play_sample(&sample, SoundCategory::Sfx, 1.0, 1.0, 0.7, 0.7, 1.0, false);
        assert_eq!(mixer.active_voice_count(), 1);

        let mut out = [0.0f32; 512]; // 256 stereo frames
        mixer.mix_interleaved_stereo(&mut out);

        // Verify output has audible signal
        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 0.0, "Mixer output should contain audio energy");

        // Verify no clipping (all samples in [-1.0, 1.0])
        for &s in &out {
            assert!(s >= -1.0 && s <= 1.0, "Sample {} clipped", s);
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

        // Volume adjustments
        engine.set_master_volume(0.5);
        engine.set_sfx_volume(0.8);
        engine.set_ambient_volume(0.6);
        engine.set_ui_volume(0.9);
        engine.set_music_volume(0.4);

        // Mute toggle
        assert!(!engine.is_muted());
        engine.toggle_mute();
        assert!(engine.is_muted());
        engine.toggle_mute();
        assert!(!engine.is_muted());

        // Play sounds
        engine.play_sound(SoundId::UiClick, 1.0, 1.0);
        engine.play_sound_3d(SoundId::StepGrass, Vec3::new(1.0, 2.0, 3.0), 0.8, 1.0);
        engine.play_sound_3d_varied(SoundId::BreakStone, Vec3::new(2.0, 0.0, 2.0), 0.9);
        engine.play_sound_3d_varied_occluded(SoundId::BreakWood, Vec3::new(3.0, 0.0, 3.0), 0.85, 0.7);

        // Cadence update
        engine.update(0.016, Vec3::new(0.0, 50.0, 0.0), 4.0, true, false, crate::world::block::BlockId::Grass);
    }

    #[test]
    fn test_spatial_effects_air_absorption_and_occlusion() {
        let listener = AudioListener::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

        // Close unobstructed sound
        let close_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 2.0), 1.0, 40.0, 1.0, 0.0);
        assert!(close_res.filter_alpha > 0.85, "Close sound should have high filter alpha (crisp highs)");

        // Distant unobstructed sound (air absorption test)
        let far_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 25.0), 1.0, 40.0, 1.0, 0.0);
        assert!(far_res.filter_alpha < close_res.filter_alpha, "Far sound should suffer air absorption lowpass");

        // Occluded sound behind solid wall (occlusion test)
        let occluded_res = listener.calculate_spatial(Vec3::new(0.0, 0.0, 4.0), 1.0, 40.0, 1.0, 1.0);
        assert!(occluded_res.filter_alpha < 0.15, "Fully occluded sound should be heavily lowpassed (muffled)");
        let unobstructed_same_dist = listener.calculate_spatial(Vec3::new(0.0, 0.0, 4.0), 1.0, 40.0, 1.0, 0.0);
        assert!(occluded_res.left_gain < unobstructed_same_dist.left_gain, "Occluded sound gain should be attenuated");
    }

    #[test]
    fn test_minecraft_break_and_dig_synthesis() {
        let sample_rate = 44100;

        // Verify break sound synthesis
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
            assert!(!sample.data.is_empty(), "{:?} generated empty sample", sound);
            let energy: f32 = sample.data.iter().map(|s| s * s).sum();
            assert!(energy > 0.05, "{:?} sample has insufficient energy", sound);
        }

        // Verify dig sounds
        let dig_sounds = [
            SoundId::DigStone,
            SoundId::DigWood,
            SoundId::DigDirt,
            SoundId::DigGrass,
            SoundId::DigSand,
        ];
        for sound in dig_sounds {
            let sample = synthesize_sound(sound, sample_rate);
            assert!(!sample.data.is_empty(), "{:?} generated empty sample", sound);
            assert!(sample.data.len() < sample_rate as usize, "{:?} dig sound too long", sound);
        }
    }
}

