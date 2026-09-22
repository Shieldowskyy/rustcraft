use crate::graphics::font;

#[inline(always)]
pub fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let ir = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let ig = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let ib = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (ir << 16) | (ig << 8) | ib
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
    for yy in min_y..max_y {
        let start = yy as usize * buf_w + min_x as usize;
        let end = yy as usize * buf_w + max_x as usize;
        buffer[start..end].fill(color);
    }
}

pub fn fill_rect_alpha(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, w: i32, h: i32, color: u32, alpha: f32) {
    let (cr, cg, cb) = unpack_color(color);
    let min_x = x.max(0);
    let min_y = y.max(0);
    let max_x = (x + w).min(buf_w as i32);
    let max_y = (y + h).min(buf_h as i32);
    for yy in min_y..max_y {
        for xx in min_x..max_x {
            let idx = yy as usize * buf_w + xx as usize;
            let (br, bg, bb) = unpack_color(buffer[idx]);
            let r = cr * alpha + br * (1.0 - alpha);
            let g = cg * alpha + bg * (1.0 - alpha);
            let b = cb * alpha + bb * (1.0 - alpha);
            buffer[idx] = pack_color(r, g, b);
        }
    }
}

pub fn set_px(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= buf_w as i32 || y >= buf_h as i32 {
        return;
    }
    let idx = y as usize * buf_w + x as usize;
    buffer[idx] = color;
}

pub fn draw_text_shadowed(buffer: &mut [u32], buf_w: usize, buf_h: usize, text: &str, x: i32, y: i32, scale: i32, spacing: i32) {
    let shadow = pack_color(0.0, 0.0, 0.0);
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
    let shadow = pack_color(0.0, 0.0, 0.0);
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
