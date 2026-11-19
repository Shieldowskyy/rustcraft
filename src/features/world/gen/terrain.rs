//! Deterministic noise-driven biome classification, terrain heightmap, and box fill primitives.
//! Maps world `(wx, wz)` coordinates and seed into biome/elevation values.

use crate::core::noise::fractal_noise2;
use crate::features::blocks::BlockId;

use crate::features::world::biome::Biome;
use crate::features::world::chunk::Chunk;
use crate::features::world::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

#[inline(always)]
pub fn get_biome_at(wx: i32, wz: i32, seed: u32) -> (Biome, f32) {
    let (fx, fz) = (wx as f32, wz as f32);
    let continentalness = fractal_noise2(fx * 0.0012, fz * 0.0012, seed.wrapping_add(2500), 4, 0.5);
    let temp = fractal_noise2(fx * 0.0010, fz * 0.0010, seed.wrapping_add(500), 4, 0.5);
    let humidity = fractal_noise2(fx * 0.0010, fz * 0.0010, seed.wrapping_add(1500), 4, 0.5);
    let rarity = fractal_noise2(fx * 0.0014, fz * 0.0014, seed.wrapping_add(4500), 4, 0.5);
    let river = fractal_noise2(fx * 0.0030, fz * 0.0030, seed.wrapping_add(6500), 2, 0.5);

    let biome = if continentalness < -0.38 {
        if rarity > 0.38 { Biome::MushroomIsland } else { Biome::DeepOcean }
    } else if continentalness < -0.16 {
        if continentalness < -0.28 && rarity > 0.40 { Biome::MushroomIsland } else { Biome::Ocean }
    } else if river.abs() < 0.035 {
        if temp < -0.30 { Biome::FrozenRiver } else { Biome::River }
    } else if continentalness < -0.105 {
        if temp < -0.30 { Biome::ColdBeach }
        else if rarity > 0.40 { Biome::StoneBeach }
        else { Biome::Beach }
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
        if humidity > 0.16 && rarity > 0.28 { Biome::MegaTaiga }
        else if humidity > 0.16 { Biome::AutumnForest }
        else if humidity > -0.05 { Biome::Taiga } else { Biome::Plains }
    } else if temp < 0.22 {
        if humidity > 0.28 {
            if rarity > 0.56 { Biome::FlowerForest }
            else if rarity > 0.16 { Biome::DarkForest }
            else if temp < 0.10 { Biome::Swamp }
            else { Biome::Forest }
        } else if humidity > 0.10 {
            if rarity < -0.18 { Biome::BirchForest } else { Biome::Forest }
        } else if humidity > -0.04 && temp > 0.09 { Biome::CherryGrove }
        else if rarity > 0.45 { Biome::SunflowerPlains }
        else { Biome::Plains }
    } else if humidity > 0.30 { Biome::Jungle }
    else if humidity > 0.14 { Biome::JungleEdge }
    else if humidity < -0.18 && rarity > 0.12 {
        if rarity > 0.30 { Biome::MesaBryce }
        else if rarity > 0.24 { Biome::MesaPlateauForest }
        else if rarity > 0.20 { Biome::MesaPlateau }
        else if rarity > 0.16 { Biome::Mesa }
        else { Biome::Badlands }
    }
    else if humidity < -0.08 { Biome::Desert }
    else if humidity < 0.16 {
        if continentalness > 0.12 && rarity > 0.30 { Biome::SavannaPlateau }
        else { Biome::Savanna }
    } else { Biome::Forest };

    (biome, continentalness)
}

