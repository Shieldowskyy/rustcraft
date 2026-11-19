//! Kaskadowy desktopowy backend audio (Linux: PulseAudio/PipeWire -> ALSA -> Null; Windows: WinMM -> Null; macOS: Null).

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

use super::AudioBackend;

pub struct DesktopAudioBackend {
    inner: Box<dyn AudioBackend>,
}

impl DesktopAudioBackend {
    pub fn new() -> Self {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(pulse) = linux::PulseAudioDriver::try_open() {
                return Self {
                    inner: Box::new(pulse),
                };
            }
            if let Some(alsa) = linux::AlsaAudioDriver::try_open() {
                return Self {
                    inner: Box::new(alsa),
                };
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(winmm) = windows::WinMmAudioDriver::try_open() {
                return Self {
                    inner: Box::new(winmm),
                };
            }
        }

        Self {
            inner: Box::new(super::null::NullAudioBackend::new()),
        }
    }
}

impl Default for DesktopAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for DesktopAudioBackend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> Result<(), String> {
        self.inner.init(sample_rate, channels)
    }

    fn write_samples(&mut self, samples: &[f32]) {
        self.inner.write_samples(samples);
    }

    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}
