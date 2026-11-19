//! Pusty backend audio (Null Sink) dla serwerów dedykowanych, testów i platform bez wyjścia dźwięku.

use super::AudioBackend;

pub struct NullAudioBackend {
    active: bool,
}

impl NullAudioBackend {
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for NullAudioBackend {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        self.active = true;
        Ok(())
    }

    #[inline(always)]
    fn write_samples(&mut self, _samples: &[f32]) {}

    #[inline(always)]
    fn is_active(&self) -> bool {
        self.active
    }

    #[inline(always)]
    fn name(&self) -> &'static str {
        "Null Audio Sink"
    }
}
