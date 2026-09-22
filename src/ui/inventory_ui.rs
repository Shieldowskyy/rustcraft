use crate::gameplay::player::{GameMode, Player};
use crate::ui::draw::{draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::ui::hud::draw_hotbar;
use crate::world::block::{BlockId, Face};

pub fn draw_inventory(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    player: &Player,
    selected_slot: Option<(usize, usize)>,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.5);

    let start_x = width as i32 / 2 - 200;
    let start_y = height as i32 / 2 - 150;

    fill_rect_alpha(buffer, width, height, start_x - 10, start_y - 10, 400, 300, pack_color(0.1, 0.1, 0.1), 0.9);
    draw_text_shadowed(buffer, width, height, "EKWIPUNEK", start_x, start_y - 30, 2, 1);
    draw_text_shadowed(buffer, width, height, "[A] Wybierz   [Y]/[B] Zamknij   [LB]/[RB] Slot", start_x, start_y + 270, 1, 1);

    if player.gamemode == GameMode::Creative {
        let mut block_id = 1;
        'outer: for row in 0..5 {
            for col in 0..8 {
                if block_id > 19 {
                    break 'outer;
                }
                let x = start_x + col as i32 * 40;
                let y = start_y + row as i32 * 40;

                let is_hovered = selected_slot == Some((row, col));
                let border_color = if is_hovered {
                    pack_color(1.0, 0.9, 0.2)
                } else {
                    pack_color(0.3, 0.3, 0.3)
                };

                fill_rect(buffer, width, height, x, y, 36, 36, border_color);
                let block = BlockId::from_u8(block_id);
                let color = block.face_color(Face::Top);
                fill_rect(buffer, width, height, x + 2, y + 2, 32, 32, pack_color(color[0], color[1], color[2]));

                block_id += 1;
            }
        }
    } else {
        draw_text_shadowed(buffer, width, height, "TRYB PRZETRWANIA - BRAK BLOKOW", start_x, start_y, 2, 1);
    }

    draw_hotbar(buffer, width, height, player);
}
