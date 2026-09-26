use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::{GameMode, Player};
use crate::gameplay::recipe::RecipeBook;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::block::{BlockId, Face};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InventorySlotTarget {
    Hotbar(usize),
    Main(usize),
    Crafting2x2(usize),
    Crafting3x3(usize),
    CraftingOutput,
    Creative(u8),
    CreativeTab(usize),
    Trash,
}

pub const CREATIVE_TABS: [&str; 4] = [
    "BLOCKS",
    "TOOLS",
    "ALL",
    "INVENTORY",
];

pub fn creative_tab_blocks(tab: usize) -> &'static [BlockId] {
    match tab {
        0 => &[
            BlockId::Grass,
            BlockId::Dirt,
            BlockId::Stone,
            BlockId::Sand,
            BlockId::Wood,
            BlockId::Leaves,
            BlockId::SnowLeaves,
            BlockId::CherryLeaves,
            BlockId::AutumnLeaves,
            BlockId::Mud,
            BlockId::Snow,
            BlockId::Cactus,
            BlockId::CoalOre,
            BlockId::IronOre,
            BlockId::Bedrock,
            BlockId::Planks,
            BlockId::Chest,
            BlockId::Furnace,
            BlockId::FurnaceLit,
            BlockId::Glass,
            BlockId::Ice,
            BlockId::Torch,
        ],
        1 => &[
            BlockId::WoodenPickaxe,
            BlockId::StonePickaxe,
            BlockId::IronPickaxe,
            BlockId::WoodenAxe,
            BlockId::StoneAxe,
            BlockId::IronAxe,
            BlockId::WoodenShovel,
            BlockId::StoneShovel,
            BlockId::IronShovel,
            BlockId::WoodenSword,
            BlockId::StoneSword,
            BlockId::IronSword,
            BlockId::Stick,
            BlockId::Torch,
            BlockId::IronIngot,
            BlockId::GoldIngot,
            BlockId::RawPorkchop,
            BlockId::CookedPorkchop,
            BlockId::RawBeef,
            BlockId::CookedBeef,
            BlockId::Apple,
            BlockId::CraftingTable,
            BlockId::Chest,
            BlockId::Furnace,
            BlockId::Door,
            BlockId::Tnt,
            BlockId::Water,
            BlockId::Lava,
        ],
        2 => &[
            BlockId::Grass,
            BlockId::Dirt,
            BlockId::Stone,
            BlockId::Sand,
            BlockId::Wood,
            BlockId::Leaves,
            BlockId::SnowLeaves,
            BlockId::CherryLeaves,
            BlockId::AutumnLeaves,
            BlockId::Mud,
            BlockId::Snow,
            BlockId::Cactus,
            BlockId::CoalOre,
            BlockId::IronOre,
            BlockId::Water,
            BlockId::Bedrock,
            BlockId::Lava,
            BlockId::Planks,
            BlockId::Chest,
            BlockId::Furnace,
            BlockId::FurnaceLit,
            BlockId::Door,
            BlockId::Glass,
            BlockId::Ice,
            BlockId::Torch,
            BlockId::IronIngot,
            BlockId::GoldIngot,
            BlockId::RawPorkchop,
            BlockId::CookedPorkchop,
            BlockId::RawBeef,
            BlockId::CookedBeef,
            BlockId::Apple,
            BlockId::Tnt,
            BlockId::CraftingTable,
            BlockId::Stick,
            BlockId::WoodenPickaxe,
            BlockId::StonePickaxe,
            BlockId::IronPickaxe,
            BlockId::WoodenAxe,
            BlockId::StoneAxe,
            BlockId::IronAxe,
            BlockId::WoodenShovel,
            BlockId::StoneShovel,
            BlockId::IronShovel,
            BlockId::WoodenSword,
            BlockId::StoneSword,
            BlockId::IronSword,
        ],
        _ => &[],
    }
}

pub struct InventoryLayout {
    pub start_x: i32,
    pub start_y: i32,
    pub crafting_table_open: bool,
}

