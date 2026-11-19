//! Natywny backend audio dla Sony PlayStation Vita (`SceAudioOut` + dedykowany wątek buforów PCM).
//! Target: `armv7-sony-vita-newlibeabihf`.

use std::sync::Mutex;
#[cfg(target_os = "vita")]
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::JoinHandle;

use super::AudioBackend;

const VITA_FRAMES_PER_BUFFER: usize = 1024;
const VITA_SAMPLES_PER_BUFFER: usize = VITA_FRAMES_PER_BUFFER * 2;
#[cfg(target_os = "vita")]
const VITA_BUFFER_POOL_SIZE: usize = 8;

type PcmBuffer = Box<[i16; VITA_SAMPLES_PER_BUFFER]>;

pub struct VitaAudioBackend {
    port: i32,
    active: bool,
    input_channels: usize,
    partial_input_frame: Vec<f32>,
    current_buffer: Option<PcmBuffer>,
    frames_in_current_buffer: usize,
    free_buffers_rx: Option<Mutex<Receiver<PcmBuffer>>>,
    output_tx: Option<SyncSender<PcmBuffer>>,
    output_thread: Option<JoinHandle<()>>,
}

impl VitaAudioBackend {
    pub fn new() -> Self {
        Self {
            port: -1,
            active: false,
            input_channels: 2,
            partial_input_frame: Vec::with_capacity(2),
            current_buffer: None,
            frames_in_current_buffer: 0,
            free_buffers_rx: None,
            output_tx: None,
            output_thread: None,
        }
    }

    #[cfg(target_os = "vita")]
    #[inline]
    fn to_i16(sample: f32) -> i16 {
        let finite = if sample.is_finite() { sample } else { 0.0 };
        (finite.clamp(-1.0, 1.0) * 32_767.0) as i16
    }

    #[cfg(target_os = "vita")]
    #[inline]
    fn push_stereo_frame(&mut self, left: f32, right: f32) {
        if !self.active {
            return;
        }

        if self.current_buffer.is_none() {
            let next = self
                .free_buffers_rx
                .as_ref()
                .and_then(|rx| rx.lock().ok())
                .and_then(|rx| rx.recv().ok());
            match next {
                Some(buffer) => self.current_buffer = Some(buffer),
                None => {
                    self.active = false;
                    crate::vita_log!("[AUDIO] PCM buffer pool disconnected; audio output stopped");
                    return;
                }
            }
        }

        let frame = self.frames_in_current_buffer;
        if let Some(buffer) = self.current_buffer.as_mut() {
            let sample_index = frame * 2;
            buffer[sample_index] = Self::to_i16(left);
            buffer[sample_index + 1] = Self::to_i16(right);
        } else {
            return;
        }

        self.frames_in_current_buffer += 1;
        if self.frames_in_current_buffer == VITA_FRAMES_PER_BUFFER {
            let Some(completed) = self.current_buffer.take() else {
                self.frames_in_current_buffer = 0;
                return;
            };
            self.frames_in_current_buffer = 0;

            match self.output_tx.as_ref() {
                Some(tx) => {
                    if tx.send(completed).is_err() {
                        self.active = false;
                        crate::vita_log!("[AUDIO] SceAudioOut worker disconnected");
                    }
                }
                None => {
                    self.active = false;
                }
            }
        }
    }
}

impl Default for VitaAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for VitaAudioBackend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> Result<(), String> {
        if self.active || self.output_thread.is_some() {
            return Err("Vita audio backend is already initialized".to_string());
        }

        let supported_rate = matches!(
            sample_rate,
            8_000 | 11_025 | 12_000 | 16_000 | 22_050 | 24_000 | 32_000 | 44_100 | 44_184 | 48_000
        );
        if !supported_rate {
            return Err(format!("Unsupported Vita audio sample rate: {sample_rate} Hz"));
        }

        if channels != 1 && channels != 2 {
            return Err(format!(
                "Vita audio backend supports mono or stereo input, got {channels} channels"
            ));
        }

        self.input_channels = channels as usize;
        self.partial_input_frame.clear();

