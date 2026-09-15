use std::collections::{HashMap, HashSet};

use crate::block::BlockId;
use crate::math::{IVec3, Vec3};
use crate::noise::fractal_noise2;

pub const CHUNK_SIZE_X: i32 = 16;
pub const CHUNK_SIZE_Y: i32 = 64;
pub const CHUNK_SIZE_Z: i32 = 16;

pub const VIEW_DISTANCE: i32 = 4;
pub const DEFAULT_WORLD_SEED: u32 = 1337;

pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<u8>,
    pub dirty: bool,
}

impl Chunk {
    #[inline(always)]
    pub fn index(x: i32, y: i32, z: i32) -> usize {
        ((x * CHUNK_SIZE_Y + y) * CHUNK_SIZE_Z + z) as usize
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
        if (x as u32) >= CHUNK_SIZE_X as u32 || (y as u32) >= CHUNK_SIZE_Y as u32 || (z as u32) >= CHUNK_SIZE_Z as u32 {
            return;
        }
        self.blocks[Chunk::index(x, y, z)] = b.to_u8();
        self.dirty = true;
    }

    fn generate(cx: i32, cz: i32, seed: u32) -> Chunk {
        use crate::noise::fractal_noise3;

        let mut blocks = vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize];

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = cx * CHUNK_SIZE_X + lx;
                let wz = cz * CHUNK_SIZE_Z + lz;

                let temp = fractal_noise2(wx as f32 * 0.006, wz as f32 * 0.006, seed.wrapping_add(500), 3, 0.5);
                let humidity = fractal_noise2(wx as f32 * 0.006, wz as f32 * 0.006, seed.wrapping_add(1500), 3, 0.5);
                let mountain_factor = fractal_noise2(wx as f32 * 0.008, wz as f32 * 0.008, seed.wrapping_add(2500), 3, 0.5);

                let is_mountain = mountain_factor > 0.32;
                let is_snowy = !is_mountain && temp < -0.18;
                let is_desert = !is_mountain && !is_snowy && temp > 0.28 && humidity < -0.05;
                let is_forest = !is_mountain && !is_snowy && !is_desert && humidity > 0.15;

