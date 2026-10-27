//! Multiplayer join menu and server connection interface with LAN discovery.

use crate::graphics::font;
use crate::net::DiscoveredServer;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};

pub const MAX_LAN_SERVERS_DISPLAY: usize = 4;
pub const MULTIPLAYER_RECT_COUNT: usize = 6 + MAX_LAN_SERVERS_DISPLAY;

/// Calculates layout bounding boxes for interactive multiplayer menu elements:
/// - `0`: Nickname input box
/// - `1`: Randomize nickname button
/// - `2`: Server address input box
/// - `3`: Connect action button
/// - `4`: Back / Cancel action button
/// - `5`: Refresh LAN servers button
/// - `6..9`: Discovered LAN server cards
pub fn multiplayer_menu_rects(width: usize, _height: usize) -> [(i32, i32, i32, i32); MULTIPLAYER_RECT_COUNT] {
    let cx = width as i32 / 2;
    let card_w = 600;
    let x_left = cx - card_w / 2; // cx - 300

    let y_nick = 78;
    let nick_input_w = 420;
    let rand_btn_w = 160;
    let rand_btn_x = x_left + nick_input_w + 20;

    let y_addr = 132;
    let addr_input_w = card_w;

    let y_bottom = 458;
    let btn_w = 190;
    let btn_h = 42;
    let btn_gap = 15;
    let btn0_x = cx - 300;
    let btn1_x = btn0_x + btn_w + btn_gap; // cx - 95
    let btn2_x = btn1_x + btn_w + btn_gap; // cx + 110

    let lan_start_y = 206;
    let lan_h = 54;
    let lan_gap = 6;

    [
        // 0: Nickname input
        (x_left, y_nick, nick_input_w, 32),
        // 1: Randomize Nick button
        (rand_btn_x, y_nick, rand_btn_w, 32),
        // 2: Server Address input
        (x_left, y_addr, addr_input_w, 32),
        // 3: Connect button
        (btn0_x, y_bottom, btn_w, btn_h),
        // 4: Back button
        (btn2_x, y_bottom, btn_w, btn_h),
        // 5: Refresh button
        (btn1_x, y_bottom, btn_w, btn_h),
        // 6..9: LAN server cards 0..3
        (x_left, lan_start_y, card_w, lan_h),
        (x_left, lan_start_y + (lan_h + lan_gap), card_w, lan_h),
        (x_left, lan_start_y + (lan_h + lan_gap) * 2, card_w, lan_h),
        (x_left, lan_start_y + (lan_h + lan_gap) * 3, card_w, lan_h),
    ]
}

