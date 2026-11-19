//! Low-level Android NDK `ANativeWindow` and Logcat FFI declarations.
//! Provides `android_log` and raw native surface locking/posting bindings.

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::CString;

use jni_sys::{jobject, JNIEnv};

// ============================================================================
// 1. Android NDK Low-Level FFI Definitions
// ============================================================================

pub enum ANativeWindow {}

#[repr(C)]
pub struct ANativeWindow_Buffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut std::ffi::c_void,
    pub reserved: [u32; 6],
}

pub const WINDOW_FORMAT_RGBA_8888: i32 = 1;
pub const WINDOW_FORMAT_RGBX_8888: i32 = 2;
pub const WINDOW_FORMAT_RGB_565: i32 = 4;

pub const ANDROID_LOG_DEBUG: i32 = 3;
pub const ANDROID_LOG_INFO: i32 = 4;
pub const ANDROID_LOG_WARN: i32 = 5;
pub const ANDROID_LOG_ERROR: i32 = 6;

#[link(name = "android")]
extern "C" {
    pub fn ANativeWindow_fromSurface(env: *mut JNIEnv, surface: jobject) -> *mut ANativeWindow;
    pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
    pub fn ANativeWindow_setBuffersGeometry(
        window: *mut ANativeWindow,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    pub fn ANativeWindow_lock(
        window: *mut ANativeWindow,
        outBuffer: *mut ANativeWindow_Buffer,
        inOutDirtyBounds: *mut std::ffi::c_void,
    ) -> i32;
    pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
}

#[link(name = "log")]
extern "C" {
    pub fn __android_log_write(
        prio: i32,
        tag: *const std::os::raw::c_char,
        text: *const std::os::raw::c_char,
    ) -> i32;
}

pub fn android_log(level: i32, tag: &str, msg: &str) {
    if let (Ok(c_tag), Ok(c_msg)) = (CString::new(tag), CString::new(msg)) {
        unsafe {
            __android_log_write(level, c_tag.as_ptr(), c_msg.as_ptr());
        }
    }
}