impl InventoryLayout {
    pub fn new(width: usize, height: usize, crafting_table_open: bool) -> Self {
        let start_x = (width as i32 - 464) / 2;
        let start_y = (height as i32 - 360) / 2;
        Self {
            start_x,
            start_y,
            crafting_table_open,
        }
    }

    pub fn find_slot_at(&self, px: i32, py: i32, player: &Player) -> Option<InventorySlotTarget> {
        if player.gamemode == GameMode::Creative {
            // Check top tabs (4 tabs)
            for i in 0..4 {
                let tab_x = self.start_x + 8 + i as i32 * 112;
                let tab_y = self.start_y - 24;
                if px >= tab_x && px < tab_x + 108 && py >= tab_y && py < tab_y + 26 {
                    return Some(InventorySlotTarget::CreativeTab(i));
                }
            }

            if player.creative_tab < 3 {
                // Creative item grid (9 cols x 4 rows)
                let blocks = creative_tab_blocks(player.creative_tab);
                let grid_x = self.start_x + 22;
                let grid_y = self.start_y + 36;
                for row in 0..4 {
                    for col in 0..9 {
                        let idx = row * 9 + col;
                        if idx < blocks.len() {
                            let x = grid_x + col as i32 * 42;
                            let y = grid_y + row as i32 * 42;
                            if px >= x && px < x + 38 && py >= y && py < y + 38 {
                                return Some(InventorySlotTarget::Creative(blocks[idx].to_u8()));
                            }
                        }
                    }
                }

                // Hotbar: 10 slots
                let hotbar_x = self.start_x + 16;
                let hotbar_y = self.start_y + 242;
                for i in 0..10 {
                    let x = hotbar_x + i as i32 * 40;
                    let y = hotbar_y;
                    if px >= x && px < x + 38 && py >= y && py < y + 38 {
                        return Some(InventorySlotTarget::Hotbar(i));
                    }
                }

                // Trash slot
                let trash_x = self.start_x + 420;
                let trash_y = self.start_y + 242;
                if px >= trash_x && px < trash_x + 36 && py >= trash_y && py < trash_y + 38 {
                    return Some(InventorySlotTarget::Trash);
                }

                return None;
            } else {
                // Tab 3: Survival Inventory in Creative
                // Hotbar: 10 slots
                let hotbar_x = self.start_x + 16;
                let hotbar_y = self.start_y + 276;
                for i in 0..10 {
                    let x = hotbar_x + i as i32 * 40;
                    let y = hotbar_y;
                    if px >= x && px < x + 38 && py >= y && py < y + 38 {
                        return Some(InventorySlotTarget::Hotbar(i));
                    }
                }

                // Trash slot
                let trash_x = self.start_x + 420;
                let trash_y = self.start_y + 276;
                if px >= trash_x && px < trash_x + 36 && py >= trash_y && py < trash_y + 38 {
                    return Some(InventorySlotTarget::Trash);
                }

                // Main inventory: 3 rows x 9 cols (27 slots)
                let main_x = self.start_x + 22;
                let main_y = self.start_y + 144;
                for row in 0..3 {
                    for col in 0..9 {
                        let x = main_x + col as i32 * 42;
                        let y = main_y + row as i32 * 42;
                        if px >= x && px < x + 38 && py >= y && py < y + 38 {
                            return Some(InventorySlotTarget::Main(row * 9 + col));
                        }
                    }
                }

                // 2x2 crafting grid
                let craft_x = self.start_x + 220;
                let craft_y = self.start_y + 42;
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
                let out_y = self.start_y + 60;
                if px >= out_x && px < out_x + 42 && py >= out_y && py < out_y + 42 {
                    return Some(InventorySlotTarget::CraftingOutput);
                }

                return None;
            }
        }

        // Survival Mode (regular)
        let hotbar_x = self.start_x + 16;
        let hotbar_y = self.start_y + 295;
        for i in 0..10 {
            let x = hotbar_x + i as i32 * 40;
            let y = hotbar_y;
            if px >= x && px < x + 38 && py >= y && py < y + 38 {
                return Some(InventorySlotTarget::Hotbar(i));
            }
        }

        // Main inventory: 3 rows x 9 cols (27 slots)
        let main_x = self.start_x + 22;
        let main_y = self.start_y + 160;
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
            let craft_y = self.start_y + 48;
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
            let out_y = self.start_y + 68;
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

    if player.gamemode == GameMode::Creative {
        // Draw 4 Top Tabs
        for i in 0..4 {
            let tab_x = start_x + 8 + i as i32 * 112;
            let tab_y = start_y - 24;
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

            fill_rect(buffer, width, height, tab_x, tab_y, 108, 26, bg_color);
            fill_rect(buffer, width, height, tab_x, tab_y, 108, 2, border_top);
            fill_rect(buffer, width, height, tab_x, tab_y, 2, 26, border_top);
            fill_rect(buffer, width, height, tab_x + 106, tab_y, 2, 26, pack_color(0.06, 0.06, 0.08));

            let text_color = if is_active {
                pack_color(1.0, 0.95, 0.3)
            } else if is_hovered {
                pack_color(0.9, 0.9, 0.9)
            } else {
                pack_color(0.55, 0.55, 0.60)
            };

            let label = CREATIVE_TABS[i];
            let tw = crate::graphics::font::measure_text(label, 1, 1);
            let tx = tab_x + (108 - tw) / 2;
            let ty = tab_y + 8;
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, 1, 1, text_color);
        }

        // Main creative window frame
        fill_rect_alpha(buffer, width, height, start_x - 8, start_y, 480, 360, pack_color(0.14, 0.14, 0.16), 0.98);
        fill_rect(buffer, width, height, start_x - 8, start_y, 480, 3, pack_color(0.38, 0.38, 0.42));
        fill_rect(buffer, width, height, start_x - 8, start_y + 357, 480, 3, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x - 8, start_y, 3, 360, pack_color(0.38, 0.38, 0.42));
        fill_rect(buffer, width, height, start_x + 469, start_y, 3, 360, pack_color(0.05, 0.05, 0.06));

        if player.creative_tab < 3 {
            // Category title
            let title = match player.creative_tab {
                0 => "BUILDING BLOCKS",
                1 => "TOOLS & ITEMS",
                2 => "ALL ITEMS",
                _ => "",
            };
            draw_text_shadowed(buffer, width, height, title, start_x + 18, start_y + 14, 2, 1);

            // Creative block grid (9 cols x 4 rows)
            let blocks = creative_tab_blocks(player.creative_tab);
            let grid_x = start_x + 22;
            let grid_y = start_y + 36;
            for row in 0..4 {
                for col in 0..9 {
                    let idx = row * 9 + col;
                    let x = grid_x + col as i32 * 42;
                    let y = grid_y + row as i32 * 42;

                    if idx < blocks.len() {
                        let block = blocks[idx];
                        let is_hovered = hovered_slot == Some(InventorySlotTarget::Creative(block.to_u8()));
                        draw_slot_custom(buffer, width, height, x, y, 38, 38, Some(ItemStack { block, count: 64 }), is_hovered, None);
                    } else {
                        // Empty slot frame
                        draw_slot_custom(buffer, width, height, x, y, 38, 38, None, false, Some(pack_color(0.18, 0.18, 0.20)));
                    }
                }
            }

            // Divider & Hotbar Header
            fill_rect(buffer, width, height, start_x + 16, start_y + 218, 432, 2, pack_color(0.25, 0.25, 0.28));
            draw_text_colored_shadowed(buffer, width, height, "QUICK ACCESS HOTBAR", start_x + 16, start_y + 226, 1, 1, pack_color(0.85, 0.85, 0.4));
            draw_text_colored_shadowed(buffer, width, height, "TRASH", start_x + 424, start_y + 226, 1, 1, pack_color(1.0, 0.4, 0.4));

            // Hotbar 10 slots
            let hotbar_x = start_x + 16;
            let hotbar_y = start_y + 242;
            for i in 0..10 {
                let x = hotbar_x + i as i32 * 40;
                let y = hotbar_y;
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Hotbar(i));
                let is_selected = player.selected_hotbar_slot as usize == i;
                let border_override = if is_selected && !is_hovered {
                    Some(pack_color(0.4, 0.75, 1.0))
                } else {
                    None
                };
                draw_slot_custom(buffer, width, height, x, y, 38, 38, player.inventory.hotbar[i], is_hovered, border_override);
            }

            // Trash Slot
            let trash_x = start_x + 420;
            let trash_y = start_y + 242;
            let is_trash_hover = hovered_slot == Some(InventorySlotTarget::Trash);
            let trash_border = if is_trash_hover {
                pack_color(1.0, 0.25, 0.25)
            } else {
                pack_color(0.45, 0.18, 0.18)
            };
            fill_rect(buffer, width, height, trash_x, trash_y, 36, 38, trash_border);
            fill_rect(buffer, width, height, trash_x + 2, trash_y + 2, 32, 34, pack_color(0.16, 0.08, 0.08));
            draw_trash_icon(buffer, width, height, trash_x + 6, trash_y + 7);

            // Controls Footer
            #[cfg(target_os = "vita")]
            draw_text_colored_shadowed(
                buffer, width, height,
                "[X]/TOUCH: TAKE STACK | [SQUARE]: DELETE | [D-PAD]: SELECT | [O]: CLOSE",
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
            #[cfg(not(target_os = "vita"))]
            draw_text_colored_shadowed(
                buffer, width, height,
                "LMB: TAKE (64) | SHIFT+LMB: TO HOTBAR | 1-0: SELECT SLOT | [E]/[ESC]: CLOSE",
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
        } else {
            // Tab 3: Survival Inventory inside Creative
            draw_text_shadowed(buffer, width, height, "SURVIVAL INVENTORY", start_x + 18, start_y + 14, 2, 1);

            // Character Info Box
            fill_rect_alpha(buffer, width, height, start_x + 22, start_y + 38, 130, 92, pack_color(0.08, 0.08, 0.10), 0.85);
            draw_text_shadowed(buffer, width, height, "CHARACTER (CREATIVE)", start_x + 30, start_y + 46, 1, 1);
            draw_text_colored_shadowed(buffer, width, height, "INFINITE RESOURCES", start_x + 30, start_y + 68, 1, 1, pack_color(0.4, 0.9, 0.4));
            draw_text_colored_shadowed(buffer, width, height, "FLYING: ENABLED", start_x + 30, start_y + 88, 1, 1, pack_color(0.4, 0.8, 1.0));

            // 2x2 Crafting Grid
            draw_text_colored_shadowed(buffer, width, height, "Crafting 2x2", start_x + 220, start_y + 26, 1, 1, pack_color(0.85, 0.85, 0.85));
            let craft_x = start_x + 220;
            let craft_y = start_y + 42;
            for row in 0..2 {
                for col in 0..2 {
                    let idx = row * 2 + col;
                    let x = craft_x + col as i32 * 40;
                    let y = craft_y + row as i32 * 40;
                    let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting2x2(idx));
                    draw_slot(buffer, width, height, x, y, 36, 36, player.inventory.crafting_2x2[idx], is_hovered);
                }
            }

            // Arrow & Output slot
            draw_text_colored_shadowed(buffer, width, height, "==>", start_x + 310, start_y + 70, 2, 1, pack_color(0.9, 0.85, 0.3));
            let out_x = start_x + 350;
            let out_y = start_y + 60;
            let result_item = recipes.match_grid(&player.inventory.crafting_2x2, 2, 2);
            let is_out_hovered = hovered_slot == Some(InventorySlotTarget::CraftingOutput);
            draw_slot(buffer, width, height, out_x, out_y, 42, 42, result_item, is_out_hovered);

            // Main 27 slots
            draw_text_colored_shadowed(buffer, width, height, "Backpack", start_x + 22, start_y + 132, 1, 1, pack_color(0.75, 0.75, 0.75));
            let main_x = start_x + 22;
            let main_y = start_y + 144;
            for row in 0..3 {
                for col in 0..9 {
                    let idx = row * 9 + col;
                    let x = main_x + col as i32 * 42;
                    let y = main_y + row as i32 * 42;
                    let is_hovered = hovered_slot == Some(InventorySlotTarget::Main(idx));
                    draw_slot(buffer, width, height, x, y, 38, 38, player.inventory.main[idx], is_hovered);
                }
            }

            // Hotbar & Trash
            draw_text_colored_shadowed(buffer, width, height, "Hotbar", start_x + 16, start_y + 264, 1, 1, pack_color(0.85, 0.85, 0.4));
            draw_text_colored_shadowed(buffer, width, height, "TRASH", start_x + 424, start_y + 264, 1, 1, pack_color(1.0, 0.4, 0.4));
            let hotbar_x = start_x + 16;
            let hotbar_y = start_y + 276;
            for i in 0..10 {
                let x = hotbar_x + i as i32 * 40;
                let y = hotbar_y;
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Hotbar(i));
                let is_selected = player.selected_hotbar_slot as usize == i;
                let border_override = if is_selected && !is_hovered {
                    Some(pack_color(0.4, 0.75, 1.0))
                } else {
                    None
                };
                draw_slot_custom(buffer, width, height, x, y, 38, 38, player.inventory.hotbar[i], is_hovered, border_override);
            }

            let trash_x = start_x + 420;
            let trash_y = start_y + 276;
            let is_trash_hover = hovered_slot == Some(InventorySlotTarget::Trash);
            let trash_border = if is_trash_hover { pack_color(1.0, 0.25, 0.25) } else { pack_color(0.45, 0.18, 0.18) };
            fill_rect(buffer, width, height, trash_x, trash_y, 36, 38, trash_border);
            fill_rect(buffer, width, height, trash_x + 2, trash_y + 2, 32, 34, pack_color(0.16, 0.08, 0.08));
            draw_trash_icon(buffer, width, height, trash_x + 6, trash_y + 7);

            // Controls Footer
            #[cfg(not(target_os = "vita"))]
            draw_text_colored_shadowed(
                buffer, width, height,
                "LPM: ALL | PPM: 1 ITEM/HALF | SHIFT+LPM: MOVE | [E]/[ESC]: CLOSE",
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
            #[cfg(target_os = "vita")]
            draw_text_colored_shadowed(
                buffer, width, height,
                "[X]: ALL | [SQUARE]: 1 ITEM/HALF | [O]: CLOSE",
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
        }
    } else {
        // Survival Mode
        let title = if crafting_table_open { "CRAFTING TABLE" } else { "INVENTORY" };
        draw_text_shadowed(buffer, width, height, title, start_x + 10, start_y + 8, 2, 1);
        #[cfg(target_os = "vita")]
        draw_text_shadowed(buffer, width, height, "[X]/TOUCH: ALL | [SQUARE]: 1 ITEM/HALF | [O]/[TRIANGLE]: CLOSE", start_x + 10, start_y + 342, 1, 1);
        #[cfg(not(target_os = "vita"))]
        draw_text_shadowed(buffer, width, height, "LMB: ALL | RMB: 1 ITEM / HALF | [E]/[ESC]/[B]: CLOSE", start_x + 10, start_y + 342, 1, 1);

        // Crafting section
        if crafting_table_open {
            draw_text_colored_shadowed(buffer, width, height, "3x3 Grid", start_x + 40, start_y + 20, 1, 1, pack_color(0.8, 0.8, 0.8));
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
            fill_rect_alpha(buffer, width, height, start_x + 22, start_y + 38, 130, 95, pack_color(0.08, 0.08, 0.10), 0.8);
            draw_text_shadowed(buffer, width, height, "CHARACTER", start_x + 60, start_y + 48, 1, 1);
            let hp_text = format!("HP: {}/20", player.health as i32);
            draw_text_colored_shadowed(buffer, width, height, &hp_text, start_x + 35, start_y + 78, 1, 1, pack_color(1.0, 0.3, 0.3));

            // 2x2 Crafting
            draw_text_colored_shadowed(buffer, width, height, "Crafting 2x2", start_x + 220, start_y + 32, 1, 1, pack_color(0.85, 0.85, 0.85));
            let craft_x = start_x + 220;
            let craft_y = start_y + 48;
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
            draw_text_colored_shadowed(buffer, width, height, "==>", start_x + 310, start_y + 78, 2, 1, pack_color(0.9, 0.85, 0.3));

            // Output slot
            let out_x = start_x + 350;
            let out_y = start_y + 68;
            let result_item = recipes.match_grid(&player.inventory.crafting_2x2, 2, 2);
            let is_out_hovered = hovered_slot == Some(InventorySlotTarget::CraftingOutput);
            draw_slot(buffer, width, height, out_x, out_y, 42, 42, result_item, is_out_hovered);
        }

        // Main Inventory
        draw_text_colored_shadowed(buffer, width, height, "Backpack", start_x + 22, start_y + 146, 1, 1, pack_color(0.75, 0.75, 0.75));
        let main_x = start_x + 22;
        let main_y = start_y + 160;
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
        draw_text_colored_shadowed(buffer, width, height, "Hotbar", start_x + 16, start_y + 282, 1, 1, pack_color(0.75, 0.75, 0.75));
        let hotbar_x = start_x + 16;
        let hotbar_y = start_y + 295;
        for i in 0..10 {
            let x = hotbar_x + i as i32 * 40;
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
            InventorySlotTarget::Hotbar(i) => hovered_item_name = player.inventory.hotbar[i].map(|s| (s.block.name().to_string(), s.count)),
            InventorySlotTarget::Main(i) => hovered_item_name = player.inventory.main[i].map(|s| (s.block.name().to_string(), s.count)),
            InventorySlotTarget::Crafting2x2(i) => hovered_item_name = player.inventory.crafting_2x2[i].map(|s| (s.block.name().to_string(), s.count)),
            InventorySlotTarget::Crafting3x3(i) => hovered_item_name = player.inventory.crafting_3x3[i].map(|s| (s.block.name().to_string(), s.count)),
            InventorySlotTarget::CraftingOutput => {
                let res = if crafting_table_open {
                    recipes.match_grid(&player.inventory.crafting_3x3, 3, 3)
                } else {
                    recipes.match_grid(&player.inventory.crafting_2x2, 2, 2)
                };
                hovered_item_name = res.map(|s| (s.block.name().to_string(), s.count));
            }
            InventorySlotTarget::Creative(id) => {
                let block = BlockId::from_u8(id);
                hovered_item_name = Some((block.name().to_string(), 64));
            }
            InventorySlotTarget::CreativeTab(idx) => {
                if idx < CREATIVE_TABS.len() {
                    hovered_item_name = Some((format!("Category: {}", CREATIVE_TABS[idx]), 1));
                }
            }
            InventorySlotTarget::Trash => {
                hovered_item_name = Some(("TRASH (LMB: Delete held item | Shift+LMB: Clear hotbar)".to_string(), 1));
            }
        }
    }

    if let Some((name, count)) = hovered_item_name {
        let text = if count > 1 { format!("{} (x{})", name.to_uppercase(), count) } else { name.to_uppercase() };
        if let Some((mx, my)) = mouse_pos {
            let tx = (mx as i32 + 15).min(width as i32 - (text.len() as i32 * 8 + 20));
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

    // Draw Cursor Pointer on screen
    if let Some((mx, my)) = mouse_pos {
        crate::ui::draw::draw_cursor_pointer(buffer, width, height, mx as i32, my as i32);
    }
}

pub fn draw_trash_icon(buffer: &mut [u32], buf_w: usize, buf_h: usize, x: i32, y: i32) {
    let red_hi = pack_color(1.0, 0.40, 0.40);
    let red_body = pack_color(0.85, 0.20, 0.20);
    let red_dark = pack_color(0.50, 0.10, 0.10);

    // Can lid handle
    fill_rect(buffer, buf_w, buf_h, x + 10, y + 4, 4, 2, red_hi);
    // Can lid
    fill_rect(buffer, buf_w, buf_h, x + 6, y + 6, 12, 2, red_hi);
    // Can body
    fill_rect(buffer, buf_w, buf_h, x + 7, y + 9, 10, 12, red_body);
    // Vertical lines on can
    fill_rect(buffer, buf_w, buf_h, x + 9, y + 10, 1, 10, red_dark);
    fill_rect(buffer, buf_w, buf_h, x + 11, y + 10, 1, 10, red_dark);
    fill_rect(buffer, buf_w, buf_h, x + 13, y + 10, 1, 10, red_dark);
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
        BlockId::RawPorkchop => {
            let outline = pack_color(0.48, 0.20, 0.26);
            let base = pack_color(0.92, 0.58, 0.64);
            let hi = pack_color(0.98, 0.78, 0.82);
            let sh = pack_color(0.75, 0.38, 0.45);
            let bone = pack_color(0.95, 0.95, 0.88);

            // Bone tip top-right
            px(buffer, 12, 3, bone);
            px(buffer, 13, 2, bone);
            px(buffer, 14, 2, bone);
            px(buffer, 13, 4, outline);

            // Meat body
            for ly in 4..=12 {
                for lx in 2..=11 {
                    let in_shape = (lx + ly >= 7) && (lx + ly <= 21) && (ly - lx <= 8) && (lx - ly <= 6);
                    if in_shape {
                        let is_border = lx == 2 || lx == 11 || ly == 4 || ly == 12;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if lx <= 5 && ly <= 7 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 8 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::CookedPorkchop => {
            let outline = pack_color(0.28, 0.12, 0.05);
            let base = pack_color(0.72, 0.42, 0.18);
            let hi = pack_color(0.90, 0.65, 0.32);
            let sh = pack_color(0.45, 0.22, 0.08);
            let bone = pack_color(0.92, 0.90, 0.82);

            px(buffer, 12, 3, bone);
            px(buffer, 13, 2, bone);
            px(buffer, 14, 2, bone);
            px(buffer, 13, 4, outline);

            for ly in 4..=12 {
                for lx in 2..=11 {
                    let in_shape = (lx + ly >= 7) && (lx + ly <= 21) && (ly - lx <= 8) && (lx - ly <= 6);
                    if in_shape {
                        let is_border = lx == 2 || lx == 11 || ly == 4 || ly == 12;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if lx <= 5 && ly <= 7 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 8 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::RawBeef => {
            let outline = pack_color(0.38, 0.08, 0.08);
            let base = pack_color(0.82, 0.20, 0.20);
            let hi = pack_color(0.94, 0.42, 0.42);
            let sh = pack_color(0.52, 0.10, 0.10);
            let fat = pack_color(0.96, 0.90, 0.88);

            for ly in 3..=13 {
                for lx in 2..=13 {
                    let in_shape = (lx + ly >= 7) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 8);
                    if in_shape {
                        let is_border = lx == 2 || lx == 13 || ly == 3 || ly == 13;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if (lx == 5 && ly == 7) || (lx == 6 && ly == 8) || (lx == 9 && ly == 6) {
                            px(buffer, lx, ly, fat);
                        } else if lx <= 6 && ly <= 6 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 9 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::CookedBeef => {
            let outline = pack_color(0.18, 0.08, 0.04);
            let base = pack_color(0.50, 0.26, 0.14);
            let hi = pack_color(0.70, 0.42, 0.22);
            let sh = pack_color(0.28, 0.12, 0.06);
            let grill = pack_color(0.15, 0.06, 0.02);

            for ly in 3..=13 {
                for lx in 2..=13 {
                    let in_shape = (lx + ly >= 7) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 8);
                    if in_shape {
                        let is_border = lx == 2 || lx == 13 || ly == 3 || ly == 13;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if (lx + ly == 12) || (lx + ly == 16) {
                            px(buffer, lx, ly, grill);
                        } else if lx <= 6 && ly <= 6 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 9 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::Apple => {
            let outline = pack_color(0.32, 0.06, 0.06);
            let base = pack_color(0.88, 0.14, 0.14);
            let hi = pack_color(0.98, 0.48, 0.48);
            let sh = pack_color(0.52, 0.08, 0.08);
            let stem = pack_color(0.42, 0.26, 0.12);
            let leaf = pack_color(0.35, 0.78, 0.22);

            // Stem & Leaf
            px(buffer, 8, 2, stem);
            px(buffer, 8, 3, stem);
            px(buffer, 9, 2, leaf);
            px(buffer, 10, 2, leaf);

            // Apple body
            for ly in 4..=13 {
                for lx in 4..=12 {
                    let in_shape = (lx + ly >= 9) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 7);
                    if in_shape {
                        let is_border = lx == 4 || lx == 12 || ly == 4 || ly == 13;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if lx >= 6 && lx <= 8 && ly >= 6 && ly <= 7 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 10 || ly >= 11 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creative_tabs_and_block_lists() {
        assert_eq!(CREATIVE_TABS.len(), 4);
        let tab0 = creative_tab_blocks(0);
        assert!(tab0.contains(&BlockId::Grass));
        assert!(tab0.contains(&BlockId::Stone));
        assert!(tab0.contains(&BlockId::Wood));

        let tab1 = creative_tab_blocks(1);
        assert!(tab1.contains(&BlockId::WoodenPickaxe));
        assert!(tab1.contains(&BlockId::IronSword));
        assert!(tab1.contains(&BlockId::Tnt));

        let tab2 = creative_tab_blocks(2);
        assert_eq!(tab2.len(), 47);
        assert!(tab2.contains(&BlockId::Grass));
        assert!(tab2.contains(&BlockId::Torch));
        assert!(tab2.contains(&BlockId::IronSword));
        assert!(tab2.contains(&BlockId::RawPorkchop));
        assert!(tab2.contains(&BlockId::CookedBeef));
        assert!(tab2.contains(&BlockId::Chest));
        assert!(tab2.contains(&BlockId::Furnace));
    }

    #[test]
    fn test_creative_layout_slot_detection() {
        let layout = InventoryLayout::new(960, 544, false);
        let mut player = Player::new();
        player.gamemode = GameMode::Creative;
        player.creative_tab = 0;

        // Top tab click (Tab 0: BLOCKS)
        let tab0_x = layout.start_x + 20;
        let tab0_y = layout.start_y - 10;
        assert_eq!(layout.find_slot_at(tab0_x, tab0_y, &player), Some(InventorySlotTarget::CreativeTab(0)));

        // Top tab click (Tab 1: TOOLS)
        let tab1_x = layout.start_x + 130;
        assert_eq!(layout.find_slot_at(tab1_x, tab0_y, &player), Some(InventorySlotTarget::CreativeTab(1)));

        // First creative block in tab 0 (Grass)
        let block0_x = layout.start_x + 30;
        let block0_y = layout.start_y + 45;
        assert_eq!(layout.find_slot_at(block0_x, block0_y, &player), Some(InventorySlotTarget::Creative(BlockId::Grass.to_u8())));

        // Hotbar slot 0 in creative mode
        let hotbar0_x = layout.start_x + 25;
        let hotbar0_y = layout.start_y + 250;
        assert_eq!(layout.find_slot_at(hotbar0_x, hotbar0_y, &player), Some(InventorySlotTarget::Hotbar(0)));

        // Trash slot in creative mode
        let trash_x = layout.start_x + 430;
        let trash_y = layout.start_y + 250;
        assert_eq!(layout.find_slot_at(trash_x, trash_y, &player), Some(InventorySlotTarget::Trash));
    }

    #[test]
    fn test_creative_hotbar_assignment_and_trash() {
        let mut inv = crate::gameplay::inventory::Inventory::new();
        // Place 64 grass into hotbar slot 0
        inv.hotbar[0] = Some(ItemStack { block: BlockId::Grass, count: 64 });
        assert_eq!(inv.hotbar[0], Some(ItemStack { block: BlockId::Grass, count: 64 }));

        // Trash action: delete held item
        inv.held_item = Some(ItemStack { block: BlockId::StonePickaxe, count: 1 });
        inv.held_item = None;
        assert_eq!(inv.held_item, None);

        // Shift-trash action: clear entire hotbar
        inv.hotbar.fill(None);
        assert!(inv.hotbar.iter().all(|s| s.is_none()));
    }
}

