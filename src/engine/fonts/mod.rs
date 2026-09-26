//! # Modular Engine Font Subsystem
//!
//! Provides abstract bitmap font interfaces, font identifiers, registry helpers,
//! and built-in typography for rendering text across all supported platforms.

pub mod rustcraft_light;
pub mod rustcraft_regular;

pub use rustcraft_light::RustcraftLight;
pub use rustcraft_regular::{RustcraftRegular, GLYPH_H, GLYPH_W};

/// Common interface for modular engine bitmap fonts.
pub trait BitmapFont: Send + Sync {
    /// Returns the human-readable name of the font.
    fn name(&self) -> &'static str;

    /// Base width of individual glyphs in pixels.
    fn glyph_width(&self) -> usize;

    /// Base height of individual glyphs in pixels.
    fn glyph_height(&self) -> usize;

    /// Plots each active pixel of `text` rendered at (`origin_x`, `origin_y`).
    fn render_text(
        &self,
        text: &str,
        origin_x: i32,
        origin_y: i32,
        scale: i32,
        spacing: i32,
        plot: &mut dyn FnMut(i32, i32),
    );

    /// Computes the pixel width of rendered `text`.
    fn measure(&self, text: &str, scale: i32, spacing: i32) -> i32;
}

/// Enumeration of all registered engine fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontId {
    /// Default compact 3x5 bitmap font.
    RustcraftRegular,
    /// Slender, open 3x5 font optimized for profiler telemetry and diagnostics.
    RustcraftLight,
}

static RUSTCRAFT_REGULAR_INSTANCE: RustcraftRegular = RustcraftRegular;
static RUSTCRAFT_LIGHT_INSTANCE: RustcraftLight = RustcraftLight;

/// Returns a static reference to the font identified by `id`.
#[inline]
pub fn get_font(id: FontId) -> &'static dyn BitmapFont {
    match id {
        FontId::RustcraftRegular => &RUSTCRAFT_REGULAR_INSTANCE,
        FontId::RustcraftLight => &RUSTCRAFT_LIGHT_INSTANCE,
    }
}

/// Returns the default engine font (`RustcraftRegular`).
#[inline]
pub fn default_font() -> &'static dyn BitmapFont {
    &RUSTCRAFT_REGULAR_INSTANCE
}

/// Returns a list of names of all available fonts in the engine.
pub fn list_fonts() -> &'static [&'static str] {
    &["RustcraftRegular", "RustcraftLight"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_registry_and_measurement() {
        let font = get_font(FontId::RustcraftRegular);
        assert_eq!(font.name(), "RustcraftRegular");
        assert_eq!(font.glyph_width(), 3);
        assert_eq!(font.glyph_height(), 5);

        let width = font.measure("HELLO", 1, 1);
        // 5 chars * 3 px + 4 spacing * 1 = 19
        assert_eq!(width, 19);

        let light = get_font(FontId::RustcraftLight);
        assert_eq!(light.name(), "RustcraftLight");
        assert_eq!(light.glyph_width(), 3);
        assert_eq!(light.glyph_height(), 5);
        assert_eq!(light.measure("FPS: 60", 1, 1), 7 * 3 + 6 * 1);

        let list = list_fonts();
        assert_eq!(list, &["RustcraftRegular", "RustcraftLight"]);
    }
}
