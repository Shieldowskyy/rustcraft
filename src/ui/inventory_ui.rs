use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::{GameMode, Player};
use crate::gameplay::recipe::RecipeBook;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::ui::hud::draw_hotbar;
use crate::world::block::{BlockId, Face};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InventorySlotTarget {
    Hotbar(usize),
    Main(usize),
    Crafting2x2(usize),
    Crafting3x3(usize),
    CraftingOutput,
    Creative(u8),
}

pub struct InventoryLayout {
    pub start_x: i32,
    pub start_y: i32,
    pub crafting_table_open: bool,
}

impl InventoryLayout {
    pub fn new(width: usize, height: usize, crafting_table_open: bool) -> Self {
        let start_x = (width as i32 - 450) / 2;
        let start_y = (height as i32 - 350) / 2;
        Self {
            start_x,
            start_y,
            crafting_table_open,
        }
    }

    pub fn find_slot_at(&self, px: i32, py: i32, gamemode: GameMode) -> Option<InventorySlotTarget> {
        if gamemode == GameMode::Creative {
            let mut block_id = 1;
            for row in 0..5 {
                for col in 0..8 {
                    if block_id > 33 {
                        return None;
                    }
                    let x = self.start_x + 35 + col as i32 * 46;
                    let y = self.start_y + 40 + row as i32 * 46;
                    if px >= x && px < x + 40 && py >= y && py < y + 40 {
                        return Some(InventorySlotTarget::Creative(block_id));
                    }
                    block_id += 1;
                }
            }
            return None;
        }

        // Hotbar: 10 slots
        let hotbar_x = self.start_x + 15;
        let hotbar_y = self.start_y + 295;
        for i in 0..10 {
            let x = hotbar_x + i as i32 * 42;
            let y = hotbar_y;
            if px >= x && px < x + 38 && py >= y && py < y + 38 {
                return Some(InventorySlotTarget::Hotbar(i));
            }
        }

        // Main inventory: 3 rows x 9 cols (27 slots)
        let main_x = self.start_x + 36;
        let main_y = self.start_y + 165;
        for row in 0..3 {
            for col in 0..9 {
                let x = main_x + col as i32 * 42;
                let y = main_y + row as i32 * 42;
                if px >= x && px < x + 38 && py >= y && py < y + 38 {
                    return Some(InventorySlotTarget::Main(row * 9 + col));
                }
            }
        }

        if self.crafting_table_open {
            // 3x3 crafting grid
            let craft_x = self.start_x + 40;
            let craft_y = self.start_y + 35;
            for row in 0..3 {
                for col in 0..3 {
                    let x = craft_x + col as i32 * 40;
                    let y = craft_y + row as i32 * 40;
                    if px >= x && px < x + 36 && py >= y && py < y + 36 {
                        return Some(InventorySlotTarget::Crafting3x3(row * 3 + col));
                    }
                }
            }
            // 3x3 Output slot
            let out_x = self.start_x + 230;
            let out_y = self.start_y + 75;
            if px >= out_x && px < out_x + 42 && py >= out_y && py < out_y + 42 {
                return Some(InventorySlotTarget::CraftingOutput);
            }
        } else {
            // 2x2 crafting grid
            let craft_x = self.start_x + 220;
            let craft_y = self.start_y + 50;
            for row in 0..2 {
                for col in 0..2 {
                    let x = craft_x + col as i32 * 40;
                    let y = craft_y + row as i32 * 40;
                    if px >= x && px < x + 36 && py >= y && py < y + 36 {
                        return Some(InventorySlotTarget::Crafting2x2(row * 2 + col));
                    }
                }
            }
            // 2x2 Output slot
            let out_x = self.start_x + 350;
            let out_y = self.start_y + 70;
            if px >= out_x && px < out_x + 42 && py >= out_y && py < out_y + 42 {
                return Some(InventorySlotTarget::CraftingOutput);
            }
        }

        None
    }
}

