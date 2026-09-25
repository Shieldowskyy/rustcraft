//! Platform abstraction layer for cross-platform engine hosting.
//!
//! Provides traits and contracts to facilitate porting RustCraft to new target platforms
//! (e.g. Nintendo Switch, Android, iOS, SDL2, Raylib, Linux DRM/KMS, Raspberry Pi) without
//! touching game logic or voxel meshing pipelines.

use crate::app::{AppOutput, InputState};

/// Configuration options for creating a platform host window / context.
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    /// Window or framebuffer title.
    pub title: String,
    /// Logical render width in pixels.
    pub width: usize,
    /// Logical render height in pixels.
    pub height: usize,
    /// Desired fullscreen presentation mode.
    pub fullscreen: bool,
    /// Vertical sync enabled.
    pub vsync: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            title: "RustCraft".to_string(),
            width: 960,
            height: 540,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// Abstract hardware/OS platform backend interface.
///
/// Implement this trait to port RustCraft to a new operating system, console SDK,
/// or windowing environment (e.g. GLFW, SDL2, LibNX on Nintendo Switch, Android NDK).
pub trait PlatformBackend {
    /// Initializes the platform backend and creates any required OS surfaces.
    fn init(&mut self, config: &PlatformConfig) -> Result<(), String>;

    /// Polls hardware inputs, OS events, and gamepads, populating `input`.
    fn poll_events(&mut self, input: &mut InputState) -> bool;

    /// Presents a 32-bit ARGB/RGBA framebuffer to the display screen.
    fn present_framebuffer(&mut self, buffer: &[u32], width: usize, height: usize) -> Result<(), String>;

    /// Updates window title or display header text.
    fn set_title(&mut self, title: &str);

    /// Locks the hardware mouse cursor to the center of the window.
    fn capture_mouse(&mut self, captured: bool);

    /// Returns the system high-resolution monotonic time in seconds.
    fn current_time_seconds(&self) -> f64;

    /// Handles application lifecycle requests from engine output.
    fn handle_output(&mut self, output: &AppOutput) {
        if output.request_capture_mouse {
            self.capture_mouse(true);
        }
        if output.request_release_mouse {
            self.capture_mouse(false);
        }
        if !output.title.is_empty() {
            self.set_title(&output.title);
        }
    }
}
