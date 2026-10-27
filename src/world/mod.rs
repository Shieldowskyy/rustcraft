pub mod block;
pub mod mesher;
pub mod storage;
pub mod tile_entity;

use std::collections::{HashMap, HashSet};

pub use block::BlockId;
pub use mesher::{
    build_chunk_mesh, build_chunk_mesh_from_job, build_subchunk_mesh, AsyncMesher, ChunkMeshJob,
    LightSource, Triangle, Vertex, NUM_SUBCHUNKS, SUBCHUNK_SIZE,
};
pub use storage::{save_current_world, save_path};
pub use tile_entity::{fuel_burn_time, smelting_result, ChestData, FurnaceData, SMELT_DURATION};

use crate::core::math::{IVec3, Vec3};
use crate::core::noise::{fractal_noise2, fractal_noise3};

pub const CHUNK_SIZE_X: i32 = 16;
pub const CHUNK_SIZE_Y: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 16;

pub const VIEW_DISTANCE: i32 = 4;
pub const DEFAULT_WORLD_SEED: u32 = 1337;

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
        if (x as u32) >= CHUNK_SIZE_X as u32 || (y as u32) >= CHUNK_SIZE_Y as u32 || (z as u32) >= CHUNK_SIZE_Z as u32 {
            return;
        }
        self.blocks[Chunk::index(x, y, z)] = b.to_u8();
        self.dirty = true;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    DeepOcean,
    Ocean,
    SnowyTaiga,
    SnowyTundra,
    AutumnForest,
    Forest,
    CherryGrove,
    Plains,
    Swamp,
    Savanna,
    Desert,
    Mountain,
    Jungle,
    Badlands,
    DarkForest,
    Taiga,
    BirchForest,
    IceSpikes,
    MushroomIsland,
    Meadow,
    JaggedPeaks,
    FrozenPeaks,
    SnowySlopes,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::DeepOcean => "DEEP OCEAN",
            Biome::Ocean => "OCEAN",
            Biome::MushroomIsland => "MUSHROOM ISLAND",
            Biome::IceSpikes => "ICE SPIKES",
            Biome::SnowyTaiga => "SNOWY TAIGA",
            Biome::SnowyTundra => "SNOWY TUNDRA",
            Biome::Taiga => "TAIGA",
            Biome::Desert => "DESERT",
            Biome::Badlands => "BADLANDS",
            Biome::Savanna => "SAVANNA",
            Biome::Jungle => "JUNGLE",
            Biome::Swamp => "SWAMP",
            Biome::Meadow => "MEADOW",
            Biome::Plains => "PLAINS",
            Biome::BirchForest => "BIRCH FOREST",
            Biome::Forest => "FOREST",
            Biome::DarkForest => "DARK FOREST",
            Biome::AutumnForest => "AUTUMN FOREST",
            Biome::CherryGrove => "CHERRY GROVE",
            Biome::Mountain => "MOUNTAINS",
            Biome::JaggedPeaks => "JAGGED PEAKS",
            Biome::FrozenPeaks => "FROZEN PEAKS",
            Biome::SnowySlopes => "SNOWY SLOPES",
        }
    }
}

#[inline(always)]
pub fn get_biome_at(wx: i32, wz: i32, seed: u32) -> (Biome, f32) {
    // Biome scale: ~800-1000 blocks per cycle for vast, cohesive regions
    let continentalness = fractal_noise2(wx as f32 * 0.0012, wz as f32 * 0.0012, seed.wrapping_add(2500), 4, 0.5);
    let temp = fractal_noise2(wx as f32 * 0.0010, wz as f32 * 0.0010, seed.wrapping_add(500), 4, 0.5);
    let humidity = fractal_noise2(wx as f32 * 0.0010, wz as f32 * 0.0010, seed.wrapping_add(1500), 4, 0.5);
    let rarity = fractal_noise2(wx as f32 * 0.0014, wz as f32 * 0.0014, seed.wrapping_add(4500), 4, 0.5);

    let biome = if continentalness < -0.38 {
        // Deep ocean: rare Mushroom Island
        if rarity > 0.38 {
            Biome::MushroomIsland
        } else {
            Biome::DeepOcean
        }
    } else if continentalness < -0.16 {
        // Ocean: very rare Mushroom Island on the ocean boundary
        if continentalness < -0.28 && rarity > 0.40 {
            Biome::MushroomIsland
        } else {
            Biome::Ocean
        }
    } else if continentalness > 0.34 {
        // Mountain region: high peaks and alpine ranges
        if temp < -0.08 {
            if rarity > 0.06 {
                Biome::FrozenPeaks
            } else if continentalness > 0.44 {
                Biome::JaggedPeaks
            } else {
                Biome::SnowySlopes
            }
        } else if continentalness < 0.44 && temp > -0.05 && temp < 0.22 && humidity > -0.05 {
            Biome::Meadow
        } else if continentalness > 0.48 && temp < 0.10 {
            Biome::JaggedPeaks
        } else if continentalness > 0.38 && temp < 0.02 {
            Biome::SnowySlopes
        } else {
            Biome::Mountain
        }
    } else if temp < -0.30 {
        // Cold / Snowy zones (rare arctic regions)
        if continentalness > 0.08 {
            if rarity > 0.08 {
                Biome::FrozenPeaks
            } else {
                Biome::SnowySlopes
            }
        } else if temp < -0.40 && rarity > 0.30 {
            // Ice Spikes: rare spikes of ice in frozen lands
            Biome::IceSpikes
        } else if humidity > 0.08 {
            Biome::SnowyTaiga
        } else {
            Biome::SnowyTundra
        }
    } else if temp < -0.05 {
        // Cold-temperate zone
        if humidity > 0.16 {
            Biome::AutumnForest
        } else if humidity > -0.05 {
            Biome::Taiga // non-snowy coniferous taiga
        } else {
            Biome::Plains
        }
    } else if temp < 0.22 {
        // Temperate zone
        if humidity > 0.28 {
            if rarity > 0.16 {
                Biome::DarkForest // dense dark oak canopy
            } else if temp < 0.10 {
                Biome::Swamp
            } else {
                Biome::Forest
            }
        } else if humidity > 0.10 {
            if rarity < -0.18 {
                Biome::BirchForest // light birch forest
            } else {
                Biome::Forest
            }
        } else if humidity > -0.04 && temp > 0.09 {
            Biome::CherryGrove
        } else {
            Biome::Plains
        }
    } else {
        // Hot / Arid & Tropical zones (temp >= 0.22)
        if humidity > 0.22 {
            Biome::Jungle // hot and humid tropical jungle
        } else if humidity < -0.18 && rarity > 0.12 {
            Biome::Badlands // rare hot terracotta mesa canyon
        } else if humidity < -0.08 {
            Biome::Desert
        } else if humidity < 0.16 {
            Biome::Savanna
        } else {
            Biome::Forest
        }
    };

    (biome, continentalness)
}

