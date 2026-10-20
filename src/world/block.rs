// Block type definitions and their solid colors.

use crate::core::math::IVec3;
use crate::graphics::atlas::TextureId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        }
    }

    pub fn is_placeable(self) -> bool {
        !matches!(
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
        )
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
        matches!(self, BlockId::WoodenSlab | BlockId::StoneSlab | BlockId::Bed)
    }

    #[inline(always)]
    pub fn is_stairs(self) -> bool {
        matches!(self, BlockId::WoodenStairs | BlockId::StoneStairs)
    }

    #[inline(always)]
    pub fn is_ladder(self) -> bool {
        matches!(self, BlockId::Ladder)
    }

    #[inline(always)]
    pub fn is_solid(self) -> bool {
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
        );
        let id = self.to_u8() as u32;
        ((SOLID_MASK >> id) & 1) != 0
    }

    pub fn spawn_mob_kind(self) -> Option<crate::gameplay::mob::MobKind> {
        match self {
            BlockId::PigSpawnEgg => Some(crate::gameplay::mob::MobKind::Pig),
            BlockId::CowSpawnEgg => Some(crate::gameplay::mob::MobKind::Cow),
            BlockId::ChickenSpawnEgg => Some(crate::gameplay::mob::MobKind::Chicken),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_targetable(self) -> bool {
        self.is_solid() || matches!(self, BlockId::DoorOpen | BlockId::Torch)
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
        matches!(
            self,
            BlockId::RawPorkchop
                | BlockId::CookedPorkchop
                | BlockId::RawBeef
                | BlockId::CookedBeef
                | BlockId::Apple
                | BlockId::RawChicken
                | BlockId::CookedChicken
        )
    }

    /// Returns (hunger_points, saturation_points)
    pub fn food_nutrition(self) -> Option<(f32, f32)> {
        match self {
            BlockId::RawPorkchop => Some((3.0, 1.8)),
            BlockId::CookedPorkchop => Some((8.0, 12.8)),
            BlockId::RawBeef => Some((3.0, 1.8)),
            BlockId::CookedBeef => Some((8.0, 12.8)),
            BlockId::Apple => Some((4.0, 2.4)),
            BlockId::RawChicken => Some((2.0, 1.2)),
            BlockId::CookedChicken => Some((6.0, 7.2)),
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
            BlockId::IronShovel => 4.0,
            BlockId::WoodenPickaxe => 4.0,
            BlockId::StoneShovel => 3.5,
            BlockId::WoodenShovel => 3.0,
            BlockId::GoldShovel => 3.0,
            BlockId::Stick => 2.5,
            _ => 1.5,
        }
    }

    pub fn mining_speed_multiplier(tool: Option<BlockId>, target: BlockId) -> f32 {
        let Some(t) = tool else { return 1.0; };
        match t {
            BlockId::DiamondPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs) { 10.0 } else { 1.0 }
            }
            BlockId::GoldPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs) { 8.0 } else { 1.0 }
            }
            BlockId::IronPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::GoldOre | BlockId::DiamondOre | BlockId::DiamondBlock | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs) { 7.0 } else { 1.0 }
            }
            BlockId::StonePickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::GoldOre | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice | BlockId::Furnace | BlockId::FurnaceLit | BlockId::IronBlock | BlockId::GoldBlock | BlockId::GoldOre | BlockId::CoalBlock | BlockId::StoneSlab | BlockId::StoneStairs) { 2.5 } else { 1.0 }
            }
            BlockId::DiamondAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder) { 10.0 } else { 1.0 }
            }
            BlockId::GoldAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder) { 8.0 } else { 1.0 }
            }
            BlockId::IronAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder) { 7.0 } else { 1.0 }
            }
            BlockId::StoneAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus | BlockId::Chest | BlockId::WoodenSlab | BlockId::WoodenStairs | BlockId::Bed | BlockId::Ladder) { 2.5 } else { 1.0 }
            }
            BlockId::DiamondShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud) { 10.0 } else { 1.0 }
            }
            BlockId::GoldShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud) { 8.0 } else { 1.0 }
            }
            BlockId::IronShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud) { 7.0 } else { 1.0 }
            }
            BlockId::StoneShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud) { 4.5 } else { 1.0 }
            }
            BlockId::WoodenShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow | BlockId::Mud) { 2.5 } else { 1.0 }
            }
            BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::DiamondSword | BlockId::GoldSword => {
                if matches!(target, BlockId::Leaves | BlockId::SnowLeaves | BlockId::CherryLeaves | BlockId::AutumnLeaves) { 3.0 } else { 1.0 }
            }
            _ => 1.0,
        }
    }

    /// Break time in seconds (like Minecraft hardness). Creative breaks instantly.
    pub fn break_time(self) -> f32 {
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
            BlockId::IronBlock => 5.0,
            BlockId::GoldBlock => 3.0,
            BlockId::DiamondBlock => 5.0,
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
            | (1u128 << 77); // Ladder
        let id = self.to_u8() as u32;
        ((TRANSPARENT_MASK >> id) & 1) != 0
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
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword => [0.60, 0.42, 0.22],
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword => [0.55, 0.55, 0.55],
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword => [0.85, 0.85, 0.88],
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword => [0.95, 0.85, 0.25],
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword => [0.25, 0.90, 0.95],
            BlockId::Bone => [0.88, 0.88, 0.84],
            BlockId::Gunpowder => [0.28, 0.28, 0.28],
            BlockId::Air => [0.0, 0.0, 0.0],
        }
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
        for id in 0..=77u8 {
            let block = BlockId::from_u8(id);
            assert_eq!(block.to_u8(), id);
            // Verify is_solid and is_transparent don't panic on u128 masks
            let _ = block.is_solid();
            let _ = block.is_transparent();
        }

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
}
