//! # RustCraft Light Font (3x5 High-Legibility Bitmap)
//!
//! Slender, open, high-contrast 3x5 bitmap font designed specifically for small diagnostic
//! text, F3 performance telemetry, memory statistics, and compact mobile/Vita screens.

use super::BitmapFont;

/// Width of each glyph in pixels.
pub const GLYPH_W: usize = 3;

/// Height of each glyph in pixels.
pub const GLYPH_H: usize = 5;

/// High-legibility 3x5 pixel font optimized for profiler telemetry and small screen stats.
#[derive(Debug, Clone, Copy, Default)]
pub struct RustcraftLight;

impl RustcraftLight {
    /// Returns the 5-row bitmask representation for character `c`.
    ///
    /// Each row is a `u8` where the 3 lower bits (bits 2, 1, 0) represent pixels left-to-right.
    #[inline]
    pub fn glyph(c: char) -> [u8; 5] {
        let upper = c.to_uppercase().next().unwrap_or(c);
        match upper {
            '0' => [0b010, 0b101, 0b101, 0b101, 0b010],
            '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
            '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
            '3' => [0b110, 0b001, 0b011, 0b001, 0b110],
            '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
            '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
            '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
            '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
            '8' => [0b010, 0b101, 0b010, 0b101, 0b010],
            '9' => [0b010, 0b101, 0b011, 0b001, 0b110],
            'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
            'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
            'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
            'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
            'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
            'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
            'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
            'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
            'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
            'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
            'K' => [0b101, 0b110, 0b100, 0b110, 0b101],
            'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
            'M' => [0b101, 0b111, 0b101, 0b101, 0b101],
            'N' => [0b101, 0b110, 0b101, 0b101, 0b101],
            'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
            'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
            'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
            'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
            'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
            'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
            'U' => [0b101, 0b101, 0b101, 0b101, 0b010],
            'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
            'W' => [0b101, 0b101, 0b101, 0b111, 0b101],
            'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
            'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
            'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
            '[' => [0b110, 0b100, 0b100, 0b100, 0b110],
            ']' => [0b011, 0b001, 0b001, 0b001, 0b011],
            '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
            '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
            '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
            '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
            '?' => [0b110, 0b001, 0b010, 0b000, 0b010],
            ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
            '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
            '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
            '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
            '(' => [0b010, 0b100, 0b100, 0b100, 0b010],
            ')' => [0b010, 0b001, 0b001, 0b001, 0b010],
            '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
            '|' => [0b010, 0b010, 0b010, 0b010, 0b010],
            ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
            '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
            '%' => [0b100, 0b001, 0b010, 0b100, 0b001],
            '*' => [0b101, 0b010, 0b111, 0b010, 0b101],
            '~' => [0b000, 0b101, 0b010, 0b000, 0b000],
            '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
            '"' => [0b101, 0b101, 0b000, 0b000, 0b000],
            '#' => [0b101, 0b111, 0b101, 0b111, 0b101],
            '$' => [0b010, 0b111, 0b010, 0b111, 0b010],
            '&' => [0b010, 0b101, 0b010, 0b101, 0b011],
            ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
            'Ą' => [0b010, 0b101, 0b111, 0b101, 0b101],
            'Ć' => [0b011, 0b100, 0b100, 0b100, 0b011],
            'Ę' => [0b111, 0b100, 0b110, 0b100, 0b111],
            'Ł' => [0b100, 0b110, 0b100, 0b100, 0b111],
            'Ń' => [0b101, 0b110, 0b101, 0b101, 0b101],
            'Ó' => [0b010, 0b101, 0b101, 0b101, 0b010],
            'Ś' => [0b011, 0b100, 0b010, 0b001, 0b110],
            'Ź' => [0b111, 0b001, 0b010, 0b100, 0b111],
            'Ż' => [0b111, 0b001, 0b010, 0b100, 0b111],
            _ => [0b010, 0b101, 0b101, 0b101, 0b010],
        }
    }

    /// Renders text using pixel callback `plot(x, y)`.
    pub fn draw_text(text: &str, origin_x: i32, origin_y: i32, scale: i32, spacing: i32, mut plot: impl FnMut(i32, i32)) {
        let mut cursor_x = origin_x;
        for ch in text.chars() {
            let rows = Self::glyph(ch);
            for (row_idx, row) in rows.iter().enumerate() {
                for col_idx in 0..GLYPH_W {
                    let bit = (row >> (GLYPH_W - 1 - col_idx)) & 1;
                    if bit == 1 {
                        let px = cursor_x + col_idx as i32 * scale;
                        let py = origin_y + row_idx as i32 * scale;
                        for sy in 0..scale {
                            for sx in 0..scale {
                                plot(px + sx, py + sy);
                            }
                        }
                    }
                }
            }
            cursor_x += (GLYPH_W as i32) * scale + spacing;
        }
    }

    /// Measures the total pixel width occupied by `text`.
    pub fn measure_text(text: &str, scale: i32, spacing: i32) -> i32 {
        let n = text.chars().count() as i32;
        if n == 0 {
            return 0;
        }
        n * (GLYPH_W as i32 * scale) + (n - 1) * spacing
    }
}

/// Renders text using the RustcraftLight font.
#[inline]
pub fn draw_text(text: &str, origin_x: i32, origin_y: i32, scale: i32, spacing: i32, plot: impl FnMut(i32, i32)) {
    RustcraftLight::draw_text(text, origin_x, origin_y, scale, spacing, plot);
}

/// Measures text width in pixels using the RustcraftLight font.
#[inline]
pub fn measure_text(text: &str, scale: i32, spacing: i32) -> i32 {
    RustcraftLight::measure_text(text, scale, spacing)
}

/// Returns the glyph bitmask for `c` in RustcraftLight.
#[inline]
pub fn glyph(c: char) -> [u8; 5] {
    RustcraftLight::glyph(c)
}

impl BitmapFont for RustcraftLight {
    fn name(&self) -> &'static str {
        "RustcraftLight"
    }

    fn glyph_width(&self) -> usize {
        GLYPH_W
    }

    fn glyph_height(&self) -> usize {
        GLYPH_H
    }

    fn render_text(
        &self,
        text: &str,
        origin_x: i32,
        origin_y: i32,
        scale: i32,
        spacing: i32,
        plot: &mut dyn FnMut(i32, i32),
    ) {
        Self::draw_text(text, origin_x, origin_y, scale, spacing, plot);
    }

    fn measure(&self, text: &str, scale: i32, spacing: i32) -> i32 {
        Self::measure_text(text, scale, spacing)
    }
}
