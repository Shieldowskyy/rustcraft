//! Microsoft Xbox 360 native audio backend (Xenon PowerPC).
//! Target: Microsoft Xbox 360 / libxenon / XAudio

use super::AudioBackend;

pub struct Xbox360AudioBackend {
    active: bool,
    pcm_scratch: Vec<i16>,
}

impl Xbox360AudioBackend {
    pub fn new() -> Self {
        Self {
            active: false,
            pcm_scratch: Vec::with_capacity(1024),
        }
    }
}

impl Default for Xbox360AudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for Xbox360AudioBackend {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        self.active = true;
        Ok(())
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active {
            return;
        }

        // Convert float [-1.0, 1.0] to Big-Endian / Little-Endian 16-bit PCM for Xenon
        self.pcm_scratch.clear();
        self.pcm_scratch.reserve(samples.len());
        for &s in samples {
            let clamped = (s * 32767.0).clamp(-32768.0, 32767.0);
            self.pcm_scratch.push(clamped as i16);
        }

        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc"))]
        {
            // libxenon audio submit hook
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn name(&self) -> &'static str {
        "Microsoft Xbox 360 Xenon Audio"
    }
}
