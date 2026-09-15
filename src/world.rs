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
        use crate::noise::fractal_noise3;

        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = cx * CHUNK_SIZE_X + lx;
                let wz = cz * CHUNK_SIZE_Z + lz;

                // --- Biome noise sampling ---
                // Temperature: < -0.2 -> Snowy, > 0.3 -> Desert
                let temp = fractal_noise2(wx as f32 * 0.006, wz as f32 * 0.006, WORLD_SEED + 500, 3, 0.5);
                // Humidity: > 0.15 -> Forest
                let humidity = fractal_noise2(wx as f32 * 0.006, wz as f32 * 0.006, WORLD_SEED + 1500, 3, 0.5);
                // Mountain factor: > 0.35 -> Mountains
                let mountain_factor = fractal_noise2(wx as f32 * 0.008, wz as f32 * 0.008, WORLD_SEED + 2500, 3, 0.5);

                let is_mountain = mountain_factor > 0.32;
                let is_snowy = !is_mountain && temp < -0.18;
                let is_desert = !is_mountain && !is_snowy && temp > 0.28 && humidity < -0.05;
                let is_forest = !is_mountain && !is_snowy && !is_desert && humidity > 0.15;

                // Base height calculation per biome
                let n = fractal_noise2(wx as f32 * 0.015, wz as f32 * 0.015, WORLD_SEED, 4, 0.5);
                let height = if is_mountain {
                    let m_extra = ((mountain_factor - 0.32) / 0.68).clamp(0.0, 1.0);
                    let m_height = 36.0 + n * 12.0 + m_extra * 16.0;
                    (m_height as i32).clamp(1, CHUNK_SIZE_Y - 4)
                } else if is_desert {
                    let d_height = 26.0 + n * 10.0;
                    (d_height as i32).clamp(1, CHUNK_SIZE_Y - 10)
                } else {
                    let p_height = 28.0 + n * 12.0;
                    (p_height as i32).clamp(1, CHUNK_SIZE_Y - 10)
                };

                for y in 0..CHUNK_SIZE_Y {
                    if y == 0 {
                        // Bedrock - niezniszczalna warstwa dna
                        blocks[Chunk::index(lx, y, lz)] = BlockId::Bedrock.to_u8();
                        continue;
                    }
                    if y == 1 {
                        // Warstwa mieszana bedrock/stone na Y=1
                        let h = hash_coords_3d(wx, y, wz);
                        blocks[Chunk::index(lx, y, lz)] = if h % 3 == 0 {
                            BlockId::Bedrock.to_u8()
                        } else {
                            BlockId::Stone.to_u8()
                        };
                        continue;
                    }

                    let block = if y > height {
                        if y <= 20 {
                            if is_snowy && y == 20 {
                                // Ice on top of frozen lakes in snow biome
                                BlockId::Snow
                            } else {
                                BlockId::Water
                            }
                        } else {
                            BlockId::Air
                        }
                    } else if y == height {
                        if height <= 21 {
                            BlockId::Sand
                        } else if is_snowy {
                            BlockId::Snow
                        } else if is_desert {
                            BlockId::Sand
                        } else if is_mountain {
                            if height >= 48 {
                                BlockId::Snow
                            } else if height >= 38 {
                                BlockId::Stone
                            } else {
                                BlockId::Grass
                            }
                        } else {
                            BlockId::Grass
                        }
                    } else if y > height - 4 {
                        if height <= 21 || is_desert {
                            BlockId::Sand
                        } else if is_mountain && height >= 38 {
                            BlockId::Stone
                        } else {
                            BlockId::Dirt
                        }
                    } else {
                        // Underground stone with ores
                        let ore_h = hash_coords_3d(wx, y, wz);
                        if ore_h % 37 == 0 {
                            BlockId::CoalOre
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else {
                            BlockId::Stone
                        }
                    };

                    blocks[Chunk::index(lx, y, lz)] = block.to_u8();

                    // --- 3D Cave generation ---
                    // Carve hollow worm tunnels & caverns through solid terrain
                    if y > 1 && y < height - 1 && block != BlockId::Water && block != BlockId::Air {
                        let c1 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, WORLD_SEED + 3141, 2, 0.5);
                        let c2 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, WORLD_SEED + 9265, 2, 0.5);
                        let is_cave = (c1 * c1 + c2 * c2) < 0.024;
                        if is_cave {
                            // Bardzo głęboko: wypełnij lawą zamiast powietrzem
                            if y <= 7 {
                                blocks[Chunk::index(lx, y, lz)] = BlockId::Lava.to_u8();
                            } else {
                                blocks[Chunk::index(lx, y, lz)] = BlockId::Air.to_u8();
                            }
                        }
                    }
                }

                // --- Vegetation & features (Trees, Cacti, Pine Trees) ---
                if height > 21 {
                    let h = hash_coords(wx, wz);
                    if is_desert {
                        // Cacti in desert
                        if h % 167 == 0 {
                            let c_height = 2 + (h % 2) as i32;
                            place_cactus(&mut blocks, lx, height + 1, lz, c_height);
                        }
                    } else if is_snowy {
                        // Pine / Spruce trees in Snowy Taiga
                        if h % 113 == 0 {
                            place_pine_tree(&mut blocks, lx, height + 1, lz);
                        }
                    } else if is_forest {
                        // Dense oak trees in Forest
                        if h % 47 == 0 {
                            place_tree(&mut blocks, lx, height + 1, lz);
                        }
                    } else if !is_mountain {
                        // Occasional oak trees in Plains
                        if h % 137 == 0 {
                            place_tree(&mut blocks, lx, height + 1, lz);
                        }
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

fn hash_coords_3d(x: i32, y: i32, z: i32) -> u32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(1103515245))
        .wrapping_add((z as u32).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn place_cactus(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32, height: i32) {
    for i in 0..height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Cactus.to_u8();
        }
    }
}

fn place_pine_tree(blocks: &mut [u8], lx: i32, base_y: i32, lz: i32) {
    let trunk_height = 6;
    for i in 0..trunk_height {
        let y = base_y + i;
        if y >= 0 && y < CHUNK_SIZE_Y {
            blocks[Chunk::index(lx, y, lz)] = BlockId::Wood.to_u8();
        }
    }
    // Conical pine canopy with snow on top
    for dy in 2..=trunk_height + 1 {
        let radius: i32 = if dy == trunk_height + 1 {
            0
        } else if dy >= trunk_height - 1 {
            1
        } else if dy % 2 == 0 {
            2
        } else {
            1
        };

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let x = lx + dx;
                let y = base_y + dy;
                let z = lz + dz;
                if x < 0 || x >= CHUNK_SIZE_X || z < 0 || z >= CHUNK_SIZE_Z || y < 0 || y >= CHUNK_SIZE_Y {
                    continue;
                }
                if dx == 0 && dz == 0 && dy < trunk_height {
                    continue;
                }
                let idx = Chunk::index(x, y, z);
                if blocks[idx] == BlockId::Air.to_u8() {
                    let block = if dy == trunk_height + 1 {
                        BlockId::SnowLeaves // wierzchołek: zimowe liście z lodem
                    } else if dy >= trunk_height && (dx != 0 || dz != 0) {
                        BlockId::SnowLeaves // zewnętrzne: zimowe liście
                    } else {
                        BlockId::SnowLeaves // wszystkie: zimowa zieleń
                    };
                    blocks[idx] = block.to_u8();
                }
            }
        }
    }
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
