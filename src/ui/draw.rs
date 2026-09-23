use crate::graphics::font;

#[inline(always)]
pub fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let ir = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let ig = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let ib = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (0xFF << 24) | (ir << 16) | (ig << 8) | ib
}

#[inline(always)]
pub fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    (r, g, b)
}

pub fn fill_rect(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    let min_x = x.max(0);
    let min_y = y.max(0);
    let max_x = (x + w).min(buf_w as i32);
    let max_y = (y + h).min(buf_h as i32);
    if min_x >= max_x || min_y >= max_y {
        return;
    }
    let fill_c = if color <= 0x00FFFFFF { (0xFF << 24) | color } else { color };
    for yy in min_y..max_y {
        let start = yy as usize * buf_w + min_x as usize;
        let end = yy as usize * buf_w + max_x as usize;
        buffer[start..end].fill(fill_c);
    }
}

#[inline]
pub fn fill_rect_alpha(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, w: i32, h: i32, color: u32, alpha: f32) {
    if alpha <= 0.005 {
        return;
    }
    if alpha >= 0.995 {
        fill_rect(buffer, buf_w, buf_h, x, y, w, h, color);
        return;
    }

    let min_x = x.max(0);
    let min_y = y.max(0);
    let max_x = (x + w).min(buf_w as i32);
    let max_y = (y + h).min(buf_h as i32);
    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let a_int = (alpha * 256.0).round() as u32;
    let a_byte = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
    let inv_a = 256 - a_int;
    let cr = (color >> 16) & 0xFF;
    let cg = (color >> 8) & 0xFF;
    let cb = color & 0xFF;

    for yy in min_y..max_y {
        let row_start = yy as usize * buf_w;
        let start = row_start + min_x as usize;
        let end = row_start + max_x as usize;
        for pixel in &mut buffer[start..end] {
            let bg = *pixel;
            let bg_a = (bg >> 24) & 0xFF;
            let br = (bg >> 16) & 0xFF;
            let bg_g = (bg >> 8) & 0xFF;
            let bb = bg & 0xFF;

            let r = (cr * a_int + br * inv_a) >> 8;
            let g = (cg * a_int + bg_g * inv_a) >> 8;
            let b = (cb * a_int + bb * inv_a) >> 8;
            let out_a = (bg_a + a_byte).min(255);
            *pixel = (out_a << 24) | (r << 16) | (g << 8) | b;
        }
    }
}

pub fn set_px(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= buf_w as i32 || y >= buf_h as i32 {
        return;
    }
    let idx = y as usize * buf_w + x as usize;
    buffer[idx] = if color <= 0x00FFFFFF { (0xFF << 24) | color } else { color };
}

pub fn draw_text_shadowed(buffer: &mut [u32], buf_w: usize, buf_h: usize, text: &str, x: i32, y: i32, scale: i32, spacing: i32) {
    let shadow = pack_color(0.08, 0.08, 0.08);
    let fg = pack_color(1.0, 1.0, 1.0);
    let w = buf_w as i32;
    let h = buf_h as i32;

    font::draw_text(text, x + 1, y + 1, scale, spacing, |px, py| {
        if px >= 0 && py >= 0 && px < w && py < h {
            buffer[(py as usize) * buf_w + (px as usize)] = shadow;
        }
    });
    font::draw_text(text, x, y, scale, spacing, |px, py| {
        if px >= 0 && py >= 0 && px < w && py < h {
            buffer[(py as usize) * buf_w + (px as usize)] = fg;
        }
    });
}

pub fn draw_text_colored_shadowed(buffer: &mut [u32], buf_w: usize, buf_h: usize, text: &str, x: i32, y: i32, scale: i32, spacing: i32, fg_color: u32) {
    let shadow = pack_color(0.08, 0.08, 0.08);
    let w = buf_w as i32;
    let h = buf_h as i32;

    font::draw_text(text, x + 1, y + 1, scale, spacing, |px, py| {
        if px >= 0 && py >= 0 && px < w && py < h {
            buffer[(py as usize) * buf_w + (px as usize)] = shadow;
        }
    });
    font::draw_text(text, x, y, scale, spacing, |px, py| {
        if px >= 0 && py >= 0 && px < w && py < h {
            buffer[(py as usize) * buf_w + (px as usize)] = fg_color;
        }
    });
}

pub fn draw_cursor_pointer(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32) {
    let cursor_pixels: [(i32, i32, u32); 26] = [
        (0, 0, 0xFFFFFFFF),
        (0, 1, 0xFFFFFFFF), (1, 1, 0xFFFFFFFF),
        (0, 2, 0xFFFFFFFF), (1, 2, 0xFFFFFFFF), (2, 2, 0xFFFFFFFF),
        (0, 3, 0xFFFFFFFF), (1, 3, 0xFFFFFFFF), (2, 3, 0xFFFFFFFF), (3, 3, 0xFFFFFFFF),
        (0, 4, 0xFFFFFFFF), (1, 4, 0xFFFFFFFF), (2, 4, 0xFFFFFFFF), (3, 4, 0xFFFFFFFF), (4, 4, 0xFFFFFFFF),
        (0, 5, 0xFFFFFFFF), (1, 5, 0xFFFFFFFF), (2, 5, 0xFFFFFFFF), (3, 5, 0xFFFFFFFF),
        (0, 6, 0xFFFFFFFF), (1, 6, 0xFFFFFFFF), (2, 6, 0xFFFFFFFF),
        (0, 7, 0xFFFFFFFF), (1, 7, 0xFFFFFFFF),
        (0, 8, 0xFFFFFFFF),
        (4, 5, 0xFFFFFFFF),
    ];
    for (dx, dy, _) in cursor_pixels {
        set_px(buffer, buf_w, buf_h, x + dx - 1, y + dy, 0xFF000000);
        set_px(buffer, buf_w, buf_h, x + dx + 1, y + dy, 0xFF000000);
        set_px(buffer, buf_w, buf_h, x + dx, y + dy - 1, 0xFF000000);
        set_px(buffer, buf_w, buf_h, x + dx, y + dy + 1, 0xFF000000);
    }
    for (dx, dy, col) in cursor_pixels {
        set_px(buffer, buf_w, buf_h, x + dx, y + dy, col);
    }
}
