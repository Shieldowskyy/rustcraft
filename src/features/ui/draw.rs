//! Low-level 2D framebuffer primitives (ARGB packing, SWAR alpha-blending, shadowed text, cursor).
//! Zero platform `#[cfg]` and zero per-pixel heap allocations.

use crate::features::rendering::font;

#[inline(always)]
pub fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let ir = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let ig = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let ib = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (0xFF << 24) | (ir << 16) | (ig << 8) | ib
}

#[inline(always)]
pub fn unpack_color(c: u32) -> (f32, f32, f32) {
    const INV_255: f32 = 1.0 / 255.0;
    let r = ((c >> 16) & 0xFF) as f32 * INV_255;
    let g = ((c >> 8) & 0xFF) as f32 * INV_255;
    let b = (c & 0xFF) as f32 * INV_255;
    (r, g, b)
}

#[inline]
pub fn fill_rect(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
) {
    let min_x = x.max(0);
    let min_y = y.max(0);
    let max_x = (x + w).min(buf_w as i32);
    let max_y = (y + h).min(buf_h as i32);
    if min_x >= max_x || min_y >= max_y {
        return;
    }
    let fill_c = if color <= 0x00FFFFFF {
        (0xFF << 24) | color
    } else {
        color
    };
    let span = (max_x - min_x) as usize;
    let mut row_start = min_y as usize * buf_w + min_x as usize;
    for _ in min_y..max_y {
        buffer[row_start..row_start + span].fill(fill_c);
        row_start += buf_w;
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn fill_rect_alpha(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
    alpha: f32,
) {
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

    let a_int = (alpha * 256.0 + 0.5) as u32;
    let a_byte = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
    let inv_a = 256 - a_int;

    // Pre-multiplied source channels using SWAR (Red & Blue packed together, Green separate)
    let src_rb = ((color & 0x00FF00FF) * a_int) & 0xFF00FF00;
    let src_g = ((color & 0x0000FF00) * a_int) & 0x00FF0000;
    let empty_bg_pixel = (a_byte << 24) | ((src_rb | src_g) >> 8);

    let span = (max_x - min_x) as usize;
    let mut row_start = min_y as usize * buf_w + min_x as usize;

    for _ in min_y..max_y {
        for pixel in &mut buffer[row_start..row_start + span] {
            let bg = *pixel;
            if bg == 0 {
                // Fast path for transparent GXM HUD background (zero per-pixel math)
                *pixel = empty_bg_pixel;
                continue;
            }
            let bg_a = bg >> 24;
            let dst_rb = ((bg & 0x00FF00FF) * inv_a) & 0xFF00FF00;
            let dst_g = ((bg & 0x0000FF00) * inv_a) & 0x00FF0000;
            let rgb = (src_rb.wrapping_add(dst_rb) | src_g.wrapping_add(dst_g)) >> 8;
            let out_a = (bg_a + a_byte).min(255);
            *pixel = (out_a << 24) | rgb;
        }
        row_start += buf_w;
    }
}

#[inline(always)]
pub fn set_px(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, color: u32) {
    if (x as u32) < (buf_w as u32) && (y as u32) < (buf_h as u32) {
        let idx = (y as usize) * buf_w + (x as usize);
        buffer[idx] = if color <= 0x00FFFFFF {
            (0xFF << 24) | color
        } else {
            color
        };
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn draw_text_impl(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    x: i32,
    y: i32,
    scale: i32,
    spacing: i32,
    fg_color: u32,
    shadow_color: u32,
    light: bool,
) {
    let fg = if fg_color <= 0x00FFFFFF {
        (0xFF << 24) | fg_color
    } else {
        fg_color
    };
    let w_u = buf_w as u32;
    let h_u = buf_h as u32;

    let mut plot = |px: i32, py: i32| {
        let sx = (px + 1) as u32;
        let sy = (py + 1) as u32;
        if sx < w_u && sy < h_u {
            let s_idx = (sy as usize) * buf_w + (sx as usize);
            if buffer[s_idx] != fg {
                buffer[s_idx] = shadow_color;
            }
        }
        let ux = px as u32;
        let uy = py as u32;
        if ux < w_u && uy < h_u {
            buffer[(uy as usize) * buf_w + (ux as usize)] = fg;
        }
    };

    if light {
        font::draw_text_light(text, x, y, scale, spacing, &mut plot);
    } else {
        font::draw_text(text, x, y, scale, spacing, &mut plot);
    }
}

pub fn draw_text_shadowed(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    x: i32,
    y: i32,
    scale: i32,
    spacing: i32,
) {
    draw_text_impl(buffer, buf_w, buf_h, text, x, y, scale, spacing, 0xFFFFFFFF, 0xFF141414, false);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_colored_shadowed(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    x: i32,
    y: i32,
    scale: i32,
    spacing: i32,
    fg_color: u32,
) {
    draw_text_impl(buffer, buf_w, buf_h, text, x, y, scale, spacing, fg_color, 0xFF141414, false);
}

pub fn draw_text_light_shadowed(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    x: i32,
    y: i32,
    scale: i32,
    spacing: i32,
) {
    draw_text_impl(buffer, buf_w, buf_h, text, x, y, scale, spacing, 0xFFF5F5F5, 0xFF0F0F0F, true);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_light_colored_shadowed(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    x: i32,
    y: i32,
    scale: i32,
    spacing: i32,
    fg_color: u32,
) {
    draw_text_impl(buffer, buf_w, buf_h, text, x, y, scale, spacing, fg_color, 0xFF0F0F0F, true);
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
