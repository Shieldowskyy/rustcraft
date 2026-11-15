//! Player list overlay interface displayed when holding the TAB key.

use crate::net::protocol::PlayerInfo;
use crate::ui::draw::{fill_rect, pack_color};
use crate::ui::widgets::{Rect, UiCanvas};

/// Renders the Minecraft-style translucent player list overlay at the top-center of the screen.
pub fn draw_player_list_tab(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    players: &[PlayerInfo],
    local_nickname: &str,
    is_multiplayer: bool,
) {
    if players.is_empty() && !is_multiplayer {
        return;
    }

    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();
    let list_len = players.len().max(1);
    let row_height = 20;
    let header_height = 28;
    let padding = 10;
    let table_width = 380;
    let table_height = header_height + (list_len as i32 * row_height) + padding;
    let start_x = cx - table_width / 2;
    let start_y = 35;
    let table_rect = Rect::new(start_x, start_y, table_width, table_height);

    // Translucent background & subtle border
    ui.fill_alpha(table_rect, pack_color(0.04, 0.05, 0.08), 0.82);
    ui.border(table_rect, 1, pack_color(0.35, 0.45, 0.60));

    // Header title
    let header_text = if is_multiplayer {
        format!("RustCraft Server  -  {} Online", players.len())
    } else {
        "RustCraft Singleplayer  -  1 Player".to_string()
    };
    ui.text_centered_x(
        &header_text,
        cx,
        start_y + 8,
        1,
        1,
        pack_color(0.95, 0.85, 0.45),
    );

    // Divider line
    ui.fill(
        Rect::new(start_x + 10, start_y + header_height - 2, table_width - 20, 1),
        pack_color(0.25, 0.30, 0.40),
    );

    let items_start_y = start_y + header_height + 4;

    if !is_multiplayer || players.is_empty() {
        let row_y = items_start_y;
        draw_player_avatar(ui.buffer, ui.width, ui.height, start_x + 14, row_y + 4);
        ui.text(
            local_nickname,
            start_x + 30,
            row_y + 5,
            1,
            1,
            pack_color(1.0, 0.95, 0.3),
        );
        ui.text_right(
            "0ms",
            start_x + table_width - 16,
            row_y + 5,
            1,
            1,
            pack_color(0.35, 0.95, 0.45),
        );
    } else {
        for (i, p) in players.iter().enumerate() {
            let row_y = items_start_y + (i as i32 * row_height);
            let is_local = p.nickname == local_nickname;

            if i % 2 == 1 {
                ui.fill_alpha(
                    Rect::new(start_x + 6, row_y, table_width - 12, row_height - 2),
                    pack_color(0.12, 0.15, 0.22),
                    0.5,
                );
            }

            draw_player_avatar(ui.buffer, ui.width, ui.height, start_x + 14, row_y + 4);

            let nick_color = if is_local {
                pack_color(1.0, 0.92, 0.35)
            } else {
                pack_color(0.92, 0.94, 0.98)
            };
            ui.text(&p.nickname, start_x + 30, row_y + 5, 1, 1, nick_color);

            let ping_color = if p.ping_ms < 50 {
                pack_color(0.35, 0.95, 0.45)
            } else if p.ping_ms < 150 {
                pack_color(0.95, 0.85, 0.3)
            } else {
                pack_color(0.95, 0.35, 0.35)
            };
            let ping_str = format!("{}ms", p.ping_ms);
            ui.text_right(&ping_str, start_x + table_width - 16, row_y + 5, 1, 1, ping_color);
        }
    }
}

/// Draws an 8x8 pixel humanoid face avatar icon.
fn draw_player_avatar(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32) {
    let skin = pack_color(0.85, 0.65, 0.50);
    let hair = pack_color(0.35, 0.20, 0.10);
    let eye_white = pack_color(0.95, 0.95, 0.95);
    let eye_blue = pack_color(0.20, 0.45, 0.85);

    // 8x8 box
    fill_rect(buffer, width, height, x, y, 10, 10, skin);
    // Hair on top 3 lines
    fill_rect(buffer, width, height, x, y, 10, 3, hair);
    fill_rect(buffer, width, height, x, y + 3, 2, 2, hair);
    fill_rect(buffer, width, height, x + 8, y + 3, 2, 2, hair);
    // Eyes
    fill_rect(buffer, width, height, x + 2, y + 5, 2, 2, eye_white);
    fill_rect(buffer, width, height, x + 3, y + 5, 1, 2, eye_blue);
    fill_rect(buffer, width, height, x + 6, y + 5, 2, 2, eye_white);
    fill_rect(buffer, width, height, x + 7, y + 5, 1, 2, eye_blue);
}
