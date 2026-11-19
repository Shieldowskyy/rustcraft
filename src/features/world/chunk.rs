//! `Chunk` voxel storage grid (16x64x16) and bounds-checked local block accessors.
//! Mutates block array elements via copy-on-write `Arc` and tracks the chunk `dirty` remesh flag.

use std::sync::Arc;

use crate::features::blocks::BlockId;

use super::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Arc<Vec<u8>>,
    pub dirty: bool,
}

impl Chunk {
    #[inline]
    pub fn new(cx: i32, cz: i32, blocks: Vec<u8>, dirty: bool) -> Self {
        Self {
            cx,
            cz,
            blocks: Arc::new(blocks),
            dirty,
        }
    }

    // DLACZEGO: 16x64x16 flat indexing packs (x << 10) | (y << 4) | z for cache-friendly vertical columns.
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
        if (x as u32) < CHUNK_SIZE_X as u32 && (y as u32) < CHUNK_SIZE_Y as u32 && (z as u32) < CHUNK_SIZE_Z as u32 {
            Arc::make_mut(&mut self.blocks)[Chunk::index(x, y, z)] = b.to_u8();
            self.dirty = true;
        }
    }
}

