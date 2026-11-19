//! Fasada 64-głosowego polifonicznego miksera audio z pogłosem pomieszczenia i śledzeniem dźwięku 3D.

mod render;
mod state;
#[cfg(test)]
mod tests;

pub use state::{AudioMixer, MAX_VOICES, Voice};
