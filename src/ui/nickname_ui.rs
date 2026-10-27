//! Change nickname modal dialogue UI layout and rendering.

use crate::core::i18n::t;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Computes bounding boxes `(x, y, width, height)` for the 4 elements in the nickname dialog:
/// - 0: Nickname input field box
/// - 1: [ DONE ] button
/// - 2: [ RANDOM ] button
/// - 3: [ CANCEL ] button
pub fn nickname_dialog_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let modal_h = 220;
    let my = cy - modal_h / 2;

    let input_w = 330;
    let input_h = 32;
    let input_x = cx - input_w / 2;
    let input_y = my + 72;

    let btn_w = 102;
    let btn_h = 30;
    let spacing = 12;
    let total_btns_w = btn_w * 3 + spacing * 2;
    let start_btn_x = cx - total_btns_w / 2;
    let btn_y = my + 144;

    [
        (input_x, input_y, input_w, input_h),
        (start_btn_x, btn_y, btn_w, btn_h),
        (start_btn_x + btn_w + spacing, btn_y, btn_w, btn_h),
        (start_btn_x + (btn_w + spacing) * 2, btn_y, btn_w, btn_h),
    ]
}

/// Renders the Change Nickname modal dialogue with text input, selection, buttons, and gamepad hints.
pub fn draw_nickname_dialog(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    current_nick: &str,
    is_selected: bool,
    hovered_elem: Option<usize>,
    cursor_visible: bool,
    is_gamepad_active: bool,
) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    // 1. Full-screen dimmed backdrop
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.70);

    // 2. Modal panel background and double beveled borders
    let modal_w = 460.min(width as i32 - 40);
    let modal_h = 220;
    let mx = cx - modal_w / 2;
    let my = cy - modal_h / 2;

    fill_rect(buffer, width, height, mx, my, modal_w, modal_h, pack_color(0.09, 0.11, 0.15));
    // Outer border
    fill_rect(buffer, width, height, mx, my, modal_w, 2, pack_color(0.40, 0.50, 0.70));
    fill_rect(buffer, width, height, mx, my, 2, modal_h, pack_color(0.40, 0.50, 0.70));
    fill_rect(buffer, width, height, mx, my + modal_h - 2, modal_w, 2, pack_color(0.04, 0.05, 0.08));
    fill_rect(buffer, width, height, mx + modal_w - 2, my, 2, modal_h, pack_color(0.04, 0.05, 0.08));

    // Inner highlight line
    fill_rect(buffer, width, height, mx + 3, my + 3, modal_w - 6, 1, pack_color(0.18, 0.22, 0.32));
    fill_rect(buffer, width, height, mx + 3, my + 3, 1, modal_h - 6, pack_color(0.18, 0.22, 0.32));

    // 3. Title & prompt
    let title = t("nickname.title");
    let tw = font::measure_text(title, 2, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - tw / 2, my + 18, 2, 2, pack_color(0.95, 0.85, 0.40));

    let prompt = t("nickname.prompt");
    let pw = font::measure_text(prompt, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, prompt, cx - pw / 2, my + 52, 1, 1, pack_color(0.80, 0.85, 0.95));

    let rects = nickname_dialog_rects(width, height);
    let (ix, iy, iw, ih) = rects[0];

    // 4. Nickname Input Field
    let is_input_focused = hovered_elem == Some(0) || hovered_elem.is_none();
    let input_bg = pack_color(0.04, 0.05, 0.07);
    let border_color = if is_input_focused {
        pack_color(0.45, 0.70, 1.0)
    } else {
        pack_color(0.25, 0.30, 0.40)
    };

    fill_rect(buffer, width, height, ix, iy, iw, ih, input_bg);
    fill_rect(buffer, width, height, ix, iy, iw, 2, border_color);
    fill_rect(buffer, width, height, ix, iy, 2, ih, border_color);
    fill_rect(buffer, width, height, ix, iy + ih - 2, iw, 2, border_color);
    fill_rect(buffer, width, height, ix + iw - 2, iy, 2, ih, border_color);

    // Text and cursor inside input box
    let text_x = ix + 10;
    let text_y = iy + (ih - font::GLYPH_H as i32 * 2) / 2;
    let rendered_text_w = font::measure_text(current_nick, 2, 1);

    if is_selected && !current_nick.is_empty() {
        fill_rect_alpha(buffer, width, height, text_x - 2, text_y - 2, rendered_text_w + 4, font::GLYPH_H as i32 * 2 + 4, pack_color(0.2, 0.4, 0.8), 0.65);
    }

    if current_nick.is_empty() {
        let placeholder = "Player";
        draw_text_colored_shadowed(buffer, width, height, placeholder, text_x, text_y, 2, 1, pack_color(0.40, 0.45, 0.55));
    } else {
        draw_text_colored_shadowed(buffer, width, height, current_nick, text_x, text_y, 2, 1, pack_color(1.0, 1.0, 1.0));
    }

    if cursor_visible && is_input_focused {
        let cursor_x = text_x + rendered_text_w + 2;
        fill_rect(buffer, width, height, cursor_x, text_y, 3, font::GLYPH_H as i32 * 2, pack_color(1.0, 0.90, 0.35));
    }

    // Character counter "12/24"
    let counter_str = format!("{}/24", current_nick.len());
    let cw = font::measure_text(&counter_str, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, &counter_str, ix + iw - cw - 8, iy + (ih - font::GLYPH_H as i32) / 2, 1, 1, pack_color(0.50, 0.55, 0.65));

    // Help text below input
    let help = t("nickname.help");
    let hw = font::measure_text(help, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, help, cx - hw / 2, iy + ih + 8, 1, 1, pack_color(0.55, 0.60, 0.70));

    // 5. Action Buttons (Done, Random, Cancel)
    let button_labels = [
        t("nickname.done"),
        t("nickname.random"),
        t("nickname.cancel"),
    ];

    for (i, (&label, &(bx, by, bw, bh))) in button_labels.iter().zip(rects[1..].iter()).enumerate() {
        let btn_idx = i + 1;
        let is_hover = hovered_elem == Some(btn_idx);

        let body_color = if is_hover {
            pack_color(0.35, 0.45, 0.65)
        } else {
            pack_color(0.20, 0.22, 0.28)
        };
        let top_left = if is_hover {
            pack_color(0.75, 0.85, 1.0)
        } else {
            pack_color(0.48, 0.52, 0.60)
        };
        let bottom_right = if is_hover {
            pack_color(0.14, 0.18, 0.28)
        } else {
            pack_color(0.08, 0.09, 0.12)
        };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 1;
        let lw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - lw) / 2;
        let ty = by + (bh - font::GLYPH_H as i32 * scale) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.35));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    // 6. Gamepad navigation hint
    let footer_hint = if is_gamepad_active {
        t("nickname.controls_hint")
    } else {
        "[ENTER] Save   [R] Randomize   [ESC] Cancel"
    };
    let fhw = font::measure_text(footer_hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, footer_hint, cx - fhw / 2, my + modal_h - 22, 1, 1, pack_color(0.60, 0.65, 0.75));
}
