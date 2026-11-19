//! Podgląd modelu gracza w ekwipunku (ze śledzeniem kursora i zbroją) oraz sloty pancerza/offhandu.

use crate::features::blocks::BlockId;
use crate::features::player::{ItemStack, Player};
use crate::features::ui::draw::{fill_rect, fill_rect_alpha, pack_color};
use crate::features::ui::widgets::{Rect, UiCanvas};

use super::layout::InventorySlotTarget;

pub fn draw_player_inventory_preview(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    mouse_pos: Option<(f32, f32)>,
    armor: &[Option<ItemStack>; 4],
    offhand: &Option<ItemStack>,
) {
    let ground_y = center_y + 44;
    fill_rect_alpha(buffer, width, height, center_x - 22, ground_y, 44, 4, pack_color(0.04, 0.04, 0.06), 0.70);
    fill_rect_alpha(buffer, width, height, center_x - 17, ground_y - 1, 34, 6, pack_color(0.02, 0.02, 0.03), 0.55);

    let (look_x, look_y) = if let Some((mx, my)) = mouse_pos {
        let dx = (mx - center_x as f32) / 110.0;
        let dy = (my - (center_y - 20) as f32) / 110.0;
        (dx.clamp(-1.0, 1.0) * 3.0, dy.clamp(-1.0, 1.0) * 2.0)
    } else {
        (0.6, -0.4)
    };
    let lx = look_x.round() as i32;
    let ly = look_y.round() as i32;

    let skin_base = pack_color(0.78, 0.56, 0.44);
    let skin_shadow = pack_color(0.66, 0.46, 0.35);
    let skin_highlight = pack_color(0.85, 0.64, 0.52);

    let hair_base = pack_color(0.28, 0.18, 0.11);
    let hair_dark = pack_color(0.20, 0.12, 0.08);

    let eye_white = pack_color(0.96, 0.96, 0.98);
    let eye_blue = pack_color(0.25, 0.35, 0.75);
    let mouth_color = pack_color(0.38, 0.22, 0.15);

    let shirt_cyan = pack_color(0.00, 0.65, 0.65);
    let shirt_shadow = pack_color(0.00, 0.50, 0.50);
    let shirt_dark = pack_color(0.00, 0.40, 0.40);
    let shirt_highlight = pack_color(0.12, 0.76, 0.76);

    let pants_blue = pack_color(0.18, 0.24, 0.55);
    let pants_shadow = pack_color(0.12, 0.16, 0.42);
    let pants_highlight = pack_color(0.25, 0.32, 0.66);

    let shoes_gray = pack_color(0.32, 0.32, 0.35);
    let shoes_dark = pack_color(0.22, 0.22, 0.24);

    let legs_y = center_y + 16;
    fill_rect(buffer, width, height, center_x - 10, legs_y, 9, 22, pants_blue);
    fill_rect(buffer, width, height, center_x - 10, legs_y, 2, 22, pants_highlight);
    fill_rect(buffer, width, height, center_x - 3, legs_y, 2, 22, pants_shadow);
    fill_rect(buffer, width, height, center_x - 10, legs_y + 22, 9, 6, shoes_gray);
    fill_rect(buffer, width, height, center_x - 10, legs_y + 26, 9, 2, shoes_dark);

    fill_rect(buffer, width, height, center_x + 1, legs_y, 9, 22, pants_blue);
    fill_rect(buffer, width, height, center_x + 1, legs_y, 2, 22, pants_highlight);
    fill_rect(buffer, width, height, center_x + 8, legs_y, 2, 22, pants_shadow);
    fill_rect(buffer, width, height, center_x + 1, legs_y + 22, 9, 6, shoes_gray);
    fill_rect(buffer, width, height, center_x + 1, legs_y + 26, 9, 2, shoes_dark);

    let torso_x = center_x - 10;
    let torso_y = center_y - 12;
    fill_rect(buffer, width, height, torso_x, torso_y, 20, 28, shirt_cyan);
    fill_rect(buffer, width, height, torso_x, torso_y, 2, 28, shirt_highlight);
    fill_rect(buffer, width, height, torso_x + 18, torso_y, 2, 28, shirt_shadow);
    fill_rect(buffer, width, height, torso_x, torso_y + 26, 20, 2, shirt_dark);
    fill_rect(buffer, width, height, torso_x + 7, torso_y, 6, 4, skin_base);
    fill_rect(buffer, width, height, torso_x + 8, torso_y + 4, 4, 2, skin_shadow);

    let left_arm_x = torso_x - 9;
    let left_arm_y = torso_y;
    fill_rect(buffer, width, height, left_arm_x, left_arm_y, 8, 9, shirt_cyan);
    fill_rect(buffer, width, height, left_arm_x, left_arm_y, 2, 9, shirt_highlight);
    fill_rect(buffer, width, height, left_arm_x + 6, left_arm_y, 2, 9, shirt_shadow);
    fill_rect(buffer, width, height, left_arm_x, left_arm_y + 9, 8, 19, skin_base);
    fill_rect(buffer, width, height, left_arm_x, left_arm_y + 9, 2, 19, skin_highlight);
    fill_rect(buffer, width, height, left_arm_x + 6, left_arm_y + 9, 2, 19, skin_shadow);

    let right_arm_x = torso_x + 21;
    let right_arm_y = torso_y;
    fill_rect(buffer, width, height, right_arm_x, right_arm_y, 8, 9, shirt_cyan);
    fill_rect(buffer, width, height, right_arm_x, right_arm_y, 2, 9, shirt_highlight);
    fill_rect(buffer, width, height, right_arm_x + 6, right_arm_y, 2, 9, shirt_shadow);
    fill_rect(buffer, width, height, right_arm_x, right_arm_y + 9, 8, 19, skin_base);
    fill_rect(buffer, width, height, right_arm_x, right_arm_y + 9, 2, 19, skin_highlight);
    fill_rect(buffer, width, height, right_arm_x + 6, right_arm_y + 9, 2, 19, skin_shadow);

    let head_x = center_x - 10 + lx;
    let head_y = torso_y - 20 + ly;
    fill_rect(buffer, width, height, head_x, head_y, 20, 20, skin_base);
    fill_rect(buffer, width, height, head_x, head_y, 20, 6, hair_base);
    fill_rect(buffer, width, height, head_x, head_y, 20, 2, hair_dark);
    fill_rect(buffer, width, height, head_x, head_y + 6, 3, 5, hair_base);
    fill_rect(buffer, width, height, head_x + 17, head_y + 6, 3, 5, hair_base);
    fill_rect(buffer, width, height, head_x + 6, head_y + 6, 2, 2, hair_base);

    let eye_y = head_y + 9;
    let eye_dx = lx.clamp(-1, 1);
    fill_rect(buffer, width, height, head_x + 3, eye_y, 5, 3, eye_white);
    fill_rect(buffer, width, height, head_x + 5 + eye_dx, eye_y, 2, 3, eye_blue);
    fill_rect(buffer, width, height, head_x + 12, eye_y, 5, 3, eye_white);
    fill_rect(buffer, width, height, head_x + 12 + eye_dx, eye_y, 2, 3, eye_blue);

    let nose_x = head_x + 9 + eye_dx;
    let nose_y = head_y + 12;
    fill_rect(buffer, width, height, nose_x, nose_y, 3, 2, skin_shadow);
    fill_rect(buffer, width, height, head_x + 7 + eye_dx, head_y + 15, 6, 3, mouth_color);
    fill_rect(buffer, width, height, head_x + 8 + eye_dx, head_y + 15, 4, 1, pack_color(0.50, 0.30, 0.20));

    let get_armor_colors = |b: BlockId| -> (u32, u32) {
        match b {
            BlockId::LeatherHelmet | BlockId::LeatherChestplate | BlockId::LeatherLeggings | BlockId::LeatherBoots => (
                pack_color(0.55, 0.32, 0.15),
                pack_color(0.68, 0.42, 0.22),
            ),
            BlockId::IronHelmet | BlockId::IronChestplate | BlockId::IronLeggings | BlockId::IronBoots => (
                pack_color(0.80, 0.82, 0.85),
                pack_color(0.95, 0.96, 0.98),
            ),
            BlockId::GoldHelmet | BlockId::GoldChestplate | BlockId::GoldLeggings | BlockId::GoldBoots => (
                pack_color(0.92, 0.78, 0.18),
                pack_color(1.00, 0.95, 0.45),
            ),
            BlockId::DiamondHelmet | BlockId::DiamondChestplate | BlockId::DiamondLeggings | BlockId::DiamondBoots => (
                pack_color(0.25, 0.82, 0.85),
                pack_color(0.65, 0.98, 0.98),
            ),
            _ => (pack_color(0.8, 0.8, 0.8), pack_color(0.95, 0.95, 0.95)),
        }
    };

    if let Some(legs) = armor[2] {
        let (base, hi) = get_armor_colors(legs.block);
        fill_rect(buffer, width, height, center_x - 10, legs_y, 9, 21, base);
        fill_rect(buffer, width, height, center_x - 10, legs_y, 2, 21, hi);
        fill_rect(buffer, width, height, center_x + 1, legs_y, 9, 21, base);
        fill_rect(buffer, width, height, center_x + 1, legs_y, 2, 21, hi);
    }
    if let Some(boots) = armor[3] {
        let (base, hi) = get_armor_colors(boots.block);
        fill_rect(buffer, width, height, center_x - 11, legs_y + 19, 11, 9, base);
        fill_rect(buffer, width, height, center_x - 11, legs_y + 19, 2, 9, hi);
        fill_rect(buffer, width, height, center_x, legs_y + 19, 11, 9, base);
        fill_rect(buffer, width, height, center_x, legs_y + 19, 2, 9, hi);
    }
    if let Some(chest) = armor[1] {
        let (base, hi) = get_armor_colors(chest.block);
        fill_rect(buffer, width, height, torso_x - 1, torso_y - 1, 22, 29, base);
        fill_rect(buffer, width, height, torso_x, torso_y - 1, 2, 29, hi);
        fill_rect(buffer, width, height, left_arm_x - 1, left_arm_y - 1, 9, 10, base);
        fill_rect(buffer, width, height, right_arm_x, right_arm_y - 1, 9, 10, base);
    }
    if let Some(helm) = armor[0] {
        let (base, hi) = get_armor_colors(helm.block);
        fill_rect(buffer, width, height, head_x - 1, head_y - 2, 22, 9, base);
        fill_rect(buffer, width, height, head_x, head_y - 2, 2, 9, hi);
        fill_rect(buffer, width, height, head_x - 1, head_y + 7, 3, 6, base);
        fill_rect(buffer, width, height, head_x + 18, head_y + 7, 3, 6, base);
        fill_rect(buffer, width, height, head_x + 9, head_y + 7, 2, 4, base);
    }
    if let Some(off) = offhand {
        if off.block.is_shield() {
            let sx = right_arm_x + 3;
            let sy = torso_y + 2;
            let rim = pack_color(0.80, 0.82, 0.86);
            let wood = pack_color(0.55, 0.35, 0.15);
            fill_rect(buffer, width, height, sx, sy, 10, 18, wood);
            fill_rect(buffer, width, height, sx - 1, sy - 1, 12, 2, rim);
            fill_rect(buffer, width, height, sx - 1, sy + 17, 12, 2, rim);
            fill_rect(buffer, width, height, sx - 1, sy, 2, 18, rim);
            fill_rect(buffer, width, height, sx + 9, sy, 2, 18, rim);
            fill_rect(buffer, width, height, sx + 4, sy + 7, 2, 4, rim);
        }
    }
}

