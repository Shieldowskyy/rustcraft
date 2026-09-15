use std::collections::HashMap;

use crate::block::BlockId;
use crate::math::{IVec3, Vec3};
use crate::noise::fractal_noise2;

pub const CHUNK_SIZE_X: i32 = 16;
pub const CHUNK_SIZE_Y: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 16;

/// View distance in chunks (radius around the player).
pub const VIEW_DISTANCE: i32 = 4;

const WORLD_SEED: u32 = 1337;

pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<u8>, // flattened [x][y][z], BlockId as u8
    pub dirty: bool,     // needs remeshing
}

impl Chunk {
    fn index(x: i32, y: i32, z: i32) -> usize {
        ((x * CHUNK_SIZE_Y + y) * CHUNK_SIZE_Z + z) as usize
    }

    pub fn get_local(&self, x: i32, y: i32, z: i32) -> BlockId {
        if x < 0 || x >= CHUNK_SIZE_X || y < 0 || y >= CHUNK_SIZE_Y || z < 0 || z >= CHUNK_SIZE_Z {
            return BlockId::Air;
        }
        BlockId::from_u8(self.blocks[Chunk::index(x, y, z)])
    }

    pub fn set_local(&mut self, x: i32, y: i32, z: i32, b: BlockId) {
        if x < 0 || x >= CHUNK_SIZE_X || y < 0 || y >= CHUNK_SIZE_Y || z < 0 || z >= CHUNK_SIZE_Z {
            return;
        }
        self.blocks[Chunk::index(x, y, z)] = b.to_u8();
        self.dirty = true;
    }

    fn generate(cx: i32, cz: i32) -> Chunk {
        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = cx * CHUNK_SIZE_X + lx;
                let wz = cz * CHUNK_SIZE_Z + lz;

                let n = fractal_noise2(wx as f32 * 0.02, wz as f32 * 0.02, WORLD_SEED, 4, 0.5);
                let height = (28.0 + n * 14.0) as i32;
                let height = height.clamp(1, CHUNK_SIZE_Y - 10);

                for y in 0..CHUNK_SIZE_Y {
                    let block = if y > height {
                        if y <= 20 {
                            BlockId::Water
                        } else {
                            BlockId::Air
                        }
                    } else if y == height {
                        if height <= 21 {
                            BlockId::Sand
                        } else {
                            BlockId::Grass
                        }
                    } else if y > height - 4 {
                        if height <= 21 {
                            BlockId::Sand
                        } else {
                            BlockId::Dirt
                        }
                    } else {
                        BlockId::Stone
                    };
                    blocks[Chunk::index(lx, y, lz)] = block.to_u8();
                }

                // Sparse tree placement using deterministic hashing of world coords.
                if height > 21 {
                    let h = hash_coords(wx, wz);
                    if h % 137 == 0 {
                        place_tree(&mut blocks, lx, height + 1, lz);
                    }
                }
            }
        }

        Chunk {
            cx,
            cz,
            blocks,
            dirty: true,
        }
    }
}

fn hash_coords(x: i32, z: i32) -> u32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((z as u32).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn place_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 4;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    // Simple leaf canopy (clipped at chunk edges - acceptable simplification).
    for dx in -2..=2 {
        for dz in -2..=2 {
            for dy in 0..=2 {
                let x = lx + dx;
                let y = base_y + trunk_height - 1 + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z {
                    continue;
                }
                if y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                let dist = dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2;
                if dist <= 6 {
                    let idx = Chunk::index(x, y, z);
                    if blocks[idx] == BlockId::Air.to_u8() {
                        blocks[idx] = BlockId::Leaves.to_u8();
                    }
                }
            }
        }
    }
}

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
}

