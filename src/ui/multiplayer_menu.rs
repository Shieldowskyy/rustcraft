//! Multiplayer join menu and server connection interface.

use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Calculates layout bounding boxes for interactive multiplayer menu elements:
/// - `0`: Nickname input box
/// - `1`: Randomize nickname button
/// - `2`: Server address input box
/// - `3`: Connect action button
/// - `4`: Back / Cancel action button
pub fn multiplayer_menu_rects(width: usize, _height: usize) -> [(i32, i32, i32, i32); 5] {
    let cx = width as i32 / 2;
    let input_w = 460;
    let input_h = 38;
    let x_left = cx - input_w / 2;

    let y_nick = 145;
    let nick_input_w = 330;
    let rand_btn_w = 120;
    let rand_btn_x = x_left + nick_input_w + 10;

    let y_addr = 230;
    let y_bottom = 360;

    let btn_w = 220;
    let btn_h = 44;
    let btn_gap = 20;

    [
        // 0: Nickname input
        (x_left, y_nick, nick_input_w, input_h),
        // 1: Randomize Nick button
        (rand_btn_x, y_nick, rand_btn_w, input_h),
        // 2: Server Address input
        (x_left, y_addr, input_w, input_h),
        // 3: Connect button
        (cx - btn_w - btn_gap / 2, y_bottom, btn_w, btn_h),
        // 4: Back button
        (cx + btn_gap / 2, y_bottom, btn_w, btn_h),
    ]
}

