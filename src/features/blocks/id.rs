//! Core block/item enum (`BlockId`), armor slots (`ArmorSlot`), face alias (`Face`), and wool helpers.
//! Defines numeric `0..=185` vanilla IDs, `Custom(u8)` mod IDs, and i18n display names.

use super::constants::CUSTOM_BLOCK_ID_START;

pub use crate::core::coords::BlockFace as Face;

macro_rules! define_blocks {
    ($(($id:literal, $variant:ident, $name_pl:literal, $name_en:literal)),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum BlockId {
            $($variant,)*
            /// Custom modded block or item (`ID 186..=255`).
            Custom(u8),
        }

        impl BlockId {
            pub const VANILLA_ALL: [BlockId; 186] = [$(Self::$variant),*];

            #[inline(always)]
            pub const fn from_u8(v: u8) -> BlockId {
                match v {
                    $($id => BlockId::$variant,)*
                    v if v >= CUSTOM_BLOCK_ID_START => BlockId::Custom(v),
                    _ => BlockId::Air,
                }
            }

            #[inline(always)]
            pub const fn to_u8(self) -> u8 {
                match self {
                    $(BlockId::$variant => $id,)*
                    BlockId::Custom(id) => id,
                }
            }

            pub fn name(self) -> &'static str {
                match self {
                    $(BlockId::$variant => $name_pl,)*
                    BlockId::Custom(id) => crate::engine::modding::get_custom_name(id),
                }
            }

            /// English display name for each block/item.
            pub fn name_en(self) -> &'static str {
                match self {
                    $(BlockId::$variant => $name_en,)*
                    BlockId::Custom(id) => crate::engine::modding::get_custom_name(id),
                }
            }

            pub fn variant_name(self) -> &'static str {
                match self {
                    $(BlockId::$variant => stringify!($variant),)*
                    BlockId::Custom(_) => "Custom",
                }
            }
        }
    };
}

