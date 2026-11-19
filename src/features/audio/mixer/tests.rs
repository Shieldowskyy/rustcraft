//! Testy jednostkowe pogłosu miksera oraz śledzenia ruchu słuchacza dla dźwięków 3D.

use super::*;
use crate::core::math::Vec3;
use crate::features::audio::sound::{SoundCategory, SoundSample};
use crate::features::audio::spatial::AudioListener;

#[test]
fn reverb_keeps_effect_tail_but_does_not_wet_ui_sounds() {
    let impulse = SoundSample::new(vec![1.0], 48000, 1);
    let mut mixer = AudioMixer::new(48000);
    mixer.set_reverb(0.5, 1.0);
    mixer.play_sample(&impulse, SoundCategory::Sfx, 1.0, 1.0, 1.0, 1.0, 1.0, false);

    let mut output = [0.0; 2048];
    mixer.mix_interleaved_stereo(&mut output);
    assert!(mixer.has_pending_audio(), "Reverb tail should keep the mixer active");

    mixer.mix_interleaved_stereo(&mut output);
    assert!(
        output.iter().any(|sample| sample.abs() > 0.0001),
        "The effect should produce audible reflections after its dry sample ends"
    );

    let mut ui_mixer = AudioMixer::new(48000);
    ui_mixer.set_reverb(0.5, 1.0);
    ui_mixer.play_sample(&impulse, SoundCategory::Ui, 1.0, 1.0, 1.0, 1.0, 1.0, false);
    ui_mixer.mix_interleaved_stereo(&mut output);
    assert!(!ui_mixer.has_pending_audio(), "UI sounds should remain dry");
}

#[test]
fn active_spatial_sound_tracks_listener_movement() {
    let sample = SoundSample::new(vec![1.0; 1024], 48000, 1);
    let mut mixer = AudioMixer::new(48000);
    let source_pos = Vec3::new(10.0, 0.0, 0.0);
    mixer.play_spatial_sample(
        &sample,
        SoundCategory::Sfx,
        1.0,
        1.0,
        source_pos,
        2.0,
        36.0,
        1.0,
        0.0,
        false,
    );

    let initial_left = mixer.voices[0].target_left;
    let initial_right = mixer.voices[0].target_right;
    assert!(initial_right > initial_left);

    mixer.update_spatial_listener(AudioListener::new(
        Vec3::new(9.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ));

    let moved_left = mixer.voices[0].target_left;
    let moved_right = mixer.voices[0].target_right;
    assert!(moved_left + moved_right > initial_left + initial_right);
    assert!(moved_right > moved_left);
}
