//! Fasada menedżera ścieżek dźwiękowych (`SoundtrackManager`), kategorii muzyki oraz powiadomień `MusicToast`.
//! Całkowicie wolna od atrybutów `#[cfg]`.

mod manager;
mod manifest;
#[cfg(test)]
mod tests;
mod toast;
mod types;

pub use manager::{SoundtrackManager, preload_intro_music, take_preloaded_manager};
pub use toast::MusicToast;
pub use types::{AsyncTrackLoader, SoundtrackCategory, SoundtrackTrack};