pub fn draw_inventory_screen(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    player: &Player,
    hovered_slot: Option<InventorySlotTarget>,
    crafting_table_open: bool,
    recipes: &RecipeBook,
    mouse_pos: Option<(f32, f32)>,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.55);

    let layout = InventoryLayout::new(width, height, crafting_table_open);
    let start_x = layout.start_x;
    let start_y = layout.start_y;

    // Outer window frame
    fill_rect_alpha(buffer, width, height, start_x - 10, start_y - 10, 470, 365, pack_color(0.12, 0.12, 0.14), 0.95);
    fill_rect(buffer, width, height, start_x - 10, start_y - 10, 470, 3, pack_color(0.35, 0.35, 0.38));
    fill_rect(buffer, width, height, start_x - 10, start_y + 352, 470, 3, pack_color(0.05, 0.05, 0.06));

    if player.gamemode == GameMode::Creative {
        draw_text_shadowed(buffer, width, height, "EKWIPUNEK KREATYWNY", start_x + 10, start_y + 10, 2, 1);
        draw_text_shadowed(buffer, width, height, "[A]/LPM Wybierz caly stack   [B]/[Y] Zamknij", start_x + 10, start_y + 335, 1, 1);

        let mut block_id = 1;
        for row in 0..5 {
            for col in 0..8 {
                if block_id > 33 {
                    break;
                }
                let x = start_x + 35 + col as i32 * 46;
                let y = start_y + 40 + row as i32 * 46;

                let is_hovered = hovered_slot == Some(InventorySlotTarget::Creative(block_id));
                let border_col = if is_hovered { pack_color(1.0, 0.9, 0.2) } else { pack_color(0.25, 0.25, 0.28) };

                fill_rect(buffer, width, height, x, y, 40, 40, border_col);
                fill_rect(buffer, width, height, x + 2, y + 2, 36, 36, pack_color(0.18, 0.18, 0.20));

                let block = BlockId::from_u8(block_id);
                draw_item_icon(buffer, width, height, block, x + 6, y + 6, 24);

                block_id += 1;
            }
        }
    } else {
        // Survival Mode
        let title = if crafting_table_open { "STOL RZEMIESLNICZY" } else { "EKWIPUNEK" };
        draw_text_shadowed(buffer, width, height, title, start_x + 10, start_y + 8, 2, 1);
        draw_text_shadowed(buffer, width, height, "LPM: Podnies/Upusc   PPM: 1 szt / Polowa   [B]/[Y]: Zamknij", start_x + 10, start_y + 342, 1, 1);

        // Crafting section
        if crafting_table_open {
            draw_text_colored_shadowed(buffer, width, height, "Siatka 3x3", start_x + 40, start_y + 20, 1, 1, pack_color(0.8, 0.8, 0.8));
            // 3x3 input slots
            let craft_x = start_x + 40;
            let craft_y = start_y + 35;
            for row in 0..3 {
                for col in 0..3 {
                    let idx = row * 3 + col;
                    let x = craft_x + col as i32 * 40;
                    let y = craft_y + row as i32 * 40;
                    let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting3x3(idx));
                    draw_slot(buffer, width, height, x, y, 36, 36, player.inventory.crafting_3x3[idx], is_hovered);
                }
            }

            // Arrow
            draw_text_colored_shadowed(buffer, width, height, "==>", start_x + 175, start_y + 85, 2, 1, pack_color(0.9, 0.85, 0.3));

            // 3x3 Output slot
            let out_x = start_x + 230;
            let out_y = start_y + 75;
            let result_item = recipes.match_grid(&player.inventory.crafting_3x3, 3, 3);
            let is_out_hovered = hovered_slot == Some(InventorySlotTarget::CraftingOutput);
            draw_slot(buffer, width, height, out_x, out_y, 42, 42, result_item, is_out_hovered);
        } else {
            // Player character info area
            fill_rect_alpha(buffer, width, height, start_x + 30, start_y + 40, 130, 100, pack_color(0.08, 0.08, 0.10), 0.8);
            draw_text_shadowed(buffer, width, height, "POSTAC", start_x + 70, start_y + 50, 1, 1);
            let hp_text = format!("HP: {}/20", player.health as i32);
            draw_text_colored_shadowed(buffer, width, height, &hp_text, start_x + 45, start_y + 80, 1, 1, pack_color(1.0, 0.3, 0.3));

            // 2x2 Crafting
            draw_text_colored_shadowed(buffer, width, height, "Tworzenie 2x2", start_x + 220, start_y + 35, 1, 1, pack_color(0.85, 0.85, 0.85));
            let craft_x = start_x + 220;
            let craft_y = start_y + 50;
            for row in 0..2 {
                for col in 0..2 {
                    let idx = row * 2 + col;
                    let x = craft_x + col as i32 * 40;
                    let y = craft_y + row as i32 * 40;
                    let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting2x2(idx));
                    draw_slot(buffer, width, height, x, y, 36, 36, player.inventory.crafting_2x2[idx], is_hovered);
                }
            }

            // Arrow
            draw_text_colored_shadowed(buffer, width, height, "==>", start_x + 310, start_y + 80, 2, 1, pack_color(0.9, 0.85, 0.3));

            // Output slot
            let out_x = start_x + 350;
            let out_y = start_y + 70;
            let result_item = recipes.match_grid(&player.inventory.crafting_2x2, 2, 2);
            let is_out_hovered = hovered_slot == Some(InventorySlotTarget::CraftingOutput);
            draw_slot(buffer, width, height, out_x, out_y, 42, 42, result_item, is_out_hovered);
        }

        // Main Inventory
        draw_text_colored_shadowed(buffer, width, height, "Plecak", start_x + 36, start_y + 150, 1, 1, pack_color(0.75, 0.75, 0.75));
        let main_x = start_x + 36;
        let main_y = start_y + 165;
        for row in 0..3 {
            for col in 0..9 {
                let idx = row * 9 + col;
                let x = main_x + col as i32 * 42;
                let y = main_y + row as i32 * 42;
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Main(idx));
                draw_slot(buffer, width, height, x, y, 38, 38, player.inventory.main[idx], is_hovered);
            }
        }

        // Hotbar
        draw_text_colored_shadowed(buffer, width, height, "Pasek podreczny", start_x + 15, start_y + 280, 1, 1, pack_color(0.75, 0.75, 0.75));
        let hotbar_x = start_x + 15;
        let hotbar_y = start_y + 295;
        for i in 0..10 {
            let x = hotbar_x + i as i32 * 42;
            let y = hotbar_y;
            let is_hovered = hovered_slot == Some(InventorySlotTarget::Hotbar(i));
            let is_selected = player.selected_hotbar_slot as usize == i;
            let border_override = if is_selected && !is_hovered {
                Some(pack_color(0.4, 0.7, 1.0))
            } else {
                None
            };
            draw_slot_custom(buffer, width, height, x, y, 38, 38, player.inventory.hotbar[i], is_hovered, border_override);
        }
    }

    // Hovered Item Name Tooltip
    let mut hovered_item_name = None;
    if let Some(target) = hovered_slot {
        match target {
            InventorySlotTarget::Hotbar(i) => hovered_item_name = player.inventory.hotbar[i].map(|s| (s.block.name(), s.count)),
            InventorySlotTarget::Main(i) => hovered_item_name = player.inventory.main[i].map(|s| (s.block.name(), s.count)),
            InventorySlotTarget::Crafting2x2(i) => hovered_item_name = player.inventory.crafting_2x2[i].map(|s| (s.block.name(), s.count)),
            InventorySlotTarget::Crafting3x3(i) => hovered_item_name = player.inventory.crafting_3x3[i].map(|s| (s.block.name(), s.count)),
            InventorySlotTarget::CraftingOutput => {
                let res = if crafting_table_open {
                    recipes.match_grid(&player.inventory.crafting_3x3, 3, 3)
                } else {
                    recipes.match_grid(&player.inventory.crafting_2x2, 2, 2)
                };
                hovered_item_name = res.map(|s| (s.block.name(), s.count));
            }
            InventorySlotTarget::Creative(id) => {
                let block = BlockId::from_u8(id);
                hovered_item_name = Some((block.name(), 64));
            }
        }
    }

    if let Some((name, count)) = hovered_item_name {
        let text = if count > 1 { format!("{} (x{})", name.to_uppercase(), count) } else { name.to_uppercase() };
        if let Some((mx, my)) = mouse_pos {
            let tx = (mx as i32 + 15).min(width as i32 - 180);
            let ty = (my as i32 - 20).max(10);
            fill_rect_alpha(buffer, width, height, tx - 4, ty - 4, (text.len() * 8 + 12) as i32, 20, pack_color(0.05, 0.05, 0.08), 0.9);
            draw_text_colored_shadowed(buffer, width, height, &text, tx, ty, 1, 1, pack_color(1.0, 0.95, 0.4));
        }
    }

    // Draw Held Item on Mouse Cursor
    if let Some(held) = player.inventory.held_item {
        if let Some((mx, my)) = mouse_pos {
            let hx = mx as i32 - 14;
            let hy = my as i32 - 14;
            draw_item_icon(buffer, width, height, held.block, hx, hy, 28);
            if held.count > 1 {
                let count_str = format!("{}", held.count);
                draw_text_colored_shadowed(buffer, width, height, &count_str, hx + 14, hy + 14, 1, 1, pack_color(1.0, 1.0, 1.0));
            }
        }
    }

    draw_hotbar(buffer, width, height, player);
}

