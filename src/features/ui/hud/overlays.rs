//! HUD crosshair, mining progress bar, environmental screen overlays, damage/health vignettes, and biome color post-processing.
//! Uses `platform::{capabilities, threading}` for zero platform `#[cfg]`.

use crate::platform::{capabilities, threading};

use super::super::draw::{fill_rect_alpha, pack_color, set_px, unpack_color};
use super::super::widgets::UiCanvas;

pub fn draw_crosshair(buffer: &mut [u32], width: usize, height: usize) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let size = 6i32;
    let color = pack_color(1.0, 1.0, 1.0);
    for i in -size..=size {
        set_px(buffer, width, height, cx + i, cy, color);
        set_px(buffer, width, height, cx, cy + i, color);
    }
}

pub fn draw_break_progress(buffer: &mut [u32], width: usize, height: usize, progress: f32) {
    if progress <= 0.0 {
        return;
    }
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let bar_w = 40i32;
    let bar_h = 3i32;
    let bar_x = cx - bar_w / 2;
    let bar_y = cy + 14;

    fill_rect_alpha(
        buffer,
        width,
        height,
        bar_x - 1,
        bar_y - 1,
        bar_w + 2,
        bar_h + 2,
        pack_color(0.0, 0.0, 0.0),
        0.25,
    );
    let t = progress.clamp(0.0, 1.0);
    let fill_w = (bar_w as f32 * t) as i32;
    if fill_w > 0 {
        fill_rect_alpha(
            buffer,
            width,
            height,
            bar_x,
            bar_y,
            fill_w,
            bar_h,
            pack_color(0.85, 0.85, 0.85),
            0.5,
        );
    }
}

pub fn draw_underwater_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    fill_rect_alpha(
        buffer,
        width,
        height,
        0,
        0,
        width as i32,
        height as i32,
        pack_color(0.04, 0.28, 0.48),
        0.34,
    );
    let wave = ((time * 1.8).sin() * 3.0) as i32;
    for y in (18 + wave)..height as i32 {
        if y % 31 == 0 {
            fill_rect_alpha(
                buffer,
                width,
                height,
                0,
                y,
                width as i32,
                1,
                pack_color(0.25, 0.65, 0.80),
                0.08,
            );
        }
    }
}

pub fn draw_lava_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    fill_rect_alpha(
        buffer,
        width,
        height,
        0,
        0,
        width as i32,
        height as i32,
        pack_color(0.92, 0.28, 0.02),
        0.85,
    );
    let wave = ((time * 2.5).sin() * 4.0) as i32;
    for y in (10 + wave)..height as i32 {
        if y % 17 == 0 {
            fill_rect_alpha(
                buffer,
                width,
                height,
                0,
                y,
                width as i32,
                2,
                pack_color(1.0, 0.55, 0.05),
                0.18,
            );
        }
    }
}

pub fn draw_burning_fire_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    let fire_height = 68i32;
    let base_y = height as i32 - fire_height;
    for x in 0..width as i32 {
        let fx = x as f32 * 0.12;
        let wave = ((fx + time * 14.0).sin() * 0.5 + (fx * 0.5 - time * 9.0).cos() * 0.5) * 18.0;
        let flame_top = (base_y + 16 - wave as i32).clamp(0, height as i32);
        for y in flame_top..height as i32 {
            let dy = y - flame_top;
            let total_h = (height as i32 - flame_top).max(1);
            let prog = dy as f32 / total_h as f32;
            let (r, g, b, alpha) = if prog < 0.30 {
                (1.0f32, 0.90f32, 0.20f32, 0.50f32)
            } else if prog < 0.65 {
                (0.98f32, 0.48f32, 0.08f32, 0.60f32)
            } else {
                (0.85f32, 0.15f32, 0.02f32, 0.68f32)
            };
            let idx = y as usize * width + x as usize;
            let (br, bg, bb) = unpack_color(buffer[idx]);
            buffer[idx] = pack_color(
                r * alpha + br * (1.0 - alpha),
                g * alpha + bg * (1.0 - alpha),
                b * alpha + bb * (1.0 - alpha),
            );
        }
    }
}

pub fn draw_damage_flash(buffer: &mut [u32], width: usize, height: usize, intensity: f32) {
    if intensity <= 0.001 {
        return;
    }
    let alpha = (intensity * 0.35).clamp(0.0, 0.55);
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.95, 0.05, 0.05), alpha);
}

/// Applies an atmospheric biome color filter postprocessing pass to the screen color buffer.
pub fn apply_biome_color_filter(buffer: &mut [u32], tint: [f32; 3]) {
    if (tint[0] - 1.0).abs() < 0.003 && (tint[1] - 1.0).abs() < 0.003 && (tint[2] - 1.0).abs() < 0.003 {
        return;
    }

    let mr = (tint[0] * 256.0).round().max(0.0) as u32;
    let mg = (tint[1] * 256.0).round().max(0.0) as u32;
    let mb = (tint[2] * 256.0).round().max(0.0) as u32;
    let skip_transparent = capabilities().skip_transparent_hud_tint;

    threading::parallel_for_chunks_mut(buffer, 4096, |_idx, chunk| {
        for pixel in chunk.iter_mut() {
            let p = *pixel;
            // DLACZEGO: On PS Vita (GXM HUD overlay) and WASM, transparent HUD pixels (alpha == 0)
            // are skipped, saving ~510k pixel operations per frame.
            if skip_transparent && (p >> 24) == 0 {
                continue;
            }
            let r = (((p >> 16) & 0xFF) * mr >> 8).min(255);
            let g = (((p >> 8) & 0xFF) * mg >> 8).min(255);
            let b = ((p & 0xFF) * mb >> 8).min(255);
            *pixel = (p & 0xFF000000) | (r << 16) | (g << 8) | b;
        }
    });
}

pub fn draw_low_health_vignette(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    let pulse = (time * 6.5).sin() * 0.5 + 0.5;
    let base_alpha = 0.22 + 0.32 * pulse;
    let color = pack_color(0.85, 0.02, 0.02);

    let w = width as i32;
    let h = height as i32;
    let border_depth = 55;

    for layer in 0..border_depth {
        let t = 1.0 - (layer as f32 / border_depth as f32);
        let alpha = base_alpha * t * t * 0.75;
        let l_w = w - layer * 2;
        let l_h = h - layer * 2;
        if l_w <= 0 || l_h <= 0 {
            break;
        }

        fill_rect_alpha(buffer, width, height, layer, layer, l_w, 1, color, alpha);
        fill_rect_alpha(buffer, width, height, layer, h - 1 - layer, l_w, 1, color, alpha);
        fill_rect_alpha(buffer, width, height, layer, layer + 1, 1, l_h - 2, color, alpha);
        fill_rect_alpha(buffer, width, height, w - 1 - layer, layer + 1, 1, l_h - 2, color, alpha);
    }
}
