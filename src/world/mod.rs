pub mod block;
pub mod mesher;
pub mod storage;
pub mod tile_entity;
pub mod weather;

use std::collections::{HashMap, HashSet};

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
pub enum VillageComponent {
    CenterWell,
    Cottage,
    Blacksmith,
    Farm,
    Library,
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

    /// Color grading / postprocessing tint multipliers [R, G, B] for this biome.
    /// Slightly adjusts the mood and atmosphere of the scene (e.g. warm golden desert,
    /// icy blue tundra, lush jungle emerald, murky swamp, romantic pink cherry blossom).
    #[inline]
    pub fn color_filter(self) -> [f32; 3] {
        match self {
            Biome::DeepOcean => [0.88, 0.97, 1.12],
            Biome::Ocean => [0.92, 1.02, 1.08],
            Biome::SnowyTaiga => [0.92, 0.98, 1.08],
            Biome::SnowyTundra => [0.93, 1.00, 1.11],
            Biome::AutumnForest => [1.11, 1.01, 0.89],
            Biome::Forest => [0.97, 1.04, 0.97],
            Biome::CherryGrove => [1.08, 0.97, 1.04],
            Biome::Plains => [1.01, 1.03, 0.98],
            Biome::Swamp => [0.94, 1.03, 0.86],
            Biome::Savanna => [1.07, 1.04, 0.91],
            Biome::Desert => [1.10, 1.03, 0.89],
            Biome::Mountain => [0.97, 1.00, 1.04],
            Biome::Jungle => [0.95, 1.08, 0.94],
            Biome::Badlands => [1.12, 0.98, 0.90],
            Biome::DarkForest => [0.91, 0.96, 0.93],
            Biome::Taiga => [0.96, 1.02, 1.02],
            Biome::BirchForest => [1.02, 1.06, 0.99],
            Biome::IceSpikes => [0.90, 1.03, 1.14],
            Biome::MushroomIsland => [1.05, 0.96, 1.08],
            Biome::Meadow => [1.04, 1.05, 0.98],
            Biome::JaggedPeaks => [0.95, 0.97, 1.05],
            Biome::FrozenPeaks => [0.89, 1.01, 1.13],
            Biome::SnowySlopes => [0.95, 1.01, 1.08],
        }
    }

    /// Whether this biome experiences snowfall instead of rain.
    #[inline]
    pub fn has_snow(self) -> bool {
        matches!(
            self,
            Biome::SnowyTaiga
                | Biome::SnowyTundra
                | Biome::IceSpikes
                | Biome::FrozenPeaks
                | Biome::SnowySlopes
                | Biome::JaggedPeaks
        )
    }

