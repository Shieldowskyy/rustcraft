//! Multi-chunk village component stampers (well, cottage, blacksmith, farm, library).
//! Adapts palette to biome (desert sand vs temperate wood/cobble).

use crate::features::blocks::BlockId;

use crate::features::world::biome::{Biome, VillageComponent};
use crate::features::world::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Z};

use super::terrain::{fill_box, set_blk, set_blk_if_air};
use super::trees::place_trunk;

pub(super) fn place_village_component(blocks: &mut [u8], _cx: i32, by: i32, _cz: i32, biome: Biome, _seed: u32, comp: VillageComponent) {
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
            set_blk(blocks, 11, by + 1, 9, BlockId::Bed);
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
            set_blk(blocks, 6, by + 1, 5, BlockId::Bed);
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

