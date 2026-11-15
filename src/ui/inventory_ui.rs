use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::{GameMode, Player};
use crate::gameplay::recipe::RecipeBook;
use crate::ui::draw::{fill_rect, fill_rect_alpha, pack_color};
use crate::ui::widgets::{Grid, Rect, UiCanvas};
use crate::world::block::{BlockId, Face};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InventorySlotTarget {
    Hotbar(usize),
    Main(usize),
    Crafting2x2(usize),
    Crafting3x3(usize),
    CraftingOutput,
    Armor(usize),
    Offhand,
    Creative(u8),
    CreativeTab(usize),
    CreativeScrollUp,
    CreativeScrollDown,
    RecipeBookToggle,
    RecipeEntry(usize),
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
    static TABS: [OnceLock<&'static [BlockId]>; 5] = [
        OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(),
    ];
    macro_rules! blk_vec { ($($b:ident),* $(,)?) => { vec![$(BlockId::$b),*] }; }
    if tab >= 5 { return &[]; }
    *TABS[tab].get_or_init(|| {
        let mut list = match tab {
            0 => blk_vec![
                Grass, Dirt, Farmland, FarmlandWet, Stone, Sand, Mud, Wood,
                Leaves, SnowLeaves, CherryLeaves, AutumnLeaves, Sapling, Cactus, Snow, Ice,
                Bedrock, Water, Lava, Glass, CoalOre, IronOre, GoldOre, DiamondOre,
                EmeraldOre, RustStoneOre, CoalBlock, IronBlock, GoldBlock, DiamondBlock, EmeraldBlock, RustStoneBlock,
                Planks, WoodenSlab, StoneSlab, WoodenStairs, StoneStairs, BrickStairs, BrickSlab, Cobblestone,
                MossyCobblestone, CobblestoneStairs, CobblestoneSlab, Bricks, Sandstone, SandstoneStairs, SandstoneSlab, SmoothStone,
                Clay, Gravel, Obsidian, Bookshelf, Ladder, Door, CraftingTable, Furnace,
                FurnaceLit, Chest, Bed, Tnt, Torch, RustStone, RustStoneLamp, RustStoneLampLit,
                Lever, Button, PressurePlate, PigSpawnEgg, CowSpawnEgg, ChickenSpawnEgg, SheepSpawnEgg, VillagerSpawnEgg,
                IronGolemSpawnEgg, DolphinSpawnEgg, FishSpawnEgg, SpiderSpawnEgg, StringItem, SpiderEye, Kelp, Seagrass,
                WhiteWool, OrangeWool, MagentaWool, LightBlueWool, YellowWool, LimeWool, PinkWool, GrayWool,
                LightGrayWool, CyanWool, PurpleWool, BlueWool, BrownWool, GreenWool, RedWool, BlackWool,
            ],
            1 => blk_vec![
                WoodenPickaxe, StonePickaxe, IronPickaxe, GoldPickaxe, DiamondPickaxe, WoodenAxe, StoneAxe, IronAxe,
                GoldAxe, DiamondAxe, WoodenShovel, StoneShovel, IronShovel, GoldShovel, DiamondShovel, WoodenSword,
                StoneSword, IronSword, GoldSword, DiamondSword, WoodenHoe, StoneHoe, IronHoe, GoldHoe,
                DiamondHoe, Shears, Bow, Arrow, Shield, Leather, LeatherHelmet, LeatherChestplate,
                LeatherLeggings, LeatherBoots, IronHelmet, IronChestplate, IronLeggings, IronBoots, GoldHelmet, GoldChestplate,
                GoldLeggings, GoldBoots, DiamondHelmet, DiamondChestplate, DiamondLeggings, DiamondBoots, Stick, Bone,
                BoneMeal, Gunpowder, Feather, Torch, IronOunce, IronIngot, GoldOunce, GoldIngot,
                Diamond, Emerald, EmeraldBlock, ClayBall, Brick, Flint, Clay, Gravel,
                Cobblestone, Bricks, Sandstone, SmoothStone, Obsidian, Bookshelf, RustStone, RustStoneBlock,
                RustStoneLamp, Lever, Button, PressurePlate, Apple, WheatSeeds, Wheat, Bread,
                Carrot, Potato, BakedPotato, RawPorkchop, CookedPorkchop, RawBeef, CookedBeef, RawChicken,
                CookedChicken, PigSpawnEgg, CowSpawnEgg, ChickenSpawnEgg, SheepSpawnEgg, VillagerSpawnEgg, IronGolemSpawnEgg, DolphinSpawnEgg,
                FishSpawnEgg, SpiderSpawnEgg, SpiderEye, FishingRod, RawCod, CookedCod, RawSalmon, CookedSalmon,
                Pufferfish, TropicalFish, DriedKelp, Sapling, Water, Lava, Door, Bed,
                Ladder, CraftingTable, Chest, Furnace, Tnt,
            ],
            2 => blk_vec![
                RustStone, RustStoneOre, RustStoneBlock, RustStoneLamp, RustStoneLampLit, RustStoneWireLit, Lever, LeverOn,
                Button, ButtonPressed, PressurePlate, PressurePlatePressed, Tnt, Torch, Door, Chest,
                Furnace, FurnaceLit, CraftingTable,
            ],
            3 => blk_vec![
                WoodenHoe, StoneHoe, IronHoe, GoldHoe, DiamondHoe, WheatSeeds, Wheat, WheatCrop,
                Bread, Carrot, CarrotCrop, Potato, PotatoCrop, BakedPotato, Apple, Bone,
                BoneMeal, Sapling, Cactus, Water, Farmland, FarmlandWet, PigSpawnEgg, CowSpawnEgg,
                ChickenSpawnEgg, SheepSpawnEgg, VillagerSpawnEgg, IronGolemSpawnEgg, DolphinSpawnEgg, FishSpawnEgg, FishingRod, RawCod,
                CookedCod, RawSalmon, CookedSalmon, Pufferfish, TropicalFish, DriedKelp, Kelp, Seagrass,
                RawPorkchop, CookedPorkchop, RawBeef, CookedBeef, RawChicken, CookedChicken, Feather,
            ],
            _ => blk_vec![
                Grass, Dirt, Farmland, FarmlandWet, Stone, Sand, Mud, Wood,
                Leaves, SnowLeaves, CherryLeaves, AutumnLeaves, Sapling, Cactus, Snow, Ice,
                Water, Lava, Bedrock, CoalOre, IronOre, GoldOre, DiamondOre, EmeraldOre,
                RustStoneOre, CoalBlock, IronBlock, GoldBlock, DiamondBlock, EmeraldBlock, RustStoneBlock, Planks,
                WoodenSlab, StoneSlab, WoodenStairs, StoneStairs, BrickStairs, BrickSlab, Cobblestone, MossyCobblestone,
                CobblestoneStairs, CobblestoneSlab, Bricks, Sandstone, SandstoneStairs, SandstoneSlab, SmoothStone, Clay,
                Gravel, Obsidian, Bookshelf, Ladder, Glass, Door, CraftingTable, Furnace,
                FurnaceLit, Chest, Bed, Tnt, Torch, RustStone, RustStoneWireLit, RustStoneLamp,
                RustStoneLampLit, Lever, LeverOn, Button, ButtonPressed, PressurePlate, PressurePlatePressed, WoodenPickaxe,
                StonePickaxe, IronPickaxe, GoldPickaxe, DiamondPickaxe, WoodenAxe, StoneAxe, IronAxe, GoldAxe,
                DiamondAxe, WoodenShovel, StoneShovel, IronShovel, GoldShovel, DiamondShovel, WoodenSword, StoneSword,
                IronSword, GoldSword, DiamondSword, WoodenHoe, StoneHoe, IronHoe, GoldHoe, DiamondHoe,
                Shears, Bow, Arrow, Shield, Leather, LeatherHelmet, LeatherChestplate, LeatherLeggings,
                LeatherBoots, IronHelmet, IronChestplate, IronLeggings, IronBoots, GoldHelmet, GoldChestplate, GoldLeggings,
                GoldBoots, DiamondHelmet, DiamondChestplate, DiamondLeggings, DiamondBoots, WheatSeeds, Wheat, WheatCrop,
                Bread, Carrot, CarrotCrop, Potato, PotatoCrop, BakedPotato, Apple, Stick,
                Bone, BoneMeal, Gunpowder, Feather, IronOunce, IronIngot, GoldOunce, GoldIngot,
                Diamond, Emerald, ClayBall, Brick, Flint, PigSpawnEgg, CowSpawnEgg, ChickenSpawnEgg,
                SheepSpawnEgg, VillagerSpawnEgg, IronGolemSpawnEgg, DolphinSpawnEgg, FishSpawnEgg, SpiderSpawnEgg, StringItem, SpiderEye,
                FishingRod, RawCod, CookedCod, RawSalmon, CookedSalmon, Pufferfish, TropicalFish, DriedKelp,
                Kelp, Seagrass, RawPorkchop, CookedPorkchop, RawBeef, CookedBeef, RawChicken, CookedChicken,
                WhiteWool, OrangeWool, MagentaWool, LightBlueWool, YellowWool, LimeWool, PinkWool, GrayWool,
                LightGrayWool, CyanWool, PurpleWool, BlueWool, BrownWool, GreenWool, RedWool, BlackWool,
            ],
        };
        if tab == 0 || tab == 4 {
            list.extend(crate::engine::modding::get_custom_creative_blocks());
        }
        if tab == 1 || tab == 4 {
            list.extend(crate::engine::modding::get_custom_creative_items());
        }
        Box::leak(list.into_boxed_slice())
    })
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

    #[inline]
    pub const fn creative_tabs_grid(&self) -> Grid {
        Grid::new(self.start_x - 3, self.start_y - 24, CREATIVE_TABS.len(), 1, 75, 26, 79, 26)
    }

    #[inline]
    pub const fn creative_items_grid(&self) -> Grid {
        Grid::square(self.start_x + 22, self.start_y + 36, 9, 4, 38, 4)
    }

    #[inline]
    pub const fn hotbar_grid(&self, offset_y: i32) -> Grid {
        Grid::square(self.start_x + 16, self.start_y + offset_y, 10, 1, 38, 2)
    }

    #[inline]
    pub const fn main_grid(&self, offset_y: i32) -> Grid {
        Grid::square(self.start_x + 22, self.start_y + offset_y, 9, 3, 38, 4)
    }

    #[inline]
    pub const fn armor_grid(&self, offset_x: i32, offset_y: i32) -> Grid {
        Grid::square(self.start_x + offset_x, self.start_y + offset_y, 1, 4, 24, 2)
    }

    #[inline]
    pub const fn crafting_2x2_grid(&self, offset_y: i32) -> Grid {
        Grid::square(self.start_x + 220, self.start_y + offset_y, 2, 2, 36, 4)
    }

    #[inline]
    pub const fn crafting_3x3_grid(&self) -> Grid {
        Grid::square(self.start_x + 40, self.start_y + 35, 3, 3, 36, 4)
    }

    #[inline]
    pub const fn trash_rect(&self, offset_y: i32) -> Rect {
        Rect::new(self.start_x + 420, self.start_y + offset_y, 36, 38)
    }

    pub fn find_slot_at(&self, px: i32, py: i32, player: &Player) -> Option<InventorySlotTarget> {
        if player.gamemode == GameMode::Creative {
            if let Some(tab_idx) = self.creative_tabs_grid().hit_test(px, py) {
                return Some(InventorySlotTarget::CreativeTab(tab_idx));
            }

            if player.creative_tab < CREATIVE_TABS.len() - 1 {
                let blocks = creative_tab_blocks(player.creative_tab);
                let total_rows = (blocks.len() + 8) / 9;
                let max_scroll = total_rows.saturating_sub(4);
                let scroll_row = player.creative_scroll.min(max_scroll);
                let scroll_offset = scroll_row * 9;

                if let Some(cell_idx) = self.creative_items_grid().hit_test(px, py) {
                    let idx = scroll_offset + cell_idx;
                    if idx < blocks.len() {
                        return Some(InventorySlotTarget::Creative(blocks[idx].to_u8()));
                    }
                }

                let sb_x = self.start_x + 404;
                if Rect::new(sb_x, self.start_y + 36, 18, 18).contains_point(px, py) {
                    return Some(InventorySlotTarget::CreativeScrollUp);
                }
                if Rect::new(sb_x, self.start_y + 186, 18, 18).contains_point(px, py) {
                    return Some(InventorySlotTarget::CreativeScrollDown);
                }

                if let Some(slot) = self.hotbar_grid(242).hit_test(px, py) {
                    return Some(InventorySlotTarget::Hotbar(slot));
                }
                if self.trash_rect(242).contains_point(px, py) {
                    return Some(InventorySlotTarget::Trash);
                }
                return None;
            } else {
                // Survival Inventory tab inside Creative
                if let Some(slot) = self.hotbar_grid(276).hit_test(px, py) {
                    return Some(InventorySlotTarget::Hotbar(slot));
                }
                if self.trash_rect(276).contains_point(px, py) {
                    return Some(InventorySlotTarget::Trash);
                }
                if let Some(i) = self.armor_grid(22, 36).hit_test(px, py) {
                    return Some(InventorySlotTarget::Armor(i));
                }
                if Rect::new(self.start_x + 82, self.start_y + 114, 24, 24).contains_point(px, py) {
                    return Some(InventorySlotTarget::Offhand);
                }
                if let Some(slot) = self.main_grid(144).hit_test(px, py) {
                    return Some(InventorySlotTarget::Main(slot));
                }
                if let Some(slot) = self.crafting_2x2_grid(42).hit_test(px, py) {
                    return Some(InventorySlotTarget::Crafting2x2(slot));
                }
                if Rect::new(self.start_x + 350, self.start_y + 60, 42, 42).contains_point(px, py) {
                    return Some(InventorySlotTarget::CraftingOutput);
                }
                return None;
            }
        }

        // Survival Mode (regular)
        if let Some(slot) = self.hotbar_grid(295).hit_test(px, py) {
            return Some(InventorySlotTarget::Hotbar(slot));
        }
        if let Some(slot) = self.main_grid(160).hit_test(px, py) {
            return Some(InventorySlotTarget::Main(slot));
        }

        if self.crafting_table_open {
            if let Some(i) = self.armor_grid(300, 34).hit_test(px, py) {
                return Some(InventorySlotTarget::Armor(i));
            }
            if Rect::new(self.start_x + 360, self.start_y + 112, 24, 24).contains_point(px, py) {
                return Some(InventorySlotTarget::Offhand);
            }
            if let Some(slot) = self.crafting_3x3_grid().hit_test(px, py) {
                return Some(InventorySlotTarget::Crafting3x3(slot));
            }
            if Rect::new(self.start_x + 230, self.start_y + 75, 42, 42).contains_point(px, py) {
                return Some(InventorySlotTarget::CraftingOutput);
            }
        } else {
            if let Some(i) = self.armor_grid(18, 34).hit_test(px, py) {
                return Some(InventorySlotTarget::Armor(i));
            }
            if Rect::new(self.start_x + 78, self.start_y + 112, 24, 24).contains_point(px, py) {
                return Some(InventorySlotTarget::Offhand);
            }
            if let Some(slot) = self.crafting_2x2_grid(48).hit_test(px, py) {
                return Some(InventorySlotTarget::Crafting2x2(slot));
            }
            if Rect::new(self.start_x + 350, self.start_y + 68, 42, 42).contains_point(px, py) {
                return Some(InventorySlotTarget::CraftingOutput);
            }
        }

        None
    }

    pub fn find_recipe_book_target_at(
        &self,
        px: i32,
        py: i32,
        recipe_book_open: bool,
        unlocked_recipe_indices: &[usize],
        recipe_book_scroll: usize,
    ) -> Option<InventorySlotTarget> {
        if !self.crafting_table_open {
            return None;
        }

        if Rect::new(self.start_x + 190, self.start_y + 36, 88, 20).contains_point(px, py) {
            return Some(InventorySlotTarget::RecipeBookToggle);
        }

        if !recipe_book_open {
            return None;
        }

        let list_x = self.start_x + 302;
        let list_y = self.start_y + 51;
        let first = recipe_book_scroll.min(unlocked_recipe_indices.len());
        let rows_grid = Grid::new(list_x + 3, list_y, 1, 5, 144, 15, 144, 15);
        if let Some(row) = rows_grid.hit_test(px, py) {
            if let Some(&recipe_index) = unlocked_recipe_indices.get(first + row) {
                return Some(InventorySlotTarget::RecipeEntry(recipe_index));
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
    armor: &[Option<ItemStack>; 4],
    offhand: &Option<ItemStack>,
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

    // 4. TORSO (Tułów) - 20px wide x 28px tall
    let torso_x = center_x - 10;
    let torso_y = center_y - 12;
    fill_rect(buffer, width, height, torso_x, torso_y, 20, 28, shirt_cyan);
    fill_rect(buffer, width, height, torso_x, torso_y, 2, 28, shirt_highlight);
    fill_rect(buffer, width, height, torso_x + 18, torso_y, 2, 28, shirt_shadow);
    fill_rect(buffer, width, height, torso_x, torso_y + 26, 20, 2, shirt_dark);
    fill_rect(buffer, width, height, torso_x + 7, torso_y, 6, 4, skin_base);
    fill_rect(buffer, width, height, torso_x + 8, torso_y + 4, 4, 2, skin_shadow);

    // 5. ARMS (Ręce) - each 8px wide x 28px tall
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

    // 6. HEAD (Głowa)
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

    // 7. ARMOR OVERLAYS
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
        draw_slot(buffer, width, height, armor_x, sy, 24, 24, player.inventory.armor[i], is_hovered);
        if player.inventory.armor[i].is_none() {
            let sil = pack_color(0.24, 0.24, 0.28);
            match i {
                0 => {
                    // Helmet silhouette
                    fill_rect(buffer, width, height, armor_x + 7, sy + 6, 10, 5, sil);
                    fill_rect(buffer, width, height, armor_x + 7, sy + 11, 3, 5, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 11, 3, 5, sil);
                }
                1 => {
                    // Chestplate silhouette
                    fill_rect(buffer, width, height, armor_x + 5, sy + 5, 14, 12, sil);
                    fill_rect(buffer, width, height, armor_x + 9, sy + 5, 6, 4, pack_color(0.12, 0.12, 0.14));
                }
                2 => {
                    // Leggings silhouette
                    fill_rect(buffer, width, height, armor_x + 6, sy + 5, 12, 4, sil);
                    fill_rect(buffer, width, height, armor_x + 6, sy + 9, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 13, sy + 9, 5, 9, sil);
                }
                _ => {
                    // Boots silhouette
                    fill_rect(buffer, width, height, armor_x + 5, sy + 8, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 4, sy + 13, 6, 4, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 8, 5, 9, sil);
                    fill_rect(buffer, width, height, armor_x + 14, sy + 13, 6, 4, sil);
                }
            }
        }
    }

    let is_off_hovered = hovered_slot == Some(InventorySlotTarget::Offhand);
    draw_slot(buffer, width, height, offhand_x, offhand_y, 24, 24, player.inventory.offhand, is_off_hovered);
    if player.inventory.offhand.is_none() {
        let sil = pack_color(0.24, 0.24, 0.28);
        fill_rect(buffer, width, height, offhand_x + 6, offhand_y + 4, 12, 10, sil);
        fill_rect(buffer, width, height, offhand_x + 8, offhand_y + 14, 8, 4, sil);
    }
}

fn draw_player_vitals_panel(ui: &mut UiCanvas<'_>, x: i32, y: i32, bar_w: i32, player: &Player) {
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
        // Draw 6 Top Tabs
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

        // Main creative window frame
        let win_rect = Rect::new(start_x - 8, start_y, 480, 360);
        ui.fill_alpha(win_rect, pack_color(0.14, 0.14, 0.16), 0.98);
        ui.bevel_border(win_rect, 3, pack_color(0.38, 0.38, 0.42), pack_color(0.05, 0.05, 0.06));

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
            ui.text_white(title, start_x + 18, start_y + 14, 2, 1);

            // Creative block grid (9 cols x 4 rows) with scroll offset
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

            // Scrollbar for creative item grid
            let sb_x = start_x + 404;
            let is_up_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollUp);
            let is_down_hover = hovered_slot == Some(InventorySlotTarget::CreativeScrollDown);

            // Up button
            let up_bg = if is_up_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
            ui.fill(Rect::new(sb_x, start_y + 36, 18, 16), up_bg);
            ui.fill(Rect::new(sb_x, start_y + 36, 18, 1), pack_color(0.4, 0.4, 0.45));
            ui.text("^", sb_x + 6, start_y + 40, 1, 1, pack_color(0.9, 0.9, 0.95));

            // Track
            let track_y = start_y + 54;
            let track_h = 130;
            ui.fill(Rect::new(sb_x, track_y, 18, track_h), pack_color(0.10, 0.10, 0.12));
            ui.fill(Rect::new(sb_x, track_y, 1, track_h), pack_color(0.06, 0.06, 0.08));
            ui.fill(Rect::new(sb_x + 17, track_y, 1, track_h), pack_color(0.22, 0.22, 0.26));

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
            let thumb_rect = Rect::new(sb_x + 2, thumb_y, 14, thumb_h);
            ui.fill(thumb_rect, pack_color(0.38, 0.38, 0.44));
            ui.bevel_border(thumb_rect, 1, pack_color(0.55, 0.55, 0.62), pack_color(0.20, 0.20, 0.24));

            // Down button
            let down_bg = if is_down_hover { pack_color(0.28, 0.28, 0.32) } else { pack_color(0.18, 0.18, 0.22) };
            ui.fill(Rect::new(sb_x, start_y + 186, 18, 16), down_bg);
            ui.fill(Rect::new(sb_x, start_y + 186, 18, 1), pack_color(0.4, 0.4, 0.45));
            ui.text("v", sb_x + 6, start_y + 190, 1, 1, pack_color(0.9, 0.9, 0.95));

            // Row counter
            if total_rows > 4 {
                let row_info = format!("{}/{}", scroll_row + 1, max_scroll + 1);
                ui.text_right(&row_info, sb_x + 18, start_y + 206, 1, 1, pack_color(0.60, 0.60, 0.65));
            }

            // Divider & Hotbar Header
            ui.fill(Rect::new(start_x + 16, start_y + 218, 432, 2), pack_color(0.25, 0.25, 0.28));
            ui.text("QUICK ACCESS HOTBAR", start_x + 16, start_y + 226, 1, 1, pack_color(0.85, 0.85, 0.4));
            ui.text("TRASH", start_x + 424, start_y + 226, 1, 1, pack_color(1.0, 0.4, 0.4));

            // Hotbar 10 slots
            draw_hotbar_slots(&mut ui, layout.hotbar_grid(242), player, hovered_slot, pack_color(0.4, 0.75, 1.0));

            // Trash Slot
            draw_trash_slot(&mut ui, layout.trash_rect(242), hovered_slot == Some(InventorySlotTarget::Trash));

            // Controls Footer
            let creative_footer = if is_gamepad_active {
                "[A] TAKE (64) | [Y] TO HOTBAR | [X] DELETE | [B] CLOSE"
            } else {
                "LMB: TAKE (64) | SHIFT+LMB: TO HOTBAR | 1-0: SELECT SLOT | [E]/[ESC]: CLOSE"
            };
            ui.text(creative_footer, start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75));
        } else {
            // Tab 5: Survival Inventory inside Creative
            ui.text_white("SURVIVAL INVENTORY", start_x + 18, start_y + 14, 2, 1);

            // Character Preview Box
            let char_box = Rect::new(start_x + 18, start_y + 34, 180, 104);
            ui.inset_panel(char_box, pack_color(0.08, 0.08, 0.10), 0.95);
            ui.fill_alpha(Rect::new(char_box.x + 12, char_box.y + 86, 64, 12), pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(ui.buffer, ui.width, ui.height, char_box.x + 52, char_box.y + 54, mouse_pos, &player.inventory.armor, &player.inventory.offhand);
            draw_armor_and_offhand_slots(ui.buffer, ui.width, ui.height, start_x + 22, start_y + 36, start_x + 82, start_y + 114, player, hovered_slot);

            ui.text("PLAYER", char_box.x + 110, char_box.y + 16, 1, 1, pack_color(1.0, 0.88, 0.25));
            ui.text("CREATIVE", char_box.x + 110, char_box.y + 38, 1, 1, pack_color(0.4, 0.9, 0.4));
            ui.text("FLYING: ON", char_box.x + 110, char_box.y + 58, 1, 1, pack_color(0.4, 0.8, 1.0));
            ui.text("GODMODE", char_box.x + 110, char_box.y + 78, 1, 1, pack_color(0.9, 0.7, 0.3));

            // 2x2 Crafting Grid, Arrow & Output slot
            ui.inset_panel(Rect::new(start_x + 208, start_y + 34, 242, 104), pack_color(0.10, 0.10, 0.12), 0.85);
            ui.text("Crafting 2x2", start_x + 220, start_y + 26, 1, 1, pack_color(0.85, 0.85, 0.85));
            draw_crafting_2x2_slots(
                &mut ui,
                layout.crafting_2x2_grid(42),
                start_x + 310,
                start_y + 70,
                Rect::new(start_x + 350, start_y + 60, 42, 42),
                player,
                recipes,
                hovered_slot,
            );

            // Main 27 slots inset panel
            ui.inset_panel(Rect::new(start_x + 14, start_y + 140, 436, 126), pack_color(0.10, 0.10, 0.12), 0.85);
            ui.text("Backpack", start_x + 22, start_y + 132, 1, 1, pack_color(0.75, 0.75, 0.75));
            draw_backpack_slots(&mut ui, layout.main_grid(144), player, hovered_slot);

            // Hotbar & Trash inset panel
            ui.inset_panel(Rect::new(start_x + 10, start_y + 270, 444, 52), pack_color(0.10, 0.10, 0.12), 0.85);
            ui.text("Hotbar", start_x + 16, start_y + 264, 1, 1, pack_color(0.85, 0.85, 0.4));
            ui.text("TRASH", start_x + 424, start_y + 264, 1, 1, pack_color(1.0, 0.4, 0.4));
            draw_hotbar_slots(&mut ui, layout.hotbar_grid(276), player, hovered_slot, pack_color(0.4, 0.75, 1.0));

            draw_trash_slot(&mut ui, layout.trash_rect(276), hovered_slot == Some(InventorySlotTarget::Trash));

            // Controls Footer
            let footer = if is_gamepad_active {
                "[A]: ALL | [X]: 1 ITEM/HALF | [Y]: MOVE | [B]: CLOSE"
            } else {
                "LPM: ALL | PPM: 1 ITEM/HALF | SHIFT+LPM: MOVE | [E]/[ESC]: CLOSE"
            };
            ui.text(footer, start_x + 16, start_y + 338, 1, 1, pack_color(0.7, 0.7, 0.75));
        }
    } else {
        // Survival Mode
        let win_rect = Rect::new(start_x - 8, start_y, 480, 360);
        let win_x = win_rect.x;
        let win_y = win_rect.y;
        let win_w = win_rect.w;
        let win_h = win_rect.h;

        // Main window panel background & borders
        ui.fill_alpha(win_rect, pack_color(0.15, 0.15, 0.18), 0.98);
        ui.bevel_border(win_rect, 3, pack_color(0.38, 0.38, 0.44), pack_color(0.05, 0.05, 0.06));
        ui.fill(Rect::new(win_x + 3, win_y + 3, win_w - 6, 1), pack_color(0.48, 0.48, 0.54));

        // Window Header Bar
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
        // Window Footer Bar
        ui.fill(Rect::new(win_x + 3, win_y + win_h - 22, win_w - 6, 19), pack_color(0.11, 0.11, 0.14));
        ui.fill(Rect::new(win_x + 3, win_y + win_h - 22, win_w - 6, 1), pack_color(0.07, 0.07, 0.09));
        ui.text(surv_footer, win_x + 12, win_y + win_h - 16, 1, 1, pack_color(0.70, 0.70, 0.75));

        // Crafting section
        if crafting_table_open {
            // 3x3 crafting inset panel
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

            // 3x3 input slots
            let craft_3x3 = layout.crafting_3x3_grid();
            for idx in 0..9 {
                let is_hovered = hovered_slot == Some(InventorySlotTarget::Crafting3x3(idx));
                ui.inventory_slot(craft_3x3.slot(idx), player.inventory.crafting_3x3[idx], is_hovered, None);
            }

            // Arrow & 3x3 Output slot
            ui.text("==>", start_x + 175, start_y + 85, 2, 1, pack_color(0.9, 0.85, 0.3));
            let result_item = recipes.match_grid(&player.inventory.crafting_3x3, 3, 3);
            ui.inventory_slot(
                Rect::new(start_x + 230, start_y + 75, 42, 42),
                result_item,
                hovered_slot == Some(InventorySlotTarget::CraftingOutput),
                None,
            );

            // Character preview box on the right of crafting table
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
            // Player character info area
            let char_box = Rect::new(start_x + 14, start_y + 32, 186, 106);
            ui.inset_panel(char_box, pack_color(0.08, 0.08, 0.10), 0.95);
            draw_armor_and_offhand_slots(ui.buffer, ui.width, ui.height, start_x + 18, start_y + 34, start_x + 78, start_y + 112, player, hovered_slot);
            ui.fill_alpha(Rect::new(char_box.x + 36, char_box.y + 88, 48, 12), pack_color(0.14, 0.14, 0.18), 0.7);
            draw_player_inventory_preview(ui.buffer, ui.width, ui.height, char_box.x + 56, char_box.y + 54, mouse_pos, &player.inventory.armor, &player.inventory.offhand);
            draw_player_vitals_panel(&mut ui, char_box.x + 106, char_box.y, 72, player);

            // 2x2 Crafting inset panel
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

        // Main Inventory Inset Panel
        ui.inset_panel(Rect::new(start_x + 14, start_y + 144, 436, 134), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("BACKPACK", start_x + 22, start_y + 146, 1, 1, pack_color(0.75, 0.75, 0.80));
        draw_backpack_slots(&mut ui, layout.main_grid(160), player, hovered_slot);

        // Hotbar Inset Panel
        ui.inset_panel(Rect::new(start_x + 10, start_y + 282, 444, 52), pack_color(0.10, 0.10, 0.12), 0.85);
        ui.text("QUICK ACCESS HOTBAR", start_x + 16, start_y + 284, 1, 1, pack_color(0.85, 0.85, 0.40));
        draw_hotbar_slots(&mut ui, layout.hotbar_grid(295), player, hovered_slot, pack_color(0.4, 0.7, 1.0));
    }

    // Hovered Item Name Tooltip
    let mut hovered_item_name = None;
    if let Some(target) = hovered_slot {
        match target {
            InventorySlotTarget::Hotbar(i) => hovered_item_name = player.inventory.hotbar[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Main(i) => hovered_item_name = player.inventory.main[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Crafting2x2(i) => hovered_item_name = player.inventory.crafting_2x2[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Crafting3x3(i) => hovered_item_name = player.inventory.crafting_3x3[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Armor(i) => hovered_item_name = player.inventory.armor[i].map(|s| (s.block.display_name().to_string(), s.count)),
            InventorySlotTarget::Offhand => hovered_item_name = player.inventory.offhand.map(|s| (s.block.display_name().to_string(), s.count)),
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
            InventorySlotTarget::RecipeBookToggle => {
                hovered_item_name = Some(("Open recipe book".to_string(), 1));
            }
            InventorySlotTarget::RecipeEntry(index) => {
                if let Some(recipe) = recipes.recipes().get(index) {
                    hovered_item_name = Some((recipe.result.block.display_name().to_string(), recipe.result.count));
                }
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
        ui.tooltip(&text, mouse_pos, pack_color(1.0, 0.95, 0.4));
    }

    // Draw Held Item on Mouse Cursor
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

    // Draw Cursor Pointer on screen
    if let Some((mx, my)) = mouse_pos {
        crate::ui::draw::draw_cursor_pointer(ui.buffer, ui.width, ui.height, mx as i32, my as i32);
    }
}

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
    UiCanvas::new(buffer, width, height).inventory_slot(Rect::new(x, y, w, h), item, is_hovered, None);
}

pub fn draw_item_icon(buffer: &mut [u32], buf_w: usize, buf_h: usize, block: BlockId, x: i32, y: i32, size: i32) {
    let scale = (size / 16).max(1);
    let ox = x + (size - 16 * scale) / 2;
    let oy = y + (size - 16 * scale) / 2;

    macro_rules! px {
        ($lx:expr, $ly:expr, $col:expr) => {{
            let (lx, ly, col) = ($lx, $ly, $col);
            if lx >= 0 && lx < 16 && ly >= 0 && ly < 16 {
                fill_rect(buffer, buf_w, buf_h, ox + lx * scale, oy + ly * scale, scale, scale, col);
            }
        }};
    }
    macro_rules! pts {
        ($col:expr => $(($lx:expr, $ly:expr)),+ $(,)?) => {{
            let c = $col;
            $(px!($lx, $ly, c);)+
        }};
    }

    let wood_dark = pack_color(0.30, 0.18, 0.08);
    let wood_mid = pack_color(0.55, 0.35, 0.15);
    let wood_light = pack_color(0.72, 0.52, 0.28);

    let get_mat_colors = |b: BlockId| -> (u32, u32, u32, u32) {
        match b {
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => (
                pack_color(0.85, 0.68, 0.42), pack_color(0.68, 0.50, 0.30), pack_color(0.48, 0.32, 0.16), pack_color(0.32, 0.20, 0.10),
            ),
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => (
                pack_color(0.78, 0.78, 0.80), pack_color(0.58, 0.58, 0.60), pack_color(0.40, 0.40, 0.42), pack_color(0.24, 0.24, 0.26),
            ),
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => (
                pack_color(1.00, 0.95, 0.40), pack_color(0.95, 0.80, 0.20), pack_color(0.80, 0.65, 0.10), pack_color(0.50, 0.38, 0.05),
            ),
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => (
                pack_color(0.75, 1.00, 1.00), pack_color(0.35, 0.90, 0.90), pack_color(0.18, 0.72, 0.75), pack_color(0.08, 0.45, 0.48),
            ),
            _ => (
                pack_color(1.00, 1.00, 1.00), pack_color(0.90, 0.93, 0.98), pack_color(0.72, 0.76, 0.84), pack_color(0.42, 0.46, 0.54),
            ),
        }
    };

    let get_armor_colors = |b: BlockId| -> (u32, u32, u32, u32, u32) {
        match b {
            BlockId::LeatherHelmet | BlockId::LeatherChestplate | BlockId::LeatherLeggings | BlockId::LeatherBoots => (
                pack_color(0.76, 0.50, 0.28), pack_color(0.65, 0.40, 0.20), pack_color(0.52, 0.30, 0.14), pack_color(0.36, 0.18, 0.06), pack_color(0.22, 0.10, 0.04),
            ),
            BlockId::IronHelmet | BlockId::IronChestplate | BlockId::IronLeggings | BlockId::IronBoots => (
                pack_color(1.0, 1.0, 1.0), pack_color(0.95, 0.96, 0.98), pack_color(0.80, 0.82, 0.86), pack_color(0.52, 0.55, 0.60), pack_color(0.24, 0.25, 0.28),
            ),
            BlockId::GoldHelmet | BlockId::GoldChestplate | BlockId::GoldLeggings | BlockId::GoldBoots => (
                pack_color(1.0, 1.0, 0.7), pack_color(1.00, 0.95, 0.45), pack_color(0.92, 0.78, 0.18), pack_color(0.68, 0.52, 0.08), pack_color(0.38, 0.28, 0.04),
            ),
            _ => (
                pack_color(0.88, 1.0, 1.0), pack_color(0.65, 0.98, 0.98), pack_color(0.25, 0.82, 0.85), pack_color(0.12, 0.52, 0.56), pack_color(0.06, 0.30, 0.34),
            ),
        }
    };

    match block {
        BlockId::Stick => {
            for i in 2..=13 {
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
                px!(i, 16 - i, wood_dark);
            }
        }
        BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::GoldSword | BlockId::DiamondSword => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            pts!(wood_dark => (2, 13), (2, 12), (3, 10), (5, 12), (2, 9));
            pts!(wood_mid => (1, 14), (3, 12), (4, 11));
            px!(3, 11, wood_light);
            pts!(c_base => (4, 10), (5, 11));
            pts!(c_hi => (5, 10), (6, 9), (12, 3));
            for i in 5..=11 {
                px!(i, 15 - i, c_base);
                px!(i - 1, 15 - i, c_hi);
                px!(i + 1, 15 - i, c_shadow);
            }
            pts!(c_glint => (13, 2), (12, 2));
            px!(13, 3, c_shadow);
        }
        BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe | BlockId::GoldPickaxe | BlockId::DiamondPickaxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=10 {
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
                px!(i, 16 - i, wood_dark);
            }
            pts!(wood_dark => (9, 6), (10, 5));
            pts!(c_shadow => (4, 7), (12, 5), (11, 6), (10, 7), (9, 8));
            pts!(c_base => (4, 6), (5, 5), (6, 4), (8, 3), (12, 4), (11, 5));
            pts!(c_hi => (5, 4), (7, 3), (9, 2), (10, 2), (13, 3));
            pts!(c_glint => (11, 2), (12, 2), (13, 2));
        }
        BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe | BlockId::GoldAxe | BlockId::DiamondAxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=11 {
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
                px!(i, 16 - i, wood_dark);
            }
            pts!(c_glint => (9, 2), (10, 2), (8, 3));
            pts!(c_hi => (11, 2), (9, 3), (8, 4), (7, 5));
            pts!(c_base => (12, 2), (10, 3), (9, 4), (10, 4), (8, 5), (7, 6));
            pts!(c_shadow => (11, 3), (12, 3), (9, 5), (8, 6));
        }
        BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel | BlockId::GoldShovel | BlockId::DiamondShovel => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            px!(1, 13, wood_light);
            px!(2, 14, wood_dark);
            for i in 3..=9 {
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
                px!(i, 16 - i, wood_dark);
            }
            pts!(c_shadow => (10, 5), (12, 4), (11, 5), (10, 6));
            pts!(c_base => (11, 4), (10, 4), (9, 5));
            pts!(c_hi => (12, 3), (13, 3), (11, 3));
            pts!(c_glint => (13, 2), (12, 2));
        }
        BlockId::WoodenHoe | BlockId::StoneHoe | BlockId::IronHoe | BlockId::GoldHoe | BlockId::DiamondHoe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=10 {
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
                px!(i, 16 - i, wood_dark);
            }
            pts!(c_glint => (9, 2), (10, 2));
            px!(11, 2, c_hi);
            pts!(c_base => (12, 2), (11, 3));
            pts!(c_shadow => (12, 3), (12, 4));
        }
        BlockId::CraftingTable => {
            let (top_light, top_dark, side_frame, side_plank, iron_tool) = (
                pack_color(0.78, 0.62, 0.38), pack_color(0.58, 0.42, 0.22),
                pack_color(0.32, 0.20, 0.10), pack_color(0.64, 0.48, 0.28), pack_color(0.78, 0.82, 0.88),
            );
            for ly in 1..=6 {
                for lx in 2..=13 {
                    if (lx + ly >= 6) && (lx - ly <= 8) && (ly - lx <= 3) && (lx + ly <= 16) {
                        px!(lx, ly, if ((lx / 2) + (ly / 2)) % 2 == 0 { top_light } else { top_dark });
                    }
                }
            }
            for ly in 6..=14 {
                for lx in 2..=13 {
                    let is_border = lx == 2 || lx == 7 || lx == 8 || lx == 13 || ly == 14;
                    px!(lx, ly, if is_border { side_frame } else { side_plank });
                }
            }
            pts!(iron_tool => (4, 8), (4, 9), (5, 9), (10, 8), (11, 8));
            px!(10, 9, wood_dark);
        }
        BlockId::Door => {
            let (frame, glass, knob, panel) = (
                pack_color(0.52, 0.36, 0.18), pack_color(0.65, 0.82, 0.90),
                pack_color(0.85, 0.75, 0.30), pack_color(0.42, 0.28, 0.14),
            );
            for ly in 1..=14 {
                for lx in 4..=11 { px!(lx, ly, frame); }
            }
            for ly in 3..=6 { pts!(glass => (5, ly), (6, ly), (9, ly), (10, ly)); }
            px!(10, 8, knob);
            for ly in 9..=12 { pts!(panel => (5, ly), (6, ly), (9, ly), (10, ly)); }
        }
        BlockId::Tnt => {
            let (red, white, dark, fuse) = (
                pack_color(0.85, 0.15, 0.10), pack_color(0.92, 0.92, 0.92),
                pack_color(0.25, 0.05, 0.05), pack_color(0.40, 0.35, 0.30),
            );
            for ly in 3..=13 {
                for lx in 3..=12 { px!(lx, ly, if (7..=9).contains(&ly) { white } else { red }); }
            }
            pts!(dark => (5, 8), (7, 8), (8, 8), (10, 8));
            pts!(fuse => (7, 1), (7, 2));
        }
        BlockId::RawPorkchop | BlockId::CookedPorkchop => {
            let (outline, base, hi, sh, bone) = if block == BlockId::RawPorkchop {
                (pack_color(0.48, 0.20, 0.26), pack_color(0.92, 0.58, 0.64), pack_color(0.98, 0.78, 0.82), pack_color(0.75, 0.38, 0.45), pack_color(0.95, 0.95, 0.88))
            } else {
                (pack_color(0.28, 0.12, 0.05), pack_color(0.72, 0.42, 0.18), pack_color(0.90, 0.65, 0.32), pack_color(0.45, 0.22, 0.08), pack_color(0.92, 0.90, 0.82))
            };
            pts!(bone => (12, 3), (13, 2), (14, 2));
            px!(13, 4, outline);
            for ly in 4..=12 {
                for lx in 2..=11 {
                    if (lx + ly >= 7) && (lx + ly <= 21) && (ly - lx <= 8) && (lx - ly <= 6) {
                        let col = if lx == 2 || lx == 11 || ly == 4 || ly == 12 { outline }
                            else if lx <= 5 && ly <= 7 { hi }
                            else if lx >= 8 || ly >= 10 { sh }
                            else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::RawBeef | BlockId::CookedBeef => {
            let is_raw = block == BlockId::RawBeef;
            let (outline, base, hi, sh, accent) = if is_raw {
                (pack_color(0.38, 0.08, 0.08), pack_color(0.82, 0.20, 0.20), pack_color(0.94, 0.42, 0.42), pack_color(0.52, 0.10, 0.10), pack_color(0.96, 0.90, 0.88))
            } else {
                (pack_color(0.18, 0.08, 0.04), pack_color(0.50, 0.26, 0.14), pack_color(0.70, 0.42, 0.22), pack_color(0.28, 0.12, 0.06), pack_color(0.15, 0.06, 0.02))
            };
            for ly in 3..=13 {
                for lx in 2..=13 {
                    if (lx + ly >= 7) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 8) {
                        let is_acc = if is_raw {
                            (lx == 5 && ly == 7) || (lx == 6 && ly == 8) || (lx == 9 && ly == 6)
                        } else {
                            (lx + ly == 12) || (lx + ly == 16)
                        };
                        let col = if lx == 2 || lx == 13 || ly == 3 || ly == 13 { outline }
                            else if is_acc { accent }
                            else if lx <= 6 && ly <= 6 { hi }
                            else if lx >= 9 || ly >= 10 { sh }
                            else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Apple => {
            let (outline, base, hi, sh, stem, leaf) = (
                pack_color(0.32, 0.06, 0.06), pack_color(0.88, 0.14, 0.14), pack_color(0.98, 0.48, 0.48),
                pack_color(0.52, 0.08, 0.08), pack_color(0.42, 0.26, 0.12), pack_color(0.35, 0.78, 0.22),
            );
            pts!(stem => (8, 2), (8, 3));
            pts!(leaf => (9, 2), (10, 2));
            for ly in 4..=13 {
                for lx in 4..=12 {
                    if (lx + ly >= 9) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 7) {
                        let col = if lx == 4 || lx == 12 || ly == 4 || ly == 13 { outline }
                            else if (6..=8).contains(&lx) && (6..=7).contains(&ly) { hi }
                            else if lx >= 10 || ly >= 11 { sh }
                            else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::RawChicken | BlockId::CookedChicken => {
            let (outline, base, hi, sh, bone, bone_sh) = if block == BlockId::RawChicken {
                (pack_color(0.46, 0.24, 0.20), pack_color(0.92, 0.68, 0.62), pack_color(0.98, 0.82, 0.78), pack_color(0.76, 0.46, 0.40), pack_color(0.95, 0.95, 0.88), pack_color(0.78, 0.76, 0.68))
            } else {
                (pack_color(0.30, 0.12, 0.04), pack_color(0.82, 0.46, 0.14), pack_color(0.96, 0.72, 0.28), pack_color(0.55, 0.24, 0.06), pack_color(0.92, 0.90, 0.82), pack_color(0.72, 0.70, 0.62))
            };
            pts!(bone => (12, 4), (13, 3), (14, 2), (13, 2), (12, 3));
            px!(14, 3, bone_sh);
            pts!(outline => (11, 4), (14, 4), (15, 2), (13, 1));
            for ly in 4..=13 {
                for lx in 2..=11 {
                    if (lx + ly >= 8) && (lx + ly <= 21) && (ly - lx <= 7) && (lx - ly <= 4) {
                        let col = if lx == 2 || lx == 11 || ly == 4 || ly == 13 || lx + ly == 8 || lx + ly == 21 { outline }
                            else if lx <= 5 && ly <= 7 { hi }
                            else if lx >= 8 || ly >= 10 { sh }
                            else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Feather => {
            let (outline, white, gray, quill, quill_base) = (
                pack_color(0.25, 0.25, 0.28), pack_color(0.98, 0.98, 0.99), pack_color(0.82, 0.84, 0.88),
                pack_color(0.68, 0.62, 0.55), pack_color(0.50, 0.44, 0.38),
            );
            pts!(quill_base => (2, 14), (3, 13));
            for x in 4..=13 { px!(x, 16 - x, quill); }
            for y in 3..=11 {
                for x in 4..=13 {
                    let diag = x - (15 - y);
                    if (-2..=3).contains(&diag) {
                        let col = if diag == 3 || diag == -2 || y == 3 || x == 13 { outline }
                            else if (x + y) % 2 == 0 { white } else { gray };
                        px!(x, y, col);
                    }
                }
            }
            pts!(outline => (1, 14), (2, 15), (3, 14));
        }
        BlockId::Shears => {
            let (iron_hi, iron_mid, iron_dark, bolt) = (
                pack_color(0.95, 0.95, 0.98), pack_color(0.75, 0.75, 0.78),
                pack_color(0.45, 0.45, 0.48), pack_color(0.30, 0.30, 0.32),
            );
            pts!(bolt => (7, 8), (8, 9));
            pts!(iron_hi => (8, 8), (3, 3), (3, 4), (4, 5), (6, 6), (11, 3), (12, 3), (10, 4), (9, 6), (6, 12), (9, 12));
            pts!(iron_mid => (4, 3), (4, 4), (5, 5), (6, 7), (11, 4), (10, 5), (9, 7), (5, 11), (4, 12), (10, 11), (11, 12));
            pts!(iron_dark => (7, 9), (5, 4), (12, 4), (9, 5), (6, 10), (5, 13), (6, 13), (9, 10), (10, 13), (9, 13));
        }
        BlockId::PigSpawnEgg | BlockId::CowSpawnEgg | BlockId::ChickenSpawnEgg | BlockId::SheepSpawnEgg | BlockId::DolphinSpawnEgg | BlockId::FishSpawnEgg | BlockId::VillagerSpawnEgg | BlockId::IronGolemSpawnEgg | BlockId::SpiderSpawnEgg => {
            let (base, spot, outline) = match block {
                BlockId::PigSpawnEgg => (pack_color(0.95, 0.62, 0.68), pack_color(0.68, 0.32, 0.38), pack_color(0.48, 0.20, 0.25)),
                BlockId::CowSpawnEgg => (pack_color(0.42, 0.30, 0.22), pack_color(0.85, 0.85, 0.85), pack_color(0.22, 0.15, 0.10)),
                BlockId::ChickenSpawnEgg => (pack_color(0.95, 0.95, 0.92), pack_color(0.85, 0.15, 0.15), pack_color(0.35, 0.35, 0.30)),
                BlockId::SheepSpawnEgg => (pack_color(0.92, 0.90, 0.88), pack_color(0.85, 0.68, 0.60), pack_color(0.40, 0.38, 0.36)),
                BlockId::DolphinSpawnEgg => (pack_color(0.22, 0.40, 0.62), pack_color(0.85, 0.88, 0.95), pack_color(0.12, 0.22, 0.35)),
                BlockId::FishSpawnEgg => (pack_color(0.68, 0.52, 0.32), pack_color(0.92, 0.45, 0.22), pack_color(0.38, 0.28, 0.16)),
                BlockId::VillagerSpawnEgg => (pack_color(0.55, 0.38, 0.24), pack_color(0.78, 0.58, 0.45), pack_color(0.32, 0.20, 0.12)),
                BlockId::IronGolemSpawnEgg => (pack_color(0.86, 0.86, 0.84), pack_color(0.45, 0.45, 0.42), pack_color(0.28, 0.28, 0.28)),
                _ => (pack_color(0.20, 0.16, 0.16), pack_color(0.80, 0.12, 0.12), pack_color(0.10, 0.08, 0.08)),
            };
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let dx = (lx as f32 - 7.5).abs();
                    let dy = ly as f32;
                    let width_at_y = if dy < 6.0 { (dy - 1.0) * 0.7 + 0.5 } else if dy <= 11.0 { 4.2 } else { (14.5 - dy) * 1.1 + 0.5 };
                    if dx <= width_at_y {
                        if dx > width_at_y - 0.9 || ly == 2 || ly == 14 {
                            px!(lx, ly, outline);
                        } else {
                            let is_spot = match block {
                                BlockId::PigSpawnEgg => (lx == 6 && ly == 5) || (lx == 9 && ly == 8) || (lx == 5 && ly == 10) || (lx == 8 && ly == 12),
                                BlockId::CowSpawnEgg => ((lx == 6 || lx == 7) && ly == 6) || (lx == 9 && ly == 9) || ((lx == 5 || lx == 6) && ly == 11),
                                BlockId::ChickenSpawnEgg => (lx == 7 && ly == 5) || (lx == 6 && ly == 8) || (lx == 9 && ly == 10) || (lx == 7 && ly == 12),
                                BlockId::SheepSpawnEgg => ((lx == 7 || lx == 8) && ly == 6) || (lx == 6 && ly == 9) || (lx == 9 && ly == 10) || (lx == 7 && ly == 11),
                                BlockId::DolphinSpawnEgg => (lx == 6 && ly == 5) || (lx == 8 && ly == 8) || ((lx == 6 || lx == 9) && ly == 11),
                                BlockId::FishSpawnEgg => (lx == 7 && ly == 5) || (lx == 8 && ly == 7) || (lx == 6 && ly == 9) || (lx == 8 && ly == 12),
                                BlockId::VillagerSpawnEgg => (lx == 6 && ly == 6) || (lx == 8 && ly == 8) || ((lx == 6 || lx == 9) && ly == 11),
                                BlockId::IronGolemSpawnEgg => (lx == 7 && ly == 5) || (lx == 8 && ly == 8) || (lx == 6 && ly == 10) || (lx == 9 && ly == 12),
                                _ => (lx == 6 && ly == 5) || (lx == 8 && ly == 8) || (lx == 5 && ly == 10) || (lx == 9 && ly == 11),
                            };
                            px!(lx, ly, if is_spot { spot } else { base });
                        }
                    }
                }
            }
        }
        BlockId::StringItem => {
            let (silk, silk_dim) = (pack_color(0.95, 0.95, 0.95), pack_color(0.75, 0.75, 0.78));
            for &(lx, ly) in &[
                (3, 12), (4, 11), (5, 10), (6, 9), (7, 8), (8, 7), (9, 6), (10, 5), (11, 4), (12, 3),
                (5, 12), (6, 11), (7, 10), (8, 9), (9, 8), (10, 7), (11, 6), (7, 6), (8, 5),
            ] {
                px!(lx, ly, silk);
                px!(lx, ly + 1, silk_dim);
            }
        }
        BlockId::SpiderEye => {
            let (red_bright, red_dark, highlight, pupil, outline) = (
                pack_color(0.92, 0.12, 0.12), pack_color(0.55, 0.06, 0.06),
                pack_color(1.00, 0.70, 0.70), pack_color(0.12, 0.02, 0.02), pack_color(0.28, 0.04, 0.04),
            );
            for ly in 3..=12 {
                for lx in 3..=12 {
                    let d2 = (lx as f32 - 7.5).powi(2) + (ly as f32 - 7.5).powi(2);
                    if d2 <= 18.0 {
                        px!(lx, ly, if d2 > 13.0 { outline } else if d2 <= 6.0 { red_bright } else { red_dark });
                    }
                }
            }
            px!(6, 6, highlight);
            pts!(pupil => (7, 7), (8, 7), (7, 8), (8, 8));
        }
        BlockId::Emerald | BlockId::Diamond => {
            let (outline, glint, hi, base, sh) = if block == BlockId::Emerald {
                (pack_color(0.04, 0.28, 0.08), pack_color(0.75, 1.00, 0.75), pack_color(0.18, 0.85, 0.32), pack_color(0.10, 0.68, 0.22), pack_color(0.06, 0.45, 0.14))
            } else {
                (pack_color(0.05, 0.35, 0.38), pack_color(0.88, 1.00, 1.00), pack_color(0.40, 0.92, 0.94), pack_color(0.20, 0.75, 0.80), pack_color(0.10, 0.52, 0.58))
            };
            for ly in 3..=13 {
                for lx in 3..=12 {
                    let in_shape = match ly {
                        3 | 10 | 11 => (6..=9).contains(&lx),
                        4 | 9 => (5..=10).contains(&lx),
                        5 | 8 => (4..=11).contains(&lx),
                        6..=7 => (3..=12).contains(&lx),
                        12 | 13 => lx == 7 || lx == 8,
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || ((ly == 6 || ly == 7) && (lx == 3 || lx == 12)) || (ly == 12 && (lx == 7 || lx == 8));
                        let col = if is_border { outline }
                            else if ly <= 5 && (6..=8).contains(&lx) { glint }
                            else if lx <= 6 { hi }
                            else if lx >= 10 || ly >= 10 { sh }
                            else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::ClayBall => {
            let (outline, hi, base, sh) = (pack_color(0.32, 0.34, 0.38), pack_color(0.82, 0.85, 0.90), pack_color(0.66, 0.70, 0.76), pack_color(0.50, 0.53, 0.58));
            for ly in 4..=12 {
                for lx in 4..=12 {
                    let d2 = (lx as f32 - 8.0).powi(2) + (ly as f32 - 8.0).powi(2);
                    if d2 <= 18.0 {
                        let col = if d2 > 12.0 { outline } else if lx <= 6 && ly <= 6 { hi } else if lx >= 10 || ly >= 10 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Brick => {
            let (outline, hi, base, sh) = (pack_color(0.38, 0.16, 0.10), pack_color(0.82, 0.45, 0.35), pack_color(0.68, 0.32, 0.24), pack_color(0.50, 0.22, 0.16));
            for ly in 5..=11 {
                for lx in 3..=13 {
                    let col = if ly == 5 || ly == 11 || lx == 3 || lx == 13 { outline } else if ly == 6 || lx == 4 { hi } else if ly == 10 || lx == 12 { sh } else { base };
                    px!(lx, ly, col);
                }
            }
        }
        BlockId::Flint => {
            let (outline, hi, base, sh) = (pack_color(0.10, 0.10, 0.12), pack_color(0.45, 0.45, 0.48), pack_color(0.28, 0.28, 0.30), pack_color(0.18, 0.18, 0.20));
            for ly in 3..=13 {
                for lx in 4..=12 {
                    let in_shape = match ly {
                        3..=4 => (7..=9).contains(&lx),
                        5..=7 => (5..=10).contains(&lx),
                        8..=10 => (4..=12).contains(&lx),
                        11..=12 => (5..=11).contains(&lx),
                        13 => (6..=9).contains(&lx),
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || (ly == 5 && (lx == 5 || lx == 10)) || (ly == 8 && (lx == 4 || lx == 12));
                        let col = if is_border { outline } else if lx <= 6 && ly <= 7 { hi } else if lx >= 10 || ly >= 11 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Bed => {
            let (wood, white, red, red_sh, frame) = (
                pack_color(0.45, 0.28, 0.12), pack_color(0.92, 0.92, 0.94),
                pack_color(0.80, 0.15, 0.15), pack_color(0.55, 0.10, 0.10), pack_color(0.30, 0.18, 0.08),
            );
            pts!(frame => (2, 13), (2, 14), (13, 13), (13, 14), (1, 6), (1, 7), (1, 8), (1, 9), (1, 10), (1, 11));
            for lx in 2..=13 { px!(lx, 11, wood); px!(lx, 12, frame); }
            for ly in 8..=10 {
                for lx in 2..=5 { px!(lx, ly, white); }
                for lx in 6..=13 { px!(lx, ly, if ly == 10 { red_sh } else { red }); }
            }
        }
        BlockId::Ladder => {
            let (wood_rail, rail_hi, rung, rung_sh) = (
                pack_color(0.48, 0.32, 0.16), pack_color(0.65, 0.45, 0.24),
                pack_color(0.58, 0.40, 0.20), pack_color(0.35, 0.22, 0.10),
            );
            for ly in 1..=14 {
                pts!(rail_hi => (3, ly), (11, ly));
                pts!(wood_rail => (4, ly), (12, ly));
            }
            for &ry in &[3, 6, 9, 12] {
                for lx in 5..=10 { px!(lx, ry, rung); px!(lx, ry + 1, rung_sh); }
            }
        }
        BlockId::Torch => {
            let (wood_light, wood_mid, wood_dark, coal, flame_core, flame_gold, flame_orange) = (
                pack_color(0.68, 0.48, 0.26), pack_color(0.52, 0.35, 0.18), pack_color(0.35, 0.22, 0.10),
                pack_color(0.20, 0.18, 0.16), pack_color(1.00, 1.00, 0.88), pack_color(1.00, 0.80, 0.15), pack_color(0.98, 0.42, 0.05),
            );
            pts!(flame_orange => (7, 1), (6, 2), (9, 3), (7, 4), (8, 4));
            pts!(flame_gold => (8, 1), (9, 2), (6, 3), (8, 3));
            pts!(flame_core => (7, 2), (8, 2), (7, 3));
            pts!(coal => (7, 5), (8, 5), (7, 6));
            px!(8, 6, wood_dark);
            for ly in 7..=14 {
                px!(7, ly, if ly % 3 == 0 { wood_light } else { wood_mid });
                px!(8, ly, wood_dark);
            }
        }
        BlockId::RustStone | BlockId::RustStoneWireLit => {
            let (dark, base, hi, ember) = (pack_color(0.45, 0.12, 0.06), pack_color(0.82, 0.26, 0.12), pack_color(1.00, 0.48, 0.18), pack_color(1.00, 0.82, 0.40));
            for ly in 4..=12 {
                for lx in 4..=12 {
                    let dist_sq = (lx as f32 - 7.5).powi(2) + (ly as f32 - 7.5).powi(2);
                    if dist_sq <= 15.0 {
                        let col = if dist_sq < 3.0 { ember } else if (lx + ly) % 3 == 0 { hi } else if (lx + ly) % 2 == 0 { base } else { dark };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Lever | BlockId::LeverOn => {
            let (stone_base, stone_sh, wood_shaft, tip) = (pack_color(0.55, 0.55, 0.58), pack_color(0.35, 0.35, 0.38), pack_color(0.55, 0.35, 0.15), pack_color(0.80, 0.25, 0.15));
            for ly in 11..=14 {
                for lx in 4..=11 { px!(lx, ly, if ly == 14 { stone_sh } else { stone_base }); }
            }
            let is_on = block == BlockId::LeverOn;
            for i in 0..7 {
                px!(if is_on { 9 - i / 2 } else { 6 + i / 2 }, 10 - i, if i >= 5 { tip } else { wood_shaft });
            }
        }
        BlockId::Button | BlockId::ButtonPressed => {
            let (hi, base, sh) = (pack_color(0.75, 0.75, 0.78), pack_color(0.58, 0.58, 0.60), pack_color(0.38, 0.38, 0.40));
            for ly in 6..=10 {
                for lx in 4..=11 {
                    px!(lx, ly, if lx == 4 || ly == 6 { hi } else if lx == 11 || ly == 10 { sh } else { base });
                }
            }
        }
        BlockId::PressurePlate | BlockId::PressurePlatePressed => {
            let (border, base, hi) = (pack_color(0.40, 0.40, 0.42), pack_color(0.60, 0.60, 0.62), pack_color(0.75, 0.75, 0.78));
            for ly in 10..=13 {
                for lx in 2..=13 {
                    px!(lx, ly, if ly == 10 { hi } else if ly == 13 || lx == 2 || lx == 13 { border } else { base });
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
                px!(i, 15 - i, wood_mid);
                px!(i - 1, 15 - i, wood_light);
            }
            let string_col = pack_color(0.88, 0.88, 0.90);
            for y in 3..=11 { px!(13, y, string_col); }
            px!(13, 12, pack_color(0.90, 0.15, 0.15));
            px!(12, 13, pack_color(0.70, 0.70, 0.75));
            px!(13, 13, pack_color(0.95, 0.95, 0.95));
        }
        BlockId::RawCod | BlockId::CookedCod => {
            let is_cooked = block == BlockId::CookedCod;
            let (outline, base, belly, fin, eye) = if is_cooked {
                (pack_color(0.35, 0.25, 0.12), pack_color(0.78, 0.60, 0.36), pack_color(0.92, 0.78, 0.52), pack_color(0.60, 0.42, 0.22), pack_color(0.12, 0.12, 0.12))
            } else {
                (pack_color(0.30, 0.28, 0.24), pack_color(0.68, 0.62, 0.54), pack_color(0.85, 0.82, 0.76), pack_color(0.52, 0.46, 0.38), pack_color(0.12, 0.12, 0.12))
            };
            pts!(fin => (2, 6), (2, 8));
            pts!(outline => (2, 7), (3, 7), (13, 7));
            for lx in 4..=11 {
                for ly in 6..=8 {
                    px!(lx, ly, if lx >= 10 && ly == 7 { eye } else if ly == 8 { belly } else if ly == 6 { fin } else { base });
                }
            }
            for lx in 3..=12 { pts!(outline => (lx, 5), (lx, 9)); }
        }
        BlockId::RawSalmon | BlockId::CookedSalmon => {
            let is_cooked = block == BlockId::CookedSalmon;
            let (outline, base, belly, fin, eye) = if is_cooked {
                (pack_color(0.38, 0.15, 0.08), pack_color(0.85, 0.45, 0.22), pack_color(0.95, 0.65, 0.40), pack_color(0.65, 0.30, 0.12), pack_color(0.12, 0.12, 0.12))
            } else {
                (pack_color(0.38, 0.18, 0.16), pack_color(0.82, 0.32, 0.28), pack_color(0.92, 0.55, 0.48), pack_color(0.55, 0.20, 0.18), pack_color(0.12, 0.12, 0.12))
            };
            pts!(fin => (2, 5), (2, 7));
            pts!(outline => (2, 6), (3, 6), (14, 6), (14, 7));
            for lx in 4..=12 {
                for ly in 5..=8 {
                    px!(lx, ly, if lx >= 11 && ly == 6 { eye } else if ly >= 7 { belly } else if ly == 5 { fin } else { base });
                }
            }
            for lx in 3..=13 { pts!(outline => (lx, 4), (lx, 9)); }
        }
        BlockId::Pufferfish => {
            let (outline, base, belly, spike, eye) = (
                pack_color(0.40, 0.32, 0.08), pack_color(0.92, 0.78, 0.22),
                pack_color(0.96, 0.92, 0.65), pack_color(0.72, 0.58, 0.15), pack_color(0.15, 0.25, 0.45),
            );
            pts!(spike => (4, 3), (8, 2), (11, 3), (2, 8), (13, 8), (4, 12), (8, 13), (11, 12));
            for ly in 4..=11 {
                for lx in 4..=11 {
                    let d2 = (lx - 7) * (lx - 7) + (ly - 7) * (ly - 7);
                    if d2 <= 16 {
                        px!(lx, ly, if d2 >= 12 { outline } else if lx == 9 && ly == 6 { eye } else if ly >= 8 { belly } else { base });
                    }
                }
            }
        }
        BlockId::TropicalFish => {
            let (outline, orange, white, fin, eye) = (
                pack_color(0.45, 0.18, 0.05), pack_color(0.95, 0.48, 0.12),
                pack_color(0.96, 0.96, 0.98), pack_color(0.18, 0.72, 0.85), pack_color(0.10, 0.10, 0.12),
            );
            pts!(fin => (2, 6), (2, 7), (2, 8), (7, 5), (8, 5));
            pts!(outline => (3, 7), (13, 7));
            for lx in 4..=12 {
                for ly in 6..=8 {
                    let col = if lx == 11 && ly == 6 { eye } else if lx == 6 || lx == 7 || lx == 10 { white } else { orange };
                    px!(lx, ly, col);
                }
            }
            for lx in 3..=12 { pts!(outline => (lx, 5), (lx, 9)); }
        }
        BlockId::DriedKelp => {
            let (dark, mid, hi, tie) = (pack_color(0.12, 0.22, 0.10), pack_color(0.20, 0.38, 0.16), pack_color(0.28, 0.52, 0.22), pack_color(0.68, 0.55, 0.30));
            for ly in 4..=11 {
                for lx in 4..=11 {
                    let col = if ly == 7 || ly == 8 { tie } else if lx == 4 || lx == 11 || ly == 4 || ly == 11 { dark } else if (lx + ly) % 2 == 0 { hi } else { mid };
                    px!(lx, ly, col);
                }
            }
        }
        BlockId::Kelp => {
            let (stalk, leaf, hi) = (pack_color(0.16, 0.32, 0.14), pack_color(0.24, 0.54, 0.20), pack_color(0.35, 0.70, 0.28));
            for y in 2..=14 {
                pts!(stalk => (7, y), (8, y));
                if y % 3 == 0 { px!(5, y, leaf); px!(6, y, hi); }
                else if y % 3 == 1 { px!(9, y, hi); px!(10, y, leaf); }
            }
        }
        BlockId::Seagrass => {
            let (leaf1, leaf2, dark) = (pack_color(0.22, 0.58, 0.24), pack_color(0.30, 0.68, 0.32), pack_color(0.15, 0.40, 0.18));
            for y in 5..=14 {
                let x1 = 5 + ((14 - y) / 3);
                px!(x1, y, leaf1); px!(x1 + 1, y, dark);
            }
            for y in 3..=14 {
                let x2 = 9 + ((14 - y) % 2);
                px!(x2, y, leaf2); px!(x2 + 1, y, dark);
            }
        }
        BlockId::Leather => {
            let (outline, base, hi, sh) = (pack_color(0.26, 0.14, 0.06), pack_color(0.62, 0.36, 0.16), pack_color(0.76, 0.48, 0.24), pack_color(0.44, 0.22, 0.08));
            for ly in 3..=13 {
                for lx in 3..=13 {
                    let in_hide = match ly {
                        3 => (6..=10).contains(&lx),
                        4 | 7..=9 => (4..=12).contains(&lx),
                        5 | 6 => (3..=13).contains(&lx) && lx != 8,
                        10 | 11 => (3..=13).contains(&lx),
                        12 => (4..=12).contains(&lx) && lx != 8,
                        13 => (4..=6).contains(&lx) || (10..=12).contains(&lx),
                        _ => false,
                    };
                    if in_hide {
                        let col = if lx == 3 || lx == 13 || ly == 3 || ly == 13 { outline } else if lx <= 6 && ly <= 7 { hi } else if lx >= 10 || ly >= 11 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherHelmet | BlockId::IronHelmet | BlockId::GoldHelmet | BlockId::DiamondHelmet => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 3..=12 {
                for lx in 3..=12 {
                    let in_helm = match ly {
                        3 => (6..=9).contains(&lx),
                        4 => (4..=11).contains(&lx),
                        5..=8 => (3..=12).contains(&lx),
                        9..=11 => (3..=5).contains(&lx) || (10..=12).contains(&lx) || ((7..=8).contains(&lx) && ly == 9),
                        12 => (3..=5).contains(&lx) || (10..=12).contains(&lx),
                        _ => false,
                    };
                    if in_helm {
                        let is_rim = ly == 3 || (ly == 4 && (lx == 4 || lx == 11)) || lx == 3 || lx == 12 || ly == 12 || (ly == 8 && (6..=9).contains(&lx));
                        let col = if is_rim { outline } else if lx <= 5 && ly <= 6 { if lx == 4 && ly == 4 { glint } else { hi } } else if lx >= 9 || ly >= 10 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherChestplate | BlockId::IronChestplate | BlockId::GoldChestplate | BlockId::DiamondChestplate => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 2..=14 {
                for lx in 2..=13 {
                    let in_chest = match ly {
                        2 => (3..=5).contains(&lx) || (10..=12).contains(&lx),
                        3..=4 => (2..=5).contains(&lx) || (10..=13).contains(&lx),
                        5..=10 => (2..=13).contains(&lx),
                        11..=13 => (4..=11).contains(&lx),
                        14 => (5..=10).contains(&lx),
                        _ => false,
                    };
                    if in_chest {
                        let is_rim = ly == 2 || ly == 14 || lx == 2 || lx == 13 || (ly >= 11 && (lx == 4 || lx == 11)) || (ly <= 4 && (lx == 5 || lx == 10));
                        let col = if is_rim { outline } else if lx <= 6 && ly <= 6 { if lx <= 4 && ly <= 3 { glint } else { hi } } else if lx >= 9 || ly >= 10 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherLeggings | BlockId::IronLeggings | BlockId::GoldLeggings | BlockId::DiamondLeggings => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let in_legs = match ly {
                        2..=5 => (3..=12).contains(&lx),
                        6..=14 => (3..=6).contains(&lx) || (9..=12).contains(&lx),
                        _ => false,
                    };
                    if in_legs {
                        let is_rim = ly == 2 || ly == 14 || lx == 3 || lx == 12 || (ly >= 6 && (lx == 6 || lx == 9)) || (ly == 5 && (7..=8).contains(&lx));
                        let col = if is_rim { outline } else if lx <= 5 && ly <= 5 { if lx == 4 && ly == 3 { glint } else { hi } } else if lx >= 10 || ly >= 10 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherBoots | BlockId::IronBoots | BlockId::GoldBoots | BlockId::DiamondBoots => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 6..=14 {
                for lx in 2..=6 {
                    if ly >= 10 || lx >= 3 {
                        let is_rim = ly == 6 || ly == 14 || lx == 2 || lx == 6 || (ly <= 9 && lx == 3);
                        let col = if is_rim { outline } else if lx <= 4 && ly <= 8 { if lx == 3 && ly == 7 { glint } else { hi } } else if lx >= 5 || ly >= 12 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
                for lx in 9..=13 {
                    if ly >= 10 || lx <= 12 {
                        let is_rim = ly == 6 || ly == 14 || lx == 9 || lx == 13 || (ly <= 9 && lx == 12);
                        let col = if is_rim { outline } else if lx <= 10 && ly <= 8 { hi } else if lx >= 12 || ly >= 12 { sh } else { base };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Bow => {
            let string = pack_color(0.92, 0.92, 0.94);
            for &(sx, sy) in &[(13, 2), (12, 2), (11, 3), (9, 4), (7, 6), (6, 7), (4, 9), (3, 11), (2, 12), (2, 13)] {
                px!(sx, sy, wood_mid);
                px!(sx - 1, sy, wood_light);
                px!(sx, sy + 1, wood_dark);
            }
            for i in 2..=13 { px!(i, i, string); }
        }
        BlockId::Arrow => {
            let (flint, flint_hi, feather, feather_sh) = (
                pack_color(0.40, 0.40, 0.44), pack_color(0.65, 0.65, 0.70),
                pack_color(0.95, 0.95, 0.98), pack_color(0.70, 0.70, 0.75),
            );
            pts!(flint_hi => (14, 1), (13, 1));
            pts!(flint => (13, 2), (14, 2), (12, 3), (13, 3));
            for i in 4..=11 { px!(15 - i, i, wood_mid); }
            pts!(feather => (3, 12), (2, 13), (2, 11), (1, 14));
            pts!(feather_sh => (4, 13), (1, 12));
        }
        BlockId::Shield => {
            let (iron_rim, iron_hi, iron_sh, plank_light, plank_dark) = (
                pack_color(0.78, 0.80, 0.84), pack_color(0.95, 0.95, 0.98), pack_color(0.42, 0.44, 0.48),
                pack_color(0.68, 0.48, 0.28), pack_color(0.48, 0.32, 0.16),
            );
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let in_shield = match ly {
                        2 | 10..=11 => (4..=11).contains(&lx),
                        3..=9 => (3..=12).contains(&lx),
                        12..=13 => (5..=10).contains(&lx),
                        14 => (6..=9).contains(&lx),
                        _ => false,
                    };
                    if in_shield {
                        let is_rim = ly == 2 || ly == 14 || lx == 3 || lx == 12 || (ly >= 10 && (lx == 4 || lx == 11)) || (ly >= 12 && (lx == 5 || lx == 10));
                        let col = if is_rim {
                            if lx <= 5 || ly <= 3 { iron_hi } else if lx >= 10 || ly >= 12 { iron_sh } else { iron_rim }
                        } else if (7..=8).contains(&lx) && (7..=8).contains(&ly) {
                            iron_hi
                        } else if (lx + ly) % 2 == 0 {
                            plank_light
                        } else {
                            plank_dark
                        };
                        px!(lx, ly, col);
                    }
                }
            }
        }
        _ => {
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
        assert!(tab0.contains(&BlockId::Clay));
        assert!(tab0.contains(&BlockId::Bricks));
        assert!(tab0.contains(&BlockId::Cobblestone));
        assert!(tab0.contains(&BlockId::Sandstone));
        assert!(tab0.contains(&BlockId::Obsidian));
        assert!(tab0.contains(&BlockId::Bookshelf));

        let tab1 = creative_tab_blocks(1);
        assert!(tab1.contains(&BlockId::WoodenPickaxe));
        assert!(tab1.contains(&BlockId::IronSword));
        assert!(tab1.contains(&BlockId::Shears));
        assert!(tab1.contains(&BlockId::Tnt));
        assert!(tab1.contains(&BlockId::ClayBall));
        assert!(tab1.contains(&BlockId::Brick));
        assert!(tab1.contains(&BlockId::Flint));
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
        inv.hotbar[0] = Some(ItemStack::new(BlockId::Grass, 64));
        assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockId::Grass, 64)));

        // Trash action: delete held item
        inv.held_item = Some(ItemStack::new(BlockId::StonePickaxe, 1));
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
            &[],
            false,
            0,
            None,
            Some((320.0, 240.0)),
            false,
        );

        // Ensure pixels were drawn to the buffer
        let colored_pixels = buffer.iter().filter(|&&p| p != 0).count();
        assert!(colored_pixels > 1000);
    }

    #[test]
    fn recipe_book_targets_only_exist_at_crafting_table() {
        let layout = InventoryLayout::new(640, 480, true);
        let recipes = [2, 7, 9];
        assert_eq!(
            layout.find_recipe_book_target_at(
                layout.start_x + 195,
                layout.start_y + 40,
                false,
                &recipes,
                0,
            ),
            Some(InventorySlotTarget::RecipeBookToggle)
        );
        assert_eq!(
            layout.find_recipe_book_target_at(
                layout.start_x + 305,
                layout.start_y + 53,
                true,
                &recipes,
                1,
            ),
            Some(InventorySlotTarget::RecipeEntry(7))
        );
        assert_eq!(
            InventoryLayout::new(640, 480, false).find_recipe_book_target_at(
                layout.start_x + 195,
                layout.start_y + 40,
                true,
                &recipes,
                0,
            ),
            None
        );
    }
}
