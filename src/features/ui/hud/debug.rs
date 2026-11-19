//! F3 diagnostic debug panels, profiler pie chart, mouse capture prompt, and multiplayer toast.
//! Zero platform `#[cfg]`.

use crate::features::rendering::font;

use super::super::draw::{pack_color, set_px};
use super::super::widgets::{Rect, UiCanvas};

/// Draws a rasterized circular debug pie chart with colored slices (zero heap allocation, no per-pixel atan2).
pub fn draw_pie_chart(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    radius: i32,
    slices: &[crate::core::profiler::ProfilerSlice],
) {
    if slices.is_empty() || radius <= 0 {
        return;
    }

    let r_sq = radius * radius;
    let r_inner_sq = (radius - 1) * (radius - 1);
    let border_color = pack_color(0.9, 0.9, 0.95);

    let slice_count = slices.len().min(31);
    let mut rays = [(0.0f32, -1.0f32); 32];
    let mut current_angle = -std::f32::consts::FRAC_PI_2;

    for (idx, s) in slices.iter().take(slice_count).enumerate() {
        current_angle += (s.percentage / 100.0) * std::f32::consts::TAU;
        rays[idx + 1] = (current_angle.cos(), current_angle.sin());
    }

    let min_x = (cx - radius).max(0);
    let max_x = (cx + radius).min(width as i32 - 1);
    let min_y = (cy - radius).max(0);
    let max_y = (cy + radius).min(height as i32 - 1);

    for y in min_y..=max_y {
        let dy = (y - cy) as f32;
        let dy_sq = (y - cy) * (y - cy);
        for x in min_x..=max_x {
            let dx = (x - cx) as f32;
            let dist_sq = (x - cx) * (x - cx) + dy_sq;
            if dist_sq <= r_sq {
                if dist_sq >= r_inner_sq {
                    set_px(buffer, width, height, x, y, border_color);
                } else {
                    let p_half = dx < 0.0;
                    let mut color = slices[slice_count - 1].color;
                    for i in 0..slice_count {
                        let (rx, ry) = rays[i + 1];
                        let r_half = rx < 0.0 || (rx == 0.0 && ry > 0.0);
                        let before_end = if p_half != r_half {
                            !p_half
                        } else {
                            rx * dy - ry * dx <= 0.0
                        };
                        if before_end {
                            color = slices[i].color;
                            break;
                        }
                    }
                    set_px(buffer, width, height, x, y, color);
                }
            }
        }
    }
}

/// Draws the comprehensive F3 debug overlay including left stats, right memory/renderer telemetry, and bottom-right pie chart.
pub fn draw_advanced_debug_overlay(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    left_lines: &[String],
    right_lines: &[String],
    profiler: &crate::core::profiler::FrameProfiler,
) {
    let mut ui = UiCanvas::new(buffer, width, height);

    ui.debug_text_panel(4, 4, left_lines, 5, 2, 0.60, false);
    ui.debug_text_panel(4, 4, right_lines, 5, 2, 0.60, true);

    let slices = profiler.pie_slices();
    let pie_r = 34;
    let pie_cx = width as i32 - pie_r - 10;
    let pie_cy = height as i32 - pie_r - 12;

    let pie_panel_w = 310;
    let pie_panel_h = (slices.len() as i32 * 12) + 26;
    let pie_panel_x = width as i32 - pie_panel_w - 4;
    let pie_panel_y = height as i32 - pie_panel_h - 4;

    ui.fill_alpha(
        Rect::new(pie_panel_x, pie_panel_y, pie_panel_w, pie_panel_h),
        pack_color(0.0, 0.0, 0.0),
        0.70,
    );

    let prof_title = "DEBUG PROFILER (root.gameRenderer.level)";
    ui.text_light_colored(
        prof_title,
        pie_panel_x + 6,
        pie_panel_y + 4,
        1,
        1,
        pack_color(0.95, 0.85, 0.35),
    );

    draw_pie_chart(ui.buffer, ui.width, ui.height, pie_cx, pie_cy, pie_r, &slices);

    let mut leg_y = pie_panel_y + 18;
    for s in &slices {
        ui.fill(Rect::new(pie_panel_x + 6, leg_y + 2, 6, 6), s.color);
        let leg_txt = format!("{:<11} {:>4.1}% {:>4.2}ms", s.name, s.percentage, s.time_ms);
        ui.text_light(&leg_txt, pie_panel_x + 16, leg_y, 1, 1);
        leg_y += 12;
    }
}

pub fn draw_debug_panel(buffer: &mut [u32], width: usize, height: usize, lines: &[String]) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.debug_text_panel(6, 6, lines, 6, 4, 0.5, false);
}

pub fn draw_capture_prompt(buffer: &mut [u32], width: usize, height: usize, native_capture: bool) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.0, 0.0, 0.0), 0.35);

    let line1 = crate::core::i18n::t("capture.prompt");
    let line2 = if native_capture {
        crate::core::i18n::t("capture.mouse_locked")
    } else {
        crate::core::i18n::t("capture.esc_pause")
    };

    let cx = ui.cx();
    let cy = ui.cy();

    ui.text_centered_x(line1, cx, cy - 20, 4, 2, pack_color(1.0, 1.0, 1.0));
    ui.text_centered_x(line2, cx, cy + 20, 2, 1, pack_color(1.0, 1.0, 1.0));
}

pub fn draw_multiplayer_toast(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    message: &str,
    is_error: bool,
) {
    let max_text_width = width.saturating_sub(40) as i32;
    let max_chars = (max_text_width / 6).max(1) as usize;
    let text: String = message.chars().take(max_chars).collect();
    let text_width = font::measure_text(&text, 1, 1);
    let box_width = (text_width + 24).min(max_text_width);
    let box_height = 28;
    let x = (width as i32 - box_width) / 2;
    let y = height as i32 - 72;
    let accent = if is_error {
        pack_color(0.92, 0.30, 0.28)
    } else {
        pack_color(0.25, 0.78, 0.50)
    };

    let mut ui = UiCanvas::new(buffer, width, height);
    ui.fill_alpha(
        Rect::new(x, y, box_width, box_height),
        pack_color(0.04, 0.05, 0.07),
        0.92,
    );
    ui.fill(Rect::new(x, y, box_width, 2), accent);
    ui.fill(Rect::new(x, y, 3, box_height), accent);
    ui.text(&text, x + 12, y + 9, 1, 1, pack_color(0.96, 0.98, 1.0));
}
