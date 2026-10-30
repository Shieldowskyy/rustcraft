pub mod block;
pub mod mesher;
pub mod storage;
pub mod tile_entity;
pub mod weather;

use std::collections::{HashMap, HashSet, VecDeque};

pub use block::BlockId;
pub use mesher::{
    build_chunk_mesh, build_chunk_mesh_from_job, build_subchunk_mesh, AsyncMesher, ChunkMeshJob,
    LightSource, Triangle, Vertex, NUM_SUBCHUNKS, SUBCHUNK_SIZE,
};
pub use storage::{save_current_world, save_path};
pub use tile_entity::{fuel_burn_time, smelting_result, ChestData, FurnaceData, SMELT_DURATION};
pub use weather::{LightningStrike, PrecipitationKind, WeatherState, WeatherType};

use crate::core::math::{IVec3, Vec3};
use crate::core::noise::{fractal_noise2, fractal_noise3};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::{GameMode, SavedPlayerData};

pub const CHUNK_SIZE_X: i32 = 16;
pub const CHUNK_SIZE_Y: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 16;

pub const VIEW_DISTANCE: i32 = 4;
pub const DEFAULT_WORLD_SEED: u32 = 1337;

const CARDINAL_OFFSETS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<u8>,
    pub dirty: bool,
}

impl Chunk {
    #[inline(always)]
    pub fn index(x: i32, y: i32, z: i32) -> usize {
        ((x << 10) | (y << 4) | z) as usize
    }

    #[inline(always)]
    pub fn get_local(&self, x: i32, y: i32, z: i32) -> BlockId {
        if (x as u32) >= CHUNK_SIZE_X as u32 || (y as u32) >= CHUNK_SIZE_Y as u32 || (z as u32) >= CHUNK_SIZE_Z as u32 {
            return BlockId::Air;
        }
        BlockId::from_u8(self.blocks[Chunk::index(x, y, z)])
    }

    #[inline(always)]
    pub fn set_local(&mut self, x: i32, y: i32, z: i32, b: BlockId) {
        if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
            self.blocks[Chunk::index(x, y, z)] = b.to_u8();
            self.dirty = true;
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VillageComponent {
    CenterWell,
    Cottage,
    Blacksmith,
    Farm,
    Library,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    DeepOcean, Ocean, SnowyTaiga, SnowyTundra, AutumnForest, Forest, CherryGrove,
    Plains, Swamp, Savanna, Desert, Mountain, Jungle, Badlands, DarkForest,
    Taiga, BirchForest, IceSpikes, MushroomIsland, Meadow, JaggedPeaks,
    FrozenPeaks, SnowySlopes,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Self::DeepOcean => "DEEP OCEAN", Self::Ocean => "OCEAN", Self::MushroomIsland => "MUSHROOM ISLAND",
            Self::IceSpikes => "ICE SPIKES", Self::SnowyTaiga => "SNOWY TAIGA", Self::SnowyTundra => "SNOWY TUNDRA",
            Self::Taiga => "TAIGA", Self::Desert => "DESERT", Self::Badlands => "BADLANDS",
            Self::Savanna => "SAVANNA", Self::Jungle => "JUNGLE", Self::Swamp => "SWAMP",
            Self::Meadow => "MEADOW", Self::Plains => "PLAINS", Self::BirchForest => "BIRCH FOREST",
            Self::Forest => "FOREST", Self::DarkForest => "DARK FOREST", Self::AutumnForest => "AUTUMN FOREST",
            Self::CherryGrove => "CHERRY GROVE", Self::Mountain => "MOUNTAINS", Self::JaggedPeaks => "JAGGED PEAKS",
            Self::FrozenPeaks => "FROZEN PEAKS", Self::SnowySlopes => "SNOWY SLOPES",
        }
    }

    #[inline]
    pub fn color_filter(self) -> [f32; 3] {
        match self {
            Self::DeepOcean => [0.88, 0.97, 1.12], Self::Ocean => [0.92, 1.02, 1.08],
            Self::SnowyTaiga => [0.92, 0.98, 1.08], Self::SnowyTundra => [0.93, 1.00, 1.11],
            Self::AutumnForest => [1.11, 1.01, 0.89], Self::Forest => [0.97, 1.04, 0.97],
            Self::CherryGrove => [1.08, 0.97, 1.04], Self::Plains => [1.01, 1.03, 0.98],
            Self::Swamp => [0.94, 1.03, 0.86], Self::Savanna => [1.07, 1.04, 0.91],
            Self::Desert => [1.10, 1.03, 0.89], Self::Mountain => [0.97, 1.00, 1.04],
            Self::Jungle => [0.95, 1.08, 0.94], Self::Badlands => [1.12, 0.98, 0.90],
            Self::DarkForest => [0.91, 0.96, 0.93], Self::Taiga => [0.96, 1.02, 1.02],
            Self::BirchForest => [1.02, 1.06, 0.99], Self::IceSpikes => [0.90, 1.03, 1.14],
            Self::MushroomIsland => [1.05, 0.96, 1.08], Self::Meadow => [1.04, 1.05, 0.98],
            Self::JaggedPeaks => [0.95, 0.97, 1.05], Self::FrozenPeaks => [0.89, 1.01, 1.13],
            Self::SnowySlopes => [0.95, 1.01, 1.08],
        }
    }

    #[inline]
    pub fn has_snow(self) -> bool {
        matches!(self, Self::SnowyTaiga | Self::SnowyTundra | Self::IceSpikes | Self::FrozenPeaks | Self::SnowySlopes | Self::JaggedPeaks)
    }

    #[inline]
    pub fn has_precipitation(self) -> bool {
        !matches!(self, Self::Desert | Self::Badlands | Self::Savanna)
    }
}

#[inline(always)]
pub fn get_biome_at(wx: i32, wz: i32, seed: u32) -> (Biome, f32) {
    let (fx, fz) = (wx as f32, wz as f32);
    let continentalness = fractal_noise2(fx * 0.0012, fz * 0.0012, seed.wrapping_add(2500), 4, 0.5);
    let temp = fractal_noise2(fx * 0.0010, fz * 0.0010, seed.wrapping_add(500), 4, 0.5);
    let humidity = fractal_noise2(fx * 0.0010, fz * 0.0010, seed.wrapping_add(1500), 4, 0.5);
    let rarity = fractal_noise2(fx * 0.0014, fz * 0.0014, seed.wrapping_add(4500), 4, 0.5);

    let biome = if continentalness < -0.38 {
        if rarity > 0.38 { Biome::MushroomIsland } else { Biome::DeepOcean }
    } else if continentalness < -0.16 {
        if continentalness < -0.28 && rarity > 0.40 { Biome::MushroomIsland } else { Biome::Ocean }
    } else if continentalness > 0.34 {
        if continentalness < 0.46 {
            if continentalness < 0.44 && temp > -0.05 && temp < 0.22 && humidity > -0.05 { Biome::Meadow }
            else if temp < 0.02 { Biome::SnowySlopes }
            else { Biome::Mountain }
        } else if temp < 0.0 {
            if rarity > -0.05 { Biome::FrozenPeaks } else { Biome::JaggedPeaks }
        } else if temp < 0.12 { Biome::JaggedPeaks } else { Biome::Mountain }
    } else if temp < -0.30 {
        if continentalness > 0.24 {
            if rarity > 0.05 { Biome::FrozenPeaks } else { Biome::SnowySlopes }
        } else if temp < -0.40 && rarity > 0.30 { Biome::IceSpikes }
        else if humidity > 0.08 { Biome::SnowyTaiga } else { Biome::SnowyTundra }
    } else if temp < -0.05 {
        if humidity > 0.16 { Biome::AutumnForest }
        else if humidity > -0.05 { Biome::Taiga } else { Biome::Plains }
    } else if temp < 0.22 {
        if humidity > 0.28 {
            if rarity > 0.16 { Biome::DarkForest } else if temp < 0.10 { Biome::Swamp } else { Biome::Forest }
        } else if humidity > 0.10 {
            if rarity < -0.18 { Biome::BirchForest } else { Biome::Forest }
        } else if humidity > -0.04 && temp > 0.09 { Biome::CherryGrove } else { Biome::Plains }
    } else if humidity > 0.22 { Biome::Jungle }
    else if humidity < -0.18 && rarity > 0.12 { Biome::Badlands }
    else if humidity < -0.08 { Biome::Desert }
    else if humidity < 0.16 { Biome::Savanna } else { Biome::Forest };

    (biome, continentalness)
}

#[inline(always)]
pub fn get_height_at(wx: i32, wz: i32, seed: u32, biome: Biome, continentalness: f32) -> i32 {
    let n = fractal_noise2(wx as f32 * 0.012, wz as f32 * 0.012, seed, 4, 0.5);
    let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
    let lift = m_t * m_t * (3.0 - 2.0 * m_t);

    let h = match biome {
        Biome::DeepOcean => 6.0 + n * 3.0,
        Biome::Ocean => 13.0 + n * 4.0,
        Biome::MushroomIsland => 23.0 + n * 3.0,
        Biome::Swamp => 20.0 + n * 2.5,
        Biome::Meadow | Biome::SnowySlopes => 31.0 + n * 6.0 + lift * 18.0,
        Biome::Mountain => 31.0 + n * 6.0 + lift * (20.0 + (1.0 - (n * 2.0).abs().clamp(0.0, 1.0)) * 8.0),
        Biome::JaggedPeaks => 31.0 + n * 6.0 + lift * (20.0 + (1.0 - (n * 2.2).abs().clamp(0.0, 1.0)) * 9.0),
        Biome::FrozenPeaks => 31.0 + n * 6.0 + lift * (20.0 + (1.0 - (n * 1.8).abs().clamp(0.0, 1.0)) * 9.0),
        Biome::Badlands => ((32.0 + n * 8.0) / 3.0).floor() * 3.0 + 1.0,
        Biome::Desert => 25.0 + n * 8.0,
        Biome::Savanna => 26.0 + n * 5.0,
        Biome::Jungle => 27.0 + n * 8.0,
        Biome::DarkForest => 27.0 + n * 7.0,
        Biome::BirchForest => 28.0 + n * 9.0,
        Biome::AutumnForest | Biome::Forest | Biome::CherryGrove => 28.0 + n * 10.0,
        Biome::Taiga | Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::Plains => 27.0 + n * 10.0,
    };
    (h as i32).clamp(1, CHUNK_SIZE_Y - 2)
}

#[inline(always)]
fn set_blk(blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId) {
    if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
        blocks[Chunk::index(x, y, z)] = b.to_u8();
    }
}

#[inline(always)]
fn set_blk_if_air(blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId) {
    if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
        let idx = Chunk::index(x, y, z);
        if blocks[idx] == BlockId::Air.to_u8() {
            blocks[idx] = b.to_u8();
        }
    }
}

fn fill_box(blocks: &mut [u8], x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32, b: BlockId) {
    for x in x0..=x1 {
        for z in z0..=z1 {
            for y in y0..=y1 {
                set_blk(blocks, x, y, z, b);
            }
        }
    }
}

impl Chunk {
    pub fn generate(cx: i32, cz: i32, seed: u32) -> Chunk {
        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];

        enum Decoration {
            Cactus(i32, i32, i32, i32), PineTree(i32, i32, i32), Tree(i32, i32, i32),
            CherryTree(i32, i32, i32), AcaciaTree(i32, i32, i32), SwampTree(i32, i32, i32),
            AutumnTree(i32, i32, i32), JungleTree(i32, i32, i32, i32), JungleBush(i32, i32, i32),
            DarkOakTree(i32, i32, i32), BirchTree(i32, i32, i32), IceSpike(i32, i32, i32, i32),
            HugeRedMushroom(i32, i32, i32), HugeBrownMushroom(i32, i32, i32),
        }
        let mut decorations = Vec::new();

        enum Structure {
            DesertWell(i32, i32, i32), StoneRuins(i32, i32, i32), ForestCabin(i32, i32, i32),
            CoralRuin(i32, i32, i32), MountainShrine(i32, i32, i32), FrozenRuin(i32, i32, i32),
            DesertBuriedRuin(i32, i32, i32), OvergrownTempleRuin(i32, i32, i32),
            CaveDungeonRuin(i32, i32, i32), Village(i32, i32, i32, Biome, VillageComponent),
        }

        let (center_wx, center_wz) = (cx * CHUNK_SIZE_X + 8, cz * CHUNK_SIZE_Z + 8);
        let (c_biome, c_continentalness) = get_biome_at(center_wx, center_wz, seed);
        let center_height = get_height_at(center_wx, center_wz, seed, c_biome, c_continentalness);

        let reg_x = cx.div_euclid(20);
        let reg_z = cz.div_euclid(20);
        let v_hash = hash_coords(reg_x, reg_z, seed.wrapping_add(8881337));
        let v_cx = reg_x * 20 + ((v_hash % 16) as i32 + 2);
        let v_cz = reg_z * 20 + (((v_hash >> 16) % 16) as i32 + 2);

        let (v_biome, v_cont) = get_biome_at(v_cx * CHUNK_SIZE_X + 8, v_cz * CHUNK_SIZE_Z + 8, seed);
        let v_height = get_height_at(v_cx * CHUNK_SIZE_X + 8, v_cz * CHUNK_SIZE_Z + 8, seed, v_biome, v_cont);

        let village_comp = if matches!(v_biome, Biome::Plains | Biome::Meadow | Biome::Savanna | Biome::Desert | Biome::Taiga)
            && v_height > 22 && v_height < CHUNK_SIZE_Y - 14
        {
            match (cx - v_cx, cz - v_cz) {
                (0, 0) => Some(VillageComponent::CenterWell),
                (1, 0) => Some(VillageComponent::Cottage),
                (-1, 0) => Some(VillageComponent::Blacksmith),
                (0, 1) => Some(VillageComponent::Farm),
                (0, -1) => Some(VillageComponent::Library),
                _ => None,
            }
        } else {
            None
        };

        let chunk_hash = hash_coords(cx, cz, seed.wrapping_add(77777));
        let mut structure: Option<Structure> = None;
        let mut struct_exclusion = (100, -100, 100, -100);

