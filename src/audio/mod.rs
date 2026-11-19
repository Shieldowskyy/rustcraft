//! Backwards-compatible facade re-exporting `crate::features::audio`.

pub mod backend;
pub mod engine;
pub mod mixer;
pub mod ring_buffer;
pub mod sound;
pub mod soundtrack;
pub mod spatial;
pub mod synth;

pub use crate::features::audio::{
    preload_intro_music, take_preloaded_manager, AmbientContext, AudioEngine, AudioListener,
    AudioStats, MusicToast, SoundCategory, SoundId, SoundSample, SoundtrackCategory,
    SoundtrackManager, SoundtrackTrack,
};
