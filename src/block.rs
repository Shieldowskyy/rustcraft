// Block type definitions and their solid colors.
// (No textures - colors are used since this is a software rasterizer demo.)

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
        }
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, BlockId::Air | BlockId::Water | BlockId::Lava)
    }

    pub fn is_bedrock(self) -> bool {
        matches!(self, BlockId::Bedrock)
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
            BlockId::Snow => 0.2,
            BlockId::Cactus => 0.4,
            BlockId::CoalOre => 1.5,
            BlockId::IronOre => 1.5,
            BlockId::Bedrock => f32::INFINITY,
            BlockId::Lava => 0.0,
            BlockId::SnowLeaves => 0.2,
            BlockId::Planks => 1.5,
            BlockId::Door => 1.5,
            BlockId::Glass => 0.3,
            BlockId::Ice => 0.5,
            BlockId::Tnt => 0.0,
            BlockId::Water => 0.0,
        }
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, BlockId::Air | BlockId::Water | BlockId::Lava | BlockId::Leaves | BlockId::SnowLeaves | BlockId::Glass)
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
        }
    }

    /// Base RGB color for each face; grass has a green top and dirt sides.
    /// Used as a lighting/tint multiplier on top of the procedural texture.
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
            BlockId::Air => [0.0, 0.0, 0.0],
        }
    }

    /// Which procedural texture to use for a given face of this block.
    pub fn texture_id(self, face: Face) -> crate::texture::TextureId {
        use crate::texture::TextureId;
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
            BlockId::Air => TextureId::Stone, // never sampled
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

    pub fn normal(self) -> crate::math::IVec3 {
        match self {
            Face::Top => crate::math::IVec3::new(0, 1, 0),
            Face::Bottom => crate::math::IVec3::new(0, -1, 0),
            Face::North => crate::math::IVec3::new(0, 0, -1),
            Face::South => crate::math::IVec3::new(0, 0, 1),
            Face::East => crate::math::IVec3::new(1, 0, 0),
            Face::West => crate::math::IVec3::new(-1, 0, 0),
        }
    }
}
