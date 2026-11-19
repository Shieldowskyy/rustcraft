//! Fasada głównego silnika audio (`AudioEngine`, `AmbientContext`, `AudioStats`).
//! Całkowicie wolna od atrybutów `#[cfg]`.

mod ambient;
mod state;
mod types;

pub use state::AudioEngine;
pub use types::{AmbientContext, AudioStats};
