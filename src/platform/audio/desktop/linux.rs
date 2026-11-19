//! Dynamiczne sterowniki audio dla Linuksa/FreeBSD (`libpulse-simple.so.0` oraz `libasound.so.2`) ładowane przez `dlopen`.

use std::ffi::c_void;

use super::super::AudioBackend;

pub(super) struct PulseAudioDriver {
    handle: *mut c_void,
    pa_simple: *mut c_void,
    fn_write: unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut i32) -> i32,
    fn_free: unsafe extern "C" fn(*mut c_void),
    active: bool,
}

unsafe impl Send for PulseAudioDriver {}

#[repr(C)]
struct PaSampleSpec {
    format: i32,
    rate: u32,
    channels: u8,
}

#[repr(C)]
struct PaBufferAttr {
    maxlength: u32,
    tlength: u32,
    prebuf: u32,
    minreq: u32,
    fragsize: u32,
}

impl PulseAudioDriver {
    pub fn try_open() -> Option<Self> {
        let lib_names: &[&[u8]] = &[b"libpulse-simple.so.0\0", b"libpulse-simple.so\0"];

        let mut lib = std::ptr::null_mut();
        for &name in lib_names {
            lib = unsafe { dlopen(name.as_ptr() as *const i8, 1) };
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
            format: 5,
            rate: 48000,
            channels: 2,
        };

        let attr = PaBufferAttr {
            maxlength: u32::MAX,
            tlength: 16384,
            prebuf: u32::MAX,
            minreq: 4096,
            fragsize: u32::MAX,
        };

        let mut error = 0i32;
        let app_name = b"RustCraft\0";
        let stream_name = b"Game Audio\0";

        let pa_simple = unsafe {
            fn_new(
                std::ptr::null(),
                app_name.as_ptr() as *const i8,
                1,
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

pub(super) struct AlsaAudioDriver {
    lib: *mut c_void,
    pcm: *mut c_void,
    fn_writei: unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> isize,
    fn_close: unsafe extern "C" fn(*mut c_void) -> i32,
    active: bool,
}

unsafe impl Send for AlsaAudioDriver {}

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

extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
}