/// Renders the Multiplayer configuration & connect screen.
#[allow(clippy::too_many_arguments)]
pub fn draw_multiplayer_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    nickname: &str,
    server_addr: &str,
    focused_field: usize,
    hovered_idx: Option<usize>,
    status_msg: Option<&str>,
    is_connecting: bool,
) {
    let cx = width as i32 / 2;
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.06, 0.07, 0.09));
    fill_rect_alpha(buffer, width, height, cx - 260, 40, 520, 440, pack_color(0.02, 0.03, 0.05), 0.78);

    // Title
    let title = crate::core::i18n::t("multiplayer.title");
    let tw = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - tw / 2, 60, 3, 2, pack_color(0.95, 0.85, 0.42));

    let rects = multiplayer_menu_rects(width, height);

    // 1. Nickname Label
    let nick_label = crate::core::i18n::t("multiplayer.nickname_label");
    draw_text_colored_shadowed(buffer, width, height, nick_label, rects[0].0, rects[0].1 - 22, 1, 1, pack_color(0.75, 0.80, 0.90));

    // 0: Nickname input box
    let (nx, ny, nw, nh) = rects[0];
    let n_focus = focused_field == 0;
    let n_bg = if n_focus { pack_color(0.12, 0.16, 0.24) } else { pack_color(0.08, 0.09, 0.12) };
    let n_border = if n_focus { pack_color(0.40, 0.65, 0.95) } else { pack_color(0.30, 0.35, 0.45) };
    fill_rect(buffer, width, height, nx, ny, nw, nh, n_bg);
    fill_rect(buffer, width, height, nx, ny, nw, 2, n_border);
    fill_rect(buffer, width, height, nx, ny, 2, nh, n_border);
    fill_rect(buffer, width, height, nx, ny + nh - 2, nw, 2, n_border);
    fill_rect(buffer, width, height, nx + nw - 2, ny, 2, nh, n_border);

    let display_nick = if n_focus {
        format!("{}_", nickname)
    } else {
        nickname.to_string()
    };
    draw_text_colored_shadowed(buffer, width, height, &display_nick, nx + 10, ny + 11, 2, 1, pack_color(1.0, 1.0, 1.0));

    // 1: Randomize Nick Button
    let (rx, ry, rw, rh) = rects[1];
    let r_hover = hovered_idx == Some(1);
    let r_bg = if r_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.24, 0.30) };
    let r_top = if r_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.50, 0.55, 0.65) };
    let r_bot = if r_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.12, 0.16) };
    fill_rect(buffer, width, height, rx, ry, rw, rh, r_bg);
    fill_rect(buffer, width, height, rx, ry, rw, 2, r_top);
    fill_rect(buffer, width, height, rx, ry, 2, rh, r_top);
    fill_rect(buffer, width, height, rx, ry + rh - 2, rw, 2, r_bot);
    fill_rect(buffer, width, height, rx + rw - 2, ry, 2, rh, r_bot);
    let rand_label = crate::core::i18n::t("multiplayer.random_nick");
    let rtw = font::measure_text(rand_label, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, rand_label, rx + (rw - rtw) / 2, ry + 13, 1, 1, pack_color(1.0, 1.0, 0.8));

    // 2. Server Address Label
    let addr_label = crate::core::i18n::t("multiplayer.server_label");
    draw_text_colored_shadowed(buffer, width, height, addr_label, rects[2].0, rects[2].1 - 22, 1, 1, pack_color(0.75, 0.80, 0.90));

    // 2: Server Address input box
    let (sx, sy, sw, sh) = rects[2];
    let s_focus = focused_field == 1;
    let s_bg = if s_focus { pack_color(0.12, 0.16, 0.24) } else { pack_color(0.08, 0.09, 0.12) };
    let s_border = if s_focus { pack_color(0.40, 0.65, 0.95) } else { pack_color(0.30, 0.35, 0.45) };
    fill_rect(buffer, width, height, sx, sy, sw, sh, s_bg);
    fill_rect(buffer, width, height, sx, sy, sw, 2, s_border);
    fill_rect(buffer, width, height, sx, sy, 2, sh, s_border);
    fill_rect(buffer, width, height, sx, sy + sh - 2, sw, 2, s_border);
    fill_rect(buffer, width, height, sx + sw - 2, sy, 2, sh, s_border);

    let display_addr = if s_focus {
        format!("{}_", server_addr)
    } else {
        server_addr.to_string()
    };
    draw_text_colored_shadowed(buffer, width, height, &display_addr, sx + 10, sy + 11, 2, 1, pack_color(0.9, 0.95, 1.0));

    // Status or Error Message
    if let Some(status) = status_msg {
        let col = if is_connecting { pack_color(0.4, 0.9, 0.4) } else { pack_color(0.95, 0.4, 0.4) };
        let stw = font::measure_text(status, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, status, cx - stw / 2, 305, 1, 1, col);
    }

    // 3: Connect Button
    let (bx, by, bw, bh) = rects[3];
    let b_hover = hovered_idx == Some(3);
    let connect_label = if is_connecting {
        crate::core::i18n::t("multiplayer.connecting")
    } else {
        crate::core::i18n::t("multiplayer.connect_btn")
    };
    let b_bg = if b_hover { pack_color(0.25, 0.55, 0.35) } else { pack_color(0.18, 0.38, 0.24) };
    let b_top = if b_hover { pack_color(0.50, 0.85, 0.60) } else { pack_color(0.35, 0.60, 0.45) };
    let b_bot = if b_hover { pack_color(0.10, 0.25, 0.15) } else { pack_color(0.08, 0.18, 0.10) };
    fill_rect(buffer, width, height, bx, by, bw, bh, b_bg);
    fill_rect(buffer, width, height, bx, by, bw, 2, b_top);
    fill_rect(buffer, width, height, bx, by, 2, bh, b_top);
    fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, b_bot);
    fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, b_bot);
    let btw = font::measure_text(connect_label, 2, 1);
    draw_text_colored_shadowed(buffer, width, height, connect_label, bx + (bw - btw) / 2, by + 12, 2, 1, pack_color(1.0, 1.0, 1.0));

    // 4: Back Button
    let (kx, ky, kw, kh) = rects[4];
    let k_hover = hovered_idx == Some(4);
    let back_label = crate::core::i18n::t("multiplayer.cancel_btn");
    let k_bg = if k_hover { pack_color(0.45, 0.25, 0.25) } else { pack_color(0.26, 0.22, 0.22) };
    let k_top = if k_hover { pack_color(0.85, 0.55, 0.55) } else { pack_color(0.55, 0.45, 0.45) };
    let k_bot = if k_hover { pack_color(0.25, 0.10, 0.10) } else { pack_color(0.12, 0.10, 0.10) };
    fill_rect(buffer, width, height, kx, ky, kw, kh, k_bg);
    fill_rect(buffer, width, height, kx, ky, kw, 2, k_top);
    fill_rect(buffer, width, height, kx, ky, 2, kh, k_top);
    fill_rect(buffer, width, height, kx, ky + kh - 2, kw, 2, k_bot);
    fill_rect(buffer, width, height, kx + kw - 2, ky, 2, kh, k_bot);
    let ktw = font::measure_text(back_label, 2, 1);
    draw_text_colored_shadowed(buffer, width, height, back_label, kx + (kw - ktw) / 2, ky + 12, 2, 1, pack_color(1.0, 1.0, 1.0));
}