                let n = fractal_noise2(wx as f32 * 0.015, wz as f32 * 0.015, seed, 4, 0.5);
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
                        blocks[Chunk::index(lx, y, lz)] = BlockId::Bedrock.to_u8();
                        continue;
                    }
                    if y == 1 {
                        let h = hash_coords_3d(wx, y, wz, seed);
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
                        let ore_h = hash_coords_3d(wx, y, wz, seed);
                        if ore_h % 37 == 0 {
                            BlockId::CoalOre
                        } else if ore_h % 67 == 0 && y < 30 {
                            BlockId::IronOre
                        } else {
                            BlockId::Stone
                        }
                    };

                    blocks[Chunk::index(lx, y, lz)] = block.to_u8();

                    if y > 1 && y < height - 1 && block != BlockId::Water && block != BlockId::Air {
                        let c1 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, seed.wrapping_add(3141), 2, 0.5);
                        let c2 = fractal_noise3(wx as f32 * 0.045, y as f32 * 0.065, wz as f32 * 0.045, seed.wrapping_add(9265), 2, 0.5);
                        let is_cave = (c1 * c1 + c2 * c2) < 0.024;
                        if is_cave {
                            if y <= 7 {
                                blocks[Chunk::index(lx, y, lz)] = BlockId::Lava.to_u8();
                            } else {
                                blocks[Chunk::index(lx, y, lz)] = BlockId::Air.to_u8();
                            }
                        }
                    }
                }

                if height > 21 {
                    if lx < 3 || lx >= CHUNK_SIZE_X - 3 || lz < 3 || lz >= CHUNK_SIZE_Z - 3 {
                        continue;
                    }
                    let h = hash_coords(wx, wz, seed);
                    if is_desert {
                        if h % 167 == 0 {
                            let c_height = 2 + (h % 2) as i32;
                            place_cactus(&mut blocks, lx, height + 1, lz, c_height);
                        }
                    } else if is_snowy {
                        if h % 113 == 0 {
                            place_pine_tree(&mut blocks, lx, height + 1, lz);
                        }
                    } else if is_forest {
                        if h % 47 == 0 {
                            place_tree(&mut blocks, lx, height + 1, lz);
                        }
                    } else if !is_mountain {
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

fn hash_coords(x: i32, z: i32, seed: u32) -> u32 {
    let mut h: u32 = (x as u32)
    .wrapping_mul(374761393)
    .wrapping_add((z as u32).wrapping_mul(668265263))
    .wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

fn hash_coords_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h: u32 = (x as u32)
    .wrapping_mul(374761393)
    .wrapping_add((y as u32).wrapping_mul(1103515245))
    .wrapping_add((z as u32).wrapping_mul(668265263))
    .wrapping_add(seed);
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
                    blocks[idx] = BlockId::SnowLeaves.to_u8();
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

/// Aktywny obiekt TNT odliczający czas do wybuchu
#[derive(Clone)]
pub struct PrimedTnt {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub fuse: f32, // Czas w sekundach do wybuchu (domyślnie ~3.5-4s)
}

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub seed: u32,
    pub player_position: Vec3,
    fluid_levels: HashMap<IVec3, u8>,
    pub active_sand: HashSet<IVec3>,
    pub sim_tick: u64,
    pub pending_explosions: Vec<Vec3>,
    pub primed_tnts: Vec<PrimedTnt>,
}

pub struct RaycastHit {
    pub block_pos: IVec3,
    pub normal: IVec3,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self::new_with_seed(DEFAULT_WORLD_SEED)
    }

    pub fn new_with_seed(seed: u32) -> Self {
        World {
            chunks: HashMap::new(),
            seed,
            player_position: Vec3::new(8.0, 90.0, 8.0),
            fluid_levels: HashMap::new(),
            active_sand: HashSet::new(),
            sim_tick: 0,
            pending_explosions: Vec::new(),
            primed_tnts: Vec::new(),
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let mut bytes = Vec::with_capacity(16 + self.chunks.len() * 4);
        bytes.extend_from_slice(b"RUSTCRAFT3");
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        bytes.extend_from_slice(&self.player_position.x.to_le_bytes());
        bytes.extend_from_slice(&self.player_position.y.to_le_bytes());
        bytes.extend_from_slice(&self.player_position.z.to_le_bytes());
        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());
        for chunk in self.chunks.values() {
            bytes.extend_from_slice(&chunk.cx.to_le_bytes());
            bytes.extend_from_slice(&chunk.cz.to_le_bytes());
            bytes.extend_from_slice(&(chunk.blocks.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&chunk.blocks);
        }
        bytes.extend_from_slice(&(self.fluid_levels.len() as u32).to_le_bytes());
        for (pos, level) in &self.fluid_levels {
            bytes.extend_from_slice(&pos.x.to_le_bytes());
            bytes.extend_from_slice(&pos.y.to_le_bytes());
            bytes.extend_from_slice(&pos.z.to_le_bytes());
            bytes.push(*level);
        }
        std::fs::write(path, bytes).map_err(|error| format!("nie udało się zapisać świata: {error}"))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|error| format!("nie udało się odczytać świata: {error}"))?;
        let mut cursor = 0usize;
        let format = read_bytes(&bytes, &mut cursor, 10)?;
        if format != b"RUSTCRAFT1" && format != b"RUSTCRAFT2" && format != b"RUSTCRAFT3" {
            return Err("nieznany format pliku świata".to_owned());
        }
        let seed = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
        let player_position = if format == b"RUSTCRAFT2" || format == b"RUSTCRAFT3" {
            Vec3::new(
                f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                      f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                      f32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
            )
        } else {
            Vec3::new(8.0, 90.0, 8.0)
        };
        let chunk_count = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
        let expected_len = (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize;
        let mut world = Self::new_with_seed(seed);
        world.player_position = player_position;
        for _ in 0..chunk_count {
            let cx = i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
            let cz = i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap());
            let block_len = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
            if block_len != expected_len {
                return Err("nieprawidłowy rozmiar chunka w pliku świata".to_owned());
            }
            let blocks = read_bytes(&bytes, &mut cursor, block_len)?.to_vec();
            world.chunks.insert((cx, cz), Chunk { cx, cz, blocks, dirty: true });
        }
        if format == b"RUSTCRAFT3" {
            let fluid_count = u32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
            for _ in 0..fluid_count {
                let pos = IVec3::new(
                    i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                                     i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                                     i32::from_le_bytes(read_bytes(&bytes, &mut cursor, 4)?.try_into().unwrap()),
                );
                let level = read_bytes(&bytes, &mut cursor, 1)?[0].clamp(1, 8);
                world.fluid_levels.insert(pos, level);
            }
        }
        Ok(world)
    }

    #[inline(always)]
    fn world_to_chunk(wx: i32, wz: i32) -> (i32, i32, i32, i32) {
        let cx = wx.div_euclid(CHUNK_SIZE_X);
        let cz = wz.div_euclid(CHUNK_SIZE_Z);
        let lx = wx.rem_euclid(CHUNK_SIZE_X);
        let lz = wz.rem_euclid(CHUNK_SIZE_Z);
        (cx, cz, lx, lz)
    }

    #[inline(always)]
    pub fn get_block(&self, pos: IVec3) -> BlockId {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => chunk.get_local(lx, pos.y, lz),
            None => BlockId::Air,
        }
    }

    #[inline(always)]
    pub fn fluid_level(&self, pos: IVec3) -> u8 {
        self.fluid_levels.get(&pos).copied().unwrap_or(8)
    }

    fn set_fluid_level(&mut self, pos: IVec3, level: u8) {
        self.fluid_levels.insert(pos, level.clamp(1, 8));
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockId) {
        if pos.y < 0 || pos.y >= CHUNK_SIZE_Y {
            return;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if self.chunks.contains_key(&(cx, cz)) {
            self.chunks.get_mut(&(cx, cz)).unwrap().set_local(lx, pos.y, lz, block);
            if matches!(block, BlockId::Water | BlockId::Lava) {
                self.set_fluid_level(pos, 8);
            } else {
                self.fluid_levels.remove(&pos);
            }

            if block == BlockId::Sand {
                self.active_sand.insert(pos);
            } else {
                self.active_sand.remove(&pos);
            }

            for offset in [
                IVec3::new(1, 0, 0),
                IVec3::new(-1, 0, 0),
                IVec3::new(0, 1, 0),
                IVec3::new(0, -1, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(0, 0, -1),
            ] {
                let neighbor = pos + offset;
                if self.get_block(neighbor) == BlockId::Sand {
                    self.active_sand.insert(neighbor);
                }
            }

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

    /// Uzbraja podany blok TNT (zamienia go na obiekt opadający z timerem)
    pub fn prime_tnt(&mut self, pos: IVec3) {
        if self.get_block(pos) == BlockId::Tnt {
            self.set_block(pos, BlockId::Air);
            self.primed_tnts.push(PrimedTnt {
                pos: Vec3::new(pos.x as f32 + 0.5, pos.y as f32, pos.z as f32 + 0.5),
                                  velocity: Vec3::new(0.0, 2.0, 0.0), // Lekki wyskok w górę jak w MC
                                  fuse: 3.5,                          // ~3.5 sekundy odliczania
            });
        }
    }

    /// Aktualizuje pozycje i timery uzbrojonych obiektów TNT
    pub fn update_primed_tnts(&mut self, dt: f32) {
        let mut exploded_positions = Vec::new();

        for i in 0..self.primed_tnts.len() {
            self.primed_tnts[i].fuse -= dt;

            // Grawitacja
            self.primed_tnts[i].velocity.y -= 20.0 * dt;

            let dy = self.primed_tnts[i].velocity.y * dt;
            let mut new_pos = self.primed_tnts[i].pos;
            new_pos.y += dy;

            // Sprawdzanie kolizji spodu kostki z blokami pod spodem
            let box_min = Vec3::new(new_pos.x - 0.49, new_pos.y, new_pos.z - 0.49);
            let box_max = Vec3::new(new_pos.x + 0.49, new_pos.y + 0.99, new_pos.z + 0.49);

            if self.box_collides(box_min, box_max) {
                // Lądowanie na podłożu
                self.primed_tnts[i].velocity = Vec3::zero();
                self.primed_tnts[i].pos.y = self.primed_tnts[i].pos.y.floor();
            } else {
                self.primed_tnts[i].pos = new_pos;
            }

            if self.primed_tnts[i].fuse <= 0.0 {
                exploded_positions.push(IVec3::new(
                    self.primed_tnts[i].pos.x.floor() as i32,
                                                   self.primed_tnts[i].pos.y.floor() as i32,
                                                   self.primed_tnts[i].pos.z.floor() as i32,
                ));
            }
        }

        self.primed_tnts.retain(|tnt| tnt.fuse > 0.0);

        for center in exploded_positions {
            self.explode(center, 3.5);
        }
    }

    /// Wyzwala eksplozję, niszczy teren i uzbraja sąsiadujące TNT (reakcja łańcuchowa z odliczaniem)
    pub fn explode(&mut self, center: IVec3, radius: f32) {
        let r_int = radius.ceil() as i32;
        let r_sq = radius * radius;
        let mut tnts_to_prime = Vec::new();

        for dx in -r_int..=r_int {
            for dy in -r_int..=r_int {
                for dz in -r_int..=r_int {
                    let dist_sq = (dx * dx + dy * dy + dz * dz) as f32;
                    if dist_sq <= r_sq {
                        let target_pos = center + IVec3::new(dx, dy, dz);
                        let b = self.get_block(target_pos);
                        if b != BlockId::Air && !b.is_bedrock() {
                            if b == BlockId::Tnt {
                                tnts_to_prime.push(target_pos);
                            } else {
                                self.set_block(target_pos, BlockId::Air);
                            }
                        }
                    }
                }
            }
        }

        self.pending_explosions.push(Vec3::new(center.x as f32, center.y as f32, center.z as f32));

        // Uzbrajamy odnalezione w zasięgu TNT
        for tnt_pos in tnts_to_prime {
            self.prime_tnt(tnt_pos);
        }
    }

    fn mark_dirty(&mut self, cx: i32, cz: i32) {
        if let Some(c) = self.chunks.get_mut(&(cx, cz)) {
            c.dirty = true;
        }
    }

    pub fn update(&mut self, player_pos: Vec3) {
        let pcx = (player_pos.x as i32).div_euclid(CHUNK_SIZE_X);
        let pcz = (player_pos.z as i32).div_euclid(CHUNK_SIZE_Z);

        for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
                let cx = pcx + dx;
                let cz = pcz + dz;
                if dx * dx + dz * dz > VIEW_DISTANCE * VIEW_DISTANCE {
                    continue;
                }
                let seed = self.seed;
                self.chunks.entry((cx, cz)).or_insert_with(|| Chunk::generate(cx, cz, seed));
            }
        }

        let unload_dist = VIEW_DISTANCE + 2;
        self.chunks.retain(|&(cx, cz), _| {
            let ddx = cx - pcx;
            let ddz = cz - pcz;
            ddx * ddx + ddz * ddz <= unload_dist * unload_dist
        });
    }

    pub fn simulate_materials(&mut self, player_pos: Vec3) {
        self.sim_tick = self.sim_tick.wrapping_add(1);
        let px = player_pos.x.floor() as i32;
        let py = player_pos.y.floor() as i32;
        let pz = player_pos.z.floor() as i32;
        let radius = 10i32;
        let min_y = (py - 12).max(1);
        let max_y = (py + 8).min(CHUNK_SIZE_Y - 1);

        // 1. Reakcja TNT z lawą (uzbrojenie od lawy)
        let mut tnts_to_prime = Vec::new();

        for y in min_y..=max_y {
            for z in (pz - radius)..=(pz + radius) {
                for x in (px - radius)..=(px + radius) {
                    let pos = IVec3::new(x, y, z);
                    if self.get_block(pos) == BlockId::Tnt {
                        let neighbors = [
                            IVec3::new(x + 1, y, z),
                            IVec3::new(x - 1, y, z),
                            IVec3::new(x, y + 1, z),
                            IVec3::new(x, y - 1, z),
                            IVec3::new(x, y, z + 1),
                            IVec3::new(x, y, z - 1),
                        ];
                        if neighbors.iter().any(|&p| self.get_block(p) == BlockId::Lava) {
                            tnts_to_prime.push(pos);
                        }
                    }
                }
            }
        }

        for tnt_pos in tnts_to_prime {
            self.prime_tnt(tnt_pos);
        }

        // 2. Fluid simulation
        let sim_lava = self.sim_tick % 3 == 0;
        let mut fluid_spawns: Vec<(IVec3, BlockId, u8)> = Vec::new();
        let mut fluid_despawns: Vec<IVec3> = Vec::new();
        let mut solidifications: Vec<(IVec3, BlockId)> = Vec::new();

        for y in (min_y..=max_y).rev() {
            for z in (pz - radius)..=(pz + radius) {
                for x in (px - radius)..=(px + radius) {
                    let pos = IVec3::new(x, y, z);
                    let block = self.get_block(pos);
                    if block != BlockId::Water && block != BlockId::Lava {
                        continue;
                    }
                    if block == BlockId::Lava && !sim_lava {
                        continue;
                    }

                    let level = self.fluid_level(pos);
                    let below = IVec3::new(x, y - 1, z);
                    let below_block = self.get_block(below);

                    let cardinals = [
                        IVec3::new(x + 1, y, z),
                        IVec3::new(x - 1, y, z),
                        IVec3::new(x, y, z + 1),
                        IVec3::new(x, y, z - 1),
                        IVec3::new(x, y + 1, z),
                    ];

                    if block == BlockId::Lava {
                        let touches_water = below_block == BlockId::Water
                        || cardinals.iter().any(|&p| self.get_block(p) == BlockId::Water);
                        if touches_water {
                            solidifications.push((pos, BlockId::Stone));
                            continue;
                        }
                    }

                    if below.y >= 0 && below_block == BlockId::Air {
                        fluid_spawns.push((below, block, 8));
                    } else if block == BlockId::Water && below_block == BlockId::Lava {
                        solidifications.push((below, BlockId::Stone));
                    } else {
                        let step = if block == BlockId::Lava { 4 } else { 2 };
                        if level > step {
                            let new_level = level - step;
                            for &npos in &cardinals[0..4] {
                                let nblock = self.get_block(npos);
                                if nblock == BlockId::Air {
                                    fluid_spawns.push((npos, block, new_level));
                                } else if block == BlockId::Water && nblock == BlockId::Lava {
                                    solidifications.push((npos, BlockId::Stone));
                                }
                            }
                        }
                    }

                    if level < 8 {
                        let above_block = self.get_block(IVec3::new(x, y + 1, z));
                        let has_parent_above = above_block == block;
                        let has_higher_neighbor = cardinals[0..4].iter().any(|&p| {
                            self.get_block(p) == block && self.fluid_level(p) > level
                        });
                        if !has_parent_above && !has_higher_neighbor {
                            fluid_despawns.push(pos);
                        }
                    }
                }
            }
        }

        for (pos, stone) in solidifications {
            self.set_block(pos, stone);
        }
        for (pos, block, level) in fluid_spawns {
            if self.get_block(pos) == BlockId::Air {
                self.set_block(pos, block);
                self.set_fluid_level(pos, level);
            }
        }
        for pos in fluid_despawns {
            if matches!(self.get_block(pos), BlockId::Water | BlockId::Lava) && self.fluid_level(pos) < 8 {
                self.set_block(pos, BlockId::Air);
            }
        }

        // 3. Sand gravity simulation
        if !self.active_sand.is_empty() {
            let mut sand_moves: Vec<(IVec3, IVec3)> = Vec::new();
            let mut sand_stopped: Vec<IVec3> = Vec::new();

            let mut active_list: Vec<IVec3> = self.active_sand.iter().copied().collect();
            active_list.sort_unstable_by_key(|p| p.y);

            for pos in active_list {
                if self.get_block(pos) != BlockId::Sand {
                    sand_stopped.push(pos);
                    continue;
                }
                let below = IVec3::new(pos.x, pos.y - 1, pos.z);
                if below.y >= 0 && self.get_block(below) == BlockId::Air {
                    sand_moves.push((pos, below));
                } else {
                    sand_stopped.push(pos);
                }
            }

            for pos in sand_stopped {
                self.active_sand.remove(&pos);
            }

            for (from, to) in sand_moves {
                if self.get_block(from) == BlockId::Sand && self.get_block(to) == BlockId::Air {
                    self.set_block(from, BlockId::Air);
                    self.set_block(to, BlockId::Sand);
                }
            }
        }
    }

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

fn read_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, count: usize) -> Result<&'a [u8], String> {
    if bytes.len().saturating_sub(*cursor) < count {
        return Err("uszkodzony plik świata".to_owned());
    }
    let slice = &bytes[*cursor..*cursor + count];
    *cursor += count;
    Ok(slice)
}