fn draw_slot(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    item: Option<ItemStack>,
    is_hovered: bool,
) {
    draw_slot_custom(buffer, width, height, x, y, w, h, item, is_hovered, None);
}

fn draw_slot_custom(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    item: Option<ItemStack>,
    is_hovered: bool,
    border_override: Option<u32>,
) {
    let border_color = if is_hovered {
        pack_color(1.0, 0.9, 0.2)
    } else if let Some(b) = border_override {
        b
    } else {
        pack_color(0.24, 0.24, 0.28)
    };

    fill_rect(buffer, width, height, x, y, w, h, border_color);
    fill_rect(buffer, width, height, x + 2, y + 2, w - 4, h - 4, pack_color(0.12, 0.12, 0.14));

    if let Some(stack) = item {
        let icon_size = (w - 12).max(16);
        let ix = x + (w - icon_size) / 2;
        let iy = y + (h - icon_size) / 2;
        draw_item_icon(buffer, width, height, stack.block, ix, iy, icon_size);

        if stack.count > 1 {
            let count_str = format!("{}", stack.count);
            let tx = x + w - (count_str.len() as i32 * 8) - 4;
            let ty = y + h - 11;
            draw_text_colored_shadowed(buffer, width, height, &count_str, tx, ty, 1, 1, pack_color(1.0, 1.0, 1.0));
        }
    }
}

