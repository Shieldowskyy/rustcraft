//! Cached chunk-neighborhood voxel, heightmap, light, and biome reader (`ChunkNeighborCache`).
//! Accelerates face culling, smooth AO/light sampling, and worker-thread meshing without borrowing `World`.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::math::IVec3;
use crate::features::blocks::BlockId;
use crate::features::world::{get_biome_at, Biome, Chunk, World, WorldGeneratorType, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

use super::types::{ChunkMeshJob, LightSource};

pub struct ChunkNeighborCache<'a> {
    pub center_blocks: &'a [u8],
    pub east_blocks: Option<&'a [u8]>,   // +X
    pub west_blocks: Option<&'a [u8]>,   // -X
    pub south_blocks: Option<&'a [u8]>,  // +Z
    pub north_blocks: Option<&'a [u8]>,  // -Z
    pub se_blocks: Option<&'a [u8]>,     // +X, +Z
    pub sw_blocks: Option<&'a [u8]>,     // -X, +Z
    pub ne_blocks: Option<&'a [u8]>,     // +X, -Z
    pub nw_blocks: Option<&'a [u8]>,     // -X, -Z
    pub base_x: i32,
    pub base_z: i32,
    pub seed: u32,
    pub generator_type: WorldGeneratorType,
    pub heightmap: [u8; 18 * 18],
    pub leaf_heightmap: [u8; 18 * 18],
    pub lights: Vec<LightSource>,
    pub fluid_map: Option<&'a std::collections::HashMap<IVec3, u8>>,
    block_light_cache: RefCell<Vec<f32>>,
    biome_cache: RefCell<HashMap<(i32, i32), Biome>>,
}

