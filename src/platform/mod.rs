#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
pub mod gamepad;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
pub mod mouse_capture;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
pub use gamepad::{GamepadEvents, GamepadManager, GamepadState};
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
pub use mouse_capture::{maximize_window, MouseCapture};

#[cfg(target_os = "vita")]
pub mod vita;

#[cfg(target_os = "vita")]
pub mod vita_gxm;
