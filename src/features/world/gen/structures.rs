//! Procedural ruins, desert wells, shrines, dungeons, and forest cabins.
//! Stamps pre-designed multi-block landmarks into generated chunks.

use crate::features::blocks::BlockId;

use super::terrain::{fill_box, hash_coords_3d, set_blk};
use super::trees::place_trunk;

pub(super) fn place_coral_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
    for &(dx, dz) in &[(-2, -2), (-2, 2), (2, -2), (2, 2), (0, -3), (3, 0), (-3, 0), (0, 3)] {
        let (x, z) = (cx + dx, cz + dz);
        let col_h = 2 + (hash_coords_3d(x, 77, z, seed) % 5) as i32;
        let b = if (dx + dz) % 2 == 0 { BlockId::Stone } else { BlockId::Sand };
        place_trunk(blocks, x, by, z, col_h + 1, b);
    }
    fill_box(blocks, cx - 1, cx + 1, by, by + 1, cz - 1, cz + 1, BlockId::Stone);
    set_blk(blocks, cx, by + 2, cz, BlockId::Lava);
}

pub(super) fn place_desert_well(blocks: &mut [u8], cx: i32, by: i32, cz: i32) {
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

pub(super) fn place_stone_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
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

pub(super) fn place_forest_cabin(blocks: &mut [u8], cx: i32, by: i32, cz: i32) {
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


pub(super) fn place_mountain_shrine(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
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

pub(super) fn place_frozen_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
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

pub(super) fn place_desert_buried_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, _seed: u32) {
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

pub(super) fn place_overgrown_temple_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
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

pub(super) fn place_cave_dungeon_ruin(blocks: &mut [u8], cx: i32, by: i32, cz: i32, seed: u32) {
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

