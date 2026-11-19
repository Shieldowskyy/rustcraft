//! Modular JSON controls system facade (`Action`, `KeyCode`, `GamepadInput`, `MouseInput`, `ControlsManager`).

mod actions;
mod config;
mod devices;
mod json;
mod keycode;
mod manager;

pub use actions::{Action, ActionBinding, ACTION_COUNT};
pub use config::ControlsConfig;
pub use devices::{GamepadInput, MouseInput};
pub use keycode::KeyCode;
pub use manager::ControlsManager;

