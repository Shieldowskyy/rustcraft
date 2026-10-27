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
    CreativeScrollUp,
    CreativeScrollDown,
    Trash,
}

pub const CREATIVE_TABS: [&str; 6] = [
    "BLOCKS",
    "TOOLS",
    "REDSTONE",
    "FARMING",
    "ALL",
    "INVENTORY",
];

pub fn creative_tab_blocks(tab: usize) -> &'static [BlockId] {
    use std::sync::OnceLock;
    static TAB_0: OnceLock<&'static [BlockId]> = OnceLock::new();
    static TAB_1: OnceLock<&'static [BlockId]> = OnceLock::new();
    static TAB_2: OnceLock<&'static [BlockId]> = OnceLock::new();
    static TAB_3: OnceLock<&'static [BlockId]> = OnceLock::new();
    static TAB_4: OnceLock<&'static [BlockId]> = OnceLock::new();

    match tab {
        0 => *TAB_0.get_or_init(|| {
            let mut list = vec![
                BlockId::Grass,
                BlockId::Dirt,
                BlockId::Farmland,
                BlockId::FarmlandWet,
                BlockId::Stone,
                BlockId::Sand,
                BlockId::Mud,
                BlockId::Wood,
                BlockId::Leaves,
                BlockId::SnowLeaves,
                BlockId::CherryLeaves,
                BlockId::AutumnLeaves,
                BlockId::Sapling,
                BlockId::Cactus,
                BlockId::Snow,
                BlockId::Ice,
                BlockId::Bedrock,
                BlockId::Water,
                BlockId::Lava,
                BlockId::Glass,
                BlockId::CoalOre,
                BlockId::IronOre,
                BlockId::GoldOre,
                BlockId::DiamondOre,
                BlockId::EmeraldOre,
                BlockId::RustStoneOre,
                BlockId::CoalBlock,
                BlockId::IronBlock,
                BlockId::GoldBlock,
                BlockId::DiamondBlock,
                BlockId::EmeraldBlock,
                BlockId::RustStoneBlock,
                BlockId::Planks,
                BlockId::WoodenSlab,
                BlockId::StoneSlab,
                BlockId::WoodenStairs,
                BlockId::StoneStairs,
                BlockId::Ladder,
                BlockId::Door,
                BlockId::CraftingTable,
                BlockId::Furnace,
                BlockId::FurnaceLit,
                BlockId::Chest,
                BlockId::Bed,
                BlockId::Tnt,
                BlockId::Torch,
                BlockId::RustStone,
                BlockId::RustStoneLamp,
                BlockId::RustStoneLampLit,
                BlockId::Lever,
                BlockId::Button,
                BlockId::PressurePlate,
                BlockId::PigSpawnEgg,
                BlockId::CowSpawnEgg,
                BlockId::ChickenSpawnEgg,
                BlockId::SheepSpawnEgg,
                BlockId::VillagerSpawnEgg,
                BlockId::DolphinSpawnEgg,
                BlockId::FishSpawnEgg,
                BlockId::Kelp,
                BlockId::Seagrass,
                BlockId::WhiteWool,
                BlockId::OrangeWool,
                BlockId::MagentaWool,
                BlockId::LightBlueWool,
                BlockId::YellowWool,
                BlockId::LimeWool,
                BlockId::PinkWool,
                BlockId::GrayWool,
                BlockId::LightGrayWool,
                BlockId::CyanWool,
                BlockId::PurpleWool,
                BlockId::BlueWool,
                BlockId::BrownWool,
                BlockId::GreenWool,
                BlockId::RedWool,
                BlockId::BlackWool,
            ];
            list.extend(crate::engine::modding::get_custom_creative_blocks());
            Box::leak(list.into_boxed_slice())
        }),
        1 => *TAB_1.get_or_init(|| {
            let mut list = vec![
                BlockId::WoodenPickaxe,
                BlockId::StonePickaxe,
                BlockId::IronPickaxe,
                BlockId::GoldPickaxe,
                BlockId::DiamondPickaxe,
                BlockId::WoodenAxe,
                BlockId::StoneAxe,
                BlockId::IronAxe,
                BlockId::GoldAxe,
                BlockId::DiamondAxe,
                BlockId::WoodenShovel,
                BlockId::StoneShovel,
                BlockId::IronShovel,
                BlockId::GoldShovel,
                BlockId::DiamondShovel,
                BlockId::WoodenSword,
                BlockId::StoneSword,
                BlockId::IronSword,
                BlockId::GoldSword,
                BlockId::DiamondSword,
                BlockId::WoodenHoe,
                BlockId::StoneHoe,
                BlockId::IronHoe,
                BlockId::GoldHoe,
                BlockId::DiamondHoe,
                BlockId::Shears,
                BlockId::Stick,
                BlockId::Bone,
                BlockId::BoneMeal,
                BlockId::Gunpowder,
                BlockId::Feather,
                BlockId::Torch,
                BlockId::IronOunce,
                BlockId::IronIngot,
                BlockId::GoldOunce,
                BlockId::GoldIngot,
                BlockId::Diamond,
                BlockId::Emerald,
                BlockId::EmeraldBlock,
                BlockId::RustStone,
                BlockId::RustStoneBlock,
                BlockId::RustStoneLamp,
                BlockId::Lever,
                BlockId::Button,
                BlockId::PressurePlate,
                BlockId::Apple,
                BlockId::WheatSeeds,
                BlockId::Wheat,
                BlockId::Bread,
                BlockId::Carrot,
                BlockId::Potato,
                BlockId::BakedPotato,
                BlockId::RawPorkchop,
                BlockId::CookedPorkchop,
                BlockId::RawBeef,
                BlockId::CookedBeef,
                BlockId::RawChicken,
                BlockId::CookedChicken,
                BlockId::PigSpawnEgg,
                BlockId::CowSpawnEgg,
                BlockId::ChickenSpawnEgg,
                BlockId::SheepSpawnEgg,
                BlockId::VillagerSpawnEgg,
                BlockId::DolphinSpawnEgg,
                BlockId::FishSpawnEgg,
                BlockId::FishingRod,
                BlockId::RawCod,
                BlockId::CookedCod,
                BlockId::RawSalmon,
                BlockId::CookedSalmon,
                BlockId::Pufferfish,
                BlockId::TropicalFish,
                BlockId::DriedKelp,
                BlockId::Sapling,
                BlockId::Water,
                BlockId::Lava,
                BlockId::Door,
                BlockId::Bed,
                BlockId::Ladder,
                BlockId::CraftingTable,
                BlockId::Chest,
                BlockId::Furnace,
                BlockId::Tnt,
            ];
            list.extend(crate::engine::modding::get_custom_creative_items());
            Box::leak(list.into_boxed_slice())
        }),
        2 => *TAB_2.get_or_init(|| {
            let list = vec![
                BlockId::RustStone,
                BlockId::RustStoneOre,
                BlockId::RustStoneBlock,
                BlockId::RustStoneLamp,
                BlockId::RustStoneLampLit,
                BlockId::RustStoneWireLit,
                BlockId::Lever,
                BlockId::LeverOn,
                BlockId::Button,
                BlockId::ButtonPressed,
                BlockId::PressurePlate,
                BlockId::PressurePlatePressed,
                BlockId::Tnt,
                BlockId::Torch,
                BlockId::Door,
                BlockId::Chest,
                BlockId::Furnace,
                BlockId::FurnaceLit,
                BlockId::CraftingTable,
            ];
            Box::leak(list.into_boxed_slice())
        }),
        3 => *TAB_3.get_or_init(|| {
            let list = vec![
                BlockId::WoodenHoe,
                BlockId::StoneHoe,
                BlockId::IronHoe,
                BlockId::GoldHoe,
                BlockId::DiamondHoe,
                BlockId::WheatSeeds,
                BlockId::Wheat,
                BlockId::WheatCrop,
                BlockId::Bread,
                BlockId::Carrot,
                BlockId::CarrotCrop,
                BlockId::Potato,
                BlockId::PotatoCrop,
                BlockId::BakedPotato,
                BlockId::Apple,
                BlockId::Bone,
                BlockId::BoneMeal,
                BlockId::Sapling,
                BlockId::Cactus,
                BlockId::Water,
                BlockId::Farmland,
                BlockId::FarmlandWet,
                BlockId::PigSpawnEgg,
                BlockId::CowSpawnEgg,
                BlockId::ChickenSpawnEgg,
                BlockId::SheepSpawnEgg,
                BlockId::VillagerSpawnEgg,
                BlockId::DolphinSpawnEgg,
                BlockId::FishSpawnEgg,
                BlockId::FishingRod,
                BlockId::RawCod,
                BlockId::CookedCod,
                BlockId::RawSalmon,
                BlockId::CookedSalmon,
                BlockId::Pufferfish,
                BlockId::TropicalFish,
                BlockId::DriedKelp,
                BlockId::Kelp,
                BlockId::Seagrass,
                BlockId::RawPorkchop,
                BlockId::CookedPorkchop,
                BlockId::RawBeef,
                BlockId::CookedBeef,
                BlockId::RawChicken,
                BlockId::CookedChicken,
                BlockId::Feather,
            ];
            Box::leak(list.into_boxed_slice())
        }),
        4 => *TAB_4.get_or_init(|| {
            let mut list = vec![
                // Natural & Terrain
                BlockId::Grass,
                BlockId::Dirt,
                BlockId::Farmland,
                BlockId::FarmlandWet,
                BlockId::Stone,
                BlockId::Sand,
                BlockId::Mud,
                BlockId::Wood,
                BlockId::Leaves,
                BlockId::SnowLeaves,
                BlockId::CherryLeaves,
                BlockId::AutumnLeaves,
                BlockId::Sapling,
                BlockId::Cactus,
                BlockId::Snow,
                BlockId::Ice,
                BlockId::Water,
                BlockId::Lava,
                BlockId::Bedrock,
                // Ores
                BlockId::CoalOre,
                BlockId::IronOre,
                BlockId::GoldOre,
                BlockId::DiamondOre,
                BlockId::EmeraldOre,
                BlockId::RustStoneOre,
                // Mineral Blocks
                BlockId::CoalBlock,
                BlockId::IronBlock,
                BlockId::GoldBlock,
                BlockId::DiamondBlock,
                BlockId::EmeraldBlock,
                BlockId::RustStoneBlock,
                // Construction & Building
                BlockId::Planks,
                BlockId::WoodenSlab,
                BlockId::StoneSlab,
                BlockId::WoodenStairs,
                BlockId::StoneStairs,
                BlockId::Ladder,
                BlockId::Glass,
                BlockId::Door,
                // Functional Blocks
                BlockId::CraftingTable,
                BlockId::Furnace,
                BlockId::FurnaceLit,
                BlockId::Chest,
                BlockId::Bed,
                BlockId::Tnt,
                BlockId::Torch,
                // RustStone / Redstone Mechanisms
                BlockId::RustStone,
                BlockId::RustStoneWireLit,
                BlockId::RustStoneLamp,
                BlockId::RustStoneLampLit,
                BlockId::Lever,
                BlockId::LeverOn,
                BlockId::Button,
                BlockId::ButtonPressed,
                BlockId::PressurePlate,
                BlockId::PressurePlatePressed,
                // Pickaxes
                BlockId::WoodenPickaxe,
                BlockId::StonePickaxe,
                BlockId::IronPickaxe,
                BlockId::GoldPickaxe,
                BlockId::DiamondPickaxe,
                // Axes
                BlockId::WoodenAxe,
                BlockId::StoneAxe,
                BlockId::IronAxe,
                BlockId::GoldAxe,
                BlockId::DiamondAxe,
                // Shovels
                BlockId::WoodenShovel,
                BlockId::StoneShovel,
                BlockId::IronShovel,
                BlockId::GoldShovel,
                BlockId::DiamondShovel,
                // Swords
                BlockId::WoodenSword,
                BlockId::StoneSword,
                BlockId::IronSword,
                BlockId::GoldSword,
                BlockId::DiamondSword,
                // Hoes
                BlockId::WoodenHoe,
                BlockId::StoneHoe,
                BlockId::IronHoe,
                BlockId::GoldHoe,
                BlockId::DiamondHoe,
                BlockId::Shears,
                // Farming, Crops & Food
                BlockId::WheatSeeds,
                BlockId::Wheat,
                BlockId::WheatCrop,
                BlockId::Bread,
                BlockId::Carrot,
                BlockId::CarrotCrop,
                BlockId::Potato,
                BlockId::PotatoCrop,
                BlockId::BakedPotato,
                BlockId::Apple,
                // Materials & Minerals
                BlockId::Stick,
                BlockId::Bone,
                BlockId::BoneMeal,
                BlockId::Gunpowder,
                BlockId::Feather,
                BlockId::IronOunce,
                BlockId::IronIngot,
                BlockId::GoldOunce,
                BlockId::GoldIngot,
                BlockId::Diamond,
                BlockId::Emerald,
                // Animals & Meats
                BlockId::PigSpawnEgg,
                BlockId::CowSpawnEgg,
                BlockId::ChickenSpawnEgg,
                BlockId::SheepSpawnEgg,
                BlockId::VillagerSpawnEgg,
                BlockId::DolphinSpawnEgg,
                BlockId::FishSpawnEgg,
                BlockId::FishingRod,
                BlockId::RawCod,
                BlockId::CookedCod,
                BlockId::RawSalmon,
                BlockId::CookedSalmon,
                BlockId::Pufferfish,
                BlockId::TropicalFish,
                BlockId::DriedKelp,
                BlockId::Kelp,
                BlockId::Seagrass,
                BlockId::RawPorkchop,
                BlockId::CookedPorkchop,
                BlockId::RawBeef,
                BlockId::CookedBeef,
                BlockId::RawChicken,
                BlockId::CookedChicken,
                // Wool Blocks
                BlockId::WhiteWool,
                BlockId::OrangeWool,
                BlockId::MagentaWool,
                BlockId::LightBlueWool,
                BlockId::YellowWool,
                BlockId::LimeWool,
                BlockId::PinkWool,
                BlockId::GrayWool,
                BlockId::LightGrayWool,
                BlockId::CyanWool,
                BlockId::PurpleWool,
                BlockId::BlueWool,
                BlockId::BrownWool,
                BlockId::GreenWool,
                BlockId::RedWool,
                BlockId::BlackWool,
            ];
            list.extend(crate::engine::modding::get_custom_creative_blocks());
            list.extend(crate::engine::modding::get_custom_creative_items());
            Box::leak(list.into_boxed_slice())
        }),
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
            // Check top tabs (6 tabs)
            for i in 0..CREATIVE_TABS.len() {
                let tab_x = self.start_x - 3 + i as i32 * 79;
                let tab_y = self.start_y - 24;
                if px >= tab_x && px < tab_x + 75 && py >= tab_y && py < tab_y + 26 {
                    return Some(InventorySlotTarget::CreativeTab(i));
                }
            }

            if player.creative_tab < CREATIVE_TABS.len() - 1 {
                // Creative item grid (9 cols x 4 rows) with scroll offset
                let blocks = creative_tab_blocks(player.creative_tab);
                let total_rows = (blocks.len() + 8) / 9;
                let max_scroll = total_rows.saturating_sub(4);
                let scroll_row = player.creative_scroll.min(max_scroll);
                let scroll_offset = scroll_row * 9;

                let grid_x = self.start_x + 22;
                let grid_y = self.start_y + 36;
                for row in 0..4 {
                    for col in 0..9 {
                        let idx = scroll_offset + row * 9 + col;
                        if idx < blocks.len() {
                            let x = grid_x + col as i32 * 42;
                            let y = grid_y + row as i32 * 42;
                            if px >= x && px < x + 38 && py >= y && py < y + 38 {
                                return Some(InventorySlotTarget::Creative(blocks[idx].to_u8()));
                            }
                        }
                    }
                }

                // Scrollbar Up and Down buttons
                let sb_x = self.start_x + 404;
                if px >= sb_x && px < sb_x + 18 {
                    if py >= self.start_y + 36 && py < self.start_y + 54 {
                        return Some(InventorySlotTarget::CreativeScrollUp);
                    }
                    if py >= self.start_y + 186 && py < self.start_y + 204 {
                        return Some(InventorySlotTarget::CreativeScrollDown);
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

/// Renders an animated Minecraft-style Steve player model inside the inventory character preview window.
///
/// Features dynamic head tracking toward cursor position, iconic cyan shirt, blue jeans,
/// hair, skin, eyes, and ground drop shadow.
pub fn draw_player_inventory_preview(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    mouse_pos: Option<(f32, f32)>,
) {
    // 1. Shadow on ground
    let ground_y = center_y + 44;
    fill_rect_alpha(buffer, width, height, center_x - 22, ground_y, 44, 4, pack_color(0.04, 0.04, 0.06), 0.70);
    fill_rect_alpha(buffer, width, height, center_x - 17, ground_y - 1, 34, 6, pack_color(0.02, 0.02, 0.03), 0.55);

    // 2. Mouse tracking gaze
    let (look_x, look_y) = if let Some((mx, my)) = mouse_pos {
        let dx = (mx - center_x as f32) / 110.0;
        let dy = (my - (center_y - 20) as f32) / 110.0;
        (dx.clamp(-1.0, 1.0) * 3.0, dy.clamp(-1.0, 1.0) * 2.0)
    } else {
        (0.6, -0.4)
    };
    let lx = look_x.round() as i32;
    let ly = look_y.round() as i32;

    // Palette
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

    // 3. LEGS (Nogi) - 2x 9px wide x 28px tall
    let legs_y = center_y + 16;
    // Left leg (z perspektywy ekranu po lewej)
    fill_rect(buffer, width, height, center_x - 10, legs_y, 9, 22, pants_blue);
    fill_rect(buffer, width, height, center_x - 10, legs_y, 2, 22, pants_highlight);
    fill_rect(buffer, width, height, center_x - 3, legs_y, 2, 22, pants_shadow);
    // Left shoe
    fill_rect(buffer, width, height, center_x - 10, legs_y + 22, 9, 6, shoes_gray);
    fill_rect(buffer, width, height, center_x - 10, legs_y + 26, 9, 2, shoes_dark);

    // Right leg (z perspektywy ekranu po prawej)
    fill_rect(buffer, width, height, center_x + 1, legs_y, 9, 22, pants_blue);
    fill_rect(buffer, width, height, center_x + 1, legs_y, 2, 22, pants_highlight);
    fill_rect(buffer, width, height, center_x + 8, legs_y, 2, 22, pants_shadow);
    // Right shoe
    fill_rect(buffer, width, height, center_x + 1, legs_y + 22, 9, 6, shoes_gray);
    fill_rect(buffer, width, height, center_x + 1, legs_y + 26, 9, 2, shoes_dark);

    // 4. TORSO (Tułów) - 20px wide x 28px tall
    let torso_x = center_x - 10;
    let torso_y = center_y - 12;
    fill_rect(buffer, width, height, torso_x, torso_y, 20, 28, shirt_cyan);
    fill_rect(buffer, width, height, torso_x, torso_y, 2, 28, shirt_highlight);
    fill_rect(buffer, width, height, torso_x + 18, torso_y, 2, 28, shirt_shadow);
    fill_rect(buffer, width, height, torso_x, torso_y + 26, 20, 2, shirt_dark);
    // Neck cut-out (dekolt pokazujący skórę)
    fill_rect(buffer, width, height, torso_x + 7, torso_y, 6, 4, skin_base);
    fill_rect(buffer, width, height, torso_x + 8, torso_y + 4, 4, 2, skin_shadow);

    // 5. ARMS (Ręce) - each 8px wide x 28px tall
    // Left arm
    let left_arm_x = torso_x - 9;
    let left_arm_y = torso_y;
    // Sleeve
    fill_rect(buffer, width, height, left_arm_x, left_arm_y, 8, 9, shirt_cyan);
    fill_rect(buffer, width, height, left_arm_x, left_arm_y, 2, 9, shirt_highlight);
    fill_rect(buffer, width, height, left_arm_x + 6, left_arm_y, 2, 9, shirt_shadow);
    // Forearm & Hand
    fill_rect(buffer, width, height, left_arm_x, left_arm_y + 9, 8, 19, skin_base);
    fill_rect(buffer, width, height, left_arm_x, left_arm_y + 9, 2, 19, skin_highlight);
    fill_rect(buffer, width, height, left_arm_x + 6, left_arm_y + 9, 2, 19, skin_shadow);

    // Right arm
    let right_arm_x = torso_x + 21;
    let right_arm_y = torso_y;
    // Sleeve
    fill_rect(buffer, width, height, right_arm_x, right_arm_y, 8, 9, shirt_cyan);
    fill_rect(buffer, width, height, right_arm_x, right_arm_y, 2, 9, shirt_highlight);
    fill_rect(buffer, width, height, right_arm_x + 6, right_arm_y, 2, 9, shirt_shadow);
    // Forearm & Hand
    fill_rect(buffer, width, height, right_arm_x, right_arm_y + 9, 8, 19, skin_base);
    fill_rect(buffer, width, height, right_arm_x, right_arm_y + 9, 2, 19, skin_highlight);
    fill_rect(buffer, width, height, right_arm_x + 6, right_arm_y + 9, 2, 19, skin_shadow);

    // 6. HEAD (Głowa) - 20px wide x 20px tall, obraca się w stronę kursora
    let head_x = center_x - 10 + lx;
    let head_y = torso_y - 20 + ly;

    // Head base skin
    fill_rect(buffer, width, height, head_x, head_y, 20, 20, skin_base);

    // Hair top (grzywka i włosy)
    fill_rect(buffer, width, height, head_x, head_y, 20, 6, hair_base);
    fill_rect(buffer, width, height, head_x, head_y, 20, 2, hair_dark);
    fill_rect(buffer, width, height, head_x, head_y + 6, 3, 5, hair_base);
    fill_rect(buffer, width, height, head_x + 17, head_y + 6, 3, 5, hair_base);
    fill_rect(buffer, width, height, head_x + 6, head_y + 6, 2, 2, hair_base);

    // Eyes (Oczy z białkami i niebieskimi tęczówkami)
    let eye_y = head_y + 9;
    let eye_dx = lx.clamp(-1, 1);
    fill_rect(buffer, width, height, head_x + 3, eye_y, 5, 3, eye_white);
    fill_rect(buffer, width, height, head_x + 5 + eye_dx, eye_y, 2, 3, eye_blue);

    fill_rect(buffer, width, height, head_x + 12, eye_y, 5, 3, eye_white);
    fill_rect(buffer, width, height, head_x + 12 + eye_dx, eye_y, 2, 3, eye_blue);

    // Nose (Nos)
    let nose_x = head_x + 9 + eye_dx;
    let nose_y = head_y + 12;
    fill_rect(buffer, width, height, nose_x, nose_y, 3, 2, skin_shadow);

    // Mouth / Beard (Usta i zarost Steve'a)
    fill_rect(buffer, width, height, head_x + 7 + eye_dx, head_y + 15, 6, 3, mouth_color);
    fill_rect(buffer, width, height, head_x + 8 + eye_dx, head_y + 15, 4, 1, pack_color(0.50, 0.30, 0.20));
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
    is_gamepad_active: bool,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.55);

    let layout = InventoryLayout::new(width, height, crafting_table_open);
    let start_x = layout.start_x;
    let start_y = layout.start_y;

    if player.gamemode == GameMode::Creative {
        // Draw 6 Top Tabs
        for i in 0..CREATIVE_TABS.len() {
            let tab_x = start_x - 3 + i as i32 * 79;
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

            fill_rect(buffer, width, height, tab_x, tab_y, 75, 26, bg_color);
            fill_rect(buffer, width, height, tab_x, tab_y, 75, 2, border_top);
            fill_rect(buffer, width, height, tab_x, tab_y, 2, 26, border_top);
            fill_rect(buffer, width, height, tab_x + 73, tab_y, 2, 26, pack_color(0.06, 0.06, 0.08));

            let text_color = if is_active {
                pack_color(1.0, 0.95, 0.3)
            } else if is_hovered {
                pack_color(0.9, 0.9, 0.9)
            } else {
                pack_color(0.55, 0.55, 0.60)
            };

            let label = CREATIVE_TABS[i];
            let tw = crate::graphics::font::measure_text(label, 1, 1);
            let tx = tab_x + (75 - tw) / 2;
            let ty = tab_y + 8;
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, 1, 1, text_color);
        }

        // Main creative window frame
        fill_rect_alpha(buffer, width, height, start_x - 8, start_y, 480, 360, pack_color(0.14, 0.14, 0.16), 0.98);
        fill_rect(buffer, width, height, start_x - 8, start_y, 480, 3, pack_color(0.38, 0.38, 0.42));
        fill_rect(buffer, width, height, start_x - 8, start_y + 357, 480, 3, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x - 8, start_y, 3, 360, pack_color(0.38, 0.38, 0.42));
        fill_rect(buffer, width, height, start_x + 469, start_y, 3, 360, pack_color(0.05, 0.05, 0.06));

        if player.creative_tab < CREATIVE_TABS.len() - 1 {
            // Category title
            let title = match player.creative_tab {
                0 => "BUILDING BLOCKS",
                1 => "TOOLS & COMBAT",
                2 => "RUSTSTONE & MECHANISMS",
                3 => "FARMING & CROPS",
                4 => "ALL ITEMS & BLOCKS",
                _ => "",
            };
            draw_text_shadowed(buffer, width, height, title, start_x + 18, start_y + 14, 2, 1);

            // Creative block grid (9 cols x 4 rows) with scroll offset
            let blocks = creative_tab_blocks(player.creative_tab);
            let total_rows = (blocks.len() + 8) / 9;
            let max_scroll = total_rows.saturating_sub(4);
            let scroll_row = player.creative_scroll.min(max_scroll);
            let scroll_offset = scroll_row * 9;

            let grid_x = start_x + 22;
            let grid_y = start_y + 36;
            for row in 0..4 {
                for col in 0..9 {
                    let idx = scroll_offset + row * 9 + col;
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

            // Scrollbar for creative item grid
            let sb_x = start_x + 404;
            let is_up_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollUp);
            let is_down_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollDown);

            // Up button
            let up_bg = if is_up_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
            fill_rect(buffer, width, height, sb_x, start_y + 36, 18, 16, up_bg);
            fill_rect(buffer, width, height, sb_x, start_y + 36, 18, 1, pack_color(0.4, 0.4, 0.45));
            draw_text_colored_shadowed(buffer, width, height, "^", sb_x + 6, start_y + 40, 1, 1, pack_color(0.9, 0.9, 0.95));

            // Track
            let track_y = start_y + 54;
            let track_h = 130;
            fill_rect(buffer, width, height, sb_x, track_y, 18, track_h, pack_color(0.10, 0.10, 0.12));
            fill_rect(buffer, width, height, sb_x, track_y, 1, track_h, pack_color(0.06, 0.06, 0.08));
            fill_rect(buffer, width, height, sb_x + 17, track_y, 1, track_h, pack_color(0.22, 0.22, 0.26));

            // Thumb
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
            fill_rect(buffer, width, height, sb_x + 2, thumb_y, 14, thumb_h, pack_color(0.38, 0.38, 0.44));
            fill_rect(buffer, width, height, sb_x + 2, thumb_y, 14, 1, pack_color(0.55, 0.55, 0.62));
            fill_rect(buffer, width, height, sb_x + 2, thumb_y, 1, thumb_h, pack_color(0.55, 0.55, 0.62));
            fill_rect(buffer, width, height, sb_x + 15, thumb_y, 1, thumb_h, pack_color(0.20, 0.20, 0.24));
            fill_rect(buffer, width, height, sb_x + 2, thumb_y + thumb_h - 1, 14, 1, pack_color(0.20, 0.20, 0.24));

            // Down button
            let down_bg = if is_down_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
            fill_rect(buffer, width, height, sb_x, start_y + 186, 18, 16, down_bg);
            fill_rect(buffer, width, height, sb_x, start_y + 186, 18, 1, pack_color(0.4, 0.4, 0.45));
            draw_text_colored_shadowed(buffer, width, height, "v", sb_x + 6, start_y + 190, 1, 1, pack_color(0.9, 0.9, 0.95));

            // Row counter
            if total_rows > 4 {
                let row_info = format!("{}/{}", scroll_row + 1, max_scroll + 1);
                let rw = crate::graphics::font::measure_text(&row_info, 1, 1);
                draw_text_colored_shadowed(buffer, width, height, &row_info, sb_x + 18 - rw, start_y + 206, 1, 1, pack_color(0.60, 0.60, 0.65));
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
            let creative_footer = if is_gamepad_active {
                "[A] TAKE (64) | [Y] TO HOTBAR | [X] DELETE | [B] CLOSE"
            } else {
                "LMB: TAKE (64) | SHIFT+LMB: TO HOTBAR | 1-0: SELECT SLOT | [E]/[ESC]: CLOSE"
            };
            draw_text_colored_shadowed(
                buffer, width, height,
                creative_footer,
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
        } else {
            // Tab 3: Survival Inventory inside Creative
            draw_text_shadowed(buffer, width, height, "SURVIVAL INVENTORY", start_x + 18, start_y + 14, 2, 1);

            // Character Preview Box
            let box_x = start_x + 18;
            let box_y = start_y + 34;
            let box_w = 180;
            let box_h = 104;
            fill_rect_alpha(buffer, width, height, box_x, box_y, box_w, box_h, pack_color(0.08, 0.08, 0.10), 0.95);
            fill_rect(buffer, width, height, box_x, box_y, box_w, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, box_x, box_y, 2, box_h, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, box_x, box_y + box_h - 2, box_w, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, box_x + box_w - 2, box_y, 2, box_h, pack_color(0.26, 0.26, 0.30));

            // Character floor shadow & model
            fill_rect_alpha(buffer, width, height, box_x + 12, box_y + 86, 64, 12, pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(buffer, width, height, box_x + 44, box_y + 54, mouse_pos);

            draw_text_colored_shadowed(buffer, width, height, "PLAYER", box_x + 85, box_y + 16, 1, 1, pack_color(1.0, 0.88, 0.25));
            draw_text_colored_shadowed(buffer, width, height, "CREATIVE", box_x + 85, box_y + 38, 1, 1, pack_color(0.4, 0.9, 0.4));
            draw_text_colored_shadowed(buffer, width, height, "FLYING: ON", box_x + 85, box_y + 58, 1, 1, pack_color(0.4, 0.8, 1.0));
            draw_text_colored_shadowed(buffer, width, height, "GODMODE", box_x + 85, box_y + 78, 1, 1, pack_color(0.9, 0.7, 0.3));

            // 2x2 Crafting Grid
            let craft_panel_x = start_x + 208;
            let craft_panel_y = start_y + 34;
            fill_rect_alpha(buffer, width, height, craft_panel_x, craft_panel_y, 242, 104, pack_color(0.10, 0.10, 0.12), 0.85);
            fill_rect(buffer, width, height, craft_panel_x, craft_panel_y, 242, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_panel_x, craft_panel_y, 2, 104, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_panel_x, craft_panel_y + 102, 242, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, craft_panel_x + 240, craft_panel_y, 2, 104, pack_color(0.26, 0.26, 0.30));

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

            // Main 27 slots inset panel
            fill_rect_alpha(buffer, width, height, start_x + 14, start_y + 140, 436, 126, pack_color(0.10, 0.10, 0.12), 0.85);
            fill_rect(buffer, width, height, start_x + 14, start_y + 140, 436, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, start_x + 14, start_y + 140, 2, 126, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, start_x + 14, start_y + 264, 436, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, start_x + 448, start_y + 140, 2, 126, pack_color(0.26, 0.26, 0.30));

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

            // Hotbar & Trash inset panel
            fill_rect_alpha(buffer, width, height, start_x + 10, start_y + 270, 444, 52, pack_color(0.10, 0.10, 0.12), 0.85);
            fill_rect(buffer, width, height, start_x + 10, start_y + 270, 444, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, start_x + 10, start_y + 270, 2, 52, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, start_x + 10, start_y + 320, 444, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, start_x + 452, start_y + 270, 2, 52, pack_color(0.26, 0.26, 0.30));

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
            let footer = if is_gamepad_active {
                "[A]: ALL | [X]: 1 ITEM/HALF | [Y]: MOVE | [B]: CLOSE"
            } else {
                "LPM: ALL | PPM: 1 ITEM/HALF | SHIFT+LPM: MOVE | [E]/[ESC]: CLOSE"
            };
            draw_text_colored_shadowed(
                buffer, width, height,
                footer,
                start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75)
            );
        }
    } else {
        // Survival Mode
        let win_x = start_x - 8;
        let win_y = start_y;
        let win_w = 480;
        let win_h = 360;

        // Main window panel background
        fill_rect_alpha(buffer, width, height, win_x, win_y, win_w, win_h, pack_color(0.15, 0.15, 0.18), 0.98);
        // Beveled window borders
        fill_rect(buffer, width, height, win_x, win_y, win_w, 3, pack_color(0.38, 0.38, 0.44));
        fill_rect(buffer, width, height, win_x, win_y, 3, win_h, pack_color(0.38, 0.38, 0.44));
        fill_rect(buffer, width, height, win_x, win_y + win_h - 3, win_w, 3, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, win_x + win_w - 3, win_y, 3, win_h, pack_color(0.05, 0.05, 0.06));
        // Inner highlight
        fill_rect(buffer, width, height, win_x + 3, win_y + 3, win_w - 6, 1, pack_color(0.48, 0.48, 0.54));

        // Window Header Bar
        fill_rect(buffer, width, height, win_x + 3, win_y + 3, win_w - 6, 25, pack_color(0.11, 0.11, 0.14));
        fill_rect(buffer, width, height, win_x + 3, win_y + 27, win_w - 6, 1, pack_color(0.07, 0.07, 0.09));

        let title = if crafting_table_open { "CRAFTING TABLE" } else { "INVENTORY" };
        draw_text_colored_shadowed(buffer, width, height, title, win_x + 14, win_y + 7, 2, 1, pack_color(1.0, 0.88, 0.25));
        draw_text_colored_shadowed(buffer, width, height, "[SURVIVAL]", win_x + win_w - 90, win_y + 9, 1, 1, pack_color(0.45, 0.85, 0.50));

        let surv_footer = if is_gamepad_active {
            "[A]: ALL | [X]: 1 ITEM/HALF | [Y]: QUICK MOVE | [B]: CLOSE"
        } else {
            "LMB: ALL | RMB: 1 ITEM / HALF | [E]/[ESC]/[B]: CLOSE"
        };
        // Window Footer Bar
        fill_rect(buffer, width, height, win_x + 3, win_y + win_h - 22, win_w - 6, 19, pack_color(0.11, 0.11, 0.14));
        fill_rect(buffer, width, height, win_x + 3, win_y + win_h - 22, win_w - 6, 1, pack_color(0.07, 0.07, 0.09));
        draw_text_colored_shadowed(buffer, width, height, surv_footer, win_x + 12, win_y + win_h - 16, 1, 1, pack_color(0.70, 0.70, 0.75));

        // Crafting section
        if crafting_table_open {
            // 3x3 crafting inset panel
            let craft_box_x = start_x + 14;
            let craft_box_y = start_y + 30;
            let craft_box_w = 276;
            let craft_box_h = 108;
            fill_rect_alpha(buffer, width, height, craft_box_x, craft_box_y, craft_box_w, craft_box_h, pack_color(0.10, 0.10, 0.12), 0.85);
            fill_rect(buffer, width, height, craft_box_x, craft_box_y, craft_box_w, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_box_x, craft_box_y, 2, craft_box_h, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_box_x, craft_box_y + craft_box_h - 2, craft_box_w, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, craft_box_x + craft_box_w - 2, craft_box_y, 2, craft_box_h, pack_color(0.26, 0.26, 0.30));

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

            // Character preview box on the right of crafting table
            let char_box_x = start_x + 300;
            let char_box_y = start_y + 30;
            let char_box_w = 150;
            let char_box_h = 108;
            fill_rect_alpha(buffer, width, height, char_box_x, char_box_y, char_box_w, char_box_h, pack_color(0.08, 0.08, 0.10), 0.95);
            fill_rect(buffer, width, height, char_box_x, char_box_y, char_box_w, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, char_box_x, char_box_y, 2, char_box_h, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, char_box_x, char_box_y + char_box_h - 2, char_box_w, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, char_box_x + char_box_w - 2, char_box_y, 2, char_box_h, pack_color(0.26, 0.26, 0.30));

            fill_rect_alpha(buffer, width, height, char_box_x + 10, char_box_y + 88, 55, 12, pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(buffer, width, height, char_box_x + 38, char_box_y + 54, mouse_pos);

            draw_text_colored_shadowed(buffer, width, height, "PLAYER", char_box_x + 72, char_box_y + 10, 1, 1, pack_color(1.0, 0.88, 0.25));
            let hp_text = format!("HP: {}/20", player.health as i32);
            draw_text_colored_shadowed(buffer, width, height, &hp_text, char_box_x + 72, char_box_y + 24, 1, 1, pack_color(1.0, 0.35, 0.35));
            let hp_ratio = (player.health / 20.0).clamp(0.0, 1.0);
            fill_rect(buffer, width, height, char_box_x + 72, char_box_y + 36, 70, 4, pack_color(0.25, 0.05, 0.05));
            fill_rect(buffer, width, height, char_box_x + 72, char_box_y + 36, (70.0 * hp_ratio) as i32, 4, pack_color(0.9, 0.2, 0.2));

            let hunger_text = format!("HUNGER: {}/20", player.hunger as i32);
            draw_text_colored_shadowed(buffer, width, height, &hunger_text, char_box_x + 72, char_box_y + 46, 1, 1, pack_color(1.0, 0.65, 0.2));
            let hunger_ratio = (player.hunger / player.max_hunger.max(1.0)).clamp(0.0, 1.0);
            fill_rect(buffer, width, height, char_box_x + 72, char_box_y + 58, 70, 4, pack_color(0.25, 0.15, 0.05));
            fill_rect(buffer, width, height, char_box_x + 72, char_box_y + 58, (70.0 * hunger_ratio) as i32, 4, pack_color(0.9, 0.55, 0.15));

            if player.fire_timer > 0.0 {
                draw_text_colored_shadowed(buffer, width, height, "BURNING!", char_box_x + 72, char_box_y + 70, 1, 1, pack_color(1.0, 0.5, 0.1));
            } else {
                draw_text_colored_shadowed(buffer, width, height, "STATUS: OK", char_box_x + 72, char_box_y + 70, 1, 1, pack_color(0.4, 0.85, 0.4));
            }
        } else {
            // Player character info area
            let char_box_x = start_x + 14;
            let char_box_y = start_y + 32;
            let char_box_w = 186;
            let char_box_h = 106;
            fill_rect_alpha(buffer, width, height, char_box_x, char_box_y, char_box_w, char_box_h, pack_color(0.08, 0.08, 0.10), 0.95);
            fill_rect(buffer, width, height, char_box_x, char_box_y, char_box_w, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, char_box_x, char_box_y, 2, char_box_h, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, char_box_x, char_box_y + char_box_h - 2, char_box_w, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, char_box_x + char_box_w - 2, char_box_y, 2, char_box_h, pack_color(0.26, 0.26, 0.30));

            // Floor pedestal under player & model preview
            fill_rect_alpha(buffer, width, height, char_box_x + 12, char_box_y + 88, 66, 12, pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(buffer, width, height, char_box_x + 45, char_box_y + 54, mouse_pos);

            draw_text_colored_shadowed(buffer, width, height, "PLAYER", char_box_x + 85, char_box_y + 10, 1, 1, pack_color(1.0, 0.88, 0.25));
            let hp_text = format!("HP: {}/20", player.health as i32);
            draw_text_colored_shadowed(buffer, width, height, &hp_text, char_box_x + 85, char_box_y + 24, 1, 1, pack_color(1.0, 0.35, 0.35));
            let hp_ratio = (player.health / 20.0).clamp(0.0, 1.0);
            fill_rect(buffer, width, height, char_box_x + 85, char_box_y + 36, 85, 4, pack_color(0.25, 0.05, 0.05));
            fill_rect(buffer, width, height, char_box_x + 85, char_box_y + 36, (85.0 * hp_ratio) as i32, 4, pack_color(0.9, 0.2, 0.2));

            let hunger_text = format!("HUNGER: {}/20", player.hunger as i32);
            draw_text_colored_shadowed(buffer, width, height, &hunger_text, char_box_x + 85, char_box_y + 46, 1, 1, pack_color(1.0, 0.65, 0.2));
            let hunger_ratio = (player.hunger / player.max_hunger.max(1.0)).clamp(0.0, 1.0);
            fill_rect(buffer, width, height, char_box_x + 85, char_box_y + 58, 85, 4, pack_color(0.25, 0.15, 0.05));
            fill_rect(buffer, width, height, char_box_x + 85, char_box_y + 58, (85.0 * hunger_ratio) as i32, 4, pack_color(0.9, 0.55, 0.15));

            if player.fire_timer > 0.0 {
                draw_text_colored_shadowed(buffer, width, height, "BURNING!", char_box_x + 85, char_box_y + 70, 1, 1, pack_color(1.0, 0.5, 0.1));
            } else {
                draw_text_colored_shadowed(buffer, width, height, "STATUS: OK", char_box_x + 85, char_box_y + 70, 1, 1, pack_color(0.4, 0.85, 0.4));
            }

            // 2x2 Crafting inset panel
            let craft_box_x = start_x + 208;
            let craft_box_y = start_y + 32;
            let craft_box_w = 242;
            let craft_box_h = 106;
            fill_rect_alpha(buffer, width, height, craft_box_x, craft_box_y, craft_box_w, craft_box_h, pack_color(0.10, 0.10, 0.12), 0.85);
            fill_rect(buffer, width, height, craft_box_x, craft_box_y, craft_box_w, 2, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_box_x, craft_box_y, 2, craft_box_h, pack_color(0.05, 0.05, 0.06));
            fill_rect(buffer, width, height, craft_box_x, craft_box_y + craft_box_h - 2, craft_box_w, 2, pack_color(0.26, 0.26, 0.30));
            fill_rect(buffer, width, height, craft_box_x + craft_box_w - 2, craft_box_y, 2, craft_box_h, pack_color(0.26, 0.26, 0.30));

            draw_text_colored_shadowed(buffer, width, height, "CRAFTING (2x2)", craft_box_x + 12, craft_box_y + 8, 1, 1, pack_color(0.85, 0.85, 0.85));
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

        // Main Inventory Inset Panel
        fill_rect_alpha(buffer, width, height, start_x + 14, start_y + 144, 436, 134, pack_color(0.10, 0.10, 0.12), 0.85);
        fill_rect(buffer, width, height, start_x + 14, start_y + 144, 436, 2, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x + 14, start_y + 144, 2, 134, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x + 14, start_y + 276, 436, 2, pack_color(0.26, 0.26, 0.30));
        fill_rect(buffer, width, height, start_x + 448, start_y + 144, 2, 134, pack_color(0.26, 0.26, 0.30));

        draw_text_colored_shadowed(buffer, width, height, "BACKPACK", start_x + 22, start_y + 146, 1, 1, pack_color(0.75, 0.75, 0.80));
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

        // Hotbar Inset Panel
        fill_rect_alpha(buffer, width, height, start_x + 10, start_y + 282, 444, 52, pack_color(0.10, 0.10, 0.12), 0.85);
        fill_rect(buffer, width, height, start_x + 10, start_y + 282, 444, 2, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x + 10, start_y + 282, 2, 52, pack_color(0.05, 0.05, 0.06));
        fill_rect(buffer, width, height, start_x + 10, start_y + 332, 444, 2, pack_color(0.26, 0.26, 0.30));
        fill_rect(buffer, width, height, start_x + 452, start_y + 282, 2, 52, pack_color(0.26, 0.26, 0.30));

        draw_text_colored_shadowed(buffer, width, height, "QUICK ACCESS HOTBAR", start_x + 16, start_y + 284, 1, 1, pack_color(0.85, 0.85, 0.40));
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
            InventorySlotTarget::Hotbar(i) => hovered_item_name = player.inventory.hotbar[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Main(i) => hovered_item_name = player.inventory.main[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Crafting2x2(i) => hovered_item_name = player.inventory.crafting_2x2[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Crafting3x3(i) => hovered_item_name = player.inventory.crafting_3x3[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::CraftingOutput => {
                let res = if crafting_table_open {
                    recipes.match_grid(&player.inventory.crafting_3x3, 3, 3)
                } else {
                    recipes.match_grid(&player.inventory.crafting_2x2, 2, 2)
                };
                hovered_item_name = res.map(|s| (s.block.display_name().to_string(), s.count));
            }
            InventorySlotTarget::Creative(id) => {
                let block = BlockId::from_u8(id);
                hovered_item_name = Some((block.display_name().to_string(), 64));
            }
            InventorySlotTarget::CreativeTab(idx) => {
                if idx < CREATIVE_TABS.len() {
                    hovered_item_name = Some((format!("Category: {}", CREATIVE_TABS[idx]), 1));
                }
            }
            InventorySlotTarget::CreativeScrollUp => {
                hovered_item_name = Some(("Scroll Up (Mouse Wheel Up)".to_string(), 1));
            }
            InventorySlotTarget::CreativeScrollDown => {
                hovered_item_name = Some(("Scroll Down (Mouse Wheel Down)".to_string(), 1));
            }
            InventorySlotTarget::Trash => {
                let trash_tip = if is_gamepad_active {
                    "TRASH ([A]: Delete held item | [X]: Clear hotbar)"
                } else {
                    "TRASH (LMB: Delete held item | Shift+LMB: Clear hotbar)"
                };
                hovered_item_name = Some((trash_tip.to_string(), 1));
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
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => (
                pack_color(0.85, 0.68, 0.42), // highlight
                pack_color(0.68, 0.50, 0.30), // base
                pack_color(0.48, 0.32, 0.16), // shadow
                pack_color(0.32, 0.20, 0.10), // outline
            ),
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => (
                pack_color(0.78, 0.78, 0.80), // highlight
                pack_color(0.58, 0.58, 0.60), // base
                pack_color(0.40, 0.40, 0.42), // shadow
                pack_color(0.24, 0.24, 0.26), // outline
            ),
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => (
                pack_color(1.00, 0.95, 0.40), // glint
                pack_color(0.95, 0.80, 0.20), // highlight
                pack_color(0.80, 0.65, 0.10), // base
                pack_color(0.50, 0.38, 0.05), // shadow
            ),
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => (
                pack_color(0.75, 1.00, 1.00), // glint
                pack_color(0.35, 0.90, 0.90), // highlight
                pack_color(0.18, 0.72, 0.75), // base
                pack_color(0.08, 0.45, 0.48), // shadow
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
        BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::GoldSword | BlockId::DiamondSword => {
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
        BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe | BlockId::GoldPickaxe | BlockId::DiamondPickaxe => {
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
        BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe | BlockId::GoldAxe | BlockId::DiamondAxe => {
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
        BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel | BlockId::GoldShovel | BlockId::DiamondShovel => {
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
        BlockId::WoodenHoe | BlockId::StoneHoe | BlockId::IronHoe | BlockId::GoldHoe | BlockId::DiamondHoe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            // Handle
            for i in 2..=10 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
                px(buffer, i, 16 - i, wood_dark);
            }
            // Hoe Head (Right-facing blade)
            px(buffer, 9, 2, c_glint);
            px(buffer, 10, 2, c_glint);
            px(buffer, 11, 2, c_hi);
            px(buffer, 12, 2, c_base);
            px(buffer, 12, 3, c_shadow);
            px(buffer, 12, 4, c_shadow);
            px(buffer, 11, 3, c_base);
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
        BlockId::RawChicken => {
            let outline = pack_color(0.46, 0.24, 0.20);
            let base = pack_color(0.92, 0.68, 0.62);
            let hi = pack_color(0.98, 0.82, 0.78);
            let sh = pack_color(0.76, 0.46, 0.40);
            let bone = pack_color(0.95, 0.95, 0.88);
            let bone_sh = pack_color(0.78, 0.76, 0.68);

            // Bone tip top-right
            px(buffer, 12, 4, bone);
            px(buffer, 13, 3, bone);
            px(buffer, 14, 2, bone);
            px(buffer, 14, 3, bone_sh);
            px(buffer, 13, 2, bone);
            px(buffer, 12, 3, bone);
            px(buffer, 11, 4, outline);
            px(buffer, 14, 4, outline);
            px(buffer, 15, 2, outline);
            px(buffer, 13, 1, outline);

            // Chicken drumstick meat body
            for ly in 4..=13 {
                for lx in 2..=11 {
                    let in_shape = (lx + ly >= 8) && (lx + ly <= 21) && (ly - lx <= 7) && (lx - ly <= 4);
                    if in_shape {
                        let is_border = lx == 2 || lx == 11 || ly == 4 || ly == 13 || (lx + ly == 8) || (lx + ly == 21);
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
        BlockId::CookedChicken => {
            let outline = pack_color(0.30, 0.12, 0.04);
            let base = pack_color(0.82, 0.46, 0.14);
            let hi = pack_color(0.96, 0.72, 0.28);
            let sh = pack_color(0.55, 0.24, 0.06);
            let bone = pack_color(0.92, 0.90, 0.82);
            let bone_sh = pack_color(0.72, 0.70, 0.62);

            // Bone tip top-right
            px(buffer, 12, 4, bone);
            px(buffer, 13, 3, bone);
            px(buffer, 14, 2, bone);
            px(buffer, 14, 3, bone_sh);
            px(buffer, 13, 2, bone);
            px(buffer, 12, 3, bone);
            px(buffer, 11, 4, outline);
            px(buffer, 14, 4, outline);
            px(buffer, 15, 2, outline);
            px(buffer, 13, 1, outline);

            // Roasted drumstick meat body
            for ly in 4..=13 {
                for lx in 2..=11 {
                    let in_shape = (lx + ly >= 8) && (lx + ly <= 21) && (ly - lx <= 7) && (lx - ly <= 4);
                    if in_shape {
                        let is_border = lx == 2 || lx == 11 || ly == 4 || ly == 13 || (lx + ly == 8) || (lx + ly == 21);
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
        BlockId::Feather => {
            let outline = pack_color(0.25, 0.25, 0.28);
            let white = pack_color(0.98, 0.98, 0.99);
            let gray = pack_color(0.82, 0.84, 0.88);
            let quill = pack_color(0.68, 0.62, 0.55);
            let quill_base = pack_color(0.50, 0.44, 0.38);

            // Feather shape diagonal from bottom-left (2, 13) to top-right (13, 2)
            // Quill shaft
            px(buffer, 2, 14, quill_base);
            px(buffer, 3, 13, quill_base);
            px(buffer, 4, 12, quill);
            px(buffer, 5, 11, quill);
            px(buffer, 6, 10, quill);
            px(buffer, 7, 9, quill);
            px(buffer, 8, 8, quill);
            px(buffer, 9, 7, quill);
            px(buffer, 10, 6, quill);
            px(buffer, 11, 5, quill);
            px(buffer, 12, 4, quill);
            px(buffer, 13, 3, quill);

            // Vanes
            for y in 2..=13 {
                for x in 2..=14 {
                    let diag = (x as i32) - (15 - y as i32);
                    if diag >= -2 && diag <= 3 && y >= 3 && y <= 11 && x >= 4 && x <= 13 {
                        let is_edge = diag == 3 || diag == -2 || y == 3 || x == 13;
                        if is_edge {
                            px(buffer, x, y, outline);
                        } else if (x + y) % 2 == 0 {
                            px(buffer, x, y, white);
                        } else {
                            px(buffer, x, y, gray);
                        }
                    }
                }
            }
            // Quill tip outline
            px(buffer, 1, 14, outline);
            px(buffer, 2, 15, outline);
            px(buffer, 3, 14, outline);
        }
        BlockId::Shears => {
            let iron_hi = pack_color(0.95, 0.95, 0.98);
            let iron_mid = pack_color(0.75, 0.75, 0.78);
            let iron_dark = pack_color(0.45, 0.45, 0.48);
            let bolt = pack_color(0.30, 0.30, 0.32);

            // Pivot bolt
            px(buffer, 7, 8, bolt);
            px(buffer, 8, 8, iron_hi);
            px(buffer, 7, 9, iron_dark);
            px(buffer, 8, 9, bolt);

            // Left blade (top-left)
            px(buffer, 3, 3, iron_hi);
            px(buffer, 4, 3, iron_mid);
            px(buffer, 3, 4, iron_hi);
            px(buffer, 4, 4, iron_mid);
            px(buffer, 5, 4, iron_dark);
            px(buffer, 4, 5, iron_hi);
            px(buffer, 5, 5, iron_mid);
            px(buffer, 6, 6, iron_hi);
            px(buffer, 6, 7, iron_mid);

            // Right blade (top-right)
            px(buffer, 11, 3, iron_hi);
            px(buffer, 12, 3, iron_hi);
            px(buffer, 11, 4, iron_mid);
            px(buffer, 12, 4, iron_dark);
            px(buffer, 10, 4, iron_hi);
            px(buffer, 10, 5, iron_mid);
            px(buffer, 9, 5, iron_dark);
            px(buffer, 9, 6, iron_hi);
            px(buffer, 9, 7, iron_mid);

            // Handles (bottom)
            px(buffer, 6, 10, iron_dark);
            px(buffer, 5, 11, iron_mid);
            px(buffer, 4, 12, iron_mid);
            px(buffer, 5, 13, iron_dark);
            px(buffer, 6, 13, iron_dark);
            px(buffer, 6, 12, iron_hi);

            px(buffer, 9, 10, iron_dark);
            px(buffer, 10, 11, iron_mid);
            px(buffer, 11, 12, iron_mid);
            px(buffer, 10, 13, iron_dark);
            px(buffer, 9, 13, iron_dark);
            px(buffer, 9, 12, iron_hi);
        }
        BlockId::PigSpawnEgg | BlockId::CowSpawnEgg | BlockId::ChickenSpawnEgg | BlockId::SheepSpawnEgg | BlockId::DolphinSpawnEgg | BlockId::FishSpawnEgg | BlockId::VillagerSpawnEgg => {
            let (base, spot, outline) = match block {
                BlockId::PigSpawnEgg => (
                    pack_color(0.95, 0.62, 0.68),
                    pack_color(0.68, 0.32, 0.38),
                    pack_color(0.48, 0.20, 0.25),
                ),
                BlockId::CowSpawnEgg => (
                    pack_color(0.42, 0.30, 0.22),
                    pack_color(0.85, 0.85, 0.85),
                    pack_color(0.22, 0.15, 0.10),
                ),
                BlockId::ChickenSpawnEgg => (
                    pack_color(0.95, 0.95, 0.92),
                    pack_color(0.85, 0.15, 0.15),
                    pack_color(0.35, 0.35, 0.30),
                ),
                BlockId::SheepSpawnEgg => (
                    pack_color(0.92, 0.90, 0.88),
                    pack_color(0.85, 0.68, 0.60),
                    pack_color(0.40, 0.38, 0.36),
                ),
                BlockId::DolphinSpawnEgg => (
                    pack_color(0.22, 0.40, 0.62),
                    pack_color(0.85, 0.88, 0.95),
                    pack_color(0.12, 0.22, 0.35),
                ),
                BlockId::FishSpawnEgg => (
                    pack_color(0.68, 0.52, 0.32),
                    pack_color(0.92, 0.45, 0.22),
                    pack_color(0.38, 0.28, 0.16),
                ),
                BlockId::VillagerSpawnEgg => (
                    pack_color(0.55, 0.38, 0.24),
                    pack_color(0.78, 0.58, 0.45),
                    pack_color(0.32, 0.20, 0.12),
                ),
                _ => unreachable!(),
            };

            for ly in 2..=14 {
                for lx in 3..=12 {
                    let dx = (lx as f32 - 7.5).abs();
                    let dy = ly as f32;
                    let width_at_y = if dy < 6.0 {
                        (dy - 1.0) * 0.7 + 0.5
                    } else if dy <= 11.0 {
                        4.2
                    } else {
                        (14.5 - dy) * 1.1 + 0.5
                    };
                    if dx <= width_at_y {
                        let is_border = dx > width_at_y - 0.9 || ly == 2 || ly == 14;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else {
                            let is_spot = match block {
                                BlockId::PigSpawnEgg => (lx == 6 && ly == 5) || (lx == 9 && ly == 8) || (lx == 5 && ly == 10) || (lx == 8 && ly == 12),
                                BlockId::CowSpawnEgg => (lx == 6 && ly == 6) || (lx == 7 && ly == 6) || (lx == 9 && ly == 9) || (lx == 5 && ly == 11) || (lx == 6 && ly == 11),
                                BlockId::ChickenSpawnEgg => (lx == 7 && ly == 5) || (lx == 6 && ly == 8) || (lx == 9 && ly == 10) || (lx == 7 && ly == 12),
                                BlockId::SheepSpawnEgg => (lx == 7 && ly == 6) || (lx == 8 && ly == 6) || (lx == 6 && ly == 9) || (lx == 9 && ly == 10) || (lx == 7 && ly == 11),
                                BlockId::DolphinSpawnEgg => (lx == 6 && ly == 5) || (lx == 8 && ly == 8) || (lx == 6 && ly == 11) || (lx == 9 && ly == 11),
                                BlockId::FishSpawnEgg => (lx == 7 && ly == 5) || (lx == 8 && ly == 7) || (lx == 6 && ly == 9) || (lx == 8 && ly == 12),
                                BlockId::VillagerSpawnEgg => (lx == 6 && ly == 6) || (lx == 8 && ly == 8) || (lx == 6 && ly == 11) || (lx == 9 && ly == 11),
                                _ => false,
                            };
                            if is_spot {
                                px(buffer, lx, ly, spot);
                            } else {
                                px(buffer, lx, ly, base);
                            }
                        }
                    }
                }
            }
        }
        BlockId::Emerald => {
            let outline = pack_color(0.04, 0.28, 0.08);
            let glint = pack_color(0.75, 1.00, 0.75);
            let hi = pack_color(0.18, 0.85, 0.32);
            let base = pack_color(0.10, 0.68, 0.22);
            let sh = pack_color(0.06, 0.45, 0.14);

            for ly in 3..=13 {
                for lx in 3..=12 {
                    let in_shape = match ly {
                        3 => lx >= 6 && lx <= 9,
                        4 => lx >= 5 && lx <= 10,
                        5 => lx >= 4 && lx <= 11,
                        6..=7 => lx >= 3 && lx <= 12,
                        8 => lx >= 4 && lx <= 11,
                        9 => lx >= 5 && lx <= 10,
                        10 => lx >= 6 && lx <= 9,
                        11 => lx >= 6 && lx <= 9,
                        12 => lx >= 7 && lx <= 8,
                        13 => lx == 7 || lx == 8,
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || (ly == 6 && (lx == 3 || lx == 12)) || (ly == 7 && (lx == 3 || lx == 12)) || (ly == 12 && (lx == 7 || lx == 8));
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if ly <= 5 && lx >= 6 && lx <= 8 {
                            px(buffer, lx, ly, glint);
                        } else if lx <= 6 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 10 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::Diamond => {
            let outline = pack_color(0.05, 0.35, 0.38);
            let glint = pack_color(0.88, 1.00, 1.00);
            let hi = pack_color(0.40, 0.92, 0.94);
            let base = pack_color(0.20, 0.75, 0.80);
            let sh = pack_color(0.10, 0.52, 0.58);

            for ly in 3..=13 {
                for lx in 3..=12 {
                    let in_shape = match ly {
                        3 => lx >= 6 && lx <= 9,
                        4 => lx >= 5 && lx <= 10,
                        5 => lx >= 4 && lx <= 11,
                        6..=7 => lx >= 3 && lx <= 12,
                        8 => lx >= 4 && lx <= 11,
                        9 => lx >= 5 && lx <= 10,
                        10 => lx >= 6 && lx <= 9,
                        11 => lx >= 6 && lx <= 9,
                        12 => lx >= 7 && lx <= 8,
                        13 => lx == 7 || lx == 8,
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || (ly == 6 && (lx == 3 || lx == 12)) || (ly == 7 && (lx == 3 || lx == 12)) || (ly == 12 && (lx == 7 || lx == 8));
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if ly <= 5 && lx >= 6 && lx <= 8 {
                            px(buffer, lx, ly, glint);
                        } else if lx <= 6 {
                            px(buffer, lx, ly, hi);
                        } else if lx >= 10 || ly >= 10 {
                            px(buffer, lx, ly, sh);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
            px(buffer, 6, 4, glint);
            px(buffer, 7, 4, glint);
            px(buffer, 5, 5, glint);
            px(buffer, 7, 6, hi);
            px(buffer, 8, 6, base);
        }
        BlockId::Bed => {
            let wood = pack_color(0.45, 0.28, 0.12);
            let white = pack_color(0.92, 0.92, 0.94);
            let red = pack_color(0.80, 0.15, 0.15);
            let red_sh = pack_color(0.55, 0.10, 0.10);
            let frame = pack_color(0.30, 0.18, 0.08);

            // Legs
            px(buffer, 2, 13, frame);
            px(buffer, 2, 14, frame);
            px(buffer, 13, 13, frame);
            px(buffer, 13, 14, frame);

            // Base wood frame
            for lx in 2..=13 {
                px(buffer, lx, 11, wood);
                px(buffer, lx, 12, frame);
            }
            // Pillow on left
            for ly in 8..=10 {
                for lx in 2..=5 {
                    px(buffer, lx, ly, white);
                }
            }
            // Red blanket on right
            for ly in 8..=10 {
                for lx in 6..=13 {
                    px(buffer, lx, ly, if ly == 10 { red_sh } else { red });
                }
            }
            // Headboard post
            px(buffer, 1, 6, frame);
            px(buffer, 1, 7, frame);
            px(buffer, 1, 8, frame);
            px(buffer, 1, 9, frame);
            px(buffer, 1, 10, frame);
            px(buffer, 1, 11, frame);
        }
        BlockId::Ladder => {
            let wood_rail = pack_color(0.48, 0.32, 0.16);
            let rail_hi = pack_color(0.65, 0.45, 0.24);
            let rung = pack_color(0.58, 0.40, 0.20);
            let rung_sh = pack_color(0.35, 0.22, 0.10);

            // Rails
            for ly in 1..=14 {
                px(buffer, 3, ly, rail_hi);
                px(buffer, 4, ly, wood_rail);
                px(buffer, 11, ly, rail_hi);
                px(buffer, 12, ly, wood_rail);
            }
            // Rungs
            for &ry in &[3, 6, 9, 12] {
                for lx in 5..=10 {
                    px(buffer, lx, ry, rung);
                    px(buffer, lx, ry + 1, rung_sh);
                }
            }
        }
        BlockId::Torch => {
            let wood_light = pack_color(0.68, 0.48, 0.26);
            let wood_mid = pack_color(0.52, 0.35, 0.18);
            let wood_dark = pack_color(0.35, 0.22, 0.10);
            let coal = pack_color(0.20, 0.18, 0.16);
            let flame_core = pack_color(1.00, 1.00, 0.88);
            let flame_gold = pack_color(1.00, 0.80, 0.15);
            let flame_orange = pack_color(0.98, 0.42, 0.05);

            // Glowing flame tip
            px(buffer, 7, 1, flame_orange);
            px(buffer, 8, 1, flame_gold);
            px(buffer, 6, 2, flame_orange);
            px(buffer, 7, 2, flame_core);
            px(buffer, 8, 2, flame_core);
            px(buffer, 9, 2, flame_gold);
            px(buffer, 6, 3, flame_gold);
            px(buffer, 7, 3, flame_core);
            px(buffer, 8, 3, flame_gold);
            px(buffer, 9, 3, flame_orange);
            px(buffer, 7, 4, flame_orange);
            px(buffer, 8, 4, flame_orange);

            // Coal / wick top
            px(buffer, 7, 5, coal);
            px(buffer, 8, 5, coal);
            px(buffer, 7, 6, coal);
            px(buffer, 8, 6, wood_dark);

            // Wooden stick body
            for ly in 7..=14 {
                px(buffer, 7, ly, if ly % 3 == 0 { wood_light } else { wood_mid });
                px(buffer, 8, ly, wood_dark);
            }
        }
        BlockId::RustStone | BlockId::RustStoneWireLit => {
            let dark = pack_color(0.45, 0.12, 0.06);
            let base = pack_color(0.82, 0.26, 0.12);
            let hi = pack_color(1.00, 0.48, 0.18);
            let ember = pack_color(1.00, 0.82, 0.40);
            for ly in 4..=12 {
                for lx in 4..=12 {
                    let dist_sq = (lx as f32 - 7.5).powi(2) + (ly as f32 - 7.5).powi(2);
                    if dist_sq <= 15.0 {
                        let col = if dist_sq < 3.0 {
                            ember
                        } else if (lx + ly) % 3 == 0 {
                            hi
                        } else if (lx + ly) % 2 == 0 {
                            base
                        } else {
                            dark
                        };
                        px(buffer, lx, ly, col);
                    }
                }
            }
        }
        BlockId::Lever | BlockId::LeverOn => {
            let stone_base = pack_color(0.55, 0.55, 0.58);
            let stone_sh = pack_color(0.35, 0.35, 0.38);
            let wood_shaft = pack_color(0.55, 0.35, 0.15);
            let tip = pack_color(0.80, 0.25, 0.15);
            for ly in 11..=14 {
                for lx in 4..=11 {
                    px(buffer, lx, ly, if ly == 14 { stone_sh } else { stone_base });
                }
            }
            let is_on = block == BlockId::LeverOn;
            for i in 0..7 {
                let lx = if is_on { 9 - i / 2 } else { 6 + i / 2 };
                let ly = 10 - i;
                px(buffer, lx, ly, if i >= 5 { tip } else { wood_shaft });
            }
        }
        BlockId::Button | BlockId::ButtonPressed => {
            let hi = pack_color(0.75, 0.75, 0.78);
            let base = pack_color(0.58, 0.58, 0.60);
            let sh = pack_color(0.38, 0.38, 0.40);
            for ly in 6..=10 {
                for lx in 4..=11 {
                    let col = if lx == 4 || ly == 6 { hi } else if lx == 11 || ly == 10 { sh } else { base };
                    px(buffer, lx, ly, col);
                }
            }
        }
        BlockId::PressurePlate | BlockId::PressurePlatePressed => {
            let border = pack_color(0.40, 0.40, 0.42);
            let base = pack_color(0.60, 0.60, 0.62);
            let hi = pack_color(0.75, 0.75, 0.78);
            for ly in 10..=13 {
                for lx in 2..=13 {
                    let col = if ly == 10 { hi } else if ly == 13 || lx == 2 || lx == 13 { border } else { base };
                    px(buffer, lx, ly, col);
                }
            }
        }
        BlockId::WoodenSlab | BlockId::StoneSlab => {
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);

            let half_h = size / 2;
            let start_y = y + half_h;
            fill_rect(buffer, buf_w, buf_h, x, start_y, size, half_h, packed);
            fill_rect(buffer, buf_w, buf_h, x, start_y, size, 2, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x, start_y, 2, half_h, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x, start_y + half_h - 2, size, 2, shadow);
            fill_rect(buffer, buf_w, buf_h, x + size - 2, start_y, 2, half_h, shadow);
            fill_rect(buffer, buf_w, buf_h, x, start_y + half_h - 1, size, 1, dark_border);
            fill_rect(buffer, buf_w, buf_h, x + size - 1, start_y, 1, half_h, dark_border);
        }
        BlockId::WoodenStairs | BlockId::StoneStairs => {
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);

            let half = size / 2;
            fill_rect(buffer, buf_w, buf_h, x, y + half, size, half, packed);
            fill_rect(buffer, buf_w, buf_h, x, y, half, half, packed);

            fill_rect(buffer, buf_w, buf_h, x, y, half, 2, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x, y, 2, size, top_highlight);
            fill_rect(buffer, buf_w, buf_h, x + half, y + half, half, 2, top_highlight);

            fill_rect(buffer, buf_w, buf_h, x + half - 2, y, 2, half, shadow);
            fill_rect(buffer, buf_w, buf_h, x + size - 2, y + half, 2, half, shadow);
            fill_rect(buffer, buf_w, buf_h, x, y + size - 1, size, 1, dark_border);
            fill_rect(buffer, buf_w, buf_h, x + size - 1, y + half, 1, half, dark_border);
        }
        BlockId::FishingRod => {
            for i in 2..=13 {
                px(buffer, i, 15 - i, wood_mid);
                px(buffer, i - 1, 15 - i, wood_light);
            }
            let string_col = pack_color(0.88, 0.88, 0.90);
            for y in 3..=11 {
                px(buffer, 13, y, string_col);
            }
            px(buffer, 13, 12, pack_color(0.90, 0.15, 0.15));
            px(buffer, 12, 13, pack_color(0.70, 0.70, 0.75));
            px(buffer, 13, 13, pack_color(0.95, 0.95, 0.95));
        }
        BlockId::RawCod | BlockId::CookedCod => {
            let is_cooked = block == BlockId::CookedCod;
            let outline = if is_cooked { pack_color(0.35, 0.25, 0.12) } else { pack_color(0.30, 0.28, 0.24) };
            let base = if is_cooked { pack_color(0.78, 0.60, 0.36) } else { pack_color(0.68, 0.62, 0.54) };
            let belly = if is_cooked { pack_color(0.92, 0.78, 0.52) } else { pack_color(0.85, 0.82, 0.76) };
            let fin = if is_cooked { pack_color(0.60, 0.42, 0.22) } else { pack_color(0.52, 0.46, 0.38) };
            let eye = pack_color(0.12, 0.12, 0.12);

            px(buffer, 2, 6, fin);
            px(buffer, 2, 7, outline);
            px(buffer, 2, 8, fin);
            px(buffer, 3, 7, outline);

            for lx in 4..=11 {
                for ly in 6..=8 {
                    let col = if lx >= 10 && ly == 7 { eye } else if ly == 8 { belly } else if ly == 6 { fin } else { base };
                    px(buffer, lx, ly, col);
                }
            }
            for lx in 3..=12 {
                px(buffer, lx, 5, outline);
                px(buffer, lx, 9, outline);
            }
            px(buffer, 13, 7, outline);
        }
        BlockId::RawSalmon | BlockId::CookedSalmon => {
            let is_cooked = block == BlockId::CookedSalmon;
            let outline = if is_cooked { pack_color(0.38, 0.15, 0.08) } else { pack_color(0.38, 0.18, 0.16) };
            let base = if is_cooked { pack_color(0.85, 0.45, 0.22) } else { pack_color(0.82, 0.32, 0.28) };
            let belly = if is_cooked { pack_color(0.95, 0.65, 0.40) } else { pack_color(0.92, 0.55, 0.48) };
            let fin = if is_cooked { pack_color(0.65, 0.30, 0.12) } else { pack_color(0.55, 0.20, 0.18) };
            let eye = pack_color(0.12, 0.12, 0.12);

            px(buffer, 2, 5, fin);
            px(buffer, 2, 6, outline);
            px(buffer, 2, 7, fin);
            px(buffer, 3, 6, outline);

            for lx in 4..=12 {
                for ly in 5..=8 {
                    let col = if lx >= 11 && ly == 6 { eye } else if ly >= 7 { belly } else if ly == 5 { fin } else { base };
                    px(buffer, lx, ly, col);
                }
            }
            for lx in 3..=13 {
                px(buffer, lx, 4, outline);
                px(buffer, lx, 9, outline);
            }
            px(buffer, 14, 6, outline);
            px(buffer, 14, 7, outline);
        }
        BlockId::Pufferfish => {
            let outline = pack_color(0.40, 0.32, 0.08);
            let base = pack_color(0.92, 0.78, 0.22);
            let belly = pack_color(0.96, 0.92, 0.65);
            let spike = pack_color(0.72, 0.58, 0.15);
            let eye = pack_color(0.15, 0.25, 0.45);

            px(buffer, 4, 3, spike);
            px(buffer, 8, 2, spike);
            px(buffer, 11, 3, spike);
            px(buffer, 2, 8, spike);
            px(buffer, 13, 8, spike);
            px(buffer, 4, 12, spike);
            px(buffer, 8, 13, spike);
            px(buffer, 11, 12, spike);

            for ly in 4..=11 {
                for lx in 4..=11 {
                    let in_circle = (lx - 7) * (lx - 7) + (ly - 7) * (ly - 7) <= 16;
                    if in_circle {
                        let is_border = (lx - 7) * (lx - 7) + (ly - 7) * (ly - 7) >= 12;
                        if is_border {
                            px(buffer, lx, ly, outline);
                        } else if lx == 9 && ly == 6 {
                            px(buffer, lx, ly, eye);
                        } else if ly >= 8 {
                            px(buffer, lx, ly, belly);
                        } else {
                            px(buffer, lx, ly, base);
                        }
                    }
                }
            }
        }
        BlockId::TropicalFish => {
            let outline = pack_color(0.45, 0.18, 0.05);
            let orange = pack_color(0.95, 0.48, 0.12);
            let white = pack_color(0.96, 0.96, 0.98);
            let fin = pack_color(0.18, 0.72, 0.85);
            let eye = pack_color(0.10, 0.10, 0.12);

            px(buffer, 2, 6, fin);
            px(buffer, 2, 7, fin);
            px(buffer, 2, 8, fin);
            px(buffer, 3, 7, outline);

            for lx in 4..=12 {
                for ly in 6..=8 {
                    let is_white_stripe = lx == 6 || lx == 7 || lx == 10;
                    let col = if lx == 11 && ly == 6 {
                        eye
                    } else if is_white_stripe {
                        white
                    } else {
                        orange
                    };
                    px(buffer, lx, ly, col);
                }
            }
            px(buffer, 7, 5, fin);
            px(buffer, 8, 5, fin);

            for lx in 3..=12 {
                px(buffer, lx, 5, outline);
                px(buffer, lx, 9, outline);
            }
            px(buffer, 13, 7, outline);
        }
        BlockId::DriedKelp => {
            let dark = pack_color(0.12, 0.22, 0.10);
            let mid = pack_color(0.20, 0.38, 0.16);
            let hi = pack_color(0.28, 0.52, 0.22);
            let tie = pack_color(0.68, 0.55, 0.30);

            for ly in 4..=11 {
                for lx in 4..=11 {
                    let is_tie = ly == 7 || ly == 8;
                    let col = if is_tie {
                        tie
                    } else if lx == 4 || lx == 11 || ly == 4 || ly == 11 {
                        dark
                    } else if (lx + ly) % 2 == 0 {
                        hi
                    } else {
                        mid
                    };
                    px(buffer, lx, ly, col);
                }
            }
        }
        BlockId::Kelp => {
            let stalk = pack_color(0.16, 0.32, 0.14);
            let leaf = pack_color(0.24, 0.54, 0.20);
            let hi = pack_color(0.35, 0.70, 0.28);
            for y in 2..=14 {
                px(buffer, 7, y, stalk);
                px(buffer, 8, y, stalk);
                if y % 3 == 0 {
                    px(buffer, 5, y, leaf);
                    px(buffer, 6, y, hi);
                } else if y % 3 == 1 {
                    px(buffer, 9, y, hi);
                    px(buffer, 10, y, leaf);
                }
            }
        }
        BlockId::Seagrass => {
            let leaf1 = pack_color(0.22, 0.58, 0.24);
            let leaf2 = pack_color(0.30, 0.68, 0.32);
            let dark = pack_color(0.15, 0.40, 0.18);
            for y in 5..=14 {
                let x1 = 5 + ((14 - y) / 3);
                px(buffer, x1, y, leaf1);
                px(buffer, x1 + 1, y, dark);
            }
            for y in 3..=14 {
                let x2 = 9 + ((14 - y) % 2);
                px(buffer, x2, y, leaf2);
                px(buffer, x2 + 1, y, dark);
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
        assert_eq!(CREATIVE_TABS.len(), 6);
        let tab0 = creative_tab_blocks(0);
        assert!(tab0.contains(&BlockId::Grass));
        assert!(tab0.contains(&BlockId::Stone));
        assert!(tab0.contains(&BlockId::Wood));
        assert!(tab0.contains(&BlockId::PigSpawnEgg));
        assert!(tab0.contains(&BlockId::CowSpawnEgg));
        assert!(tab0.contains(&BlockId::ChickenSpawnEgg));
        assert!(tab0.contains(&BlockId::SheepSpawnEgg));
        assert!(tab0.contains(&BlockId::WhiteWool));

        let tab1 = creative_tab_blocks(1);
        assert!(tab1.contains(&BlockId::WoodenPickaxe));
        assert!(tab1.contains(&BlockId::IronSword));
        assert!(tab1.contains(&BlockId::Shears));
        assert!(tab1.contains(&BlockId::Tnt));
        assert!(tab1.contains(&BlockId::PigSpawnEgg));
        assert!(tab1.contains(&BlockId::CowSpawnEgg));
        assert!(tab1.contains(&BlockId::ChickenSpawnEgg));
        assert!(tab1.contains(&BlockId::SheepSpawnEgg));

        let tab2 = creative_tab_blocks(2); // Redstone
        assert!(tab2.contains(&BlockId::RustStone));
        assert!(tab2.contains(&BlockId::RustStoneBlock));
        assert!(tab2.contains(&BlockId::RustStoneLamp));
        assert!(tab2.contains(&BlockId::Lever));
        assert!(tab2.contains(&BlockId::Button));
        assert!(tab2.contains(&BlockId::PressurePlate));

        let tab3 = creative_tab_blocks(3); // Farming
        assert!(tab3.contains(&BlockId::WoodenHoe));
        assert!(tab3.contains(&BlockId::DiamondHoe));
        assert!(tab3.contains(&BlockId::WheatSeeds));
        assert!(tab3.contains(&BlockId::Carrot));
        assert!(tab3.contains(&BlockId::Potato));
        assert!(tab3.contains(&BlockId::BoneMeal));
        assert!(tab3.contains(&BlockId::Farmland));
        assert!(tab3.contains(&BlockId::SheepSpawnEgg));

        let tab4 = creative_tab_blocks(4); // All
        assert!(tab4.len() >= 76);
        assert!(tab4.contains(&BlockId::Grass));
        assert!(tab4.contains(&BlockId::Torch));
        assert!(tab4.contains(&BlockId::IronSword));
        assert!(tab4.contains(&BlockId::DiamondSword));
        assert!(tab4.contains(&BlockId::DiamondPickaxe));
        assert!(tab4.contains(&BlockId::Diamond));
        assert!(tab4.contains(&BlockId::Bed));
        assert!(tab4.contains(&BlockId::Ladder));
        assert!(tab4.contains(&BlockId::WoodenSlab));
        assert!(tab4.contains(&BlockId::WoodenStairs));
        assert!(tab4.contains(&BlockId::RawPorkchop));
        assert!(tab4.contains(&BlockId::CookedBeef));
        assert!(tab4.contains(&BlockId::RawChicken));
        assert!(tab4.contains(&BlockId::CookedChicken));
        assert!(tab4.contains(&BlockId::Feather));
        assert!(tab4.contains(&BlockId::PigSpawnEgg));
        assert!(tab4.contains(&BlockId::CowSpawnEgg));
        assert!(tab4.contains(&BlockId::ChickenSpawnEgg));
        assert!(tab4.contains(&BlockId::SheepSpawnEgg));
        assert!(tab4.contains(&BlockId::DolphinSpawnEgg));
        assert!(tab4.contains(&BlockId::FishSpawnEgg));
        assert!(tab4.contains(&BlockId::FishingRod));
        assert!(tab4.contains(&BlockId::RawCod));
        assert!(tab4.contains(&BlockId::CookedCod));
        assert!(tab4.contains(&BlockId::RawSalmon));
        assert!(tab4.contains(&BlockId::CookedSalmon));
        assert!(tab4.contains(&BlockId::Pufferfish));
        assert!(tab4.contains(&BlockId::TropicalFish));
        assert!(tab4.contains(&BlockId::DriedKelp));
        assert!(tab4.contains(&BlockId::Kelp));
        assert!(tab4.contains(&BlockId::Seagrass));
        assert!(tab4.contains(&BlockId::Shears));
        assert!(tab4.contains(&BlockId::WhiteWool));
        assert!(tab4.contains(&BlockId::RedWool));
        assert!(tab4.contains(&BlockId::Chest));
        assert!(tab4.contains(&BlockId::Furnace));
        assert!(tab4.contains(&BlockId::IronOunce));
        assert!(tab4.contains(&BlockId::IronBlock));
        assert!(tab4.contains(&BlockId::GoldOre));
        assert!(tab4.contains(&BlockId::GoldBlock));
        assert!(tab4.contains(&BlockId::CoalBlock));
        assert!(tab4.contains(&BlockId::Farmland));
        assert!(tab4.contains(&BlockId::RustStone));
        assert!(tab4.contains(&BlockId::RustStoneBlock));
        assert!(tab4.contains(&BlockId::RustStoneLamp));
        assert!(tab4.contains(&BlockId::Lever));
        assert!(tab4.contains(&BlockId::Button));
        assert!(tab4.contains(&BlockId::PressurePlate));
        assert!(tab4.contains(&BlockId::Bone));
        assert!(tab4.contains(&BlockId::Gunpowder));
        assert!(tab4.contains(&BlockId::WheatSeeds));
        assert!(tab4.contains(&BlockId::DiamondHoe));
        assert!(tab4.contains(&BlockId::Water));
        assert!(tab4.contains(&BlockId::Lava));
        assert!(tab4.contains(&BlockId::Tnt));
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
        let tab1_x = layout.start_x + 90;
        assert_eq!(layout.find_slot_at(tab1_x, tab0_y, &player), Some(InventorySlotTarget::CreativeTab(1)));

        // First creative block in tab 0 (Grass)
        let block0_x = layout.start_x + 30;
        let block0_y = layout.start_y + 45;
        assert_eq!(layout.find_slot_at(block0_x, block0_y, &player), Some(InventorySlotTarget::Creative(BlockId::Grass.to_u8())));

        // Scroll Up and Down button detection
        let sb_up_x = layout.start_x + 410;
        let sb_up_y = layout.start_y + 40;
        assert_eq!(layout.find_slot_at(sb_up_x, sb_up_y, &player), Some(InventorySlotTarget::CreativeScrollUp));
        let sb_down_y = layout.start_y + 195;
        assert_eq!(layout.find_slot_at(sb_up_x, sb_down_y, &player), Some(InventorySlotTarget::CreativeScrollDown));

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

    #[test]
    fn test_draw_inventory_screen_renders_player_preview_and_background() {
        let width = 640;
        let height = 480;
        let mut buffer = vec![0u32; width * height];
        let player = Player::new();
        let recipes = RecipeBook::new();

        draw_inventory_screen(
            &mut buffer,
            width,
            height,
            &player,
            None,
            false,
            &recipes,
            Some((320.0, 240.0)),
            false,
        );

        // Ensure pixels were drawn to the buffer
        let colored_pixels = buffer.iter().filter(|&&p| p != 0).count();
        assert!(colored_pixels > 1000);
    }
}

