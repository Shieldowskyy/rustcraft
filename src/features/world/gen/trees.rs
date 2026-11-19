//! Procedural tree, ice spike, and giant mushroom stampers for chunk population.
//! Writes canopy and trunk voxels directly into a chunk's flat `blocks` buffer.

use crate::features::blocks::BlockId;

use super::terrain::{set_blk, set_blk_if_air};

pub(super) fn place_trunk(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32, block: BlockId) {
    for i in 0..height {
        set_blk(blocks, lx, base_y + i, lz, block);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn place_ellipsoid_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, trunk_h: i32, rx: i32, ymax: i32, wx: i32, wz: i32, wy: i32, max_d: i32, leaf: BlockId) {
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

pub(super) fn place_pine_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_acacia_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_swamp_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_jungle_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, trunk_h: i32) {
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

pub(super) fn place_jungle_bush(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    set_blk(blocks, lx, base_y, lz, BlockId::Wood);
    for dx in -1..=1 {
        for dz in -1..=1 {
            for dy in 0..=1 {
                set_blk_if_air(blocks, lx + dx, base_y + dy, lz + dz, BlockId::Leaves);
            }
        }
    }
}

pub(super) fn place_dark_oak_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_birch_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_ice_spike(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32) {
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

pub(super) fn place_huge_red_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
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

pub(super) fn place_huge_brown_mushroom(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    place_trunk(blocks, lx, base_y, lz, 4, BlockId::Snow);
    for dx in -2..=2 {
        for dz in -2..=2 {
            set_blk_if_air(blocks, lx + dx, base_y + 4, lz + dz, BlockId::Mud);
        }
    }
}

