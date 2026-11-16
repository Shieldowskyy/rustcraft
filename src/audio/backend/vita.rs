//! PlayStation Vita (PS Vita) native audio backend via VitaSDK SceAudioOut.
//! Target: `armv7-sony-vita-newlibeabihf`
//!
//! # PS Vita SceAudioOut constraints (vitasdk docs):
//!   - `sceAudioOutOpenPort` `len` MUST be a multiple of 64 (SCE_AUDIO_MIN_LEN).
//!   - Maximum `len` is 65 472 frames (SCE_AUDIO_MAX_LEN).
//!   - `sceAudioOutOutput` is *blocking* — it blocks until the hardware consumes the
//!     previous buffer, so it must receive exactly `len` stereo frames (i.e. `len * 2`
//!     i16 values) every call.  Sending a different count corrupts the stream or panics.
//!   - The hardware only accepts 48 000 Hz; other rates cause sceAudioOut to reject the
//!     port or produce garbage output.

use super::AudioBackend;

/// Number of stereo frames per hardware buffer.
///
/// Must be a multiple of 64 and must equal **exactly** the `scratch_buffer` frame
/// count used by the audio-engine worker thread.  The engine uses a fixed 2 048-sample
/// interleaved stereo buffer ⇒ 1 024 stereo frames.  1 024 / 64 = 16 ✓.
const VITA_FRAMES_PER_BUFFER: i32 = 1024;

pub struct VitaAudioBackend {
    #[allow(dead_code)] // read inside #[cfg(target_os = "vita")] blocks only
    port: i32,
    active: bool,
    /// Pre-allocated i16 scratch — sized for exactly VITA_FRAMES_PER_BUFFER stereo frames.
    /// Allocated once in `init` to avoid per-frame heap churn on Vita's slow allocator.
    pcm_scratch: Vec<i16>,
}

impl VitaAudioBackend {
    pub fn new() -> Self {
        Self {
            port: -1,
            active: false,
            // Zero-capacity — real allocation happens in init() so the size is known.
            pcm_scratch: Vec::new(),
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
        // Pre-allocate the i16 scratch buffer once so write_samples() never allocates.
        // VITA_FRAMES_PER_BUFFER stereo frames × 2 channels = total i16 samples needed.
        let scratch_capacity = (VITA_FRAMES_PER_BUFFER as usize) * 2;
        self.pcm_scratch = vec![0i16; scratch_capacity];

        #[cfg(target_os = "vita")]
        {
            // SCE_AUDIO_OUT_PORT_TYPE_MAIN = 0
            // len must be a multiple of 64 — VITA_FRAMES_PER_BUFFER = 1024 ✓
            // freq must be 48 000 Hz (hardware-native rate on Vita)
            // SCE_AUDIO_OUT_MODE_STEREO = 0
            let port = unsafe {
                sceAudioOutOpenPort(0, VITA_FRAMES_PER_BUFFER, 48_000, 0)
            };
            if port >= 0 {
                self.port = port;
                // SCE_AUDIO_VOLUME_0DB = 0x8000  (full volume, left & right)
                // flag = 3 means set both L and R channels
                let vols = [0x8000i32, 0x8000i32];
                unsafe { sceAudioOutSetVolume(port, 3, vols.as_ptr()) };
                self.active = true;
                Ok(())
            } else {
                Err(format!(
                    "sceAudioOutOpenPort(len={VITA_FRAMES_PER_BUFFER}, freq=48000) \
                     failed with error code {port:#010x}"
                ))
            }
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.active = true;
            Ok(())
        }
    }

    /// Converts `samples` (interleaved stereo f32) to i16 PCM and feeds the Vita hardware.
    ///
    /// `samples.len()` must equal `VITA_FRAMES_PER_BUFFER * 2` (= 2 048).
    /// The engine worker always passes exactly that many samples; any other length is
    /// silently skipped to prevent passing a mismatched buffer to `sceAudioOutOutput`.
    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active {
            return;
        }

        let expected = (VITA_FRAMES_PER_BUFFER as usize) * 2;

        // Guard: sceAudioOutOutput is a blocking call that expects EXACTLY `len` stereo
        // frames.  Sending the wrong amount crashes or corrupts the audio DMA engine.
        if samples.len() != expected {
            return;
        }

        // Convert f32 [-1.0, 1.0] → i16 in-place using the pre-allocated scratch.
        // No allocation happens here.
        for (dst, &src) in self.pcm_scratch.iter_mut().zip(samples.iter()) {
            *dst = (src * 32_767.0).clamp(-32_768.0, 32_767.0) as i16;
        }

        #[cfg(target_os = "vita")]
        if self.port >= 0 {
            // sceAudioOutOutput blocks until the hardware DMA consumes the previous
            // buffer, then copies the new one.  This is the correct Vita audio pattern.
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
