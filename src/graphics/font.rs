//! # Built-in Engine Typography & Font Module
//!
//! Provides text measuring, font constants, and pixel plotting functions backed
//! by the modular [`crate::engine::fonts::RustcraftRegular`] engine font.

pub use crate::engine::fonts::rustcraft_regular::{
    draw_text, measure_text, GLYPH_H, GLYPH_W, RustcraftRegular,
};
pub use crate::engine::fonts::{BitmapFont, FontId, default_font, get_font, list_fonts};

/// Retrieves the raw glyph bitmask for character `c` from the default font.
#[inline]
pub fn glyph(c: char) -> [u8; 5] {
    RustcraftRegular::glyph(c)
}
