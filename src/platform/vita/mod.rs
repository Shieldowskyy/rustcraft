//! PlayStation Vita PAL facade (`armv7-sony-vita-newlibeabihf`).
//! Exposes VitaSDK FFI, crash-safe panic/log utilities, IME dialog, hardware display, input, and main loop.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod display;
mod ffi;
pub mod gxm;
mod ime;
mod input;
mod lifecycle;
mod panic_log;

pub use display::VitaDisplay;
pub use ffi::*;
pub use ime::VitaImeManager;
pub use input::VitaInputTracker;
pub use lifecycle::run_vita_app;
pub use panic_log::{install_vita_panic_hook, sync_crash_log, write_crash_bytes};