#[inline(always)]
pub fn get_height_at(wx: i32, wz: i32, seed: u32, biome: Biome, continentalness: f32) -> i32 {
    let n = fractal_noise2(wx as f32 * 0.012, wz as f32 * 0.012, seed, 4, 0.5);
    let h = match biome {
        Biome::DeepOcean => 6.0 + n * 3.0,
        Biome::Ocean => 13.0 + n * 4.0,
        Biome::MushroomIsland => 23.0 + n * 3.0,
        Biome::Swamp => 20.0 + n * 2.5,
        Biome::Mountain => {
            let m_extra = ((continentalness - 0.34) / 0.66).clamp(0.0, 1.0);
            let ridge = 1.0 - (n * 2.0).abs().clamp(0.0, 1.0);
            38.0 + n * 8.0 + ridge * 12.0 + m_extra * 8.0
        }
        Biome::JaggedPeaks => {
            let m_extra = ((continentalness - 0.34) / 0.66).clamp(0.0, 1.0);
            let ridge = 1.0 - (n * 2.4).abs().clamp(0.0, 1.0);
            44.0 + ridge * 15.0 + m_extra * 4.0 + n * 3.0
        }
        Biome::FrozenPeaks => {
            let m_extra = ((continentalness - 0.34) / 0.66).clamp(0.0, 1.0);
            let ridge = 1.0 - (n * 2.0).abs().clamp(0.0, 1.0);
            42.0 + ridge * 15.0 + m_extra * 5.0 + n * 3.0
        }
        Biome::SnowySlopes => {
            let m_extra = ((continentalness - 0.18) / 0.50).clamp(0.0, 1.0);
            35.0 + n * 8.0 + m_extra * 12.0
        }
        Biome::Meadow => {
            let m_extra = ((continentalness - 0.34) / 0.10).clamp(0.0, 1.0);
            31.0 + n * 6.0 + m_extra * 6.0
        }
        Biome::Badlands => {
            let base = 32.0 + n * 8.0;
            (base / 3.0).floor() * 3.0 + 1.0
        }
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

impl Chunk {
    pub fn generate(cx: i32, cz: i32, seed: u32) -> Chunk {
        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];

        enum Decoration {
            Cactus(i32, i32, i32, i32),
            PineTree(i32, i32, i32),
            Tree(i32, i32, i32),
            CherryTree(i32, i32, i32),
            AcaciaTree(i32, i32, i32),
            SwampTree(i32, i32, i32),
            AutumnTree(i32, i32, i32),
            JungleTree(i32, i32, i32, i32),
            JungleBush(i32, i32, i32),
            DarkOakTree(i32, i32, i32),
            BirchTree(i32, i32, i32),
            IceSpike(i32, i32, i32, i32),
            HugeRedMushroom(i32, i32, i32),
            HugeBrownMushroom(i32, i32, i32),
        }
        let mut decorations = Vec::new();

        enum Structure {
            DesertWell(i32, i32, i32),
            StoneRuins(i32, i32, i32),
            ForestCabin(i32, i32, i32),
            CoralRuin(i32, i32, i32),
            MountainShrine(i32, i32, i32),
            FrozenRuin(i32, i32, i32),
            DesertBuriedRuin(i32, i32, i32),
            OvergrownTempleRuin(i32, i32, i32),
            CaveDungeonRuin(i32, i32, i32),
        }

        let center_wx = cx * CHUNK_SIZE_X + 8;
        let center_wz = cz * CHUNK_SIZE_Z + 8;
        let (c_biome, c_continentalness) = get_biome_at(center_wx, center_wz, seed);
        let center_height = get_height_at(center_wx, center_wz, seed, c_biome, c_continentalness);

        let chunk_hash = hash_coords(cx, cz, seed.wrapping_add(77777));
        let mut structure: Option<Structure> = None;
        let mut struct_exclusion = (100, -100, 100, -100);

        if center_height > 22 && center_height < CHUNK_SIZE_Y - 8 {
            if (c_biome == Biome::Desert || c_biome == Biome::Badlands) && (chunk_hash % 71 == 0) {
                structure = Some(Structure::DesertWell(8, center_height, 8));
                struct_exclusion = (4, 12, 4, 12);
            } else if (c_biome == Biome::Desert || c_biome == Biome::Badlands) && (chunk_hash % 61 == 0) {
                structure = Some(Structure::DesertBuriedRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if (c_biome == Biome::Mountain || c_biome == Biome::JaggedPeaks || c_biome == Biome::SnowySlopes) && (chunk_hash % 59 == 0) {
                structure = Some(Structure::MountainShrine(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if (c_biome == Biome::FrozenPeaks || c_biome == Biome::SnowyTundra || c_biome == Biome::IceSpikes || c_biome == Biome::SnowyTaiga) && (chunk_hash % 63 == 0) {
                structure = Some(Structure::FrozenRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if (c_biome == Biome::Jungle || c_biome == Biome::DarkForest || c_biome == Biome::Swamp) && (chunk_hash % 67 == 0) {
                structure = Some(Structure::OvergrownTempleRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if (c_biome == Biome::Mountain || c_biome == Biome::SnowyTundra || c_biome == Biome::Taiga) && (chunk_hash % 73 == 0) {
                structure = Some(Structure::StoneRuins(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if (c_biome == Biome::Forest || c_biome == Biome::AutumnForest || c_biome == Biome::DarkForest || c_biome == Biome::BirchForest) && (chunk_hash % 79 == 0) {
                structure = Some(Structure::ForestCabin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            }
        }
        // Deep ocean: coral ruins near the seabed
        if c_biome == Biome::DeepOcean && center_height >= 4 && center_height < 12 && (chunk_hash % 53 == 0) {
            structure = Some(Structure::CoralRuin(8, center_height, 8));
            struct_exclusion = (4, 12, 4, 12);
        }
        // Underground cave dungeon ruin
        if structure.is_none() && center_height >= 30 && (chunk_hash % 79 == 0) {
            let dungeon_y = 10 + (chunk_hash % 8) as i32;
            structure = Some(Structure::CaveDungeonRuin(8, dungeon_y, 8));
        }

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = cx * CHUNK_SIZE_X + lx;
                let wz = cz * CHUNK_SIZE_Z + lz;

                let (biome, continentalness) = get_biome_at(wx, wz, seed);
                let height = get_height_at(wx, wz, seed, biome, continentalness);

                for y in 0..CHUNK_SIZE_Y {
                    if y == 0 {
                        blocks[Chunk::index(lx, y, lz)] = BlockId::Bedrock.to_u8();
                        continue;
                    }
                    if y == 1 {
                        let h = hash_coords_3d(wx, y, wz, seed);
                        blocks[Chunk::index(lx, y, lz)] = if h % 3 == 0 {
                            BlockId::Bedrock.to_u8()
                        } else {
                            BlockId::Stone.to_u8()
                        };
                        continue;
                    }

                    let block = if y > height {
                        if y <= 20 {
                            if (biome == Biome::SnowyTaiga || biome == Biome::SnowyTundra || biome == Biome::IceSpikes || biome == Biome::FrozenPeaks) && y == 20 {
                                BlockId::Ice
                            } else {
                                BlockId::Water
                            }
                        } else {
                            BlockId::Air
                        }
                    } else if y == height {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean => BlockId::Sand,
                            Biome::Swamp => {
                                if height <= 20 { BlockId::Mud } else { BlockId::Grass }
                            }
                            Biome::MushroomIsland => BlockId::Mud,
                            Biome::Desert => BlockId::Sand,
                            Biome::Badlands => {
                                if (height / 2) % 3 == 0 { BlockId::Sand } else if (height / 2) % 3 == 1 { BlockId::Stone } else { BlockId::Mud }
                            }
                            Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::SnowySlopes => BlockId::Snow,
                            Biome::JaggedPeaks => {
                                if height >= 52 {
                                    BlockId::Snow
                                } else if height >= 42 {
                                    if (wx + wz) % 3 == 0 { BlockId::Stone } else { BlockId::Snow }
                                } else {
                                    BlockId::Snow
                                }
                            }
                            Biome::FrozenPeaks => {
                                let h = hash_coords_3d(wx, y, wz, seed);
                                if h % 4 == 0 {
                                    BlockId::Ice
                                } else {
                                    BlockId::Snow
                                }
                            }
                            Biome::Mountain => {
                                if height >= 48 {
                                    BlockId::Snow
                                } else if height >= 38 {
                                    BlockId::Stone
                                } else {
                                    BlockId::Grass
                                }
                            }
                            _ => {
                                if height <= 21 {
                                    BlockId::Sand
                                } else {
                                    BlockId::Grass
                                }
                            }
                        }
                    } else if y > height - 4 {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean | Biome::Desert => BlockId::Sand,
                            Biome::Badlands => {
                                if (y / 2) % 3 == 0 { BlockId::Sand } else if (y / 2) % 3 == 1 { BlockId::Stone } else { BlockId::Mud }
                            }
                            Biome::Swamp | Biome::MushroomIsland => BlockId::Mud,
                            Biome::IceSpikes => {
                                if y > height - 2 { BlockId::Snow } else { BlockId::Ice }
                            }
                            Biome::FrozenPeaks => {
                                if y > height - 2 {
                                    BlockId::Snow
                                } else if y > height - 6 {
                                    BlockId::Ice
                                } else {
                                    BlockId::Stone
                                }
                            }
                            Biome::JaggedPeaks => {
                                if y > height - 2 { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::SnowySlopes => {
                                if y > height - 3 { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::Mountain if height >= 38 => BlockId::Stone,
                            _ => if height <= 21 { BlockId::Sand } else { BlockId::Dirt },
                        }
                    } else if biome == Biome::Badlands && y >= 14 {
                        let ore_h = hash_coords_3d(wx, y, wz, seed);
                        if ore_h % 37 == 0 {
                            BlockId::CoalOre
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else if ore_h % 127 == 0 && y < 20 {
                            BlockId::GoldOre
                        } else {
                            match (y / 2) % 4 {
                                0 => BlockId::Sand,
                                1 => BlockId::Stone,
                                2 => BlockId::Mud,
                                _ => BlockId::Stone,
                            }
                        }
                    } else {
                        let ore_h = hash_coords_3d(wx, y, wz, seed);
                        if ore_h % 37 == 0 {
                            BlockId::CoalOre
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else if ore_h % 127 == 0 && y < 20 {
                            BlockId::GoldOre
                        } else if ore_h % 199 == 0 && y < 14 {
                            BlockId::DiamondOre
                        } else {
                            BlockId::Stone
                        }
                    };

                    blocks[Chunk::index(lx, y, lz)] = block.to_u8();

                    let is_mountain = matches!(biome, Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes);
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
                        // Carve hollow chambers inside mountain, and open portals on steep mountain faces
                        if cave_dist_sq < 0.026 && (y < height || (cave_dist_sq < 0.015 && height >= 32)) {
                            is_cave = true;
                        }
                    }

                    if is_cave {
                        if y <= 7 {
                            blocks[Chunk::index(lx, y, lz)] = BlockId::Lava.to_u8();
                        } else {
                            blocks[Chunk::index(lx, y, lz)] = BlockId::Air.to_u8();
                        }
                    }
                }

                if height > 21 {
                    if lx >= struct_exclusion.0 && lx <= struct_exclusion.1 && lz >= struct_exclusion.2 && lz <= struct_exclusion.3 {
                        continue;
                    }

                    if lx >= 3 && lx < CHUNK_SIZE_X - 3 && lz >= 3 && lz < CHUNK_SIZE_Z - 3 {
                        let h = hash_coords(wx, wz, seed);
                        match biome {
                            Biome::Desert => {
                                if h % 167 == 0 {
                                    let c_height = 2 + (h % 2) as i32;
                                    decorations.push(Decoration::Cactus(lx, height + 1, lz, c_height));
                                }
                            }
                            Biome::Badlands => {
                                if h % 181 == 0 {
                                    let c_height = 2 + (h % 2) as i32;
                                    decorations.push(Decoration::Cactus(lx, height + 1, lz, c_height));
                                }
                            }
                            Biome::SnowyTaiga => {
                                if h % 97 == 0 {
                                    decorations.push(Decoration::PineTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Taiga => {
                                if h % 59 == 0 {
                                    decorations.push(Decoration::PineTree(lx, height + 1, lz));
                                }
                            }
                            Biome::SnowyTundra => {
                                if h % 233 == 0 {
                                    decorations.push(Decoration::PineTree(lx, height + 1, lz));
                                }
                            }
                            Biome::IceSpikes => {
                                if h % 47 == 0 {
                                    let spike_h = 7 + (h % 9) as i32;
                                    decorations.push(Decoration::IceSpike(lx, height + 1, lz, spike_h));
                                }
                            }
                            Biome::AutumnForest => {
                                if h % 47 == 0 {
                                    decorations.push(Decoration::AutumnTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Forest => {
                                if h % 43 == 0 {
                                    decorations.push(Decoration::Tree(lx, height + 1, lz));
                                }
                            }
                            Biome::BirchForest => {
                                if h % 41 == 0 {
                                    decorations.push(Decoration::BirchTree(lx, height + 1, lz));
                                }
                            }
                            Biome::DarkForest => {
                                if h % 29 == 0 {
                                    decorations.push(Decoration::DarkOakTree(lx, height + 1, lz));
                                } else if h % 83 == 0 {
                                    decorations.push(Decoration::HugeRedMushroom(lx, height + 1, lz));
                                } else if h % 89 == 0 {
                                    decorations.push(Decoration::HugeBrownMushroom(lx, height + 1, lz));
                                }
                            }
                            Biome::Jungle => {
                                if h % 43 == 0 {
                                    let j_height = 10 + (h % 7) as i32;
                                    decorations.push(Decoration::JungleTree(lx, height + 1, lz, j_height));
                                } else if h % 31 == 0 {
                                    decorations.push(Decoration::JungleBush(lx, height + 1, lz));
                                }
                            }
                            Biome::MushroomIsland => {
                                if h % 37 == 0 {
                                    decorations.push(Decoration::HugeRedMushroom(lx, height + 1, lz));
                                } else if h % 41 == 0 {
                                    decorations.push(Decoration::HugeBrownMushroom(lx, height + 1, lz));
                                }
                            }
                            Biome::CherryGrove => {
                                if h % 53 == 0 {
                                    decorations.push(Decoration::CherryTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Swamp => {
                                if h % 67 == 0 {
                                    decorations.push(Decoration::SwampTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Savanna => {
                                if h % 163 == 0 {
                                    decorations.push(Decoration::AcaciaTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Plains => {
                                if h % 149 == 0 {
                                    decorations.push(Decoration::Tree(lx, height + 1, lz));
                                }
                            }
                            Biome::Meadow => {
                                if h % 199 == 0 {
                                    decorations.push(Decoration::Tree(lx, height + 1, lz));
                                }
                            }
                            Biome::SnowySlopes => {
                                if h % 127 == 0 {
                                    decorations.push(Decoration::PineTree(lx, height + 1, lz));
                                }
                            }
                            Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::Ocean | Biome::DeepOcean => {}
                        }
                    }
                }
            }
        }

        if let Some(struct_type) = structure {
            match struct_type {
                Structure::DesertWell(cx, base_y, cz) => place_desert_well(&mut blocks, cx, base_y, cz),
                Structure::StoneRuins(cx, base_y, cz) => place_stone_ruin(&mut blocks, cx, base_y, cz, seed),
                Structure::ForestCabin(cx, base_y, cz) => place_forest_cabin(&mut blocks, cx, base_y, cz),
                Structure::CoralRuin(cx, base_y, cz) => place_coral_ruin(&mut blocks, cx, base_y, cz, seed),
                Structure::MountainShrine(cx, base_y, cz) => place_mountain_shrine(&mut blocks, cx, base_y, cz, seed),
                Structure::FrozenRuin(cx, base_y, cz) => place_frozen_ruin(&mut blocks, cx, base_y, cz, seed),
                Structure::DesertBuriedRuin(cx, base_y, cz) => place_desert_buried_ruin(&mut blocks, cx, base_y, cz, seed),
                Structure::OvergrownTempleRuin(cx, base_y, cz) => place_overgrown_temple_ruin(&mut blocks, cx, base_y, cz, seed),
                Structure::CaveDungeonRuin(cx, base_y, cz) => place_cave_dungeon_ruin(&mut blocks, cx, base_y, cz, seed),
            }
        }

        for deco in decorations {
            match deco {
                Decoration::Cactus(lx, base_y, lz, h) => place_cactus(&mut blocks, lx, base_y, lz, h),
                Decoration::PineTree(lx, base_y, lz) => place_pine_tree(&mut blocks, lx, base_y, lz),
                Decoration::Tree(lx, base_y, lz) => place_tree(&mut blocks, lx, base_y, lz),
                Decoration::CherryTree(lx, base_y, lz) => place_cherry_tree(&mut blocks, lx, base_y, lz),
                Decoration::AcaciaTree(lx, base_y, lz) => place_acacia_tree(&mut blocks, lx, base_y, lz),
                Decoration::SwampTree(lx, base_y, lz) => place_swamp_tree(&mut blocks, lx, base_y, lz),
                Decoration::AutumnTree(lx, base_y, lz) => place_autumn_tree(&mut blocks, lx, base_y, lz),
                Decoration::JungleTree(lx, base_y, lz, h) => place_jungle_tree(&mut blocks, lx, base_y, lz, h),
                Decoration::JungleBush(lx, base_y, lz) => place_jungle_bush(&mut blocks, lx, base_y, lz),
                Decoration::DarkOakTree(lx, base_y, lz) => place_dark_oak_tree(&mut blocks, lx, base_y, lz),
                Decoration::BirchTree(lx, base_y, lz) => place_birch_tree(&mut blocks, lx, base_y, lz),
                Decoration::IceSpike(lx, base_y, lz, h) => place_ice_spike(&mut blocks, lx, base_y, lz, h),
                Decoration::HugeRedMushroom(lx, base_y, lz) => place_huge_red_mushroom(&mut blocks, lx, base_y, lz),
                Decoration::HugeBrownMushroom(lx, base_y, lz) => place_huge_brown_mushroom(&mut blocks, lx, base_y, lz),
            }
        }

        Chunk {
            cx,
            cz,
            blocks,
            dirty: true,
        }
    }
}

fn hash_coords(x: i32, z: i32, seed: u32) -> u32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((z as u32).wrapping_mul(668265263))
        .wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn hash_coords_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(1103515245))
        .wrapping_add((z as u32).wrapping_mul(668265263))
        .wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn place_cactus(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32) {
    for i in 0..height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Cactus.to_u8();
        }
    }
}

fn place_pine_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 6;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dy in 2..=trunk_height + 1 {
        let radius: i32 = if dy == trunk_height + 1 {
            0
        } else if dy >= trunk_height - 1 {
            1
        } else if dy % 2 == 0 {
            2
        } else {
            1
        };

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let x = lx + dx;
                let y = base_y + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                if dx == 0 && dz == 0 && dy < trunk_height {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    blocks[idx] = BlockId::SnowLeaves.to_u8();
                }
            }
        }
    }
}

fn place_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 4;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dx in -2..=2 {
        for dz in -2..=2 {
            for dy in 0..=2 {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z {
                    continue;
                }
                if y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let dist = dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2;
                if dist <= 6 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Cherry Blossom tree: short trunk + wide rounded crown of pink CherryLeaves
fn place_cherry_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 3;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    // Wide rounded canopy of cherry leaves
    for dx in -3..=3 {
        for dz in -3..=3 {
            for dy in 0..=3 {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                // Ellipsoid shape: wide (r=3 XZ) and short (r=2 Y)
                let dist_sq = dx * dx * 10 + dz * dz * 10 + (dy - 1) * (dy - 1) * 25;
                if dist_sq <= 90 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::CherryLeaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Acacia tree (Savanna): tall trunk + wide, flat umbrella canopy
fn place_acacia_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 5;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    // Flat wide canopy: 2 leaf layers spread over radius 3
    for dy in 0..=1i32 {
        let radius = if dy == 0 { 3i32 } else { 2i32 };
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let x = lx + dx;
                let y = base_y + trunk_height + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let dist = dx * dx + dz * dz;
                if dist <= radius * radius {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Swamp tree: tall thin trunk with drooping wide leaf canopy
fn place_swamp_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 6;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    // Wide drooping canopy: 3 layers, shrinking upward
    for dy in 0..=3i32 {
        let radius = match dy {
            0 => 3i32,
            1 => 3,
            2 => 2,
            _ => 1,
        };
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= radius * radius {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Autumn tree: medium trunk with rounded AutumnLeaves canopy
fn place_autumn_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 4;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dx in -2..=2 {
        for dz in -2..=2 {
            for dy in 0..=3 {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z {
                    continue;
                }
                if y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let dist = dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2;
                if dist <= 7 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::AutumnLeaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Jungle tree: tall trunk reaching into the sky with a massive umbrella canopy
fn place_jungle_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, trunk_height: i32) {
    for dy in 0..trunk_height {
        let y = base_y + dy;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dy in -1..=2i32 {
        let r = match dy {
            -1 => 2i32,
            0 => 3i32,
            1 => 3i32,
            _ => 1i32,
        };
        for dx in -r..=r {
            for dz in -r..=r {
                let x = lx + dx;
                let y = base_y + trunk_height + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                if dx * dx + dz * dz <= r * r + 1 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Jungle bush: short ground bush with a single wood core and dense leaves
fn place_jungle_bush(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    if base_y >= 0 && base_y < CHUNK_SIZE_Y {
        blocks[Chunk::index(lx, base_y, lz)] = BlockId::Wood.to_u8();
    }
    for dx in -1..=1 {
        for dz in -1..=1 {
            for dy in 0..=1 {
                let x = lx + dx;
                let y = base_y + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    blocks[idx] = BlockId::Leaves.to_u8();
                }
            }
        }
    }
}

/// Dark Oak tree: thick trunk with wide, dense roof canopy that blocks the sky
fn place_dark_oak_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 5;
    for dy in 0..trunk_height {
        let y = base_y + dy;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dy in 0..=2i32 {
        let r = if dy == 2 { 2i32 } else { 3i32 };
        for dx in -r..=r {
            for dz in -r..=r {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    blocks[idx] = BlockId::Leaves.to_u8();
                }
            }
        }
    }
}

/// Birch tree: slender trunk with light, elegant foliage
fn place_birch_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 5;
    for dy in 0..trunk_height {
        let y = base_y + dy;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    for dy in 0..=2i32 {
        let r = if dy == 2 { 1i32 } else { 2i32 };
        for dx in -r..=r {
            for dz in -r..=r {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                if dx.abs() + dz.abs() <= r + 1 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

/// Ice Spike: tall sharp spire of pure ice with a snow tip
fn place_ice_spike(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32) {
    for dy in 0..height {
        let y = base_y + dy;
        if y < 0 || y >= CHUNK_SIZE_Y {
            continue;
        }
        let radius = if dy < height / 3 {
            2i32
        } else if dy < (height * 2) / 3 {
            1i32
        } else {
            0i32
        };
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dz * dz <= radius * radius + 1 {
                    let x = lx + dx;
                    let z = lz + dz;
                    if x >= 0 && x < CHUNK_SIZE_X && z >= 0 && z < CHUNK_SIZE_Z {
                        let idx = Chunk::index(x, y, z);
                        blocks[idx] = BlockId::Ice.to_u8();
                    }
                }
            }
        }
    }
    let tip_y = base_y + height;
    if tip_y >= 0 && tip_y < CHUNK_SIZE_Y && lx >= 0 && lx < CHUNK_SIZE_X && lz >= 0 && lz < CHUNK_SIZE_Z {
        blocks[Chunk::index(lx, tip_y, lz)] = BlockId::Snow.to_u8();
    }
}

/// Giant Red Mushroom: white stalk with rounded red umbrella cap
fn place_huge_red_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let stem_height = 5;
    for dy in 0..stem_height {
        let y = base_y + dy;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Snow.to_u8();
        }
    }
    for dy in 0..=2i32 {
        let r = if dy == 2 { 1i32 } else { 2i32 };
        for dx in -r..=r {
            for dz in -r..=r {
                let x = lx + dx;
                let y = base_y + stem_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                if dy == 0 && (dx.abs() != r && dz.abs() != r) {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    blocks[idx] = BlockId::AutumnLeaves.to_u8();
                }
            }
        }
    }
}

/// Giant Brown Mushroom: white stalk with wide flat saucer cap
fn place_huge_brown_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let stem_height = 4;
    for dy in 0..stem_height {
        let y = base_y + dy;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Snow.to_u8();
        }
    }
    let y = base_y + stem_height;
    if y >= 0 && y < CHUNK_SIZE_Y {
        for dx in -2..=2 {
            for dz in -2..=2 {
                let x = lx + dx;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    blocks[idx] = BlockId::Mud.to_u8();
                }
            }
        }
    }
}

/// Coral ruin (Deep Ocean): underwater columns and arches of stone and sand
fn place_coral_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    // Scattered stone columns rising from seabed
    let offsets: &[(i32, i32)] = &[(-2, -2), (-2, 2), (2, -2), (2, 2), (0, -3), (3, 0), (-3, 0), (0, 3)];
    for &(dx, dz) in offsets {
        let cx2 = center_x + dx;
        let cz2 = center_z + dz;
        let col_h = 2 + (hash_coords_3d(cx2, 77, cz2, seed) % 5) as i32;
        let block = if (dx + dz) % 2 == 0 { BlockId::Stone } else { BlockId::Sand };
        for dy in 0..=col_h {
            set(blocks, cx2, base_y + dy, cz2, block);
        }
    }

    // Central stone pedestal
    for dx in -1..=1i32 {
        for dz in -1..=1i32 {
            set(blocks, center_x + dx, base_y, center_z + dz, BlockId::Stone);
            set(blocks, center_x + dx, base_y + 1, center_z + dz, BlockId::Stone);
        }
    }
    // Lava core at center for a dramatic glowing ruin effect
    set(blocks, center_x, base_y + 2, center_z, BlockId::Lava);
}

fn place_desert_well(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32) {
    let min_x = center_x - 2;
    let max_x = center_x + 2;
    let min_z = center_z - 2;
    let max_z = center_z + 2;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 4)..base_y {
                set(blocks, x, y, z, BlockId::Sand);
            }
            for y in (base_y + 1)..=(base_y + 6) {
                set(blocks, x, y, z, BlockId::Air);
            }

            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
            if is_border {
                set(blocks, x, base_y, z, BlockId::Sand);
            } else {
                set(blocks, x, base_y - 1, z, BlockId::Sand);
                set(blocks, x, base_y, z, BlockId::Water);
            }
        }
    }

    for &(cx, cz) in &[(min_x, min_z), (min_x, max_z), (max_x, min_z), (max_x, max_z)] {
        for dy in 1..=3 {
            set(blocks, cx, base_y + dy, cz, BlockId::Sand);
        }
    }

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            set(blocks, x, base_y + 4, z, BlockId::Sand);
        }
    }
    set(blocks, center_x, base_y + 5, center_z, BlockId::Sand);
}

fn place_stone_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 4)..base_y {
                set(blocks, x, y, z, BlockId::Stone);
            }
            for y in (base_y + 1)..=(base_y + 6) {
                set(blocks, x, y, z, BlockId::Air);
            }

            let h = hash_coords_3d(x, base_y, z, seed);
            let floor_b = if h % 11 == 0 {
                BlockId::CoalOre
            } else if h % 17 == 0 {
                BlockId::IronOre
            } else {
                BlockId::Stone
            };
            set(blocks, x, base_y, z, floor_b);
        }
    }

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
            if !is_border {
                continue;
            }
            if (x == center_x || x == center_x + 1) && z == min_z {
                continue;
            }
            let wall_h = (hash_coords_3d(x, 101, z, seed) % 4) as i32;
            for dy in 1..=wall_h {
                set(blocks, x, base_y + dy, z, BlockId::Stone);
            }
        }
    }

    for &(cx, cz) in &[(min_x, min_z), (min_x, max_z), (max_x, min_z), (max_x, max_z)] {
        let pil_h = 2 + (hash_coords_3d(cx, 137, cz, seed) % 3) as i32;
        for dy in 1..=pil_h {
            set(blocks, cx, base_y + dy, cz, BlockId::Stone);
        }
    }

    set(blocks, center_x, base_y + 1, center_z, BlockId::Stone);
    set(blocks, center_x, base_y + 2, center_z, BlockId::Lava);
    set(blocks, center_x - 1, base_y + 1, center_z, BlockId::Stone);
    set(blocks, center_x + 1, base_y + 1, center_z, BlockId::Stone);
    set(blocks, center_x, base_y + 1, center_z - 1, BlockId::Stone);
    set(blocks, center_x, base_y + 1, center_z + 1, BlockId::Stone);
}

fn place_forest_cabin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32) {
    let min_x = center_x - 3;
    let max_x = center_x + 2;
    let min_z = center_z - 3;
    let max_z = center_z + 2;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    for x in (min_x - 1)..=(max_x + 1) {
        for z in (min_z - 1)..=(max_z + 1) {
            for y in (base_y + 1)..=(base_y + 8) {
                set(blocks, x, y, z, BlockId::Air);
            }
        }
    }

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 4)..base_y {
                set(blocks, x, y, z, BlockId::Wood);
            }
            set(blocks, x, base_y, z, BlockId::Planks);
        }
    }

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let is_corner = (x == min_x || x == max_x) && (z == min_z || z == max_z);
            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;

            if is_corner {
                for dy in 1..=3 {
                    set(blocks, x, base_y + dy, z, BlockId::Wood);
                }
            } else if is_border {
                for dy in 1..=3 {
                    if x == center_x && z == min_z {
                        if dy == 1 || dy == 2 {
                            set(blocks, x, base_y + dy, z, BlockId::Door);
                        } else {
                            set(blocks, x, base_y + dy, z, BlockId::Planks);
                        }
                    } else if dy == 2
                        && (((x == min_x || x == max_x) && (z == min_z + 2 || z == min_z + 3))
                        || (z == max_z && (x == min_x + 2 || x == min_x + 3)))
                    {
                        set(blocks, x, base_y + dy, z, BlockId::Glass);
                    } else {
                        set(blocks, x, base_y + dy, z, BlockId::Planks);
                    }
                }
            }
        }
    }

    for x in (min_x - 1)..=(max_x + 1) {
        for z in (min_z - 1)..=(max_z + 1) {
            let is_edge = x == min_x - 1 || x == max_x + 1 || z == min_z - 1 || z == max_z + 1;
            set(blocks, x, base_y + 4, z, if is_edge { BlockId::Wood } else { BlockId::Planks });
        }
    }
    for x in min_x..=max_x {
        for z in (min_z - 1)..=(max_z + 1) {
            set(blocks, x, base_y + 5, z, BlockId::Planks);
        }
    }
    for x in (center_x - 1)..=center_x {
        for z in (min_z - 1)..=(max_z + 1) {
            set(blocks, x, base_y + 6, z, BlockId::Wood);
        }
    }

    // Cabin interior furniture: Crafting table, Furnace, Chest and Torch
    set(blocks, min_x + 1, base_y + 1, max_z - 1, BlockId::CraftingTable);
    set(blocks, min_x + 2, base_y + 1, max_z - 1, BlockId::Furnace);
    set(blocks, max_x - 1, base_y + 1, max_z - 1, BlockId::Chest);
    set(blocks, min_x + 1, base_y + 3, max_z - 1, BlockId::Torch);
}

fn place_mountain_shrine(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    // Solid foundation down and open air above
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 4)..base_y {
                set(blocks, x, y, z, BlockId::Stone);
            }
            for y in (base_y + 1)..=(base_y + 7) {
                set(blocks, x, y, z, BlockId::Air);
            }
            set(blocks, x, base_y, z, BlockId::Stone);
        }
    }

    // 4 Corner pillars with torches
    for &(cx, cz) in &[(min_x, min_z), (min_x, max_z), (max_x, min_z), (max_x, max_z)] {
        let pil_h = 4 + (hash_coords_3d(cx, 137, cz, seed) % 2) as i32;
        for dy in 1..=pil_h {
            set(blocks, cx, base_y + dy, cz, BlockId::Stone);
        }
        set(blocks, cx, base_y + pil_h + 1, cz, BlockId::Torch);
    }

    // Central altar pedestal with gold block and shrine chest
    for dx in -1..=1 {
        for dz in -1..=1 {
            set(blocks, center_x + dx, base_y + 1, center_z + dz, BlockId::Stone);
        }
    }
    set(blocks, center_x, base_y + 2, center_z, BlockId::GoldBlock);
    set(blocks, center_x, base_y + 3, center_z, BlockId::Torch);
    set(blocks, center_x + 1, base_y + 2, center_z, BlockId::Chest);
}

fn place_frozen_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 4)..base_y {
                set(blocks, x, y, z, BlockId::Ice);
            }
            for y in (base_y + 1)..=(base_y + 6) {
                set(blocks, x, y, z, BlockId::Air);
            }
            let h = hash_coords_3d(x, base_y, z, seed);
            set(blocks, x, base_y, z, if h % 2 == 0 { BlockId::Ice } else { BlockId::Stone });
        }
    }

    // Broken ice & stone walls
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
            if !is_border { continue; }
            if (x == center_x || x == center_x + 1) && z == min_z { continue; }
            let wall_h = (hash_coords_3d(x, 101, z, seed) % 4) as i32;
            for dy in 1..=wall_h {
                let b = if (x + z + dy) % 2 == 0 { BlockId::Ice } else { BlockId::Stone };
                set(blocks, x, base_y + dy, z, b);
            }
        }
    }

    // Trapped frozen chest and ice shrine
    set(blocks, center_x, base_y + 1, center_z, BlockId::Chest);
    set(blocks, center_x - 1, base_y + 1, center_z, BlockId::Ice);
    set(blocks, center_x + 1, base_y + 1, center_z, BlockId::Ice);
    set(blocks, center_x, base_y + 1, center_z - 1, BlockId::Ice);
    set(blocks, center_x, base_y + 1, center_z + 1, BlockId::Ice);
    set(blocks, center_x, base_y + 2, center_z + 1, BlockId::Snow);
}

fn place_desert_buried_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, _seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    // Hollow out chamber
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 3)..base_y {
                set(blocks, x, y, z, BlockId::Sand);
            }
            for y in (base_y + 1)..=(base_y + 4) {
                set(blocks, x, y, z, BlockId::Air);
            }
            set(blocks, x, base_y, z, BlockId::Sand);
        }
    }

    // Outer walls
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
            if is_border {
                for dy in 1..=4 {
                    set(blocks, x, base_y + dy, z, BlockId::Sand);
                }
            }
        }
    }

    // Roof layer with broken skylight
    for x in (min_x + 1)..=(max_x - 1) {
        for z in (min_z + 1)..=(max_z - 1) {
            set(blocks, x, base_y + 4, z, BlockId::Sand);
        }
    }
    set(blocks, center_x, base_y + 4, center_z, BlockId::Air);
    set(blocks, center_x + 1, base_y + 4, center_z, BlockId::Air);

    // Crypt pit with TNT and chest
    set(blocks, center_x, base_y - 1, center_z, BlockId::Tnt);
    set(blocks, center_x, base_y, center_z, BlockId::Air);
    set(blocks, center_x + 1, base_y + 1, center_z, BlockId::Chest);
    set(blocks, center_x - 1, base_y + 2, center_z, BlockId::Torch);
}

fn place_overgrown_temple_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in (base_y - 3)..base_y {
                set(blocks, x, y, z, BlockId::Stone);
            }
            for y in (base_y + 1)..=(base_y + 5) {
                set(blocks, x, y, z, BlockId::Air);
            }
            let h = hash_coords_3d(x, base_y, z, seed);
            let floor_b = if h % 3 == 0 { BlockId::Mud } else { BlockId::Stone };
            set(blocks, x, base_y, z, floor_b);
        }
    }

    // Overgrown stone pillars and arches
    for &(cx, cz) in &[(min_x, min_z), (min_x, max_z), (max_x, min_z), (max_x, max_z)] {
        for dy in 1..=4 {
            set(blocks, cx, base_y + dy, cz, BlockId::Stone);
        }
        set(blocks, cx, base_y + 4, cz, BlockId::Leaves);
        set(blocks, cx, base_y + 3, cz, BlockId::Leaves);
    }

    // Cross beam with leaves
    for x in min_x..=max_x {
        if hash_coords_3d(x, 202, center_z, seed) % 2 == 0 {
            set(blocks, x, base_y + 4, center_z, BlockId::Stone);
            set(blocks, x, base_y + 3, center_z, BlockId::Leaves);
        }
    }

    // Altar with chest
    set(blocks, center_x, base_y + 1, center_z, BlockId::Stone);
    set(blocks, center_x, base_y + 2, center_z, BlockId::Chest);
    set(blocks, center_x - 1, base_y + 1, center_z, BlockId::CraftingTable);
    set(blocks, center_x + 1, base_y + 2, center_z, BlockId::Torch);
}