        if let Some(comp) = village_comp {
            structure = Some(Structure::Village(8, v_height, 8, v_biome, comp));
            struct_exclusion = (0, 16, 0, 16);
        } else if center_height > 22 && center_height < CHUNK_SIZE_Y - 12 {
            let is_desert = matches!(c_biome, Biome::Desert | Biome::Badlands);
            if is_desert && chunk_hash % 71 == 0 {
                structure = Some(Structure::DesertWell(8, center_height, 8));
                struct_exclusion = (4, 12, 4, 12);
            } else if is_desert && chunk_hash % 61 == 0 {
                structure = Some(Structure::DesertBuriedRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Mountain | Biome::JaggedPeaks | Biome::SnowySlopes) && chunk_hash % 59 == 0 {
                structure = Some(Structure::MountainShrine(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::FrozenPeaks | Biome::SnowyTundra | Biome::IceSpikes | Biome::SnowyTaiga) && chunk_hash % 63 == 0 {
                structure = Some(Structure::FrozenRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Jungle | Biome::DarkForest | Biome::Swamp) && chunk_hash % 67 == 0 {
                structure = Some(Structure::OvergrownTempleRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Mountain | Biome::SnowyTundra | Biome::Taiga) && chunk_hash % 73 == 0 {
                structure = Some(Structure::StoneRuins(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Forest | Biome::AutumnForest | Biome::DarkForest | Biome::BirchForest) && chunk_hash % 79 == 0 {
                structure = Some(Structure::ForestCabin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            }
        }
        if c_biome == Biome::DeepOcean && (4..12).contains(&center_height) && chunk_hash % 53 == 0 {
            structure = Some(Structure::CoralRuin(8, center_height, 8));
            struct_exclusion = (4, 12, 4, 12);
        }
        if structure.is_none() && center_height >= 30 && chunk_hash % 79 == 0 {
            structure = Some(Structure::CaveDungeonRuin(8, 10 + (chunk_hash % 8) as i32, 8));
        }

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let (wx, wz) = (cx * CHUNK_SIZE_X + lx, cz * CHUNK_SIZE_Z + lz);
                let (biome, continentalness) = get_biome_at(wx, wz, seed);
                let height = get_height_at(wx, wz, seed, biome, continentalness);
                let patch_noise = fractal_noise2(wx as f32 * 0.045, wz as f32 * 0.045, seed.wrapping_add(8881), 2, 0.5);
                let is_mountain = matches!(biome, Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes);

                for y in 0..CHUNK_SIZE_Y {
                    if y == 0 {
                        blocks[Chunk::index(lx, y, lz)] = BlockId::Bedrock.to_u8();
                        continue;
                    }
                    if y == 1 {
                        let b = if hash_coords_3d(wx, y, wz, seed) % 3 == 0 { BlockId::Bedrock } else { BlockId::Stone };
                        blocks[Chunk::index(lx, y, lz)] = b.to_u8();
                        continue;
                    }

                    let block = if y > height {
                        if y <= 20 {
                            if y == 20 && matches!(biome, Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::FrozenPeaks) {
                                BlockId::Ice
                            } else {
                                BlockId::Water
                            }
                        } else {
                            BlockId::Air
                        }
                    } else if y == height {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean | Biome::Desert => BlockId::Sand,
                            Biome::Swamp => if height <= 20 { BlockId::Mud } else { BlockId::Grass },
                            Biome::MushroomIsland => BlockId::Mud,
                            Biome::Badlands => match (height / 2) % 3 { 0 => BlockId::Sand, 1 => BlockId::Stone, _ => BlockId::Mud },
                            Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::SnowySlopes => BlockId::Snow,
                            Biome::JaggedPeaks => {
                                let snow = if height >= 52 { patch_noise >= -0.35 } else if height >= 40 { patch_noise > -0.05 } else { patch_noise > 0.15 };
                                if snow { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::FrozenPeaks => if patch_noise > 0.28 { BlockId::Ice } else { BlockId::Snow },
                            Biome::Mountain => {
                                if (height as f32) >= 48.0 + patch_noise * 3.0 { BlockId::Snow }
                                else if (height as f32) >= 37.0 + patch_noise * 3.0 { BlockId::Stone }
                                else { BlockId::Grass }
                            }
                            _ if height <= 21 => match hash_coords(wx, wz, seed.wrapping_add(51893)) % 10 {
                                0..=2 => BlockId::Clay,
                                3..=4 => BlockId::Gravel,
                                _ => BlockId::Sand,
                            },
                            _ => BlockId::Grass,
                        }
                    } else if y > height - 4 {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean => BlockId::Sand,
                            Biome::Desert => if y <= height - 3 { BlockId::Sandstone } else { BlockId::Sand },
                            Biome::Badlands => match (y / 2) % 3 { 0 => BlockId::Sand, 1 => BlockId::Stone, _ => BlockId::Mud },
                            Biome::Swamp | Biome::MushroomIsland => BlockId::Mud,
                            Biome::IceSpikes => if y > height - 2 { BlockId::Snow } else { BlockId::Ice },
                            Biome::FrozenPeaks => {
                                if patch_noise > 0.28 { if y > height - 5 { BlockId::Ice } else { BlockId::Stone } }
                                else if y > height - 3 { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::JaggedPeaks => {
                                let is_snow = if height >= 52 { patch_noise >= -0.35 } else if height >= 40 { patch_noise > -0.05 } else { patch_noise > 0.15 };
                                if is_snow && y > height - 2 { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::SnowySlopes => if y > height - 3 { BlockId::Snow } else { BlockId::Stone },
                            Biome::Mountain => {
                                if (height as f32) >= 37.0 + patch_noise * 3.0 { BlockId::Stone }
                                else if height <= 21 { BlockId::Sand } else { BlockId::Dirt }
                            }
                            _ if height <= 21 => match hash_coords(wx, wz, seed.wrapping_add(51893)) % 10 {
                                0..=2 => BlockId::Clay,
                                3..=4 => BlockId::Gravel,
                                _ => BlockId::Sand,
                            },
                            _ => BlockId::Dirt,
                        }
                    } else {
                        let ore_h = hash_coords_3d(wx, y, wz, seed);
                        if biome == Biome::Badlands && y >= 14 {
                            if ore_h % 37 == 0 { BlockId::CoalOre }
                            else if ore_h % 67 == 0 && y < 30 { BlockId::IronOre }
                            else if ore_h % 127 == 0 && y < 20 { BlockId::GoldOre }
                            else if ore_h % 79 == 0 && y < 24 { BlockId::RustStoneOre }
                            else { match (y / 2) % 4 { 0 => BlockId::Sand, 2 => BlockId::Mud, _ => BlockId::Stone } }
                        } else if ore_h % 37 == 0 { BlockId::CoalOre }
                        else if ore_h % 43 == 0 && y > 15 && y < 55 { BlockId::Gravel }
                        else if ore_h % 67 == 0 && y < 30 { BlockId::IronOre }
                        else if ore_h % 127 == 0 && y < 20 { BlockId::GoldOre }
                        else if ore_h % 79 == 0 && y < 24 { BlockId::RustStoneOre }
                        else if ore_h % 151 == 0 && y < 14 { BlockId::Obsidian }
                        else if ore_h % 199 == 0 && y < 14 { BlockId::DiamondOre }
                        else if is_mountain && ore_h % 149 == 0 && (10..=60).contains(&y) { BlockId::EmeraldOre }
                        else { BlockId::Stone }
                    };

                    blocks[Chunk::index(lx, y, lz)] = block.to_u8();

                    let mut is_cave = false;
                    if y > 1 && y < height - 1 && block != BlockId::Water && block != BlockId::Air {
                        let c1 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, seed.wrapping_add(3141), 2, 0.5);
                        let c2 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, seed.wrapping_add(9265), 2, 0.5);
                        is_cave = (c1 * c1 + c2 * c2) < 0.024;
                    }
                    if is_mountain && y >= 20 && block != BlockId::Water && block != BlockId::Air {
                        let mc1 = fractal_noise3(wx as f32 * 0.036, y as f32 * 0.044, wz as f32 * 0.036, seed.wrapping_add(14423), 2, 0.5);
                        let mc2 = fractal_noise3(wx as f32 * 0.036, y as f32 * 0.044, wz as f32 * 0.036, seed.wrapping_add(27819), 2, 0.5);
                        let cave_dist_sq = mc1 * mc1 + mc2 * mc2;
                        if cave_dist_sq < 0.026 && (y < height || (cave_dist_sq < 0.015 && height >= 32)) {
                            is_cave = true;
                        }
                    }
                    if is_cave {
                        blocks[Chunk::index(lx, y, lz)] = if y <= 7 { BlockId::Lava.to_u8() } else { BlockId::Air.to_u8() };
                    }
                }

                if height > 21 {
                    if lx >= struct_exclusion.0 && lx <= struct_exclusion.1 && lz >= struct_exclusion.2 && lz <= struct_exclusion.3 {
                        continue;
                    }
                    if (3..CHUNK_SIZE_X - 3).contains(&lx) && (3..CHUNK_SIZE_Z - 3).contains(&lz) {
                        let h = hash_coords(wx, wz, seed);
                        let by = height + 1;
                        match biome {
                            Biome::Desert if h % 167 == 0 => decorations.push(Decoration::Cactus(lx, by, lz, 2 + (h % 2) as i32)),
                            Biome::Badlands if h % 181 == 0 => decorations.push(Decoration::Cactus(lx, by, lz, 2 + (h % 2) as i32)),
                            Biome::SnowyTaiga if h % 97 == 0 => decorations.push(Decoration::PineTree(lx, by, lz)),
                            Biome::Taiga if h % 59 == 0 => decorations.push(Decoration::PineTree(lx, by, lz)),
                            Biome::SnowyTundra if h % 233 == 0 => decorations.push(Decoration::PineTree(lx, by, lz)),
                            Biome::IceSpikes if h % 47 == 0 => decorations.push(Decoration::IceSpike(lx, by, lz, 7 + (h % 9) as i32)),
                            Biome::AutumnForest if h % 47 == 0 => decorations.push(Decoration::AutumnTree(lx, by, lz)),
                            Biome::Forest if h % 43 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::BirchForest if h % 41 == 0 => decorations.push(Decoration::BirchTree(lx, by, lz)),
                            Biome::DarkForest => {
                                if h % 29 == 0 { decorations.push(Decoration::DarkOakTree(lx, by, lz)); }
                                else if h % 83 == 0 { decorations.push(Decoration::HugeRedMushroom(lx, by, lz)); }
                                else if h % 89 == 0 { decorations.push(Decoration::HugeBrownMushroom(lx, by, lz)); }
                            }
                            Biome::Jungle => {
                                if h % 43 == 0 { decorations.push(Decoration::JungleTree(lx, by, lz, 10 + (h % 7) as i32)); }
                                else if h % 31 == 0 { decorations.push(Decoration::JungleBush(lx, by, lz)); }
                            }
                            Biome::MushroomIsland => {
                                if h % 37 == 0 { decorations.push(Decoration::HugeRedMushroom(lx, by, lz)); }
                                else if h % 41 == 0 { decorations.push(Decoration::HugeBrownMushroom(lx, by, lz)); }
                            }
                            Biome::CherryGrove if h % 53 == 0 => decorations.push(Decoration::CherryTree(lx, by, lz)),
                            Biome::Swamp if h % 67 == 0 => decorations.push(Decoration::SwampTree(lx, by, lz)),
                            Biome::Savanna if h % 163 == 0 => decorations.push(Decoration::AcaciaTree(lx, by, lz)),
                            Biome::Plains if h % 149 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::Meadow if h % 199 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::SnowySlopes if h % 127 == 0 => decorations.push(Decoration::PineTree(lx, by, lz)),
                            _ => {}
                        }
                    }
                } else if height < 20 {
                    let bed_block = BlockId::from_u8(blocks[Chunk::index(lx, height, lz)]);
                    if matches!(bed_block, BlockId::Sand | BlockId::Dirt | BlockId::Mud) {
                        let h = hash_coords(wx, wz, seed.wrapping_add(8831));
                        let water_depth = 20 - height;
                        if water_depth >= 3 && matches!(biome, Biome::Ocean | Biome::DeepOcean) {
                            if h % 5 == 0 {
                                let kelp_len = 1 + (h % (water_depth - 1).clamp(1, 14) as u32) as i32;
                                for dy in 1..=kelp_len {
                                    if height + dy < 20 {
                                        blocks[Chunk::index(lx, height + dy, lz)] = BlockId::Kelp.to_u8();
                                    }
                                }
                            } else if h % 3 == 0 {
                                blocks[Chunk::index(lx, height + 1, lz)] = BlockId::Seagrass.to_u8();
                            }
                        } else if water_depth >= 1 && h % 4 == 0 {
                            blocks[Chunk::index(lx, height + 1, lz)] = BlockId::Seagrass.to_u8();
                        }
                    }
                }
            }
        }

        if let Some(struct_type) = structure {
            match struct_type {
                Structure::DesertWell(cx, y, cz) => place_desert_well(&mut blocks, cx, y, cz),
                Structure::StoneRuins(cx, y, cz) => place_stone_ruin(&mut blocks, cx, y, cz, seed),
                Structure::ForestCabin(cx, y, cz) => place_forest_cabin(&mut blocks, cx, y, cz),
                Structure::CoralRuin(cx, y, cz) => place_coral_ruin(&mut blocks, cx, y, cz, seed),
                Structure::MountainShrine(cx, y, cz) => place_mountain_shrine(&mut blocks, cx, y, cz, seed),
                Structure::FrozenRuin(cx, y, cz) => place_frozen_ruin(&mut blocks, cx, y, cz, seed),
                Structure::DesertBuriedRuin(cx, y, cz) => place_desert_buried_ruin(&mut blocks, cx, y, cz, seed),
                Structure::OvergrownTempleRuin(cx, y, cz) => place_overgrown_temple_ruin(&mut blocks, cx, y, cz, seed),
                Structure::CaveDungeonRuin(cx, y, cz) => place_cave_dungeon_ruin(&mut blocks, cx, y, cz, seed),
                Structure::Village(cx, y, cz, b, comp) => place_village_component(&mut blocks, cx, y, cz, b, seed, comp),
            }
        }

        for deco in decorations {
            match deco {
                Decoration::Cactus(x, y, z, h) => place_trunk(&mut blocks, x, y, z, h, BlockId::Cactus),
                Decoration::PineTree(x, y, z) => place_pine_tree(&mut blocks, x, y, z),
                Decoration::Tree(x, y, z) => place_ellipsoid_tree(&mut blocks, x, y, z, 4, 2, 2, 1, 1, 2, 6, BlockId::Leaves),
                Decoration::CherryTree(x, y, z) => place_ellipsoid_tree(&mut blocks, x, y, z, 3, 3, 3, 10, 10, 25, 90, BlockId::CherryLeaves),
                Decoration::AutumnTree(x, y, z) => place_ellipsoid_tree(&mut blocks, x, y, z, 4, 2, 3, 1, 1, 2, 7, BlockId::AutumnLeaves),
                Decoration::AcaciaTree(x, y, z) => place_acacia_tree(&mut blocks, x, y, z),
                Decoration::SwampTree(x, y, z) => place_swamp_tree(&mut blocks, x, y, z),
                Decoration::JungleTree(x, y, z, h) => place_jungle_tree(&mut blocks, x, y, z, h),
                Decoration::JungleBush(x, y, z) => place_jungle_bush(&mut blocks, x, y, z),
                Decoration::DarkOakTree(x, y, z) => place_dark_oak_tree(&mut blocks, x, y, z),
                Decoration::BirchTree(x, y, z) => place_birch_tree(&mut blocks, x, y, z),
                Decoration::IceSpike(x, y, z, h) => place_ice_spike(&mut blocks, x, y, z, h),
                Decoration::HugeRedMushroom(x, y, z) => place_huge_red_mushroom(&mut blocks, x, y, z),
                Decoration::HugeBrownMushroom(x, y, z) => place_huge_brown_mushroom(&mut blocks, x, y, z),
            }
        }

        Chunk { cx, cz, blocks, dirty: true }
    }
}

fn hash_coords(x: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((z as u32).wrapping_mul(668265263)).wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn hash_coords_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(1103515245)).wrapping_add((z as u32).wrapping_mul(668265263)).wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn place_trunk(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32, block: BlockId) {
    for i in 0..height {
        set_blk(blocks, lx, base_y + i, lz, block);
    }
}

#[allow(clippy::too_many_arguments)]
fn place_ellipsoid_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, trunk_h: i32, rx: i32, ymax: i32, wx: i32, wz: i32, wy: i32, max_d: i32, leaf: BlockId) {
    place_trunk(blocks, lx, base_y, lz, trunk_h, BlockId::Wood);
    for dx in -rx..=rx {
        for dz in -rx..=rx {
            for dy in 0..=ymax {
                if dx * dx * wx + dz * dz * wz + (dy - 1) * (dy - 1) * wy <= max_d {
                    set_blk_if_air(blocks, lx + dx, base_y + trunk_h - 1 + dy, lz + dz, leaf);
                }
            }
        }
    }
}

fn place_pine_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 6, BlockId::Wood);
    for dy in 2..=7 {
        let r = if dy == 7 { 0 } else if dy >= 5 { 1 } else if dy % 2 == 0 { 2 } else { 1 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx == 0 && dz == 0 && dy < 6 { continue; }
                set_blk_if_air(blocks, lx + dx, base_y + dy, lz + dz, BlockId::SnowLeaves);
            }
        }
    }
}

fn place_acacia_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 5, BlockId::Wood);
    for dy in 0..=1 {
        let r = if dy == 0 { 3 } else { 2 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r * r {
                    set_blk_if_air(blocks, lx + dx, base_y + 5 + dy, lz + dz, BlockId::Leaves);
                }
            }
        }
    }
}

