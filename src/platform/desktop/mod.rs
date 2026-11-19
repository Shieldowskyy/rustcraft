//! Desktop platform backend (`minifb` windowing, `gilrs` gamepads, native mouse capture).

pub mod gamepad;
mod input_map;
pub mod mouse_capture;
mod window;

pub use gamepad::{GamepadEvents, GamepadManager, GamepadState};
pub use mouse_capture::{maximize_window, MouseCapture};
pub use window::run_desktop_app;

