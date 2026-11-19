//! Renderowanie ekranu ekwipunku w trybach Creative i Survival, księgi receptur oraz podpowiedzi.

use crate::features::blocks::BlockId;
use crate::features::player::{GameMode, ItemStack, Player, RecipeBook};
use crate::features::ui::draw::{draw_cursor_pointer, fill_rect, pack_color};
use crate::features::ui::widgets::{Grid, Rect, UiCanvas};

use super::icons::draw_item_icon;
use super::layout::{creative_tab_blocks, InventoryLayout, InventorySlotTarget, CREATIVE_TABS};
use super::preview::{draw_armor_and_offhand_slots, draw_player_inventory_preview, draw_player_vitals_panel};

pub fn draw_trash_icon(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32) {
    let red_hi = pack_color(1.0, 0.40, 0.40);
    let red_body = pack_color(0.85, 0.20, 0.20);
    let red_dark = pack_color(0.50, 0.10, 0.10);

    fill_rect(buffer, buf_w, buf_h, x + 10, y + 4, 4, 2, red_hi);
    fill_rect(buffer, buf_w, buf_h, x + 6, y + 6, 12, 2, red_hi);
    fill_rect(buffer, buf_w, buf_h, x + 7, y + 9, 10, 12, red_body);
    fill_rect(buffer, buf_w, buf_h, x + 9, y + 10, 1, 10, red_dark);
    fill_rect(buffer, buf_w, buf_h, x + 11, y + 10, 1, 10, red_dark);
    fill_rect(buffer, buf_w, buf_h, x + 13, y + 10, 1, 10, red_dark);
}

fn draw_trash_slot(ui: &mut UiCanvas<'_>, rect: Rect, is_hovered: bool) {
    let trash_border = if is_hovered {
        pack_color(1.0, 0.25, 0.25)
    } else {
        pack_color(0.45, 0.18, 0.18)
    };
    ui.bordered_box(rect, 2, pack_color(0.16, 0.08, 0.08), trash_border);
    draw_trash_icon(ui.buffer, ui.width, ui.height, rect.x + 6, rect.y + 7);
}

fn draw_backpack_slots(
    ui: &mut UiCanvas<'_>,
    main_grid: Grid,
    player: &Player,
    hovered_slot: Option<InventorySlotTarget>,
) {
    for idx in 0..27 {
        let is_hovered = hovered_slot == Some(InventorySlotTarget::Main(idx));
        ui.inventory_slot(main_grid.slot(idx), player.inventory.main[idx], is_hovered, None);
    }
}

fn draw_hotbar_slots(
    ui: &mut UiCanvas<'_>,
    hotbar: Grid,
    player: &Player,
    hovered_slot: Option<InventorySlotTarget>,
    selected_border_color: u32,
) {
    for i in 0..10 {
        let is_hovered = hovered_slot == Some(InventorySlotTarget::Hotbar(i));
        let is_selected = player.selected_hotbar_slot as usize == i;
        let border_override = if is_selected && !is_hovered {
            Some(selected_border_color)
        } else {
            None
        };
        ui.inventory_slot(hotbar.slot(i), player.inventory.hotbar[i], is_hovered, border_override);
    }
}

fn draw_crafting_2x2_slots(
    ui: &mut UiCanvas<'_>,
    craft_2x2: Grid,
    arrow_x: i32,
    arrow_y: i32,
    output_rect: Rect,
    player: &Player,
    recipes: &RecipeBook,
    hovered_slot: Option<InventorySlotTarget>,
) {
    for idx in 0..4 {
        let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting2x2(idx));
        ui.inventory_slot(craft_2x2.slot(idx), player.inventory.crafting_2x2[idx], is_hovered, None);
    }
    ui.text("==>", arrow_x, arrow_y, 2, 1, pack_color(0.9, 0.85, 0.3));
    let result_item = recipes.match_grid(&player.inventory.crafting_2x2, 2, 2);
    ui.inventory_slot(
        output_rect,
        result_item,
        hovered_slot == Some(InventorySlotTarget::CraftingOutput),
        None,
    );
}