impl<'a> ChunkNeighborCache<'a> {
    pub fn new(chunk: &'a Chunk, world: &'a World) -> Self {
        let cx = chunk.cx;
        let cz = chunk.cz;

        let east_blocks = world.chunks.get(&(cx + 1, cz)).map(|c| c.blocks.as_slice());
        let west_blocks = world.chunks.get(&(cx - 1, cz)).map(|c| c.blocks.as_slice());
        let south_blocks = world.chunks.get(&(cx, cz + 1)).map(|c| c.blocks.as_slice());
        let north_blocks = world.chunks.get(&(cx, cz - 1)).map(|c| c.blocks.as_slice());
        let se_blocks = world.chunks.get(&(cx + 1, cz + 1)).map(|c| c.blocks.as_slice());
        let sw_blocks = world.chunks.get(&(cx - 1, cz + 1)).map(|c| c.blocks.as_slice());
        let ne_blocks = world.chunks.get(&(cx + 1, cz - 1)).map(|c| c.blocks.as_slice());
        let nw_blocks = world.chunks.get(&(cx - 1, cz - 1)).map(|c| c.blocks.as_slice());

        let base_x = cx * CHUNK_SIZE_X;
        let base_z = cz * CHUNK_SIZE_Z;

        let mut lights = Vec::new();
        for &t_pos in &world.torches {
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
        for (&pos, &(radius, intensity)) in &world.ruststone_lights {
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

        let mut cache = Self {
            center_blocks: chunk.blocks.as_slice(),
            east_blocks,
            west_blocks,
            south_blocks,
            north_blocks,
            se_blocks,
            sw_blocks,
            ne_blocks,
            nw_blocks,
            base_x,
            base_z,
            seed: world.seed,
            generator_type: world.generator_type,
            heightmap: [0u8; 18 * 18],
            leaf_heightmap: [0u8; 18 * 18],
            lights,
            fluid_map: None,
            block_light_cache: RefCell::new(vec![
                -1.0;
                (CHUNK_SIZE_X as usize + 2)
                    * (CHUNK_SIZE_Y as usize + 2)
                    * (CHUNK_SIZE_Z as usize + 2)
            ]),
            biome_cache: RefCell::new(HashMap::with_capacity(256)),
        };

        cache.populate_heightmaps();
        cache
    }

    pub fn from_job(job: &'a ChunkMeshJob) -> Self {
        let (cx, cz) = job.key;
        let base_x = cx * CHUNK_SIZE_X;
        let base_z = cz * CHUNK_SIZE_Z;

        let mut cache = Self {
            center_blocks: job.center_blocks.as_slice(),
            east_blocks: job.east_blocks.as_deref(),
            west_blocks: job.west_blocks.as_deref(),
            south_blocks: job.south_blocks.as_deref(),
            north_blocks: job.north_blocks.as_deref(),
            se_blocks: job.se_blocks.as_deref(),
            sw_blocks: job.sw_blocks.as_deref(),
            ne_blocks: job.ne_blocks.as_deref(),
            nw_blocks: job.nw_blocks.as_deref(),
            base_x,
            base_z,
            seed: job.seed,
            generator_type: job.generator_type,
            heightmap: [0u8; 18 * 18],
            leaf_heightmap: [0u8; 18 * 18],
            lights: job.lights.clone(),
            fluid_map: Some(&job.fluid_map),
            block_light_cache: RefCell::new(vec![
                -1.0;
                (CHUNK_SIZE_X as usize + 2)
                    * (CHUNK_SIZE_Y as usize + 2)
                    * (CHUNK_SIZE_Z as usize + 2)
            ]),
            biome_cache: RefCell::new(HashMap::with_capacity(256)),
        };

        cache.populate_heightmaps();
        cache
    }

    fn populate_heightmaps(&mut self) {
        for lx in -1..=16 {
            for lz in -1..=16 {
                let mut h = 0u8;
                let mut leaf_h = 0u8;
                for ly in (0..CHUNK_SIZE_Y).rev() {
                    let b = self.sample_block(lx, ly, lz);
                    if b.is_leaf() && leaf_h == 0 {
                        leaf_h = (ly + 1) as u8;
                    }
                    if b.is_opaque() {
                        h = (ly + 1) as u8;
                        break;
                    }
                }
                let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
                self.heightmap[idx] = h;
                self.leaf_heightmap[idx] = leaf_h;
            }
        }
    }

    #[inline(always)]
    pub(super) fn sample_voxel_lights(&self, wx: i32, wy: i32, wz: i32, y_offset: f32) -> (f32, f32) {
        let lx = wx - self.base_x;
        let lz = wz - self.base_z;
        let sky = self.sample_sky_light(lx, wy, lz);
        let block_l = self.sample_block_light(wx as f32 + 0.5, wy as f32 + y_offset, wz as f32 + 0.5);
        (sky, block_l)
    }

    #[inline(always)]
    pub fn fluid_level(&self, pos: IVec3) -> u8 {
        if let Some(fm) = self.fluid_map {
            fm.get(&pos).copied().unwrap_or(8)
        } else {
            8
        }
    }

    #[inline(always)]
    pub fn sample_block(&self, lx: i32, ly: i32, lz: i32) -> BlockId {
        if (ly as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        if (lx as u32) < CHUNK_SIZE_X as u32 && (lz as u32) < CHUNK_SIZE_Z as u32 {
            return BlockId::from_u8(self.center_blocks[((lx << 10) | (ly << 4) | lz) as usize]);
        }
        let id = if lx >= CHUNK_SIZE_X {
            if lz >= CHUNK_SIZE_Z {
                self.se_blocks.map(|b| b[(((lx - CHUNK_SIZE_X) << 10) | (ly << 4) | (lz - CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
            } else if lz < 0 {
                self.ne_blocks.map(|b| b[(((lx - CHUNK_SIZE_X) << 10) | (ly << 4) | (lz + CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
            } else {
                self.east_blocks.map(|b| b[(((lx - CHUNK_SIZE_X) << 10) | (ly << 4) | lz) as usize]).unwrap_or(0)
            }
        } else if lx < 0 {
            if lz >= CHUNK_SIZE_Z {
                self.sw_blocks.map(|b| b[(((lx + CHUNK_SIZE_X) << 10) | (ly << 4) | (lz - CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
            } else if lz < 0 {
                self.nw_blocks.map(|b| b[(((lx + CHUNK_SIZE_X) << 10) | (ly << 4) | (lz + CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
            } else {
                self.west_blocks.map(|b| b[(((lx + CHUNK_SIZE_X) << 10) | (ly << 4) | lz) as usize]).unwrap_or(0)
            }
        } else if lz >= CHUNK_SIZE_Z {
            self.south_blocks.map(|b| b[((lx << 10) | (ly << 4) | (lz - CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
        } else if lz < 0 {
            self.north_blocks.map(|b| b[((lx << 10) | (ly << 4) | (lz + CHUNK_SIZE_Z)) as usize]).unwrap_or(0)
        } else {
            0
        };
        BlockId::from_u8(id)
    }

    #[inline(always)]
    pub fn solid_at(&self, lx: i32, ly: i32, lz: i32) -> bool {
        self.sample_block(lx, ly, lz).is_opaque()
    }

    #[inline(always)]
    pub fn get_heights(&self, lx: i32, lz: i32) -> (i32, i32) {
        if lx >= -1 && lx <= 16 && lz >= -1 && lz <= 16 {
            let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
            (self.heightmap[idx] as i32, self.leaf_heightmap[idx] as i32)
        } else {
            (0, 0)
        }
    }

    #[inline(always)]
    pub fn sample_sky_light(&self, lx: i32, ly: i32, lz: i32) -> f32 {
        let (opaque_h, leaf_h) = self.get_heights(lx, lz);
        if ly >= opaque_h {
            if leaf_h > 0 && ly < leaf_h {
                0.85
            } else {
                1.0
            }
        } else {
            let depth = (opaque_h - ly).max(1);
            if depth >= 8 {
                0.0
            } else {
                let factor = 1.0 - (depth as f32 / 8.0);
                0.35 * factor * factor
            }
        }
    }

    #[inline]
    pub fn is_light_occluded(&self, from: IVec3, to_x: i32, to_y: i32, to_z: i32) -> bool {
        let dx = to_x - from.x;
        let dy = to_y - from.y;
        let dz = to_z - from.z;
        let steps = dx.abs().max(dy.abs()).max(dz.abs());
        if steps <= 1 {
            return false;
        }
        let inv_steps = 1.0 / steps as f32;
        let step_x = dx as f32 * inv_steps;
        let step_y = dy as f32 * inv_steps;
        let step_z = dz as f32 * inv_steps;

        let mut cur_x = from.x as f32 + 0.5;
        let mut cur_y = from.y as f32 + 0.5;
        let mut cur_z = from.z as f32 + 0.5;

        for _ in 1..steps {
            cur_x += step_x;
            cur_y += step_y;
            cur_z += step_z;
            let bx = cur_x.floor() as i32;
            let by = cur_y.floor() as i32;
            let bz = cur_z.floor() as i32;
            let lx = bx - self.base_x;
            let lz = bz - self.base_z;
            if self.solid_at(lx, by, lz) {
                return true;
            }
        }
        false
    }

    #[inline(always)]
    fn block_light_cache_index(&self, wx: f32, wy: f32, wz: f32) -> Option<usize> {
        let fx = wx.floor();
        let fy = wy.floor();
        let fz = wz.floor();
        if (wx - fx - 0.5).abs() > 0.0001
            || (wy - fy - 0.5).abs() > 0.0001
            || (wz - fz - 0.5).abs() > 0.0001
        {
            return None;
        }

        let lx = fx as i32 - self.base_x;
        let ly = fy as i32;
        let lz = fz as i32 - self.base_z;
        if lx < -1 || lx > CHUNK_SIZE_X || ly < -1 || ly > CHUNK_SIZE_Y
            || lz < -1 || lz > CHUNK_SIZE_Z
        {
            return None;
        }

        let _sx = CHUNK_SIZE_X as usize + 2;
        let sy = CHUNK_SIZE_Y as usize + 2;
        let sz = CHUNK_SIZE_Z as usize + 2;
        Some((((lx + 1) as usize) * sy + (ly + 1) as usize) * sz + (lz + 1) as usize)
    }

    #[inline(always)]
    fn sample_block_light_uncached(&self, wx: f32, wy: f32, wz: f32) -> f32 {
        if self.lights.is_empty() {
            return 0.0;
        }
        let to_x = wx.floor() as i32;
        let to_y = wy.floor() as i32;
        let to_z = wz.floor() as i32;
        let mut max_l = 0.0f32;
        for light in &self.lights {
            let dx = wx - (light.pos.x as f32 + 0.5);
            let dy = wy - (light.pos.y as f32 + 0.5);
            let dz = wz - (light.pos.z as f32 + 0.5);
            let d_sq = dx * dx + dy * dy + dz * dz;
            let r = light.radius;
            if d_sq < r * r {
                let dist = d_sq.sqrt();
                let t = (1.0 - dist / r).max(0.0);
                let falloff = t * t * (3.0 - 2.0 * t);
                let l = falloff * light.intensity;
                if l > max_l
                    && !self.is_light_occluded(light.pos, to_x, to_y, to_z)
                {
                    max_l = l;
                    if max_l >= 1.0 {
                        return max_l;
                    }
                }
            }
        }
        max_l
    }

    #[inline(always)]
    pub fn sample_block_light(&self, wx: f32, wy: f32, wz: f32) -> f32 {
        if let Some(index) = self.block_light_cache_index(wx, wy, wz) {
            let cached = self.block_light_cache.borrow()[index];
            if cached >= 0.0 {
                return cached;
            }
            let value = self.sample_block_light_uncached(wx, wy, wz);
            self.block_light_cache.borrow_mut()[index] = value;
            value
        } else {
            self.sample_block_light_uncached(wx, wy, wz)
        }
    }

    #[inline]
    pub(super) fn biome_at_cached(&self, wx: i32, wz: i32) -> Biome {
        if self.generator_type == WorldGeneratorType::Superflat {
            return Biome::Plains;
        }
        if let Some(biome) = self.biome_cache.borrow().get(&(wx, wz)).cloned() {
            return biome;
        }
        let biome = get_biome_at(wx, wz, self.seed).0;
        self.biome_cache.borrow_mut().insert((wx, wz), biome.clone());
        biome
    }
}
