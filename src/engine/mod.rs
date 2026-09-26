//! # RustCraft Core Voxel Engine API
//!
//! Clean, modular, and human-readable engine abstractions designed for rapid feature development,
//! custom platform porting, and plugin extension.
//!
//! ## Subsystems
//!
//! - [`platform`]: Abstract OS windowing, event polling, input polling, and display presentation.
//! - [`registry`]: Extensible voxel/block properties, flammability, luminance, and hardness metadata.
//! - [`module`]: Pluggable lifecycle hooks for gameplay modules, mod extensions, and custom subsystems.
//!
//! ## Adding a New Platform Backend
//!
//! To port RustCraft to a new OS or handheld console (e.g. Nintendo Switch, Android, SDL2),
//! implement the [`platform::PlatformBackend`] trait.
//!
//! ## Adding a New Engine Module
//!
//! Implement [`module::EngineModule`] and register it with the [`module::ModuleManager`].

pub mod platform;
pub mod registry;
pub mod module;
pub mod fonts;

pub use platform::{PlatformBackend, PlatformConfig};
pub use registry::{BlockProperties, BlockRegistry};
pub use module::{EngineModule, ModuleContext, ModuleManager};
pub use fonts::{BitmapFont, FontId, RustcraftLight, RustcraftRegular, default_font, get_font, list_fonts};
pub use crate::graphics::traits::RenderBackend;

/// Core application lifecycle interface.
pub trait EngineApp {
    /// Advances game simulation by `dt` seconds given current input state.
    fn update(&mut self, dt: f32, input: &crate::app::InputState) -> crate::app::AppOutput;

    /// Renders current frame to the internal framebuffer.
    fn render(&mut self);

    /// Resizes the engine framebuffer resolution.
    fn resize(&mut self, width: usize, height: usize);

    /// Returns the active 32-bit ARGB/RGBA color buffer slice.
    fn pixel_buffer(&self) -> &[u32];
}