fn draw_creative_screen(
    ui: &mut UiCanvas<'_>,
    layout: &InventoryLayout,
    player: &Player,
    hovered_slot: Option<InventorySlotTarget>,
    recipes: &RecipeBook,
    mouse_pos: Option<(f32, f32)>,
    is_gamepad_active: bool,
) {
    let start_x = layout.start_x;
    let start_y = layout.start_y;
    let tabs_grid = layout.creative_tabs_grid();
    for (i, &label) in CREATIVE_TABS.iter().enumerate() {
        let tab_rect = tabs_grid.slot(i);
        let is_active = player.creative_tab == i;
        let is_hovered = hovered_slot == Some(InventorySlotTarget::CreativeTab(i));

        let bg_color = if is_active {
            pack_color(0.20, 0.20, 0.24)
        } else if is_hovered {
            pack_color(0.16, 0.16, 0.19)
        } else {
            pack_color(0.10, 0.10, 0.12)
        };

        let border_top = if is_active {
            pack_color(1.0, 0.88, 0.2)
        } else if is_hovered {
            pack_color(0.65, 0.65, 0.70)
        } else {
            pack_color(0.30, 0.30, 0.34)
        };

        ui.fill(tab_rect, bg_color);
        ui.fill(Rect::new(tab_rect.x, tab_rect.y, tab_rect.w, 2), border_top);
        ui.fill(Rect::new(tab_rect.x, tab_rect.y, 2, tab_rect.h), border_top);
        ui.fill(Rect::new(tab_rect.right() - 2, tab_rect.y, 2, tab_rect.h), pack_color(0.06, 0.06, 0.08));

        let text_color = if is_active {
            pack_color(1.0, 0.95, 0.3)
        } else if is_hovered {
            pack_color(0.9, 0.9, 0.9)
        } else {
            pack_color(0.55, 0.55, 0.60)
        };
        ui.text_centered_x(label, tab_rect.center_x(), tab_rect.y + 8, 1, 1, text_color);
    }

    let win_rect = Rect::new(start_x - 8, start_y, 480, 360);
    ui.fill_alpha(win_rect, pack_color(0.14, 0.14, 0.16), 0.98);
    ui.bevel_border(win_rect, 3, pack_color(0.38, 0.38, 0.42), pack_color(0.05, 0.05, 0.06));

    if player.creative_tab < CREATIVE_TABS.len() - 1 {
        let title = match player.creative_tab {
            0 => "BUILDING BLOCKS",
            1 => "TOOLS & COMBAT",
            2 => "RUSTSTONE & MECHANISMS",
            3 => "FARMING & CROPS",
            4 => "ALL ITEMS & BLOCKS",
            _ => "",
        };
        ui.text_white(title, start_x + 18, start_y + 14, 2, 1);

        let blocks = creative_tab_blocks(player.creative_tab);
        let total_rows = (blocks.len() + 8) / 9;
        let max_scroll = total_rows.saturating_sub(4);
        let scroll_row = player.creative_scroll.min(max_scroll);
        let scroll_offset = scroll_row * 9;

        let items_grid = layout.creative_items_grid();
        for cell_idx in 0..36 {
            let idx = scroll_offset + cell_idx;
            let cell = items_grid.slot(cell_idx);
            if idx < blocks.len() {
                let block = blocks[idx];
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Creative(block.to_u8()));
                ui.inventory_slot(cell, Some(ItemStack::new(block, 64)), is_hovered, None);
            } else {
                ui.inventory_slot(cell, None, false, Some(pack_color(0.18, 0.18, 0.20)));
            }
        }

        let sb_x = start_x + 404;
        let is_up_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollUp);
        let is_down_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollDown);

        let up_bg = if is_up_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
        ui.fill(Rect::new(sb_x, start_y + 36, 18, 16), up_bg);
        ui.fill(Rect::new(sb_x, start_y + 36, 18, 1), pack_color(0.4, 0.4, 0.45));
        ui.text("^", sb_x + 6, start_y + 40, 1, 1, pack_color(0.9, 0.9, 0.95));

        let track_y = start_y + 54;
        let track_h = 130;
        ui.fill(Rect::new(sb_x, track_y, 18, track_h), pack_color(0.10, 0.10, 0.12));
        ui.fill(Rect::new(sb_x, track_y, 1, track_h), pack_color(0.06, 0.06, 0.08));
        ui.fill(Rect::new(sb_x + 17, track_y, 1, track_h), pack_color(0.22, 0.22, 0.26));

        let thumb_h = if total_rows > 4 {
            ((track_h as f32 * 4.0 / total_rows as f32) as i32).clamp(24, track_h)
        } else {
            track_h
        };
        let thumb_y = if max_scroll > 0 {
            track_y + (scroll_row as f32 / max_scroll as f32 * (track_h - thumb_h) as f32) as i32
        } else {
            track_y
        };
        let thumb_rect = Rect::new(sb_x + 2, thumb_y, 14, thumb_h);
        ui.fill(thumb_rect, pack_color(0.38, 0.38, 0.44));
        ui.bevel_border(thumb_rect, 1, pack_color(0.55, 0.55, 0.62), pack_color(0.20, 0.20, 0.24));

        let down_bg = if is_down_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
        ui.fill(Rect::new(sb_x, start_y + 186, 18, 16), down_bg);
        ui.fill(Rect::new(sb_x, start_y + 186, 18, 1), pack_color(0.4, 0.4, 0.45));
        ui.text("v", sb_x + 6, start_y + 190, 1, 1, pack_color(0.9, 0.9, 0.95));

        if total_rows > 4 {
            let row_info = format!("{}/{}", scroll_row + 1, max_scroll + 1);
            ui.text_right(&row_info, sb_x + 18, start_y + 206, 1, 1, pack_color(0.60, 0.60, 0.65));
        }

        ui.fill(Rect::new(start_x + 16, start_y + 218, 432, 2), pack_color(0.25, 0.25, 0.28));
        ui.text("QUICK ACCESS HOTBAR", start_x + 16, start_y + 226, 1, 1, pack_color(0.85, 0.85, 0.4));
        ui.text("TRASH", start_x + 424, start_y + 226, 1, 1, pack_color(1.0, 0.4, 0.4));

        draw_hotbar_slots(ui, layout.hotbar_grid(242), player, hovered_slot, pack_color(0.4, 0.75, 1.0));
        draw_trash_slot(ui, layout.trash_rect(242), hovered_slot == Some(InventorySlotTarget::Trash));

        let creative_footer = if is_gamepad_active {
            "[A] TAKE (64) | [Y] TO HOTBAR | [X] DELETE | [B] CLOSE"
        } else {
            "LMB: TAKE (64) | SHIFT+LMB: TO HOTBAR | 1-0: SELECT SLOT | [E]/[ESC]: CLOSE"
        };
        ui.text(creative_footer, start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75));
    } else {
        ui.text_white("SURVIVAL INVENTORY", start_x + 18, start_y + 14, 2, 1);

        let char_box = Rect::new(start_x + 18, start_y + 34, 180, 104);
        ui.inset_panel(char_box, pack_color(0.08, 0.08, 0.10), 0.95);
        ui.fill_alpha(Rect::new(char_box.x + 12, char_box.y + 86, 64, 12), pack_color(0.14, 0.14, 0.18), 0.7);
        draw_player_inventory_preview(ui.buffer, ui.width, ui.height, char_box.x + 52, char_box.y + 54, mouse_pos, &player.inventory.armor, &player.inventory.offhand);
        draw_armor_and_offhand_slots(ui.buffer, ui.width, ui.height, start_x + 22, start_y + 36, start_x + 82, start_y + 114, player, hovered_slot);

        ui.text("PLAYER", char_box.x + 110, char_box.y + 16, 1, 1, pack_color(1.0, 0.88, 0.25));
        ui.text("CREATIVE", char_box.x + 110, char_box.y + 38, 1, 1, pack_color(0.4, 0.9, 0.4));
        ui.text("FLYING: ON", char_box.x + 110, char_box.y + 58, 1, 1, pack_color(0.4, 0.8, 1.0));
        ui.text("GODMODE", char_box.x + 110, char_box.y + 78, 1, 1, pack_color(0.9, 0.7, 0.3));

        ui.inset_panel(Rect::new(start_x + 208, start_y + 34, 242, 104), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("Crafting 2x2", start_x + 220, start_y + 26, 1, 1, pack_color(0.85, 0.85, 0.85));
        draw_crafting_2x2_slots(
            ui,
            layout.crafting_2x2_grid(42),
            start_x + 310,
            start_y + 70,
            Rect::new(start_x + 350, start_y + 60, 42, 42),
            player,
            recipes,
            hovered_slot,
        );

        ui.inset_panel(Rect::new(start_x + 14, start_y + 140, 436, 126), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("Backpack", start_x + 22, start_y + 132, 1, 1, pack_color(0.75, 0.75, 0.75));
        draw_backpack_slots(ui, layout.main_grid(144), player, hovered_slot);

        ui.inset_panel(Rect::new(start_x + 10, start_y + 270, 444, 52), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("Hotbar", start_x + 16, start_y + 264, 1, 1, pack_color(0.85, 0.85, 0.4));
        ui.text("TRASH", start_x + 424, start_y + 264, 1, 1, pack_color(1.0, 0.4, 0.4));
        draw_hotbar_slots(ui, layout.hotbar_grid(276), player, hovered_slot, pack_color(0.4, 0.75, 1.0));
        draw_trash_slot(ui, layout.trash_rect(276), hovered_slot == Some(InventorySlotTarget::Trash));

        let footer = if is_gamepad_active {
            "[A]: ALL | [X]: 1 ITEM/HALF | [Y]: MOVE | [B]: CLOSE"
        } else {
            "LPM: ALL | PPM: 1 ITEM/HALF | SHIFT+LPM: MOVE | [E]/[ESC]: CLOSE"
        };
        ui.text(footer, start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75));
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
    unlocked_recipe_indices: &[usize],
    recipe_book_open: bool,
    recipe_book_scroll: usize,
    recipe_book_status: Option<&str>,
    mouse_pos: Option<(f32, f32)>,
    is_gamepad_active: bool,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.0, 0.0, 0.0), 0.55);

    let layout = InventoryLayout::new(width, height, crafting_table_open);
    let start_x = layout.start_x;
    let start_y = layout.start_y;

    if player.gamemode == GameMode::Creative {
        draw_creative_screen(&mut ui, &layout, player, hovered_slot, recipes, mouse_pos, is_gamepad_active);
    } else {
        let win_rect = Rect::new(start_x - 8, start_y, 480, 360);
        let (win_x, win_y, win_w, win_h) = (win_rect.x, win_rect.y, win_rect.w, win_rect.h);

        ui.fill_alpha(win_rect, pack_color(0.15, 0.15, 0.18), 0.98);
        ui.bevel_border(win_rect, 3, pack_color(0.38, 0.38, 0.44), pack_color(0.05, 0.05, 0.06));
        ui.fill(Rect::new(win_x + 3, win_y + 3, win_w - 6, 1), pack_color(0.48, 0.48, 0.54));

        ui.fill(Rect::new(win_x + 3, win_y + 3, win_w - 6, 25), pack_color(0.11, 0.11, 0.14));
        ui.fill(Rect::new(win_x + 3, win_y + 27, win_w - 6, 1), pack_color(0.07, 0.07, 0.09));

        let title = if crafting_table_open { "CRAFTING TABLE" } else { "INVENTORY" };
        ui.text(title, win_x + 14, win_y + 7, 2, 1, pack_color(1.0, 0.88, 0.25));
        ui.text("[SURVIVAL]", win_x + win_w - 90, win_y + 9, 1, 1, pack_color(0.45, 0.85, 0.50));

        let surv_footer = if is_gamepad_active {
            "[A]: ALL | [X]: 1 ITEM/HALF | [Y]: QUICK MOVE | [B]: CLOSE"
        } else {
            "LMB: ALL | RMB: 1 ITEM / HALF | [E]/[ESC]/[B]: CLOSE"
        };
        ui.fill(Rect::new(win_x + 3, win_y + win_h - 22, win_w - 6, 19), pack_color(0.11, 0.11, 0.14));
        ui.fill(Rect::new(win_x + 3, win_y + win_h - 22, win_w - 6, 1), pack_color(0.07, 0.07, 0.09));
        ui.text(surv_footer, win_x + 12, win_y + win_h - 16, 1, 1, pack_color(0.70, 0.70, 0.75));

        if crafting_table_open {
            ui.inset_panel(Rect::new(start_x + 14, start_y + 30, 276, 108), pack_color(0.10, 0.10, 0.12), 0.85);
            ui.text("3x3 Grid", start_x + 40, start_y + 20, 1, 1, pack_color(0.8, 0.8, 0.8));

            let recipe_button = Rect::new(start_x + 190, start_y + 36, 88, 20);
            let recipe_button_hovered = hovered_slot == Some(InventorySlotTarget::RecipeBookToggle);
            let recipe_button_color = if recipe_book_open {
                pack_color(0.46, 0.32, 0.10)
            } else if recipe_button_hovered {
                pack_color(0.28, 0.28, 0.33)
            } else {
                pack_color(0.18, 0.18, 0.22)
            };
            ui.fill(recipe_button, recipe_button_color);
            ui.fill(Rect::new(recipe_button.x, recipe_button.y, recipe_button.w, 1), pack_color(0.44, 0.44, 0.49));
            ui.text("RECIPE BOOK", recipe_button.x + 6, recipe_button.y + 6, 1, 1, pack_color(1.0, 0.88, 0.25));

            let craft_3x3 = layout.crafting_3x3_grid();
            for idx in 0..9 {
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting3x3(idx));
                ui.inventory_slot(craft_3x3.slot(idx), player.inventory.crafting_3x3[idx], is_hovered, None);
            }

            ui.text("==>", start_x + 175, start_y + 85, 2, 1, pack_color(0.9, 0.85, 0.3));
            let result_item = recipes.match_grid(&player.inventory.crafting_3x3, 3, 3);
            ui.inventory_slot(
                Rect::new(start_x + 230, start_y + 75, 42, 42),
                result_item,
                hovered_slot == Some(InventorySlotTarget::CraftingOutput),
                None,
            );

            let char_box = Rect::new(start_x + 300, start_y + 30, 150, 108);
            ui.inset_panel(char_box, pack_color(0.08, 0.08, 0.10), 0.95);
            draw_armor_and_offhand_slots(ui.buffer, ui.width, ui.height, start_x + 300, start_y + 34, start_x + 360, start_y + 112, player, hovered_slot);
            ui.fill_alpha(Rect::new(char_box.x + 30, char_box.y + 88, 45, 12), pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(ui.buffer, ui.width, ui.height, char_box.x + 50, char_box.y + 54, mouse_pos, &player.inventory.armor, &player.inventory.offhand);
            draw_player_vitals_panel(&mut ui, char_box.x + 75, char_box.y, 68, player);

            if recipe_book_open {
                ui.fill_alpha(char_box.inset(2), pack_color(0.08, 0.08, 0.10), 0.99);
                ui.text("RECIPE BOOK", char_box.x + 8, char_box.y + 7, 1, 1, pack_color(1.0, 0.88, 0.25));
                let first = recipe_book_scroll.min(unlocked_recipe_indices.len());
                for (row, &recipe_index) in unlocked_recipe_indices.iter().skip(first).take(5).enumerate() {
                    let y = char_box.y + 21 + row as i32 * 15;
                    let is_hovered = hovered_slot == Some(InventorySlotTarget::RecipeEntry(recipe_index));
                    if is_hovered {
                        ui.fill(Rect::new(char_box.x + 4, y, char_box.w - 8, 14), pack_color(0.26, 0.24, 0.17));
                    }
                    if let Some(recipe) = recipes.recipes().get(recipe_index) {
                        draw_item_icon(ui.buffer, ui.width, ui.height, recipe.result.block, char_box.x + 6, y, 13);
                        let name = recipe.result.block.display_name().to_uppercase();
                        let visible_name: String = name.chars().take(13).collect();
                        ui.text(&visible_name, char_box.x + 25, y + 4, 1, 1, pack_color(0.9, 0.9, 0.92));
                    }
                }
                if unlocked_recipe_indices.is_empty() {
                    ui.text("Collect ingredients", char_box.x + 6, char_box.y + 33, 1, 1, pack_color(0.68, 0.68, 0.72));
                } else {
                    let status = recipe_book_status.unwrap_or("[WHEEL] SCROLL");
                    ui.text(status, char_box.x + 6, char_box.y + 96, 1, 1, pack_color(0.72, 0.72, 0.76));
                }
            }
        } else {
            let char_box = Rect::new(start_x + 14, start_y + 32, 186, 106);
            ui.inset_panel(char_box, pack_color(0.08, 0.08, 0.10), 0.95);
            draw_armor_and_offhand_slots(ui.buffer, ui.width, ui.height, start_x + 18, start_y + 34, start_x + 78, start_y + 112, player, hovered_slot);
            ui.fill_alpha(Rect::new(char_box.x + 36, char_box.y + 88, 48, 12), pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(ui.buffer, ui.width, ui.height, char_box.x + 56, char_box.y + 54, mouse_pos, &player.inventory.armor, &player.inventory.offhand);
            draw_player_vitals_panel(&mut ui, char_box.x + 106, char_box.y, 72, player);

            let craft_box = Rect::new(start_x + 208, start_y + 32, 242, 106);
            ui.inset_panel(craft_box, pack_color(0.10, 0.10, 0.12), 0.85);
            ui.text("CRAFTING (2x2)", craft_box.x + 12, craft_box.y + 8, 1, 1, pack_color(0.85, 0.85, 0.85));
            draw_crafting_2x2_slots(
                &mut ui,
                layout.crafting_2x2_grid(48),
                start_x + 310,
                start_y + 78,
                Rect::new(start_x + 350, start_y + 68, 42, 42),
                player,
                recipes,
                hovered_slot,
            );
        }

        ui.inset_panel(Rect::new(start_x + 14, start_y + 144, 436, 134), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("BACKPACK", start_x + 22, start_y + 146, 1, 1, pack_color(0.75, 0.75, 0.80));
        draw_backpack_slots(&mut ui, layout.main_grid(160), player, hovered_slot);

        ui.inset_panel(Rect::new(start_x + 10, start_y + 282, 444, 52), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("QUICK ACCESS HOTBAR", start_x + 16, start_y + 284, 1, 1, pack_color(0.85, 0.85, 0.40));
        draw_hotbar_slots(&mut ui, layout.hotbar_grid(295), player, hovered_slot, pack_color(0.4, 0.7, 1.0));
    }

    let hovered_item_name = hovered_slot.and_then(|target| match target {
        InventorySlotTarget::Hotbar(i) => player.inventory.hotbar[i].map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::Main(i) => player.inventory.main[i].map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::Crafting2x2(i) => player.inventory.crafting_2x2[i].map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::Crafting3x3(i) => player.inventory.crafting_3x3[i].map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::Armor(i) => player.inventory.armor[i].map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::Offhand => player.inventory.offhand.map(|s| (s.block.display_name().to_string(), s.count)),
        InventorySlotTarget::CraftingOutput => {
            let res = if crafting_table_open {
                recipes.match_grid(&player.inventory.crafting_3x3, 3, 3)
            } else {
                recipes.match_grid(&player.inventory.crafting_2x2, 2, 2)
            };
            res.map(|s| (s.block.display_name().to_string(), s.count))
        }
        InventorySlotTarget::Creative(id) => Some((BlockId::from_u8(id).display_name().to_string(), 64)),
        InventorySlotTarget::CreativeTab(idx) => CREATIVE_TABS.get(idx).map(|t| (format!("Category: {}", t), 1)),
        InventorySlotTarget::CreativeScrollUp => Some(("Scroll Up (Mouse Wheel Up)".to_string(), 1)),
        InventorySlotTarget::CreativeScrollDown => Some(("Scroll Down (Mouse Wheel Down)".to_string(), 1)),
        InventorySlotTarget::RecipeBookToggle => Some(("Open recipe book".to_string(), 1)),
        InventorySlotTarget::RecipeEntry(index) => recipes.recipes().get(index).map(|r| (r.result.block.display_name().to_string(), r.result.count)),
        InventorySlotTarget::Trash => {
            let tip = if is_gamepad_active {
                "TRASH ([A]: Delete held item | [X]: Clear hotbar)"
            } else {
                "TRASH (LMB: Delete held item | Shift+LMB: Clear hotbar)"
            };
            Some((tip.to_string(), 1))
        }
    });

    if let Some((name, count)) = hovered_item_name {
        let text = if count > 1 { format!("{} (x{})", name.to_uppercase(), count) } else { name.to_uppercase() };
        ui.tooltip(&text, mouse_pos, pack_color(1.0, 0.95, 0.4));
    }

    if let Some(held) = player.inventory.held_item {
        if let Some((mx, my)) = mouse_pos {
            let hx = mx as i32 - 14;
            let hy = my as i32 - 14;
            draw_item_icon(ui.buffer, ui.width, ui.height, held.block, hx, hy, 28);
            if held.count > 1 {
                let count_str = format!("{}", held.count);
                ui.text(&count_str, hx + 14, hy + 14, 1, 1, pack_color(1.0, 1.0, 1.0));
            }
        }
    }

    if let Some((mx, my)) = mouse_pos {
        draw_cursor_pointer(ui.buffer, ui.width, ui.height, mx as i32, my as i32);
    }
}
