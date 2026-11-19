//! macOS (Apple Silicon & Intel) platform profile and Darwin integration hooks.
//! Uses `minifb` windowing + `wgpu` Metal backend (via `desktop` host) and `~/.config/rustcraft` storage.

use std::path::PathBuf;

/// Default macOS graphics backend identifier passed to `Renderer`.
pub const MACOS_PREFERRED_RENDERER: &str = "metal";

/// Returns the macOS saves directory (`$HOME/.config/rustcraft/saves`).
pub fn macos_saves_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        let dir = home.join(".config").join("rustcraft").join("saves");
        if std::fs::create_dir_all(&dir).is_ok() {
            return dir;
        }
    }
    PathBuf::from(".")
}