fn place_swamp_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 6, BlockId::Wood);
    for dy in 0..=3 {
        let r = match dy { 0 | 1 => 3, 2 => 2, _ => 1 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r * r {
                    set_blk_if_air(blocks, lx + dx, base_y + 5 + dy, lz + dz, BlockId::Leaves);
                }
            }
        }
    }
}

fn place_jungle_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, trunk_h: i32) {
    place_trunk(blocks, lx, base_y, lz, trunk_h, BlockId::Wood);
    for dy in -1..=2 {
        let r = match dy { -1 => 2, 0 | 1 => 3, _ => 1 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r * r + 1 {
                    set_blk_if_air(blocks, lx + dx, base_y + trunk_h + dy, lz + dz, BlockId::Leaves);
                }
            }
        }
    }
}

fn place_jungle_bush(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    set_blk(blocks, lx, base_y, lz, BlockId::Wood);
    for dx in -1..=1 {
        for dz in -1..=1 {
            for dy in 0..=1 {
                set_blk_if_air(blocks, lx + dx, base_y + dy, lz + dz, BlockId::Leaves);
            }
        }
    }
}

fn place_dark_oak_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 5, BlockId::Wood);
    for dy in 0..=2 {
        let r = if dy == 2 { 2 } else { 3 };
        for dx in -r..=r {
            for dz in -r..=r {
                set_blk_if_air(blocks, lx + dx, base_y + 4 + dy, lz + dz, BlockId::Leaves);
            }
        }
    }
}

fn place_birch_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 5, BlockId::Wood);
    for dy in 0..=2 {
        let r: i32 = if dy == 2 { 1 } else { 2 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx.abs() + dz.abs() <= r + 1 {
                    set_blk_if_air(blocks, lx + dx, base_y + 4 + dy, lz + dz, BlockId::Leaves);
                }
            }
        }
    }
}

fn place_ice_spike(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32) {
    for dy in 0..height {
        let r = if dy < height / 3 { 2 } else if dy < (height * 2) / 3 { 1 } else { 0 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r * r + 1 {
                    set_blk(blocks, lx + dx, base_y + dy, lz + dz, BlockId::Ice);
                }
            }
        }
    }
    set_blk(blocks, lx, base_y + height, lz, BlockId::Snow);
}

fn place_huge_red_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 5, BlockId::Snow);
    for dy in 0..=2 {
        let r: i32 = if dy == 2 { 1 } else { 2 };
        for dx in -r..=r {
            for dz in -r..=r {
                if dy == 0 && dx.abs() != r && dz.abs() != r { continue; }
                set_blk_if_air(blocks, lx + dx, base_y + 4 + dy, lz + dz, BlockId::AutumnLeaves);
            }
        }
    }
}

fn place_huge_brown_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 4, BlockId::Snow);
    for dx in -2..=2 {
        for dz in -2..=2 {
            set_blk_if_air(blocks, lx + dx, base_y + 4, lz + dz, BlockId::Mud);
        }
    }
}

fn place_coral_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    for &(dx, dz) in &[(-2, -2), (-2, 2), (2, -2), (2, 2), (0, -3), (3, 0), (-3, 0), (0, 3)] {
        let (x, z) = (cx + dx, cz + dz);
        let col_h = 2 + (hash_coords_3d(x, 77, z, seed) % 5) as i32;
        let b = if (dx + dz) % 2 == 0 { BlockId::Stone } else { BlockId::Sand };
        place_trunk(blocks, x, by, z, col_h + 1, b);
    }
    fill_box(blocks, cx - 1, cx + 1, by, by + 1, cz - 1, cz + 1, BlockId::Stone);
    set_blk(blocks, cx, by + 2, cz, BlockId::Lava);
}

fn place_desert_well(blocks: &mut [u8], cx: i32, by: i32, cz: i32) {
    fill_box(blocks, cx - 2, cx + 2, by - 4, by - 1, cz - 2, cz + 2, BlockId::Sand);
    fill_box(blocks, cx - 2, cx + 2, by + 1, by + 6, cz - 2, cz + 2, BlockId::Air);
    fill_box(blocks, cx - 2, cx + 2, by, by, cz - 2, cz + 2, BlockId::Sand);
    fill_box(blocks, cx - 1, cx + 1, by, by, cz - 1, cz + 1, BlockId::Water);

    for &(x, z) in &[(cx - 2, cz - 2), (cx - 2, cz + 2), (cx + 2, cz - 2), (cx + 2, cz + 2)] {
        place_trunk(blocks, x, by + 1, z, 3, BlockId::Sand);
    }
    fill_box(blocks, cx - 2, cx + 2, by + 4, by + 4, cz - 2, cz + 2, BlockId::Sand);
    set_blk(blocks, cx, by + 5, cz, BlockId::Sand);
}

fn place_stone_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by - 4, by - 1, z0, z1, BlockId::Stone);
    fill_box(blocks, x0, x1, by + 1, by + 6, z0, z1, BlockId::Air);

    for x in x0..=x1 {
        for z in z0..=z1 {
            let h = hash_coords_3d(x, by, z, seed);
            let fb = if h % 11 == 0 { BlockId::CoalOre } else if h % 17 == 0 { BlockId::IronOre } else { BlockId::Stone };
            set_blk(blocks, x, by, z, fb);

            if (x == x0 || x == x1 || z == z0 || z == z1) && !((x == cx || x == cx + 1) && z == z0) {
                let wh = (hash_coords_3d(x, 101, z, seed) % 4) as i32;
                place_trunk(blocks, x, by + 1, z, wh, BlockId::Stone);
            }
        }
    }
    for &(x, z) in &[(x0, z0), (x0, z1), (x1, z0), (x1, z1)] {
        let ph = 2 + (hash_coords_3d(x, 137, z, seed) % 3) as i32;
        place_trunk(blocks, x, by + 1, z, ph, BlockId::Stone);
    }
    for &(dx, dz) in &[(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        set_blk(blocks, cx + dx, by + 1, cz + dz, BlockId::Stone);
    }
    set_blk(blocks, cx, by + 2, cz, BlockId::Lava);
}

fn place_forest_cabin(blocks: &mut [u8], cx: i32, by: i32, cz: i32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 2, cz - 3, cz + 2);
    fill_box(blocks, x0 - 1, x1 + 1, by + 1, by + 8, z0 - 1, z1 + 1, BlockId::Air);
    fill_box(blocks, x0, x1, by - 4, by - 1, z0, z1, BlockId::Wood);
    fill_box(blocks, x0, x1, by, by, z0, z1, BlockId::Planks);

    for x in x0..=x1 {
        for z in z0..=z1 {
            if (x == x0 || x == x1) && (z == z0 || z == z1) {
                place_trunk(blocks, x, by + 1, z, 3, BlockId::Wood);
            } else if x == x0 || x == x1 || z == z0 || z == z1 {
                for dy in 1..=3 {
                    let b = if x == cx && z == z0 && dy <= 2 { BlockId::Door }
                    else if dy == 2 && (((x == x0 || x == x1) && (z == z0 + 2 || z == z0 + 3)) || (z == z1 && (x == x0 + 2 || x == x0 + 3))) { BlockId::Glass }
                    else { BlockId::Planks };
                    set_blk(blocks, x, by + dy, z, b);
                }
            }
        }
    }
    for x in (x0 - 1)..=(x1 + 1) {
        for z in (z0 - 1)..=(z1 + 1) {
            let edge = x == x0 - 1 || x == x1 + 1 || z == z0 - 1 || z == z1 + 1;
            set_blk(blocks, x, by + 4, z, if edge { BlockId::Wood } else { BlockId::Planks });
        }
    }
    fill_box(blocks, x0, x1, by + 5, by + 5, z0 - 1, z1 + 1, BlockId::Planks);
    fill_box(blocks, cx - 1, cx, by + 6, by + 6, z0 - 1, z1 + 1, BlockId::Wood);

    set_blk(blocks, x0 + 1, by + 1, z1 - 1, BlockId::CraftingTable);
    set_blk(blocks, x0 + 2, by + 1, z1 - 1, BlockId::Furnace);
    set_blk(blocks, x1 - 1, by + 1, z1 - 1, BlockId::Chest);
    set_blk(blocks, x0 + 1, by + 3, z1 - 1, BlockId::Torch);
}

fn place_village_component(blocks: &mut [u8], _cx: i32, by: i32, _cz: i32, biome: Biome, _seed: u32, comp: VillageComponent) {
    let is_desert = matches!(biome, Biome::Desert | Biome::Badlands);
    let wood = if is_desert { BlockId::Sand } else { BlockId::Wood };
    let plank = if is_desert { BlockId::Sand } else { BlockId::Planks };
    let path = if is_desert { BlockId::Sand } else { BlockId::Dirt };
    let stone = BlockId::Stone;

    fill_box(blocks, 0, CHUNK_SIZE_X - 1, by + 1, by + 10, 0, CHUNK_SIZE_Z - 1, BlockId::Air);
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            for y in (by - 4)..=by {
                set_blk_if_air(blocks, x, y, z, path);
            }
        }
    }

    let place_lamp = |blocks: &mut [u8], lx: i32, lz: i32| {
        place_trunk(blocks, lx, by + 1, lz, 2, wood);
        set_blk(blocks, lx, by + 3, lz, BlockId::Torch);
    };

    match comp {
        VillageComponent::CenterWell => {
            fill_box(blocks, 7, 8, by, by, 0, CHUNK_SIZE_Z - 1, path);
            fill_box(blocks, 0, CHUNK_SIZE_X - 1, by, by, 7, 8, path);
            for x in 6..=9 {
                for z in 6..=9 {
                    if x == 6 || x == 9 || z == 6 || z == 9 {
                        set_blk(blocks, x, by + 1, z, stone);
                    } else {
                        fill_box(blocks, x, x, by - 3, by, z, z, BlockId::Water);
                    }
                    set_blk(blocks, x, by - 4, z, stone);
                }
            }
            for &(wx, wz) in &[(6, 6), (9, 6), (6, 9), (9, 9)] {
                place_trunk(blocks, wx, by + 2, wz, 3, wood);
            }
            fill_box(blocks, 6, 9, by + 5, by + 5, 6, 9, stone);
            for &(lx, lz) in &[(4, 4), (11, 4), (4, 11), (11, 11)] { place_lamp(blocks, lx, lz); }
            for &(bx, bz) in &[(4, 7), (4, 8), (11, 7), (11, 8)] { set_blk(blocks, bx, by + 1, bz, plank); }
        }
        VillageComponent::Cottage => {
            fill_box(blocks, 0, 5, by, by, 7, 8, path);
            fill_box(blocks, 5, 12, by, by, 4, 11, plank);
            for x in 5..=12 {
                for z in 4..=11 {
                    if (x == 5 || x == 12) && (z == 4 || z == 11) {
                        place_trunk(blocks, x, by + 1, z, 4, wood);
                    } else if x == 5 || x == 12 || z == 4 || z == 11 {
                        for dy in 1..=3 {
                            let b = if x == 5 && z == 7 && dy <= 2 { BlockId::Door }
                            else if dy == 2 && ((z == 7 && x == 12) || (x == 8 && (z == 4 || z == 11))) { BlockId::Glass }
                            else { plank };
                            set_blk(blocks, x, by + dy, z, b);
                        }
                    }
                }
            }
            fill_box(blocks, 4, 13, by + 4, by + 4, 3, 12, wood);
            fill_box(blocks, 5, 12, by + 5, by + 5, 5, 10, plank);
            set_blk(blocks, 11, by + 1, 10, BlockId::Bed);
            set_blk(blocks, 11, by + 1, 5, BlockId::CraftingTable);
            set_blk(blocks, 10, by + 1, 5, BlockId::Planks);
            set_blk(blocks, 7, by + 1, 10, BlockId::Chest);
            set_blk(blocks, 9, by + 3, 10, BlockId::Torch);
            place_lamp(blocks, 3, 5);
        }
        VillageComponent::Blacksmith => {
            fill_box(blocks, 10, CHUNK_SIZE_X - 1, by, by, 7, 8, path);
            fill_box(blocks, 4, 11, by, by, 4, 11, stone);
            for &(fx, fz) in &[(4, 4), (11, 4), (4, 11), (11, 11), (7, 4), (7, 11)] {
                place_trunk(blocks, fx, by + 1, fz, 3, stone);
            }
            fill_box(blocks, 4, 11, by + 4, by + 4, 4, 11, stone);
            set_blk(blocks, 9, by + 1, 5, BlockId::Furnace);
            set_blk(blocks, 10, by + 1, 5, BlockId::Furnace);
            set_blk(blocks, 8, by + 1, 5, BlockId::Lava);
            set_blk(blocks, 8, by + 1, 8, BlockId::StoneSlab);
            set_blk(blocks, 9, by + 1, 10, BlockId::Chest);
            set_blk(blocks, 7, by + 3, 5, BlockId::Torch);
            set_blk(blocks, 7, by + 3, 10, BlockId::Torch);
            fill_box(blocks, 4, 4, by + 1, by + 3, 5, 10, stone);
            set_blk(blocks, 5, by + 1, 5, BlockId::Bed);
            set_blk(blocks, 5, by + 1, 10, BlockId::CraftingTable);
            set_blk(blocks, 5, by + 3, 7, BlockId::Torch);
        }
        VillageComponent::Farm => {
            fill_box(blocks, 7, 8, by, by, 0, CHUNK_SIZE_Z - 1, path);
            for &(x0, x1, wx, c1, c2) in &[(2, 6, 4, 3, 5), (9, 13, 11, 10, 12)] {
                fill_box(blocks, x0, x1, by, by, 3, 3, wood);
                fill_box(blocks, x0, x1, by, by, 13, 13, wood);
                fill_box(blocks, x0, x0, by, by, 3, 13, wood);
                fill_box(blocks, x1, x1, by, by, 3, 13, wood);
                for z in 4..=12 {
                    set_blk(blocks, wx, by, z, BlockId::Water);
                    for &x in &[c1, c2] {
                        set_blk(blocks, x, by, z, BlockId::FarmlandWet);
                        let crop = match (x + z) % 3 { 0 => BlockId::WheatCrop, 1 => BlockId::CarrotCrop, _ => BlockId::PotatoCrop };
                        set_blk(blocks, x, by + 1, z, crop);
                    }
                }
            }
            place_lamp(blocks, 6, 2);
            place_lamp(blocks, 9, 2);
        }
        VillageComponent::Library => {
            fill_box(blocks, 7, 8, by, by, 10, CHUNK_SIZE_Z - 1, path);
            fill_box(blocks, 4, 11, by, by, 3, 10, stone);
            for x in 4..=11 {
                for z in 3..=10 {
                    if (x == 4 || x == 11) && (z == 3 || z == 10) {
                        place_trunk(blocks, x, by + 1, z, 4, wood);
                    } else if x == 4 || x == 11 || z == 3 || z == 10 {
                        for dy in 1..=3 {
                            let b = if (x == 7 || x == 8) && z == 10 && dy <= 2 { BlockId::Door }
                            else if dy == 2 && (x == 4 || x == 11) && z == 6 { BlockId::Glass }
                            else { plank };
                            set_blk(blocks, x, by + dy, z, b);
                        }
                    }
                }
            }
            fill_box(blocks, 3, 12, by + 4, by + 4, 2, 11, wood);
            fill_box(blocks, 5, 10, by + 5, by + 5, 4, 9, plank);
            fill_box(blocks, 5, 10, by + 1, by + 2, 4, 4, BlockId::Planks);
            set_blk(blocks, 6, by + 1, 7, BlockId::CraftingTable);
            set_blk(blocks, 7, by + 1, 7, BlockId::WoodenSlab);
            set_blk(blocks, 10, by + 1, 8, BlockId::Chest);
            set_blk(blocks, 5, by + 3, 6, BlockId::Torch);
            set_blk(blocks, 10, by + 3, 6, BlockId::Torch);
            place_lamp(blocks, 5, 11);
        }
    }
}

