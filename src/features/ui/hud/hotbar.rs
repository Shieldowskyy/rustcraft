//! HUD hotbar, off-hand slot, survival status bars (health, armor, hunger, air bubbles), and held item popup.
//! Zero platform `#[cfg]`.

use crate::features::player::{GameMode, Player};
use crate::features::rendering::font;

use super::super::draw::{pack_color, set_px};
use super::super::widgets::{Grid, Rect, UiCanvas};
use super::icons::{
    ArmorIconState, BubbleState, HeartIconState, HungerIconState, draw_armor_icon, draw_bubble,
    draw_heart_icon, draw_hunger_icon,
};

pub fn draw_hotbar(buffer: &mut [u32], width: usize, height: usize, player: &Player) {
    let slot_size = 28i32;
    let margin = 3i32;
    let total_slots = 10usize;
    let hotbar_grid = Grid::square(0, 0, total_slots, 1, slot_size, margin);
    let total_w = hotbar_grid.total_width();
    let start_x = (width as i32 - total_w) / 2;
    let y = height as i32 - slot_size - 12;
    let hotbar_grid = Grid::square(start_x, y, total_slots, 1, slot_size, margin);

    let mut ui = UiCanvas::new(buffer, width, height);
    ui.fill_alpha(
        Rect::new(start_x - 6, y - 6, total_w + 12, slot_size + 12),
        pack_color(0.0, 0.0, 0.0),
        0.40,
    );

    for i in 0..total_slots {
        let is_selected = i == player.selected_hotbar_slot as usize;
        let border_color = if is_selected {
            pack_color(1.0, 1.0, 0.2)
        } else {
            pack_color(0.20, 0.20, 0.20)
        };
        ui.hotbar_slot(
            hotbar_grid.slot(i),
            player.inventory.hotbar[i].as_ref(),
            border_color,
            pack_color(0.3, 0.3, 0.3),
        );
    }

    let offhand_x = start_x - slot_size - 10;
    let offhand_rect = Rect::new(offhand_x, y, slot_size, slot_size);
    ui.fill_alpha(offhand_rect.inset(-4), pack_color(0.0, 0.0, 0.0), 0.35);
    ui.hotbar_slot(
        offhand_rect,
        player.inventory.offhand.as_ref(),
        pack_color(0.25, 0.25, 0.25),
        pack_color(0.28, 0.28, 0.28),
    );
    if player.inventory.offhand.is_none() {
        let sx = offhand_x + 8;
        let sy = y + 7;
        let sil = pack_color(0.38, 0.38, 0.38);
        ui.fill(Rect::new(sx, sy, 12, 2), sil);
        ui.fill(Rect::new(sx + 1, sy + 2, 10, 6), sil);
        ui.fill(Rect::new(sx + 2, sy + 8, 8, 3), sil);
        ui.fill(Rect::new(sx + 4, sy + 11, 4, 2), sil);
        set_px(ui.buffer, ui.width, ui.height, sx + 5, sy + 13, sil);
        set_px(ui.buffer, ui.width, ui.height, sx + 6, sy + 13, sil);
    }

    if player.gamemode == GameMode::Survival {
        let health_start_x = start_x;
        let health_y = y - 16;
        let hp = player.health.ceil() as i32;
        let max_hearts = (player.max_health as i32 / 2).max(1);
        let is_flashing = player.invulnerability_timer > 0.0;
        let is_low_health = player.health <= 4.0 && player.health > 0.0;

        for i in 0..max_hearts {
            let heart_x = health_start_x + i * 10;
            let threshold = (i + 1) * 2;
            let state = if hp >= threshold {
                HeartIconState::Full
            } else if hp == threshold - 1 {
                HeartIconState::Half
            } else {
                HeartIconState::Empty
            };
            let wobble = if is_low_health {
                if (i + (player.invulnerability_timer * 15.0) as i32) % 3 == 1 {
                    -1
                } else {
                    0
                }
            } else {
                0
            };
            draw_heart_icon(
                ui.buffer,
                ui.width,
                ui.height,
                heart_x,
                health_y + wobble,
                state,
                is_flashing,
            );
        }

        let defense_pts = player.armor_defense_points().round() as i32;
        if defense_pts > 0 {
            let armor_y = health_y - 10;
            for i in 0..10 {
                let armor_x = health_start_x + i * 10;
                let threshold = (i + 1) * 2;
                let state = if defense_pts >= threshold {
                    ArmorIconState::Full
                } else if defense_pts == threshold - 1 {
                    ArmorIconState::Half
                } else {
                    ArmorIconState::Empty
                };
                draw_armor_icon(ui.buffer, ui.width, ui.height, armor_x, armor_y, state);
            }
        }

        let hunger_start_x = start_x + total_w - 98;
        let hunger_pts = player.hunger.ceil() as i32;
        let is_starving = player.hunger <= 6.0;

        for i in 0..10 {
            let threshold = (i + 1) * 2;
            let state = if hunger_pts >= threshold {
                HungerIconState::Full
            } else if hunger_pts == threshold - 1 {
                HungerIconState::Half
            } else {
                HungerIconState::Empty
            };

            let wobble = if is_starving && player.hunger > 0.0 {
                if (i + (player.starvation_timer * 15.0) as i32) % 3 == 1 {
                    -1
                } else {
                    0
                }
            } else {
                0
            };

            let hunger_x = hunger_start_x + i * 10;
            draw_hunger_icon(
                ui.buffer,
                ui.width,
                ui.height,
                hunger_x,
                health_y + wobble,
                state,
            );
        }

        if player.head_underwater || player.air < player.max_air {
            let bubble_y = health_y - 10;
            let air_pts = player.air.ceil() as i32;
            for i in 0..10 {
                let bubble_x = hunger_start_x + i * 10;
                let state = if air_pts >= (i + 1) * 2 {
                    BubbleState::Full
                } else if air_pts == (i + 1) * 2 - 1 {
                    BubbleState::Pop
                } else {
                    BubbleState::Empty
                };
                draw_bubble(ui.buffer, ui.width, ui.height, bubble_x, bubble_y, state);
            }
        }
    }

    if player.held_item_name_timer > 0.0 {
        if let Some(stack) = &player.inventory.hotbar[player.selected_hotbar_slot as usize] {
            let item_name = stack.block.display_name();
            let scale = 1;
            let tw = font::measure_text(item_name, scale, 1);
            let tx = (width as i32 - tw) / 2;
            let ty = if player.gamemode == GameMode::Survival {
                if player.head_underwater || player.air < player.max_air {
                    y - 40
                } else {
                    y - 28
                }
            } else {
                y - 18
            };

            let alpha = if player.held_item_name_timer > 0.6 {
                1.0
            } else {
                (player.held_item_name_timer / 0.6).clamp(0.0, 1.0)
            };

            ui.fill_alpha(
                Rect::new(tx - 6, ty - 3, tw + 12, font::GLYPH_H as i32 + 6),
                pack_color(0.0, 0.0, 0.0),
                0.60 * alpha,
            );
            ui.text_white(item_name, tx, ty, scale, 1);
        }
    }
}
