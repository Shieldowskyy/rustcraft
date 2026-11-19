//! Pure CPU chunk meshing constants (subchunk height, count, triangle cap).
//! Consumed across culling, neighbor cache, and async mesh queue modules.

use crate::features::world::CHUNK_SIZE_Y;

pub const SUBCHUNK_SIZE: i32 = 16;
pub const NUM_SUBCHUNKS: usize = (CHUNK_SIZE_Y / SUBCHUNK_SIZE) as usize; // 4
pub const VITA_MAX_MESH_TRIANGLES: usize = 6144;
pub const DEFAULT_MAX_MESH_TRIANGLES: usize = VITA_MAX_MESH_TRIANGLES;