fn place_mountain_shrine(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by - 4, by, z0, z1, BlockId::Stone);
    fill_box(blocks, x0, x1, by + 1, by + 7, z0, z1, BlockId::Air);
    for &(x, z) in &[(x0, z0), (x0, z1), (x1, z0), (x1, z1)] {
        let ph = 4 + (hash_coords_3d(x, 137, z, seed) % 2) as i32;
        place_trunk(blocks, x, by + 1, z, ph, BlockId::Stone);
        set_blk(blocks, x, by + ph + 1, z, BlockId::Torch);
    }
    fill_box(blocks, cx - 1, cx + 1, by + 1, by + 1, cz - 1, cz + 1, BlockId::Stone);
    set_blk(blocks, cx, by + 2, cz, BlockId::GoldBlock);
    set_blk(blocks, cx, by + 3, cz, BlockId::Torch);
    set_blk(blocks, cx + 1, by + 2, cz, BlockId::Chest);
}

fn place_frozen_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by - 4, by - 1, z0, z1, BlockId::Ice);
    fill_box(blocks, x0, x1, by + 1, by + 6, z0, z1, BlockId::Air);
    for x in x0..=x1 {
        for z in z0..=z1 {
            set_blk(blocks, x, by, z, if hash_coords_3d(x, by, z, seed) % 2 == 0 { BlockId::Ice } else { BlockId::Stone });
            if (x == x0 || x == x1 || z == z0 || z == z1) && !((x == cx || x == cx + 1) && z == z0) {
                let wh = (hash_coords_3d(x, 101, z, seed) % 4) as i32;
                for dy in 1..=wh {
                    let b = if hash_coords_3d(x, by + dy, z, seed) % 3 == 0 { BlockId::Ice } else { BlockId::Stone };
                    set_blk(blocks, x, by + dy, z, b);
                }
            }
        }
    }
    for &(dx, dz) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
        set_blk(blocks, cx + dx, by + 1, cz + dz, BlockId::Ice);
    }
    set_blk(blocks, cx, by + 1, cz, BlockId::Chest);
    set_blk(blocks, cx, by + 2, cz + 1, BlockId::Snow);
}

fn place_desert_buried_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, _seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by - 3, by, z0, z1, BlockId::Sand);
    fill_box(blocks, x0, x1, by + 1, by + 4, z0, z1, BlockId::Air);
    for x in x0..=x1 {
        for z in z0..=z1 {
            if x == x0 || x == x1 || z == z0 || z == z1 {
                place_trunk(blocks, x, by + 1, z, 4, BlockId::Sand);
            }
        }
    }
    fill_box(blocks, x0 + 1, x1 - 1, by + 4, by + 4, z0 + 1, z1 - 1, BlockId::Sand);
    set_blk(blocks, cx, by + 4, cz, BlockId::Air);
    set_blk(blocks, cx + 1, by + 4, cz, BlockId::Air);
    set_blk(blocks, cx, by - 1, cz, BlockId::Tnt);
    set_blk(blocks, cx, by, cz, BlockId::Air);
    set_blk(blocks, cx + 1, by + 1, cz, BlockId::Chest);
    set_blk(blocks, cx - 1, by + 2, cz, BlockId::Torch);
}

fn place_overgrown_temple_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by - 3, by - 1, z0, z1, BlockId::Stone);
    fill_box(blocks, x0, x1, by + 1, by + 5, z0, z1, BlockId::Air);
    for x in x0..=x1 {
        for z in z0..=z1 {
            set_blk(blocks, x, by, z, if hash_coords_3d(x, by, z, seed) % 3 == 0 { BlockId::Mud } else { BlockId::Stone });
        }
    }
    for &(x, z) in &[(x0, z0), (x0, z1), (x1, z0), (x1, z1)] {
        place_trunk(blocks, x, by + 1, z, 2, BlockId::Stone);
        place_trunk(blocks, x, by + 3, z, 2, BlockId::Leaves);
    }
    for x in x0..=x1 {
        if hash_coords_3d(x, 202, cz, seed) % 2 == 0 {
            set_blk(blocks, x, by + 4, cz, BlockId::Stone);
            set_blk(blocks, x, by + 3, cz, BlockId::Leaves);
        }
    }
    set_blk(blocks, cx, by + 1, cz, BlockId::Stone);
    set_blk(blocks, cx, by + 2, cz, BlockId::Chest);
    set_blk(blocks, cx - 1, by + 1, cz, BlockId::CraftingTable);
    set_blk(blocks, cx + 1, by + 2, cz, BlockId::Torch);
}

fn place_cave_dungeon_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    let (x0, x1, z0, z1) = (cx - 3, cx + 3, cz - 3, cz + 3);
    fill_box(blocks, x0, x1, by, by, z0, z1, BlockId::Stone);
    fill_box(blocks, x0, x1, by + 1, by + 4, z0, z1, BlockId::Air);
    fill_box(blocks, x0, x1, by + 5, by + 5, z0, z1, BlockId::Stone);

    for x in x0..=x1 {
        for z in z0..=z1 {
            if x == x0 || x == x1 || z == z0 || z == z1 {
                for dy in 1..=4 {
                    let h = hash_coords_3d(x, by + dy, z, seed);
                    let b = if h % 17 == 0 { BlockId::IronOre } else if h % 23 == 0 { BlockId::Air } else { BlockId::Stone };
                    set_blk(blocks, x, by + dy, z, b);
                }
            }
        }
    }
    set_blk(blocks, cx, by + 1, cz, BlockId::Stone);
    set_blk(blocks, cx, by + 2, cz, BlockId::Chest);
    set_blk(blocks, cx - 1, by + 1, cz, BlockId::Torch);
    set_blk(blocks, cx + 1, by + 1, cz, BlockId::Torch);
}

#[derive(Clone)]
pub struct PrimedTnt {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
}

pub struct World {
    pub name: String,
    pub gamemode: GameMode,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub seed: u32,
    pub player_position: Vec3,
    fluid_levels: HashMap<IVec3, u8>,
    pub active_sand: HashSet<IVec3>,
    pub chests: HashMap<IVec3, ChestData>,
    pub furnaces: HashMap<IVec3, FurnaceData>,
    pub saved_player: Option<SavedPlayerData>,
    pub sim_tick: u64,
    pub time_of_day: f32,
    pub torches: HashSet<IVec3>,
    pub ruststone_lights: HashMap<IVec3, (f32, f32)>,
    pub button_timers: HashMap<IVec3, f32>,
    pub active_pressure_plates: HashSet<IVec3>,
    pub ruststone_sound_events: Vec<(IVec3, bool)>,
    pub pending_explosions: Vec<Vec3>,
    pub primed_tnts: Vec<PrimedTnt>,
    pub weather: WeatherState,
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    chunk_req_tx: Option<std::sync::mpsc::Sender<(i32, i32, u32)>>,
    #[cfg(not(target_arch = "wasm32"))]
    chunk_res_rx: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Chunk>>>,
    pending_chunks: HashSet<(i32, i32)>,
    last_stream_chunk: Option<(i32, i32)>,
    last_stream_dist: Option<i32>,
}

pub struct RaycastHit {
    pub block_pos: IVec3,
    pub normal: IVec3,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn ruststone_light_params(b: BlockId) -> Option<(f32, f32)> {
    match b {
        BlockId::RustStoneOre => Some((8.0, 0.75)),
        BlockId::RustStoneBlock => Some((9.0, 0.8)),
        BlockId::RustStoneLampLit => Some((14.0, 1.0)),
        BlockId::RustStoneWireLit => Some((7.0, 0.65)),
        _ => None,
    }
}

impl World {
    pub const DAY_DURATION: f32 = 480.0;

    pub fn new() -> Self {
        Self::new_with_seed(DEFAULT_WORLD_SEED)
    }

    pub fn get_or_create_chest(&mut self, pos: IVec3) -> &mut ChestData {
        let seed = self.seed;
        let is_blacksmith = self.get_block(pos + IVec3::new(-1, 0, 0)) == BlockId::StoneSlab
            || self.get_block(pos + IVec3::new(-2, 0, 0)) == BlockId::Furnace
            || self.get_block(pos + IVec3::new(1, 0, 0)) == BlockId::Furnace;
        self.chests.entry(pos).or_insert_with(|| {
            if is_blacksmith { ChestData::generate_blacksmith_loot(seed, pos) }
            else { ChestData::generate_house_loot(seed, pos) }
        })
    }

    pub fn get_or_create_furnace(&mut self, pos: IVec3) -> &mut FurnaceData {
        self.furnaces.entry(pos).or_insert_with(FurnaceData::new)
    }

    pub fn update_furnaces(&mut self, dt: f32) {
        let state_changes: Vec<_> = self.furnaces.iter_mut().filter_map(|(&pos, f)| {
            f.tick(dt).then_some((pos, if f.is_burning() { BlockId::FurnaceLit } else { BlockId::Furnace }))
        }).collect();

        for (pos, new_block) in state_changes {
            if matches!(self.get_block(pos), BlockId::Furnace | BlockId::FurnaceLit) {
                self.set_block(pos, new_block);
            }
        }
    }

    pub fn find_spawn_position(seed: u32) -> Vec3 {
        let (mut x, mut z, mut dx, mut dz) = (0i32, 0i32, 0i32, -1i32);
        for _ in 0..1500 {
            let (wx, wz) = (x + 8, z + 8);
            let (biome, cont) = get_biome_at(wx, wz, seed);
            if !matches!(biome, Biome::DeepOcean | Biome::Ocean | Biome::Swamp | Biome::MushroomIsland) {
                let height = get_height_at(wx, wz, seed, biome, cont);
                if height >= 24 {
                    return Vec3::new(wx as f32 + 0.5, (height + 3) as f32 + 0.1, wz as f32 + 0.5);
                }
            }
            if x == z || (x < 0 && x == -z) || (x > 0 && x == 1 - z) {
                (dx, dz) = (-dz, dx);
            }
            x += dx * 16;
            z += dz * 16;
        }
        Vec3::new(8.5, 92.0, 8.5)
    }

