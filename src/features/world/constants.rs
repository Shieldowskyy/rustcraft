//! Shared chunk dimension, lighting radius, view distance, and cardinal neighbor constants.
//! Used across world generation, meshing, physics, and simulation without magic numbers.

use crate::core::math::IVec3;

pub const CHUNK_SIZE_X: i32 = 16;
pub const CHUNK_SIZE_Y: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 16;

pub const MAX_BLOCK_LIGHT_RADIUS: i32 = 14;

pub const VIEW_DISTANCE: i32 = 4;
pub const DEFAULT_WORLD_SEED: u32 = 1337;

pub const CARDINAL_OFFSETS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

