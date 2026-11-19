//! Procedural chunk population pipeline (`Chunk::generate`, `generate_superflat`, `generate_with_type`).
//! Orchestrates base height/biome generation, structure selection, and vegetation stamping.

mod structures;
mod terrain;
mod trees;
mod village;

pub use terrain::{get_biome_at, get_height_at, hash_coords};

use crate::core::noise::{fractal_noise2, fractal_noise3};
use crate::features::blocks::BlockId;

use super::biome::{Biome, VillageComponent, WorldGeneratorType};
use super::chunk::Chunk;
use super::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

use structures::{
    place_cave_dungeon_ruin, place_coral_ruin, place_desert_buried_ruin, place_desert_well,
    place_forest_cabin, place_frozen_ruin, place_mountain_shrine, place_overgrown_temple_ruin,
    place_stone_ruin,
};
use terrain::hash_coords_3d;
use trees::{
    place_acacia_tree, place_birch_tree, place_dark_oak_tree, place_ellipsoid_tree,
    place_huge_brown_mushroom, place_huge_red_mushroom, place_ice_spike, place_jungle_bush,
    place_jungle_tree, place_pine_tree, place_swamp_tree, place_trunk,
};
use village::place_village_component;

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

        let village_comp = if matches!(v_biome, Biome::Plains | Biome::SunflowerPlains | Biome::Meadow | Biome::Savanna | Biome::SavannaPlateau | Biome::Desert | Biome::Taiga)
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
            let is_desert = matches!(c_biome, Biome::Desert | Biome::Badlands | Biome::Mesa | Biome::MesaPlateau | Biome::MesaPlateauForest | Biome::MesaBryce);
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
            } else if matches!(c_biome, Biome::Jungle | Biome::JungleEdge | Biome::DarkForest | Biome::Swamp) && chunk_hash % 67 == 0 {
                structure = Some(Structure::OvergrownTempleRuin(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Mountain | Biome::SnowyTundra | Biome::Taiga) && chunk_hash % 73 == 0 {
                structure = Some(Structure::StoneRuins(8, center_height, 8));
                struct_exclusion = (3, 13, 3, 13);
            } else if matches!(c_biome, Biome::Forest | Biome::FlowerForest | Biome::AutumnForest | Biome::DarkForest | Biome::BirchForest | Biome::MegaTaiga) && chunk_hash % 79 == 0 {
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
                let is_mountain = matches!(biome, Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes | Biome::MesaBryce);

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
                            if y == 20 && biome.has_snow() {
                                BlockId::Ice
                            } else {
                                BlockId::Water
                            }
                        } else {
                            BlockId::Air
                        }
                    } else if y == height {
                        match biome {
                            Biome::DeepOcean | Biome::Ocean | Biome::Desert | Biome::Beach => BlockId::Sand,
                            Biome::StoneBeach => BlockId::Stone,
                            Biome::River | Biome::FrozenRiver => {
                                if hash_coords(wx, wz, seed.wrapping_add(51893)) % 3 == 0 { BlockId::Sand } else { BlockId::Gravel }
                            }
                            Biome::Swamp => if height <= 20 { BlockId::Mud } else { BlockId::Grass },
                            Biome::MushroomIsland => BlockId::Mud,
                            Biome::Badlands | Biome::Mesa | Biome::MesaPlateau | Biome::MesaPlateauForest | Biome::MesaBryce => match (height / 2) % 3 { 0 => BlockId::Sand, 1 => BlockId::Stone, _ => BlockId::Mud },
                            Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::SnowySlopes | Biome::ColdBeach => BlockId::Snow,
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
                            Biome::DeepOcean | Biome::Ocean | Biome::Beach | Biome::ColdBeach => BlockId::Sand,
                            Biome::StoneBeach | Biome::River | Biome::FrozenRiver => BlockId::Gravel,
                            Biome::Desert => if y <= height - 3 { BlockId::Sandstone } else { BlockId::Sand },
                            Biome::Badlands | Biome::Mesa | Biome::MesaPlateau | Biome::MesaPlateauForest | Biome::MesaBryce => match (y / 2) % 3 { 0 => BlockId::Sand, 1 => BlockId::Stone, _ => BlockId::Mud },
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
                        if matches!(biome, Biome::Badlands | Biome::Mesa | Biome::MesaPlateau | Biome::MesaPlateauForest | Biome::MesaBryce) && y >= 14 {
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
                            Biome::Forest | Biome::FlowerForest if h % 43 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::BirchForest if h % 41 == 0 => decorations.push(Decoration::BirchTree(lx, by, lz)),
                            Biome::DarkForest => {
                                if h % 29 == 0 { decorations.push(Decoration::DarkOakTree(lx, by, lz)); }
                                else if h % 83 == 0 { decorations.push(Decoration::HugeRedMushroom(lx, by, lz)); }
                                else if h % 89 == 0 { decorations.push(Decoration::HugeBrownMushroom(lx, by, lz)); }
                            }
                            Biome::Jungle | Biome::JungleEdge => {
                                if h % 43 == 0 { decorations.push(Decoration::JungleTree(lx, by, lz, 10 + (h % 7) as i32)); }
                                else if h % 31 == 0 { decorations.push(Decoration::JungleBush(lx, by, lz)); }
                            }
                            Biome::MushroomIsland => {
                                if h % 37 == 0 { decorations.push(Decoration::HugeRedMushroom(lx, by, lz)); }
                                else if h % 41 == 0 { decorations.push(Decoration::HugeBrownMushroom(lx, by, lz)); }
                            }
                            Biome::CherryGrove if h % 53 == 0 => decorations.push(Decoration::CherryTree(lx, by, lz)),
                            Biome::Swamp if h % 67 == 0 => decorations.push(Decoration::SwampTree(lx, by, lz)),
                            Biome::Savanna | Biome::SavannaPlateau if h % 163 == 0 => decorations.push(Decoration::AcaciaTree(lx, by, lz)),
                            Biome::Plains | Biome::SunflowerPlains if h % 149 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::Meadow if h % 199 == 0 => decorations.push(Decoration::Tree(lx, by, lz)),
                            Biome::SnowySlopes | Biome::MegaTaiga if h % 127 == 0 => decorations.push(Decoration::PineTree(lx, by, lz)),
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

    pub fn generate_superflat(cx: i32, cz: i32) -> Chunk {
        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];
        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                blocks[Chunk::index(lx, 0, lz)] = BlockId::Bedrock.to_u8();
                blocks[Chunk::index(lx, 1, lz)] = BlockId::Dirt.to_u8();
                blocks[Chunk::index(lx, 2, lz)] = BlockId::Dirt.to_u8();
                blocks[Chunk::index(lx, 3, lz)] = BlockId::Grass.to_u8();
            }
        }
        Chunk {
            cx,
            cz,
            blocks,
            dirty: true,
        }
    }

    pub fn generate_with_type(cx: i32, cz: i32, seed: u32, generator_type: WorldGeneratorType) -> Chunk {
        match generator_type {
            WorldGeneratorType::Default => Self::generate(cx, cz, seed),
            WorldGeneratorType::Superflat => Self::generate_superflat(cx, cz),
        }
    }
}

