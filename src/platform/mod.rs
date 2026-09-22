#[cfg(not(target_arch = "wasm32"))]
pub mod gamepad;
#[cfg(not(target_arch = "wasm32"))]
pub mod mouse_capture;

#[cfg(not(target_arch = "wasm32"))]
pub use gamepad::{GamepadEvents, GamepadManager, GamepadState};
#[cfg(not(target_arch = "wasm32"))]
pub use mouse_capture::{maximize_window, MouseCapture};