pub fn draw_item_icon(buffer: &mut [u32], buf_w: usize, buf_h: usize, block: BlockId, x: i32, y: i32, size: i32) {
    let scale = (size / 16).max(1);
    let ox = x + (size - 16 * scale) / 2;
    let oy = y + (size - 16 * scale) / 2;

    let px = |buf: &mut [u32], lx: i32, ly: i32, col: u32| {
        if lx >= 0 && lx < 16 && ly >= 0 && ly < 16 {
            fill_rect(buf, buf_w, buf_h, ox + lx * scale, oy + ly * scale, scale, scale, col);
        }
    };

    let wood_dark = pack_color(0.30, 0.18, 0.08);
    let wood_mid = pack_color(0.55, 0.35, 0.15);
    let wood_light = pack_color(0.72, 0.52, 0.28);

    let get_mat_colors = |b: BlockId| -> (u32, u32, u32, u32) {
        match b {
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword => (
                pack_color(0.85, 0.68, 0.42), // highlight
                pack_color(0.68, 0.50, 0.30), // base
                pack_color(0.48, 0.32, 0.16), // shadow
                pack_color(0.32, 0.20, 0.10), // outline
            ),
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword => (
                pack_color(0.78, 0.78, 0.80), // highlight
                pack_color(0.58, 0.58, 0.60), // base
                pack_color(0.40, 0.40, 0.42), // shadow
                pack_color(0.24, 0.24, 0.26), // outline
            ),
            _ => (
                // Iron
                pack_color(1.00, 1.00, 1.00), // glint
                pack_color(0.90, 0.93, 0.98), // highlight
                pack_color(0.72, 0.76, 0.84), // base
                pack_color(0.42, 0.46, 0.54), // shadow
            ),
        }
    };

    match block {
        BlockId::Stick => {
            for i in 2..=13 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
                px(buffer, i, 16 - i, wood_dark);
            }
        }
        BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            // Pommel
            px(buffer, 2, 13, wood_dark);
            px(buffer, 1, 14, wood_mid);
            // Grip
            px(buffer, 3, 12, wood_mid);
            px(buffer, 4, 11, wood_mid);
            px(buffer, 2, 12, wood_dark);
            px(buffer, 3, 11, wood_light);
            // Guard
            px(buffer, 3, 10, wood_dark);
            px(buffer, 4, 10, c_base);
            px(buffer, 5, 10, c_hi);
            px(buffer, 5, 11, c_base);
            px(buffer, 5, 12, wood_dark);
            px(buffer, 6, 9, c_hi);
            px(buffer, 2, 9, wood_dark);
            // Blade
            for i in 5..=11 {
                px(buffer, i, 15 - i, c_base);
                px(buffer, i - 1, 15 - i, c_hi);
                px(buffer, i + 1, 15 - i, c_shadow);
            }
            // Tip
            px(buffer, 12, 3, c_hi);
            px(buffer, 13, 2, c_glint);
            px(buffer, 12, 2, c_glint);
            px(buffer, 13, 3, c_shadow);
        }
        BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            // Handle
            for i in 2..=10 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
                px(buffer, i, 16 - i, wood_dark);
            }
            // Center binding
            px(buffer, 9, 6, wood_dark);
            px(buffer, 10, 5, wood_dark);

            // Pickaxe Head
            // Left hook
            px(buffer, 4, 7, c_shadow);
            px(buffer, 4, 6, c_base);
            px(buffer, 5, 5, c_base);
            px(buffer, 5, 4, c_hi);
            px(buffer, 6, 4, c_base);
            px(buffer, 7, 3, c_hi);
            px(buffer, 8, 3, c_base);
            // Peak / Top
            px(buffer, 9, 2, c_hi);
            px(buffer, 10, 2, c_hi);
            px(buffer, 11, 2, c_glint);
            px(buffer, 12, 2, c_glint);
            px(buffer, 13, 2, c_glint);
            px(buffer, 13, 3, c_hi);
            // Right hook
            px(buffer, 12, 4, c_base);
            px(buffer, 12, 5, c_shadow);
            px(buffer, 11, 5, c_base);
            px(buffer, 11, 6, c_shadow);
            px(buffer, 10, 7, c_shadow);
            px(buffer, 9, 8, c_shadow);
        }
        BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            // Handle
            for i in 2..=11 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
                px(buffer, i, 16 - i, wood_dark);
            }
            // Axe Head Block
            px(buffer, 9, 2, c_glint);
            px(buffer, 10, 2, c_glint);
            px(buffer, 11, 2, c_hi);
            px(buffer, 12, 2, c_base);
            px(buffer, 8, 3, c_glint);
            px(buffer, 9, 3, c_hi);
            px(buffer, 10, 3, c_base);
            px(buffer, 11, 3, c_shadow);
            px(buffer, 12, 3, c_shadow);
            px(buffer, 8, 4, c_hi);
            px(buffer, 9, 4, c_base);
            px(buffer, 10, 4, c_base);
            px(buffer, 8, 5, c_base);
            px(buffer, 9, 5, c_shadow);
            px(buffer, 7, 5, c_hi);
            px(buffer, 7, 6, c_base);
            px(buffer, 8, 6, c_shadow);
        }
        BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            // Grip topper
            px(buffer, 1, 13, wood_light);
            px(buffer, 2, 14, wood_dark);
            // Handle
            for i in 3..=9 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
                px(buffer, i, 16 - i, wood_dark);
            }
            // Spade blade
            px(buffer, 10, 5, c_shadow);
            px(buffer, 11, 4, c_base);
            px(buffer, 12, 3, c_hi);
            px(buffer, 13, 2, c_glint);
            px(buffer, 12, 2, c_glint);
            px(buffer, 13, 3, c_hi);
            px(buffer, 11, 3, c_hi);
            px(buffer, 10, 4, c_base);
            px(buffer, 9, 5, c_base);
            px(buffer, 12, 4, c_shadow);
            px(buffer, 11, 5, c_shadow);
            px(buffer, 10, 6, c_shadow);
        }
        BlockId::CraftingTable => {
            // Isometric Crafting Table Icon
            let top_light = pack_color(0.78, 0.62, 0.38);
            let top_dark = pack_color(0.58, 0.42, 0.22);
            let side_frame = pack_color(0.32, 0.20, 0.10);
            let side_plank = pack_color(0.64, 0.48, 0.28);
            let iron_tool = pack_color(0.78, 0.82, 0.88);

            // Top grid
            for ly in 1..=6 {
                for lx in 2..=13 {
                    let in_top = (lx + ly >= 6) && (lx - ly <= 8) && (ly - lx <= 3) && (lx + ly <= 16);
                    if in_top {
                        let is_check = ((lx / 2) + (ly / 2)) % 2 == 0;
                        px(buffer, lx, ly, if is_check { top_light } else { top_dark });
                    }
                }
            }
            // Side face Left
            for ly in 6..=14 {
                for lx in 2..=7 {
                    let is_border = lx == 2 || lx == 7 || ly == 14;
                    px(buffer, lx, ly, if is_border { side_frame } else { side_plank });
                }
            }
            // Side face Right
            for ly in 6..=14 {
                for lx in 8..=13 {
                    let is_border = lx == 8 || lx == 13 || ly == 14;
                    px(buffer, lx, ly, if is_border { side_frame } else { side_plank });
                }
            }
            // Mini tool icons on side
            px(buffer, 4, 8, iron_tool);
            px(buffer, 4, 9, iron_tool);
            px(buffer, 5, 9, iron_tool);
            px(buffer, 10, 8, iron_tool);
            px(buffer, 11, 8, iron_tool);
            px(buffer, 10, 9, wood_dark);
        }
        BlockId::Door => {
            let frame = pack_color(0.52, 0.36, 0.18);
            let glass = pack_color(0.65, 0.82, 0.90);
            let knob = pack_color(0.85, 0.75, 0.30);
            let panel = pack_color(0.42, 0.28, 0.14);

            for ly in 1..=14 {
                for lx in 4..=11 {
                    px(buffer, lx, ly, frame);
                }
            }
            // Glass windows
            for ly in 3..=6 {
                px(buffer, 5, ly, glass);
                px(buffer, 6, ly, glass);
                px(buffer, 9, ly, glass);
                px(buffer, 10, ly, glass);
            }
            // Knob
            px(buffer, 10, 8, knob);
            // Lower panels
            for ly in 9..=12 {
                px(buffer, 5, ly, panel);
                px(buffer, 6, ly, panel);
                px(buffer, 9, ly, panel);
                px(buffer, 10, ly, panel);
            }
        }
        BlockId::Tnt => {
            let red = pack_color(0.85, 0.15, 0.10);
            let white = pack_color(0.92, 0.92, 0.92);
            let dark = pack_color(0.25, 0.05, 0.05);
            let fuse = pack_color(0.40, 0.35, 0.30);

            for ly in 3..=13 {
                for lx in 3..=12 {
                    let is_band = ly >= 7 && ly <= 9;
                    px(buffer, lx, ly, if is_band { white } else { red });
                }
            }
            // "TNT" lettering on band
            px(buffer, 5, 8, dark);
            px(buffer, 7, 8, dark);
            px(buffer, 8, 8, dark);
            px(buffer, 10, 8, dark);
            // Fuse on top
            px(buffer, 7, 1, fuse);
            px(buffer, 7, 2, fuse);
        }
        _ => {
            // High-detail 3D beveled block
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            fill_rect(buffer, buf_w, buf_h, x, y, size, size, packed);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);

            fill_rect(buffer, buf_w, buf_h, x, y, size, 2, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x, y, 2, size, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x, y + size - 2, size, 2, shadow);
            fill_rect(buffer, buf_w, buf_h, x + size - 2, y, 2, size, shadow);
            fill_rect(buffer, buf_w, buf_h, x, y + size - 1, size, 1, dark_border);
            fill_rect(buffer, buf_w, buf_h, x + size - 1, y, 1, size, dark_border);
        }
    }
}
