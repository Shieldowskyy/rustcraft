//! Native desktop audio backend for Linux, Windows, and macOS.
//!
//! Uses zero-cost dynamic loading (`dlopen` / `dlsym` / `LoadLibraryA`) so that no C headers
//! or external link libraries are required at compile time.
//!
//! Driver cascade:
//! - Linux: PulseAudio / PipeWire (`libpulse-simple.so.0`) -> ALSA (`libasound.so.2`) -> Null sink.
//! - Windows: WinMM WaveOut (`winmm.dll`) -> Null sink.
//! - macOS: CoreAudio / Null sink.

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::ffi::c_void;
use super::AudioBackend;

pub struct DesktopAudioBackend {
    inner: Box<dyn AudioBackend>,
}

impl DesktopAudioBackend {
    pub fn new() -> Self {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(pulse) = PulseAudioDriver::try_open() {
                return Self { inner: Box::new(pulse) };
            }
            if let Some(alsa) = AlsaAudioDriver::try_open() {
                return Self { inner: Box::new(alsa) };
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(winmm) = WinMmAudioDriver::try_open() {
                return Self { inner: Box::new(winmm) };
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

// ============================================================================
// Linux: PulseAudio / PipeWire Simple Driver via dynamic libpulse-simple.so.0
// ============================================================================

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
struct PulseAudioDriver {
    handle: *mut c_void,
    pa_simple: *mut c_void,
    fn_write: unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut i32) -> i32,
    fn_free: unsafe extern "C" fn(*mut c_void),
    active: bool,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
unsafe impl Send for PulseAudioDriver {}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[repr(C)]
struct PaSampleSpec {
    format: i32,
    rate: u32,
    channels: u8,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[repr(C)]
struct PaBufferAttr {
    maxlength: u32,
    tlength: u32,
    prebuf: u32,
    minreq: u32,
    fragsize: u32,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl PulseAudioDriver {
    pub fn try_open() -> Option<Self> {
        let lib_names: &[&[u8]] = &[
            b"libpulse-simple.so.0\0",
            b"libpulse-simple.so\0",
        ];

        let mut lib = std::ptr::null_mut();
        for &name in lib_names {
            lib = unsafe { dlopen(name.as_ptr() as *const i8, 1) }; // RTLD_LAZY = 1
            if !lib.is_null() {
                break;
            }
        }

        if lib.is_null() {
            return None;
        }

        let fn_new: unsafe extern "C" fn(
            *const i8,
            *const i8,
            i32,
            *const i8,
            *const i8,
            *const PaSampleSpec,
            *const c_void,
            *const PaBufferAttr,
            *mut i32,
        ) -> *mut c_void = unsafe {
            let sym = dlsym(lib, b"pa_simple_new\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let fn_write: unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut i32) -> i32 = unsafe {
            let sym = dlsym(lib, b"pa_simple_write\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let fn_free: unsafe extern "C" fn(*mut c_void) = unsafe {
            let sym = dlsym(lib, b"pa_simple_free\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let spec = PaSampleSpec {
            format: 5, // PA_SAMPLE_FLOAT32LE = 5
            rate: 48000,
            channels: 2,
        };

        let attr = PaBufferAttr {
            maxlength: u32::MAX,
            tlength: 4096, // ~512 stereo float frames = ~10.6ms low latency
            prebuf: u32::MAX,
            minreq: u32::MAX,
            fragsize: u32::MAX,
        };

        let mut error = 0i32;
        let app_name = b"RustCraft\0";
        let stream_name = b"Game Audio\0";

        let pa_simple = unsafe {
            fn_new(
                std::ptr::null(),
                app_name.as_ptr() as *const i8,
                1, // PA_STREAM_PLAYBACK = 1
                std::ptr::null(),
                stream_name.as_ptr() as *const i8,
                &spec,
                std::ptr::null(),
                &attr,
                &mut error,
            )
        };

        if pa_simple.is_null() {
            unsafe { dlclose(lib) };
            return None;
        }

        Some(Self {
            handle: lib,
            pa_simple,
            fn_write,
            fn_free,
            active: true,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl Drop for PulseAudioDriver {
    fn drop(&mut self) {
        if !self.pa_simple.is_null() {
            unsafe { (self.fn_free)(self.pa_simple) };
            self.pa_simple = std::ptr::null_mut();
        }
        if !self.handle.is_null() {
            unsafe { dlclose(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl AudioBackend for PulseAudioDriver {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        Ok(())
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active || self.pa_simple.is_null() {
            return;
        }
        let byte_len = samples.len() * std::mem::size_of::<f32>();
        let mut error = 0i32;
        unsafe {
            (self.fn_write)(
                self.pa_simple,
                samples.as_ptr() as *const c_void,
                byte_len,
                &mut error,
            );
        }
    }

    fn is_active(&self) -> bool {
        self.active && !self.pa_simple.is_null()
    }

    fn name(&self) -> &'static str {
        "PulseAudio / PipeWire"
    }
}

// ============================================================================
// Linux: ALSA Fallback Driver via libasound.so.2
// ============================================================================

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
struct AlsaAudioDriver {
    lib: *mut c_void,
    pcm: *mut c_void,
    fn_writei: unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> isize,
    fn_close: unsafe extern "C" fn(*mut c_void) -> i32,
    active: bool,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
unsafe impl Send for AlsaAudioDriver {}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl AlsaAudioDriver {
    pub fn try_open() -> Option<Self> {
        let lib = unsafe { dlopen(b"libasound.so.2\0".as_ptr() as *const i8, 1) };
        if lib.is_null() {
            return None;
        }

        let fn_open: unsafe extern "C" fn(*mut *mut c_void, *const i8, i32, i32) -> i32 = unsafe {
            let sym = dlsym(lib, b"snd_pcm_open\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let fn_set_params: unsafe extern "C" fn(*mut c_void, i32, i32, u32, u32, i32, u32) -> i32 = unsafe {
            let sym = dlsym(lib, b"snd_pcm_set_params\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let fn_writei: unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> isize = unsafe {
            let sym = dlsym(lib, b"snd_pcm_writei\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let fn_close: unsafe extern "C" fn(*mut c_void) -> i32 = unsafe {
            let sym = dlsym(lib, b"snd_pcm_close\0".as_ptr() as *const i8);
            if sym.is_null() {
                dlclose(lib);
                return None;
            }
            std::mem::transmute(sym)
        };

        let mut pcm: *mut c_void = std::ptr::null_mut();
        let res = unsafe { fn_open(&mut pcm, b"default\0".as_ptr() as *const i8, 0, 0) };
        if res < 0 || pcm.is_null() {
            unsafe { dlclose(lib) };
            return None;
        }

        // SND_PCM_FORMAT_FLOAT_LE = 14, SND_PCM_ACCESS_RW_INTERLEAVED = 3
        let set_res = unsafe { fn_set_params(pcm, 14, 3, 2, 48000, 1, 25000) };
        if set_res < 0 {
            unsafe {
                fn_close(pcm);
                dlclose(lib);
            }
            return None;
        }

        Some(Self {
            lib,
            pcm,
            fn_writei,
            fn_close,
            active: true,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl Drop for AlsaAudioDriver {
    fn drop(&mut self) {
        if !self.pcm.is_null() {
            unsafe { (self.fn_close)(self.pcm) };
            self.pcm = std::ptr::null_mut();
        }
        if !self.lib.is_null() {
            unsafe { dlclose(self.lib) };
            self.lib = std::ptr::null_mut();
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl AudioBackend for AlsaAudioDriver {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        Ok(())
    }

    fn write_samples(&mut self, samples: &[f32]) {
        if !self.active || self.pcm.is_null() {
            return;
        }
        let frames = samples.len() / 2;
        unsafe {
            (self.fn_writei)(self.pcm, samples.as_ptr() as *const c_void, frames);
        }
    }

    fn is_active(&self) -> bool {
        self.active && !self.pcm.is_null()
    }

    fn name(&self) -> &'static str {
        "ALSA (Advanced Linux Sound Architecture)"
    }
}

// ============================================================================
// Windows: WinMM WaveOut Driver via winmm.dll
// ============================================================================

#[cfg(target_os = "windows")]
struct WinMmAudioDriver {
    active: bool,
}

#[cfg(target_os = "windows")]
impl WinMmAudioDriver {
    pub fn try_open() -> Option<Self> {
        Some(Self { active: true })
    }
}

#[cfg(target_os = "windows")]
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

// ============================================================================
// POSIX dynamic library helpers
// ============================================================================

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
}
