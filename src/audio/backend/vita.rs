//! PlayStation Vita (PS Vita) native audio backend via VitaSDK SceAudioOut.
//! Target: `armv7-sony-vita-newlibeabihf`

use super::AudioBackend;

pub struct VitaAudioBackend {
    #[allow(dead_code)]
    port: i32,
    active: bool,
    pcm_scratch: Vec<i16>,
}

impl VitaAudioBackend {
    pub fn new() -> Self {
        Self {
            port: -1,
            active: false,
            pcm_scratch: Vec::with_capacity(1024),
        }
    }
}

impl Default for VitaAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for VitaAudioBackend {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        #[cfg(target_os = "vita")]
        {
            // SCE_AUDIO_OUT_PORT_TYPE_MAIN = 0, SCE_AUDIO_OUT_MODE_STEREO = 0
            let port = unsafe { sceAudioOutOpenPort(0, 512, 48000, 0) };
            if port >= 0 {
                self.port = port;
                let vols = [0x8000, 0x8000]; // 100% hardware volume
                unsafe { sceAudioOutSetVolume(port, 3, vols.as_ptr()) };
                self.active = true;
                return Ok(());
            } else {
                return Err(format!("sceAudioOutOpenPort failed with code {port}"));
            }
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.active = true;
            Ok(())
        }
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active {
            return;
        }

        // Convert 32-bit float [-1.0, 1.0] to 16-bit signed PCM
        self.pcm_scratch.clear();
        self.pcm_scratch.reserve(samples.len());
        for &s in samples {
            let clamped = (s * 32767.0).clamp(-32768.0, 32767.0);
            self.pcm_scratch.push(clamped as i16);
        }

        #[cfg(target_os = "vita")]
        if self.port >= 0 {
            unsafe {
                sceAudioOutOutput(self.port, self.pcm_scratch.as_ptr() as *const _);
            }
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn name(&self) -> &'static str {
        "Sony PlayStation Vita SceAudioOut"
    }
}

impl Drop for VitaAudioBackend {
    fn drop(&mut self) {
        #[cfg(target_os = "vita")]
        if self.port >= 0 {
            unsafe {
                sceAudioOutReleasePort(self.port);
            }
            self.port = -1;
            self.active = false;
        }
    }
}

#[cfg(target_os = "vita")]
#[link(name = "SceAudio_stub")]
extern "C" {
    fn sceAudioOutOpenPort(port_type: i32, len: i32, freq: i32, mode: i32) -> i32;
    fn sceAudioOutOutput(port: i32, buf: *const std::ffi::c_void) -> i32;
    fn sceAudioOutSetVolume(port: i32, ch: i32, vol: *const i32) -> i32;
    fn sceAudioOutReleasePort(port: i32) -> i32;
}
