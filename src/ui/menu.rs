//! Main menu screen, voxel logo rendering, and button hitboxes.

use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Calculates layout rectangles for the 3 main menu buttons `[(x, y, width, height); 3]`.
pub fn main_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 3] {
    let btn_w = 380;
    let btn_h = 44;
    let spacing = 16;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 + 10;
    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
    ]
}

/// Draws the procedural pixel-art voxel logo on the main menu.
pub fn draw_rustcraft_logo(buffer: &mut [u32], width: usize, height: usize, cx: i32, y: i32) {
    let gold = pack_color(0.95, 0.72, 0.20);
    let light = pack_color(1.0, 0.90, 0.48);
    let dark = pack_color(0.38, 0.22, 0.08);
    let blocks = [
        (-48, 0, 8, 8, gold), (-40, 0, 8, 8, light), (-32, 0, 8, 8, gold),
        (-24, 0, 8, 8, dark), (-16, 0, 8, 8, gold), (-8, 0, 8, 8, light),
        (-40, 8, 8, 8, gold), (-32, 16, 8, 8, light), (-24, 24, 8, 8, gold),
        (-16, 32, 8, 8, dark), (-8, 40, 8, 8, gold), (0, 32, 8, 8, light),
        (8, 24, 8, 8, gold), (16, 16, 8, 8, dark), (24, 8, 8, 8, gold),
        (32, 0, 8, 8, light), (40, -8, 8, 8, gold),
    ];
    for (x, offset_y, w, h, color) in blocks {
        fill_rect(buffer, width, height, cx + x, y + offset_y, w, h, color);
    }
}

/// Renders the title screen, animated logo, menu choices, and platform controls guide.
pub fn draw_main_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    hovered_idx: Option<usize>,
) {
    let cx = width as i32 / 2;
    // Clear screen to prevent ghost frames from other menus
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.06, 0.07, 0.09));
    fill_rect_alpha(buffer, width, height, cx - 240, 30, 480, 465, pack_color(0.02, 0.03, 0.05), 0.75);
    draw_rustcraft_logo(buffer, width, height, cx, 65);

    let title = "RUSTCRAFT";
    let title_w = font::measure_text(title, 4, 3);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, 135, 4, 3, pack_color(0.95, 0.85, 0.42));

    let subtitle = "VOXEL ENGINE";
    let subtitle_w = font::measure_text(subtitle, 2, 2);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - subtitle_w / 2, 185, 2, 2, pack_color(0.7, 0.75, 0.85));

    let buttons = [
        crate::core::i18n::t("menu.play"),
        crate::core::i18n::t("menu.settings"),
        crate::core::i18n::t("menu.quit"),
    ];
    let rects = main_menu_button_rects(width, height);
    for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);
        let body_color = if is_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.22, 0.26) };
        let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
        let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - font::GLYPH_H as i32 * scale) / 2;
        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let footer = crate::core::i18n::t("menu.controls");
    let fw = font::measure_text(footer, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, footer, cx - fw / 2, 460, 1, 1, pack_color(0.6, 0.65, 0.75));
}

