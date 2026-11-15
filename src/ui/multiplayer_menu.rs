//! Multiplayer join menu and server connection interface with LAN discovery.

use crate::net::DiscoveredServer;
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Button, Rect, TextInput, UiCanvas};

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
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    // Dimmed background & modal panel
    ui.fill(ui.screen_rect(), pack_color(0.05, 0.06, 0.08));
    ui.modal_panel(
        Rect::centered_x(width, 18, 650, 498),
        pack_color(0.02, 0.03, 0.05),
        0.88,
        pack_color(0.25, 0.30, 0.45),
        pack_color(0.15, 0.18, 0.25),
    );

    // Title
    ui.text_centered_x(
        crate::core::i18n::t("multiplayer.title"),
        cx,
        30,
        3,
        2,
        pack_color(0.98, 0.86, 0.42),
    );

    let rects = multiplayer_menu_rects(width, height);

    // 0: Nickname input box
    ui.text_input(
        rects[0],
        TextInput::new(nickname)
            .label(crate::core::i18n::t("multiplayer.nickname_label"))
            .focused(focused_field == 0)
            .cursor(true),
    );

    // 1: Randomize Nick Button
    ui.button(
        rects[1],
        Button::new(crate::core::i18n::t("multiplayer.random_nick"))
            .scale(1)
            .hovered(hovered_idx == Some(1))
            .text_color(pack_color(1.0, 0.95, 0.70)),
    );

    // 2: Server Address input box
    let addr_rect = Rect::from_tuple(rects[2]);
    let max_visible_chars = ((addr_rect.w - 20) / 12).max(1) as usize;
    let mut visible_addr: String = server_addr.chars().take(max_visible_chars).collect();
    if server_addr.chars().count() > max_visible_chars {
        visible_addr.pop();
        visible_addr.push_str("...");
    }

    ui.text_input(
        addr_rect,
        TextInput::new(&visible_addr)
            .label(crate::core::i18n::t("multiplayer.server_invite_label"))
            .focused(focused_field == 1)
            .cursor(true),
    );
    ui.text(
        crate::core::i18n::t("multiplayer.address_hint"),
        addr_rect.x,
        addr_rect.bottom() + 2,
        1,
        1,
        pack_color(0.55, 0.72, 0.88),
    );

    // Section 3: LAN Local Servers Header
    let lan_title = crate::core::i18n::t("multiplayer.lan_title");
    let scan_hint = if is_refreshing {
        "[*] SCANNING LOCAL NETWORK (UDP 26655)..."
    } else if lan_servers.is_empty() {
        "[!] NO LOCAL HOSTS FOUND"
    } else {
        "[+] LOCAL SERVERS DISCOVERED"
    };

    ui.text(lan_title, addr_rect.x, 180, 1, 1, pack_color(0.95, 0.85, 0.40));
    let sh_col = if is_refreshing {
        pack_color(0.35, 0.85, 1.0)
    } else if lan_servers.is_empty() {
        pack_color(0.85, 0.65, 0.35)
    } else {
        pack_color(0.40, 0.95, 0.50)
    };
    ui.text_right(scan_hint, addr_rect.right(), 180, 1, 1, sh_col);

    // Render LAN Server Cards (or Empty State)
    if lan_servers.is_empty() {
        let empty_rect = Rect::new(addr_rect.x, 206, addr_rect.w, 236);
        ui.bordered_box(
            empty_rect,
            2,
            pack_color(0.04, 0.05, 0.08),
            pack_color(0.18, 0.22, 0.30),
        );

        let msg1 = if is_refreshing {
            "Searching for RustCraft servers on local Wi-Fi / LAN..."
        } else {
            crate::core::i18n::t("multiplayer.lan_empty")
        };
        ui.text_centered_x(msg1, cx, empty_rect.y + 80, 1, 1, pack_color(0.80, 0.85, 0.95));
        ui.text_centered_x(
            "Dedicated servers broadcast automatically on UDP port 26655.",
            cx,
            empty_rect.y + 110,
            1,
            1,
            pack_color(0.55, 0.60, 0.70),
        );
        ui.text_centered_x(
            "Works across PC, Linux, macOS, Android, and PS Vita.",
            cx,
            empty_rect.y + 130,
            1,
            1,
            pack_color(0.40, 0.70, 0.85),
        );
    } else {
        for (i, server) in lan_servers.iter().take(MAX_LAN_SERVERS_DISPLAY).enumerate() {
            let card = Rect::from_tuple(rects[6 + i]);
            let is_selected = selected_lan == Some(i);
            let is_card_hover = hovered_idx == Some(6 + i);

            let (card_bg, border_col) = if is_selected {
                (pack_color(0.10, 0.18, 0.30), pack_color(0.35, 0.75, 1.0))
            } else if is_card_hover {
                (pack_color(0.08, 0.12, 0.20), pack_color(0.45, 0.60, 0.80))
            } else {
                (pack_color(0.04, 0.06, 0.09), pack_color(0.20, 0.24, 0.34))
            };

            ui.bordered_box(card, 2, card_bg, border_col);

            // Left badge: "[LAN]"
            ui.text("[LAN]", card.x + 12, card.y + 10, 1, 1, pack_color(0.35, 0.85, 1.0));

            // MOTD
            let motd_display = if server.motd.is_empty() {
                "RustCraft World"
            } else {
                &server.motd
            };
            ui.text(motd_display, card.x + 65, card.y + 9, 2, 1, pack_color(1.0, 1.0, 1.0));

            // Player count badge on the right
            let player_txt = format!("{}/{} players", server.player_count, server.max_players);
            let p_col = if server.player_count >= server.max_players {
                pack_color(0.95, 0.45, 0.45)
            } else {
                pack_color(0.40, 0.95, 0.55)
            };
            ui.text_right(&player_txt, card.right() - 14, card.y + 10, 1, 1, p_col);

            // Subtitle: Address and selection status
            let addr_line = format!("Server: {}", server.addr());
            ui.text(&addr_line, card.x + 14, card.y + 32, 1, 1, pack_color(0.65, 0.72, 0.85));

            let (status_tag, tag_col) = if is_selected {
                (
                    "[SELECTED - CLICK CONNECT OR PRESS ENTER]",
                    pack_color(1.0, 0.90, 0.40),
                )
            } else {
                ("[CLICK TO SELECT]", pack_color(0.45, 0.50, 0.60))
            };
            ui.text_right(status_tag, card.right() - 14, card.y + 32, 1, 1, tag_col);
        }
    }

    // Status or Error Message (if any)
    if let Some(status) = status_msg {
        let col = if is_connecting {
            pack_color(0.40, 0.95, 0.50)
        } else {
            pack_color(0.98, 0.40, 0.40)
        };
        ui.text_centered_x(status, cx, 442, 1, 1, col);
    }

    // 3: Connect Button
    let connect_label = if is_connecting {
        crate::core::i18n::t("multiplayer.connecting")
    } else {
        crate::core::i18n::t("multiplayer.connect_btn")
    };
    ui.button(
        rects[3],
        Button::new(connect_label)
            .primary()
            .hovered(hovered_idx == Some(3)),
    );

    // 5: Refresh Button
    let refresh_text = if is_refreshing {
        "SCANNING...".to_string()
    } else {
        format!("{} [R]", crate::core::i18n::t("multiplayer.refresh_btn"))
    };
    let refresh_col = if is_refreshing {
        pack_color(0.5, 0.9, 1.0)
    } else {
        pack_color(1.0, 1.0, 1.0)
    };
    ui.button(
        rects[5],
        Button::new(&refresh_text)
            .accent()
            .hovered(hovered_idx == Some(5))
            .text_color(refresh_col),
    );

    // 4: Back Button
    ui.button(
        rects[4],
        Button::new(crate::core::i18n::t("multiplayer.cancel_btn"))
            .danger()
            .hovered(hovered_idx == Some(4)),
    );

    let build_info = crate::core::build_info::get_build_info();
    ui.text_centered_x(
        build_info,
        cx,
        height as i32 - 14,
        1,
        1,
        pack_color(0.48, 0.54, 0.65),
    );
}
