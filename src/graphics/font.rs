//! # Built-in Engine Typography & Font Module
//!
//! Provides text measuring, font constants, and pixel plotting functions backed
//! by the modular [`crate::engine::fonts::RustcraftRegular`] and [`crate::engine::fonts::RustcraftLight`] engine fonts.

use std::sync::atomic::{AtomicBool, Ordering};

static FORCE_UNICODE: AtomicBool = AtomicBool::new(false);

/// Sets whether unicode / high-legibility light font is forced globally across UI.
pub fn set_force_unicode(force: bool) {
    FORCE_UNICODE.store(force, Ordering::Relaxed);
}

/// Checks whether unicode / high-legibility light font is currently forced.
pub fn is_force_unicode() -> bool {
    FORCE_UNICODE.load(Ordering::Relaxed)
}

pub use crate::engine::fonts::rustcraft_light::{
    draw_text as draw_text_light, measure_text as measure_text_light, RustcraftLight,
};
pub use crate::engine::fonts::rustcraft_regular::{
    GLYPH_H, GLYPH_W, RustcraftRegular,
};
pub use crate::engine::fonts::{BitmapFont, FontId, default_font, get_font, list_fonts};

/// Retrieves the raw glyph bitmask for character `c` from the active or forced font.
#[inline]
pub fn glyph(c: char) -> [u8; 5] {
    if is_force_unicode() {
        crate::engine::fonts::rustcraft_light::glyph(c)
    } else {
        crate::engine::fonts::rustcraft_regular::glyph(c)
    }
}

/// Retrieves the raw glyph bitmask for character `c` from the RustcraftLight font.
#[inline]
pub fn glyph_light(c: char) -> [u8; 5] {
    crate::engine::fonts::rustcraft_light::glyph(c)
}

/// Renders text using either the standard or forced unicode font.
#[inline]
pub fn draw_text(text: &str, origin_x: i32, origin_y: i32, scale: i32, spacing: i32, plot: impl FnMut(i32, i32)) {
    if is_force_unicode() {
        crate::engine::fonts::rustcraft_light::draw_text(text, origin_x, origin_y, scale, spacing, plot);
    } else {
        crate::engine::fonts::rustcraft_regular::draw_text(text, origin_x, origin_y, scale, spacing, plot);
    }
}

/// Measures text width in pixels using either the standard or forced unicode font.
#[inline]
pub fn measure_text(text: &str, scale: i32, spacing: i32) -> i32 {
    if is_force_unicode() {
        crate::engine::fonts::rustcraft_light::measure_text(text, scale, spacing)
    } else {
        crate::engine::fonts::rustcraft_regular::measure_text(text, scale, spacing)
    }
}
