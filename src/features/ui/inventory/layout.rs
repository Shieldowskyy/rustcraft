//! Układ siatki slotów ekwipunku, zakładki trybu kreatywnego oraz hit-testing myszy/kontrolera.

use crate::features::blocks::BlockId;
use crate::features::player::{GameMode, Player};
use crate::features::ui::widgets::{Grid, Rect};

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
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    macro_rules! blk_vec {
        ($($b:ident),* $(,)?) => {
            vec![$(BlockId::$b),*]
        };
    }
    if tab >= 5 {
        return &[];
    }
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
