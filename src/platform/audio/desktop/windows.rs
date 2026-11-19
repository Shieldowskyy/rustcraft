//! Sterownik Windows Multimedia WaveOut (`winmm.dll`) dla platformy Windows.

use super::super::AudioBackend;

pub(super) struct WinMmAudioDriver {
    active: bool,
}

impl WinMmAudioDriver {
    pub fn try_open() -> Option<Self> {
        Some(Self { active: true })
    }
}

impl AudioBackend for WinMmAudioDriver {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        Ok(())
    }

    fn write_samples(&mut self, _samples: &[f32]) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn name(&self) -> &'static str {
        "Windows Multimedia WaveOut"
    }
}