#[inline(always)]
pub fn get_height_at(wx: i32, wz: i32, seed: u32, biome: Biome, continentalness: f32) -> i32 {
    let n = fractal_noise2(wx as f32 * 0.012, wz as f32 * 0.012, seed, 4, 0.5);
    let broad = fractal_noise2(wx as f32 * 0.0035, wz as f32 * 0.0035, seed.wrapping_add(7103), 3, 0.55);
    let detail = fractal_noise2(wx as f32 * 0.020, wz as f32 * 0.020, seed.wrapping_add(7109), 2, 0.5);
    let m_t = ((continentalness - 0.34) / 0.26).clamp(0.0, 1.0);
    let lift = m_t * m_t * (3.0 - 2.0 * m_t);
    let ridge_noise = fractal_noise2(wx as f32 * 0.0045, wz as f32 * 0.0045, seed.wrapping_add(7117), 3, 0.55);
    let ridge = 1.0 - ridge_noise.abs();

    let h = match biome {
        Biome::DeepOcean => 7.0 + n * 3.0 + broad * 1.5,
        Biome::Ocean => 14.0 + n * 3.5 + broad * 2.0,
        Biome::Beach | Biome::ColdBeach | Biome::StoneBeach => 20.0 + broad * 1.5 + n * 2.0,
        Biome::River | Biome::FrozenRiver => 25.0 + broad * 2.5 + n * 2.5,
        Biome::MushroomIsland => 25.0 + broad * 2.0 + n * 2.5,
        Biome::Swamp => 23.0 + broad * 1.5 + n * 2.0,
        Biome::Meadow | Biome::SnowySlopes | Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks => {
            27.0 + broad * 3.0 + n * 2.0 + lift * (ridge * 31.0 + detail * 3.0)
        }
        Biome::MesaBryce => 30.0 + broad * 2.0 + n * 2.5 + ridge * 8.0 + lift * (ridge * 25.0 + detail * 4.0),
        Biome::SavannaPlateau |         Biome::Mesa | Biome::MesaPlateau | Biome::MesaPlateauForest => {
            31.0 + broad * 1.5 + n * 1.5 + lift * ridge * 8.0
        }
        Biome::Badlands => ((30.0 + broad * 2.5 + n * 4.0) / 3.0).floor() * 3.0 + 1.0,
        Biome::Desert => 26.0 + broad * 3.0 + n * 4.0,
        Biome::Savanna => 27.0 + broad * 3.0 + n * 3.5,
        Biome::Jungle | Biome::JungleEdge => 27.0 + broad * 3.5 + n * 4.0,
        Biome::DarkForest | Biome::FlowerForest => 27.0 + broad * 3.5 + n * 3.5,
        Biome::BirchForest => 27.0 + broad * 3.5 + n * 3.5,
        Biome::AutumnForest | Biome::Forest | Biome::CherryGrove | Biome::MegaTaiga => 27.0 + broad * 3.5 + n * 3.5,
        Biome::SunflowerPlains | Biome::Taiga | Biome::SnowyTaiga | Biome::SnowyTundra | Biome::IceSpikes | Biome::Plains => {
            27.0 + broad * 3.0 + n * 3.0
        }
    };
    let river_noise = fractal_noise2(wx as f32 * 0.0030, wz as f32 * 0.0030, seed.wrapping_add(6500), 2, 0.5);
    let river_t = ((0.10 - river_noise.abs()) / 0.065).clamp(0.0, 1.0);
    let river_cut = river_t * river_t * (3.0 - 2.0 * river_t) * 10.0;
    let h = if continentalness >= -0.16 { h - river_cut } else { h };
    (h as i32).clamp(1, CHUNK_SIZE_Y - 2)
}

#[inline(always)]
pub(super) fn set_blk(blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId) {
    if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
        blocks[Chunk::index(x, y, z)] = b.to_u8();
    }
}

#[inline(always)]
pub(super) fn set_blk_if_air(blocks: &mut [u8], x: i32, y: i32, z: i32, b: BlockId) {
    if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
        let idx = Chunk::index(x, y, z);
        if blocks[idx] == BlockId::Air.to_u8() {
            blocks[idx] = b.to_u8();
        }
    }
}

pub(super) fn fill_box(blocks: &mut [u8], x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32, b: BlockId) {
    for x in x0..=x1 {
        for z in z0..=z1 {
            for y in y0..=y1 {
                set_blk(blocks, x, y, z, b);
            }
        }
    }
}


pub fn hash_coords(x: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((z as u32).wrapping_mul(668265263)).wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

pub(super) fn hash_coords_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(1103515245)).wrapping_add((z as u32).wrapping_mul(668265263)).wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

