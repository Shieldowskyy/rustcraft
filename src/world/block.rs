// Block type definitions and their solid colors.

use crate::core::math::IVec3;
use crate::graphics::atlas::TextureId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Leaves,
    Water,
    Snow,
    Cactus,
    CoalOre,
    IronOre,
    Bedrock,
    Lava,
    SnowLeaves,
    Planks,
    Door,
    Glass,
    Ice,
    Tnt,
    CraftingTable,
    Stick,
    WoodenPickaxe,
    StonePickaxe,
    IronPickaxe,
    WoodenAxe,
    StoneAxe,
    IronAxe,
    WoodenShovel,
    StoneShovel,
    IronShovel,
    WoodenSword,
    StoneSword,
    IronSword,
    /// Pink cherry-blossom leaves (Cherry Blossom biome)
    CherryLeaves,
    /// Mud block for swamp biome
    Mud,
    /// Autumn/fall leaves in orange-red tones
    AutumnLeaves,
    /// Torch block / item emitting light
    Torch,
    /// Raw Porkchop dropped by pigs
    RawPorkchop,
    /// Cooked Porkchop (restores 8 hunger)
    CookedPorkchop,
    /// Raw Beef dropped by cows
    RawBeef,
    /// Cooked Beef / Steak (restores 8 hunger)
    CookedBeef,
    /// Apple dropped from leaves (restores 4 hunger)
    Apple,
    /// Open Door
    DoorOpen,
    /// Storage chest block
    Chest,
    /// Smelting furnace block
    Furnace,
    /// Burning/lit furnace block
    FurnaceLit,
    /// Smelted iron ingot item
    IronIngot,
    /// Gold ingot item
    GoldIngot,
    /// Iron ounce / nugget (smelted from 1 iron ore, 8 craft 1 iron ingot)
    IronOunce,
    /// Solid block of iron (crafted from 9 iron ingots)
    IronBlock,
    /// Gold ounce / nugget (smelted from 1 gold ore, 8 craft 1 gold ingot)
    GoldOunce,
    /// Solid block of gold (crafted from 9 gold ingots)
    GoldBlock,
    /// Gold ore block
    GoldOre,
    /// Solid block of coal (crafted from 9 coal ore)
    CoalBlock,
    /// Raw Chicken dropped by chickens
    RawChicken,
    /// Cooked Chicken (restores 6 hunger)
    CookedChicken,
    /// Feather dropped by chickens
    Feather,
    /// Pig spawn egg
    PigSpawnEgg,
    /// Cow spawn egg
    CowSpawnEgg,
    /// Chicken spawn egg
    ChickenSpawnEgg,
    /// Diamond ore found deep underground
    DiamondOre,
    /// Diamond gem item
    Diamond,
    /// Solid block of diamond
    DiamondBlock,
    /// Diamond pickaxe
    DiamondPickaxe,
    /// Diamond axe
    DiamondAxe,
    /// Diamond shovel
    DiamondShovel,
    /// Diamond sword
    DiamondSword,
    /// Gold pickaxe
    GoldPickaxe,
    /// Gold axe
    GoldAxe,
    /// Gold shovel
    GoldShovel,
    /// Gold sword
    GoldSword,
    /// Bed block for sleeping and setting spawn point
    Bed,
    /// Wooden half-block slab
    WoodenSlab,
    /// Stone half-block slab
    StoneSlab,
    /// Wooden stairs
    WoodenStairs,
    /// Stone stairs
    StoneStairs,
    /// Ladder for climbing vertical walls
    Ladder,
    /// Bone dropped by skeletons
    Bone,
    /// Gunpowder dropped by creepers
    Gunpowder,
    /// Wooden Hoe
    WoodenHoe,
    /// Stone Hoe
    StoneHoe,
    /// Iron Hoe
    IronHoe,
    /// Diamond Hoe
    DiamondHoe,
    /// Gold Hoe
    GoldHoe,
    /// Tilled farmland block
    Farmland,
    /// Hydrated moist farmland block
    FarmlandWet,
    /// Wheat seeds item
    WheatSeeds,
    /// Growing/mature wheat crop
    WheatCrop,
    /// Harvested wheat item
    Wheat,
    /// Baked bread item
    Bread,
    /// Carrot food and plantable item
    Carrot,
    /// Growing/mature carrot crop
    CarrotCrop,
    /// Potato item
    Potato,
    /// Growing/mature potato crop
    PotatoCrop,
    /// Baked potato food item
    BakedPotato,
    /// Tree sapling
    Sapling,
    /// Bone meal fertilizing item
    BoneMeal,
    /// RustStone ore found in caves underground
    RustStoneOre,
    /// RustStone dust / unpowered wire
    RustStone,
    /// Active / lit RustStone wire carrying power
    RustStoneWireLit,
    /// Solid block of RustStone (craftable from 9 RustStone, permanent power source)
    RustStoneBlock,
    /// RustStone lamp (unlit)
    RustStoneLamp,
    /// Lit RustStone lamp emitting bright light
    RustStoneLampLit,
    /// Lever (off)
    Lever,
    /// Lever (on / active power source)
    LeverOn,
    /// Button (off / unpressed)
    Button,
    /// Pressed button (temporary active power source)
    ButtonPressed,
    /// Pressure plate (unpressed)
    PressurePlate,
    /// Pressed pressure plate (active power source when entity stands on it)
    PressurePlatePressed,
    /// White wool block
    WhiteWool,
    /// Orange wool block
    OrangeWool,
    /// Magenta wool block
    MagentaWool,
    /// Light blue wool block
    LightBlueWool,
    /// Yellow wool block
    YellowWool,
    /// Lime wool block
    LimeWool,
    /// Pink wool block
    PinkWool,
    /// Gray wool block
    GrayWool,
    /// Light gray wool block
    LightGrayWool,
    /// Cyan wool block
    CyanWool,
    /// Purple wool block
    PurpleWool,
    /// Blue wool block
    BlueWool,
    /// Brown wool block
    BrownWool,
    /// Green wool block
    GreenWool,
    /// Red wool block
    RedWool,
    /// Black wool block
    BlackWool,
    /// Sheep spawn egg item
    SheepSpawnEgg,
    /// Shears tool for shearing sheep and fast leaf/wool harvesting
    Shears,
    /// Underwater kelp plant / item
    Kelp,
    /// Underwater seagrass vegetation
    Seagrass,
    /// Raw Cod dropped by cod fish
    RawCod,
    /// Cooked Cod food item
    CookedCod,
    /// Raw Salmon dropped by salmon fish
    RawSalmon,
    /// Cooked Salmon food item
    CookedSalmon,
    /// Pufferfish item (poisonous if eaten!)
    Pufferfish,
    /// Tropical Fish food item
    TropicalFish,
    /// Dried Kelp quick snack
    DriedKelp,
    /// Dolphin spawn egg
    DolphinSpawnEgg,
    /// Fish spawn egg
    FishSpawnEgg,
    /// Fishing rod tool
    FishingRod,
    /// Emerald gemstone item (currency used for villager trading)
    Emerald,
    /// Rare emerald ore block found in mountains and underground
    EmeraldOre,
    /// Solid block of emerald crafted from 9 emeralds
    EmeraldBlock,
    /// Villager spawn egg item
    VillagerSpawnEgg,
    /// Iron Golem spawn egg item
    IronGolemSpawnEgg,
    /// Clay block found in rivers and shallow waters
    Clay,
    /// Clay ball item dropped by mining clay
    ClayBall,
    /// Brick item smelted from clay ball in a furnace
    Brick,
    /// Bricks block crafted from 4 bricks (2x2)
    Bricks,
    /// Brick stairs block
    BrickStairs,
    /// Brick slab block
    BrickSlab,
    /// Gravel block subject to gravity/falling or underground veins
    Gravel,
    /// Flint item dropped by gravel
    Flint,
    /// Cobblestone block mined from stone
    Cobblestone,
    /// Mossy cobblestone block
    MossyCobblestone,
    /// Cobblestone stairs block
    CobblestoneStairs,
    /// Cobblestone slab block
    CobblestoneSlab,
    /// Sandstone block crafted from 4 sand (2x2)
    Sandstone,
    /// Sandstone stairs block
    SandstoneStairs,
    /// Sandstone slab block
    SandstoneSlab,
    /// Smooth stone block smelted from stone
    SmoothStone,
    /// Obsidian blast-resistant block mined with diamond pickaxe
    Obsidian,
    /// Bookshelf block crafted from wood planks and wheat
    Bookshelf,
    /// Leather material item dropped by cows
    Leather,
    /// Leather helmet
    LeatherHelmet,
    /// Leather chestplate / tunic
    LeatherChestplate,
    /// Leather leggings / pants
    LeatherLeggings,
    /// Leather boots
    LeatherBoots,
    /// Iron helmet
    IronHelmet,
    /// Iron chestplate
    IronChestplate,
    /// Iron leggings
    IronLeggings,
    /// Iron boots
    IronBoots,
    /// Gold helmet
    GoldHelmet,
    /// Gold chestplate
    GoldChestplate,
    /// Gold leggings
    GoldLeggings,
    /// Gold boots
    GoldBoots,
    /// Diamond helmet
    DiamondHelmet,
    /// Diamond chestplate
    DiamondChestplate,
    /// Diamond leggings
    DiamondLeggings,
    /// Diamond boots
    DiamondBoots,
    /// Bow ranged weapon
    Bow,
    /// Arrow ammunition projectile
    Arrow,
    /// Shield defensive equipment for blocking attacks
    Shield,
    /// Custom modded block or item (ID 183..=255)
    Custom(u8),
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
    pub fn from_u8(v: u8) -> BlockId {
        match v {
            1 => BlockId::Grass,
            2 => BlockId::Dirt,
            3 => BlockId::Stone,
            4 => BlockId::Sand,
            5 => BlockId::Wood,
            6 => BlockId::Leaves,
            7 => BlockId::Snow,
            8 => BlockId::Cactus,
            9 => BlockId::CoalOre,
            10 => BlockId::IronOre,
            11 => BlockId::Water,
            12 => BlockId::Bedrock,
            13 => BlockId::Lava,
            14 => BlockId::SnowLeaves,
            15 => BlockId::Planks,
            16 => BlockId::Door,
            17 => BlockId::Glass,
            18 => BlockId::Ice,
            19 => BlockId::Tnt,
            20 => BlockId::CraftingTable,
            21 => BlockId::Stick,
            22 => BlockId::WoodenPickaxe,
            23 => BlockId::StonePickaxe,
            24 => BlockId::IronPickaxe,
            25 => BlockId::WoodenAxe,
            26 => BlockId::StoneAxe,
            27 => BlockId::IronAxe,
            28 => BlockId::WoodenShovel,
            29 => BlockId::StoneShovel,
            30 => BlockId::IronShovel,
            31 => BlockId::WoodenSword,
            32 => BlockId::StoneSword,
            33 => BlockId::IronSword,
            34 => BlockId::CherryLeaves,
            35 => BlockId::Mud,
            36 => BlockId::AutumnLeaves,
            37 => BlockId::Torch,
            38 => BlockId::RawPorkchop,
            39 => BlockId::CookedPorkchop,
            40 => BlockId::RawBeef,
            41 => BlockId::CookedBeef,
            42 => BlockId::Apple,
            43 => BlockId::DoorOpen,
            44 => BlockId::Chest,
            45 => BlockId::Furnace,
            46 => BlockId::FurnaceLit,
            47 => BlockId::IronIngot,
            48 => BlockId::GoldIngot,
            49 => BlockId::IronOunce,
            50 => BlockId::IronBlock,
            51 => BlockId::GoldOunce,
            52 => BlockId::GoldBlock,
            53 => BlockId::GoldOre,
            54 => BlockId::CoalBlock,
            55 => BlockId::RawChicken,
            56 => BlockId::CookedChicken,
            57 => BlockId::Feather,
            58 => BlockId::PigSpawnEgg,
            59 => BlockId::CowSpawnEgg,
            60 => BlockId::ChickenSpawnEgg,
            61 => BlockId::DiamondOre,
            62 => BlockId::Diamond,
            63 => BlockId::DiamondBlock,
            64 => BlockId::DiamondPickaxe,
            65 => BlockId::DiamondAxe,
            66 => BlockId::DiamondShovel,
            67 => BlockId::DiamondSword,
            68 => BlockId::GoldPickaxe,
            69 => BlockId::GoldAxe,
            70 => BlockId::GoldShovel,
            71 => BlockId::GoldSword,
            72 => BlockId::Bed,
            73 => BlockId::WoodenSlab,
            74 => BlockId::StoneSlab,
            75 => BlockId::WoodenStairs,
            76 => BlockId::StoneStairs,
            77 => BlockId::Ladder,
            78 => BlockId::Bone,
            79 => BlockId::Gunpowder,
            80 => BlockId::WoodenHoe,
            81 => BlockId::StoneHoe,
            82 => BlockId::IronHoe,
            83 => BlockId::DiamondHoe,
            84 => BlockId::GoldHoe,
            85 => BlockId::Farmland,
            86 => BlockId::FarmlandWet,
            87 => BlockId::WheatSeeds,
            88 => BlockId::WheatCrop,
            89 => BlockId::Wheat,
            90 => BlockId::Bread,
            91 => BlockId::Carrot,
            92 => BlockId::CarrotCrop,
            93 => BlockId::Potato,
            94 => BlockId::PotatoCrop,
            95 => BlockId::BakedPotato,
            96 => BlockId::Sapling,
            97 => BlockId::BoneMeal,
            98 => BlockId::RustStoneOre,
            99 => BlockId::RustStone,
            100 => BlockId::RustStoneWireLit,
            101 => BlockId::RustStoneBlock,
            102 => BlockId::RustStoneLamp,
            103 => BlockId::RustStoneLampLit,
            104 => BlockId::Lever,
            105 => BlockId::LeverOn,
            106 => BlockId::Button,
            107 => BlockId::ButtonPressed,
            108 => BlockId::PressurePlate,
            109 => BlockId::PressurePlatePressed,
            110 => BlockId::WhiteWool,
            111 => BlockId::OrangeWool,
            112 => BlockId::MagentaWool,
            113 => BlockId::LightBlueWool,
            114 => BlockId::YellowWool,
            115 => BlockId::LimeWool,
            116 => BlockId::PinkWool,
            117 => BlockId::GrayWool,
            118 => BlockId::LightGrayWool,
            119 => BlockId::CyanWool,
            120 => BlockId::PurpleWool,
            121 => BlockId::BlueWool,
            122 => BlockId::BrownWool,
            123 => BlockId::GreenWool,
            124 => BlockId::RedWool,
            125 => BlockId::BlackWool,
            126 => BlockId::SheepSpawnEgg,
            127 => BlockId::Shears,
            128 => BlockId::Kelp,
            129 => BlockId::Seagrass,
            130 => BlockId::RawCod,
            131 => BlockId::CookedCod,
            132 => BlockId::RawSalmon,
            133 => BlockId::CookedSalmon,
            134 => BlockId::Pufferfish,
            135 => BlockId::TropicalFish,
            136 => BlockId::DriedKelp,
            137 => BlockId::DolphinSpawnEgg,
            138 => BlockId::FishSpawnEgg,
            139 => BlockId::FishingRod,
            140 => BlockId::Emerald,
            141 => BlockId::EmeraldOre,
            142 => BlockId::EmeraldBlock,
            143 => BlockId::VillagerSpawnEgg,
            144 => BlockId::IronGolemSpawnEgg,
            145 => BlockId::Clay,
            146 => BlockId::ClayBall,
            147 => BlockId::Brick,
            148 => BlockId::Bricks,
            149 => BlockId::BrickStairs,
            150 => BlockId::BrickSlab,
            151 => BlockId::Gravel,
            152 => BlockId::Flint,
            153 => BlockId::Cobblestone,
            154 => BlockId::MossyCobblestone,
            155 => BlockId::CobblestoneStairs,
            156 => BlockId::CobblestoneSlab,
            157 => BlockId::Sandstone,
            158 => BlockId::SandstoneStairs,
            159 => BlockId::SandstoneSlab,
            160 => BlockId::SmoothStone,
            161 => BlockId::Obsidian,
            162 => BlockId::Bookshelf,
            163 => BlockId::Leather,
            164 => BlockId::LeatherHelmet,
            165 => BlockId::LeatherChestplate,
            166 => BlockId::LeatherLeggings,
            167 => BlockId::LeatherBoots,
            168 => BlockId::IronHelmet,
            169 => BlockId::IronChestplate,
            170 => BlockId::IronLeggings,
            171 => BlockId::IronBoots,
            172 => BlockId::GoldHelmet,
            173 => BlockId::GoldChestplate,
            174 => BlockId::GoldLeggings,
            175 => BlockId::GoldBoots,
            176 => BlockId::DiamondHelmet,
            177 => BlockId::DiamondChestplate,
            178 => BlockId::DiamondLeggings,
            179 => BlockId::DiamondBoots,
            180 => BlockId::Bow,
            181 => BlockId::Arrow,
            182 => BlockId::Shield,
            v if v >= 183 => BlockId::Custom(v),
            _ => BlockId::Air,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            BlockId::Air => 0,
            BlockId::Grass => 1,
            BlockId::Dirt => 2,
            BlockId::Stone => 3,
            BlockId::Sand => 4,
            BlockId::Wood => 5,
            BlockId::Leaves => 6,
            BlockId::Snow => 7,
            BlockId::Cactus => 8,
            BlockId::CoalOre => 9,
            BlockId::IronOre => 10,
            BlockId::Water => 11,
            BlockId::Bedrock => 12,
            BlockId::Lava => 13,
            BlockId::SnowLeaves => 14,
            BlockId::Planks => 15,
            BlockId::Door => 16,
            BlockId::Glass => 17,
            BlockId::Ice => 18,
            BlockId::Tnt => 19,
            BlockId::CraftingTable => 20,
            BlockId::Stick => 21,
            BlockId::WoodenPickaxe => 22,
            BlockId::StonePickaxe => 23,
            BlockId::IronPickaxe => 24,
            BlockId::WoodenAxe => 25,
            BlockId::StoneAxe => 26,
            BlockId::IronAxe => 27,
            BlockId::WoodenShovel => 28,
            BlockId::StoneShovel => 29,
            BlockId::IronShovel => 30,
            BlockId::WoodenSword => 31,
            BlockId::StoneSword => 32,
            BlockId::IronSword => 33,
            BlockId::CherryLeaves => 34,
            BlockId::Mud => 35,
            BlockId::AutumnLeaves => 36,
            BlockId::Torch => 37,
            BlockId::RawPorkchop => 38,
            BlockId::CookedPorkchop => 39,
            BlockId::RawBeef => 40,
            BlockId::CookedBeef => 41,
            BlockId::Apple => 42,
            BlockId::DoorOpen => 43,
            BlockId::Chest => 44,
            BlockId::Furnace => 45,
            BlockId::FurnaceLit => 46,
            BlockId::IronIngot => 47,
            BlockId::GoldIngot => 48,
            BlockId::IronOunce => 49,
            BlockId::IronBlock => 50,
            BlockId::GoldOunce => 51,
            BlockId::GoldBlock => 52,
            BlockId::GoldOre => 53,
            BlockId::CoalBlock => 54,
            BlockId::RawChicken => 55,
            BlockId::CookedChicken => 56,
            BlockId::Feather => 57,
            BlockId::PigSpawnEgg => 58,
            BlockId::CowSpawnEgg => 59,
            BlockId::ChickenSpawnEgg => 60,
            BlockId::DiamondOre => 61,
            BlockId::Diamond => 62,
            BlockId::DiamondBlock => 63,
            BlockId::DiamondPickaxe => 64,
            BlockId::DiamondAxe => 65,
            BlockId::DiamondShovel => 66,
            BlockId::DiamondSword => 67,
            BlockId::GoldPickaxe => 68,
            BlockId::GoldAxe => 69,
            BlockId::GoldShovel => 70,
            BlockId::GoldSword => 71,
            BlockId::Bed => 72,
            BlockId::WoodenSlab => 73,
            BlockId::StoneSlab => 74,
            BlockId::WoodenStairs => 75,
            BlockId::StoneStairs => 76,
            BlockId::Ladder => 77,
            BlockId::Bone => 78,
            BlockId::Gunpowder => 79,
            BlockId::WoodenHoe => 80,
            BlockId::StoneHoe => 81,
            BlockId::IronHoe => 82,
            BlockId::DiamondHoe => 83,
            BlockId::GoldHoe => 84,
            BlockId::Farmland => 85,
            BlockId::FarmlandWet => 86,
            BlockId::WheatSeeds => 87,
            BlockId::WheatCrop => 88,
            BlockId::Wheat => 89,
            BlockId::Bread => 90,
            BlockId::Carrot => 91,
            BlockId::CarrotCrop => 92,
            BlockId::Potato => 93,
            BlockId::PotatoCrop => 94,
            BlockId::BakedPotato => 95,
            BlockId::Sapling => 96,
            BlockId::BoneMeal => 97,
            BlockId::RustStoneOre => 98,
            BlockId::RustStone => 99,
            BlockId::RustStoneWireLit => 100,
            BlockId::RustStoneBlock => 101,
            BlockId::RustStoneLamp => 102,
            BlockId::RustStoneLampLit => 103,
            BlockId::Lever => 104,
            BlockId::LeverOn => 105,
            BlockId::Button => 106,
            BlockId::ButtonPressed => 107,
            BlockId::PressurePlate => 108,
            BlockId::PressurePlatePressed => 109,
            BlockId::WhiteWool => 110,
            BlockId::OrangeWool => 111,
            BlockId::MagentaWool => 112,
            BlockId::LightBlueWool => 113,
            BlockId::YellowWool => 114,
            BlockId::LimeWool => 115,
            BlockId::PinkWool => 116,
            BlockId::GrayWool => 117,
            BlockId::LightGrayWool => 118,
            BlockId::CyanWool => 119,
            BlockId::PurpleWool => 120,
            BlockId::BlueWool => 121,
            BlockId::BrownWool => 122,
            BlockId::GreenWool => 123,
            BlockId::RedWool => 124,
            BlockId::BlackWool => 125,
            BlockId::SheepSpawnEgg => 126,
            BlockId::Shears => 127,
            BlockId::Kelp => 128,
            BlockId::Seagrass => 129,
            BlockId::RawCod => 130,
            BlockId::CookedCod => 131,
            BlockId::RawSalmon => 132,
            BlockId::CookedSalmon => 133,
            BlockId::Pufferfish => 134,
            BlockId::TropicalFish => 135,
            BlockId::DriedKelp => 136,
            BlockId::DolphinSpawnEgg => 137,
            BlockId::FishSpawnEgg => 138,
            BlockId::FishingRod => 139,
            BlockId::Emerald => 140,
            BlockId::EmeraldOre => 141,
            BlockId::EmeraldBlock => 142,
            BlockId::VillagerSpawnEgg => 143,
            BlockId::IronGolemSpawnEgg => 144,
            BlockId::Clay => 145,
            BlockId::ClayBall => 146,
            BlockId::Brick => 147,
            BlockId::Bricks => 148,
            BlockId::BrickStairs => 149,
            BlockId::BrickSlab => 150,
            BlockId::Gravel => 151,
            BlockId::Flint => 152,
            BlockId::Cobblestone => 153,
            BlockId::MossyCobblestone => 154,
            BlockId::CobblestoneStairs => 155,
            BlockId::CobblestoneSlab => 156,
            BlockId::Sandstone => 157,
            BlockId::SandstoneStairs => 158,
            BlockId::SandstoneSlab => 159,
            BlockId::SmoothStone => 160,
            BlockId::Obsidian => 161,
            BlockId::Bookshelf => 162,
            BlockId::Leather => 163,
            BlockId::LeatherHelmet => 164,
            BlockId::LeatherChestplate => 165,
            BlockId::LeatherLeggings => 166,
            BlockId::LeatherBoots => 167,
            BlockId::IronHelmet => 168,
            BlockId::IronChestplate => 169,
            BlockId::IronLeggings => 170,
            BlockId::IronBoots => 171,
            BlockId::GoldHelmet => 172,
            BlockId::GoldChestplate => 173,
            BlockId::GoldLeggings => 174,
            BlockId::GoldBoots => 175,
            BlockId::DiamondHelmet => 176,
            BlockId::DiamondChestplate => 177,
            BlockId::DiamondLeggings => 178,
            BlockId::DiamondBoots => 179,
            BlockId::Bow => 180,
            BlockId::Arrow => 181,
            BlockId::Shield => 182,
            BlockId::Custom(id) => id,
        }
    }

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
                } else if id >= 163 && id <= 182 {
                    // Leather, Armor, Bow, Arrow, Shield are non-solid items
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
        let mult = Self::mining_speed_multiplier(tool, self);
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

    pub fn name(self) -> &'static str {
        match self {
            BlockId::Air => "powietrze",
            BlockId::Grass => "trawa",
            BlockId::Dirt => "ziemia",
            BlockId::Stone => "kamien",
            BlockId::Sand => "piasek",
            BlockId::Wood => "drewno",
            BlockId::Leaves => "liscie",
            BlockId::Water => "woda",
            BlockId::Snow => "snieg",
            BlockId::Cactus => "kaktus",
            BlockId::CoalOre => "wegiel",
            BlockId::IronOre => "zelazo",
            BlockId::Bedrock => "bedrock",
            BlockId::Lava => "lawa",
            BlockId::SnowLeaves => "liscie zimowe",
            BlockId::Planks => "deski",
            BlockId::Door => "drzwi",
            BlockId::DoorOpen => "drzwi (otwarte)",
            BlockId::Glass => "szyba",
            BlockId::Ice => "lod",
            BlockId::Tnt => "TNT",
            BlockId::CraftingTable => "stol rzemieslniczy",
            BlockId::Stick => "patyk",
            BlockId::WoodenPickaxe => "drewniany kilof",
            BlockId::StonePickaxe => "kamienny kilof",
            BlockId::IronPickaxe => "zelazny kilof",
            BlockId::WoodenAxe => "drewniany topor",
            BlockId::StoneAxe => "kamienny topor",
            BlockId::IronAxe => "zelazny topor",
            BlockId::WoodenShovel => "drewniana lopata",
            BlockId::StoneShovel => "kamienna lopata",
            BlockId::IronShovel => "zelazna lopata",
            BlockId::WoodenSword => "drewniany miecz",
            BlockId::StoneSword => "kamienny miecz",
            BlockId::IronSword => "zelazny miecz",
            BlockId::CherryLeaves => "rozowe liscie",
            BlockId::Mud => "bloto",
            BlockId::AutumnLeaves => "jesienne liscie",
            BlockId::Torch => "pochodnia",
            BlockId::RawPorkchop => "surowa wieprzowina",
            BlockId::CookedPorkchop => "pieczona wieprzowina",
            BlockId::RawBeef => "surowa wolowina",
            BlockId::CookedBeef => "stek",
            BlockId::Apple => "jablko",
            BlockId::Chest => "skrzynia",
            BlockId::Furnace => "piec",
            BlockId::FurnaceLit => "rozpalony piec",
            BlockId::IronIngot => "sztabka zelaza",
            BlockId::GoldIngot => "sztabka zlota",
            BlockId::IronOunce => "uncja zelaza",
            BlockId::IronBlock => "blok zelaza",
            BlockId::GoldOunce => "uncja zlota",
            BlockId::GoldBlock => "blok zlota",
            BlockId::GoldOre => "ruda zlota",
            BlockId::CoalBlock => "blok wegla",
            BlockId::RawChicken => "surowy kurczak",
            BlockId::CookedChicken => "pieczony kurczak",
            BlockId::Feather => "pioro",
            BlockId::PigSpawnEgg => "jajko spawnujace swinie",
            BlockId::CowSpawnEgg => "jajko spawnujace krowe",
            BlockId::ChickenSpawnEgg => "jajko spawnujace kurczaka",
            BlockId::DiamondOre => "ruda diamentu",
            BlockId::Diamond => "diament",
            BlockId::DiamondBlock => "blok diamentu",
            BlockId::DiamondPickaxe => "diamentowy kilof",
            BlockId::DiamondAxe => "diamentowy topor",
            BlockId::DiamondShovel => "diamentowa lopata",
            BlockId::DiamondSword => "diamentowy miecz",
            BlockId::GoldPickaxe => "zloty kilof",
            BlockId::GoldAxe => "zloty topor",
            BlockId::GoldShovel => "zlota lopata",
            BlockId::GoldSword => "zloty miecz",
            BlockId::Bed => "lozko",
            BlockId::WoodenSlab => "drewniany polblok",
            BlockId::StoneSlab => "kamienny polblok",
            BlockId::WoodenStairs => "drewniane schody",
            BlockId::StoneStairs => "kamienne schody",
            BlockId::Ladder => "drabina",
            BlockId::Bone => "kosc",
            BlockId::Gunpowder => "proch",
            BlockId::WoodenHoe => "drewniana motyka",
            BlockId::StoneHoe => "kamienna motyka",
            BlockId::IronHoe => "zelazna motyka",
            BlockId::DiamondHoe => "diamentowa motyka",
            BlockId::GoldHoe => "zlota motyka",
            BlockId::Farmland => "ziemia uprawna",
            BlockId::FarmlandWet => "nawodniona ziemia uprawna",
            BlockId::WheatSeeds => "nasiona pszenicy",
            BlockId::WheatCrop => "uprawa pszenicy",
            BlockId::Wheat => "pszenica",
            BlockId::Bread => "chleb",
            BlockId::Carrot => "marchewka",
            BlockId::CarrotCrop => "uprawa marchewki",
            BlockId::Potato => "ziemniak",
            BlockId::PotatoCrop => "uprawa ziemniakow",
            BlockId::BakedPotato => "pieczony ziemniak",
            BlockId::Sapling => "sadzonka debu",
            BlockId::BoneMeal => "maczka kostna",
            BlockId::RustStoneOre => "ruda rdzawnika",
            BlockId::RustStone => "rdzawnik",
            BlockId::RustStoneWireLit => "aktywny rdzawnik",
            BlockId::RustStoneBlock => "blok rdzawnika",
            BlockId::RustStoneLamp => "lampa rdzawnikowa",
            BlockId::RustStoneLampLit => "swiecaca lampa rdzawnikowa",
            BlockId::Lever => "dzwignia",
            BlockId::LeverOn => "wlaczona dzwignia",
            BlockId::Button => "przycisk",
            BlockId::ButtonPressed => "wcisniety przycisk",
            BlockId::PressurePlate => "plytka naciskowa",
            BlockId::PressurePlatePressed => "wcisnieta plytka naciskowa",
            BlockId::WhiteWool => "biala welna",
            BlockId::OrangeWool => "pomaranczowa welna",
            BlockId::MagentaWool => "karmazynowa welna",
            BlockId::LightBlueWool => "jasnoniebieska welna",
            BlockId::YellowWool => "zolta welna",
            BlockId::LimeWool => "jasnozielona welna",
            BlockId::PinkWool => "rozowa welna",
            BlockId::GrayWool => "szara welna",
            BlockId::LightGrayWool => "jasnoszara welna",
            BlockId::CyanWool => "cyjanowa welna",
            BlockId::PurpleWool => "fioletowa welna",
            BlockId::BlueWool => "niebieska welna",
            BlockId::BrownWool => "brazowa welna",
            BlockId::GreenWool => "zielona welna",
            BlockId::RedWool => "czerwona welna",
            BlockId::BlackWool => "czarna welna",
            BlockId::SheepSpawnEgg => "jajko owcy",
            BlockId::Shears => "nozyce",
            BlockId::Kelp => "wodorosty (kelp)",
            BlockId::Seagrass => "trawa morska",
            BlockId::RawCod => "surowy dorsz",
            BlockId::CookedCod => "pieczony dorsz",
            BlockId::RawSalmon => "surowy losos",
            BlockId::CookedSalmon => "pieczony losos",
            BlockId::Pufferfish => "rozdymka",
            BlockId::TropicalFish => "tropikalna ryba",
            BlockId::DriedKelp => "suszone wodorosty",
            BlockId::DolphinSpawnEgg => "jajko delfina",
            BlockId::FishSpawnEgg => "jajko ryby",
            BlockId::FishingRod => "wedka",
            BlockId::Emerald => "szmaragd",
            BlockId::EmeraldOre => "ruda szmaragdu",
            BlockId::EmeraldBlock => "blok szmaragdu",
            BlockId::VillagerSpawnEgg => "jajko wiesniaka",
            BlockId::IronGolemSpawnEgg => "jajko zelaznego golema",
            BlockId::Clay => "glina",
            BlockId::ClayBall => "kula gliny",
            BlockId::Brick => "cegla",
            BlockId::Bricks => "cegly",
            BlockId::BrickStairs => "ceglane schody",
            BlockId::BrickSlab => "ceglana plyta",
            BlockId::Gravel => "zwir",
            BlockId::Flint => "krzemien",
            BlockId::Cobblestone => "bruk",
            BlockId::MossyCobblestone => "zamszony bruk",
            BlockId::CobblestoneStairs => "brukowe schody",
            BlockId::CobblestoneSlab => "brukowa plyta",
            BlockId::Sandstone => "piaskowiec",
            BlockId::SandstoneStairs => "piaskowcowe schody",
            BlockId::SandstoneSlab => "piaskowcowa plyta",
            BlockId::SmoothStone => "gladki kamien",
            BlockId::Obsidian => "obsydian",
            BlockId::Bookshelf => "biblioteczka",
            BlockId::Leather => "skora",
            BlockId::LeatherHelmet => "skorzany helm",
            BlockId::LeatherChestplate => "skorzana kurtka",
            BlockId::LeatherLeggings => "skorzane spodnie",
            BlockId::LeatherBoots => "skorzane buty",
            BlockId::IronHelmet => "zelazny helm",
            BlockId::IronChestplate => "zelazna klata",
            BlockId::IronLeggings => "zelazne spodnie",
            BlockId::IronBoots => "zelazne buty",
            BlockId::GoldHelmet => "zloty helm",
            BlockId::GoldChestplate => "zlota klata",
            BlockId::GoldLeggings => "zlote spodnie",
            BlockId::GoldBoots => "zlote buty",
            BlockId::DiamondHelmet => "diamentowy helm",
            BlockId::DiamondChestplate => "diamentowy napiersnik",
            BlockId::DiamondLeggings => "diamentowe spodnie",
            BlockId::DiamondBoots => "diamentowe buty",
            BlockId::Bow => "luk",
            BlockId::Arrow => "strzala",
            BlockId::Shield => "tarcza",
            BlockId::Custom(id) => crate::engine::modding::get_custom_name(id),
        }
    }

    /// English display name for each block/item.
    pub fn name_en(self) -> &'static str {
        match self {
            BlockId::Air => "Air",
            BlockId::Grass => "Grass Block",
            BlockId::Dirt => "Dirt",
            BlockId::Stone => "Stone",
            BlockId::Sand => "Sand",
            BlockId::Wood => "Oak Wood",
            BlockId::Leaves => "Oak Leaves",
            BlockId::Water => "Water",
            BlockId::Snow => "Snow",
            BlockId::Cactus => "Cactus",
            BlockId::CoalOre => "Coal Ore",
            BlockId::IronOre => "Iron Ore",
            BlockId::Bedrock => "Bedrock",
            BlockId::Lava => "Lava",
            BlockId::SnowLeaves => "Snow Leaves",
            BlockId::Planks => "Oak Planks",
            BlockId::Door => "Wooden Door",
            BlockId::DoorOpen => "Wooden Door (Open)",
            BlockId::Glass => "Glass",
            BlockId::Ice => "Ice",
            BlockId::Tnt => "TNT",
            BlockId::CraftingTable => "Crafting Table",
            BlockId::Stick => "Stick",
            BlockId::WoodenPickaxe => "Wooden Pickaxe",
            BlockId::StonePickaxe => "Stone Pickaxe",
            BlockId::IronPickaxe => "Iron Pickaxe",
            BlockId::WoodenAxe => "Wooden Axe",
            BlockId::StoneAxe => "Stone Axe",
            BlockId::IronAxe => "Iron Axe",
            BlockId::WoodenShovel => "Wooden Shovel",
            BlockId::StoneShovel => "Stone Shovel",
            BlockId::IronShovel => "Iron Shovel",
            BlockId::WoodenSword => "Wooden Sword",
            BlockId::StoneSword => "Stone Sword",
            BlockId::IronSword => "Iron Sword",
            BlockId::CherryLeaves => "Cherry Leaves",
            BlockId::Mud => "Mud",
            BlockId::AutumnLeaves => "Autumn Leaves",
            BlockId::Torch => "Torch",
            BlockId::RawPorkchop => "Raw Porkchop",
            BlockId::CookedPorkchop => "Cooked Porkchop",
            BlockId::RawBeef => "Raw Beef",
            BlockId::CookedBeef => "Steak",
            BlockId::Apple => "Apple",
            BlockId::Chest => "Chest",
            BlockId::Furnace => "Furnace",
            BlockId::FurnaceLit => "Lit Furnace",
            BlockId::IronIngot => "Iron Ingot",
            BlockId::GoldIngot => "Gold Ingot",
            BlockId::IronOunce => "Iron Nugget",
            BlockId::IronBlock => "Block of Iron",
            BlockId::GoldOunce => "Gold Nugget",
            BlockId::GoldBlock => "Block of Gold",
            BlockId::GoldOre => "Gold Ore",
            BlockId::CoalBlock => "Block of Coal",
            BlockId::RawChicken => "Raw Chicken",
            BlockId::CookedChicken => "Cooked Chicken",
            BlockId::Feather => "Feather",
            BlockId::PigSpawnEgg => "Spawn Pig",
            BlockId::CowSpawnEgg => "Spawn Cow",
            BlockId::ChickenSpawnEgg => "Spawn Chicken",
            BlockId::DiamondOre => "Diamond Ore",
            BlockId::Diamond => "Diamond",
            BlockId::DiamondBlock => "Block of Diamond",
            BlockId::DiamondPickaxe => "Diamond Pickaxe",
            BlockId::DiamondAxe => "Diamond Axe",
            BlockId::DiamondShovel => "Diamond Shovel",
            BlockId::DiamondSword => "Diamond Sword",
            BlockId::GoldPickaxe => "Golden Pickaxe",
            BlockId::GoldAxe => "Golden Axe",
            BlockId::GoldShovel => "Golden Shovel",
            BlockId::GoldSword => "Golden Sword",
            BlockId::Bed => "Bed",
            BlockId::WoodenSlab => "Oak Slab",
            BlockId::StoneSlab => "Stone Slab",
            BlockId::WoodenStairs => "Oak Stairs",
            BlockId::StoneStairs => "Stone Stairs",
            BlockId::Ladder => "Ladder",
            BlockId::Bone => "Bone",
            BlockId::Gunpowder => "Gunpowder",
            BlockId::WoodenHoe => "Wooden Hoe",
            BlockId::StoneHoe => "Stone Hoe",
            BlockId::IronHoe => "Iron Hoe",
            BlockId::DiamondHoe => "Diamond Hoe",
            BlockId::GoldHoe => "Golden Hoe",
            BlockId::Farmland => "Farmland",
            BlockId::FarmlandWet => "Moist Farmland",
            BlockId::WheatSeeds => "Wheat Seeds",
            BlockId::WheatCrop => "Wheat Crop",
            BlockId::Wheat => "Wheat",
            BlockId::Bread => "Bread",
            BlockId::Carrot => "Carrot",
            BlockId::CarrotCrop => "Carrot Crop",
            BlockId::Potato => "Potato",
            BlockId::PotatoCrop => "Potato Crop",
            BlockId::BakedPotato => "Baked Potato",
            BlockId::Sapling => "Oak Sapling",
            BlockId::BoneMeal => "Bone Meal",
            BlockId::RustStoneOre => "Ruststone Ore",
            BlockId::RustStone => "Ruststone",
            BlockId::RustStoneWireLit => "Active Ruststone",
            BlockId::RustStoneBlock => "Block of Ruststone",
            BlockId::RustStoneLamp => "Ruststone Lamp",
            BlockId::RustStoneLampLit => "Lit Ruststone Lamp",
            BlockId::Lever => "Lever",
            BlockId::LeverOn => "Active Lever",
            BlockId::Button => "Button",
            BlockId::ButtonPressed => "Pressed Button",
            BlockId::PressurePlate => "Pressure Plate",
            BlockId::PressurePlatePressed => "Pressed Pressure Plate",
            BlockId::WhiteWool => "White Wool",
            BlockId::OrangeWool => "Orange Wool",
            BlockId::MagentaWool => "Magenta Wool",
            BlockId::LightBlueWool => "Light Blue Wool",
            BlockId::YellowWool => "Yellow Wool",
            BlockId::LimeWool => "Lime Wool",
            BlockId::PinkWool => "Pink Wool",
            BlockId::GrayWool => "Gray Wool",
            BlockId::LightGrayWool => "Light Gray Wool",
            BlockId::CyanWool => "Cyan Wool",
            BlockId::PurpleWool => "Purple Wool",
            BlockId::BlueWool => "Blue Wool",
            BlockId::BrownWool => "Brown Wool",
            BlockId::GreenWool => "Green Wool",
            BlockId::RedWool => "Red Wool",
            BlockId::BlackWool => "Black Wool",
            BlockId::SheepSpawnEgg => "Spawn Sheep",
            BlockId::Shears => "Shears",
            BlockId::Kelp => "Kelp",
            BlockId::Seagrass => "Seagrass",
            BlockId::RawCod => "Raw Cod",
            BlockId::CookedCod => "Cooked Cod",
            BlockId::RawSalmon => "Raw Salmon",
            BlockId::CookedSalmon => "Cooked Salmon",
            BlockId::Pufferfish => "Pufferfish",
            BlockId::TropicalFish => "Tropical Fish",
            BlockId::DriedKelp => "Dried Kelp",
            BlockId::DolphinSpawnEgg => "Spawn Dolphin",
            BlockId::FishSpawnEgg => "Spawn Fish",
            BlockId::FishingRod => "Fishing Rod",
            BlockId::Emerald => "Emerald",
            BlockId::EmeraldOre => "Emerald Ore",
            BlockId::EmeraldBlock => "Emerald Block",
            BlockId::VillagerSpawnEgg => "Spawn Villager",
            BlockId::IronGolemSpawnEgg => "Spawn Iron Golem",
            BlockId::Clay => "Clay Block",
            BlockId::ClayBall => "Clay Ball",
            BlockId::Brick => "Brick",
            BlockId::Bricks => "Bricks",
            BlockId::BrickStairs => "Brick Stairs",
            BlockId::BrickSlab => "Brick Slab",
            BlockId::Gravel => "Gravel",
            BlockId::Flint => "Flint",
            BlockId::Cobblestone => "Cobblestone",
            BlockId::MossyCobblestone => "Mossy Cobblestone",
            BlockId::CobblestoneStairs => "Cobblestone Stairs",
            BlockId::CobblestoneSlab => "Cobblestone Slab",
            BlockId::Sandstone => "Sandstone",
            BlockId::SandstoneStairs => "Sandstone Stairs",
            BlockId::SandstoneSlab => "Sandstone Slab",
            BlockId::SmoothStone => "Smooth Stone",
            BlockId::Obsidian => "Obsidian",
            BlockId::Bookshelf => "Bookshelf",
            BlockId::Leather => "Leather",
            BlockId::LeatherHelmet => "Leather Cap",
            BlockId::LeatherChestplate => "Leather Tunic",
            BlockId::LeatherLeggings => "Leather Pants",
            BlockId::LeatherBoots => "Leather Boots",
            BlockId::IronHelmet => "Iron Helmet",
            BlockId::IronChestplate => "Iron Chestplate",
            BlockId::IronLeggings => "Iron Leggings",
            BlockId::IronBoots => "Iron Boots",
            BlockId::GoldHelmet => "Golden Helmet",
            BlockId::GoldChestplate => "Golden Chestplate",
            BlockId::GoldLeggings => "Golden Leggings",
            BlockId::GoldBoots => "Golden Boots",
            BlockId::DiamondHelmet => "Diamond Helmet",
            BlockId::DiamondChestplate => "Diamond Chestplate",
            BlockId::DiamondLeggings => "Diamond Leggings",
            BlockId::DiamondBoots => "Diamond Boots",
            BlockId::Bow => "Bow",
            BlockId::Arrow => "Arrow",
            BlockId::Shield => "Shield",
            BlockId::Custom(id) => crate::engine::modding::get_custom_name(id),
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
        for id in 0..=182u8 {
            let block = BlockId::from_u8(id);
            assert_eq!(block.to_u8(), id);
            // Verify is_solid and is_transparent don't panic on u128 masks
            let _ = block.is_solid();
            let _ = block.is_transparent();
        }
        let custom = BlockId::from_u8(183);
        assert_eq!(custom, BlockId::Custom(183));
        assert_eq!(custom.to_u8(), 183);

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