fn place_cave_dungeon_ruin(blocks: &mut [u8], center_x: i32, base_y: i32, center_z: i32, seed: u32) {
    let min_x = center_x - 3;
    let max_x = center_x + 3;
    let min_z = center_z - 3;
    let max_z = center_z + 3;

    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    // Hollow out dungeon room
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            set(blocks, x, base_y, z, BlockId::Stone);
            for y in (base_y + 1)..=(base_y + 4) {
                set(blocks, x, y, z, BlockId::Air);
            }
            set(blocks, x, base_y + 5, z, BlockId::Stone);
        }
    }

    // Dungeon walls with ore veins and cracks
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
            if is_border {
                for dy in 1..=4 {
                    let h = hash_coords_3d(x, base_y + dy, z, seed);
                    let b = if h % 17 == 0 {
                        BlockId::IronOre
                    } else if h % 23 == 0 {
                        BlockId::Air
                    } else {
                        BlockId::Stone
                    };
                    set(blocks, x, base_y + dy, z, b);
                }
            }
        }
    }

    // Dungeon chest on pedestal, surrounded by torches
    set(blocks, center_x, base_y + 1, center_z, BlockId::Stone);
    set(blocks, center_x, base_y + 2, center_z, BlockId::Chest);
    set(blocks, center_x - 1, base_y + 1, center_z, BlockId::Torch);
    set(blocks, center_x + 1, base_y + 1, center_z, BlockId::Torch);
}

