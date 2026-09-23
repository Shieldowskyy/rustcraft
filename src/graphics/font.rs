// A minimal built-in bitmap font (3 pixels wide, 5 pixels tall per glyph)
// so the UI can draw text without any external font files or font-loading
// dependencies. Supports uppercase letters, digits, and a few symbols.

pub const GLYPH_W: usize = 3;
pub const GLYPH_H: usize = 5;

fn glyph(c: char) -> [u8; 5] {
    match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
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
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b111, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '[' => [0b110, 0b100, 0b100, 0b100, 0b110],
        ']' => [0b011, 0b001, 0b001, 0b001, 0b011],
        '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '?' => [0b111, 0b001, 0b010, 0b000, 0b010],
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
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        '*' => [0b101, 0b010, 0b111, 0b010, 0b101],
        ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        'Ą' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'Ć' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'Ę' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'Ł' => [0b100, 0b110, 0b100, 0b100, 0b111],
        'Ń' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'Ó' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'Ś' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'Ź' => [0b111, 0b001, 0b010, 0b100, 0b111],
        'Ż' => [0b111, 0b001, 0b010, 0b100, 0b111],
        _ => [0b111, 0b101, 0b101, 0b101, 0b111],
    }
}

/// Calls `plot(x, y)` for every "on" pixel of `text` rendered at the given
/// top-left origin, scaled by `scale` (integer pixel multiplier), with
/// `spacing` pixels (pre-scale) between glyphs.
pub fn draw_text(text: &str, origin_x: i32, origin_y: i32, scale: i32, spacing: i32, mut plot: impl FnMut(i32, i32)) {
    let mut cursor_x = origin_x;
    for ch in text.chars() {
        let upper = ch.to_uppercase().next().unwrap_or(ch);
        let rows = glyph(upper);
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

/// Measures the pixel width `draw_text` would occupy for `text`.
pub fn measure_text(text: &str, scale: i32, spacing: i32) -> i32 {
    let n = text.chars().count() as i32;
    if n == 0 {
        return 0;
    }
    n * (GLYPH_W as i32 * scale) + (n - 1) * spacing
}
