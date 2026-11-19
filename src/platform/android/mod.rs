//! Android Native PAL facade (ANativeWindow surface, multi-touch virtual controls, JNI bridge).
//! Exposes `android_log`, `TouchTracker`, `AndroidGameContext`, and exported JNI activity entrypoints.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod ffi;
mod jni;
mod lifecycle;
mod overlay;
mod touch;
mod touch_layout;

pub use ffi::*;
pub use jni::*;
pub use lifecycle::*;
pub use touch::TouchTracker;