pub fn draw_armor_and_offhand_slots(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    armor_x: i32,
    armor_y_start: i32,
    offhand_x: i32,
    offhand_y: i32,
    player: &Player,
    hovered_slot: Option<InventorySlotTarget>,
) {
    for i in 0..4 {
        let sy = armor_y_start + i as i32 * 26;
        let is_hovered = hovered_slot == Some(InventorySlotTarget::Armor(i));
        UiCanvas::new(buffer, width, height).inventory_slot(
            Rect::new(armor_x, sy, 24, 24),
            player.inventory.armor[i],
            is_hovered,
            None,
        );
        if player.inventory.armor[i].is_none() {
            let sil = pack_color(0.24, 0.24, 0.28);
            match i {
                0 => {
                    fill_rect(buffer, width, height, armor_x + 7, sy + 6, 10, 5, sil);
                    fill_rect(buffer, width, height, armor_x + 7, sy + 11, 3, 5, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 11, 3, 5, sil);
                }
                1 => {
                    fill_rect(buffer, width, height, armor_x + 5, sy + 5, 14, 12, sil);
                    fill_rect(buffer, width, height, armor_x + 9, sy + 5, 6, 4, pack_color(0.12, 0.12, 0.14));
                }
                2 => {
                    fill_rect(buffer, width, height, armor_x + 6, sy + 5, 12, 4, sil);
                    fill_rect(buffer, width, height, armor_x + 6, sy + 9, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 13, sy + 9, 5, 9, sil);
                }
                _ => {
                    fill_rect(buffer, width, height, armor_x + 5, sy + 8, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 4, sy + 13, 6, 4, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 8, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 13, 6, 4, sil);
                }
            }
        }
    }

    let is_off_hovered = hovered_slot == Some(InventorySlotTarget::Offhand);
    UiCanvas::new(buffer, width, height).inventory_slot(
        Rect::new(offhand_x, offhand_y, 24, 24),
        player.inventory.offhand,
        is_off_hovered,
        None,
    );
    if player.inventory.offhand.is_none() {
        let sil = pack_color(0.24, 0.24, 0.28);
        fill_rect(buffer, width, height, offhand_x + 6, offhand_y + 4, 12, 10, sil);
        fill_rect(buffer, width, height, offhand_x + 8, offhand_y + 14, 8, 4, sil);
    }
}