#[derive(Clone)]
pub struct PrimedTnt {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
}

pub struct World {
    pub name: String,
    pub gamemode: crate::gameplay::player::GameMode,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub seed: u32,
    pub player_position: Vec3,
    fluid_levels: HashMap<IVec3, u8>,
    pub active_sand: HashSet<IVec3>,
    pub chests: HashMap<IVec3, ChestData>,
    pub furnaces: HashMap<IVec3, FurnaceData>,
    pub saved_player: Option<crate::gameplay::player::SavedPlayerData>,
    pub sim_tick: u64,
    pub time_of_day: f32,
    pub torches: HashSet<IVec3>,
    pub pending_explosions: Vec<Vec3>,
    pub primed_tnts: Vec<PrimedTnt>,
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

impl World {
    pub fn new() -> Self {
        Self::new_with_seed(DEFAULT_WORLD_SEED)
    }

    pub fn get_or_create_chest(&mut self, pos: IVec3) -> &mut ChestData {
        let seed = self.seed;
        self.chests.entry(pos).or_insert_with(|| ChestData::generate_house_loot(seed, pos))
    }

    pub fn get_or_create_furnace(&mut self, pos: IVec3) -> &mut FurnaceData {
        self.furnaces.entry(pos).or_insert_with(FurnaceData::new)
    }

    pub fn update_furnaces(&mut self, dt: f32) {
        let mut state_changes = Vec::new();
        for (&pos, furnace) in &mut self.furnaces {
            let changed = furnace.tick(dt);
            if changed {
                let new_block = if furnace.is_burning() {
                    BlockId::FurnaceLit
                } else {
                    BlockId::Furnace
                };
                state_changes.push((pos, new_block));
            }
        }
        for (pos, new_block) in state_changes {
            if matches!(self.get_block(pos), BlockId::Furnace | BlockId::FurnaceLit) {
                self.set_block(pos, new_block);
            }
        }
    }

    /// Searches for a safe player spawn position, avoiding deep oceans, open water, and swamps.
    pub fn find_spawn_position(seed: u32) -> Vec3 {
        let mut x = 0i32;
        let mut z = 0i32;
        let mut dx = 0i32;
        let mut dz = -1i32;
        let step = 16i32;
        let max_steps = 1500;

        for _ in 0..max_steps {
            let wx = x + 8;
            let wz = z + 8;
            let (biome, cont) = get_biome_at(wx, wz, seed);
            if !matches!(biome, Biome::DeepOcean | Biome::Ocean | Biome::Swamp | Biome::MushroomIsland) {
                let height = get_height_at(wx, wz, seed, biome, cont);
                if height >= 24 {
                    return Vec3::new(wx as f32 + 0.5, (height + 3) as f32 + 0.1, wz as f32 + 0.5);
                }
            }

            if x == z || (x < 0 && x == -z) || (x > 0 && x == 1 - z) {
                let temp = dx;
                dx = -dz;
                dz = temp;
            }
            x += dx * step;
            z += dz * step;
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
                #[cfg(target_os = "vita")]
                { 2 }
                #[cfg(not(target_os = "vita"))]
                {
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(4)
                        .clamp(2, 6)
                }
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
                            let thid = sceKernelGetThreadId();
                            // Pin thread to Core 1 (0x20000) or Core 2 (0x40000), freeing Core 0 for rendering
                            let mask = if i % 2 == 0 { 0x00020000 } else { 0x00040000 };
                            sceKernelChangeThreadCpuAffinityMask(thid, mask);
                        }

                        loop {
                            let job = {
                                if let Ok(rx) = req_rx_clone.lock() {
                                    rx.recv().ok()
                                } else {
                                    None
                                }
                            };
                            match job {
                                Some((cx, cz, chunk_seed)) => {
                                    crate::log_debug!("CHUNK", "Worker generating chunk ({}, {}) [seed={}]", cx, cz, chunk_seed);
                                    let chunk = Chunk::generate(cx, cz, chunk_seed);
                                    if res_tx_clone.send(chunk).is_err() {
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    });
            }

            (Some(req_tx), Some(std::sync::Mutex::new(res_rx)))
        };

        #[cfg(any(target_arch = "wasm32", feature = "xbox360"))]
        let (chunk_req_tx, chunk_res_rx) = (None, None);

        let spawn_pos = Self::find_spawn_position(seed);
        let spawn_cx = (spawn_pos.x.floor() as i32) >> 4;
        let spawn_cz = (spawn_pos.z.floor() as i32) >> 4;
        crate::log_info!("WORLD", "Spawn point found at ({:.1}, {:.1}, {:.1}) [chunk ({}, {})]", spawn_pos.x, spawn_pos.y, spawn_pos.z, spawn_cx, spawn_cz);

        let mut chunks = HashMap::new();
        let radius: i32 = {
            #[cfg(any(target_os = "xbox360", target_os = "xenon", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
            { 1 }
            #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_os = "vita", target_arch = "powerpc", feature = "xbox360")))]
            { 2 }
        };
        let total_chunks = (radius * 2 + 1) * (radius * 2 + 1);
        crate::log_info!("WORLD", "Pregenerating initial {}x{} spawn chunks around ({}, {})", radius * 2 + 1, radius * 2 + 1, spawn_cx, spawn_cz);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let cx = spawn_cx + dx;
                let cz = spawn_cz + dz;
                let chunk = Chunk::generate(cx, cz, seed);
                chunks.insert((cx, cz), chunk);
                crate::log_info!("WORLD", "Spawn chunk ({}, {}) ready [{}/{}]", cx, cz, chunks.len(), total_chunks);
            }
        }
        crate::log_info!("WORLD", "Pregeneration completed ({} chunks loaded in memory)", chunks.len());