/// Renders the Multiplayer configuration & connect screen with LAN server discovery list.
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
    lan_servers: &[DiscoveredServer],
    selected_lan: Option<usize>,
    is_refreshing: bool,
) {
    let cx = width as i32 / 2;

    // Dimmed background
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.05, 0.06, 0.08));
    fill_rect_alpha(buffer, width, height, cx - 325, 18, 650, 498, pack_color(0.02, 0.03, 0.05), 0.88);

    // Modal border
    fill_rect(buffer, width, height, cx - 325, 18, 650, 2, pack_color(0.25, 0.30, 0.45));
    fill_rect(buffer, width, height, cx - 325, 514, 650, 2, pack_color(0.15, 0.18, 0.25));
    fill_rect(buffer, width, height, cx - 325, 18, 2, 498, pack_color(0.20, 0.25, 0.35));
    fill_rect(buffer, width, height, cx + 323, 18, 2, 498, pack_color(0.20, 0.25, 0.35));

    // Title
    let title = crate::core::i18n::t("multiplayer.title");
    let tw = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - tw / 2, 30, 3, 2, pack_color(0.98, 0.86, 0.42));

    let rects = multiplayer_menu_rects(width, height);

    // 1. Nickname Label
    let nick_label = crate::core::i18n::t("multiplayer.nickname_label");
    draw_text_colored_shadowed(buffer, width, height, nick_label, rects[0].0, rects[0].1 - 18, 1, 1, pack_color(0.75, 0.82, 0.92));

    // 0: Nickname input box
    let (nx, ny, nw, nh) = rects[0];
    let n_focus = focused_field == 0;
    let n_bg = if n_focus { pack_color(0.12, 0.16, 0.25) } else { pack_color(0.07, 0.09, 0.13) };
    let n_border = if n_focus { pack_color(0.40, 0.70, 1.0) } else { pack_color(0.28, 0.32, 0.42) };
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
    draw_text_colored_shadowed(buffer, width, height, &display_nick, nx + 10, ny + 9, 2, 1, pack_color(1.0, 1.0, 1.0));

    // 1: Randomize Nick Button
    let (rx, ry, rw, rh) = rects[1];
    let r_hover = hovered_idx == Some(1);
    let r_bg = if r_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.20, 0.23, 0.30) };
    let r_top = if r_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.45, 0.50, 0.60) };
    let r_bot = if r_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.12, 0.16) };
    fill_rect(buffer, width, height, rx, ry, rw, rh, r_bg);
    fill_rect(buffer, width, height, rx, ry, rw, 2, r_top);
    fill_rect(buffer, width, height, rx, ry, 2, rh, r_top);
    fill_rect(buffer, width, height, rx, ry + rh - 2, rw, 2, r_bot);
    fill_rect(buffer, width, height, rx + rw - 2, ry, 2, rh, r_bot);
    let rand_label = crate::core::i18n::t("multiplayer.random_nick");
    let rtw = font::measure_text(rand_label, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, rand_label, rx + (rw - rtw) / 2, ry + 11, 1, 1, pack_color(1.0, 0.95, 0.70));

    // 2. Server Address Label
    let addr_label = crate::core::i18n::t("multiplayer.server_label");
    draw_text_colored_shadowed(buffer, width, height, addr_label, rects[2].0, rects[2].1 - 18, 1, 1, pack_color(0.75, 0.82, 0.92));

    // 2: Server Address input box
    let (sx, sy, sw, sh) = rects[2];
    let s_focus = focused_field == 1;
    let s_bg = if s_focus { pack_color(0.12, 0.16, 0.25) } else { pack_color(0.07, 0.09, 0.13) };
    let s_border = if s_focus { pack_color(0.40, 0.70, 1.0) } else { pack_color(0.28, 0.32, 0.42) };
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
    draw_text_colored_shadowed(buffer, width, height, &display_addr, sx + 10, sy + 9, 2, 1, pack_color(0.90, 0.95, 1.0));

    // Section 3: LAN Local Servers Header
    let lan_title = crate::core::i18n::t("multiplayer.lan_title");
    let scan_hint = if is_refreshing {
        "[*] SCANNING LOCAL NETWORK (UDP 26655)..."
    } else if lan_servers.is_empty() {
        "[!] NO LOCAL HOSTS FOUND"
    } else {
        "[+] LOCAL SERVERS DISCOVERED"
    };

    draw_text_colored_shadowed(buffer, width, height, lan_title, rects[2].0, 180, 1, 1, pack_color(0.95, 0.85, 0.40));
    let sh_w = font::measure_text(scan_hint, 1, 1);
    let sh_col = if is_refreshing {
        pack_color(0.35, 0.85, 1.0)
    } else if lan_servers.is_empty() {
        pack_color(0.85, 0.65, 0.35)
    } else {
        pack_color(0.40, 0.95, 0.50)
    };
    draw_text_colored_shadowed(buffer, width, height, scan_hint, rects[2].0 + rects[2].2 - sh_w, 180, 1, 1, sh_col);

    // Render LAN Server Cards (or Empty State)
    if lan_servers.is_empty() {
        let empty_x = rects[2].0;
        let empty_y = 206;
        let empty_w = rects[2].2;
        let empty_h = 236;

        fill_rect(buffer, width, height, empty_x, empty_y, empty_w, empty_h, pack_color(0.04, 0.05, 0.08));
        fill_rect(buffer, width, height, empty_x, empty_y, empty_w, 2, pack_color(0.18, 0.22, 0.30));
        fill_rect(buffer, width, height, empty_x, empty_y, 2, empty_h, pack_color(0.18, 0.22, 0.30));
        fill_rect(buffer, width, height, empty_x, empty_y + empty_h - 2, empty_w, 2, pack_color(0.18, 0.22, 0.30));
        fill_rect(buffer, width, height, empty_x + empty_w - 2, empty_y, 2, empty_h, pack_color(0.18, 0.22, 0.30));

        let msg1 = if is_refreshing {
            "Searching for RustCraft servers on local Wi-Fi / LAN..."
        } else {
            crate::core::i18n::t("multiplayer.lan_empty")
        };
        let m1w = font::measure_text(msg1, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, msg1, cx - m1w / 2, empty_y + 80, 1, 1, pack_color(0.80, 0.85, 0.95));

        let msg2 = "Dedicated servers broadcast automatically on UDP port 26655.";
        let m2w = font::measure_text(msg2, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, msg2, cx - m2w / 2, empty_y + 110, 1, 1, pack_color(0.55, 0.60, 0.70));

        let msg3 = "Works across PC, Linux, macOS, Android, and PS Vita.";
        let m3w = font::measure_text(msg3, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, msg3, cx - m3w / 2, empty_y + 130, 1, 1, pack_color(0.40, 0.70, 0.85));
    } else {
        for (i, server) in lan_servers.iter().take(MAX_LAN_SERVERS_DISPLAY).enumerate() {
            let (lx, ly, lw, lh) = rects[6 + i];
            let is_selected = selected_lan == Some(i);
            let is_card_hover = hovered_idx == Some(6 + i);

            let (card_bg, border_col) = if is_selected {
                (pack_color(0.10, 0.18, 0.30), pack_color(0.35, 0.75, 1.0))
            } else if is_card_hover {
                (pack_color(0.08, 0.12, 0.20), pack_color(0.45, 0.60, 0.80))
            } else {
                (pack_color(0.04, 0.06, 0.09), pack_color(0.20, 0.24, 0.34))
            };

            fill_rect(buffer, width, height, lx, ly, lw, lh, card_bg);
            fill_rect(buffer, width, height, lx, ly, lw, 2, border_col);
            fill_rect(buffer, width, height, lx, ly, 2, lh, border_col);
            fill_rect(buffer, width, height, lx, ly + lh - 2, lw, 2, border_col);
            fill_rect(buffer, width, height, lx + lw - 2, ly, 2, lh, border_col);

            // Left badge: "[LAN]"
            draw_text_colored_shadowed(buffer, width, height, "[LAN]", lx + 12, ly + 10, 1, 1, pack_color(0.35, 0.85, 1.0));

            // MOTD
            let motd_display = if server.motd.is_empty() {
                "RustCraft World"
            } else {
                &server.motd
            };
            draw_text_colored_shadowed(buffer, width, height, motd_display, lx + 65, ly + 9, 2, 1, pack_color(1.0, 1.0, 1.0));

            // Player count badge on the right
            let player_txt = format!("{}/{} players", server.player_count, server.max_players);
            let p_len = font::measure_text(&player_txt, 1, 1);
            let p_col = if server.player_count >= server.max_players {
                pack_color(0.95, 0.45, 0.45)
            } else {
                pack_color(0.40, 0.95, 0.55)
            };
            draw_text_colored_shadowed(buffer, width, height, &player_txt, lx + lw - p_len - 14, ly + 10, 1, 1, p_col);

            // Subtitle: Address and selection status
            let addr_line = format!("Server: {}", server.addr());
            draw_text_colored_shadowed(buffer, width, height, &addr_line, lx + 14, ly + 32, 1, 1, pack_color(0.65, 0.72, 0.85));

            let status_tag = if is_selected {
                "[SELECTED - CLICK CONNECT OR PRESS ENTER]"
            } else {
                "[CLICK TO SELECT]"
            };
            let tag_len = font::measure_text(status_tag, 1, 1);
            let tag_col = if is_selected {
                pack_color(1.0, 0.90, 0.40)
            } else {
                pack_color(0.45, 0.50, 0.60)
            };
            draw_text_colored_shadowed(buffer, width, height, status_tag, lx + lw - tag_len - 14, ly + 32, 1, 1, tag_col);
        }
    }

    // Status or Error Message (if any)
    if let Some(status) = status_msg {
        let col = if is_connecting { pack_color(0.40, 0.95, 0.50) } else { pack_color(0.98, 0.40, 0.40) };
        let stw = font::measure_text(status, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, status, cx - stw / 2, 442, 1, 1, col);
    }

    // 3: Connect Button
    let (bx, by, bw, bh) = rects[3];
    let b_hover = hovered_idx == Some(3);
    let connect_label = if is_connecting {
        crate::core::i18n::t("multiplayer.connecting")
    } else {
        crate::core::i18n::t("multiplayer.connect_btn")
    };
    let b_bg = if b_hover { pack_color(0.24, 0.56, 0.34) } else { pack_color(0.16, 0.38, 0.24) };
    let b_top = if b_hover { pack_color(0.50, 0.85, 0.60) } else { pack_color(0.32, 0.58, 0.42) };
    let b_bot = if b_hover { pack_color(0.10, 0.25, 0.15) } else { pack_color(0.08, 0.18, 0.10) };
    fill_rect(buffer, width, height, bx, by, bw, bh, b_bg);
    fill_rect(buffer, width, height, bx, by, bw, 2, b_top);
    fill_rect(buffer, width, height, bx, by, 2, bh, b_top);
    fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, b_bot);
    fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, b_bot);
    let btw = font::measure_text(connect_label, 2, 1);
    draw_text_colored_shadowed(buffer, width, height, connect_label, bx + (bw - btw) / 2, by + 13, 2, 1, pack_color(1.0, 1.0, 1.0));

    // 5: Refresh Button
    let (fx, fy, fw, fh) = rects[5];
    let f_hover = hovered_idx == Some(5);
    let refresh_text = if is_refreshing {
        "SCANNING...".to_string()
    } else {
        format!("{} [R]", crate::core::i18n::t("multiplayer.refresh_btn"))
    };
    let f_bg = if is_refreshing {
        pack_color(0.18, 0.32, 0.52)
    } else if f_hover {
        pack_color(0.26, 0.44, 0.70)
    } else {
        pack_color(0.18, 0.28, 0.44)
    };
    let f_top = if f_hover { pack_color(0.55, 0.75, 1.0) } else { pack_color(0.35, 0.50, 0.75) };
    let f_bot = if f_hover { pack_color(0.12, 0.18, 0.30) } else { pack_color(0.08, 0.12, 0.20) };
    fill_rect(buffer, width, height, fx, fy, fw, fh, f_bg);
    fill_rect(buffer, width, height, fx, fy, fw, 2, f_top);
    fill_rect(buffer, width, height, fx, fy, 2, fh, f_top);
    fill_rect(buffer, width, height, fx, fy + fh - 2, fw, 2, f_bot);
    fill_rect(buffer, width, height, fx + fw - 2, fy, 2, fh, f_bot);
    let ftw = font::measure_text(&refresh_text, 2, 1);
    let f_text_col = if is_refreshing { pack_color(0.5, 0.9, 1.0) } else { pack_color(1.0, 1.0, 1.0) };
    draw_text_colored_shadowed(buffer, width, height, &refresh_text, fx + (fw - ftw) / 2, fy + 13, 2, 1, f_text_col);

    // 4: Back Button
    let (kx, ky, kw, kh) = rects[4];
    let k_hover = hovered_idx == Some(4);
    let back_label = crate::core::i18n::t("multiplayer.cancel_btn");
    let k_bg = if k_hover { pack_color(0.48, 0.24, 0.24) } else { pack_color(0.28, 0.20, 0.20) };
    let k_top = if k_hover { pack_color(0.85, 0.50, 0.50) } else { pack_color(0.55, 0.40, 0.40) };
    let k_bot = if k_hover { pack_color(0.24, 0.08, 0.08) } else { pack_color(0.12, 0.08, 0.08) };
    fill_rect(buffer, width, height, kx, ky, kw, kh, k_bg);
    fill_rect(buffer, width, height, kx, ky, kw, 2, k_top);
    fill_rect(buffer, width, height, kx, ky, 2, kh, k_top);
    fill_rect(buffer, width, height, kx, ky + kh - 2, kw, 2, k_bot);
    fill_rect(buffer, width, height, kx + kw - 2, ky, 2, kh, k_bot);
    let ktw = font::measure_text(back_label, 2, 1);
    draw_text_colored_shadowed(buffer, width, height, back_label, kx + (kw - ktw) / 2, ky + 13, 2, 1, pack_color(1.0, 1.0, 1.0));

    let build_info = crate::core::build_info::get_build_info();
    let bw = font::measure_text(build_info, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, build_info, (width as i32 - bw) / 2, height as i32 - 14, 1, 1, pack_color(0.48, 0.54, 0.65));
}
