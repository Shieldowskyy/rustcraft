//! Modular UI framework facade (`Rect`, `VStack`, `HStack`, `Grid`, `Button`, `TextInput`, `Toast`, `UiCanvas`).
//! Zero platform `#[cfg]` and zero heap allocations in hot render loops.

mod canvas;
mod controls;
mod layout;
mod types;

#[cfg(test)]
mod tests;

pub use canvas::UiCanvas;
pub use layout::{Grid, HStack, Rect, VStack};
pub use types::{Button, ButtonVariant, TextInput, Toast};