pub struct RaycastHit {
    pub block_pos: IVec3,
    pub normal: IVec3,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
        }
    }

    fn world_to_chunk(wx: i32, wz: i32) -> (i32, i32, i32, i32) {
        let cx = wx.div_euclid(CHUNK_SIZE_X);
        let cz = wz.div_euclid(CHUNK_SIZE_Z);
        let lx = wx.rem_euclid(CHUNK_SIZE_X);
        let lz = wz.rem_euclid(CHUNK_SIZE_Z);
        (cx, cz, lx, lz)
    }

    pub fn get_block(&self, pos: IVec3) -> BlockId {
        if pos.y < 0 || pos.y >= CHUNK_SIZE_Y {
            return BlockId::Air;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => chunk.get_local(lx, pos.y, lz),
            None => BlockId::Air,
        }
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockId) {
        if pos.y < 0 || pos.y >= CHUNK_SIZE_Y {
            return;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            chunk.set_local(lx, pos.y, lz, block);
            // Mark neighbor chunks dirty too if we edited a boundary block,
            // so seams re-mesh correctly.
            if lx == 0 {
                self.mark_dirty(cx - 1, cz);
            }
            if lx == CHUNK_SIZE_X - 1 {
                self.mark_dirty(cx + 1, cz);
            }
            if lz == 0 {
                self.mark_dirty(cx, cz - 1);
            }
            if lz == CHUNK_SIZE_Z - 1 {
                self.mark_dirty(cx, cz + 1);
            }
        }
    }

    fn mark_dirty(&mut self, cx: i32, cz: i32) {
        if let Some(c) = self.chunks.get_mut(&(cx, cz)) {
            c.dirty = true;
        }
    }

    /// Streams chunks in/out around the player position.
    pub fn update(&mut self, player_pos: Vec3) {
        let pcx = (player_pos.x as i32).div_euclid(CHUNK_SIZE_X);
        let pcz = (player_pos.z as i32).div_euclid(CHUNK_SIZE_Z);

        // Load missing chunks within view distance.
        for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
                let cx = pcx + dx;
                let cz = pcz + dz;
                if dx * dx + dz * dz > VIEW_DISTANCE * VIEW_DISTANCE {
                    continue;
                }
                self.chunks.entry((cx, cz)).or_insert_with(|| Chunk::generate(cx, cz));
            }
        }

        // Unload chunks that are far away, to bound memory use.
        let unload_dist = VIEW_DISTANCE + 2;
        self.chunks.retain(|&(cx, cz), _| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist * unload_dist
        });
    }

    /// Voxel DDA raycast; returns the first solid block hit and the face
    /// normal that was struck (used for block placement).
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RaycastHit> {
        let dir = dir.normalized();
        if dir.length_sq() < 1e-8 {
            return None;
        }

        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let next_boundary = |p: f32, step: i32| -> f32 {
            if step > 0 {
                p.floor() + 1.0 - p
            } else {
                p - p.floor()
            }
        };

        let mut t_max_x = if dir.x.abs() > 1e-8 {
            next_boundary(origin.x, step_x) / dir.x.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_y = if dir.y.abs() > 1e-8 {
            next_boundary(origin.y, step_y) / dir.y.abs()
        } else {
            f32::INFINITY
        };
        let mut t_max_z = if dir.z.abs() > 1e-8 {
            next_boundary(origin.z, step_z) / dir.z.abs()
        } else {
            f32::INFINITY
        };

        let mut last_normal = IVec3::new(0, 0, 0);
        let mut traveled = 0.0f32;

        while traveled <= max_dist {
            let block = self.get_block(IVec3::new(x, y, z));
            if block.is_solid() {
                return Some(RaycastHit {
                    block_pos: IVec3::new(x, y, z),
                    normal: last_normal,
                });
            }

            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x;
                traveled = t_max_x;
                t_max_x += t_delta_x;
                last_normal = IVec3::new(-step_x, 0, 0);
            } else if t_max_y < t_max_z {
                y += step_y;
                traveled = t_max_y;
                t_max_y += t_delta_y;
                last_normal = IVec3::new(0, -step_y, 0);
            } else {
                z += step_z;
                traveled = t_max_z;
                t_max_z += t_delta_z;
                last_normal = IVec3::new(0, 0, -step_z);
            }
        }

        None
    }

    /// Returns true if the given axis-aligned box (min/max corners in world
    /// space) overlaps any solid block. Used for simple player physics.
    pub fn box_collides(&self, min: Vec3, max: Vec3) -> bool {
        let x0 = min.x.floor() as i32;
        let x1 = max.x.floor() as i32;
        let y0 = min.y.floor() as i32;
        let y1 = max.y.floor() as i32;
        let z0 = min.z.floor() as i32;
        let z1 = max.z.floor() as i32;

        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    if self.get_block(IVec3::new(x, y, z)).is_solid() {
                        return true;
                    }
                }
            }
        }
        false
    }
}