        World {
            name: "New World".to_string(),
            gamemode: crate::gameplay::player::GameMode::Survival,
            chunks,
            seed,
            player_position: spawn_pos,
            fluid_levels: HashMap::new(),
            active_sand: HashSet::new(),
            chests: HashMap::new(),
            furnaces: HashMap::new(),
            saved_player: None,
            sim_tick: 0,
            time_of_day: 120.0, // Start around 9:00 AM bright daylight
            torches: HashSet::new(),
            pending_explosions: Vec::new(),
            primed_tnts: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            chunk_req_tx,
            #[cfg(not(target_arch = "wasm32"))]
            chunk_res_rx,
            pending_chunks: HashSet::new(),
            last_stream_chunk: None,
            last_stream_dist: None,
        }
    }

    pub const DAY_DURATION: f32 = 480.0;

    #[inline]
    pub fn pending_chunks_count(&self) -> usize {
        self.pending_chunks.len()
    }

    #[inline]
    pub fn advance_time(&mut self, dt: f32) {
        self.time_of_day = (self.time_of_day + dt).rem_euclid(Self::DAY_DURATION);
    }

    #[inline]
    pub fn day_fraction(&self) -> f32 {
        self.time_of_day / Self::DAY_DURATION
    }

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
        let a = self.sun_angle();
        let elev = a.sin(); // -1.0 to 1.0
        // Smooth transition around horizon (-0.2 to 0.2)
        let t = ((elev + 0.2) / 0.4).clamp(0.0, 1.0);
        let ease = t * t * (3.0 - 2.0 * t); // smoothstep
        0.08 + 0.92 * ease
    }

    /// Returns (sky_top, sky_bottom, fog_color) based on current time of day
    pub fn sky_colors(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let elev = self.sun_angle().sin();

        // Day palette
        let day_top = [0.32, 0.54, 0.96];
        let day_bot = [0.72, 0.84, 0.98];
        let day_fog = [0.72, 0.84, 0.98];

        // Sunset/sunrise horizon palette
        let sunset_top = [0.24, 0.28, 0.58];
        let sunset_bot = [0.98, 0.48, 0.18];
        let sunset_fog = [0.92, 0.52, 0.25];

        // Night palette (deep indigo night sky)
        let night_top = [0.015, 0.025, 0.07];
        let night_bot = [0.05, 0.07, 0.15];
        let night_fog = [0.04, 0.06, 0.12];

        let day_t = ((elev + 0.25) / 0.50).clamp(0.0, 1.0);
        let day_smooth = day_t * day_t * (3.0 - 2.0 * day_t);

        let sunset_factor = (1.0 - (elev.abs() / 0.28)).max(0.0);
        let sunset_smooth = sunset_factor * sunset_factor;

        let lerp3 = |a: [f32; 3], b: [f32; 3], t: f32| [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ];

        let base_top = lerp3(night_top, day_top, day_smooth);
        let base_bot = lerp3(night_bot, day_bot, day_smooth);
        let base_fog = lerp3(night_fog, day_fog, day_smooth);

        let final_top = lerp3(base_top, sunset_top, sunset_smooth * 0.7);
        let final_bot = lerp3(base_bot, sunset_bot, sunset_smooth);
        let final_fog = lerp3(base_fog, sunset_fog, sunset_smooth);

        (final_top, final_bot, final_fog)
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        crate::log_info!("SAVE", "Serializing world '{}' (seed: {}, mode: {:?}, chunks: {}, chests: {}, furnaces: {}) to '{}'", self.name, self.seed, self.gamemode, self.chunks.len(), self.chests.len(), self.furnaces.len(), path);
        let mut bytes = Vec::with_capacity(32 + self.chunks.len() * 4);
        bytes.extend_from_slice(b"RUSTCRAFT6");
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        let mode_byte: u8 = match self.gamemode {
            crate::gameplay::player::GameMode::Survival => 0,
            crate::gameplay::player::GameMode::Creative => 1,
        };
        bytes.push(mode_byte);
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(255) as u8;
        bytes.push(name_len);
        bytes.extend_from_slice(&name_bytes[..name_len as usize]);
        bytes.extend_from_slice(&self.player_position.x.to_le_bytes());
        bytes.extend_from_slice(&self.player_position.y.to_le_bytes());
        bytes.extend_from_slice(&self.player_position.z.to_le_bytes());
        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());
        for chunk in self.chunks.values() {
            bytes.extend_from_slice(&chunk.cx.to_le_bytes());
            bytes.extend_from_slice(&chunk.cz.to_le_bytes());
            bytes.extend_from_slice(&(chunk.blocks.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&chunk.blocks);
        }
        bytes.extend_from_slice(&(self.fluid_levels.len() as u32).to_le_bytes());
        for (pos, level) in &self.fluid_levels {
            bytes.extend_from_slice(&pos.x.to_le_bytes());
            bytes.extend_from_slice(&pos.y.to_le_bytes());
            bytes.extend_from_slice(&pos.z.to_le_bytes());
            bytes.push(*level);
        }
        // Save chests
        bytes.extend_from_slice(&(self.chests.len() as u32).to_le_bytes());
        for (pos, chest) in &self.chests {
            bytes.extend_from_slice(&pos.x.to_le_bytes());
            bytes.extend_from_slice(&pos.y.to_le_bytes());
            bytes.extend_from_slice(&pos.z.to_le_bytes());
            for item in &chest.items {
                if let Some(stack) = item {
                    bytes.push(stack.block.to_u8());
                    bytes.push(stack.count);
                } else {
                    bytes.push(0);
                    bytes.push(0);
                }
            }
        }
        // Save furnaces
        bytes.extend_from_slice(&(self.furnaces.len() as u32).to_le_bytes());
        for (pos, furnace) in &self.furnaces {
            bytes.extend_from_slice(&pos.x.to_le_bytes());
            bytes.extend_from_slice(&pos.y.to_le_bytes());
            bytes.extend_from_slice(&pos.z.to_le_bytes());
            for stack_opt in [&furnace.input, &furnace.fuel, &furnace.output] {
                if let Some(stack) = stack_opt {
                    bytes.push(stack.block.to_u8());
                    bytes.push(stack.count);
                } else {
                    bytes.push(0);
                    bytes.push(0);
                }
            }
            bytes.extend_from_slice(&furnace.burn_time_remaining.to_le_bytes());
            bytes.extend_from_slice(&furnace.burn_time_total.to_le_bytes());
            bytes.extend_from_slice(&furnace.cook_progress.to_le_bytes());
        }
        // Save player state
        if let Some(ref p) = self.saved_player {
            bytes.push(1);
            bytes.extend_from_slice(&p.health.to_le_bytes());
            bytes.extend_from_slice(&p.max_health.to_le_bytes());
            bytes.extend_from_slice(&p.hunger.to_le_bytes());
            bytes.extend_from_slice(&p.max_hunger.to_le_bytes());
            bytes.extend_from_slice(&p.saturation.to_le_bytes());
            bytes.extend_from_slice(&p.exhaustion.to_le_bytes());
            bytes.extend_from_slice(&p.air.to_le_bytes());
            bytes.push(p.selected_hotbar_slot);
            for item in &p.hotbar {
                if let Some(stack) = item {
                    bytes.push(stack.block.to_u8());
                    bytes.push(stack.count);
                } else {
                    bytes.push(0);
                    bytes.push(0);
                }
            }
            for item in &p.main {
                if let Some(stack) = item {
                    bytes.push(stack.block.to_u8());
                    bytes.push(stack.count);
                } else {
                    bytes.push(0);
                    bytes.push(0);
                }
            }
        } else {
            bytes.push(0);
        }
        std::fs::write(path, bytes).map_err(|error| format!("failed to save world: {error}"))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        crate::log_info!("SAVE", "Loading world from file '{}'...", path);
        let bytes = std::fs::read(path).map_err(|error| format!("failed to read world file: {error}"))?;
        let mut cursor = 0usize;
        let format = read_bytes(&bytes, &mut cursor, 10)?;
        if format != b"RUSTCRAFT1" && format != b"RUSTCRAFT2" && format != b"RUSTCRAFT3" && format != b"RUSTCRAFT4" && format != b"RUSTCRAFT5" && format != b"RUSTCRAFT6" {
            return Err("unknown world file format".to_owned());
        }
        let seed = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
        let (gamemode, name) = if format == b"RUSTCRAFT4" || format == b"RUSTCRAFT5" || format == b"RUSTCRAFT6" {
            let gm_byte = read_bytes(&bytes, &mut cursor, 1)?[0];
            let gm = if gm_byte == 1 { crate::gameplay::player::GameMode::Creative } else { crate::gameplay::player::GameMode::Survival };
            let name_len = read_bytes(&bytes, &mut cursor, 1)?[0] as usize;
            let name_str = if name_len > 0 {
                String::from_utf8_lossy(read_bytes(&bytes, &mut cursor, name_len)?).to_string()
            } else {
                format!("World {}", seed)
            };
            (gm, name_str)
        } else {
            (crate::gameplay::player::GameMode::Survival, format!("World {}", seed))
        };
        let player_position = if format == b"RUSTCRAFT2" || format == b"RUSTCRAFT3" || format == b"RUSTCRAFT4" || format == b"RUSTCRAFT5" || format == b"RUSTCRAFT6" {
            Vec3::new(
                f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
            )
        } else {
            Self::find_spawn_position(seed)
        };
        let chunk_count = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
        let expected_len = (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize;
        let mut world = Self::new_with_seed(seed);
        world.name = name;
        world.gamemode = gamemode;
        world.player_position = player_position;
        for _ in 0..chunk_count {
            let cx = i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
            let cz = i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
            let block_len = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
            if block_len != expected_len {
                return Err("invalid chunk data length in world file".to_owned());
            }
            let blocks = read_bytes(&bytes, &mut cursor, block_len)?.to_vec();
            world.chunks.insert((cx, cz), Chunk { cx, cz, blocks, dirty: true });
        }
        if format == b"RUSTCRAFT3" || format == b"RUSTCRAFT4" || format == b"RUSTCRAFT5" || format == b"RUSTCRAFT6" {
            let fluid_count = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
            for _ in 0..fluid_count {
                let pos = IVec3::new(
                    i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                    i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                    i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                );
                let level = read_bytes(&bytes, &mut cursor, 1)?[0].clamp(1, 8);
                world.fluid_levels.insert(pos, level);
            }
        }
        if (format == b"RUSTCRAFT5" || format == b"RUSTCRAFT6") && bytes.len() > cursor {
            if let Ok(chest_count_bytes) = read_bytes(&bytes, &mut cursor, 4) {
                let chest_count = u32::from_le_bytes(chest_count_bytes.try_into().unwrap_or_default()) as usize;
                for _ in 0..chest_count {
                    if let (Ok(x_bytes), Ok(y_bytes), Ok(z_bytes)) = (
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                    ) {
                        let pos = IVec3::new(
                            i32::from_le_bytes(x_bytes.try_into().unwrap_or_default()),
                            i32::from_le_bytes(y_bytes.try_into().unwrap_or_default()),
                            i32::from_le_bytes(z_bytes.try_into().unwrap_or_default()),
                        );
                        let mut chest = ChestData::new();
                        for slot in &mut chest.items {
                            if let Ok(item_bytes) = read_bytes(&bytes, &mut cursor, 2) {
                                let block_id = item_bytes[0];
                                let count = item_bytes[1];
                                if block_id > 0 && count > 0 {
                                    *slot = Some(crate::gameplay::inventory::ItemStack {
                                        block: BlockId::from_u8(block_id),
                                        count,
                                    });
                                }
                            }
                        }
                        world.chests.insert(pos, chest);
                    }
                }
            }
            if let Ok(furnace_count_bytes) = read_bytes(&bytes, &mut cursor, 4) {
                let furnace_count = u32::from_le_bytes(furnace_count_bytes.try_into().unwrap_or_default()) as usize;
                for _ in 0..furnace_count {
                    if let (Ok(x_bytes), Ok(y_bytes), Ok(z_bytes)) = (
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                    ) {
                        let pos = IVec3::new(
                            i32::from_le_bytes(x_bytes.try_into().unwrap_or_default()),
                            i32::from_le_bytes(y_bytes.try_into().unwrap_or_default()),
                            i32::from_le_bytes(z_bytes.try_into().unwrap_or_default()),
                        );
                        let mut furnace = FurnaceData::new();
                        for slot in [&mut furnace.input, &mut furnace.fuel, &mut furnace.output] {
                            if let Ok(item_bytes) = read_bytes(&bytes, &mut cursor, 2) {
                                let block_id = item_bytes[0];
                                let count = item_bytes[1];
                                if block_id > 0 && count > 0 {
                                    *slot = Some(crate::gameplay::inventory::ItemStack {
                                        block: BlockId::from_u8(block_id),
                                        count,
                                    });
                                }
                            }
                        }
                        if let (Ok(rem), Ok(tot), Ok(prog)) = (
                            read_bytes(&bytes, &mut cursor, 4),
                            read_bytes(&bytes, &mut cursor, 4),
                            read_bytes(&bytes, &mut cursor, 4),
                        ) {
                            furnace.burn_time_remaining = f32::from_le_bytes(rem.try_into().unwrap_or_default());
                            furnace.burn_time_total = f32::from_le_bytes(tot.try_into().unwrap_or_default());
                            furnace.cook_progress = f32::from_le_bytes(prog.try_into().unwrap_or_default());
                        }
                        world.furnaces.insert(pos, furnace);
                    }
                }
            }
        }
        if format == b"RUSTCRAFT6" && bytes.len() > cursor {
            if let Ok(has_p) = read_bytes(&bytes, &mut cursor, 1) {
                if has_p[0] == 1 {
                    if let (Ok(h), Ok(mh), Ok(hg), Ok(mhg), Ok(sat), Ok(exh), Ok(air), Ok(slot)) = (
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 4),
                        read_bytes(&bytes, &mut cursor, 1),
                    ) {
                        let mut hotbar = [None; 10];
                        for slot_item in &mut hotbar {
                            if let Ok(ib) = read_bytes(&bytes, &mut cursor, 2) {
                                if ib[0] > 0 && ib[1] > 0 {
                                    *slot_item = Some(crate::gameplay::inventory::ItemStack {
                                        block: BlockId::from_u8(ib[0]),
                                        count: ib[1],
                                    });
                                }
                            }
                        }
                        let mut main = [None; 27];
                        for slot_item in &mut main {
                            if let Ok(ib) = read_bytes(&bytes, &mut cursor, 2) {
                                if ib[0] > 0 && ib[1] > 0 {
                                    *slot_item = Some(crate::gameplay::inventory::ItemStack {
                                        block: BlockId::from_u8(ib[0]),
                                        count: ib[1],
                                    });
                                }
                            }
                        }
                        world.saved_player = Some(crate::gameplay::player::SavedPlayerData {
                            health: f32::from_le_bytes(h.try_into().unwrap_or_default()),
                            max_health: f32::from_le_bytes(mh.try_into().unwrap_or_default()),
                            hunger: f32::from_le_bytes(hg.try_into().unwrap_or_default()),
                            max_hunger: f32::from_le_bytes(mhg.try_into().unwrap_or_default()),
                            saturation: f32::from_le_bytes(sat.try_into().unwrap_or_default()),
                            exhaustion: f32::from_le_bytes(exh.try_into().unwrap_or_default()),
                            air: f32::from_le_bytes(air.try_into().unwrap_or_default()),
                            selected_hotbar_slot: slot[0],
                            hotbar,
                            main,
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
        let cx = wx >> 4;
        let cz = wz >> 4;
        let lx = wx & 15;
        let lz = wz & 15;
        (cx, cz, lx, lz)
    }

    #[inline(always)]
    pub fn get_block(&self, pos: IVec3) -> BlockId {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => chunk.get_local(lx, pos.y, lz),
            None => BlockId::Air,
        }
    }

    /// Checks if a leaf block at `start` has a valid connection to a Wood block within `max_dist` steps.
    pub fn is_leaf_supported(&self, start: IVec3, max_dist: i32) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((pos, dist)) = queue.pop_front() {
            let block = self.get_block(pos);
            if block.is_wood() {
                return true;
            }
            if dist < max_dist {
                for offset in [
                    IVec3::new(1, 0, 0),
                    IVec3::new(-1, 0, 0),
                    IVec3::new(0, 1, 0),
                    IVec3::new(0, -1, 0),
                    IVec3::new(0, 0, 1),
                    IVec3::new(0, 0, -1),
                ] {
                    let neighbor = pos + offset;
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        let n_block = self.get_block(neighbor);
                        if n_block.is_wood() {
                            return true;
                        }
                        if n_block.is_leaves() {
                            queue.push_back((neighbor, dist + 1));
                        }
                    }
                }
            }
        }
        false
    }

    /// Finds all leaves in the surrounding canopy that no longer have a connection to any Wood block.
    pub fn find_unsupported_leaves_around(&self, center: IVec3) -> Vec<IVec3> {
        let mut unsupported = Vec::new();
        for dx in -5..=5 {
            for dz in -5..=5 {
                for dy in -5..=10 {
                    let leaf_pos = center + IVec3::new(dx, dy, dz);
                    if self.get_block(leaf_pos).is_leaves() {
                        if !self.is_leaf_supported(leaf_pos, 5) {
                            unsupported.push(leaf_pos);
                        }
                    }
                }
            }
        }
        unsupported
    }

    /// Returns the biome name at world coordinates (wx, wz) using the same
    /// logic as chunk generation.
    pub fn biome_name_at(&self, wx: i32, wz: i32) -> &'static str {
        let (biome, _) = get_biome_at(wx, wz, self.seed);
        biome.name()
    }

    /// Checks whether the camera view frustum is completely blocked by an opaque solid block immediately in front.
    #[inline]
    pub fn is_view_fully_occluded(&self, camera_pos: crate::core::math::Vec3, yaw: f32, pitch: f32) -> bool {
        let eye_block = self.get_block(camera_pos.floor_to_ivec());
        if eye_block.is_opaque() {
            return true;
        }

        let (sin_y, cos_y) = yaw.sin_cos();
        let (sin_p, cos_p) = pitch.sin_cos();
        let forward = crate::core::math::Vec3::new(sin_y * cos_p, -sin_p, cos_y * cos_p);
        let right = crate::core::math::Vec3::new(cos_y, 0.0, -sin_y);
        let up = crate::core::math::Vec3::new(sin_y * sin_p, cos_p, cos_y * sin_p);

        let test_dist = 0.38f32;
        let spread_x = 0.26f32;
        let spread_y = 0.20f32;

        let center = camera_pos + forward * test_dist;
        let tl = center - right * spread_x + up * spread_y;
        let tr = center + right * spread_x + up * spread_y;
        let bl = center - right * spread_x - up * spread_y;
        let br = center + right * spread_x - up * spread_y;

        self.get_block(center.floor_to_ivec()).is_opaque()
            && self.get_block(tl.floor_to_ivec()).is_opaque()
            && self.get_block(tr.floor_to_ivec()).is_opaque()
            && self.get_block(bl.floor_to_ivec()).is_opaque()
            && self.get_block(br.floor_to_ivec()).is_opaque()
    }

    #[inline(always)]
    pub fn fluid_level(&self, pos: IVec3) -> u8 {
        self.fluid_levels.get(&pos).copied().unwrap_or(8)
    }

    fn set_fluid_level(&mut self, pos: IVec3, level: u8) {
        self.fluid_levels.insert(pos, level.clamp(1, 8));
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockId) {
        if pos.y < 0 || pos.y >= CHUNK_SIZE_Y {
            return;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if self.chunks.contains_key(&(cx, cz)) {
            self.chunks.get_mut(&(cx, cz)).unwrap().set_local(lx, pos.y, lz, block);
            if matches!(block, BlockId::Water | BlockId::Lava) {
                self.set_fluid_level(pos, 8);
            } else {
                self.fluid_levels.remove(&pos);
            }

            if block == BlockId::Sand {
                self.active_sand.insert(pos);
            } else {
                self.active_sand.remove(&pos);
            }

            if block == BlockId::Torch {
                self.torches.insert(pos);
            } else {
                self.torches.remove(&pos);
            }

            if block == BlockId::Chest {
                self.chests.entry(pos).or_insert_with(ChestData::new);
            } else {
                self.chests.remove(&pos);
            }

            if matches!(block, BlockId::Furnace | BlockId::FurnaceLit) {
                self.furnaces.entry(pos).or_insert_with(FurnaceData::new);
            } else {
                self.furnaces.remove(&pos);
            }

            for offset in [
                IVec3::new(1, 0, 0),
                IVec3::new(-1, 0, 0),
                IVec3::new(0, 1, 0),
                IVec3::new(0, -1, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(0, 0, -1),
            ] {
                let neighbor = pos + offset;
                if self.get_block(neighbor) == BlockId::Sand {
                    self.active_sand.insert(neighbor);
                }
            }

            if lx == 0 {
                self.mark_dirty(cx - 1, cz);
            }
            if lx == CHUNK_SIZE_X - 1 {
                self.mark_dirty(cx + 1, cz);
            }
            if lz == 0 {
                self.mark_dirty(cx, cz - 1);
            }
            if lz == CHUNK_SIZE_Z - 1 {
                self.mark_dirty(cx, cz + 1);
            }
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

    pub fn update_primed_tnts(&mut self, dt: f32) {
        let mut exploded_positions = Vec::new();

        for i in 0..self.primed_tnts.len() {
            self.primed_tnts[i].fuse -= dt;
            self.primed_tnts[i].velocity.y -= 20.0 * dt;

            let dy = self.primed_tnts[i].velocity.y * dt;
            let mut new_pos = self.primed_tnts[i].pos;
            new_pos.y += dy;

            let box_min = Vec3::new(new_pos.x - 0.49, new_pos.y, new_pos.z - 0.49);
            let box_max = Vec3::new(new_pos.x + 0.49, new_pos.y + 0.99, new_pos.z + 0.49);

            if self.box_collides(box_min, box_max) {
                self.primed_tnts[i].velocity = Vec3::zero();
                self.primed_tnts[i].pos.y = self.primed_tnts[i].pos.y.floor();
            } else {
                self.primed_tnts[i].pos = new_pos;
            }

            if self.primed_tnts[i].fuse <= 0.0 {
                exploded_positions.push(IVec3::new(
                    self.primed_tnts[i].pos.x.floor() as i32,
                    self.primed_tnts[i].pos.y.floor() as i32,
                    self.primed_tnts[i].pos.z.floor() as i32,
                ));
            }
        }

        self.primed_tnts.retain(|tnt| tnt.fuse > 0.0);

        for center in exploded_positions {
            self.explode(center, 3.5);
        }
    }

    /// Grows a full tree at the specified position.
    pub fn grow_tree_at(&mut self, pos: IVec3) -> bool {
        let base_y = pos.y;
        let lx = pos.x;
        let lz = pos.z;
        let trunk_height = 4 + (pos.x.wrapping_mul(7) ^ pos.z.wrapping_mul(13)).rem_euclid(2);
        for i in 0..trunk_height {
            self.set_block(IVec3::new(lx, base_y + i, lz), BlockId::Wood);
        }
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..=2 {
                    let p = IVec3::new(lx + dx, base_y + trunk_height - 1 + dy, lz + dz);
                    let dist = dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2;
                    if dist <= 6 {
                        if self.get_block(p) == BlockId::Air {
                            self.set_block(p, BlockId::Leaves);
                        }
                    }
                }
            }
        }
        true
    }

    /// Random tick for nearby farmland hydration and sapling growth
    pub fn tick_farming(&mut self, player_pos: Vec3, rng_seed: &mut u32) {
        let pcx = (player_pos.x.floor() as i32) >> 4;
        let pcz = (player_pos.z.floor() as i32) >> 4;

        let mut next_rand = || -> u32 {
            *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *rng_seed
        };

        for _ in 0..4 {
            let dcx = (next_rand() % 5) as i32 - 2;
            let dcz = (next_rand() % 5) as i32 - 2;
            let cx = pcx + dcx;
            let cz = pcz + dcz;

            if self.chunks.contains_key(&(cx, cz)) {
                for _ in 0..3 {
                    let rx = (next_rand() % 16) as i32;
                    let rz = (next_rand() % 16) as i32;
                    let ry = 20 + (next_rand() % 35) as i32;
                    let wx = cx * 16 + rx;
                    let wz = cz * 16 + rz;
                    let pos = IVec3::new(wx, ry, wz);
                    let blk = self.get_block(pos);

                    if blk == BlockId::Farmland {
                        let mut has_water = false;
                        for wx2 in -4..=4 {
                            for wz2 in -4..=4 {
                                for wy2 in 0..=1 {
                                    if self.get_block(pos + IVec3::new(wx2, wy2, wz2)) == BlockId::Water {
                                        has_water = true;
                                        break;
                                    }
                                }
                                if has_water { break; }
                            }
                            if has_water { break; }
                        }
                        if has_water {
                            self.set_block(pos, BlockId::FarmlandWet);
                        } else {
                            let above = pos + IVec3::new(0, 1, 0);
                            if !matches!(self.get_block(above), BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop) {
                                if next_rand() % 6 == 0 {
                                    self.set_block(pos, BlockId::Dirt);
                                }
                            }
                        }
                    } else if blk == BlockId::Sapling {
                        if next_rand() % 35 == 0 {
                            self.set_block(pos, BlockId::Air);
                            self.grow_tree_at(pos);
                        }
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
                    let dist_sq = (dx * dx + dy * dy + dz * dz) as f32;
                    if dist_sq <= r_sq {
                        let target_pos = center + IVec3::new(dx, dy, dz);
                        let b = self.get_block(target_pos);
                        if b != BlockId::Air && !b.is_bedrock() {
                            if b == BlockId::Tnt {
                                tnts_to_prime.push(target_pos);
                            } else {
                                self.set_block(target_pos, BlockId::Air);
                            }
                        }
                    }
                }
            }
        }

        self.pending_explosions.push(Vec3::new(
            center.x as f32 + 0.5,
            center.y as f32 + 0.5,
            center.z as f32 + 0.5,
        ));

        for tnt_pos in tnts_to_prime {
            self.prime_tnt(tnt_pos);
        }
    }

    pub fn create_mesh_job(&self, key: (i32, i32)) -> Option<ChunkMeshJob> {
        let chunk = self.chunks.get(&key)?;
        let (cx, cz) = key;
        let base_x = cx * CHUNK_SIZE_X;
        let base_z = cz * CHUNK_SIZE_Z;

        let mut lights = Vec::new();
        for &t_pos in &self.torches {
            let dx = (t_pos.x - (base_x + 8)).abs();
            let dz = (t_pos.z - (base_z + 8)).abs();
            if dx <= 24 && dz <= 24 {
                lights.push(LightSource {
                    pos: t_pos,
                    radius: 12.0,
                    intensity: 1.0,
                });
            }
        }

        Some(ChunkMeshJob {
            key,
            center_blocks: chunk.blocks.clone(),
            east_blocks: self.chunks.get(&(cx + 1, cz)).map(|c| c.blocks.clone()),
            west_blocks: self.chunks.get(&(cx - 1, cz)).map(|c| c.blocks.clone()),
            south_blocks: self.chunks.get(&(cx, cz + 1)).map(|c| c.blocks.clone()),
            north_blocks: self.chunks.get(&(cx, cz - 1)).map(|c| c.blocks.clone()),
            se_blocks: self.chunks.get(&(cx + 1, cz + 1)).map(|c| c.blocks.clone()),
            sw_blocks: self.chunks.get(&(cx - 1, cz + 1)).map(|c| c.blocks.clone()),
            ne_blocks: self.chunks.get(&(cx + 1, cz - 1)).map(|c| c.blocks.clone()),
            nw_blocks: self.chunks.get(&(cx - 1, cz - 1)).map(|c| c.blocks.clone()),
            seed: self.seed,
            lights,
            fluid_map: self.fluid_levels.clone(),
        })
    }

    fn mark_dirty(&mut self, cx: i32, cz: i32) {
        if let Some(c) = self.chunks.get_mut(&(cx, cz)) {
            c.dirty = true;
        }
    }

    pub fn update(&mut self, player_pos: Vec3, view_distance: i32) {
        let view_distance = view_distance.clamp(2, 16);
        let pcx = (player_pos.x.floor() as i32) >> 4;
        let pcz = (player_pos.z.floor() as i32) >> 4;

        let unload_dist = view_distance + 3;
        let unload_dist_sq = unload_dist * unload_dist;

        // Drain chunks generated asynchronously in the background threads
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(rx_mutex) = &self.chunk_res_rx {
            if let Ok(rx) = rx_mutex.lock() {
                while let Ok(chunk) = rx.try_recv() {
                    let cx = chunk.cx;
                    let cz = chunk.cz;
                    self.pending_chunks.remove(&(cx, cz));
                    let ddx = cx - pcx;
                    let ddz = cz - pcz;
                    if ddx * ddx + ddz * ddz <= unload_dist_sq {
                        self.chunks.insert((cx, cz), chunk);
                        crate::log_debug!("CHUNK", "Integrated chunk ({}, {}) into active world (loaded: {})", cx, cz, self.chunks.len());
                        if let Some(n) = self.chunks.get_mut(&(cx + 1, cz)) { n.dirty = true; }
                        if let Some(n) = self.chunks.get_mut(&(cx - 1, cz)) { n.dirty = true; }
                        if let Some(n) = self.chunks.get_mut(&(cx, cz + 1)) { n.dirty = true; }
                        if let Some(n) = self.chunks.get_mut(&(cx, cz - 1)) { n.dirty = true; }
                    }
                }
            }
        }

        if self.last_stream_chunk == Some((pcx, pcz))
            && self.last_stream_dist == Some(view_distance)
            && self.pending_chunks.is_empty()
        {
            return;
        }

        let mut missing_chunks = Vec::new();
        let view_dist_sq = view_distance * view_distance;

        for dz in -view_distance..=view_distance {
            for dx in -view_distance..=view_distance {
                if dx * dx + dz * dz <= view_dist_sq {
                    let cx = pcx + dx;
                    let cz = pcz + dz;
                    if !self.chunks.contains_key(&(cx, cz)) && !self.pending_chunks.contains(&(cx, cz)) {
                        missing_chunks.push((cx, cz));
                    }
                }
            }
        }

        if !missing_chunks.is_empty() {
            crate::log_debug!("CHUNK", "Requesting generation for {} missing chunks around player chunk ({}, {}) [view_dist={}]", missing_chunks.len(), pcx, pcz, view_distance);
            // Sort missing chunks so the closest chunks to the player generate first
            missing_chunks.sort_unstable_by_key(|&(cx, cz)| {
                let dx = cx - pcx;
                let dz = cz - pcz;
                dx * dx + dz * dz
            });

            #[cfg(all(not(target_arch = "wasm32"), not(feature = "xbox360")))]
            {
                if let Some(tx) = &self.chunk_req_tx {
                    for (cx, cz) in missing_chunks {
                        self.pending_chunks.insert((cx, cz));
                        let _ = tx.send((cx, cz, self.seed));
                    }
                }
            }

            #[cfg(any(target_arch = "wasm32", feature = "xbox360"))]
            {
                let count = missing_chunks.len().min(1);
                for (cx, cz) in missing_chunks.into_iter().take(count) {
                    let chunk = Chunk::generate(cx, cz, self.seed);
                    self.chunks.insert((cx, cz), chunk);
                    if let Some(n) = self.chunks.get_mut(&(cx + 1, cz)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx - 1, cz)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx, cz + 1)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx, cz - 1)) { n.dirty = true; }
                }
            }
        }

        if self.pending_chunks.is_empty() {
            self.last_stream_chunk = Some((pcx, pcz));
            self.last_stream_dist = Some(view_distance);
        } else {
            self.last_stream_chunk = None;
        }

        let count_before = self.chunks.len();
        self.chunks.retain(|&(cx, cz), _| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist_sq
        });
        self.pending_chunks.retain(|&(cx, cz)| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist_sq
        });
        if self.chunks.len() < count_before {
            crate::log_debug!("CHUNK", "Unloaded {} distant chunks (remaining active: {})", count_before - self.chunks.len(), self.chunks.len());
        }
    }

    pub fn mark_all_chunks_dirty(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.dirty = true;
        }
        crate::log_info!("WORLD", "Marked all {} chunks dirty for mesh regeneration", self.chunks.len());
    }

    pub fn purge_distant_chunks(&mut self, player_pos: Vec3, view_distance: i32) {
        let view_distance = view_distance.clamp(2, 16);
        let pcx = (player_pos.x.floor() as i32) >> 4;
        let pcz = (player_pos.z.floor() as i32) >> 4;
        let unload_dist = view_distance + 2;
        let unload_dist_sq = unload_dist * unload_dist;
        let before_count = self.chunks.len();

        self.chunks.retain(|&(cx, cz), _| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist_sq
        });
        self.pending_chunks.retain(|&(cx, cz)| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist_sq
        });

        for chunk in self.chunks.values_mut() {
            chunk.dirty = true;
        }

        self.last_stream_chunk = None;
        self.last_stream_dist = None;
        crate::log_info!("CHUNK", "Purged distant chunks: {} -> {} active (view distance: {} chunks)", before_count, self.chunks.len(), view_distance);
    }

    #[inline]
    fn get_block_cached<'a>(
        &'a self,
        pos: IVec3,
        hint: Option<((i32, i32), &'a Chunk)>,
    ) -> (BlockId, Option<((i32, i32), &'a Chunk)>) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return (BlockId::Air, hint);
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some((key, chunk)) = hint {
            if key == (cx, cz) {
                return (chunk.get_local(lx, pos.y, lz), Some((key, chunk)));
            }
        }
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => (chunk.get_local(lx, pos.y, lz), Some(((cx, cz), chunk))),
            None => (BlockId::Air, hint),
        }
    }

    pub fn simulate_materials(&mut self, player_pos: Vec3) {
        self.sim_tick = self.sim_tick.wrapping_add(1);
        let px = player_pos.x.floor() as i32;
        let py = player_pos.y.floor() as i32;
        let pz = player_pos.z.floor() as i32;
        let radius = 10i32;
        let min_y = (py - 12).max(1);
        let max_y = (py + 8).min(CHUNK_SIZE_Y - 1);

        let sim_lava = self.sim_tick % 3 == 0;
        let mut fluid_spawns: Vec<(IVec3, BlockId, u8)> = Vec::new();
        let mut fluid_despawns: Vec<IVec3> = Vec::new();
        let mut solidifications: Vec<(IVec3, BlockId)> = Vec::new();
        let mut tnts_to_prime = Vec::new();

        let mut hint: Option<((i32, i32), &Chunk)> = None;

        for y in (min_y..=max_y).rev() {
            for z in (pz - radius)..=(pz + radius) {
                for x in (px - radius)..=(px + radius) {
                    let pos = IVec3::new(x, y, z);
                    let (block, new_hint) = self.get_block_cached(pos, hint);
                    hint = new_hint;

                    if block == BlockId::Tnt {
                        let neighbors = [
                            IVec3::new(x + 1, y, z),
                            IVec3::new(x - 1, y, z),
                            IVec3::new(x, y + 1, z),
                            IVec3::new(x, y - 1, z),
                            IVec3::new(x, y, z + 1),
                            IVec3::new(x, y, z - 1),
                        ];
                        let mut found_lava = false;
                        for &p in &neighbors {
                            let (nblock, new_hint) = self.get_block_cached(p, hint);
                            hint = new_hint;
                            if nblock == BlockId::Lava {
                                found_lava = true;
                                break;
                            }
                        }
                        if found_lava {
                            tnts_to_prime.push(pos);
                        }
                        continue;
                    }

                    if block != BlockId::Water && block != BlockId::Lava {
                        continue;
                    }
                    if block == BlockId::Lava && !sim_lava {
                        continue;
                    }

                    let level = self.fluid_level(pos);
                    let below = IVec3::new(x, y - 1, z);
                    let below_block = self.get_block(below);

                    let cardinals = [
                        IVec3::new(x + 1, y, z),
                        IVec3::new(x - 1, y, z),
                        IVec3::new(x, y, z + 1),
                        IVec3::new(x, y, z - 1),
                        IVec3::new(x, y + 1, z),
                    ];

                    if block == BlockId::Lava {
                        let touches_water = below_block == BlockId::Water
                            || cardinals.iter().any(|&p| self.get_block(p) == BlockId::Water);
                        if touches_water {
                            solidifications.push((pos, BlockId::Stone));
                            continue;
                        }
                    }

                    if below.y >= 0 && below_block == BlockId::Air {
                        fluid_spawns.push((below, block, 8));
                    } else if block == BlockId::Water && below_block == BlockId::Lava {
                        solidifications.push((below, BlockId::Stone));
                    } else {
                        let step = if block == BlockId::Lava { 4 } else { 2 };
                        if level > step {
                            let new_level = level - step;
                            for &npos in &cardinals[0..4] {
                                let nblock = self.get_block(npos);
                                if nblock == BlockId::Air {
                                    fluid_spawns.push((npos, block, new_level));
                                } else if block == BlockId::Water && nblock == BlockId::Lava {
                                    solidifications.push((npos, BlockId::Stone));
                                }
                            }
                        }
                    }

                    if level < 8 {
                        let above_block = self.get_block(IVec3::new(x, y + 1, z));
                        let has_parent_above = above_block == block;
                        let has_higher_neighbor = cardinals[0..4].iter().any(|&p| {
                            self.get_block(p) == block && self.fluid_level(p) > level
                        });
                        if !has_parent_above && !has_higher_neighbor {
                            fluid_despawns.push(pos);
                        }
                    }
                }
            }
        }

        for tnt_pos in tnts_to_prime {
            self.prime_tnt(tnt_pos);
        }

        for (pos, stone) in solidifications {
            self.set_block(pos, stone);
        }
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
            let mut sand_moves: Vec<(IVec3, IVec3)> = Vec::new();
            let mut sand_stopped: Vec<IVec3> = Vec::new();

            let mut active_list: Vec<IVec3> = self.active_sand.iter().copied().collect();
            active_list.sort_unstable_by_key(|p| p.y);

            for pos in active_list {
                if self.get_block(pos) != BlockId::Sand {
                    sand_stopped.push(pos);
                    continue;
                }
                let below = IVec3::new(pos.x, pos.y - 1, pos.z);
                if below.y >= 0 && self.get_block(below) == BlockId::Air {
                    sand_moves.push((pos, below));
                } else {
                    sand_stopped.push(pos);
                }
            }

            for pos in sand_stopped {
                self.active_sand.remove(&pos);
            }

            for (from, to) in sand_moves {
                if self.get_block(from) == BlockId::Sand && self.get_block(to) == BlockId::Air {
                    self.set_block(from, BlockId::Air);
                    self.set_block(to, BlockId::Sand);
                }
            }
        }
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RaycastHit> {
        let dir = dir.normalized();
        if dir.length_sq() < 1e-8 {
            return None;
        }

        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let next_boundary = |p: f32, step: i32| -> f32 {
            if step > 0 {
                p.floor() + 1.0 - p
            } else {
                p - p.floor()
            }
        };

        let mut t_max_x = if dir.x.abs() > 1e-8 {
            next_boundary(origin.x, step_x) / dir.x.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_y = if dir.y.abs() > 1e-8 {
            next_boundary(origin.y, step_y) / dir.y.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_z = if dir.z.abs() > 1e-8 {
            next_boundary(origin.z, step_z) / dir.z.abs()
        } else {
            f32::INFINITY
        };

        let mut last_normal = IVec3::new(0, 0, 0);
        let mut traveled = 0.0f32;

        while traveled <= max_dist {
            let block = self.get_block(IVec3::new(x, y, z));
            if block.is_targetable() {
                return Some(RaycastHit {
                    block_pos: IVec3::new(x, y, z),
                    normal: last_normal,
                });
            }

            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x;
                traveled = t_max_x;
                t_max_x += t_delta_x;
                last_normal = IVec3::new(-step_x, 0, 0);
            } else if t_max_y < t_max_z {
                y += step_y;
                traveled = t_max_y;
                t_max_y += t_delta_y;
                last_normal = IVec3::new(0, -step_y, 0);
            } else {
                z += step_z;
                traveled = t_max_z;
                t_max_z += t_delta_z;
                last_normal = IVec3::new(0, 0, -step_z);
            }
        }

        None
    }

    /// Traces a ray from `origin` along `dir` for up to `max_dist` blocks,
    /// stopping if any solid voxel is encountered and leaving a 0.25-block buffer
    /// to prevent the third-person camera from clipping into walls.
    pub fn raycast_camera_collision(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> f32 {
        let dir = dir.normalized();
        if dir.length_sq() < 1e-8 {
            return max_dist;
        }

        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let next_boundary = |p: f32, step: i32| -> f32 {
            if step > 0 {
                p.floor() + 1.0 - p
            } else {
                p - p.floor()
            }
        };

        let mut t_max_x = if dir.x.abs() > 1e-8 {
            next_boundary(origin.x, step_x) / dir.x.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_y = if dir.y.abs() > 1e-8 {
            next_boundary(origin.y, step_y) / dir.y.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_z = if dir.z.abs() > 1e-8 {
            next_boundary(origin.z, step_z) / dir.z.abs()
        } else {
            f32::INFINITY
        };

        let mut traveled = 0.0f32;
        while traveled <= max_dist {
            let block = self.get_block(IVec3::new(x, y, z));
            if block.is_solid() {
                return (traveled - 0.25).max(0.25);
            }

            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x;
                traveled = t_max_x;
                t_max_x += t_delta_x;
            } else if t_max_y < t_max_z {
                y += step_y;
                traveled = t_max_y;
                t_max_y += t_delta_y;
            } else {
                z += step_z;
                traveled = t_max_z;
                t_max_z += t_delta_z;
            }
        }

        max_dist
    }

    /// Calculates acoustic occlusion factor in [0.0, 1.0] between a listener and sound emitter.
    ///
    /// Traces the 3D line through the voxel grid. Solid blocks (stone, wood, dirt, etc.)
    /// absorb acoustic energy and cause high-frequency dampening and volume loss.
    pub fn calculate_acoustic_occlusion(&self, listener: Vec3, emitter: Vec3) -> f32 {
        let delta = emitter - listener;
        let dist = delta.length();
        if dist <= 0.6 {
            return 0.0;
        }

        let dir = delta * (1.0 / dist);
        let mut x = listener.x.floor() as i32;
        let mut y = listener.y.floor() as i32;
        let mut z = listener.z.floor() as i32;

        let end_x = emitter.x.floor() as i32;
        let end_y = emitter.y.floor() as i32;
        let end_z = emitter.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let next_boundary = |p: f32, step: i32| -> f32 {
            if step > 0 { p.floor() + 1.0 - p } else { p - p.floor() }
        };

        let mut t_max_x = if dir.x.abs() > 1e-8 { next_boundary(listener.x, step_x) / dir.x.abs() } else { f32::INFINITY };
        let mut t_max_y = if dir.y.abs() > 1e-8 { next_boundary(listener.y, step_y) / dir.y.abs() } else { f32::INFINITY };
        let mut t_max_z = if dir.z.abs() > 1e-8 { next_boundary(listener.z, step_z) / dir.z.abs() } else { f32::INFINITY };

        let mut occlusion_accum = 0.0f32;
        let mut traveled = 0.0f32;
        let mut steps = 0;
        const MAX_STEPS: usize = 64;

        while traveled < dist && steps < MAX_STEPS {
            steps += 1;
            // Ignore the listener's own voxel and the emitter's own voxel
            if (x != listener.x.floor() as i32 || y != listener.y.floor() as i32 || z != listener.z.floor() as i32)
                && (x != end_x || y != end_y || z != end_z)
            {
                let block = self.get_block(IVec3::new(x, y, z));
                if block != BlockId::Air {
                    let weight = match block {
                        BlockId::Glass => 0.15,
                        BlockId::Water => 0.25,
                        BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => 0.12,
                        BlockId::Wood | BlockId::Planks | BlockId::Door | BlockId::DoorOpen => 0.35,
                        _ => 0.50, // Dense blocks: Stone, Dirt, Cobblestone, Ores, etc.
                    };
                    occlusion_accum += weight;
                    if occlusion_accum >= 1.0 {
                        return 1.0;
                    }
                }
            }

            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x;
                traveled = t_max_x;
                t_max_x += t_delta_x;
            } else if t_max_y < t_max_z {
                y += step_y;
                traveled = t_max_y;
                t_max_y += t_delta_y;
            } else {
                z += step_z;
                traveled = t_max_z;
                t_max_z += t_delta_z;
            }
        }

        occlusion_accum.min(1.0)
    }

    pub fn box_collides(&self, min: Vec3, max: Vec3) -> bool {
        let eps = 1e-4;
        let x0 = min.x.floor() as i32;
        let x1 = (max.x - eps).floor() as i32;
        let y0 = (min.y.floor() as i32).max(0);
        let y1 = ((max.y - eps).floor() as i32).min(CHUNK_SIZE_Y as i32 - 1);
        let z0 = min.z.floor() as i32;
        let z1 = (max.z - eps).floor() as i32;

        if y0 > y1 {
            return false;
        }

        let mut last_cx = i32::MIN;
        let mut last_cz = i32::MIN;
        let mut cached_chunk: Option<&Chunk> = None;

        for x in x0..=x1 {
            for z in z0..=z1 {
                let (cx, cz, lx, lz) = World::world_to_chunk(x, z);
                if cx != last_cx || cz != last_cz {
                    last_cx = cx;
                    last_cz = cz;
                    cached_chunk = self.chunks.get(&(cx, cz));
                }

                if let Some(chunk) = cached_chunk {
                    for y in y0..=y1 {
                        let b = chunk.get_local(lx, y, lz);
                        if b.is_solid() {
                            if b.is_half_block() {
                                if min.y < y as f32 + 0.5 {
                                    return true;
                                }
                            } else {
                                return true;
                            }
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
        let mut chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            dirty: false,
        };

        chunk.set_local(0, 0, 0, BlockId::Stone);
        assert_eq!(chunk.get_local(0, 0, 0), BlockId::Stone);
        assert!(chunk.dirty);

        chunk.set_local(15, 63, 15, BlockId::IronOre);
        assert_eq!(chunk.get_local(15, 63, 15), BlockId::IronOre);

        // Out of bounds returns Air
        assert_eq!(chunk.get_local(-1, 0, 0), BlockId::Air);
        assert_eq!(chunk.get_local(16, 0, 0), BlockId::Air);
    }

    #[test]
    fn test_find_spawn_position_disallowed_biomes() {
        for seed in [1337, 42, 999999, 1234567, 777] {
            let spawn = World::find_spawn_position(seed);
            let (biome, _) = get_biome_at(spawn.x.floor() as i32, spawn.z.floor() as i32, seed);
            assert_ne!(biome, Biome::DeepOcean, "Spawn cannot be in DeepOcean for seed {}", seed);
            assert_ne!(biome, Biome::Ocean, "Spawn cannot be in Ocean for seed {}", seed);
            assert_ne!(biome, Biome::Swamp, "Spawn cannot be in Swamp for seed {}", seed);
            assert!(spawn.y >= 24.0, "Spawn Y ({}) should be above water level", spawn.y);
        }
    }

    #[test]
    fn test_leaf_support_and_decay_after_trunk_removal() {
        let mut world = World::new();
        // Insert empty chunk at (0, 0)
        let chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            dirty: false,
        };
        world.chunks.insert((0, 0), chunk);

        // Build a mini tree: trunk at (5, 10, 5) and (5, 11, 5)
        world.set_block(IVec3::new(5, 10, 5), BlockId::Wood);
        world.set_block(IVec3::new(5, 11, 5), BlockId::Wood);

        // Canopy leaves around (5, 11, 5) and (5, 12, 5)
        world.set_block(IVec3::new(5, 12, 5), BlockId::Leaves);
        world.set_block(IVec3::new(6, 11, 5), BlockId::Leaves);
        world.set_block(IVec3::new(4, 11, 5), BlockId::Leaves);

        // Leaves are supported initially
        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert!(world.is_leaf_supported(IVec3::new(6, 11, 5), 5));
        assert!(world.is_leaf_supported(IVec3::new(4, 11, 5), 5));
        assert!(world.find_unsupported_leaves_around(IVec3::new(5, 10, 5)).is_empty());

        // Remove lower trunk: top trunk still supports leaves
        world.set_block(IVec3::new(5, 10, 5), BlockId::Air);
        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert!(world.find_unsupported_leaves_around(IVec3::new(5, 10, 5)).is_empty());

        // Remove top trunk: now trunk is completely destroyed!
        world.set_block(IVec3::new(5, 11, 5), BlockId::Air);
        assert!(!world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert!(!world.is_leaf_supported(IVec3::new(6, 11, 5), 5));

        let unsupported = world.find_unsupported_leaves_around(IVec3::new(5, 11, 5));
        assert_eq!(unsupported.len(), 3);
        assert!(unsupported.contains(&IVec3::new(5, 12, 5)));
        assert!(unsupported.contains(&IVec3::new(6, 11, 5)));
        assert!(unsupported.contains(&IVec3::new(4, 11, 5)));
    }

    #[test]
    fn test_world_save_and_load_with_player_stats() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("rustcraft_test_player_world.bin");
        let path_str = path.to_string_lossy().to_string();

        let mut world = World::new_with_seed(12345);
        world.name = "PlayerSaveTest".to_string();

        let mut player = crate::gameplay::player::Player::new();
        player.health = 13.0;
        player.hunger = 15.0;
        player.saturation = 4.0;
        player.exhaustion = 2.0;
        player.air = 18.0;
        player.selected_hotbar_slot = 4;
        player.inventory.hotbar[4] = Some(crate::gameplay::inventory::ItemStack {
            block: BlockId::Torch,
            count: 16,
        });

        world.saved_player = Some(player.save_state());
        assert!(world.save(&path_str).is_ok());

        let loaded = World::load(&path_str).expect("Should load RUSTCRAFT6 world file");
        assert_eq!(loaded.name, "PlayerSaveTest");
        let loaded_player = loaded.saved_player.expect("Should load player data");
        assert_eq!(loaded_player.health, 13.0);
        assert_eq!(loaded_player.hunger, 15.0);
        assert_eq!(loaded_player.saturation, 4.0);
        assert_eq!(loaded_player.selected_hotbar_slot, 4);
        assert_eq!(loaded_player.hotbar[4].unwrap().block, BlockId::Torch);
        assert_eq!(loaded_player.hotbar[4].unwrap().count, 16);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_all_new_biomes_generation_and_rarity() {
        use std::collections::HashMap;

        let seed = 424242;
        let mut biome_counts: HashMap<Biome, u32> = HashMap::new();

        // Sample a wide 4000x4000 block area
        for x in (-2000..=2000).step_by(40) {
            for z in (-2000..=2000).step_by(40) {
                let (biome, _) = get_biome_at(x, z, seed);
                *biome_counts.entry(biome).or_insert(0) += 1;
            }
        }

        // Verify that all requested biomes generate in the world
        assert!(biome_counts.contains_key(&Biome::Jungle), "Jungle must generate");
        assert!(biome_counts.contains_key(&Biome::Badlands), "Badlands must generate");
        assert!(biome_counts.contains_key(&Biome::DarkForest), "DarkForest must generate");
        assert!(biome_counts.contains_key(&Biome::Taiga), "Taiga must generate");
        assert!(biome_counts.contains_key(&Biome::BirchForest), "BirchForest must generate");
        assert!(biome_counts.contains_key(&Biome::IceSpikes), "IceSpikes must generate");
        assert!(biome_counts.contains_key(&Biome::MushroomIsland), "MushroomIsland must generate");
        assert!(biome_counts.contains_key(&Biome::Meadow), "Meadow must generate");
        assert!(biome_counts.contains_key(&Biome::JaggedPeaks), "JaggedPeaks must generate");
        assert!(biome_counts.contains_key(&Biome::FrozenPeaks), "FrozenPeaks must generate");
        assert!(biome_counts.contains_key(&Biome::SnowySlopes), "SnowySlopes must generate");

        // Verify rarity: MushroomIsland and IceSpikes should be much rarer than common biomes like Plains / Forest / Taiga
        let mushroom_count = *biome_counts.get(&Biome::MushroomIsland).unwrap_or(&0);
        let icespikes_count = *biome_counts.get(&Biome::IceSpikes).unwrap_or(&0);
        let plains_count = *biome_counts.get(&Biome::Plains).unwrap_or(&0);
        let forest_count = *biome_counts.get(&Biome::Forest).unwrap_or(&0);
        let taiga_count = *biome_counts.get(&Biome::Taiga).unwrap_or(&0);

        assert!(mushroom_count < plains_count, "Mushroom Island should be much rarer than Plains");
        assert!(mushroom_count < forest_count, "Mushroom Island should be much rarer than Forest");
        assert!(icespikes_count < taiga_count, "Ice Spikes should be rarer than Taiga");
    }

    #[test]
    fn test_biome_name_at_all_biomes() {
        let world = World::new_with_seed(123);
        let name = world.biome_name_at(0, 0);
        assert!(!name.is_empty());
    }
}