define_blocks! {
    (0, Air, "powietrze", "Air"),
    (1, Grass, "trawa", "Grass Block"),
    (2, Dirt, "ziemia", "Dirt"),
    (3, Stone, "kamien", "Stone"),
    (4, Sand, "piasek", "Sand"),
    (5, Wood, "drewno", "Oak Wood"),
    (6, Leaves, "liscie", "Oak Leaves"),
    (7, Snow, "snieg", "Snow"),
    (8, Cactus, "kaktus", "Cactus"),
    (9, CoalOre, "wegiel", "Coal Ore"),
    (10, IronOre, "zelazo", "Iron Ore"),
    (11, Water, "woda", "Water"),
    (12, Bedrock, "bedrock", "Bedrock"),
    (13, Lava, "lawa", "Lava"),
    (14, SnowLeaves, "liscie zimowe", "Snow Leaves"),
    (15, Planks, "deski", "Oak Planks"),
    (16, Door, "drzwi", "Wooden Door"),
    (17, Glass, "szyba", "Glass"),
    (18, Ice, "lod", "Ice"),
    (19, Tnt, "TNT", "TNT"),
    (20, CraftingTable, "stol rzemieslniczy", "Crafting Table"),
    (21, Stick, "patyk", "Stick"),
    (22, WoodenPickaxe, "drewniany kilof", "Wooden Pickaxe"),
    (23, StonePickaxe, "kamienny kilof", "Stone Pickaxe"),
    (24, IronPickaxe, "zelazny kilof", "Iron Pickaxe"),
    (25, WoodenAxe, "drewniany topor", "Wooden Axe"),
    (26, StoneAxe, "kamienny topor", "Stone Axe"),
    (27, IronAxe, "zelazny topor", "Iron Axe"),
    (28, WoodenShovel, "drewniana lopata", "Wooden Shovel"),
    (29, StoneShovel, "kamienna lopata", "Stone Shovel"),
    (30, IronShovel, "zelazna lopata", "Iron Shovel"),
    (31, WoodenSword, "drewniany miecz", "Wooden Sword"),
    (32, StoneSword, "kamienny miecz", "Stone Sword"),
    (33, IronSword, "zelazny miecz", "Iron Sword"),
    (34, CherryLeaves, "rozowe liscie", "Cherry Leaves"),
    (35, Mud, "bloto", "Mud"),
    (36, AutumnLeaves, "jesienne liscie", "Autumn Leaves"),
    (37, Torch, "pochodnia", "Torch"),
    (38, RawPorkchop, "surowa wieprzowina", "Raw Porkchop"),
    (39, CookedPorkchop, "pieczona wieprzowina", "Cooked Porkchop"),
    (40, RawBeef, "surowa wolowina", "Raw Beef"),
    (41, CookedBeef, "stek", "Steak"),
    (42, Apple, "jablko", "Apple"),
    (43, DoorOpen, "drzwi (otwarte)", "Wooden Door (Open)"),
    (44, Chest, "skrzynia", "Chest"),
    (45, Furnace, "piec", "Furnace"),
    (46, FurnaceLit, "rozpalony piec", "Lit Furnace"),
    (47, IronIngot, "sztabka zelaza", "Iron Ingot"),
    (48, GoldIngot, "sztabka zlota", "Gold Ingot"),
    (49, IronOunce, "uncja zelaza", "Iron Nugget"),
    (50, IronBlock, "blok zelaza", "Block of Iron"),
    (51, GoldOunce, "uncja zlota", "Gold Nugget"),
    (52, GoldBlock, "blok zlota", "Block of Gold"),
    (53, GoldOre, "ruda zlota", "Gold Ore"),
    (54, CoalBlock, "blok wegla", "Block of Coal"),
    (55, RawChicken, "surowy kurczak", "Raw Chicken"),
    (56, CookedChicken, "pieczony kurczak", "Cooked Chicken"),
    (57, Feather, "pioro", "Feather"),
    (58, PigSpawnEgg, "jajko spawnujace swinie", "Spawn Pig"),
    (59, CowSpawnEgg, "jajko spawnujace krowe", "Spawn Cow"),
    (60, ChickenSpawnEgg, "jajko spawnujace kurczaka", "Spawn Chicken"),
    (61, DiamondOre, "ruda diamentu", "Diamond Ore"),
    (62, Diamond, "diament", "Diamond"),
    (63, DiamondBlock, "blok diamentu", "Block of Diamond"),
    (64, DiamondPickaxe, "diamentowy kilof", "Diamond Pickaxe"),
    (65, DiamondAxe, "diamentowy topor", "Diamond Axe"),
    (66, DiamondShovel, "diamentowa lopata", "Diamond Shovel"),
    (67, DiamondSword, "diamentowy miecz", "Diamond Sword"),
    (68, GoldPickaxe, "zloty kilof", "Golden Pickaxe"),
    (69, GoldAxe, "zloty topor", "Golden Axe"),
    (70, GoldShovel, "zlota lopata", "Golden Shovel"),
    (71, GoldSword, "zloty miecz", "Golden Sword"),
    (72, Bed, "lozko", "Bed"),
    (73, WoodenSlab, "drewniany polblok", "Oak Slab"),
    (74, StoneSlab, "kamienny polblok", "Stone Slab"),
    (75, WoodenStairs, "drewniane schody", "Oak Stairs"),
    (76, StoneStairs, "kamienne schody", "Stone Stairs"),
    (77, Ladder, "drabina", "Ladder"),
    (78, Bone, "kosc", "Bone"),
    (79, Gunpowder, "proch", "Gunpowder"),
    (80, WoodenHoe, "drewniana motyka", "Wooden Hoe"),
    (81, StoneHoe, "kamienna motyka", "Stone Hoe"),
    (82, IronHoe, "zelazna motyka", "Iron Hoe"),
    (83, DiamondHoe, "diamentowa motyka", "Diamond Hoe"),
    (84, GoldHoe, "zlota motyka", "Golden Hoe"),
    (85, Farmland, "ziemia uprawna", "Farmland"),
    (86, FarmlandWet, "nawodniona ziemia uprawna", "Moist Farmland"),
    (87, WheatSeeds, "nasiona pszenicy", "Wheat Seeds"),
    (88, WheatCrop, "uprawa pszenicy", "Wheat Crop"),
    (89, Wheat, "pszenica", "Wheat"),
    (90, Bread, "chleb", "Bread"),
    (91, Carrot, "marchewka", "Carrot"),
    (92, CarrotCrop, "uprawa marchewki", "Carrot Crop"),
    (93, Potato, "ziemniak", "Potato"),
    (94, PotatoCrop, "uprawa ziemniakow", "Potato Crop"),
    (95, BakedPotato, "pieczony ziemniak", "Baked Potato"),
    (96, Sapling, "sadzonka debu", "Oak Sapling"),
    (97, BoneMeal, "maczka kostna", "Bone Meal"),
    (98, RustStoneOre, "ruda rdzawnika", "Ruststone Ore"),
    (99, RustStone, "rdzawnik", "Ruststone"),
    (100, RustStoneWireLit, "aktywny rdzawnik", "Active Ruststone"),
    (101, RustStoneBlock, "blok rdzawnika", "Block of Ruststone"),
    (102, RustStoneLamp, "lampa rdzawnikowa", "Ruststone Lamp"),
    (103, RustStoneLampLit, "swiecaca lampa rdzawnikowa", "Lit Ruststone Lamp"),
    (104, Lever, "dzwignia", "Lever"),
    (105, LeverOn, "wlaczona dzwignia", "Active Lever"),
    (106, Button, "przycisk", "Button"),
    (107, ButtonPressed, "wcisniety przycisk", "Pressed Button"),
    (108, PressurePlate, "plytka naciskowa", "Pressure Plate"),
    (109, PressurePlatePressed, "wcisnieta plytka naciskowa", "Pressed Pressure Plate"),
    (110, WhiteWool, "biala welna", "White Wool"),
    (111, OrangeWool, "pomaranczowa welna", "Orange Wool"),
    (112, MagentaWool, "karmazynowa welna", "Magenta Wool"),
    (113, LightBlueWool, "jasnoniebieska welna", "Light Blue Wool"),
    (114, YellowWool, "zolta welna", "Yellow Wool"),
    (115, LimeWool, "jasnozielona welna", "Lime Wool"),
    (116, PinkWool, "rozowa welna", "Pink Wool"),
    (117, GrayWool, "szara welna", "Gray Wool"),
    (118, LightGrayWool, "jasnoszara welna", "Light Gray Wool"),
    (119, CyanWool, "cyjanowa welna", "Cyan Wool"),
    (120, PurpleWool, "fioletowa welna", "Purple Wool"),
    (121, BlueWool, "niebieska welna", "Blue Wool"),
    (122, BrownWool, "brazowa welna", "Brown Wool"),
    (123, GreenWool, "zielona welna", "Green Wool"),
    (124, RedWool, "czerwona welna", "Red Wool"),
    (125, BlackWool, "czarna welna", "Black Wool"),
    (126, SheepSpawnEgg, "jajko owcy", "Spawn Sheep"),
    (127, Shears, "nozyce", "Shears"),
    (128, Kelp, "wodorosty (kelp)", "Kelp"),
    (129, Seagrass, "trawa morska", "Seagrass"),
    (130, RawCod, "surowy dorsz", "Raw Cod"),
    (131, CookedCod, "pieczony dorsz", "Cooked Cod"),
    (132, RawSalmon, "surowy losos", "Raw Salmon"),
    (133, CookedSalmon, "pieczony losos", "Cooked Salmon"),
    (134, Pufferfish, "rozdymka", "Pufferfish"),
    (135, TropicalFish, "tropikalna ryba", "Tropical Fish"),
    (136, DriedKelp, "suszone wodorosty", "Dried Kelp"),
    (137, DolphinSpawnEgg, "jajko delfina", "Spawn Dolphin"),
    (138, FishSpawnEgg, "jajko ryby", "Spawn Fish"),
    (139, FishingRod, "wedka", "Fishing Rod"),
    (140, Emerald, "szmaragd", "Emerald"),
    (141, EmeraldOre, "ruda szmaragdu", "Emerald Ore"),
    (142, EmeraldBlock, "blok szmaragdu", "Emerald Block"),
    (143, VillagerSpawnEgg, "jajko wiesniaka", "Spawn Villager"),
    (144, IronGolemSpawnEgg, "jajko zelaznego golema", "Spawn Iron Golem"),
    (145, Clay, "glina", "Clay Block"),
    (146, ClayBall, "kula gliny", "Clay Ball"),
    (147, Brick, "cegla", "Brick"),
    (148, Bricks, "cegly", "Bricks"),
    (149, BrickStairs, "ceglane schody", "Brick Stairs"),
    (150, BrickSlab, "ceglana plyta", "Brick Slab"),
    (151, Gravel, "zwir", "Gravel"),
    (152, Flint, "krzemien", "Flint"),
    (153, Cobblestone, "bruk", "Cobblestone"),
    (154, MossyCobblestone, "zamszony bruk", "Mossy Cobblestone"),
    (155, CobblestoneStairs, "brukowe schody", "Cobblestone Stairs"),
    (156, CobblestoneSlab, "brukowa plyta", "Cobblestone Slab"),
    (157, Sandstone, "piaskowiec", "Sandstone"),
    (158, SandstoneStairs, "piaskowcowe schody", "Sandstone Stairs"),
    (159, SandstoneSlab, "piaskowcowa plyta", "Sandstone Slab"),
    (160, SmoothStone, "gladki kamien", "Smooth Stone"),
    (161, Obsidian, "obsydian", "Obsidian"),
    (162, Bookshelf, "biblioteczka", "Bookshelf"),
    (163, Leather, "skora", "Leather"),
    (164, LeatherHelmet, "skorzany helm", "Leather Cap"),
    (165, LeatherChestplate, "skorzana kurtka", "Leather Tunic"),
    (166, LeatherLeggings, "skorzane spodnie", "Leather Pants"),
    (167, LeatherBoots, "skorzane buty", "Leather Boots"),
    (168, IronHelmet, "zelazny helm", "Iron Helmet"),
    (169, IronChestplate, "zelazna klata", "Iron Chestplate"),
    (170, IronLeggings, "zelazne spodnie", "Iron Leggings"),
    (171, IronBoots, "zelazne buty", "Iron Boots"),
    (172, GoldHelmet, "zloty helm", "Golden Helmet"),
    (173, GoldChestplate, "zlota klata", "Golden Chestplate"),
    (174, GoldLeggings, "zlote spodnie", "Golden Leggings"),
    (175, GoldBoots, "zlote buty", "Golden Boots"),
    (176, DiamondHelmet, "diamentowy helm", "Diamond Helmet"),
    (177, DiamondChestplate, "diamentowy napiersnik", "Diamond Chestplate"),
    (178, DiamondLeggings, "diamentowe spodnie", "Diamond Leggings"),
    (179, DiamondBoots, "diamentowe buty", "Diamond Boots"),
    (180, Bow, "luk", "Bow"),
    (181, Arrow, "strzala", "Arrow"),
    (182, Shield, "tarcza", "Shield"),
    (183, StringItem, "nic", "String"),
    (184, SpiderEye, "oko pajaka", "Spider Eye"),
    (185, SpiderSpawnEgg, "jajo przyzywajace pajaka", "Spider Spawn Egg"),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArmorSlot {
    Helmet = 0,
    Chestplate = 1,
    Leggings = 2,
    Boots = 3,
}

impl ArmorSlot {
    #[inline(always)]
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => ArmorSlot::Helmet,
            1 => ArmorSlot::Chestplate,
            2 => ArmorSlot::Leggings,
            _ => ArmorSlot::Boots,
        }
    }
}

impl BlockId {
    pub const WOOL_BLOCKS: [BlockId; 16] = [
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

    #[inline(always)]
    pub fn wool_block_from_color(color_idx: u8) -> BlockId {
        Self::WOOL_BLOCKS[(color_idx as usize) % Self::WOOL_BLOCKS.len()]
    }

    #[inline(always)]
    pub fn wool_color_index(self) -> Option<u8> {
        let id = self.to_u8();
        if (110..=125).contains(&id) {
            Some(id - 110)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_wool(self) -> bool {
        self.wool_color_index().is_some()
    }

    /// Localized display name based on current language setting.
    pub fn display_name(self) -> &'static str {
        match crate::core::i18n::current_language() {
            crate::core::i18n::Language::English => self.name_en(),
            crate::core::i18n::Language::Polish => self.name(),
        }
    }
}