    /// Whether this biome experiences precipitation (rain or snow).
    /// Arid biomes like deserts, badlands, and savannas experience overcast skies without falling precipitation.
    #[inline]
    pub fn has_precipitation(self) -> bool {
        !matches!(self, Biome::Desert | Biome::Badlands | Biome::Savanna)
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
        if continentalness < 0.46 {
            // Lower mountain foothills / slopes buffer zone
            if continentalness < 0.44 && temp > -0.05 && temp < 0.22 && humidity > -0.05 {
                Biome::Meadow
            } else if temp < 0.02 {
                Biome::SnowySlopes
            } else {
                Biome::Mountain
            }
        } else {
            // High mountain summits & peaks (continentalness >= 0.46)
            if temp < 0.0 {
                if rarity > -0.05 {
                    Biome::FrozenPeaks
                } else {
                    Biome::JaggedPeaks
                }
            } else if temp < 0.12 {
                Biome::JaggedPeaks
            } else {
                Biome::Mountain
            }
        }
    } else if temp < -0.30 {
        // Cold / Snowy zones (rare arctic regions)
        if continentalness > 0.24 {
            if rarity > 0.05 {
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
        Biome::Meadow => {
            let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
            let lift = m_t * m_t * (3.0 - 2.0 * m_t);
            31.0 + n * 6.0 + lift * 18.0
        }
        Biome::SnowySlopes => {
            let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
            let lift = m_t * m_t * (3.0 - 2.0 * m_t);
            31.0 + n * 6.0 + lift * 18.0
        }
        Biome::Mountain => {
            let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
            let lift = m_t * m_t * (3.0 - 2.0 * m_t);
            let ridge = 1.0 - (n * 2.0).abs().clamp(0.0, 1.0);
            31.0 + n * 6.0 + lift * 20.0 + (ridge * 8.0) * lift
        }
        Biome::JaggedPeaks => {
            let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
            let lift = m_t * m_t * (3.0 - 2.0 * m_t);
            let ridge = 1.0 - (n * 2.2).abs().clamp(0.0, 1.0);
            31.0 + n * 6.0 + lift * 20.0 + (ridge * 9.0) * lift
        }
        Biome::FrozenPeaks => {
            let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
            let lift = m_t * m_t * (3.0 - 2.0 * m_t);
            let ridge = 1.0 - (n * 1.8).abs().clamp(0.0, 1.0);
            31.0 + n * 6.0 + lift * 20.0 + (ridge * 9.0) * lift
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
            Village(i32, i32, i32, Biome, VillageComponent),
        }

        let center_wx = cx * CHUNK_SIZE_X + 8;
        let center_wz = cz * CHUNK_SIZE_Z + 8;
        let (c_biome, c_continentalness) = get_biome_at(center_wx, center_wz, seed);
        let center_height = get_height_at(center_wx, center_wz, seed, c_biome, c_continentalness);

        // Multi-chunk village grid: 20x20 chunk regions (~320+ blocks spacing between villages)
        let village_region_size = 20i32;
        let reg_x = if cx >= 0 { cx / village_region_size } else { (cx - village_region_size + 1) / village_region_size };
        let reg_z = if cz >= 0 { cz / village_region_size } else { (cz - village_region_size + 1) / village_region_size };
        let v_hash = hash_coords(reg_x, reg_z, seed.wrapping_add(8881337));
        let v_cx = reg_x * village_region_size + ((v_hash % (village_region_size - 4) as u32) as i32 + 2);
        let v_cz = reg_z * village_region_size + (((v_hash >> 16) % (village_region_size - 4) as u32) as i32 + 2);

        let v_center_wx = v_cx * CHUNK_SIZE_X + 8;
        let v_center_wz = v_cz * CHUNK_SIZE_Z + 8;
        let (v_biome, v_cont) = get_biome_at(v_center_wx, v_center_wz, seed);
        let v_height = get_height_at(v_center_wx, v_center_wz, seed, v_biome, v_cont);

        let is_valid_village_biome = matches!(v_biome, Biome::Plains | Biome::Meadow | Biome::Savanna | Biome::Desert | Biome::Taiga);
        let is_valid_village_height = v_height > 22 && v_height < CHUNK_SIZE_Y - 14;

        let village_comp = if is_valid_village_biome && is_valid_village_height {
            if cx == v_cx && cz == v_cz {
                Some(VillageComponent::CenterWell)
            } else if cx == v_cx + 1 && cz == v_cz {
                Some(VillageComponent::Cottage)
            } else if cx == v_cx - 1 && cz == v_cz {
                Some(VillageComponent::Blacksmith)
            } else if cx == v_cx && cz == v_cz + 1 {
                Some(VillageComponent::Farm)
            } else if cx == v_cx && cz == v_cz - 1 {
                Some(VillageComponent::Library)
            } else {
                None
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
                let patch_noise = fractal_noise2(wx as f32 * 0.045, wz as f32 * 0.045, seed.wrapping_add(8881), 2, 0.5);

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
                                // Natural craggy alpine peaks: high summits are snow-covered,
                                // with organic rock faces and snowfields defined by continuous noise
                                if height >= 52 {
                                    if patch_noise < -0.35 { BlockId::Stone } else { BlockId::Snow }
                                } else if height >= 40 {
                                    if patch_noise > -0.05 { BlockId::Snow } else { BlockId::Stone }
                                } else {
                                    if patch_noise > 0.15 { BlockId::Snow } else { BlockId::Stone }
                                }
                            }
                            Biome::FrozenPeaks => {
                                // Cohesive glacial ice sheets and basins rather than salt-and-pepper checkerboard
                                if patch_noise > 0.28 {
                                    BlockId::Ice
                                } else {
                                    BlockId::Snow
                                }
                            }
                            Biome::Mountain => {
                                let snow_thresh = 48.0 + patch_noise * 3.0;
                                let stone_thresh = 37.0 + patch_noise * 3.0;
                                if (height as f32) >= snow_thresh {
                                    BlockId::Snow
                                } else if (height as f32) >= stone_thresh {
                                    BlockId::Stone
                                } else {
                                    BlockId::Grass
                                }
                            }
                            _ => {
                                if height <= 21 {
                                    let water_h = hash_coords(wx, wz, seed.wrapping_add(51893));
                                    if water_h % 10 < 3 {
                                        BlockId::Clay
                                    } else if water_h % 10 < 5 {
                                        BlockId::Gravel
                                    } else {
                                        BlockId::Sand
                                    }
                                } else {
                                    BlockId::Grass
                                }
                            }
                        }
                    } else if y > height - 4 {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean => BlockId::Sand,
                            Biome::Desert => {
                                if y <= height - 3 {
                                    BlockId::Sandstone
                                } else {
                                    BlockId::Sand
                                }
                            }
                            Biome::Badlands => {
                                if (y / 2) % 3 == 0 { BlockId::Sand } else if (y / 2) % 3 == 1 { BlockId::Stone } else { BlockId::Mud }
                            }
                            Biome::Swamp | Biome::MushroomIsland => BlockId::Mud,
                            Biome::IceSpikes => {
                                if y > height - 2 { BlockId::Snow } else { BlockId::Ice }
                            }
                            Biome::FrozenPeaks => {
                                if patch_noise > 0.28 {
                                    if y > height - 5 { BlockId::Ice } else { BlockId::Stone }
                                } else if y > height - 3 {
                                    BlockId::Snow
                                } else {
                                    BlockId::Stone
                                }
                            }
                            Biome::JaggedPeaks => {
                                let is_snow = if height >= 52 {
                                    patch_noise >= -0.35
                                } else if height >= 40 {
                                    patch_noise > -0.05
                                } else {
                                    patch_noise > 0.15
                                };
                                if is_snow && y > height - 2 {
                                    BlockId::Snow
                                } else {
                                    BlockId::Stone
                                }
                            }
                            Biome::SnowySlopes => {
                                if y > height - 3 { BlockId::Snow } else { BlockId::Stone }
                            }
                            Biome::Mountain => {
                                let stone_thresh = 37.0 + patch_noise * 3.0;
                                if (height as f32) >= stone_thresh {
                                    BlockId::Stone
                                } else if height <= 21 {
                                    BlockId::Sand
                                } else {
                                    BlockId::Dirt
                                }
                            }
                            _ => {
                                if height <= 21 {
                                    let water_h = hash_coords(wx, wz, seed.wrapping_add(51893));
                                    if water_h % 10 < 3 {
                                        BlockId::Clay
                                    } else if water_h % 10 < 5 {
                                        BlockId::Gravel
                                    } else {
                                        BlockId::Sand
                                    }
                                } else {
                                    BlockId::Dirt
                                }
                            }
                        }
                    } else if biome == Biome::Badlands && y >= 14 {
                        let ore_h = hash_coords_3d(wx, y, wz, seed);
                        if ore_h % 37 == 0 {
                            BlockId::CoalOre
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else if ore_h % 127 == 0 && y < 20 {
                            BlockId::GoldOre
                        } else if ore_h % 79 == 0 && y < 24 {
                            BlockId::RustStoneOre
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
                        } else if ore_h % 43 == 0 && y > 15 && y < 55 {
                            BlockId::Gravel
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else if ore_h % 127 == 0 && y < 20 {
                            BlockId::GoldOre
                        } else if ore_h % 79 == 0 && y < 24 {
                            BlockId::RustStoneOre
                        } else if ore_h % 151 == 0 && y < 14 {
                            BlockId::Obsidian
                        } else if ore_h % 199 == 0 && y < 14 {
                            BlockId::DiamondOre
                        } else if matches!(biome, Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes) && ore_h % 149 == 0 && y >= 10 && y <= 60 {
                            BlockId::EmeraldOre
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
                } else if height < 20 {
                    // Underwater vegetation: Kelp forests and Seagrass
                    let bed_idx = Chunk::index(lx, height, lz);
                    let bed_block = BlockId::from_u8(blocks[bed_idx]);
                    if bed_block == BlockId::Sand || bed_block == BlockId::Dirt || bed_block == BlockId::Mud {
                        let h = hash_coords(wx, wz, seed.wrapping_add(8831));
                        let water_depth = 20 - height;
                        if water_depth >= 3 && (biome == Biome::Ocean || biome == Biome::DeepOcean) {
                            if h % 5 == 0 {
                                // Kelp stalks growing upwards towards water surface
                                let max_kelp_h = (water_depth - 1).clamp(1, 14);
                                let kelp_len = 1 + (h % (max_kelp_h as u32)) as i32;
                                for dy in 1..=kelp_len {
                                    let y = height + dy;
                                    if y < 20 {
                                        blocks[Chunk::index(lx, y, lz)] = BlockId::Kelp.to_u8();
                                    }
                                }
                            } else if h % 3 == 0 {
                                // Seagrass on seabed
                                blocks[Chunk::index(lx, height + 1, lz)] = BlockId::Seagrass.to_u8();
                            }
                        } else if water_depth >= 1 && h % 4 == 0 {
                            // Shallow seagrass
                            blocks[Chunk::index(lx, height + 1, lz)] = BlockId::Seagrass.to_u8();
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
                Structure::Village(cx, base_y, cz, biome, comp) => place_village_component(&mut blocks, cx, base_y, cz, biome, seed, comp),
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

fn place_village_component(
    blocks: &mut [u8],
    _center_x: i32,
    base_y: i32,
    _center_z: i32,
    biome: Biome,
    _seed: u32,
    comp: VillageComponent,
) {
    let set = |blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId| {
        if x >= 0 && x < CHUNK_SIZE_X && y >= 0 && y < CHUNK_SIZE_Y && z >= 0 && z < CHUNK_SIZE_Z {
            blocks[Chunk::index(x, y, z)] = b.to_u8();
        }
    };

    let is_desert = matches!(biome, Biome::Desert | Biome::Badlands);
    let wood_mat = if is_desert { BlockId::Sand } else { BlockId::Wood };
    let plank_mat = if is_desert { BlockId::Sand } else { BlockId::Planks };
    let stone_mat = BlockId::Stone;
    let path_mat = if is_desert { BlockId::Sand } else { BlockId::Dirt };

    // 1. Clear air above structure and level ground foundation
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            for y in (base_y + 1)..=(base_y + 10) {
                set(blocks, x, y, z, BlockId::Air);
            }
            for y in (base_y - 4)..=base_y {
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    set(blocks, x, y, z, if is_desert { BlockId::Sand } else { BlockId::Dirt });
                }
            }
        }
    }

    match comp {
        VillageComponent::CenterWell => {
            // Crossroads leading across all 4 chunk borders to neighbor structures
            for z in 0..CHUNK_SIZE_Z {
                set(blocks, 7, base_y, z, path_mat);
                set(blocks, 8, base_y, z, path_mat);
            }
            for x in 0..CHUNK_SIZE_X {
                set(blocks, x, base_y, 7, path_mat);
                set(blocks, x, base_y, 8, path_mat);
            }

            // Central Village Well
            for x in 6..=9 {
                for z in 6..=9 {
                    if x == 6 || x == 9 || z == 6 || z == 9 {
                        set(blocks, x, base_y + 1, z, stone_mat);
                    } else {
                        for y in (base_y - 3)..=base_y {
                            set(blocks, x, y, z, BlockId::Water);
                        }
                    }
                    set(blocks, x, base_y - 4, z, stone_mat);
                }
            }
            let well_posts = [(6, 6), (9, 6), (6, 9), (9, 9)];
            for &(wx, wz) in &well_posts {
                for dy in 2..=4 {
                    set(blocks, wx, base_y + dy, wz, wood_mat);
                }
            }
            for x in 6..=9 {
                for z in 6..=9 {
                    set(blocks, x, base_y + 5, z, stone_mat);
                }
            }

            // 4 Plaza Lampposts
            let lamps = [(4, 4), (11, 4), (4, 11), (11, 11)];
            for &(lx, lz) in &lamps {
                set(blocks, lx, base_y + 1, lz, wood_mat);
                set(blocks, lx, base_y + 2, lz, wood_mat);
                set(blocks, lx, base_y + 3, lz, BlockId::Torch);
            }

            // Village Benches
            for &(bx, bz) in &[(4, 7), (4, 8), (11, 7), (11, 8)] {
                set(blocks, bx, base_y + 1, bz, plank_mat);
            }
        }

        VillageComponent::Cottage => {
            // Connecting path entering from West (-X)
            for x in 0..=5 {
                set(blocks, x, base_y, 7, path_mat);
                set(blocks, x, base_y, 8, path_mat);
            }

            // Spacious Cottage (x: 5..=12, z: 4..=11)
            for x in 5..=12 {
                for z in 4..=11 {
                    set(blocks, x, base_y, z, plank_mat);
                }
            }
            for x in 5..=12 {
                for z in 4..=11 {
                    let is_corner = (x == 5 || x == 12) && (z == 4 || z == 11);
                    let is_wall = x == 5 || x == 12 || z == 4 || z == 11;
                    if is_corner {
                        for dy in 1..=4 {
                            set(blocks, x, base_y + dy, z, wood_mat);
                        }
                    } else if is_wall {
                        for dy in 1..=3 {
                            if x == 5 && z == 7 {
                                if dy == 1 || dy == 2 {
                                    set(blocks, x, base_y + dy, z, BlockId::Door);
                                } else {
                                    set(blocks, x, base_y + dy, z, plank_mat);
                                }
                            } else if dy == 2 && ((z == 7 && x == 12) || (x == 8 && (z == 4 || z == 11))) {
                                set(blocks, x, base_y + dy, z, BlockId::Glass);
                            } else {
                                set(blocks, x, base_y + dy, z, plank_mat);
                            }
                        }
                    }
                }
            }
            // Roof
            for x in 4..=13 {
                for z in 3..=12 {
                    set(blocks, x, base_y + 4, z, wood_mat);
                }
            }
            for x in 5..=12 {
                for z in 5..=10 {
                    set(blocks, x, base_y + 5, z, plank_mat);
                }
            }

            // Interior furniture
            set(blocks, 11, base_y + 1, 10, BlockId::Bed);
            set(blocks, 11, base_y + 1, 5, BlockId::CraftingTable);
            set(blocks, 10, base_y + 1, 5, BlockId::Planks);
            set(blocks, 7, base_y + 1, 10, BlockId::Chest);
            set(blocks, 9, base_y + 3, 10, BlockId::Torch);

            // Front Yard Lamppost
            set(blocks, 3, base_y + 1, 5, wood_mat);
            set(blocks, 3, base_y + 2, 5, wood_mat);
            set(blocks, 3, base_y + 3, 5, BlockId::Torch);
        }

        VillageComponent::Blacksmith => {
            // Connecting path entering from East (+X)
            for x in 10..CHUNK_SIZE_X {
                set(blocks, x, base_y, 7, path_mat);
                set(blocks, x, base_y, 8, path_mat);
            }

            // Stone Forge Workshop & Resting Quarters (x: 4..=11, z: 4..=11)
            for x in 4..=11 {
                for z in 4..=11 {
                    set(blocks, x, base_y, z, stone_mat);
                }
            }
            let forge_pillars = [(4, 4), (11, 4), (4, 11), (11, 11), (7, 4), (7, 11)];
            for &(fx, fz) in &forge_pillars {
                for dy in 1..=3 {
                    set(blocks, fx, base_y + dy, fz, stone_mat);
                }
            }
            for x in 4..=11 {
                for z in 4..=11 {
                    set(blocks, x, base_y + 4, z, stone_mat);
                }
            }

            // Outdoor smithy counter & furnace
            set(blocks, 9, base_y + 1, 5, BlockId::Furnace);
            set(blocks, 10, base_y + 1, 5, BlockId::Furnace);
            set(blocks, 8, base_y + 1, 5, BlockId::Lava);
            set(blocks, 8, base_y + 1, 8, BlockId::StoneSlab);
            set(blocks, 9, base_y + 1, 10, BlockId::Chest);
            set(blocks, 7, base_y + 3, 5, BlockId::Torch);
            set(blocks, 7, base_y + 3, 10, BlockId::Torch);

            // Indoor rest room
            for z in 5..=10 {
                set(blocks, 4, base_y + 1, z, stone_mat);
                set(blocks, 4, base_y + 2, z, stone_mat);
                set(blocks, 4, base_y + 3, z, stone_mat);
            }
            set(blocks, 5, base_y + 1, 5, BlockId::Bed);
            set(blocks, 5, base_y + 1, 10, BlockId::CraftingTable);
            set(blocks, 5, base_y + 3, 7, BlockId::Torch);
        }

        VillageComponent::Farm => {
            // Central path entering from North (-Z)
            for z in 0..CHUNK_SIZE_Z {
                set(blocks, 7, base_y, z, path_mat);
                set(blocks, 8, base_y, z, path_mat);
            }

            // Left Crop Field (x: 2..=6, z: 3..=13)
            for x in 2..=6 {
                set(blocks, x, base_y, 3, wood_mat);
                set(blocks, x, base_y, 13, wood_mat);
            }
            for z in 3..=13 {
                set(blocks, 2, base_y, z, wood_mat);
                set(blocks, 6, base_y, z, wood_mat);
            }
            for z in 4..=12 {
                set(blocks, 4, base_y, z, BlockId::Water);
                for &x in &[3, 5] {
                    set(blocks, x, base_y, z, BlockId::FarmlandWet);
                    let crop = match (x + z) % 3 {
                        0 => BlockId::WheatCrop,
                        1 => BlockId::CarrotCrop,
                        _ => BlockId::PotatoCrop,
                    };
                    set(blocks, x, base_y + 1, z, crop);
                }
            }

            // Right Crop Field (x: 9..=13, z: 3..=13)
            for x in 9..=13 {
                set(blocks, x, base_y, 3, wood_mat);
                set(blocks, x, base_y, 13, wood_mat);
            }
            for z in 3..=13 {
                set(blocks, 9, base_y, z, wood_mat);
                set(blocks, 13, base_y, z, wood_mat);
            }
            for z in 4..=12 {
                set(blocks, 11, base_y, z, BlockId::Water);
                for &x in &[10, 12] {
                    set(blocks, x, base_y, z, BlockId::FarmlandWet);
                    let crop = match (x + z) % 3 {
                        0 => BlockId::WheatCrop,
                        1 => BlockId::CarrotCrop,
                        _ => BlockId::PotatoCrop,
                    };
                    set(blocks, x, base_y + 1, z, crop);
                }
            }

            // Lampposts at farm gate
            for &(lx, lz) in &[(6, 2), (9, 2)] {
                set(blocks, lx, base_y + 1, lz, wood_mat);
                set(blocks, lx, base_y + 2, lz, wood_mat);
                set(blocks, lx, base_y + 3, lz, BlockId::Torch);
            }
        }

        VillageComponent::Library => {
            // Connecting road entering from South (+Z)
            for z in 10..CHUNK_SIZE_Z {
                set(blocks, 7, base_y, z, path_mat);
                set(blocks, 8, base_y, z, path_mat);
            }

            // Library Building (x: 4..=11, z: 3..=10)
            for x in 4..=11 {
                for z in 3..=10 {
                    set(blocks, x, base_y, z, stone_mat);
                }
            }
            for x in 4..=11 {
                for z in 3..=10 {
                    let is_corner = (x == 4 || x == 11) && (z == 3 || z == 10);
                    let is_wall = x == 4 || x == 11 || z == 3 || z == 10;
                    if is_corner {
                        for dy in 1..=4 {
                            set(blocks, x, base_y + dy, z, wood_mat);
                        }
                    } else if is_wall {
                        for dy in 1..=3 {
                            if (x == 7 || x == 8) && z == 10 {
                                if dy == 1 || dy == 2 {
                                    set(blocks, x, base_y + dy, z, BlockId::Door);
                                } else {
                                    set(blocks, x, base_y + dy, z, plank_mat);
                                }
                            } else if dy == 2 && ((x == 4 || x == 11) && z == 6) {
                                set(blocks, x, base_y + dy, z, BlockId::Glass);
                            } else {
                                set(blocks, x, base_y + dy, z, plank_mat);
                            }
                        }
                    }
                }
            }
            // Roof
            for x in 3..=12 {
                for z in 2..=11 {
                    set(blocks, x, base_y + 4, z, wood_mat);
                }
            }
            for x in 5..=10 {
                for z in 4..=9 {
                    set(blocks, x, base_y + 5, z, plank_mat);
                }
            }

            // Interior archive shelving lining the back wall
            for x in 5..=10 {
                set(blocks, x, base_y + 1, 4, BlockId::Planks);
                set(blocks, x, base_y + 2, 4, BlockId::Planks);
            }
            // Reading Table & Bench
            set(blocks, 6, base_y + 1, 7, BlockId::CraftingTable);
            set(blocks, 7, base_y + 1, 7, BlockId::WoodenSlab);
            set(blocks, 10, base_y + 1, 8, BlockId::Chest);
            set(blocks, 5, base_y + 3, 6, BlockId::Torch);
            set(blocks, 10, base_y + 3, 6, BlockId::Torch);

            // Library Entrance Lamppost
            set(blocks, 5, base_y + 1, 11, wood_mat);
            set(blocks, 5, base_y + 2, 11, wood_mat);
            set(blocks, 5, base_y + 3, 11, BlockId::Torch);
        }
    }
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
                let b = if hash_coords_3d(x, base_y + dy, z, seed) % 3 == 0 { BlockId::Ice } else { BlockId::Stone };
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

impl World {
    pub fn new() -> Self {
        Self::new_with_seed(DEFAULT_WORLD_SEED)
    }

    pub fn get_or_create_chest(&mut self, pos: IVec3) -> &mut ChestData {
        let seed = self.seed;
        let is_blacksmith = self.get_block(pos + IVec3::new(-1, 0, 0)) == BlockId::StoneSlab
            || self.get_block(pos + IVec3::new(-2, 0, 0)) == BlockId::Furnace
            || self.get_block(pos + IVec3::new(1, 0, 0)) == BlockId::Furnace;
        self.chests.entry(pos).or_insert_with(|| {
            if is_blacksmith {
                ChestData::generate_blacksmith_loot(seed, pos)
            } else {
                ChestData::generate_house_loot(seed, pos)
            }
        })
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

        let mut ruststone_lights = HashMap::new();
        let mut torches = HashSet::new();
        for chunk in chunks.values() {
            for ly in 0..CHUNK_SIZE_Y {
                for lz in 0..CHUNK_SIZE_Z {
                    for lx in 0..CHUNK_SIZE_X {
                        let b = chunk.get_local(lx, ly, lz);
                        if b == BlockId::Torch {
                            let wx = chunk.cx * CHUNK_SIZE_X + lx;
                            let wz = chunk.cz * CHUNK_SIZE_Z + lz;
                            torches.insert(IVec3::new(wx, ly, wz));
                        } else if matches!(b, BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLampLit | BlockId::RustStoneWireLit) {
                            let wx = chunk.cx * CHUNK_SIZE_X + lx;
                            let wz = chunk.cz * CHUNK_SIZE_Z + lz;
                            let rad_int = match b {
                                BlockId::RustStoneOre => (8.0, 0.75),
                                BlockId::RustStoneBlock => (9.0, 0.8),
                                BlockId::RustStoneLampLit => (14.0, 1.0),
                                _ => (7.0, 0.65),
                            };
                            ruststone_lights.insert(IVec3::new(wx, ly, wz), rad_int);
                        }
                    }
                }
            }
        }

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
            torches,
            ruststone_lights,
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
        let base_sun = 0.08 + 0.92 * ease;
        self.weather.modulate_daylight(base_sun)
    }

    /// Returns (sky_top, sky_bottom, fog_color) based on current time of day and weather conditions
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

        let mut final_top = lerp3(base_top, sunset_top, sunset_smooth * 0.7);
        let mut final_bot = lerp3(base_bot, sunset_bot, sunset_smooth);
        let mut final_fog = lerp3(base_fog, sunset_fog, sunset_smooth);

        // Weather overcast palette
        if self.weather.rain_strength > 0.001 {
            let rain_top = [0.28, 0.32, 0.38];
            let rain_bot = [0.42, 0.46, 0.52];
            let rain_fog = [0.40, 0.44, 0.50];

            let storm_top = [0.14, 0.16, 0.20];
            let storm_bot = [0.22, 0.25, 0.30];
            let storm_fog = [0.20, 0.22, 0.27];

            let r_strength = self.weather.rain_strength;
            let t_strength = self.weather.thunder_strength;

            let overcast_top = lerp3(rain_top, storm_top, t_strength);
            let overcast_bot = lerp3(rain_bot, storm_bot, t_strength);
            let overcast_fog = lerp3(rain_fog, storm_fog, t_strength);

            final_top = lerp3(final_top, overcast_top, r_strength * 0.75);
            final_bot = lerp3(final_bot, overcast_bot, r_strength * 0.85);
            final_fog = lerp3(final_fog, overcast_fog, r_strength * 0.90);
        }

        // Lightning flash effect spikes sky to brilliant electric white
        if self.weather.lightning_flash > 0.005 {
            let flash_col = [0.95, 0.97, 1.0];
            let f = self.weather.lightning_flash;
            final_top = lerp3(final_top, flash_col, f * 0.85);
            final_bot = lerp3(final_bot, flash_col, f * 0.95);
            final_fog = lerp3(final_fog, flash_col, f * 0.95);
        }

        (final_top, final_bot, final_fog)
    }

    /// Finds the highest non-air block Y coordinate at `(wx, wz)`.
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

    /// Checks if a world position has direct, unoccluded exposure to the sky.
    pub fn is_exposed_to_sky(&self, pos: IVec3) -> bool {
        self.get_highest_block_y(pos.x, pos.z) <= pos.y
    }

    /// Returns the biome at the given world horizontal coordinates.
    #[inline]
    pub fn get_biome_at(&self, wx: i32, wz: i32) -> Biome {
        get_biome_at(wx, wz, self.seed).0
    }

    /// Updates the dynamic weather state machine and returns a lightning strike event if one triggered.
    pub fn update_weather(&mut self, dt: f32, player_pos: Vec3) -> Option<LightningStrike> {
        let default_y = player_pos.y.max(60.0);
        let chunk_map = &self.chunks;

        self.weather.update(dt, player_pos, |x, z| {
            let (wx, wz) = (x.floor() as i32, z.floor() as i32);
            let (cx, cz, lx, lz) = World::world_to_chunk(wx, wz);
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

            if block == BlockId::RustStoneOre {
                self.ruststone_lights.insert(pos, (8.0, 0.75));
            } else if block == BlockId::RustStoneBlock {
                self.ruststone_lights.insert(pos, (9.0, 0.8));
            } else if block == BlockId::RustStoneLampLit {
                self.ruststone_lights.insert(pos, (14.0, 1.0));
            } else if block == BlockId::RustStoneWireLit {
                self.ruststone_lights.insert(pos, (7.0, 0.65));
            } else {
                self.ruststone_lights.remove(&pos);
            }

            if block == BlockId::PressurePlatePressed {
                self.active_pressure_plates.insert(pos);
            } else {
                self.active_pressure_plates.remove(&pos);
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

    pub fn index_chunk_lights(&mut self, chunk: &Chunk) {
        for ly in 0..CHUNK_SIZE_Y {
            for lz in 0..CHUNK_SIZE_Z {
                for lx in 0..CHUNK_SIZE_X {
                    let b = chunk.get_local(lx, ly, lz);
                    if b == BlockId::Torch {
                        let wx = chunk.cx * CHUNK_SIZE_X + lx;
                        let wz = chunk.cz * CHUNK_SIZE_Z + lz;
                        self.torches.insert(IVec3::new(wx, ly, wz));
                    } else if matches!(b, BlockId::RustStoneOre | BlockId::RustStoneBlock | BlockId::RustStoneLampLit | BlockId::RustStoneWireLit) {
                        let wx = chunk.cx * CHUNK_SIZE_X + lx;
                        let wz = chunk.cz * CHUNK_SIZE_Z + lz;
                        let rad_int = match b {
                            BlockId::RustStoneOre => (8.0, 0.75),
                            BlockId::RustStoneBlock => (9.0, 0.8),
                            BlockId::RustStoneLampLit => (14.0, 1.0),
                            _ => (7.0, 0.65),
                        };
                        self.ruststone_lights.insert(IVec3::new(wx, ly, wz), rad_int);
                    }
                }
            }
        }
    }

    pub fn update_ruststone_network(&mut self, center: IVec3, range: i32) {
        use std::collections::VecDeque;

        let mut sources = Vec::new();
        let mut wires = HashSet::new();
        let mut lamps = HashSet::new();
        let mut doors = HashSet::new();
        let mut tnts = Vec::new();

        for dy in -range..=range {
            for dz in -range..=range {
                for dx in -range..=range {
                    let pos = center + IVec3::new(dx, dy, dz);
                    let block = self.get_block(pos);
                    match block {
                        BlockId::RustStoneBlock
                        | BlockId::LeverOn
                        | BlockId::ButtonPressed
                        | BlockId::PressurePlatePressed => {
                            sources.push(pos);
                        }
                        BlockId::RustStone | BlockId::RustStoneWireLit => {
                            wires.insert(pos);
                        }
                        BlockId::RustStoneLamp | BlockId::RustStoneLampLit => {
                            lamps.insert(pos);
                        }
                        BlockId::Door | BlockId::DoorOpen => {
                            doors.insert(pos);
                        }
                        BlockId::Tnt => {
                            tnts.push(pos);
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut wire_power: HashMap<IVec3, u8> = HashMap::new();
        let mut powered_lamps: HashSet<IVec3> = HashSet::new();
        let mut powered_doors: HashSet<IVec3> = HashSet::new();
        let mut tnts_to_ignite: Vec<IVec3> = Vec::new();

        let mut queue = VecDeque::new();

        let neighbors = [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ];

        // 1. Direct power from active power sources
        for s_pos in sources {
            for &n_off in &neighbors {
                let n_pos = s_pos + n_off;
                if wires.contains(&n_pos) {
                    if wire_power.get(&n_pos).copied().unwrap_or(0) < 15 {
                        wire_power.insert(n_pos, 15);
                        queue.push_back((n_pos, 15));
                    }
                }
                if lamps.contains(&n_pos) {
                    powered_lamps.insert(n_pos);
                }
                if doors.contains(&n_pos) {
                    powered_doors.insert(n_pos);
                }
                if tnts.contains(&n_pos) {
                    tnts_to_ignite.push(n_pos);
                }
            }
        }

        // 2. Propagate power through wires
        while let Some((w_pos, pwr)) = queue.pop_front() {
            for &n_off in &neighbors {
                let n_pos = w_pos + n_off;
                if lamps.contains(&n_pos) {
                    powered_lamps.insert(n_pos);
                }
                if doors.contains(&n_pos) {
                    powered_doors.insert(n_pos);
                }
                if tnts.contains(&n_pos) {
                    tnts_to_ignite.push(n_pos);
                }
                if pwr > 1 && wires.contains(&n_pos) {
                    let next_pwr = pwr - 1;
                    if wire_power.get(&n_pos).copied().unwrap_or(0) < next_pwr {
                        wire_power.insert(n_pos, next_pwr);
                        queue.push_back((n_pos, next_pwr));
                    }
                }
            }
        }

        // 3. Ignite any TNT powered by circuit
        for tnt_pos in tnts_to_ignite {
            self.prime_tnt(tnt_pos);
        }

        // 4. Update wires
        for w_pos in wires {
            let pwr = wire_power.get(&w_pos).copied().unwrap_or(0);
            let cur = self.get_block(w_pos);
            if pwr > 0 && cur == BlockId::RustStone {
                self.set_block(w_pos, BlockId::RustStoneWireLit);
            } else if pwr == 0 && cur == BlockId::RustStoneWireLit {
                self.set_block(w_pos, BlockId::RustStone);
            }
        }

        // 5. Update lamps
        for l_pos in lamps {
            let is_pw = powered_lamps.contains(&l_pos);
            let cur = self.get_block(l_pos);
            if is_pw && cur == BlockId::RustStoneLamp {
                self.set_block(l_pos, BlockId::RustStoneLampLit);
            } else if !is_pw && cur == BlockId::RustStoneLampLit {
                self.set_block(l_pos, BlockId::RustStoneLamp);
            }
        }

        // 6. Update doors
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
        // 1. Decrement button timers
        let mut expired = Vec::new();
        for (&pos, timer) in self.button_timers.iter_mut() {
            *timer -= dt;
            if *timer <= 0.0 {
                expired.push(pos);
            }
        }
        for pos in expired {
            self.button_timers.remove(&pos);
            if self.get_block(pos) == BlockId::ButtonPressed {
                self.set_block(pos, BlockId::Button);
                self.ruststone_sound_events.push((pos, false));
                self.update_ruststone_network(pos, 16);
            }
        }

        // 2. Scan pressure plates
        let px = player_pos.x.floor() as i32;
        let py = player_pos.y.floor() as i32;
        let pz = player_pos.z.floor() as i32;

        let is_entity_on_plate = |pos: IVec3, ent: Vec3| -> bool {
            let ex = ent.x.floor() as i32;
            let ey = ent.y.floor() as i32;
            let ez = ent.z.floor() as i32;
            let ey_below = (ent.y - 0.2).floor() as i32;
            ex == pos.x && ez == pos.z && (ey == pos.y || ey_below == pos.y)
        };

        let mut changed_plates = Vec::new();

        // Check already active plates to see if entities walked off
        for &plate_pos in &self.active_pressure_plates {
            let entity_on = is_entity_on_plate(plate_pos, player_pos)
                || mob_positions.iter().any(|&m| is_entity_on_plate(plate_pos, m));
            if !entity_on && self.get_block(plate_pos) == BlockId::PressurePlatePressed {
                changed_plates.push((plate_pos, BlockId::PressurePlate, false));
            }
        }

        // Check nearby plates to see if entities walked onto them
        for dy in -3..=3 {
            for dz in -12..=12 {
                for dx in -12..=12 {
                    let test_pos = IVec3::new(px + dx, py + dy, pz + dz);
                    let block = self.get_block(test_pos);
                    if block == BlockId::PressurePlate {
                        let entity_on = is_entity_on_plate(test_pos, player_pos)
                            || mob_positions.iter().any(|&m| is_entity_on_plate(test_pos, m));
                        if entity_on {
                            changed_plates.push((test_pos, BlockId::PressurePlatePressed, true));
                        }
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
                    radius: 14.0,
                    intensity: 1.0,
                });
            }
        }
        for (&pos, &(radius, intensity)) in &self.ruststone_lights {
            let dx = (pos.x - (base_x + 8)).abs();
            let dz = (pos.z - (base_z + 8)).abs();
            if dx <= 24 && dz <= 24 {
                lights.push(LightSource {
                    pos,
                    radius,
                    intensity,
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
        {
            let mut received_chunks = Vec::new();
            if let Some(rx_mutex) = &self.chunk_res_rx {
                if let Ok(rx) = rx_mutex.lock() {
                    while let Ok(chunk) = rx.try_recv() {
                        received_chunks.push(chunk);
                    }
                }
            }
            for chunk in received_chunks {
                let cx = chunk.cx;
                let cz = chunk.cz;
                self.pending_chunks.remove(&(cx, cz));
                let ddx = cx - pcx;
                let ddz = cz - pcz;
                if ddx * ddx + ddz * ddz <= unload_dist_sq {
                    self.index_chunk_lights(&chunk);
                    self.chunks.insert((cx, cz), chunk);
                    crate::log_debug!("CHUNK", "Integrated chunk ({}, {}) into active world (loaded: {})", cx, cz, self.chunks.len());
                    if let Some(n) = self.chunks.get_mut(&(cx + 1, cz)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx - 1, cz)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx, cz + 1)) { n.dirty = true; }
                    if let Some(n) = self.chunks.get_mut(&(cx, cz - 1)) { n.dirty = true; }
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
                    self.index_chunk_lights(&chunk);
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

    #[test]
    fn test_all_biomes_have_distinct_color_filters() {
        let all_biomes = [
            Biome::DeepOcean,
            Biome::Ocean,
            Biome::SnowyTaiga,
            Biome::SnowyTundra,
            Biome::AutumnForest,
            Biome::Forest,
            Biome::CherryGrove,
            Biome::Plains,
            Biome::Swamp,
            Biome::Savanna,
            Biome::Desert,
            Biome::Mountain,
            Biome::Jungle,
            Biome::Badlands,
            Biome::DarkForest,
            Biome::Taiga,
            Biome::BirchForest,
            Biome::IceSpikes,
            Biome::MushroomIsland,
            Biome::Meadow,
            Biome::JaggedPeaks,
            Biome::FrozenPeaks,
            Biome::SnowySlopes,
        ];

        let mut seen = std::collections::HashSet::new();
        for &biome in &all_biomes {
            let filter = biome.color_filter();
            for &c in &filter {
                assert!(c > 0.5 && c < 1.5, "Biome {:?} filter channel out of reasonable bounds: {}", biome, c);
            }
            // Quantize to avoid floating point representations
            let key = (
                (filter[0] * 1000.0).round() as i32,
                (filter[1] * 1000.0).round() as i32,
                (filter[2] * 1000.0).round() as i32,
            );
            assert!(seen.insert(key), "Biome {:?} does not have a unique color filter! Duplicate: {:?}", biome, filter);
        }
        assert_eq!(seen.len(), 23, "All 23 biomes must have unique color filters");
    }

    #[test]
    fn test_smooth_mountain_transitions_no_sheer_biome_cliffs() {
        let seed = 424242;
        // Sample points across mountain and foothill biomes (Meadow, SnowySlopes, Mountain, JaggedPeaks, FrozenPeaks)
        for x in (-1500..=1500).step_by(10) {
            for z in (-1500..=1500).step_by(10) {
                let (b1, c1) = get_biome_at(x, z, seed);
                let (b2, c2) = get_biome_at(x + 1, z, seed);
                let is_m1 = matches!(b1, Biome::Meadow | Biome::SnowySlopes | Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks);
                let is_m2 = matches!(b2, Biome::Meadow | Biome::SnowySlopes | Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks);
                if is_m1 && is_m2 {
                    let h1 = get_height_at(x, z, seed, b1, c1);
                    let h2 = get_height_at(x + 1, z, seed, b2, c2);
                    let diff = (h1 - h2).abs();
                    assert!(diff <= 3, "Adjacent mountain blocks at ({}, {}) -> ({}, {}) should have smooth transition, found height diff {} ({} in {:?} vs {} in {:?})",
                        x, z, x + 1, z, diff, h1, b1, h2, b2);
                }
            }
        }
    }

    #[test]
    fn test_mountain_biome_block_continuity_no_checkerboard() {
        let seed = 2858965613;
        // Verify that in generated chunks with JaggedPeaks or FrozenPeaks, surface blocks don't form artificial checkerboards
        let chunk = Chunk::generate(29, 41, seed);
        let mut checker_2x2_count = 0;
        for lx in 0..15 {
            for lz in 0..15 {
                let wx = chunk.cx * CHUNK_SIZE_X + lx;
                let wz = chunk.cz * CHUNK_SIZE_Z + lz;
                let (b, c) = get_biome_at(wx, wz, seed);
                let h00 = get_height_at(wx, wz, seed, b, c);
                let h10 = get_height_at(wx + 1, wz, seed, b, c);
                let h01 = get_height_at(wx, wz + 1, seed, b, c);
                let h11 = get_height_at(wx + 1, wz + 1, seed, b, c);

                let b00 = chunk.get_local(lx, h00, lz);
                let b10 = chunk.get_local(lx + 1, h10, lz);
                let b01 = chunk.get_local(lx, h01, lz + 1);
                let b11 = chunk.get_local(lx + 1, h11, lz + 1);

                if b00 == b11 && b10 == b01 && b00 != b10 {
                    checker_2x2_count += 1;
                }
            }
        }
        assert_eq!(checker_2x2_count, 0, "Terrain surface should not contain 2x2 checkerboard block patterns in mountains");
    }

    #[test]
    fn test_ruststone_circuit_simulation() {
        let mut world = World::new_with_seed(12345);

        // Build a small test rig at world coords (0, 50, 0)
        let base_pos = IVec3::new(0, 50, 0);
        let lever_pos = IVec3::new(0, 51, 0);
        let wire_pos = IVec3::new(1, 51, 0);
        let lamp_pos = IVec3::new(2, 51, 0);

        world.set_block(base_pos, BlockId::Stone);
        world.set_block(IVec3::new(1, 50, 0), BlockId::Stone);
        world.set_block(IVec3::new(2, 50, 0), BlockId::Stone);

        world.set_block(lever_pos, BlockId::Lever);
        world.set_block(wire_pos, BlockId::RustStone);
        world.set_block(lamp_pos, BlockId::RustStoneLamp);

        // 1. Initial unpowered state
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(wire_pos), BlockId::RustStone);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        // 2. Power on lever -> wire and lamp should light up
        world.set_block(lever_pos, BlockId::LeverOn);
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(wire_pos), BlockId::RustStoneWireLit);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);
        assert!(world.ruststone_lights.contains_key(&lamp_pos));

        // 3. Power off lever -> wire and lamp should shut off
        world.set_block(lever_pos, BlockId::Lever);
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(wire_pos), BlockId::RustStone);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);
        assert!(!world.ruststone_lights.contains_key(&lamp_pos));

        // 4. Button activation and expiration
        world.set_block(lever_pos, BlockId::ButtonPressed);
        world.button_timers.insert(lever_pos, 0.5);
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(wire_pos), BlockId::RustStoneWireLit);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);

        // Tick past 0.5s with player far away
        world.update_ruststone(0.6, Vec3::new(100.0, 50.0, 100.0), &[]);
        assert_eq!(world.get_block(lever_pos), BlockId::Button);
        assert_eq!(world.get_block(wire_pos), BlockId::RustStone);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        // 5. Pressure plate activation by player standing on it
        world.set_block(wire_pos, BlockId::PressurePlate);
        world.update_ruststone_network(wire_pos, 10);
        assert_eq!(world.get_block(wire_pos), BlockId::PressurePlate);

        // Player steps on pressure plate at (1, 51, 0)
        world.update_ruststone(0.1, Vec3::new(1.2, 51.0, 0.2), &[]);
        assert_eq!(world.get_block(wire_pos), BlockId::PressurePlatePressed);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);

        // Player steps off
        world.update_ruststone(0.1, Vec3::new(50.0, 51.0, 50.0), &[]);
        assert_eq!(world.get_block(wire_pos), BlockId::PressurePlate);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);
    }

    #[test]
    fn test_spaced_villages_and_torch_lighting() {
        let seed: u32 = 1337;
        let village_region_size = 20i32;

        // Find a village in the world
        let mut found_village = None;
        for reg_x in -2..=2 {
            for reg_z in -2..=2 {
                let v_hash = hash_coords(reg_x, reg_z, seed.wrapping_add(8881337));
                let v_cx = reg_x * village_region_size + ((v_hash % (village_region_size - 4) as u32) as i32 + 2);
                let v_cz = reg_z * village_region_size + (((v_hash >> 16) % (village_region_size - 4) as u32) as i32 + 2);

                let v_center_wx = v_cx * CHUNK_SIZE_X + 8;
                let v_center_wz = v_cz * CHUNK_SIZE_Z + 8;
                let (v_biome, v_cont) = get_biome_at(v_center_wx, v_center_wz, seed);
                let v_height = get_height_at(v_center_wx, v_center_wz, seed, v_biome, v_cont);

                if matches!(v_biome, Biome::Plains | Biome::Meadow | Biome::Savanna | Biome::Desert | Biome::Taiga)
                    && v_height > 22 && v_height < CHUNK_SIZE_Y - 14
                {
                    found_village = Some((v_cx, v_cz, v_height, v_biome));
                    break;
                }
            }
            if found_village.is_some() {
                break;
            }
        }

        assert!(found_village.is_some(), "Should find at least one village in nearby regions");
        let (v_cx, v_cz, _v_height, _v_biome) = found_village.unwrap();

        // Generate the central well chunk and surrounding building chunks
        let center_chunk = Chunk::generate(v_cx, v_cz, seed);
        let cottage_chunk = Chunk::generate(v_cx + 1, v_cz, seed);
        let blacksmith_chunk = Chunk::generate(v_cx - 1, v_cz, seed);
        let farm_chunk = Chunk::generate(v_cx, v_cz + 1, seed);
        let _library_chunk = Chunk::generate(v_cx, v_cz - 1, seed);

        // Center chunk should contain the village well (water) and torches on lampposts
        let mut center_has_water = false;
        let mut center_has_torch = false;
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                for x in 0..CHUNK_SIZE_X {
                    let b = center_chunk.get_local(x, y, z);
                    if b == BlockId::Water { center_has_water = true; }
                    if b == BlockId::Torch { center_has_torch = true; }
                }
            }
        }
        assert!(center_has_water, "Center village chunk must contain the village well");
        assert!(center_has_torch, "Center village chunk must have torch lampposts");

        // Cottage chunk should contain a bed and crafting table
        let mut cottage_has_bed = false;
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                for x in 0..CHUNK_SIZE_X {
                    if cottage_chunk.get_local(x, y, z) == BlockId::Bed {
                        cottage_has_bed = true;
                    }
                }
            }
        }
        assert!(cottage_has_bed, "Cottage chunk must contain a bed");

        // Blacksmith chunk should contain furnace and chest
        let mut blacksmith_has_furnace = false;
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                for x in 0..CHUNK_SIZE_X {
                    if blacksmith_chunk.get_local(x, y, z) == BlockId::Furnace {
                        blacksmith_has_furnace = true;
                    }
                }
            }
        }
        assert!(blacksmith_has_furnace, "Blacksmith chunk must contain furnace");

        // Farm chunk should contain farmland crops
        let mut farm_has_crops = false;
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                for x in 0..CHUNK_SIZE_X {
                    let b = farm_chunk.get_local(x, y, z);
                    if b == BlockId::WheatCrop || b == BlockId::CarrotCrop || b == BlockId::PotatoCrop {
                        farm_has_crops = true;
                    }
                }
            }
        }
        assert!(farm_has_crops, "Farm chunk must contain crops");

        // Test World torch indexing: index_chunk_lights should index all torches in the chunk
        let mut world = World::new();
        let initial_torches = world.torches.len();
        world.index_chunk_lights(&center_chunk);
        assert!(world.torches.len() > initial_torches, "World should automatically index torches from generated chunks");

        // Test torch light radius is 14.0 in mesh job
        world.chunks.insert((v_cx, v_cz), center_chunk);
        let mesh_job = world.create_mesh_job((v_cx, v_cz)).expect("Mesh job should exist");
        let torch_lights: Vec<_> = mesh_job.lights.iter().filter(|l| l.radius == 14.0).collect();
        assert!(!torch_lights.is_empty(), "Mesh job must receive torch light sources with radius 14.0");
        for light in &torch_lights {
            assert_eq!(light.intensity, 1.0, "Torch intensity must be 1.0");
        }
    }
}