    pub fn new_with_seed(seed: u32) -> Self {
        crate::log_info!("WORLD", "Initializing world with seed {}", seed);
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "xbox360")))]
        let (chunk_req_tx, chunk_res_rx) = {
            let (req_tx, req_rx) = std::sync::mpsc::channel::<(i32, i32, u32)>();
            let (res_tx, res_rx) = std::sync::mpsc::channel::<Chunk>();
            let req_rx = std::sync::Arc::new(std::sync::Mutex::new(req_rx));
            let num_threads = {
                #[cfg(target_os = "vita")] { 2 }
                #[cfg(not(target_os = "vita"))]
                { std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 6) }
            };
            crate::log_debug!("WORLD", "Spawned {} asynchronous chunk generation worker threads", num_threads);

            for i in 0..num_threads {
                let req_rx_clone = std::sync::Arc::clone(&req_rx);
                let res_tx_clone = res_tx.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("chunk_gen_{}", i))
                    .spawn(move || {
                        #[cfg(target_os = "vita")]
                        unsafe {
                            extern "C" {
                                fn sceKernelGetThreadId() -> i32;
                                fn sceKernelChangeThreadCpuAffinityMask(thid: i32, bitmask: i32) -> i32;
                            }
                            let mask = if i % 2 == 0 { 0x00020000 } else { 0x00040000 };
                            sceKernelChangeThreadCpuAffinityMask(sceKernelGetThreadId(), mask);
                        }
                        while let Some((cx, cz, chunk_seed)) = req_rx_clone.lock().ok().and_then(|rx| rx.recv().ok()) {
                            crate::log_debug!("CHUNK", "Worker generating chunk ({}, {}) [seed={}]", cx, cz, chunk_seed);
                            if res_tx_clone.send(Chunk::generate(cx, cz, chunk_seed)).is_err() {
                                break;
                            }
                        }
                    });
            }
            (Some(req_tx), Some(std::sync::Mutex::new(res_rx)))
        };

        #[cfg(any(target_arch = "wasm32", feature = "xbox360"))]
        let (chunk_req_tx, chunk_res_rx): (
            Option<std::sync::mpsc::Sender<(i32, i32, u32)>>,
            Option<std::sync::Mutex<std::sync::mpsc::Receiver<Chunk>>>,
        ) = (None, None);

        let spawn_pos = Self::find_spawn_position(seed);
        let (spawn_cx, spawn_cz) = ((spawn_pos.x.floor() as i32) >> 4, (spawn_pos.z.floor() as i32) >> 4);
        crate::log_info!("WORLD", "Spawn point found at ({:.1}, {:.1}, {:.1}) [chunk ({}, {})]", spawn_pos.x, spawn_pos.y, spawn_pos.z, spawn_cx, spawn_cz);

        let radius: i32 = if cfg!(any(target_os = "xbox360", target_os = "xenon", target_os = "vita", target_arch = "powerpc", feature = "xbox360")) { 1 } else { 2 };
        let total_chunks = (radius * 2 + 1) * (radius * 2 + 1);
        crate::log_info!("WORLD", "Pregenerating initial {}x{} spawn chunks around ({}, {})", radius * 2 + 1, radius * 2 + 1, spawn_cx, spawn_cz);

        let mut world = World {
            name: "New World".to_string(),
            gamemode: GameMode::Survival,
            chunks: HashMap::new(),
            seed,
            player_position: spawn_pos,
            fluid_levels: HashMap::new(),
            active_sand: HashSet::new(),
            chests: HashMap::new(),
            furnaces: HashMap::new(),
            saved_player: None,
            sim_tick: 0,
            time_of_day: 120.0,
            torches: HashSet::new(),
            ruststone_lights: HashMap::new(),
            button_timers: HashMap::new(),
            active_pressure_plates: HashSet::new(),
            ruststone_sound_events: Vec::new(),
            pending_explosions: Vec::new(),
            primed_tnts: Vec::new(),
            weather: WeatherState::new(seed),
            #[cfg(not(target_arch = "wasm32"))]
            chunk_req_tx,
            #[cfg(not(target_arch = "wasm32"))]
            chunk_res_rx,
            pending_chunks: HashSet::new(),
            last_stream_chunk: None,
            last_stream_dist: None,
        };

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let (cx, cz) = (spawn_cx + dx, spawn_cz + dz);
                let chunk = Chunk::generate(cx, cz, seed);
                world.index_chunk_lights(&chunk);
                world.chunks.insert((cx, cz), chunk);
                crate::log_info!("WORLD", "Spawn chunk ({}, {}) ready [{}/{}]", cx, cz, world.chunks.len(), total_chunks);
            }
        }
        crate::log_info!("WORLD", "Pregeneration completed ({} chunks loaded in memory)", world.chunks.len());
        world
    }

    #[inline]
    pub fn pending_chunks_count(&self) -> usize { self.pending_chunks.len() }

    #[inline]
    pub fn advance_time(&mut self, dt: f32) {
        self.time_of_day = (self.time_of_day + dt).rem_euclid(Self::DAY_DURATION);
    }

    #[inline]
    pub fn day_fraction(&self) -> f32 { self.time_of_day / Self::DAY_DURATION }

    #[inline]
    pub fn sun_angle(&self) -> f32 {
        self.day_fraction() * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2
    }

    #[inline]
    pub fn sun_direction(&self) -> Vec3 {
        let a = self.sun_angle();
        Vec3::new(a.cos(), a.sin(), 0.25).normalized()
    }

    #[inline]
    pub fn sunlight_factor(&self) -> f32 {
        let t = ((self.sun_angle().sin() + 0.2) / 0.4).clamp(0.0, 1.0);
        let ease = t * t * (3.0 - 2.0 * t);
        self.weather.modulate_daylight(0.08 + 0.92 * ease)
    }

    pub fn sky_colors(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let elev = self.sun_angle().sin();
        let day_t = ((elev + 0.25) / 0.50).clamp(0.0, 1.0);
        let day_smooth = day_t * day_t * (3.0 - 2.0 * day_t);
        let sunset_factor = (1.0 - (elev.abs() / 0.28)).max(0.0);
        let sunset_smooth = sunset_factor * sunset_factor;

        let lerp3 = |a: [f32; 3], b: [f32; 3], t: f32| [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ];

        let mut final_top = lerp3(lerp3([0.015, 0.025, 0.07], [0.32, 0.54, 0.96], day_smooth), [0.24, 0.28, 0.58], sunset_smooth * 0.7);
        let mut final_bot = lerp3(lerp3([0.05, 0.07, 0.15], [0.72, 0.84, 0.98], day_smooth), [0.98, 0.48, 0.18], sunset_smooth);
        let mut final_fog = lerp3(lerp3([0.04, 0.06, 0.12], [0.72, 0.84, 0.98], day_smooth), [0.92, 0.52, 0.25], sunset_smooth);

        if self.weather.rain_strength > 0.001 {
            let (r, t) = (self.weather.rain_strength, self.weather.thunder_strength);
            final_top = lerp3(final_top, lerp3([0.28, 0.32, 0.38], [0.14, 0.16, 0.20], t), r * 0.75);
            final_bot = lerp3(final_bot, lerp3([0.42, 0.46, 0.52], [0.22, 0.25, 0.30], t), r * 0.85);
            final_fog = lerp3(final_fog, lerp3([0.40, 0.44, 0.50], [0.20, 0.22, 0.27], t), r * 0.90);
        }

        if self.weather.lightning_flash > 0.005 {
            let f = self.weather.lightning_flash;
            let flash_col = [0.95, 0.97, 1.0];
            final_top = lerp3(final_top, flash_col, f * 0.85);
            final_bot = lerp3(final_bot, flash_col, f * 0.95);
            final_fog = lerp3(final_fog, flash_col, f * 0.95);
        }

        (final_top, final_bot, final_fog)
    }

    pub fn get_highest_block_y(&self, wx: i32, wz: i32) -> i32 {
        let (cx, cz, lx, lz) = World::world_to_chunk(wx, wz);
        if let Some(chunk) = self.chunks.get(&(cx, cz)) {
            for y in (0..CHUNK_SIZE_Y).rev() {
                if chunk.get_local(lx, y, lz) != BlockId::Air {
                    return y;
                }
            }
        }
        0
    }

    pub fn is_exposed_to_sky(&self, pos: IVec3) -> bool {
        self.get_highest_block_y(pos.x, pos.z) <= pos.y
    }

    #[inline]
    pub fn get_biome_at(&self, wx: i32, wz: i32) -> Biome {
        get_biome_at(wx, wz, self.seed).0
    }

    pub fn update_weather(&mut self, dt: f32, player_pos: Vec3) -> Option<LightningStrike> {
        let default_y = player_pos.y.max(60.0);
        let chunk_map = &self.chunks;
        self.weather.update(dt, player_pos, |x, z| {
            let (cx, cz, lx, lz) = World::world_to_chunk(x.floor() as i32, z.floor() as i32);
            if let Some(chunk) = chunk_map.get(&(cx, cz)) {
                for y in (0..CHUNK_SIZE_Y).rev() {
                    if chunk.get_local(lx, y, lz) != BlockId::Air {
                        return y as f32 + 1.0;
                    }
                }
            }
            default_y
        })
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        crate::log_info!("SAVE", "Serializing world '{}' (seed: {}, mode: {:?}, chunks: {}, chests: {}, furnaces: {}) to '{}'", self.name, self.seed, self.gamemode, self.chunks.len(), self.chests.len(), self.furnaces.len(), path);
        let mut bytes = Vec::with_capacity(32 + self.chunks.len() * 4);
        bytes.extend_from_slice(b"RUSTCRAFT6");
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        bytes.push(match self.gamemode { GameMode::Survival => 0, GameMode::Creative => 1 });

        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(255) as u8;
        bytes.push(name_len);
        bytes.extend_from_slice(&name_bytes[..name_len as usize]);

        for f in [self.player_position.x, self.player_position.y, self.player_position.z] {
            bytes.extend_from_slice(&f.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());
        for chunk in self.chunks.values() {
            bytes.extend_from_slice(&chunk.cx.to_le_bytes());
            bytes.extend_from_slice(&chunk.cz.to_le_bytes());
            bytes.extend_from_slice(&(chunk.blocks.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&chunk.blocks);
        }

        let push_pos = |b: &mut Vec<u8>, pos: &IVec3| {
            b.extend_from_slice(&pos.x.to_le_bytes());
            b.extend_from_slice(&pos.y.to_le_bytes());
            b.extend_from_slice(&pos.z.to_le_bytes());
        };
        let push_item = |b: &mut Vec<u8>, item: &Option<ItemStack>| {
            if let Some(s) = item { b.extend_from_slice(&[s.block.to_u8(), s.count]); }
            else { b.extend_from_slice(&[0, 0]); }
        };

        bytes.extend_from_slice(&(self.fluid_levels.len() as u32).to_le_bytes());
        for (pos, &level) in &self.fluid_levels {
            push_pos(&mut bytes, pos);
            bytes.push(level);
        }

        bytes.extend_from_slice(&(self.chests.len() as u32).to_le_bytes());
        for (pos, chest) in &self.chests {
            push_pos(&mut bytes, pos);
            for item in &chest.items { push_item(&mut bytes, item); }
        }

        bytes.extend_from_slice(&(self.furnaces.len() as u32).to_le_bytes());
        for (pos, furnace) in &self.furnaces {
            push_pos(&mut bytes, pos);
            for s in [&furnace.input, &furnace.fuel, &furnace.output] { push_item(&mut bytes, s); }
            for f in [furnace.burn_time_remaining, furnace.burn_time_total, furnace.cook_progress] {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
        }

        if let Some(ref p) = self.saved_player {
            bytes.push(1);
            for f in [p.health, p.max_health, p.hunger, p.max_hunger, p.saturation, p.exhaustion, p.air] {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            bytes.push(p.selected_hotbar_slot);
            for item in p.hotbar.iter().chain(p.main.iter()).chain(p.armor.iter()).chain(std::iter::once(&p.offhand)) {
                push_item(&mut bytes, item);
            }
        } else {
            bytes.push(0);
        }

        std::fs::write(path, bytes).map_err(|e| format!("failed to save world: {e}"))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        crate::log_info!("SAVE", "Loading world from file '{}'...", path);
        let bytes = std::fs::read(path).map_err(|e| format!("failed to read world file: {e}"))?;
        let mut cursor = 0usize;
        let format = read_bytes(&bytes, &mut cursor, 10)?;
        let version = match format {
            b"RUSTCRAFT1" => 1, b"RUSTCRAFT2" => 2, b"RUSTCRAFT3" => 3,
            b"RUSTCRAFT4" => 4, b"RUSTCRAFT5" => 5, b"RUSTCRAFT6" => 6,
            _ => return Err("unknown world file format".to_owned()),
        };

        let read_u32 = |c: &mut usize| -> Result<u32, String> { Ok(u32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_i32 = |c: &mut usize| -> Result<i32, String> { Ok(i32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_f32 = |c: &mut usize| -> Result<f32, String> { Ok(f32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_item = |c: &mut usize| -> Option<ItemStack> {
            read_bytes(&bytes, c, 2).ok().and_then(|ib| (ib[0] > 0 && ib[1] > 0).then(|| ItemStack::new(BlockId::from_u8(ib[0]), ib[1])))
        };

        let seed = read_u32(&mut cursor)?;
        let (gamemode, name) = if version >= 4 {
            let gm = if read_bytes(&bytes, &mut cursor, 1)?[0] == 1 { GameMode::Creative } else { GameMode::Survival };
            let name_len = read_bytes(&bytes, &mut cursor, 1)?[0] as usize;
            let name_str = if name_len > 0 { String::from_utf8_lossy(read_bytes(&bytes, &mut cursor, name_len)?).to_string() } else { format!("World {}", seed) };
            (gm, name_str)
        } else {
            (GameMode::Survival, format!("World {}", seed))
        };

        let player_position = if version >= 2 {
            Vec3::new(read_f32(&mut cursor)?, read_f32(&mut cursor)?, read_f32(&mut cursor)?)
        } else {
            Self::find_spawn_position(seed)
        };

        let chunk_count = read_u32(&mut cursor)? as usize;
        let expected_len = (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize;
        let mut world = Self::new_with_seed(seed);
        world.name = name;
        world.gamemode = gamemode;
        world.player_position = player_position;

        for _ in 0..chunk_count {
            let (cx, cz, block_len) = (read_i32(&mut cursor)?, read_i32(&mut cursor)?, read_u32(&mut cursor)? as usize);
            if block_len != expected_len {
                return Err("invalid chunk data length in world file".to_owned());
            }
            let blocks = read_bytes(&bytes, &mut cursor, block_len)?.to_vec();
            world.chunks.insert((cx, cz), Chunk { cx, cz, blocks, dirty: true });
        }

        if version >= 3 {
            let fluid_count = read_u32(&mut cursor)? as usize;
            for _ in 0..fluid_count {
                let pos = IVec3::new(read_i32(&mut cursor)?, read_i32(&mut cursor)?, read_i32(&mut cursor)?);
                let level = read_bytes(&bytes, &mut cursor, 1)?[0].clamp(1, 8);
                world.fluid_levels.insert(pos, level);
            }
        }

        if version >= 5 && bytes.len() > cursor {
            if let Ok(chest_count) = read_u32(&mut cursor) {
                for _ in 0..chest_count {
                    if let (Ok(x), Ok(y), Ok(z)) = (read_i32(&mut cursor), read_i32(&mut cursor), read_i32(&mut cursor)) {
                        let mut chest = ChestData::new();
                        for slot in &mut chest.items { *slot = read_item(&mut cursor); }
                        world.chests.insert(IVec3::new(x, y, z), chest);
                    }
                }
            }
            if let Ok(furnace_count) = read_u32(&mut cursor) {
                for _ in 0..furnace_count {
                    if let (Ok(x), Ok(y), Ok(z)) = (read_i32(&mut cursor), read_i32(&mut cursor), read_i32(&mut cursor)) {
                        let mut furnace = FurnaceData::new();
                        for slot in [&mut furnace.input, &mut furnace.fuel, &mut furnace.output] { *slot = read_item(&mut cursor); }
                        if let (Ok(rem), Ok(tot), Ok(prog)) = (read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor)) {
                            furnace.burn_time_remaining = rem;
                            furnace.burn_time_total = tot;
                            furnace.cook_progress = prog;
                        }
                        world.furnaces.insert(IVec3::new(x, y, z), furnace);
                    }
                }
            }
        }

        if version >= 6 && bytes.len() > cursor {
            if let Ok(has_p) = read_bytes(&bytes, &mut cursor, 1) {
                if has_p[0] == 1 {
                    if let (Ok(health), Ok(max_health), Ok(hunger), Ok(max_hunger), Ok(saturation), Ok(exhaustion), Ok(air), Ok(slot)) = (
                        read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor),
                        read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor), read_bytes(&bytes, &mut cursor, 1)
                    ) {
                        let mut hotbar = [None; 10];
                        for s in &mut hotbar { *s = read_item(&mut cursor); }
                        let mut main = [None; 27];
                        for s in &mut main { *s = read_item(&mut cursor); }
                        let mut armor = [None; 4];
                        for s in &mut armor { *s = read_item(&mut cursor); }
                        let offhand = read_item(&mut cursor);

                        world.saved_player = Some(SavedPlayerData {
                            health, max_health, hunger, max_hunger, saturation, exhaustion, air,
                            selected_hotbar_slot: slot[0], hotbar, main, armor, offhand,
                        });
                    }
                }
            }
        }
        crate::log_info!("SAVE", "Successfully loaded world '{}' (seed: {}, mode: {:?}, chunks: {}, chests: {}, furnaces: {})", world.name, world.seed, world.gamemode, world.chunks.len(), world.chests.len(), world.furnaces.len());
        Ok(world)
    }

    #[inline(always)]
    pub fn world_to_chunk(wx: i32, wz: i32) -> (i32, i32, i32, i32) {
        (wx >> 4, wz >> 4, wx & 15, wz & 15)
    }

    #[inline(always)]
    pub fn get_block(&self, pos: IVec3) -> BlockId {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        self.chunks.get(&(cx, cz)).map_or(BlockId::Air, |c| c.get_local(lx, pos.y, lz))
    }

    pub fn is_leaf_supported(&self, start: IVec3, max_dist: i32) -> bool {
        let mut visited = HashSet::from([start]);
        let mut queue = VecDeque::from([(start, 0)]);

        while let Some((pos, dist)) = queue.pop_front() {
            if self.get_block(pos).is_wood() {
                return true;
            }
            if dist < max_dist {
                for offset in CARDINAL_OFFSETS {
                    let neighbor = pos + offset;
                    if visited.insert(neighbor) {
                        let n_block = self.get_block(neighbor);
                        if n_block.is_wood() { return true; }
                        if n_block.is_leaves() { queue.push_back((neighbor, dist + 1)); }
                    }
                }
            }
        }
        false
    }

    pub fn find_unsupported_leaves_around(&self, center: IVec3) -> Vec<IVec3> {
        let mut unsupported = Vec::new();
        for dx in -5..=5 {
            for dz in -5..=5 {
                for dy in -5..=10 {
                    let leaf_pos = center + IVec3::new(dx, dy, dz);
                    if self.get_block(leaf_pos).is_leaves() && !self.is_leaf_supported(leaf_pos, 5) {
                        unsupported.push(leaf_pos);
                    }
                }
            }
        }
        unsupported
    }

    pub fn biome_name_at(&self, wx: i32, wz: i32) -> &'static str {
        get_biome_at(wx, wz, self.seed).0.name()
    }

    #[inline]
    pub fn is_view_fully_occluded(&self, camera_pos: Vec3, yaw: f32, pitch: f32) -> bool {
        if self.get_block(camera_pos.floor_to_ivec()).is_opaque() {
            return true;
        }
        let (sin_y, cos_y) = yaw.sin_cos();
        let (sin_p, cos_p) = pitch.sin_cos();
        let forward = Vec3::new(sin_y * cos_p, -sin_p, cos_y * cos_p);
        let right = Vec3::new(cos_y, 0.0, -sin_y) * 0.26;
        let up = Vec3::new(sin_y * sin_p, cos_p, cos_y * sin_p) * 0.20;
        let c = camera_pos + forward * 0.38;

        [c, c - right + up, c + right + up, c - right - up, c + right - up]
            .into_iter()
            .all(|p| self.get_block(p.floor_to_ivec()).is_opaque())
    }

    #[inline(always)]
    pub fn fluid_level(&self, pos: IVec3) -> u8 {
        self.fluid_levels.get(&pos).copied().unwrap_or(8)
    }

    fn set_fluid_level(&mut self, pos: IVec3, level: u8) {
        self.fluid_levels.insert(pos, level.clamp(1, 8));
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockId) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            chunk.set_local(lx, pos.y, lz, block);

            if matches!(block, BlockId::Water | BlockId::Lava) { self.set_fluid_level(pos, 8); } else { self.fluid_levels.remove(&pos); }
            if block == BlockId::Sand { self.active_sand.insert(pos); } else { self.active_sand.remove(&pos); }
            if block == BlockId::Torch { self.torches.insert(pos); } else { self.torches.remove(&pos); }
            if let Some(params) = ruststone_light_params(block) { self.ruststone_lights.insert(pos, params); } else { self.ruststone_lights.remove(&pos); }
            if block == BlockId::PressurePlatePressed { self.active_pressure_plates.insert(pos); } else { self.active_pressure_plates.remove(&pos); }
            if block == BlockId::Chest { self.chests.entry(pos).or_insert_with(ChestData::new); } else { self.chests.remove(&pos); }
            if matches!(block, BlockId::Furnace | BlockId::FurnaceLit) { self.furnaces.entry(pos).or_insert_with(FurnaceData::new); } else { self.furnaces.remove(&pos); }

            for offset in CARDINAL_OFFSETS {
                let neighbor = pos + offset;
                if self.get_block(neighbor) == BlockId::Sand {
                    self.active_sand.insert(neighbor);
                }
            }

            if lx == 0 { self.mark_dirty(cx - 1, cz); }
            if lx == CHUNK_SIZE_X - 1 { self.mark_dirty(cx + 1, cz); }
            if lz == 0 { self.mark_dirty(cx, cz - 1); }
            if lz == CHUNK_SIZE_Z - 1 { self.mark_dirty(cx, cz + 1); }
        }
    }

    pub fn prime_tnt(&mut self, pos: IVec3) {
        if self.get_block(pos) == BlockId::Tnt {
            self.set_block(pos, BlockId::Air);
            self.primed_tnts.push(PrimedTnt {
                pos: Vec3::new(pos.x as f32 + 0.5, pos.y as f32, pos.z as f32 + 0.5),
                velocity: Vec3::new(0.0, 2.0, 0.0),
                fuse: 3.5,
            });
        }
    }

    pub fn index_chunk_lights(&mut self, chunk: &Chunk) {
        let (base_x, base_z) = (chunk.cx * CHUNK_SIZE_X, chunk.cz * CHUNK_SIZE_Z);
        for ly in 0..CHUNK_SIZE_Y {
            for lz in 0..CHUNK_SIZE_Z {
                for lx in 0..CHUNK_SIZE_X {
                    let b = chunk.get_local(lx, ly, lz);
                    if b == BlockId::Torch {
                        self.torches.insert(IVec3::new(base_x + lx, ly, base_z + lz));
                    } else if let Some(params) = ruststone_light_params(b) {
                        self.ruststone_lights.insert(IVec3::new(base_x + lx, ly, base_z + lz), params);
                    }
                }
            }
        }
    }

    pub fn update_ruststone_network(&mut self, center: IVec3, range: i32) {
        let mut sources = Vec::new();
        let mut wires = HashSet::new();
        let mut lamps = HashSet::new();
        let mut doors = HashSet::new();
        let mut tnts = Vec::new();

        for dy in -range..=range {
            for dz in -range..=range {
                for dx in -range..=range {
                    let pos = center + IVec3::new(dx, dy, dz);
                    match self.get_block(pos) {
                        BlockId::RustStoneBlock | BlockId::LeverOn | BlockId::ButtonPressed | BlockId::PressurePlatePressed => sources.push(pos),
                        BlockId::RustStone | BlockId::RustStoneWireLit => { wires.insert(pos); }
                        BlockId::RustStoneLamp | BlockId::RustStoneLampLit => { lamps.insert(pos); }
                        BlockId::Door | BlockId::DoorOpen => { doors.insert(pos); }
                        BlockId::Tnt => tnts.push(pos),
                        _ => {}
                    }
                }
            }
        }

        let mut wire_power: HashMap<IVec3, u8> = HashMap::new();
        let mut powered_lamps = HashSet::new();
        let mut powered_doors = HashSet::new();
        let mut tnts_to_ignite = Vec::new();
        let mut queue = VecDeque::new();

        for s_pos in sources {
            for &n_off in &CARDINAL_OFFSETS {
                let n_pos = s_pos + n_off;
                if wires.contains(&n_pos) && wire_power.get(&n_pos).copied().unwrap_or(0) < 15 {
                    wire_power.insert(n_pos, 15);
                    queue.push_back((n_pos, 15));
                }
                if lamps.contains(&n_pos) { powered_lamps.insert(n_pos); }
                if doors.contains(&n_pos) { powered_doors.insert(n_pos); }
                if tnts.contains(&n_pos) { tnts_to_ignite.push(n_pos); }
            }
        }

        while let Some((w_pos, pwr)) = queue.pop_front() {
            for &n_off in &CARDINAL_OFFSETS {
                let n_pos = w_pos + n_off;
                if lamps.contains(&n_pos) { powered_lamps.insert(n_pos); }
                if doors.contains(&n_pos) { powered_doors.insert(n_pos); }
                if tnts.contains(&n_pos) { tnts_to_ignite.push(n_pos); }
                if pwr > 1 && wires.contains(&n_pos) {
                    let next_pwr = pwr - 1;
                    if wire_power.get(&n_pos).copied().unwrap_or(0) < next_pwr {
                        wire_power.insert(n_pos, next_pwr);
                        queue.push_back((n_pos, next_pwr));
                    }
                }
            }
        }

        for tnt_pos in tnts_to_ignite { self.prime_tnt(tnt_pos); }

        for w_pos in wires {
            let pwr = wire_power.get(&w_pos).copied().unwrap_or(0);
            let cur = self.get_block(w_pos);
            if pwr > 0 && cur == BlockId::RustStone { self.set_block(w_pos, BlockId::RustStoneWireLit); }
            else if pwr == 0 && cur == BlockId::RustStoneWireLit { self.set_block(w_pos, BlockId::RustStone); }
        }

        for l_pos in lamps {
            let is_pw = powered_lamps.contains(&l_pos);
            let cur = self.get_block(l_pos);
            if is_pw && cur == BlockId::RustStoneLamp { self.set_block(l_pos, BlockId::RustStoneLampLit); }
            else if !is_pw && cur == BlockId::RustStoneLampLit { self.set_block(l_pos, BlockId::RustStoneLamp); }
        }

        for d_pos in doors {
            let is_pw = powered_doors.contains(&d_pos);
            let cur = self.get_block(d_pos);
            if is_pw && cur == BlockId::Door {
                self.set_block(d_pos, BlockId::DoorOpen);
                self.ruststone_sound_events.push((d_pos, true));
            } else if !is_pw && cur == BlockId::DoorOpen {
                self.set_block(d_pos, BlockId::Door);
                self.ruststone_sound_events.push((d_pos, false));
            }
        }
    }

    pub fn update_ruststone(&mut self, dt: f32, player_pos: Vec3, mob_positions: &[Vec3]) {
        let mut expired = Vec::new();
        for (&pos, timer) in &mut self.button_timers {
            *timer -= dt;
            if *timer <= 0.0 { expired.push(pos); }
        }
        for pos in expired {
            self.button_timers.remove(&pos);
            if self.get_block(pos) == BlockId::ButtonPressed {
                self.set_block(pos, BlockId::Button);
                self.ruststone_sound_events.push((pos, false));
                self.update_ruststone_network(pos, 16);
            }
        }

        let (px, py, pz) = (player_pos.x.floor() as i32, player_pos.y.floor() as i32, player_pos.z.floor() as i32);
        let is_entity_on_plate = |pos: IVec3, ent: Vec3| -> bool {
            ent.x.floor() as i32 == pos.x && ent.z.floor() as i32 == pos.z
                && (ent.y.floor() as i32 == pos.y || (ent.y - 0.2).floor() as i32 == pos.y)
        };
        let any_on = |pos: IVec3| is_entity_on_plate(pos, player_pos) || mob_positions.iter().any(|&m| is_entity_on_plate(pos, m));

        let mut changed_plates = Vec::new();
        for &plate_pos in &self.active_pressure_plates {
            if !any_on(plate_pos) && self.get_block(plate_pos) == BlockId::PressurePlatePressed {
                changed_plates.push((plate_pos, BlockId::PressurePlate, false));
            }
        }

        for dy in -3..=3 {
            for dz in -12..=12 {
                for dx in -12..=12 {
                    let test_pos = IVec3::new(px + dx, py + dy, pz + dz);
                    if self.get_block(test_pos) == BlockId::PressurePlate && any_on(test_pos) {
                        changed_plates.push((test_pos, BlockId::PressurePlatePressed, true));
                    }
                }
            }
        }

        for (pos, new_block, is_pressed) in changed_plates {
            self.set_block(pos, new_block);
            self.ruststone_sound_events.push((pos, is_pressed));
            self.update_ruststone_network(pos, 16);
        }
    }

    pub fn update_primed_tnts(&mut self, dt: f32) {
        let mut exploded_positions = Vec::new();
        for i in 0..self.primed_tnts.len() {
            self.primed_tnts[i].fuse -= dt;
            self.primed_tnts[i].velocity.y -= 20.0 * dt;
            let mut new_pos = self.primed_tnts[i].pos;
            new_pos.y += self.primed_tnts[i].velocity.y * dt;

            if self.box_collides(Vec3::new(new_pos.x - 0.49, new_pos.y, new_pos.z - 0.49), Vec3::new(new_pos.x + 0.49, new_pos.y + 0.99, new_pos.z + 0.49)) {
                self.primed_tnts[i].velocity = Vec3::zero();
                self.primed_tnts[i].pos.y = self.primed_tnts[i].pos.y.floor();
            } else {
                self.primed_tnts[i].pos = new_pos;
            }

            if self.primed_tnts[i].fuse <= 0.0 {
                exploded_positions.push(self.primed_tnts[i].pos.floor_to_ivec());
            }
        }
        self.primed_tnts.retain(|tnt| tnt.fuse > 0.0);
        for center in exploded_positions {
            self.explode(center, 3.5);
        }
    }

    pub fn grow_tree_at(&mut self, pos: IVec3) -> bool {
        let trunk_height = 4 + (pos.x.wrapping_mul(7) ^ pos.z.wrapping_mul(13)).rem_euclid(2);
        for i in 0..trunk_height {
            self.set_block(IVec3::new(pos.x, pos.y + i, pos.z), BlockId::Wood);
        }
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..=2 {
                    if dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2 <= 6 {
                        let p = IVec3::new(pos.x + dx, pos.y + trunk_height - 1 + dy, pos.z + dz);
                        if self.get_block(p) == BlockId::Air {
                            self.set_block(p, BlockId::Leaves);
                        }
                    }
                }
            }
        }
        true
    }

    pub fn tick_farming(&mut self, player_pos: Vec3, rng_seed: &mut u32) {
        let (pcx, pcz) = ((player_pos.x.floor() as i32) >> 4, (player_pos.z.floor() as i32) >> 4);
        let mut next_rand = || -> u32 {
            *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *rng_seed
        };

        for _ in 0..4 {
            let cx = pcx + (next_rand() % 5) as i32 - 2;
            let cz = pcz + (next_rand() % 5) as i32 - 2;
            if self.chunks.contains_key(&(cx, cz)) {
                for _ in 0..3 {
                    let pos = IVec3::new(cx * 16 + (next_rand() % 16) as i32, 20 + (next_rand() % 35) as i32, cz * 16 + (next_rand() % 16) as i32);
                    match self.get_block(pos) {
                        BlockId::Farmland => {
                            let mut has_water = false;
                            'water_search: for wx2 in -4..=4 {
                                for wz2 in -4..=4 {
                                    for wy2 in 0..=1 {
                                        if self.get_block(pos + IVec3::new(wx2, wy2, wz2)) == BlockId::Water {
                                            has_water = true;
                                            break 'water_search;
                                        }
                                    }
                                }
                            }
                            if has_water {
                                self.set_block(pos, BlockId::FarmlandWet);
                            } else if !matches!(self.get_block(pos + IVec3::new(0, 1, 0)), BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop) && next_rand() % 6 == 0 {
                                self.set_block(pos, BlockId::Dirt);
                            }
                        }
                        BlockId::Sapling if next_rand() % 35 == 0 => {
                            self.set_block(pos, BlockId::Air);
                            self.grow_tree_at(pos);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn explode(&mut self, center: IVec3, radius: f32) {
        let r_int = radius.ceil() as i32;
        let r_sq = radius * radius;
        let mut tnts_to_prime = Vec::new();

        for dx in -r_int..=r_int {
            for dy in -r_int..=r_int {
                for dz in -r_int..=r_int {
                    if (dx * dx + dy * dy + dz * dz) as f32 <= r_sq {
                        let target_pos = center + IVec3::new(dx, dy, dz);
                        let b = self.get_block(target_pos);
                        if b != BlockId::Air && !b.is_bedrock() {
                            if b == BlockId::Tnt { tnts_to_prime.push(target_pos); }
                            else { self.set_block(target_pos, BlockId::Air); }
                        }
                    }
                }
            }
        }
        self.pending_explosions.push(Vec3::new(center.x as f32 + 0.5, center.y as f32 + 0.5, center.z as f32 + 0.5));
        for tnt_pos in tnts_to_prime { self.prime_tnt(tnt_pos); }
    }

    pub fn create_mesh_job(&self, key: (i32, i32)) -> Option<ChunkMeshJob> {
        let chunk = self.chunks.get(&key)?;
        let (cx, cz) = key;
        let (center_x, center_z) = (cx * CHUNK_SIZE_X + 8, cz * CHUNK_SIZE_Z + 8);

        let mut lights = Vec::new();
        for &pos in &self.torches {
            if (pos.x - center_x).abs() <= 24 && (pos.z - center_z).abs() <= 24 {
                lights.push(LightSource { pos, radius: 14.0, intensity: 1.0 });
            }
        }
        for (&pos, &(radius, intensity)) in &self.ruststone_lights {
            if (pos.x - center_x).abs() <= 24 && (pos.z - center_z).abs() <= 24 {
                lights.push(LightSource { pos, radius, intensity });
            }
        }

        let get_nb = |dx: i32, dz: i32| self.chunks.get(&(cx + dx, cz + dz)).map(|c| c.blocks.clone());

        Some(ChunkMeshJob {
            key,
            center_blocks: chunk.blocks.clone(),
            east_blocks: get_nb(1, 0),
            west_blocks: get_nb(-1, 0),
            south_blocks: get_nb(0, 1),
            north_blocks: get_nb(0, -1),
            se_blocks: get_nb(1, 1),
            sw_blocks: get_nb(-1, 1),
            ne_blocks: get_nb(1, -1),
            nw_blocks: get_nb(-1, -1),
            seed: self.seed,
            lights,
            fluid_map: self.fluid_levels.clone(),
        })
    }

    fn mark_dirty(&mut self, cx: i32, cz: i32) {
        if let Some(c) = self.chunks.get_mut(&(cx, cz)) { c.dirty = true; }
    }

    fn mark_neighbors_dirty(&mut self, cx: i32, cz: i32) {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.mark_dirty(cx + dx, cz + dz);
        }
    }

    pub fn update(&mut self, player_pos: Vec3, view_distance: i32) {
        let view_distance = view_distance.clamp(2, 16);
        let (pcx, pcz) = ((player_pos.x.floor() as i32) >> 4, (player_pos.z.floor() as i32) >> 4);
        let unload_dist_sq = (view_distance + 3) * (view_distance + 3);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut received_chunks = Vec::new();
            if let Some(rx_mutex) = &self.chunk_res_rx {
                if let Ok(rx) = rx_mutex.lock() {
                    while let Ok(chunk) = rx.try_recv() { received_chunks.push(chunk); }
                }
            }
            for chunk in received_chunks {
                let (cx, cz) = (chunk.cx, chunk.cz);
                self.pending_chunks.remove(&(cx, cz));
                if (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq {
                    self.index_chunk_lights(&chunk);
                    self.chunks.insert((cx, cz), chunk);
                    crate::log_debug!("CHUNK", "Integrated chunk ({}, {}) into active world (loaded: {})", cx, cz, self.chunks.len());
                    self.mark_neighbors_dirty(cx, cz);
                }
            }
        }

        if self.last_stream_chunk == Some((pcx, pcz)) && self.last_stream_dist == Some(view_distance) && self.pending_chunks.is_empty() {
            return;
        }

        let mut missing_chunks = Vec::new();
        let view_dist_sq = view_distance * view_distance;
        for dz in -view_distance..=view_distance {
            for dx in -view_distance..=view_distance {
                if dx * dx + dz * dz <= view_dist_sq {
                    let key = (pcx + dx, pcz + dz);
                    if !self.chunks.contains_key(&key) && !self.pending_chunks.contains(&key) {
                        missing_chunks.push(key);
                    }
                }
            }
        }

        if !missing_chunks.is_empty() {
            crate::log_debug!("CHUNK", "Requesting generation for {} missing chunks around player chunk ({}, {}) [view_dist={}]", missing_chunks.len(), pcx, pcz, view_distance);
            missing_chunks.sort_unstable_by_key(|&(cx, cz)| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz));

            #[cfg(all(not(target_arch = "wasm32"), not(feature = "xbox360")))]
            if let Some(tx) = &self.chunk_req_tx {
                for (cx, cz) in missing_chunks {
                    self.pending_chunks.insert((cx, cz));
                    let _ = tx.send((cx, cz, self.seed));
                }
            }

            #[cfg(any(target_arch = "wasm32", feature = "xbox360"))]
            for (cx, cz) in missing_chunks.into_iter().take(1) {
                let chunk = Chunk::generate(cx, cz, self.seed);
                self.index_chunk_lights(&chunk);
                self.chunks.insert((cx, cz), chunk);
                self.mark_neighbors_dirty(cx, cz);
            }
        }

        self.last_stream_chunk = self.pending_chunks.is_empty().then_some((pcx, pcz));
        if self.pending_chunks.is_empty() { self.last_stream_dist = Some(view_distance); }

        let count_before = self.chunks.len();
        self.chunks.retain(|&(cx, cz), _| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        self.pending_chunks.retain(|&(cx, cz)| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        if self.chunks.len() < count_before {
            crate::log_debug!("CHUNK", "Unloaded {} distant chunks (remaining active: {})", count_before - self.chunks.len(), self.chunks.len());
        }
    }

    pub fn mark_all_chunks_dirty(&mut self) {
        for chunk in self.chunks.values_mut() { chunk.dirty = true; }
        crate::log_info!("WORLD", "Marked all {} chunks dirty for mesh regeneration", self.chunks.len());
    }

    pub fn purge_distant_chunks(&mut self, player_pos: Vec3, view_distance: i32) {
        let view_distance = view_distance.clamp(2, 16);
        let (pcx, pcz) = ((player_pos.x.floor() as i32) >> 4, (player_pos.z.floor() as i32) >> 4);
        let unload_dist_sq = (view_distance + 2) * (view_distance + 2);
        let before_count = self.chunks.len();

        self.chunks.retain(|&(cx, cz), _| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        self.pending_chunks.retain(|&(cx, cz)| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        for chunk in self.chunks.values_mut() { chunk.dirty = true; }

        self.last_stream_chunk = None;
        self.last_stream_dist = None;
        crate::log_info!("CHUNK", "Purged distant chunks: {} -> {} active (view distance: {} chunks)", before_count, self.chunks.len(), view_distance);
    }

    #[inline]
    fn get_block_cached<'a>(&'a self, pos: IVec3, hint: Option<((i32, i32), &'a Chunk)>) -> (BlockId, Option<((i32, i32), &'a Chunk)>) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 { return (BlockId::Air, hint); }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some((key, chunk)) = hint {
            if key == (cx, cz) { return (chunk.get_local(lx, pos.y, lz), Some((key, chunk))); }
        }
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => (chunk.get_local(lx, pos.y, lz), Some(((cx, cz), chunk))),
            None => (BlockId::Air, hint),
        }
    }

    pub fn simulate_materials(&mut self, player_pos: Vec3) {
        self.sim_tick = self.sim_tick.wrapping_add(1);
        let (px, py, pz) = (player_pos.x.floor() as i32, player_pos.y.floor() as i32, player_pos.z.floor() as i32);
        let (min_y, max_y) = ((py - 12).max(1), (py + 8).min(CHUNK_SIZE_Y - 1));
        let sim_lava = self.sim_tick % 3 == 0;

        let mut fluid_spawns = Vec::new();
        let mut fluid_despawns = Vec::new();
        let mut solidifications = Vec::new();
        let mut tnts_to_prime = Vec::new();
        let mut hint: Option<((i32, i32), &Chunk)> = None;

        for y in (min_y..=max_y).rev() {
            for z in (pz - 10)..=(pz + 10) {
                for x in (px - 10)..=(px + 10) {
                    let pos = IVec3::new(x, y, z);
                    let (block, new_hint) = self.get_block_cached(pos, hint);
                    hint = new_hint;

                    if block == BlockId::Tnt {
                        for off in CARDINAL_OFFSETS {
                            let (nblock, nh) = self.get_block_cached(pos + off, hint);
                            hint = nh;
                            if nblock == BlockId::Lava {
                                tnts_to_prime.push(pos);
                                break;
                            }
                        }
                        continue;
                    }

                    if !matches!(block, BlockId::Water | BlockId::Lava) || (block == BlockId::Lava && !sim_lava) {
                        continue;
                    }

                    let level = self.fluid_level(pos);
                    let below = IVec3::new(x, y - 1, z);
                    let below_block = self.get_block(below);
                    let cardinals = [
                        IVec3::new(x + 1, y, z), IVec3::new(x - 1, y, z),
                        IVec3::new(x, y, z + 1), IVec3::new(x, y, z - 1),
                        IVec3::new(x, y + 1, z),
                    ];

                    if block == BlockId::Lava && (below_block == BlockId::Water || cardinals.iter().any(|&p| self.get_block(p) == BlockId::Water)) {
                        solidifications.push((pos, BlockId::Stone));
                        continue;
                    }

                    if below.y >= 0 && below_block == BlockId::Air {
                        fluid_spawns.push((below, block, 8));
                    } else if block == BlockId::Water && below_block == BlockId::Lava {
                        solidifications.push((below, BlockId::Stone));
                    } else {
                        let step = if block == BlockId::Lava { 4 } else { 2 };
                        if level > step {
                            for &npos in &cardinals[0..4] {
                                let nblock = self.get_block(npos);
                                if nblock == BlockId::Air { fluid_spawns.push((npos, block, level - step)); }
                                else if block == BlockId::Water && nblock == BlockId::Lava { solidifications.push((npos, BlockId::Stone)); }
                            }
                        }
                    }

                    if level < 8 && self.get_block(cardinals[4]) != block && !cardinals[0..4].iter().any(|&p| self.get_block(p) == block && self.fluid_level(p) > level) {
                        fluid_despawns.push(pos);
                    }
                }
            }
        }

        for tnt_pos in tnts_to_prime { self.prime_tnt(tnt_pos); }
        for (pos, stone) in solidifications { self.set_block(pos, stone); }
        for (pos, block, level) in fluid_spawns {
            if self.get_block(pos) == BlockId::Air {
                self.set_block(pos, block);
                self.set_fluid_level(pos, level);
            }
        }
        for pos in fluid_despawns {
            if matches!(self.get_block(pos), BlockId::Water | BlockId::Lava) && self.fluid_level(pos) < 8 {
                self.set_block(pos, BlockId::Air);
            }
        }

        if !self.active_sand.is_empty() {
            let mut active_list: Vec<IVec3> = self.active_sand.iter().copied().collect();
            active_list.sort_unstable_by_key(|p| p.y);
            let mut sand_moves = Vec::new();

            for pos in active_list {
                let below = IVec3::new(pos.x, pos.y - 1, pos.z);
                if self.get_block(pos) == BlockId::Sand && below.y >= 0 && self.get_block(below) == BlockId::Air {
                    sand_moves.push((pos, below));
                } else {
                    self.active_sand.remove(&pos);
                }
            }
            for (from, to) in sand_moves {
                if self.get_block(from) == BlockId::Sand && self.get_block(to) == BlockId::Air {
                    self.set_block(from, BlockId::Air);
                    self.set_block(to, BlockId::Sand);
                }
            }
        }
    }

    fn traverse_voxels<T>(origin: Vec3, dir: Vec3, max_dist: f32, mut visitor: impl FnMut(IVec3, IVec3, f32) -> Option<T>) -> Option<T> {
        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let next_b = |p: f32, s: i32| if s > 0 { p.floor() + 1.0 - p } else { p - p.floor() };
        let mut t_max_x = if dir.x.abs() > 1e-8 { next_b(origin.x, step_x) * t_delta_x } else { f32::INFINITY };
        let mut t_max_y = if dir.y.abs() > 1e-8 { next_b(origin.y, step_y) * t_delta_y } else { f32::INFINITY };
        let mut t_max_z = if dir.z.abs() > 1e-8 { next_b(origin.z, step_z) * t_delta_z } else { f32::INFINITY };

        let mut last_normal = IVec3::new(0, 0, 0);
        let mut traveled = 0.0f32;

        while traveled <= max_dist {
            if let Some(res) = visitor(IVec3::new(x, y, z), last_normal, traveled) {
                return Some(res);
            }
            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x; traveled = t_max_x; t_max_x += t_delta_x; last_normal = IVec3::new(-step_x, 0, 0);
            } else if t_max_y < t_max_z {
                y += step_y; traveled = t_max_y; t_max_y += t_delta_y; last_normal = IVec3::new(0, -step_y, 0);
            } else {
                z += step_z; traveled = t_max_z; t_max_z += t_delta_z; last_normal = IVec3::new(0, 0, -step_z);
            }
        }
        None
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RaycastHit> {
        let dir = dir.normalized();
        if dir.length_sq() < 1e-8 { return None; }
        Self::traverse_voxels(origin, dir, max_dist, |pos, normal, _| {
            self.get_block(pos).is_targetable().then_some(RaycastHit { block_pos: pos, normal })
        })
    }

    pub fn raycast_camera_collision(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> f32 {
        let dir = dir.normalized();
        if dir.length_sq() < 1e-8 { return max_dist; }
        Self::traverse_voxels(origin, dir, max_dist, |pos, _, traveled| {
            self.get_block(pos).is_solid().then(|| (traveled - 0.25).max(0.25))
        }).unwrap_or(max_dist)
    }

    pub fn calculate_acoustic_occlusion(&self, listener: Vec3, emitter: Vec3) -> f32 {
        let delta = emitter - listener;
        let dist = delta.length();
        if dist <= 0.6 { return 0.0; }

        let start_v = listener.floor_to_ivec();
        let end_v = emitter.floor_to_ivec();
        let mut occlusion_accum = 0.0f32;
        let mut steps = 0;

        Self::traverse_voxels(listener, delta * (1.0 / dist), dist, |pos, _, traveled| {
            if traveled >= dist || steps >= 64 { return Some(()); }
            steps += 1;
            if pos != start_v && pos != end_v {
                let block = self.get_block(pos);
                if block != BlockId::Air {
                    occlusion_accum += match block {
                        BlockId::Glass => 0.15,
                        BlockId::Water => 0.25,
                        BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => 0.12,
                        BlockId::Wood | BlockId::Planks | BlockId::Door | BlockId::DoorOpen => 0.35,
                        _ => 0.50,
                    };
                    if occlusion_accum >= 1.0 { return Some(()); }
                }
            }
            None
        });
        occlusion_accum.min(1.0)
    }

    pub fn box_collides(&self, min: Vec3, max: Vec3) -> bool {
        let eps = 1e-4;
        let (x0, x1) = (min.x.floor() as i32, (max.x - eps).floor() as i32);
        let (y0, y1) = ((min.y.floor() as i32).max(0), ((max.y - eps).floor() as i32).min(CHUNK_SIZE_Y - 1));
        let (z0, z1) = (min.z.floor() as i32, (max.z - eps).floor() as i32);
        if y0 > y1 { return false; }

        let (mut last_cx, mut last_cz, mut cached_chunk) = (i32::MIN, i32::MIN, None);
        for x in x0..=x1 {
            for z in z0..=z1 {
                let (cx, cz, lx, lz) = World::world_to_chunk(x, z);
                if cx != last_cx || cz != last_cz {
                    (last_cx, last_cz) = (cx, cz);
                    cached_chunk = self.chunks.get(&(cx, cz));
                }
                if let Some(chunk) = cached_chunk {
                    for y in y0..=y1 {
                        let b = chunk.get_local(lx, y, lz);
                        if b.is_solid() && (!b.is_half_block() || min.y < y as f32 + 0.5) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

pub(crate) fn read_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, count: usize) -> Result<&'a [u8], String> {
    if bytes.len().saturating_sub(*cursor) < count {
        return Err("corrupted world file".to_owned());
    }
    let slice = &bytes[*cursor..*cursor + count];
    *cursor += count;
    Ok(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_index_and_local_blocks() {
        let mut chunk = Chunk { cx: 0, cz: 0, blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize], dirty: false };
        chunk.set_local(0, 0, 0, BlockId::Stone);
        assert_eq!(chunk.get_local(0, 0, 0), BlockId::Stone);
        assert!(chunk.dirty);
        chunk.set_local(15, 63, 15, BlockId::IronOre);
        assert_eq!(chunk.get_local(15, 63, 15), BlockId::IronOre);
        assert_eq!(chunk.get_local(-1, 0, 0), BlockId::Air);
        assert_eq!(chunk.get_local(16, 0, 0), BlockId::Air);
    }

    #[test]
    fn test_find_spawn_position_disallowed_biomes() {
        for seed in [1337, 42, 999999, 1234567, 777] {
            let spawn = World::find_spawn_position(seed);
            let (biome, _) = get_biome_at(spawn.x.floor() as i32, spawn.z.floor() as i32, seed);
            assert!(!matches!(biome, Biome::DeepOcean | Biome::Ocean | Biome::Swamp));
            assert!(spawn.y >= 24.0);
        }
    }

    #[test]
    fn test_leaf_support_and_decay_after_trunk_removal() {
        let mut world = World::new();
        world.chunks.insert((0, 0), Chunk { cx: 0, cz: 0, blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize], dirty: false });
        for pos in [IVec3::new(5, 10, 5), IVec3::new(5, 11, 5)] { world.set_block(pos, BlockId::Wood); }
        for pos in [IVec3::new(5, 12, 5), IVec3::new(6, 11, 5), IVec3::new(4, 11, 5)] { world.set_block(pos, BlockId::Leaves); }

        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert!(world.find_unsupported_leaves_around(IVec3::new(5, 10, 5)).is_empty());

        world.set_block(IVec3::new(5, 10, 5), BlockId::Air);
        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));

        world.set_block(IVec3::new(5, 11, 5), BlockId::Air);
        assert!(!world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert_eq!(world.find_unsupported_leaves_around(IVec3::new(5, 11, 5)).len(), 3);
    }

    #[test]
    fn test_world_save_and_load_with_player_stats() {
        let path_str = std::env::temp_dir().join("rustcraft_test_player_world.bin").to_string_lossy().to_string();
        let mut world = World::new_with_seed(12345);
        world.name = "PlayerSaveTest".to_string();

        let mut player = crate::gameplay::player::Player::new();
        player.health = 13.0;
        player.hunger = 15.0;
        player.saturation = 4.0;
        player.exhaustion = 2.0;
        player.air = 18.0;
        player.selected_hotbar_slot = 4;
        player.inventory.hotbar[4] = Some(ItemStack::new(BlockId::Torch, 16));
        player.inventory.armor[1] = Some(ItemStack::new(BlockId::IronChestplate, 1));
        player.inventory.offhand = Some(ItemStack::new(BlockId::Shield, 1));

        world.saved_player = Some(player.save_state());
        assert!(world.save(&path_str).is_ok());

        let loaded = World::load(&path_str).expect("Should load world");
        assert_eq!(loaded.name, "PlayerSaveTest");
        let lp = loaded.saved_player.expect("Should load player data");
        assert_eq!((lp.health, lp.hunger, lp.saturation, lp.selected_hotbar_slot), (13.0, 15.0, 4.0, 4));
        assert_eq!(lp.hotbar[4].unwrap().block, BlockId::Torch);
        assert_eq!(lp.armor[1].unwrap().block, BlockId::IronChestplate);
        assert_eq!(lp.offhand.unwrap().block, BlockId::Shield);
        let _ = std::fs::remove_file(path_str);
    }

    #[test]
    fn test_all_new_biomes_generation_and_rarity() {
        let mut counts = HashMap::new();
        for x in (-2000..=2000).step_by(40) {
            for z in (-2000..=2000).step_by(40) {
                *counts.entry(get_biome_at(x, z, 424242).0).or_insert(0) += 1;
            }
        }
        for b in [Biome::Jungle, Biome::Badlands, Biome::DarkForest, Biome::Taiga, Biome::BirchForest, Biome::IceSpikes, Biome::MushroomIsland, Biome::Meadow, Biome::JaggedPeaks, Biome::FrozenPeaks, Biome::SnowySlopes] {
            assert!(counts.contains_key(&b), "{:?} must generate", b);
        }
        assert!(counts[&Biome::MushroomIsland] < counts[&Biome::Plains]);
        assert!(counts[&Biome::IceSpikes] < counts[&Biome::Taiga]);
    }

    #[test]
    fn test_biome_name_at_all_biomes() {
        assert!(!World::new_with_seed(123).biome_name_at(0, 0).is_empty());
    }

    #[test]
    fn test_all_biomes_have_distinct_color_filters() {
        let all = [
            Biome::DeepOcean, Biome::Ocean, Biome::SnowyTaiga, Biome::SnowyTundra, Biome::AutumnForest, Biome::Forest,
            Biome::CherryGrove, Biome::Plains, Biome::Swamp, Biome::Savanna, Biome::Desert, Biome::Mountain,
            Biome::Jungle, Biome::Badlands, Biome::DarkForest, Biome::Taiga, Biome::BirchForest, Biome::IceSpikes,
            Biome::MushroomIsland, Biome::Meadow, Biome::JaggedPeaks, Biome::FrozenPeaks, Biome::SnowySlopes,
        ];
        let mut seen = HashSet::new();
        for b in all {
            let f = b.color_filter();
            assert!(seen.insert(((f[0] * 1000.0).round() as i32, (f[1] * 1000.0).round() as i32, (f[2] * 1000.0).round() as i32)));
        }
        assert_eq!(seen.len(), 23);
    }

    #[test]
    fn test_smooth_mountain_transitions_no_sheer_biome_cliffs() {
        let seed = 424242;
        for x in (-1500..=1500).step_by(10) {
            for z in (-1500..=1500).step_by(10) {
                let ((b1, c1), (b2, c2)) = (get_biome_at(x, z, seed), get_biome_at(x + 1, z, seed));
                let is_m = |b| matches!(b, Biome::Meadow | Biome::SnowySlopes | Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks);
                if is_m(b1) && is_m(b2) {
                    assert!((get_height_at(x, z, seed, b1, c1) - get_height_at(x + 1, z, seed, b2, c2)).abs() <= 3);
                }
            }
        }
    }

    #[test]
    fn test_mountain_biome_block_continuity_no_checkerboard() {
        let seed = 2858965613;
        let chunk = Chunk::generate(29, 41, seed);
        for lx in 0..15 {
            for lz in 0..15 {
                let (wx, wz) = (chunk.cx * CHUNK_SIZE_X + lx, chunk.cz * CHUNK_SIZE_Z + lz);
                let gb = |dx, dz| {
                    let (b, c) = get_biome_at(wx + dx, wz + dz, seed);
                    chunk.get_local(lx + dx, get_height_at(wx + dx, wz + dz, seed, b, c), lz + dz)
                };
                let (b00, b10, b01, b11) = (gb(0, 0), gb(1, 0), gb(0, 1), gb(1, 1));
                assert!(!(b00 == b11 && b10 == b01 && b00 != b10));
            }
        }
    }

    #[test]
    fn test_ruststone_circuit_simulation() {
        let mut world = World::new_with_seed(12345);
        let (lever_pos, wire_pos, lamp_pos) = (IVec3::new(0, 51, 0), IVec3::new(1, 51, 0), IVec3::new(2, 51, 0));
        for x in 0..=2 { world.set_block(IVec3::new(x, 50, 0), BlockId::Stone); }
        world.set_block(lever_pos, BlockId::Lever);
        world.set_block(wire_pos, BlockId::RustStone);
        world.set_block(lamp_pos, BlockId::RustStoneLamp);

        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        world.set_block(lever_pos, BlockId::LeverOn);
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);

        world.set_block(lever_pos, BlockId::ButtonPressed);
        world.button_timers.insert(lever_pos, 0.5);
        world.update_ruststone(0.6, Vec3::new(100.0, 50.0, 100.0), &[]);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        world.set_block(wire_pos, BlockId::PressurePlate);
        world.update_ruststone(0.1, Vec3::new(1.2, 51.0, 0.2), &[]);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);
    }

    #[test]
    fn test_spaced_villages_and_torch_lighting() {
        let seed: i32 = 1337;
        let mut found = None;
        for rx in -2..=2 {
            for rz in -2..=2 {
                let vh = hash_coords(rx, rz, seed.wrapping_add(8881337));
                let (vcx, vcz) = (rx * 20 + (vh % 16) as i32 + 2, rz * 20 + ((vh >> 16) % 16) as i32 + 2);
                let (vb, vc) = get_biome_at(vcx * 16 + 8, vcz * 16 + 8, seed);
                let vh_y = get_height_at(vcx * 16 + 8, vcz * 16 + 8, seed, vb, vc);
                if matches!(vb, Biome::Plains | Biome::Meadow | Biome::Savanna | Biome::Desert | Biome::Taiga) && (23..CHUNK_SIZE_Y - 14).contains(&vh_y) {
                    found = Some((vcx, vcz));
                    break;
                }
            }
        }
        let (vcx, vcz) = found.unwrap();
        let center_chunk = Chunk::generate(vcx, vcz, seed);
        assert!(center_chunk.blocks.contains(&BlockId::Torch.to_u8()));
        assert!(Chunk::generate(vcx + 1, vcz, seed).blocks.contains(&BlockId::Bed.to_u8()));
        assert!(Chunk::generate(vcx - 1, vcz, seed).blocks.contains(&BlockId::Furnace.to_u8()));
    }
}