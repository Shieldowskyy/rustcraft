//! Android native audio backend via AAudio with low-latency PCM float streaming.
//!
//! Uses zero-cost dynamic linking (`dlopen` / `dlsym`) against Android's native `libaaudio.so`.
//! Provides real-time polyphonic procedural sound playback directly to Android hardware.

use std::ffi::c_void;
use super::AudioBackend;

pub const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
pub const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;
pub const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
pub const AAUDIO_SHARING_MODE_SHARED: i32 = 0;
pub const AAUDIO_OK: i32 = 0;

#[inline]
fn log_android(level: i32, tag: &str, msg: &str) {
    #[cfg(target_os = "android")]
    {
        crate::platform::android::android_log(level, tag, msg);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (level, tag, msg);
    }
}

extern "C" {
    fn dlopen(filename: *const std::os::raw::c_char, flags: std::os::raw::c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const std::os::raw::c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> std::os::raw::c_int;
}

pub struct AndroidAudioBackend {
    lib_handle: *mut c_void,
    stream: *mut c_void,
    fn_write: Option<unsafe extern "C" fn(*mut c_void, *const c_void, i32, i64) -> i32>,
    fn_request_stop: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    fn_close: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    active: bool,
}

unsafe impl Send for AndroidAudioBackend {}

impl AndroidAudioBackend {
    pub fn new() -> Self {
        Self {
            lib_handle: std::ptr::null_mut(),
            stream: std::ptr::null_mut(),
            fn_write: None,
            fn_request_stop: None,
            fn_close: None,
            active: false,
        }
    }
}

impl Default for AndroidAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for AndroidAudioBackend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> Result<(), String> {
        let lib_names: &[&[u8]] = &[b"libaaudio.so\0"];
        let mut lib = std::ptr::null_mut();

        for &name in lib_names {
            lib = unsafe { dlopen(name.as_ptr() as *const std::os::raw::c_char, 1) }; // RTLD_LAZY = 1
            if !lib.is_null() {
                break;
            }
        }

        if lib.is_null() {
            log_android(
                5, // ANDROID_LOG_WARN
                "RustCraftAudio",
                "AAudio library (libaaudio.so) not available on this Android build",
            );
            return Err("libaaudio.so not found".to_string());
        }

        unsafe {
            let fn_create_builder: unsafe extern "C" fn(*mut *mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudio_createStreamBuilder\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudio_createStreamBuilder symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_direction: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setDirection\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setDirection symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_sample_rate: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setSampleRate\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setSampleRate symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_channel_count: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setChannelCount\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setChannelCount symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_format: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setFormat\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setFormat symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_performance_mode: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setPerformanceMode\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setPerformanceMode symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_set_sharing_mode: unsafe extern "C" fn(*mut c_void, i32) = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_setSharingMode\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_setSharingMode symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_open_stream: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_openStream\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_openStream symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_builder_delete: unsafe extern "C" fn(*mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudioStreamBuilder_delete\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStreamBuilder_delete symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_stream_request_start: unsafe extern "C" fn(*mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudioStream_requestStart\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStream_requestStart symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_stream_request_stop: unsafe extern "C" fn(*mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudioStream_requestStop\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStream_requestStop symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_stream_write: unsafe extern "C" fn(*mut c_void, *const c_void, i32, i64) -> i32 = {
                let sym = dlsym(lib, b"AAudioStream_write\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStream_write symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let fn_stream_close: unsafe extern "C" fn(*mut c_void) -> i32 = {
                let sym = dlsym(lib, b"AAudioStream_close\0".as_ptr() as *const _);
                if sym.is_null() {
                    dlclose(lib);
                    return Err("Missing AAudioStream_close symbol".to_string());
                }
                std::mem::transmute(sym)
            };

            let mut builder: *mut c_void = std::ptr::null_mut();
            let res = fn_create_builder(&mut builder);
            if res != AAUDIO_OK || builder.is_null() {
                dlclose(lib);
                return Err(format!("AAudio_createStreamBuilder failed: {}", res));
            }

            fn_builder_set_direction(builder, AAUDIO_DIRECTION_OUTPUT);
            fn_builder_set_sample_rate(builder, sample_rate as i32);
            fn_builder_set_channel_count(builder, channels as i32);
            fn_builder_set_format(builder, AAUDIO_FORMAT_PCM_FLOAT);
            fn_builder_set_performance_mode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
            fn_builder_set_sharing_mode(builder, AAUDIO_SHARING_MODE_SHARED);

            let mut stream: *mut c_void = std::ptr::null_mut();
            let open_res = fn_builder_open_stream(builder, &mut stream);
            fn_builder_delete(builder);

            if open_res != AAUDIO_OK || stream.is_null() {
                dlclose(lib);
                return Err(format!("AAudioStreamBuilder_openStream failed: {}", open_res));
            }

            let start_res = fn_stream_request_start(stream);
            if start_res != AAUDIO_OK {
                fn_stream_close(stream);
                dlclose(lib);
                return Err(format!("AAudioStream_requestStart failed: {}", start_res));
            }

            self.lib_handle = lib;
            self.stream = stream;
            self.fn_write = Some(fn_stream_write);
            self.fn_request_stop = Some(fn_stream_request_stop);
            self.fn_close = Some(fn_stream_close);
            self.active = true;

            log_android(
                4, // ANDROID_LOG_INFO
                "RustCraftAudio",
                &format!(
                    "AAudio stream successfully opened ({} Hz, {} channels, low-latency PCM float)",
                    sample_rate, channels
                ),
            );
        }

        Ok(())
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active || self.stream.is_null() {
            return;
        }

        if let Some(write_fn) = self.fn_write {
            let num_frames = (samples.len() / 2) as i32;
            // 15ms timeout prevents underrun and avoids blocking the audio thread indefinitely
            let timeout_ns = 15_000_000i64;
            unsafe {
                let _ = write_fn(self.stream, samples.as_ptr() as *const c_void, num_frames, timeout_ns);
            }
        }
    }

    fn is_active(&self) -> bool {
        self.active && !self.stream.is_null()
    }

    fn name(&self) -> &'static str {
        "Android AAudio (Low-Latency Native)"
    }
}

impl Drop for AndroidAudioBackend {
    fn drop(&mut self) {
        if !self.stream.is_null() {
            if let Some(stop_fn) = self.fn_request_stop {
                unsafe { stop_fn(self.stream) };
            }
            if let Some(close_fn) = self.fn_close {
                unsafe { close_fn(self.stream) };
            }
            self.stream = std::ptr::null_mut();
        }

        if !self.lib_handle.is_null() {
            unsafe { dlclose(self.lib_handle) };
            self.lib_handle = std::ptr::null_mut();
        }
    }
}
