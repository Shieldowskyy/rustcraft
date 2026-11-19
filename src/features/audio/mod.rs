//! Fasada domeny audio (`features::audio`): silnik, mikser 64-głosowy, synteza proceduralna, dźwięk 3D i soundtracki.
//! Całkowicie wolna od atrybutów `#[cfg]` — sprzętowe backendy znajdują się w `crate::platform::audio`.

pub mod engine;
pub mod mixer;
pub mod ring_buffer;
pub mod sound;
pub mod soundtrack;
pub mod spatial;
pub mod synth;
#[cfg(test)]
mod tests;

pub use engine::{AmbientContext, AudioEngine, AudioStats};
pub use sound::{SoundCategory, SoundId, SoundSample};
pub use soundtrack::{
    MusicToast, SoundtrackCategory, SoundtrackManager, SoundtrackTrack, preload_intro_music,
    take_preloaded_manager,
};
pub use spatial::AudioListener;