pub(super) fn draw_player_vitals_panel(ui: &mut UiCanvas<'_>, x: i32, y: i32, bar_w: i32, player: &Player) {
    ui.text("PLAYER", x, y + 10, 1, 1, pack_color(1.0, 0.88, 0.25));

    let hp_text = format!("HP: {}/20", player.health as i32);
    ui.text(&hp_text, x, y + 24, 1, 1, pack_color(1.0, 0.35, 0.35));
    let hp_ratio = (player.health / 20.0).clamp(0.0, 1.0);
    ui.progress_bar(
        Rect::new(x, y + 36, bar_w, 4),
        hp_ratio,
        pack_color(0.25, 0.05, 0.05),
        pack_color(0.9, 0.2, 0.2),
    );

    let hunger_text = format!("HUNGER: {}/20", player.hunger as i32);
    ui.text(&hunger_text, x, y + 46, 1, 1, pack_color(1.0, 0.65, 0.2));
    let hunger_ratio = (player.hunger / player.max_hunger.max(1.0)).clamp(0.0, 1.0);
    ui.progress_bar(
        Rect::new(x, y + 58, bar_w, 4),
        hunger_ratio,
        pack_color(0.25, 0.15, 0.05),
        pack_color(0.9, 0.55, 0.15),
    );

    let def = player.armor_defense_points();
    if def > 0.0 {
        let def_text = format!("ARMOR: {:.0}", def);
        ui.text(&def_text, x, y + 68, 1, 1, pack_color(0.8, 0.85, 0.95));
    } else if player.fire_timer > 0.0 {
        ui.text("BURNING!", x, y + 70, 1, 1, pack_color(1.0, 0.5, 0.1));
    } else {
        ui.text("STATUS: OK", x, y + 70, 1, 1, pack_color(0.4, 0.85, 0.4));
    }
}
