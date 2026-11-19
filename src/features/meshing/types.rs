//! Pure CPU mesh geometry data types (`Vertex`, `Triangle`, `LightSource`, `ChunkMeshJob`).
//! Thread-safe snapshot structures decoupled from GPU buffers and OS threading.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::math::{IVec3, Vec3};
use crate::features::rendering::atlas::TextureId;
use crate::features::world::{Chunk, World, WorldGeneratorType, CHUNK_SIZE_X, CHUNK_SIZE_Z};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: (f32, f32),
    pub ao: f32,
    pub sky_light: f32,
    pub block_light: f32,
}

impl Vertex {
    #[inline(always)]
    pub fn new(pos: Vec3, uv: (f32, f32), ao: f32) -> Self {
        Self {
            pos,
            uv,
            ao,
            sky_light: 1.0,
            block_light: 0.0,
        }
    }

    #[inline(always)]
    pub fn lit(pos: Vec3, uv: (f32, f32), ao: f32, sky_light: f32, block_light: f32) -> Self {
        Self {
            pos,
            uv,
            ao,
            sky_light,
            block_light,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
    pub texture: TextureId,
}

#[derive(Copy, Clone)]
pub struct LightSource {
    pub pos: IVec3,
    pub radius: f32,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct ChunkMeshJob {
    pub key: (i32, i32),
    pub center_blocks: Arc<Vec<u8>>,
    pub east_blocks: Option<Arc<Vec<u8>>>,
    pub west_blocks: Option<Arc<Vec<u8>>>,
    pub south_blocks: Option<Arc<Vec<u8>>>,
    pub north_blocks: Option<Arc<Vec<u8>>>,
    pub se_blocks: Option<Arc<Vec<u8>>>,
    pub sw_blocks: Option<Arc<Vec<u8>>>,
    pub ne_blocks: Option<Arc<Vec<u8>>>,
    pub nw_blocks: Option<Arc<Vec<u8>>>,
    pub seed: u32,
    pub generator_type: WorldGeneratorType,
    pub lights: Vec<LightSource>,
    pub fluid_map: std::collections::HashMap<IVec3, u8>,
}

impl ChunkMeshJob {
    /// Creates an owned, thread-safe snapshot of the chunk and its 8 neighbors.
    /// The worker never borrows `World`, so meshing cannot stall or race the game thread.
    pub fn from_world(chunk: &Chunk, world: &World) -> Self {
        let (cx, cz) = (chunk.cx, chunk.cz);
        let copy_blocks = |x: i32, z: i32| {
            world.chunks.get(&(x, z)).map(|neighbor| Arc::clone(&neighbor.blocks))
        };
        let base_x = cx * CHUNK_SIZE_X;
        let base_z = cz * CHUNK_SIZE_Z;

        let mut lights = Vec::new();
        for &pos in &world.torches {
            let dx = (pos.x - (base_x + 8)).abs();
            let dz = (pos.z - (base_z + 8)).abs();
            if dx <= 24 && dz <= 24 {
                lights.push(LightSource { pos, radius: 14.0, intensity: 1.0 });
            }
        }
        for (&pos, &(radius, intensity)) in &world.ruststone_lights {
            let dx = (pos.x - (base_x + 8)).abs();
            let dz = (pos.z - (base_z + 8)).abs();
            if dx <= 24 && dz <= 24 {
                lights.push(LightSource { pos, radius, intensity });
            }
        }

        Self {
            key: (cx, cz),
            center_blocks: Arc::clone(&chunk.blocks),
            east_blocks: copy_blocks(cx + 1, cz),
            west_blocks: copy_blocks(cx - 1, cz),
            south_blocks: copy_blocks(cx, cz + 1),
            north_blocks: copy_blocks(cx, cz - 1),
            se_blocks: copy_blocks(cx + 1, cz + 1),
            sw_blocks: copy_blocks(cx - 1, cz + 1),
            ne_blocks: copy_blocks(cx + 1, cz - 1),
            nw_blocks: copy_blocks(cx - 1, cz - 1),
            seed: world.seed,
            generator_type: world.generator_type,
            lights,
            fluid_map: HashMap::new(),
        }
    }
}