        #[cfg(target_os = "vita")]
        {
            const SCE_AUDIO_OUT_PORT_TYPE_MAIN: i32 = 0;
            const SCE_AUDIO_OUT_MODE_STEREO: i32 = 1;
            const SCE_AUDIO_VOLUME_FLAG_LR: i32 = 3;

            let port = unsafe {
                sceAudioOutOpenPort(
                    SCE_AUDIO_OUT_PORT_TYPE_MAIN,
                    VITA_FRAMES_PER_BUFFER as i32,
                    sample_rate as i32,
                    SCE_AUDIO_OUT_MODE_STEREO,
                )
            };

            if port < 0 {
                let err = format!(
                    "sceAudioOutOpenPort(len={}, freq={}, stereo) failed: {:#010x}",
                    VITA_FRAMES_PER_BUFFER, sample_rate, port as u32
                );
                crate::vita_log!("[AUDIO] {}", err);
                return Err(err);
            }

            let volumes = [0x8000i32, 0x8000i32];
            let volume_result =
                unsafe { sceAudioOutSetVolume(port, SCE_AUDIO_VOLUME_FLAG_LR, volumes.as_ptr()) };
            if volume_result < 0 {
                unsafe {
                    sceAudioOutReleasePort(port);
                }
                let err = format!("sceAudioOutSetVolume failed: {:#010x}", volume_result as u32);
                crate::vita_log!("[AUDIO] {}", err);
                return Err(err);
            }

            let (output_tx, output_rx) = mpsc::sync_channel::<PcmBuffer>(VITA_BUFFER_POOL_SIZE);
            let (free_tx, free_rx) = mpsc::sync_channel::<PcmBuffer>(VITA_BUFFER_POOL_SIZE);

            for _ in 0..VITA_BUFFER_POOL_SIZE {
                if free_tx
                    .send(Box::new([0i16; VITA_SAMPLES_PER_BUFFER]))
                    .is_err()
                {
                    unsafe {
                        sceAudioOutReleasePort(port);
                    }
                    return Err("Failed to initialize Vita PCM buffer pool".to_string());
                }
            }

            let output_thread = std::thread::Builder::new()
                .name("vita_audio_out".to_string())
                .spawn(move || {
                    let mut consecutive_errors = 0u32;

                    while let Ok(buffer) = output_rx.recv() {
                        let result = unsafe {
                            sceAudioOutOutput(port, buffer.as_ptr() as *const std::ffi::c_void)
                        };

                        if result < 0 {
                            consecutive_errors = consecutive_errors.saturating_add(1);
                            if consecutive_errors == 1 || consecutive_errors % 128 == 0 {
                                crate::vita_log!(
                                    "[AUDIO] sceAudioOutOutput error {:#010x} (count={})",
                                    result as u32,
                                    consecutive_errors
                                );
                            }
                        } else {
                            consecutive_errors = 0;
                        }

                        if free_tx.send(buffer).is_err() {
                            break;
                        }
                    }

                    unsafe {
                        sceAudioOutReleasePort(port);
                    }
                    crate::vita_log!("[AUDIO] SceAudioOut worker stopped");
                });

            let output_thread = match output_thread {
                Ok(thread) => thread,
                Err(error) => {
                    unsafe {
                        sceAudioOutReleasePort(port);
                    }
                    return Err(format!("Failed to spawn Vita audio output thread: {error}"));
                }
            };

            self.port = port;
            self.free_buffers_rx = Some(Mutex::new(free_rx));
            self.output_tx = Some(output_tx);
            self.output_thread = Some(output_thread);
            self.active = true;

            crate::vita_log!(
                "[AUDIO] SceAudioOut initialized (port={}, len={} frames, freq={}, stereo, queued worker)",
                port,
                VITA_FRAMES_PER_BUFFER,
                sample_rate
            );
            Ok(())
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.active = true;
            Ok(())
        }
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active || samples.is_empty() {
            return;
        }

        #[cfg(target_os = "vita")]
        {
            let channels = self.input_channels;
            for &sample in samples {
                self.partial_input_frame.push(sample);
                if self.partial_input_frame.len() == channels {
                    let left = self.partial_input_frame[0];
                    let right = if channels == 1 {
                        left
                    } else {
                        self.partial_input_frame[1]
                    };
                    self.partial_input_frame.clear();
                    self.push_stereo_frame(left, right);
                    if !self.active {
                        break;
                    }
                }
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
        self.active = false;
        self.output_tx.take();
        self.current_buffer.take();
        self.frames_in_current_buffer = 0;

        if let Some(worker) = self.output_thread.take() {
            let _ = worker.join();
        }

        self.free_buffers_rx.take();
        self.port = -1;
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
