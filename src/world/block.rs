// Block type definitions and their solid colors.

use crate::core::math::IVec3;
use crate::graphics::atlas::TextureId;

macro_rules! define_blocks {
    ($(($id:literal, $variant:ident, $name_pl:literal, $name_en:literal)),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum BlockId {
            $($variant,)*
            /// Custom modded block or item (ID 186..=255)
            Custom(u8),
        }

        impl BlockId {
            pub const VANILLA_ALL: [BlockId; 186] = [$(Self::$variant),*];

            #[inline(always)]
            pub const fn from_u8(v: u8) -> BlockId {
                match v {
                    $($id => BlockId::$variant,)*
                    v if v >= 186 => BlockId::Custom(v),
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
        match color_idx % 16 {
            0 => BlockId::WhiteWool,
            1 => BlockId::OrangeWool,
            2 => BlockId::MagentaWool,
            3 => BlockId::LightBlueWool,
            4 => BlockId::YellowWool,
            5 => BlockId::LimeWool,
            6 => BlockId::PinkWool,
            7 => BlockId::GrayWool,
            8 => BlockId::LightGrayWool,
            9 => BlockId::CyanWool,
            10 => BlockId::PurpleWool,
            11 => BlockId::BlueWool,
            12 => BlockId::BrownWool,
            13 => BlockId::GreenWool,
            14 => BlockId::RedWool,
            _ => BlockId::BlackWool,
        }
    }

    #[inline(always)]
    pub fn wool_color_index(self) -> Option<u8> {
        match self {
            BlockId::WhiteWool => Some(0),
            BlockId::OrangeWool => Some(1),
            BlockId::MagentaWool => Some(2),
            BlockId::LightBlueWool => Some(3),
            BlockId::YellowWool => Some(4),
            BlockId::LimeWool => Some(5),
            BlockId::PinkWool => Some(6),
            BlockId::GrayWool => Some(7),
            BlockId::LightGrayWool => Some(8),
            BlockId::CyanWool => Some(9),
            BlockId::PurpleWool => Some(10),
            BlockId::BlueWool => Some(11),
            BlockId::BrownWool => Some(12),
            BlockId::GreenWool => Some(13),
            BlockId::RedWool => Some(14),
            BlockId::BlackWool => Some(15),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_wool(self) -> bool {
        self.wool_color_index().is_some()
    }

    /// Returns the item given when picking this block in the world (e.g. in Creative mode pick-block).
    pub fn pick_item(self) -> Option<BlockId> {
        if self == BlockId::Air || self == BlockId::Water || self == BlockId::Lava {
            return None;
        }
        let item = match self {
            BlockId::RustStoneWireLit => BlockId::RustStone,
            BlockId::RustStoneLampLit => BlockId::RustStoneLamp,
            BlockId::LeverOn => BlockId::Lever,
            BlockId::ButtonPressed => BlockId::Button,
            BlockId::PressurePlatePressed => BlockId::PressurePlate,
            BlockId::DoorOpen => BlockId::Door,
            BlockId::FurnaceLit => BlockId::Furnace,
            BlockId::Farmland | BlockId::FarmlandWet => BlockId::Dirt,
            BlockId::WheatCrop => BlockId::WheatSeeds,
            BlockId::CarrotCrop => BlockId::Carrot,
            BlockId::PotatoCrop => BlockId::Potato,
            other => other,
        };
        Some(item)
    }

    pub fn is_placeable(self) -> bool {
        match self {
            BlockId::Custom(id) => !crate::engine::modding::is_custom_item_only(id),
            _ => !matches!(
                self,
                BlockId::Air
                    | BlockId::DoorOpen
                    | BlockId::Stick
                    | BlockId::WoodenPickaxe
                    | BlockId::StonePickaxe
                    | BlockId::IronPickaxe
                    | BlockId::WoodenAxe
                    | BlockId::StoneAxe
                    | BlockId::IronAxe
                    | BlockId::WoodenShovel
                    | BlockId::StoneShovel
                    | BlockId::IronShovel
                    | BlockId::WoodenSword
                    | BlockId::StoneSword
                    | BlockId::IronSword
                    | BlockId::WoodenHoe
                    | BlockId::StoneHoe
                    | BlockId::IronHoe
                    | BlockId::DiamondHoe
                    | BlockId::GoldHoe
                    | BlockId::RawPorkchop
                    | BlockId::CookedPorkchop
                    | BlockId::RawBeef
                    | BlockId::CookedBeef
                    | BlockId::Apple
                    | BlockId::IronIngot
                    | BlockId::GoldIngot
                    | BlockId::IronOunce
                    | BlockId::GoldOunce
                    | BlockId::RawChicken
                    | BlockId::CookedChicken
                    | BlockId::Feather
                    | BlockId::PigSpawnEgg
                    | BlockId::CowSpawnEgg
                    | BlockId::ChickenSpawnEgg
                    | BlockId::Diamond
                    | BlockId::DiamondPickaxe
                    | BlockId::DiamondAxe
                    | BlockId::DiamondShovel
                    | BlockId::DiamondSword
                    | BlockId::GoldPickaxe
                    | BlockId::GoldAxe
                    | BlockId::GoldShovel
                    | BlockId::GoldSword
                    | BlockId::Bone
                    | BlockId::Gunpowder
                    | BlockId::WheatSeeds
                    | BlockId::Wheat
                    | BlockId::Bread
                    | BlockId::Carrot
                    | BlockId::Potato
                    | BlockId::BakedPotato
                    | BlockId::BoneMeal
                    | BlockId::RustStoneWireLit
                    | BlockId::RustStoneLampLit
                    | BlockId::LeverOn
                    | BlockId::ButtonPressed
                    | BlockId::PressurePlatePressed
                    | BlockId::Shears
                    | BlockId::SheepSpawnEgg
                    | BlockId::RawCod
                    | BlockId::CookedCod
                    | BlockId::RawSalmon
                    | BlockId::CookedSalmon
                    | BlockId::Pufferfish
                    | BlockId::TropicalFish
                    | BlockId::DriedKelp
                    | BlockId::DolphinSpawnEgg
                    | BlockId::FishSpawnEgg
                    | BlockId::FishingRod
                    | BlockId::Emerald
                    | BlockId::VillagerSpawnEgg
                    | BlockId::IronGolemSpawnEgg
                    | BlockId::ClayBall
                    | BlockId::Brick
                    | BlockId::Flint
                    | BlockId::Leather
                    | BlockId::LeatherHelmet
                    | BlockId::LeatherChestplate
                    | BlockId::LeatherLeggings
                    | BlockId::LeatherBoots
                    | BlockId::IronHelmet
                    | BlockId::IronChestplate
                    | BlockId::IronLeggings
                    | BlockId::IronBoots
                    | BlockId::GoldHelmet
                    | BlockId::GoldChestplate
                    | BlockId::GoldLeggings
                    | BlockId::GoldBoots
                    | BlockId::DiamondHelmet
                    | BlockId::DiamondChestplate
                    | BlockId::DiamondLeggings
                    | BlockId::DiamondBoots
                    | BlockId::Bow
                    | BlockId::Arrow
                    | BlockId::Shield
                    | BlockId::StringItem
                    | BlockId::SpiderEye
                    | BlockId::SpiderSpawnEgg
            ),
        }
    }

    #[inline(always)]
    pub fn is_leaves(self) -> bool {
        matches!(
            self,
            BlockId::Leaves | BlockId::SnowLeaves | BlockId::CherryLeaves | BlockId::AutumnLeaves
        )
    }

    #[inline(always)]
    pub fn is_wood(self) -> bool {
        matches!(self, BlockId::Wood)
    }

    #[inline(always)]
    pub fn is_half_block(self) -> bool {
        matches!(
            self,
            BlockId::WoodenSlab
                | BlockId::StoneSlab
                | BlockId::BrickSlab
                | BlockId::CobblestoneSlab
                | BlockId::SandstoneSlab
                | BlockId::Bed
        )
    }

    #[inline(always)]
    pub fn is_stairs(self) -> bool {
        matches!(
            self,
            BlockId::WoodenStairs
                | BlockId::StoneStairs
                | BlockId::BrickStairs
                | BlockId::CobblestoneStairs
                | BlockId::SandstoneStairs
        )
    }

    #[inline(always)]
    pub fn is_ladder(self) -> bool {
        matches!(self, BlockId::Ladder)
    }

    #[inline(always)]
    pub fn is_solid(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_solid(id),
            _ => {
                const SOLID_MASK: u128 = !(
                    (1u128 << 0)   // Air
                    | (1u128 << 11) // Water
                    | (1u128 << 13) // Lava
                    | (1u128 << 21) // Stick
                    | (1u128 << 22) // WoodenPickaxe
                    | (1u128 << 23) // StonePickaxe
                    | (1u128 << 24) // IronPickaxe
                    | (1u128 << 25) // WoodenAxe
                    | (1u128 << 26) // StoneAxe
                    | (1u128 << 27) // IronAxe
                    | (1u128 << 28) // WoodenShovel
                    | (1u128 << 29) // StoneShovel
                    | (1u128 << 30) // IronShovel
                    | (1u128 << 31) // WoodenSword
                    | (1u128 << 32) // StoneSword
                    | (1u128 << 33) // IronSword
                    | (1u128 << 37) // Torch
                    | (1u128 << 38) // RawPorkchop
                    | (1u128 << 39) // CookedPorkchop
                    | (1u128 << 40) // RawBeef
                    | (1u128 << 41) // CookedBeef
                    | (1u128 << 42) // Apple
                    | (1u128 << 43) // DoorOpen (passable)
                    | (1u128 << 47) // IronIngot
                    | (1u128 << 48) // GoldIngot
                    | (1u128 << 49) // IronOunce
                    | (1u128 << 51) // GoldOunce
                    | (1u128 << 55) // RawChicken
                    | (1u128 << 56) // CookedChicken
                    | (1u128 << 57) // Feather
                    | (1u128 << 58) // PigSpawnEgg
                    | (1u128 << 59) // CowSpawnEgg
                    | (1u128 << 60) // ChickenSpawnEgg
                    | (1u128 << 62) // Diamond (item)
                    | (1u128 << 64) // DiamondPickaxe
                    | (1u128 << 65) // DiamondAxe
                    | (1u128 << 66) // DiamondShovel
                    | (1u128 << 67) // DiamondSword
                    | (1u128 << 68) // GoldPickaxe
                    | (1u128 << 69) // GoldAxe
                    | (1u128 << 70) // GoldShovel
                    | (1u128 << 71) // GoldSword
                    | (1u128 << 77) // Ladder (passable/climbable)
                    | (1u128 << 80) // WoodenHoe
                    | (1u128 << 81) // StoneHoe
                    | (1u128 << 82) // IronHoe
                    | (1u128 << 83) // DiamondHoe
                    | (1u128 << 84) // GoldHoe
                    | (1u128 << 87) // WheatSeeds
                    | (1u128 << 88) // WheatCrop (passable)
                    | (1u128 << 89) // Wheat
                    | (1u128 << 90) // Bread
                    | (1u128 << 91) // Carrot
                    | (1u128 << 92) // CarrotCrop (passable)
                    | (1u128 << 93) // Potato
                    | (1u128 << 94) // PotatoCrop (passable)
                    | (1u128 << 95) // BakedPotato
                    | (1u128 << 96) // Sapling (passable)
                    | (1u128 << 97) // BoneMeal
                    | (1u128 << 99)  // RustStone wire (passable)
                    | (1u128 << 100) // RustStoneWireLit (passable)
                    | (1u128 << 104) // Lever (passable)
                    | (1u128 << 105) // LeverOn (passable)
                    | (1u128 << 106) // Button (passable)
                    | (1u128 << 107) // ButtonPressed (passable)
                    | (1u128 << 108) // PressurePlate (passable)
                    | (1u128 << 109) // PressurePlatePressed (passable)
                    | (1u128 << 126) // SheepSpawnEgg (item entity)
                    | (1u128 << 127) // Shears (item entity)
                );
                let id = self.to_u8() as u32;
                if id < 128 {
                    ((SOLID_MASK >> id) & 1) != 0
                } else if id < 140 {
                    // Kelp (128), Seagrass (129), and items (130..=139) are non-solid
                    false
                } else if id == 140 || id == 143 || id == 144 || id == 146 || id == 147 || id == 152 {
                    // Emerald (140), VillagerSpawnEgg (143), IronGolemSpawnEgg (144), ClayBall (146), Brick (147), and Flint (152) are non-solid items
                    false
                } else if id == 141 || id == 142 {
                    // EmeraldOre (141) and EmeraldBlock (142) are solid blocks
                    true
                } else if id >= 163 && id <= 185 {
                    // Leather, Armor, Bow, Arrow, Shield, String, SpiderEye, SpiderSpawnEgg are non-solid items
                    false
                } else {
                    true
                }
            }
        }
    }

    pub fn spawn_mob_kind(self) -> Option<crate::gameplay::mob::MobKind> {
        match self {
            BlockId::PigSpawnEgg => Some(crate::gameplay::mob::MobKind::Pig),
            BlockId::CowSpawnEgg => Some(crate::gameplay::mob::MobKind::Cow),
            BlockId::ChickenSpawnEgg => Some(crate::gameplay::mob::MobKind::Chicken),
            BlockId::SheepSpawnEgg => Some(crate::gameplay::mob::MobKind::Sheep),
            BlockId::DolphinSpawnEgg => Some(crate::gameplay::mob::MobKind::Dolphin),
            BlockId::FishSpawnEgg => Some(crate::gameplay::mob::MobKind::Cod),
            BlockId::VillagerSpawnEgg => Some(crate::gameplay::mob::MobKind::Villager),
            BlockId::IronGolemSpawnEgg => Some(crate::gameplay::mob::MobKind::IronGolem),
            BlockId::SpiderSpawnEgg => Some(crate::gameplay::mob::MobKind::Spider),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_targetable(self) -> bool {
        self.is_solid()
            || matches!(
                self,
                BlockId::DoorOpen
                    | BlockId::Torch
                    | BlockId::Ladder
                    | BlockId::WheatCrop
                    | BlockId::CarrotCrop
                    | BlockId::PotatoCrop
                    | BlockId::Sapling
                    | BlockId::RustStone
                    | BlockId::RustStoneWireLit
                    | BlockId::Lever
                    | BlockId::LeverOn
                    | BlockId::Button
                    | BlockId::ButtonPressed
                    | BlockId::PressurePlate
                    | BlockId::PressurePlatePressed
                    | BlockId::Kelp
                    | BlockId::Seagrass
            )
    }

    #[inline(always)]
    pub fn is_opaque(self) -> bool {
        self.is_solid() && !self.is_transparent()
    }

    #[inline(always)]
    pub fn is_leaf(self) -> bool {
        matches!(
            self,
            BlockId::Leaves | BlockId::SnowLeaves | BlockId::CherryLeaves | BlockId::AutumnLeaves
        )
    }

    pub fn is_food(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_food(id),
            _ => matches!(
                self,
                BlockId::RawPorkchop
                    | BlockId::CookedPorkchop
                    | BlockId::RawBeef
                    | BlockId::CookedBeef
                    | BlockId::Apple
                    | BlockId::RawChicken
                    | BlockId::CookedChicken
                    | BlockId::Bread
                    | BlockId::Carrot
                    | BlockId::Potato
                    | BlockId::BakedPotato
                    | BlockId::RawCod
                    | BlockId::CookedCod
                    | BlockId::RawSalmon
                    | BlockId::CookedSalmon
                    | BlockId::Pufferfish
                    | BlockId::TropicalFish
                    | BlockId::DriedKelp
                    | BlockId::SpiderEye
            ),
        }
    }

    /// Returns (hunger_points, saturation_points)
    pub fn food_nutrition(self) -> Option<(f32, f32)> {
        match self {
            BlockId::Custom(id) => crate::engine::modding::get_custom_nutrition(id),
            BlockId::RawPorkchop => Some((3.0, 1.8)),
            BlockId::CookedPorkchop => Some((8.0, 12.8)),
            BlockId::RawBeef => Some((3.0, 1.8)),
            BlockId::CookedBeef => Some((8.0, 12.8)),
            BlockId::Apple => Some((4.0, 2.4)),
            BlockId::RawChicken => Some((2.0, 1.2)),
            BlockId::CookedChicken => Some((6.0, 7.2)),
            BlockId::Bread => Some((5.0, 6.0)),
            BlockId::Carrot => Some((3.0, 3.6)),
            BlockId::Potato => Some((1.0, 0.6)),
            BlockId::BakedPotato => Some((5.0, 6.0)),
            BlockId::RawCod => Some((2.0, 0.4)),
            BlockId::CookedCod => Some((5.0, 6.0)),
            BlockId::RawSalmon => Some((2.0, 0.4)),
            BlockId::CookedSalmon => Some((6.0, 9.6)),
            BlockId::Pufferfish => Some((1.0, 0.2)),
            BlockId::TropicalFish => Some((1.0, 0.2)),
            BlockId::DriedKelp => Some((1.0, 0.6)),
            BlockId::SpiderEye => Some((2.0, 3.2)),
            _ => None,
        }
    }

    pub fn is_meat(self) -> bool {
        matches!(
            self,
            BlockId::RawPorkchop
                | BlockId::CookedPorkchop
                | BlockId::RawBeef
                | BlockId::CookedBeef
                | BlockId::RawChicken
                | BlockId::CookedChicken
                | BlockId::RawCod
                | BlockId::CookedCod
                | BlockId::RawSalmon
                | BlockId::CookedSalmon
        )
    }

    pub fn is_fish(self) -> bool {
        matches!(
            self,
            BlockId::RawCod
                | BlockId::CookedCod
                | BlockId::RawSalmon
                | BlockId::CookedSalmon
                | BlockId::Pufferfish
                | BlockId::TropicalFish
        )
    }

    pub fn is_hoe(self) -> bool {
        matches!(
            self,
            BlockId::WoodenHoe
                | BlockId::StoneHoe
                | BlockId::IronHoe
                | BlockId::DiamondHoe
                | BlockId::GoldHoe
        )
    }

    pub fn is_crop(self) -> bool {
        matches!(
            self,
            BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop
        )
    }

    pub fn is_tool(self) -> bool {
        matches!(
            self,
            BlockId::Stick
                | BlockId::WoodenPickaxe
                | BlockId::StonePickaxe
                | BlockId::IronPickaxe
                | BlockId::DiamondPickaxe
                | BlockId::GoldPickaxe
                | BlockId::WoodenAxe
                | BlockId::StoneAxe
                | BlockId::IronAxe
                | BlockId::DiamondAxe
                | BlockId::GoldAxe
                | BlockId::WoodenShovel
                | BlockId::StoneShovel
                | BlockId::IronShovel
                | BlockId::DiamondShovel
                | BlockId::GoldShovel
                | BlockId::WoodenSword
                | BlockId::StoneSword
                | BlockId::IronSword
                | BlockId::DiamondSword
                | BlockId::GoldSword
                | BlockId::WoodenHoe
                | BlockId::StoneHoe
                | BlockId::IronHoe
                | BlockId::DiamondHoe
                | BlockId::GoldHoe
                | BlockId::Shears
                | BlockId::FishingRod
        )
    }

    pub fn is_bedrock(self) -> bool {
        matches!(self, BlockId::Bedrock)
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            BlockId::DiamondSword => 13.0,
            BlockId::DiamondAxe => 11.0,
            BlockId::IronSword => 10.0,
            BlockId::IronAxe => 9.0,
            BlockId::StoneSword => 8.0,
            BlockId::StoneAxe => 8.0,
            BlockId::DiamondPickaxe => 8.0,
            BlockId::GoldSword => 7.0,
            BlockId::WoodenAxe => 7.0,
            BlockId::IronPickaxe => 6.0,
            BlockId::WoodenSword => 6.0,
            BlockId::GoldAxe => 6.0,
            BlockId::DiamondShovel => 6.0,
            BlockId::StonePickaxe => 5.0,
            BlockId::GoldPickaxe => 4.5,
            BlockId::DiamondHoe => 4.0,
            BlockId::IronShovel => 4.0,
            BlockId::WoodenPickaxe => 4.0,
            BlockId::StoneShovel => 3.5,
            BlockId::IronHoe => 3.0,
            BlockId::WoodenShovel => 3.0,
            BlockId::GoldShovel => 3.0,
            BlockId::Shears => 3.0,
            BlockId::StoneHoe => 2.5,
            BlockId::Stick => 2.5,
            BlockId::WoodenHoe | BlockId::GoldHoe => 2.0,
            _ => 1.5,
        }
    }

    pub fn is_sword(self) -> bool {
        matches!(
            self,
            BlockId::WoodenSword
                | BlockId::StoneSword
                | BlockId::IronSword
                | BlockId::GoldSword
                | BlockId::DiamondSword
        )
    }

    pub fn mining_speed_multiplier(tool: Option<BlockId>, target: BlockId) -> f32 {
        let Some(t) = tool else { return 1.0; };
        match t {
            BlockId::DiamondPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::EmeraldOre | BlockId::EmeraldBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs | BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Cobblestone | BlockId::MossyCobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab | BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab | BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab | BlockId::SmoothStone | BlockId::Obsidian) { 10.0 } else { 1.0 }
            }
            BlockId::GoldPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::EmeraldOre | BlockId::EmeraldBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs | BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Cobblestone | BlockId::MossyCobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab | BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab | BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab | BlockId::SmoothStone) { 8.0 } else { 1.0 }
            }
            BlockId::IronPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::EmeraldOre | BlockId::EmeraldBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs | BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Cobblestone | BlockId::MossyCobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab | BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab | BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab | BlockId::SmoothStone) { 7.0 } else { 1.0 }
            }
            BlockId::StonePickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::GoldOre | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs | BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Cobblestone | BlockId::MossyCobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab | BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab | BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab | BlockId::SmoothStone) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::GoldOre | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs | BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Cobblestone | BlockId::MossyCobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab | BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab | BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab | BlockId::SmoothStone) { 2.5 } else { 1.0 }
            }
            BlockId::DiamondAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder | BlockId::Bookshelf) { 10.0 } else { 1.0 }
            }
            BlockId::GoldAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder | BlockId::Bookshelf) { 8.0 } else { 1.0 }
            }
            BlockId::IronAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder | BlockId::Bookshelf) { 7.0 } else { 1.0 }
            }
            BlockId::StoneAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder | BlockId::Bookshelf) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder | BlockId::Bookshelf) { 2.5 } else { 1.0 }
            }
            BlockId::DiamondShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud | BlockId::Farmland | BlockId::FarmlandWet | BlockId::Clay | BlockId::Gravel) { 10.0 } else { 1.0 }
            }
            BlockId::GoldShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud | BlockId::Farmland | BlockId::FarmlandWet | BlockId::Clay | BlockId::Gravel) { 8.0 } else { 1.0 }
            }
            BlockId::IronShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud | BlockId::Farmland | BlockId::FarmlandWet | BlockId::Clay | BlockId::Gravel) { 7.0 } else { 1.0 }
            }
            BlockId::StoneShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud | BlockId::Farmland | BlockId::FarmlandWet | BlockId::Clay | BlockId::Gravel) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud | BlockId::Farmland | BlockId::FarmlandWet | BlockId::Clay | BlockId::Gravel) { 2.5 } else { 1.0 }
            }
            BlockId::WoodenHoe | BlockId::StoneHoe | BlockId::IronHoe | BlockId::DiamondHoe | BlockId::GoldHoe => {
                if matches!(target, BlockId::Leaves | BlockId::SnowLeaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop | BlockId::Sapling | BlockId::Farmland | BlockId::FarmlandWet) { 6.0 } else { 1.0 }
            }
            BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::DiamondSword | BlockId::GoldSword => {
                if matches!(target, BlockId::Leaves | BlockId::SnowLeaves | BlockId::CherryLeaves | BlockId::AutumnLeaves) { 3.0 } else if target.is_wool() { 2.0 } else { 1.0 }
            }
            BlockId::Shears => {
                if target.is_wool() || target.is_leaf() { 10.0 } else { 1.0 }
            }
            _ => 1.0,
        }
    }

    pub fn can_harvest_with_tool(self, tool: Option<BlockId>) -> bool {
        let required_tier = match self {
            BlockId::Stone | BlockId::Cobblestone | BlockId::CoalOre => 0,
            BlockId::IronOre => 1,
            BlockId::GoldOre
            | BlockId::DiamondOre
            | BlockId::EmeraldOre
            | BlockId::RustStoneOre => 2,
            _ => return true,
        };

        let Some(tool) = tool else {
            return false;
        };
        let tool_tier = match tool {
            BlockId::WoodenPickaxe | BlockId::GoldPickaxe => 0,
            BlockId::StonePickaxe => 1,
            BlockId::IronPickaxe => 2,
            BlockId::DiamondPickaxe => 3,
            _ => return false,
        };

        tool_tier >= required_tier
    }

    /// Break time in seconds (like Minecraft hardness). Creative breaks instantly.
    pub fn break_time(self) -> f32 {
        if self.is_wool() {
            return 0.8;
        }
        match self {
            BlockId::Air => 0.0,
            BlockId::Grass => 0.6,
            BlockId::Dirt => 0.5,
            BlockId::Stone => 1.5,
            BlockId::Sand => 0.5,
            BlockId::Wood => 2.0,
            BlockId::Leaves => 0.2,
            BlockId::CherryLeaves => 0.2,
            BlockId::AutumnLeaves => 0.2,
            BlockId::Snow => 0.2,
            BlockId::Cactus => 0.4,
            BlockId::CoalOre => 1.5,
            BlockId::IronOre => 1.5,
            BlockId::GoldOre => 3.0,
            BlockId::DiamondOre => 3.0,
            BlockId::EmeraldOre => 3.0,
            BlockId::IronBlock => 5.0,
            BlockId::GoldBlock => 3.0,
            BlockId::DiamondBlock => 5.0,
            BlockId::EmeraldBlock => 5.0,
            BlockId::CoalBlock => 5.0,
            BlockId::Bedrock => f32::INFINITY,
            BlockId::Lava => 0.0,
            BlockId::SnowLeaves => 0.2,
            BlockId::Planks => 1.5,
            BlockId::CraftingTable => 1.5,
            BlockId::Chest => 2.0,
            BlockId::Furnace | BlockId::FurnaceLit => 3.5,
            BlockId::Door | BlockId::DoorOpen => 1.5,
            BlockId::Glass => 0.3,
            BlockId::Ice => 0.5,
            BlockId::Tnt => 0.0,
            BlockId::Water => 0.0,
            BlockId::Mud => 0.6,
            BlockId::Bed => 0.4,
            BlockId::WoodenSlab => 1.0,
            BlockId::StoneSlab => 1.0,
            BlockId::WoodenStairs => 1.2,
            BlockId::StoneStairs => 1.2,
            BlockId::Ladder => 0.4,
            BlockId::Farmland | BlockId::FarmlandWet => 0.6,
            BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop | BlockId::Sapling => 0.05,
            BlockId::Kelp | BlockId::Seagrass => 0.05,
            BlockId::RustStoneOre => 3.0,
            BlockId::RustStoneBlock => 5.0,
            BlockId::RustStoneLamp | BlockId::RustStoneLampLit => 0.3,
            BlockId::RustStone | BlockId::RustStoneWireLit => 0.05,
            BlockId::Lever | BlockId::LeverOn => 0.5,
            BlockId::Button | BlockId::ButtonPressed => 0.5,
            BlockId::PressurePlate | BlockId::PressurePlatePressed => 0.5,
            BlockId::Clay => 0.6,
            BlockId::Gravel => 0.6,
            BlockId::Cobblestone
            | BlockId::MossyCobblestone
            | BlockId::CobblestoneStairs
            | BlockId::CobblestoneSlab => 2.0,
            BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab => 2.0,
            BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab => 0.8,
            BlockId::SmoothStone => 2.0,
            BlockId::Obsidian => 9.0,
            BlockId::Bookshelf => 1.5,
            _ => 0.0,
        }
    }

    pub fn break_time_with_tool(self, tool: Option<BlockId>) -> f32 {
        let base = self.break_time();
        if base == 0.0 || base.is_infinite() {
            return base;
        }
        let mult = if !self.can_harvest_with_tool(tool) {
            1.0
        } else {
            Self::mining_speed_multiplier(tool, self)
        };
        base / mult
    }

    #[inline(always)]
    pub fn is_transparent(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_transparent(id),
            _ => {
                const TRANSPARENT_MASK: u128 = (1u128 << 0)   // Air
                    | (1u128 << 6)  // Leaves
                    | (1u128 << 11) // Water
                    | (1u128 << 13) // Lava
                    | (1u128 << 14) // SnowLeaves
                    | (1u128 << 16) // Door (thin)
                    | (1u128 << 17) // Glass
                    | (1u128 << 34) // CherryLeaves
                    | (1u128 << 36) // AutumnLeaves
                    | (1u128 << 37) // Torch
                    | (1u128 << 43) // DoorOpen
                    | (1u128 << 72) // Bed
                    | (1u128 << 73) // WoodenSlab
                    | (1u128 << 74) // StoneSlab
                    | (1u128 << 75) // WoodenStairs
                    | (1u128 << 76) // StoneStairs
                    | (1u128 << 77) // Ladder
                    | (1u128 << 88) // WheatCrop
                    | (1u128 << 92) // CarrotCrop
                    | (1u128 << 94) // PotatoCrop
                    | (1u128 << 96) // Sapling
                    | (1u128 << 99)  // RustStone wire
                    | (1u128 << 100) // RustStoneWireLit
                    | (1u128 << 104) // Lever
                    | (1u128 << 105) // LeverOn
                    | (1u128 << 106) // Button
                    | (1u128 << 107) // ButtonPressed
                    | (1u128 << 108) // PressurePlate
                    | (1u128 << 109); // PressurePlatePressed
                let id = self.to_u8() as u32;
                if id < 128 {
                    ((TRANSPARENT_MASK >> id) & 1) != 0
                } else if id < 140 {
                    // Kelp (128), Seagrass (129), and item entities (130..=139) are transparent
                    true
                } else if matches!(
                    self,
                    BlockId::ClayBall
                        | BlockId::Brick
                        | BlockId::Flint
                        | BlockId::BrickStairs
                        | BlockId::BrickSlab
                        | BlockId::CobblestoneStairs
                        | BlockId::CobblestoneSlab
                        | BlockId::SandstoneStairs
                        | BlockId::SandstoneSlab
                ) {
                    true
                } else if id >= 163 && id <= 182 {
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Localized display name based on current language setting.
    pub fn display_name(self) -> &'static str {
        match crate::core::i18n::current_language() {
            crate::core::i18n::Language::English => self.name_en(),
            crate::core::i18n::Language::Polish => self.name(),
        }
    }

    /// Base RGB color for each face; grass has a green top and dirt sides.
    pub fn face_color(self, face: Face) -> [f32; 3] {
        match self {
            BlockId::Grass => match face {
                Face::Top => [0.36, 0.68, 0.24],
                Face::Bottom => [0.45, 0.33, 0.20],
                _ => [0.42, 0.34, 0.20],
            },
            BlockId::Dirt => [0.45, 0.33, 0.20],
            BlockId::Stone => [0.55, 0.55, 0.55],
            BlockId::Sand => [0.87, 0.80, 0.55],
            BlockId::Wood => match face {
                Face::Top | Face::Bottom => [0.55, 0.42, 0.25],
                _ => [0.42, 0.30, 0.16],
            },
            BlockId::Leaves => [0.20, 0.50, 0.16],
            BlockId::CherryLeaves => [0.92, 0.55, 0.72],
            BlockId::AutumnLeaves => [0.78, 0.35, 0.10],
            BlockId::Mud => [0.35, 0.25, 0.15],
            BlockId::Water => [0.15, 0.35, 0.75],
            BlockId::Snow => [0.93, 0.95, 0.98],
            BlockId::Cactus => match face {
                Face::Top | Face::Bottom => [0.18, 0.55, 0.15],
                _ => [0.15, 0.48, 0.12],
            },
            BlockId::CoalOre => [0.48, 0.48, 0.48],
            BlockId::IronOre => [0.60, 0.55, 0.50],
            BlockId::GoldOre => [0.58, 0.55, 0.48],
            BlockId::DiamondOre => [0.45, 0.65, 0.68],
            BlockId::IronBlock => [0.88, 0.88, 0.90],
            BlockId::GoldBlock => [0.96, 0.85, 0.22],
            BlockId::DiamondBlock => [0.35, 0.95, 0.95],
            BlockId::CoalBlock => [0.15, 0.15, 0.15],
            BlockId::Bedrock => [0.22, 0.22, 0.22],
            BlockId::Lava => [0.95, 0.42, 0.05],
            BlockId::SnowLeaves => [0.55, 0.72, 0.80],
            BlockId::Planks | BlockId::WoodenSlab | BlockId::WoodenStairs => [0.62, 0.42, 0.20],
            BlockId::StoneSlab | BlockId::StoneStairs => [0.55, 0.55, 0.55],
            BlockId::Bed => match face {
                Face::Top => [0.85, 0.20, 0.20],
                _ => [0.58, 0.35, 0.18],
            },
            BlockId::Ladder => [0.58, 0.40, 0.22],
            BlockId::Door | BlockId::DoorOpen => [0.50, 0.30, 0.12],
            BlockId::Glass => [0.55, 0.82, 0.95],
            BlockId::Ice => [0.70, 0.88, 1.0],
            BlockId::Tnt => match face {
                Face::Top | Face::Bottom => [0.75, 0.70, 0.60],
                _ => [0.85, 0.15, 0.10],
            },
            BlockId::CraftingTable => match face {
                Face::Top => [0.68, 0.48, 0.26],
                Face::Bottom => [0.58, 0.38, 0.18],
                _ => [0.54, 0.36, 0.18],
            },
            BlockId::Chest => [0.55, 0.40, 0.20],
            BlockId::Furnace => [0.52, 0.52, 0.52],
            BlockId::FurnaceLit => match face {
                Face::North | Face::South => [0.95, 0.55, 0.15],
                _ => [0.52, 0.52, 0.52],
            },
            BlockId::Stick => [0.55, 0.38, 0.18],
            BlockId::Torch => [0.98, 0.72, 0.22],
            BlockId::RawPorkchop => [0.92, 0.55, 0.60],
            BlockId::CookedPorkchop => [0.72, 0.45, 0.25],
            BlockId::RawBeef => [0.85, 0.25, 0.25],
            BlockId::CookedBeef => [0.55, 0.30, 0.18],
            BlockId::RawChicken => [0.90, 0.70, 0.65],
            BlockId::CookedChicken => [0.72, 0.45, 0.20],
            BlockId::Feather => [0.95, 0.95, 0.95],
            BlockId::PigSpawnEgg => [0.95, 0.62, 0.68],
            BlockId::CowSpawnEgg => [0.45, 0.35, 0.28],
            BlockId::ChickenSpawnEgg => [0.95, 0.95, 0.90],
            BlockId::Apple => [0.90, 0.15, 0.15],
            BlockId::IronIngot => [0.85, 0.85, 0.88],
            BlockId::GoldIngot => [0.95, 0.85, 0.25],
            BlockId::Diamond => [0.25, 0.90, 0.95],
            BlockId::IronOunce => [0.82, 0.82, 0.85],
            BlockId::GoldOunce => [0.95, 0.85, 0.25],
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => [0.60, 0.42, 0.22],
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => [0.55, 0.55, 0.55],
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword | BlockId::IronHoe => [0.85, 0.85, 0.88],
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => [0.95, 0.85, 0.25],
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => [0.25, 0.90, 0.95],
            BlockId::Farmland => match face {
                Face::Top => [0.42, 0.28, 0.16],
                _ => [0.45, 0.33, 0.20],
            },
            BlockId::FarmlandWet => match face {
                Face::Top => [0.24, 0.16, 0.08],
                _ => [0.35, 0.24, 0.14],
            },
            BlockId::WheatSeeds => [0.65, 0.60, 0.30],
            BlockId::WheatCrop => [0.80, 0.75, 0.22],
            BlockId::Wheat => [0.85, 0.75, 0.25],
            BlockId::Bread => [0.75, 0.50, 0.20],
            BlockId::Carrot => [0.95, 0.45, 0.05],
            BlockId::CarrotCrop => [0.25, 0.65, 0.20],
            BlockId::Potato => [0.75, 0.62, 0.35],
            BlockId::PotatoCrop => [0.28, 0.60, 0.22],
            BlockId::BakedPotato => [0.82, 0.65, 0.30],
            BlockId::Sapling => [0.30, 0.62, 0.22],
            BlockId::BoneMeal => [0.92, 0.92, 0.88],
            BlockId::Bone => [0.88, 0.88, 0.84],
            BlockId::Gunpowder => [0.28, 0.28, 0.28],
            BlockId::RustStoneOre => [0.65, 0.25, 0.15],
            BlockId::RustStone => [0.75, 0.22, 0.10],
            BlockId::RustStoneWireLit => [1.0, 0.38, 0.14],
            BlockId::RustStoneBlock => [0.78, 0.24, 0.12],
            BlockId::RustStoneLamp => [0.38, 0.22, 0.14],
            BlockId::RustStoneLampLit => [0.98, 0.78, 0.35],
            BlockId::Lever | BlockId::LeverOn => [0.55, 0.50, 0.45],
            BlockId::Button | BlockId::ButtonPressed => [0.58, 0.58, 0.58],
            BlockId::PressurePlate | BlockId::PressurePlatePressed => [0.60, 0.60, 0.60],
            BlockId::WhiteWool => [0.91, 0.92, 0.92],
            BlockId::OrangeWool => [0.94, 0.46, 0.07],
            BlockId::MagentaWool => [0.74, 0.27, 0.70],
            BlockId::LightBlueWool => [0.23, 0.69, 0.85],
            BlockId::YellowWool => [0.97, 0.78, 0.15],
            BlockId::LimeWool => [0.44, 0.73, 0.10],
            BlockId::PinkWool => [0.93, 0.55, 0.67],
            BlockId::GrayWool => [0.24, 0.27, 0.28],
            BlockId::LightGrayWool => [0.56, 0.56, 0.53],
            BlockId::CyanWool => [0.08, 0.54, 0.57],
            BlockId::PurpleWool => [0.47, 0.16, 0.67],
            BlockId::BlueWool => [0.21, 0.22, 0.62],
            BlockId::BrownWool => [0.45, 0.28, 0.16],
            BlockId::GreenWool => [0.33, 0.43, 0.11],
            BlockId::RedWool => [0.63, 0.15, 0.13],
            BlockId::BlackWool => [0.08, 0.08, 0.10],
            BlockId::SheepSpawnEgg => [0.88, 0.88, 0.86],
            BlockId::Shears => [0.75, 0.75, 0.78],
            BlockId::Kelp => [0.18, 0.44, 0.16],
            BlockId::Seagrass => [0.22, 0.58, 0.22],
            BlockId::RawCod => [0.72, 0.68, 0.58],
            BlockId::CookedCod => [0.82, 0.70, 0.45],
            BlockId::RawSalmon => [0.88, 0.42, 0.38],
            BlockId::CookedSalmon => [0.92, 0.55, 0.32],
            BlockId::Pufferfish => [0.88, 0.80, 0.25],
            BlockId::TropicalFish => [0.96, 0.48, 0.15],
            BlockId::DriedKelp => [0.14, 0.28, 0.12],
            BlockId::DolphinSpawnEgg => [0.55, 0.68, 0.78],
            BlockId::FishSpawnEgg => [0.78, 0.55, 0.35],
            BlockId::FishingRod => [0.55, 0.38, 0.22],
            BlockId::Emerald => [0.10, 0.82, 0.35],
            BlockId::EmeraldOre => [0.35, 0.55, 0.35],
            BlockId::EmeraldBlock => [0.08, 0.85, 0.30],
            BlockId::VillagerSpawnEgg => [0.55, 0.38, 0.24],
            BlockId::IronGolemSpawnEgg => [0.84, 0.84, 0.82],
            BlockId::Clay => [0.62, 0.65, 0.70],
            BlockId::ClayBall => [0.68, 0.71, 0.76],
            BlockId::Brick => [0.70, 0.35, 0.25],
            BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab => [0.65, 0.30, 0.22],
            BlockId::Gravel => [0.52, 0.50, 0.50],
            BlockId::Flint => [0.25, 0.25, 0.27],
            BlockId::Cobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab => [0.48, 0.48, 0.48],
            BlockId::MossyCobblestone => [0.42, 0.50, 0.40],
            BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab => match face {
                Face::Top => [0.86, 0.82, 0.65],
                Face::Bottom => [0.82, 0.77, 0.60],
                _ => [0.84, 0.80, 0.62],
            },
            BlockId::SmoothStone => [0.62, 0.62, 0.62],
            BlockId::Obsidian => [0.12, 0.08, 0.18],
            BlockId::Bookshelf => match face {
                Face::Top | Face::Bottom => [0.55, 0.42, 0.25],
                _ => [0.45, 0.35, 0.25],
            },
            BlockId::Leather => [0.65, 0.42, 0.22],
            BlockId::LeatherHelmet | BlockId::LeatherChestplate | BlockId::LeatherLeggings | BlockId::LeatherBoots => [0.62, 0.38, 0.20],
            BlockId::IronHelmet | BlockId::IronChestplate | BlockId::IronLeggings | BlockId::IronBoots => [0.85, 0.85, 0.88],
            BlockId::GoldHelmet | BlockId::GoldChestplate | BlockId::GoldLeggings | BlockId::GoldBoots => [0.95, 0.85, 0.25],
            BlockId::DiamondHelmet | BlockId::DiamondChestplate | BlockId::DiamondLeggings | BlockId::DiamondBoots => [0.25, 0.90, 0.95],
            BlockId::Bow => [0.55, 0.38, 0.18],
            BlockId::Arrow => [0.75, 0.75, 0.72],
            BlockId::Shield => [0.60, 0.45, 0.25],
            BlockId::StringItem => [0.92, 0.92, 0.95],
            BlockId::SpiderEye => [0.65, 0.12, 0.15],
            BlockId::SpiderSpawnEgg => [0.18, 0.14, 0.14],
            BlockId::Custom(id) => crate::engine::modding::get_custom_face_color(id, face),
            BlockId::Air => [0.0, 0.0, 0.0],
        }
    }

    #[inline(always)]
    pub fn armor_slot(self) -> Option<ArmorSlot> {
        match self {
            BlockId::LeatherHelmet | BlockId::IronHelmet | BlockId::GoldHelmet | BlockId::DiamondHelmet => Some(ArmorSlot::Helmet),
            BlockId::LeatherChestplate | BlockId::IronChestplate | BlockId::GoldChestplate | BlockId::DiamondChestplate => Some(ArmorSlot::Chestplate),
            BlockId::LeatherLeggings | BlockId::IronLeggings | BlockId::GoldLeggings | BlockId::DiamondLeggings => Some(ArmorSlot::Leggings),
            BlockId::LeatherBoots | BlockId::IronBoots | BlockId::GoldBoots | BlockId::DiamondBoots => Some(ArmorSlot::Boots),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn armor_defense(self) -> f32 {
        match self {
            BlockId::LeatherHelmet => 1.0,
            BlockId::LeatherChestplate => 3.0,
            BlockId::LeatherLeggings => 2.0,
            BlockId::LeatherBoots => 1.0,
            BlockId::GoldHelmet => 2.0,
            BlockId::GoldChestplate => 5.0,
            BlockId::GoldLeggings => 3.0,
            BlockId::GoldBoots => 1.0,
            BlockId::IronHelmet => 2.0,
            BlockId::IronChestplate => 6.0,
            BlockId::IronLeggings => 5.0,
            BlockId::IronBoots => 2.0,
            BlockId::DiamondHelmet => 3.0,
            BlockId::DiamondChestplate => 8.0,
            BlockId::DiamondLeggings => 6.0,
            BlockId::DiamondBoots => 3.0,
            _ => 0.0,
        }
    }

    #[inline(always)]
    pub fn is_armor(self) -> bool {
        self.armor_slot().is_some()
    }

    #[inline(always)]
    pub fn is_shield(self) -> bool {
        matches!(self, BlockId::Shield)
    }

    #[inline(always)]
    pub fn is_bow(self) -> bool {
        matches!(self, BlockId::Bow)
    }

    /// Returns the maximum durability of tools, weapons, armor, bows, and shields.
    /// Returns 0 for non-damageable items/blocks.
    pub const fn max_durability(self) -> u16 {
        match self {
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => 59,
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => 131,
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword | BlockId::IronHoe => 250,
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => 32,
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => 1561,
            BlockId::Shears => 238,
            BlockId::FishingRod => 64,
            BlockId::Bow => 384,
            BlockId::Shield => 336,
            BlockId::LeatherHelmet => 55,
            BlockId::LeatherChestplate => 80,
            BlockId::LeatherLeggings => 75,
            BlockId::LeatherBoots => 65,
            BlockId::IronHelmet => 165,
            BlockId::IronChestplate => 240,
            BlockId::IronLeggings => 225,
            BlockId::IronBoots => 195,
            BlockId::GoldHelmet => 77,
            BlockId::GoldChestplate => 112,
            BlockId::GoldLeggings => 105,
            BlockId::GoldBoots => 91,
            BlockId::DiamondHelmet => 363,
            BlockId::DiamondChestplate => 528,
            BlockId::DiamondLeggings => 495,
            BlockId::DiamondBoots => 429,
            _ => 0,
        }
    }

    #[inline(always)]
    pub const fn is_damageable(self) -> bool {
        self.max_durability() > 0
    }

    #[inline(always)]
    pub fn is_arrow(self) -> bool {
        matches!(self, BlockId::Arrow)
    }

    /// Which procedural texture to use for a given face of this block.
    pub fn texture_id(self, face: Face) -> TextureId {
        match self {
            BlockId::Grass => match face {
                Face::Top => TextureId::GrassTop,
                Face::Bottom => TextureId::Dirt,
                _ => TextureId::GrassSide,
            },
            BlockId::Dirt => TextureId::Dirt,
            BlockId::Stone | BlockId::StoneSlab | BlockId::StoneStairs => TextureId::Stone,
            BlockId::Sand => TextureId::Sand,
            BlockId::Wood => match face {
                Face::Top | Face::Bottom => TextureId::WoodTop,
                _ => TextureId::WoodSide,
            },
            BlockId::Leaves => TextureId::Leaves,
            BlockId::CherryLeaves => TextureId::CherryLeaves,
            BlockId::AutumnLeaves => TextureId::AutumnLeaves,
            BlockId::Mud => TextureId::Mud,
            BlockId::Water => TextureId::Water,
            BlockId::Snow => TextureId::Snow,
            BlockId::Cactus => match face {
                Face::Top | Face::Bottom => TextureId::CactusTop,
                _ => TextureId::CactusSide,
            },
            BlockId::CoalOre => TextureId::CoalOre,
            BlockId::IronOre => TextureId::IronOre,
            BlockId::GoldOre => TextureId::GoldOre,
            BlockId::DiamondOre => TextureId::DiamondOre,
            BlockId::IronBlock => TextureId::IronBlock,
            BlockId::GoldBlock => TextureId::GoldBlock,
            BlockId::DiamondBlock => TextureId::DiamondBlock,
            BlockId::CoalBlock => TextureId::CoalBlock,
            BlockId::RawChicken => TextureId::RawChicken,
            BlockId::CookedChicken => TextureId::CookedChicken,
            BlockId::Feather => TextureId::Feather,
            BlockId::PigSpawnEgg => TextureId::PigSpawnEgg,
            BlockId::CowSpawnEgg => TextureId::CowSpawnEgg,
            BlockId::ChickenSpawnEgg => TextureId::ChickenSpawnEgg,
            BlockId::Bedrock => TextureId::Bedrock,
            BlockId::Lava => TextureId::Lava,
            BlockId::SnowLeaves => TextureId::SnowLeaves,
            BlockId::Planks | BlockId::WoodenSlab | BlockId::WoodenStairs => TextureId::Planks,
            BlockId::Door | BlockId::DoorOpen => TextureId::Door,
            BlockId::Glass => TextureId::Glass,
            BlockId::Ice => TextureId::Ice,
            BlockId::Tnt => match face {
                Face::Top | Face::Bottom => TextureId::TntTop,
                _ => TextureId::TntSide,
            },
            BlockId::CraftingTable => match face {
                Face::Top => TextureId::CraftingTableTop,
                Face::Bottom => TextureId::Planks,
                Face::North | Face::South => TextureId::CraftingTableFront,
                Face::East | Face::West => TextureId::CraftingTableSide,
            },
            BlockId::Chest => match face {
                Face::Top | Face::Bottom => TextureId::ChestTop,
                Face::North | Face::South => TextureId::ChestFront,
                Face::East | Face::West => TextureId::ChestSide,
            },
            BlockId::Furnace => match face {
                Face::Top | Face::Bottom => TextureId::FurnaceTop,
                Face::North | Face::South => TextureId::FurnaceFront,
                Face::East | Face::West => TextureId::FurnaceSide,
            },
            BlockId::FurnaceLit => match face {
                Face::Top | Face::Bottom => TextureId::FurnaceTop,
                Face::North | Face::South => TextureId::FurnaceFrontLit,
                Face::East | Face::West => TextureId::FurnaceSide,
            },
            BlockId::Torch => TextureId::Torch,
            BlockId::RawPorkchop => TextureId::RawPorkchop,
            BlockId::CookedPorkchop => TextureId::CookedPorkchop,
            BlockId::RawBeef => TextureId::RawBeef,
            BlockId::CookedBeef => TextureId::CookedBeef,
            BlockId::Apple => TextureId::Apple,
            BlockId::IronIngot => TextureId::IronIngot,
            BlockId::GoldIngot => TextureId::GoldIngot,
            BlockId::IronOunce => TextureId::IronOunce,
            BlockId::GoldOunce => TextureId::GoldOunce,
            BlockId::Diamond => TextureId::Diamond,
            BlockId::Bed => match face {
                Face::Top => TextureId::BedTop,
                _ => TextureId::BedSide,
            },
            BlockId::Ladder => TextureId::Ladder,
            BlockId::Bone => TextureId::Bone,
            BlockId::Gunpowder => TextureId::Gunpowder,
            BlockId::Farmland => match face {
                Face::Top => TextureId::Farmland,
                Face::Bottom => TextureId::Dirt,
                _ => TextureId::FarmlandSide,
            },
            BlockId::FarmlandWet => match face {
                Face::Top => TextureId::FarmlandWet,
                Face::Bottom => TextureId::Dirt,
                _ => TextureId::FarmlandSide,
            },
            BlockId::WheatSeeds => TextureId::WheatSeeds,
            BlockId::WheatCrop => TextureId::WheatCrop,
            BlockId::Wheat => TextureId::Wheat,
            BlockId::Bread => TextureId::Bread,
            BlockId::Carrot => TextureId::Carrot,
            BlockId::CarrotCrop => TextureId::CarrotCrop,
            BlockId::Potato => TextureId::Potato,
            BlockId::PotatoCrop => TextureId::PotatoCrop,
            BlockId::BakedPotato => TextureId::BakedPotato,
            BlockId::Sapling => TextureId::Sapling,
            BlockId::BoneMeal => TextureId::BoneMeal,
            BlockId::RustStoneOre => TextureId::RustStoneOre,
            BlockId::RustStone => TextureId::RustStone,
            BlockId::RustStoneWireLit => TextureId::RustStoneWireLit,
            BlockId::RustStoneBlock => TextureId::RustStoneBlock,
            BlockId::RustStoneLamp => TextureId::RustStoneLamp,
            BlockId::RustStoneLampLit => TextureId::RustStoneLampLit,
            BlockId::Lever | BlockId::LeverOn => TextureId::Lever,
            BlockId::Button | BlockId::ButtonPressed => TextureId::Button,
            BlockId::PressurePlate | BlockId::PressurePlatePressed => TextureId::PressurePlate,
            BlockId::WhiteWool => TextureId::WhiteWool,
            BlockId::OrangeWool => TextureId::OrangeWool,
            BlockId::MagentaWool => TextureId::MagentaWool,
            BlockId::LightBlueWool => TextureId::LightBlueWool,
            BlockId::YellowWool => TextureId::YellowWool,
            BlockId::LimeWool => TextureId::LimeWool,
            BlockId::PinkWool => TextureId::PinkWool,
            BlockId::GrayWool => TextureId::GrayWool,
            BlockId::LightGrayWool => TextureId::LightGrayWool,
            BlockId::CyanWool => TextureId::CyanWool,
            BlockId::PurpleWool => TextureId::PurpleWool,
            BlockId::BlueWool => TextureId::BlueWool,
            BlockId::BrownWool => TextureId::BrownWool,
            BlockId::GreenWool => TextureId::GreenWool,
            BlockId::RedWool => TextureId::RedWool,
            BlockId::BlackWool => TextureId::BlackWool,
            BlockId::SheepSpawnEgg => TextureId::SheepSpawnEgg,
            BlockId::Shears => TextureId::Shears,
            BlockId::Kelp => TextureId::Kelp,
            BlockId::Seagrass => TextureId::Seagrass,
            BlockId::RawCod => TextureId::RawCod,
            BlockId::CookedCod => TextureId::CookedCod,
            BlockId::RawSalmon => TextureId::RawSalmon,
            BlockId::CookedSalmon => TextureId::CookedSalmon,
            BlockId::Pufferfish => TextureId::Pufferfish,
            BlockId::TropicalFish => TextureId::TropicalFish,
            BlockId::DriedKelp => TextureId::DriedKelp,
            BlockId::DolphinSpawnEgg => TextureId::DolphinSpawnEgg,
            BlockId::FishSpawnEgg => TextureId::FishSpawnEgg,
            BlockId::FishingRod => TextureId::FishingRod,
            BlockId::Emerald => TextureId::Emerald,
            BlockId::EmeraldOre => TextureId::EmeraldOre,
            BlockId::EmeraldBlock => TextureId::EmeraldBlock,
            BlockId::VillagerSpawnEgg => TextureId::VillagerSpawnEgg,
            BlockId::IronGolemSpawnEgg => TextureId::IronGolemSpawnEgg,
            BlockId::SpiderSpawnEgg => TextureId::SpiderSpawnEgg,
            BlockId::StringItem => TextureId::StringItem,
            BlockId::SpiderEye => TextureId::SpiderEye,
            BlockId::Clay => TextureId::Clay,
            BlockId::ClayBall => TextureId::ClayBall,
            BlockId::Brick => TextureId::Brick,
            BlockId::Bricks | BlockId::BrickStairs | BlockId::BrickSlab => TextureId::Bricks,
            BlockId::Gravel => TextureId::Gravel,
            BlockId::Flint => TextureId::Flint,
            BlockId::Cobblestone | BlockId::CobblestoneStairs | BlockId::CobblestoneSlab => TextureId::Cobblestone,
            BlockId::MossyCobblestone => TextureId::MossyCobblestone,
            BlockId::Sandstone | BlockId::SandstoneStairs | BlockId::SandstoneSlab => match face {
                Face::Top => TextureId::SandstoneTop,
                Face::Bottom => TextureId::SandstoneBottom,
                _ => TextureId::SandstoneSide,
            },
            BlockId::SmoothStone => TextureId::SmoothStone,
            BlockId::Obsidian => TextureId::Obsidian,
            BlockId::Bookshelf => match face {
                Face::Top | Face::Bottom => TextureId::BookshelfTop,
                _ => TextureId::BookshelfSide,
            },
            BlockId::Custom(id) => crate::engine::modding::get_custom_texture_id(id),
            _ => TextureId::Planks,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Face {
    Top,
    Bottom,
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::North,
        Face::South,
        Face::East,
        Face::West,
    ];

    pub fn shade(self) -> f32 {
        match self {
            Face::Top => 1.0,
            Face::North => 0.85,
            Face::South => 0.85,
            Face::East => 0.7,
            Face::West => 0.7,
            Face::Bottom => 0.5,
        }
    }

    pub fn normal(self) -> IVec3 {
        match self {
            Face::Top => IVec3::new(0, 1, 0),
            Face::Bottom => IVec3::new(0, -1, 0),
            Face::North => IVec3::new(0, 0, -1),
            Face::South => IVec3::new(0, 0, 1),
            Face::East => IVec3::new(1, 0, 0),
            Face::West => IVec3::new(-1, 0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meat_properties_and_textures() {
        let meats = [
            (BlockId::RawPorkchop, TextureId::RawPorkchop),
            (BlockId::CookedPorkchop, TextureId::CookedPorkchop),
            (BlockId::RawBeef, TextureId::RawBeef),
            (BlockId::CookedBeef, TextureId::CookedBeef),
            (BlockId::RawChicken, TextureId::RawChicken),
            (BlockId::CookedChicken, TextureId::CookedChicken),
        ];

        for (block, expected_tex) in meats {
            assert!(block.is_meat(), "{:?} should be classified as meat", block);
            assert!(block.is_food(), "{:?} should be classified as food", block);
            assert!(!block.is_placeable(), "{:?} must not be placeable as a block", block);
            assert_ne!(
                block.texture_id(Face::Top),
                TextureId::Planks,
                "{:?} should not have Planks texture!",
                block
            );
            assert_eq!(
                block.texture_id(Face::Top),
                expected_tex,
                "{:?} texture must match {:?}",
                block,
                expected_tex
            );
        }

        assert!(!BlockId::Planks.is_meat());
        assert!(!BlockId::Dirt.is_meat());
        assert!(!BlockId::IronSword.is_meat());
    }

    #[test]
    fn test_new_blocks_and_tools() {
        // Roundtrip u8
        for id in 0..=185u8 {
            let block = BlockId::from_u8(id);
            assert_eq!(block.to_u8(), id);
            // Verify is_solid and is_transparent don't panic on u128 masks
            let _ = block.is_solid();
            let _ = block.is_transparent();
        }
        let custom = BlockId::from_u8(186);
        assert_eq!(custom, BlockId::Custom(186));
        assert_eq!(custom.to_u8(), 186);

        // Armor and combat item properties
        assert_eq!(BlockId::DiamondChestplate.armor_defense(), 8.0);
        assert_eq!(BlockId::DiamondChestplate.armor_slot(), Some(ArmorSlot::Chestplate));
        assert!(BlockId::DiamondChestplate.is_armor());
        assert!(!BlockId::DiamondChestplate.is_solid());
        assert!(!BlockId::DiamondChestplate.is_placeable());
        assert!(BlockId::Shield.is_shield());
        assert!(BlockId::Bow.is_bow());
        assert!(BlockId::Arrow.is_arrow());

        // New building blocks and materials properties
        assert!(BlockId::Clay.is_solid());
        assert!(BlockId::Clay.is_placeable());
        assert!(!BlockId::ClayBall.is_solid());
        assert!(!BlockId::ClayBall.is_placeable());
        assert!(BlockId::Bricks.is_solid());
        assert!(BlockId::BrickStairs.is_stairs());
        assert!(BlockId::BrickSlab.is_half_block());
        assert!(BlockId::Gravel.is_solid());
        assert!(!BlockId::Flint.is_solid());
        assert!(!BlockId::Flint.is_placeable());
        assert!(BlockId::Cobblestone.is_solid());
        assert!(BlockId::MossyCobblestone.is_solid());
        assert!(BlockId::CobblestoneStairs.is_stairs());
        assert!(BlockId::CobblestoneSlab.is_half_block());
        assert!(BlockId::Sandstone.is_solid());
        assert!(BlockId::SandstoneStairs.is_stairs());
        assert!(BlockId::SandstoneSlab.is_half_block());
        assert!(BlockId::SmoothStone.is_solid());
        assert!(BlockId::Obsidian.is_solid());
        assert_eq!(BlockId::Obsidian.break_time(), 9.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::Obsidian), 10.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::StonePickaxe), BlockId::Obsidian), 1.0);
        assert!(BlockId::Bookshelf.is_solid());
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::IronAxe), BlockId::Bookshelf), 7.0);

        // Emerald, Villager, and Iron Golem properties
        assert!(!BlockId::Emerald.is_placeable());
        assert!(!BlockId::Emerald.is_solid());
        assert!(BlockId::EmeraldOre.is_placeable());
        assert!(BlockId::EmeraldOre.is_solid());
        assert!(BlockId::EmeraldBlock.is_solid());
        assert_eq!(BlockId::VillagerSpawnEgg.spawn_mob_kind(), Some(crate::gameplay::mob::MobKind::Villager));
        assert_eq!(BlockId::IronGolemSpawnEgg.spawn_mob_kind(), Some(crate::gameplay::mob::MobKind::IronGolem));

        // Aquatic properties
        assert!(BlockId::Kelp.is_placeable());
        assert!(BlockId::Seagrass.is_placeable());
        assert!(!BlockId::RawCod.is_placeable());
        assert!(!BlockId::FishingRod.is_placeable());
        assert!(!BlockId::Kelp.is_solid());
        assert!(BlockId::Kelp.is_transparent());
        assert!(BlockId::RawCod.is_fish());
        assert!(BlockId::RawSalmon.is_meat());
        assert!(BlockId::FishingRod.is_tool());
        assert!(BlockId::DriedKelp.is_food());

        // Wool properties
        assert!(BlockId::WhiteWool.is_wool());
        assert!(BlockId::BlackWool.is_wool());
        assert!(BlockId::WhiteWool.is_solid());
        assert_eq!(BlockId::WhiteWool.break_time(), 0.8);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::Shears), BlockId::WhiteWool), 10.0);
        assert_eq!(BlockId::wool_color_index(BlockId::WhiteWool), Some(0));
        assert_eq!(BlockId::wool_color_index(BlockId::BlackWool), Some(15));
        assert_eq!(BlockId::wool_block_from_color(0), BlockId::WhiteWool);
        assert_eq!(BlockId::wool_block_from_color(15), BlockId::BlackWool);

        // Shears tool
        assert!(BlockId::Shears.is_tool());
        assert_eq!(BlockId::Shears.attack_damage(), 3.0);
        assert!(!BlockId::Shears.is_solid());

        // Diamond & Gold tools
        assert!(BlockId::DiamondPickaxe.is_tool());
        assert_eq!(BlockId::DiamondPickaxe.attack_damage(), 8.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::Stone), 10.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::GoldPickaxe), BlockId::Stone), 8.0);
        assert_eq!(BlockId::DiamondSword.attack_damage(), 13.0);

        // Architectural block properties
        assert!(BlockId::WoodenSlab.is_half_block());
        assert!(BlockId::StoneSlab.is_half_block());
        assert!(BlockId::Bed.is_half_block());
        assert!(BlockId::WoodenStairs.is_stairs());
        assert!(BlockId::StoneStairs.is_stairs());
        assert!(BlockId::Ladder.is_ladder());
        assert!(!BlockId::Ladder.is_solid());
    }

    #[test]
    fn test_ore_mining_tool_tiers() {
        assert!(!BlockId::Stone.can_harvest_with_tool(None));
        assert!(!BlockId::Cobblestone.can_harvest_with_tool(None));
        assert!(BlockId::Stone.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
        assert!(BlockId::Cobblestone.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
        assert!(
            BlockId::Stone.break_time_with_tool(None)
                > BlockId::Stone.break_time_with_tool(Some(BlockId::WoodenPickaxe)),
            "stone should take longer to mine by hand than with a pickaxe"
        );

        assert!(BlockId::CoalOre.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
        assert!(BlockId::CoalOre.can_harvest_with_tool(Some(BlockId::GoldPickaxe)));
        assert!(!BlockId::CoalOre.can_harvest_with_tool(None));

        assert!(!BlockId::IronOre.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
        assert!(!BlockId::IronOre.can_harvest_with_tool(Some(BlockId::GoldPickaxe)));
        assert!(BlockId::IronOre.can_harvest_with_tool(Some(BlockId::StonePickaxe)));

        for ore in [
            BlockId::GoldOre,
            BlockId::DiamondOre,
            BlockId::EmeraldOre,
            BlockId::RustStoneOre,
        ] {
            assert!(
                !ore.can_harvest_with_tool(Some(BlockId::StonePickaxe)),
                "{ore:?} should require an iron pickaxe"
            );
            assert!(
                ore.can_harvest_with_tool(Some(BlockId::IronPickaxe)),
                "{ore:?} should be minable with an iron pickaxe"
            );
            assert!(
                ore.can_harvest_with_tool(Some(BlockId::DiamondPickaxe)),
                "{ore:?} should be minable with a diamond pickaxe"
            );
        }

        assert!(
            BlockId::Dirt.can_harvest_with_tool(None),
            "non-ores should keep existing mining behavior"
        );

        assert!(
            BlockId::DiamondOre.break_time_with_tool(Some(BlockId::StonePickaxe))
                > BlockId::DiamondOre.break_time_with_tool(Some(BlockId::IronPickaxe)),
            "under-tier tools should mine ore more slowly"
        );
    }

    #[test]
    fn test_ruststone_properties() {
        // Solid vs non-solid
        assert!(BlockId::RustStoneOre.is_solid());
        assert!(BlockId::RustStoneBlock.is_solid());
        assert!(BlockId::RustStoneLamp.is_solid());
        assert!(BlockId::RustStoneLampLit.is_solid());

        assert!(!BlockId::RustStone.is_solid(), "RustStone wire must be passable");
        assert!(!BlockId::RustStoneWireLit.is_solid(), "Lit wire must be passable");
        assert!(!BlockId::Lever.is_solid(), "Lever must be passable");
        assert!(!BlockId::LeverOn.is_solid(), "LeverOn must be passable");
        assert!(!BlockId::Button.is_solid(), "Button must be passable");
        assert!(!BlockId::ButtonPressed.is_solid(), "ButtonPressed must be passable");
        assert!(!BlockId::PressurePlate.is_solid(), "PressurePlate must be passable");
        assert!(!BlockId::PressurePlatePressed.is_solid(), "PressurePlatePressed must be passable");

        // Targetable (can be raycast, mined, and picked)
        assert!(BlockId::RustStone.is_targetable(), "RustStone wire must be targetable");
        assert!(BlockId::RustStoneWireLit.is_targetable(), "Lit wire must be targetable");
        assert!(BlockId::Lever.is_targetable(), "Lever must be targetable");
        assert!(BlockId::Button.is_targetable(), "Button must be targetable");
        assert!(BlockId::PressurePlate.is_targetable(), "PressurePlate must be targetable");

        // Transparent
        assert!(BlockId::RustStone.is_transparent());
        assert!(BlockId::RustStoneWireLit.is_transparent());
        assert!(BlockId::Lever.is_transparent());
        assert!(BlockId::Button.is_transparent());
        assert!(BlockId::PressurePlate.is_transparent());

        // Mining speeds with pickaxe
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::IronPickaxe), BlockId::RustStoneOre), 7.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::RustStoneBlock), 10.0);
        assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::StonePickaxe), BlockId::RustStoneLamp), 4.5);

        // Polish names check
        assert_eq!(BlockId::RustStoneOre.name(), "ruda rdzawnika");
        assert_eq!(BlockId::RustStone.name(), "rdzawnik");
        assert_eq!(BlockId::RustStoneBlock.name(), "blok rdzawnika");
        assert_eq!(BlockId::RustStoneLamp.name(), "lampa rdzawnikowa");
        assert_eq!(BlockId::Lever.name(), "dzwignia");
        assert_eq!(BlockId::Button.name(), "przycisk");
        assert_eq!(BlockId::PressurePlate.name(), "plytka naciskowa");

        // English names check
        assert_eq!(BlockId::RustStoneOre.name_en(), "Ruststone Ore");
        assert_eq!(BlockId::RustStone.name_en(), "Ruststone");
        assert_eq!(BlockId::RustStoneBlock.name_en(), "Block of Ruststone");
        assert_eq!(BlockId::RustStoneLamp.name_en(), "Ruststone Lamp");
        assert_eq!(BlockId::Grass.name_en(), "Grass Block");
        assert_eq!(BlockId::Dirt.name_en(), "Dirt");
        assert_eq!(BlockId::Stone.name_en(), "Stone");
        assert_eq!(BlockId::DiamondSword.name_en(), "Diamond Sword");

        // Localization switching check
        crate::core::i18n::set_language(crate::core::i18n::Language::English);
        assert_eq!(BlockId::Grass.display_name(), "Grass Block");
        assert_eq!(BlockId::RustStone.display_name(), "Ruststone");

        crate::core::i18n::set_language(crate::core::i18n::Language::Polish);
        assert_eq!(BlockId::Grass.display_name(), "trawa");
        assert_eq!(BlockId::RustStone.display_name(), "rdzawnik");

        // Restore default English
        crate::core::i18n::set_language(crate::core::i18n::Language::English);
    }
}
