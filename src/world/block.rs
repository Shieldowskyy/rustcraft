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
        }
    }

    pub fn is_placeable(self) -> bool {
        !matches!(
            self,
            BlockId::Air
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
        )
    }

    #[inline(always)]
    pub fn is_solid(self) -> bool {
        const SOLID_MASK: u64 = !(
            (1u64 << 0)   // Air
            | (1u64 << 11) // Water
            | (1u64 << 13) // Lava
            | (1u64 << 21) // Stick
            | (1u64 << 22) // WoodenPickaxe
            | (1u64 << 23) // StonePickaxe
            | (1u64 << 24) // IronPickaxe
            | (1u64 << 25) // WoodenAxe
            | (1u64 << 26) // StoneAxe
            | (1u64 << 27) // IronAxe
            | (1u64 << 28) // WoodenShovel
            | (1u64 << 29) // StoneShovel
            | (1u64 << 30) // IronShovel
            | (1u64 << 31) // WoodenSword
            | (1u64 << 32) // StoneSword
            | (1u64 << 33) // IronSword
            | (1u64 << 37) // Torch
            | (1u64 << 38) // RawPorkchop
            | (1u64 << 39) // CookedPorkchop
            | (1u64 << 40) // RawBeef
            | (1u64 << 41) // CookedBeef
            | (1u64 << 42) // Apple
        );
        let id = self.to_u8() as u32;
        ((SOLID_MASK >> id) & 1) != 0
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
            _ => None,
        }
    }

    pub fn is_tool(self) -> bool {
        matches!(
            self,
            BlockId::Stick
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
        )
    }

    pub fn is_bedrock(self) -> bool {
        matches!(self, BlockId::Bedrock)
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            BlockId::IronSword => 9.0,
            BlockId::StoneSword => 7.0,
            BlockId::WoodenSword => 5.0,
            BlockId::IronAxe => 6.0,
            BlockId::StoneAxe => 5.0,
            BlockId::WoodenAxe => 4.0,
            BlockId::IronPickaxe => 4.5,
            BlockId::StonePickaxe => 3.5,
            BlockId::WoodenPickaxe => 3.0,
            BlockId::IronShovel => 3.5,
            BlockId::StoneShovel => 3.0,
            BlockId::WoodenShovel => 2.5,
            BlockId::Stick => 2.0,
            _ => 1.5,
        }
    }

    pub fn mining_speed_multiplier(tool: Option<BlockId>, target: BlockId) -> f32 {
        let Some(t) = tool else { return 1.0; };
        match t {
            BlockId::WoodenPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice) { 2.5 } else { 1.0 }
            }
            BlockId::StonePickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice) { 4.5 } else { 1.0 }
            }
            BlockId::IronPickaxe => {
                if matches!(target, BlockId::Stone | BlockId::CoalOre | BlockId::IronOre | BlockId::Ice) { 7.0 } else { 1.0 }
            }
            BlockId::WoodenAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus) { 2.5 } else { 1.0 }
            }
            BlockId::StoneAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus) { 4.5 } else { 1.0 }
            }
            BlockId::IronAxe => {
                if matches!(target, BlockId::Wood | BlockId::Planks | BlockId::CraftingTable | BlockId::Door | BlockId::Cactus) { 7.0 } else { 1.0 }
            }
            BlockId::WoodenShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow) { 2.5 } else { 1.0 }
            }
            BlockId::StoneShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow) { 4.5 } else { 1.0 }
            }
            BlockId::IronShovel => {
                if matches!(target, BlockId::Dirt | BlockId::Grass | BlockId::Sand | BlockId::Snow) { 7.0 } else { 1.0 }
            }
            BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword => {
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
            BlockId::Bedrock => f32::INFINITY,
            BlockId::Lava => 0.0,
            BlockId::SnowLeaves => 0.2,
            BlockId::Planks => 1.5,
            BlockId::CraftingTable => 1.5,
            BlockId::Door => 1.5,
            BlockId::Glass => 0.3,
            BlockId::Ice => 0.5,
            BlockId::Tnt => 0.0,
            BlockId::Water => 0.0,
            BlockId::Mud => 0.6,
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
        const TRANSPARENT_MASK: u64 = (1u64 << 0)   // Air
            | (1u64 << 6)  // Leaves
            | (1u64 << 11) // Water
            | (1u64 << 13) // Lava
            | (1u64 << 14) // SnowLeaves
            | (1u64 << 17) // Glass
            | (1u64 << 34) // CherryLeaves
            | (1u64 << 36) // AutumnLeaves
            | (1u64 << 37); // Torch
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
            BlockId::Bedrock => [0.22, 0.22, 0.22],
            BlockId::Lava => [0.95, 0.42, 0.05],
            BlockId::SnowLeaves => [0.55, 0.72, 0.80],
            BlockId::Planks => [0.62, 0.42, 0.20],
            BlockId::Door => [0.50, 0.30, 0.12],
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
            BlockId::Stick => [0.55, 0.38, 0.18],
            BlockId::Torch => [0.98, 0.72, 0.22],
            BlockId::RawPorkchop => [0.92, 0.55, 0.60],
            BlockId::CookedPorkchop => [0.72, 0.45, 0.25],
            BlockId::RawBeef => [0.85, 0.25, 0.25],
            BlockId::CookedBeef => [0.55, 0.30, 0.18],
            BlockId::Apple => [0.90, 0.15, 0.15],
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword => [0.60, 0.42, 0.22],
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword => [0.55, 0.55, 0.55],
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword => [0.85, 0.85, 0.88],
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
            BlockId::Stone => TextureId::Stone,
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
            BlockId::Bedrock => TextureId::Bedrock,
            BlockId::Lava => TextureId::Lava,
            BlockId::SnowLeaves => TextureId::SnowLeaves,
            BlockId::Planks => TextureId::Planks,
            BlockId::Door => TextureId::Door,
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
            BlockId::Torch => TextureId::Torch,
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
