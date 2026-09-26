//! # Built-in Engine Typography & Font Module
//!
//! Provides text measuring, font constants, and pixel plotting functions backed
//! by the modular [`crate::engine::fonts::RustcraftRegular`] engine font.

pub use crate::engine::fonts::rustcraft_light::{
    draw_text as draw_text_light, measure_text as measure_text_light, RustcraftLight,
};
pub use crate::engine::fonts::rustcraft_regular::{
    draw_text, measure_text, GLYPH_H, GLYPH_W, RustcraftRegular,
};
pub use crate::engine::fonts::{BitmapFont, FontId, default_font, get_font, list_fonts};

/// Retrieves the raw glyph bitmask for character `c` from the default font.
#[inline]
pub fn glyph(c: char) -> [u8; 5] {
    RustcraftRegular::glyph(c)
}

/// Retrieves the raw glyph bitmask for character `c` from the RustcraftLight font.
#[inline]
pub fn glyph_light(c: char) -> [u8; 5] {
    crate::engine::fonts::rustcraft_light::glyph(c)
}
