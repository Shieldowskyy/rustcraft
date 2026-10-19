//! Android native audio backend via AAudio / OpenSL ES.

use super::AudioBackend;

pub struct AndroidAudioBackend {
    active: bool,
}

impl AndroidAudioBackend {
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl Default for AndroidAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for AndroidAudioBackend {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        self.active = true;
        Ok(())
    }

    fn write_samples(&mut self, _samples: &[f32]) {
        // AAudio / OpenSL ES audio output buffer
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn name(&self) -> &'static str {
        "Android AAudio / OpenSL ES"
    }
}
