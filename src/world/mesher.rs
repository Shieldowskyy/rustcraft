// Converts a chunk's voxel grid into a list of world-space, textured
// triangles. Uses simple face-culling (skip faces touching a solid, opaque
// neighbor) rather than full greedy meshing, to keep the code easy to
// follow. Vita builds use worker-thread meshing and memoized AO light samples.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::math::{IVec3, Vec3};
use crate::graphics::atlas::TextureId;
use crate::world::block::{BlockId, Face};
use crate::world::{get_biome_at, Biome, Chunk, World, WorldGeneratorType, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

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
    pub center_blocks: Vec<u8>,
    pub east_blocks: Option<Vec<u8>>,
    pub west_blocks: Option<Vec<u8>>,
    pub south_blocks: Option<Vec<u8>>,
    pub north_blocks: Option<Vec<u8>>,
    pub se_blocks: Option<Vec<u8>>,
    pub sw_blocks: Option<Vec<u8>>,
    pub ne_blocks: Option<Vec<u8>>,
    pub nw_blocks: Option<Vec<u8>>,
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
            world.chunks.get(&(x, z)).map(|neighbor| neighbor.blocks.clone())
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
            center_blocks: chunk.blocks.clone(),
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

pub struct AsyncMesher {
    #[allow(dead_code)]
    req_tx: std::sync::mpsc::Sender<ChunkMeshJob>,
    res_rx: std::sync::mpsc::Receiver<((i32, i32), Vec<Triangle>)>,
    #[allow(dead_code)]
    res_tx: std::sync::mpsc::Sender<((i32, i32), Vec<Triangle>)>,
    worker_count: usize,
}

impl Default for AsyncMesher {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncMesher {
    pub fn new() -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel::<ChunkMeshJob>();
        let (res_tx, res_rx) = std::sync::mpsc::channel::<((i32, i32), Vec<Triangle>)>();

        let req_rx = std::sync::Arc::new(std::sync::Mutex::new(req_rx));
        let num_threads = {
            #[cfg(target_os = "vita")]
            { 2 }
            #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
            { 0 }
            #[cfg(not(any(target_os = "vita", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
            {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(2)
                    .clamp(1, 4)
            }
        };

        let mut worker_count = 0usize;
        for i in 0..num_threads {
            let req_rx_clone = std::sync::Arc::clone(&req_rx);
            let res_tx_clone = res_tx.clone();
            let spawn_result = std::thread::Builder::new()
                .name(format!("chunk_mesh_{}", i))
                .spawn(move || {
                    #[cfg(target_os = "vita")]
                    unsafe {
                        extern "C" {
                            fn sceKernelGetThreadId() -> i32;
                            fn sceKernelChangeThreadCpuAffinityMask(thid: i32, bitmask: i32) -> i32;
                        }
                        let thid = sceKernelGetThreadId();
                        let mask = if i % 2 == 0 { 0x00020000 } else { 0x00040000 };
                        let _ = sceKernelChangeThreadCpuAffinityMask(thid, mask);
                    }

                    loop {
                        let job = {
                            if let Ok(rx) = req_rx_clone.lock() {
                                rx.recv().ok()
                            } else {
                                None
                            }
                        };
                        match job {
                            Some(job) => {
                                let key = job.key;
                                let mesh = build_chunk_mesh_from_job_limited(&job, 4096);
                                if res_tx_clone.send((key, mesh)).is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                });
            if spawn_result.is_ok() {
                worker_count += 1;
            }
        }

        Self { req_tx, res_rx, res_tx, worker_count }
    }

    pub fn submit_job(&self, job: ChunkMeshJob) -> bool {
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        {
            let key = job.key;
            let mesh = build_chunk_mesh_from_job_limited(&job, 4096);
            self.res_tx.send((key, mesh)).is_ok()
        }
        #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
        {
            if self.worker_count == 0 {
                let key = job.key;
                let mesh = build_chunk_mesh_from_job_limited(&job, 4096);
                self.res_tx.send((key, mesh)).is_ok()
            } else {
                self.req_tx.send(job).is_ok()
            }
        }
    }

    pub fn drain_completed(&self) -> Vec<((i32, i32), Vec<Triangle>)> {
        let mut completed = Vec::new();
        while let Ok(res) = self.res_rx.try_recv() {
            completed.push(res);
        }
        completed
    }
}

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
    fn sample_voxel_lights(&self, wx: i32, wy: i32, wz: i32, y_offset: f32) -> (f32, f32) {
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

        let sx = CHUNK_SIZE_X as usize + 2;
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
    fn biome_at_cached(&self, wx: i32, wz: i32) -> Biome {
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

pub const SUBCHUNK_SIZE: i32 = 16;
pub const NUM_SUBCHUNKS: usize = (CHUNK_SIZE_Y / SUBCHUNK_SIZE) as usize; // 4

pub fn build_subchunk_mesh_from_cache(
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    let y_start = (sub_y as i32) * SUBCHUNK_SIZE;
    let y_end = y_start + SUBCHUNK_SIZE;

    // --- OPTYMALIZACJA: Fast-path dla pustych subchunków ---
    // Sprawdzamy, czy maksymalna wysokość terenu w tym subchunku jest niższa niż y_start.
    // Jeśli tak, to w tym przedziale Y nie ma żadnych pełnych bloków ziemi/kamienia.
    let mut max_h_in_subchunk = 0u8;
    for lx in 0..16 {
        for lz in 0..16 {
            let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
            let h = cache.heightmap[idx];
            if h > max_h_in_subchunk {
                max_h_in_subchunk = h;
            }
        }
    }
    // Jeśli cały teren jest poniżej tego subchunka oraz brak świateł/płynów mogących generować geometrię,
    // możemy pominąć całą iterację po blokach (zabezpieczenie: dla sub_y > 0 sprawdzamy czy y_start >= max_h_in_subchunk).
    if (y_start as u8) >= max_h_in_subchunk && cache.lights.is_empty() {
        // Dodatkowo upewniamy się, że nie ma tam płynów (np. oceanu powyżej dna)
        // Jeśli fluid_map jest pusta, możemy bezpiecznie zwrócić puste wektory.
        if cache.fluid_map.map_or(true, |fm| fm.is_empty()) {
            return (Vec::new(), Vec::new());
        }
    }
    // ------------------------------------------------------

    let mut solid_tris = Vec::with_capacity(384);
    let mut trans_tris = Vec::with_capacity(64);

    let base_x = cache.base_x;
    let base_z = cache.base_z;
    let blocks = cache.center_blocks;

    for lx in 0..CHUNK_SIZE_X {
        for ly in y_start..y_end {
            for lz in 0..CHUNK_SIZE_Z {
                let block_u8 = blocks[((lx << 10) | (ly << 4) | lz) as usize];
                if block_u8 == 0 {
                    continue;
                }
                let block = BlockId::from_u8(block_u8);
                let wx = base_x + lx;
                let wy = ly;
                let wz = base_z + lz;

                if block == BlockId::Torch {
                    emit_torch(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::Door {
                    emit_door(&mut solid_tris, wx, wy, wz, false, cache);
                    continue;
                }
                if block == BlockId::DoorOpen {
                    emit_door(&mut solid_tris, wx, wy, wz, true, cache);
                    continue;
                }
                if block == BlockId::Bed {
                    emit_bed(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::WoodenSlab || block == BlockId::StoneSlab {
                    emit_slab(&mut solid_tris, wx, wy, wz, block, cache);
                    continue;
                }
                if block == BlockId::WoodenStairs || block == BlockId::StoneStairs {
                    emit_stairs(&mut solid_tris, wx, wy, wz, block, cache);
                    continue;
                }
                if block == BlockId::Ladder {
                    emit_ladder(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::RustStone || block == BlockId::RustStoneWireLit {
                    emit_ruststone_wire(&mut solid_tris, wx, wy, wz, block == BlockId::RustStoneWireLit, cache);
                    continue;
                }
                if block == BlockId::Lever || block == BlockId::LeverOn {
                    emit_lever(&mut solid_tris, wx, wy, wz, block == BlockId::LeverOn, cache);
                    continue;
                }
                if block == BlockId::Button || block == BlockId::ButtonPressed {
                    emit_button(&mut solid_tris, wx, wy, wz, block == BlockId::ButtonPressed, cache);
                    continue;
                }
                if block == BlockId::PressurePlate || block == BlockId::PressurePlatePressed {
                    emit_pressure_plate(&mut solid_tris, wx, wy, wz, block == BlockId::PressurePlatePressed, cache);
                    continue;
                }
                if matches!(block, BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop | BlockId::Sapling | BlockId::Kelp | BlockId::Seagrass) {
                    emit_cross(&mut solid_tris, wx, wy, wz, block.texture_id(Face::Top), cache);
                    if matches!(block, BlockId::Kelp | BlockId::Seagrass) {
                        emit_waterlogged_faces(&mut trans_tris, wx, wy, wz, lx, ly, lz, cache);
                    }
                    continue;
                }

                let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                let is_trans = matches!(block, BlockId::Water | BlockId::Glass);

                for face in Face::ALL {
                    let n = face.normal();
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    let visible = if is_fluid {
                        let neighbor_loaded = match face {
                            Face::East => lx < CHUNK_SIZE_X - 1 || cache.east_blocks.is_some(),
                            Face::West => lx > 0 || cache.west_blocks.is_some(),
                            Face::South => lz < CHUNK_SIZE_Z - 1 || cache.south_blocks.is_some(),
                            Face::North => lz > 0 || cache.north_blocks.is_some(),
                            Face::Top | Face::Bottom => true,
                        };

                        if !neighbor_loaded {
                            false
                        } else {
                            let same_fluid = if block == BlockId::Water {
                                matches!(neighbor, BlockId::Water | BlockId::Kelp | BlockId::Seagrass)
                            } else {
                                neighbor == block
                            };

                            match face {
                                Face::Top => !same_fluid,
                                Face::Bottom => wy > 0 && !same_fluid && (neighbor == BlockId::Air || (neighbor.is_transparent() && !same_fluid)),
                                _ => {
                                    if !same_fluid {
                                        neighbor == BlockId::Air || (neighbor.is_transparent() && !same_fluid)
                                    } else {
                                        let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                                        let above_same_fluid = if block == BlockId::Water {
                                            matches!(above_neighbor, BlockId::Water | BlockId::Kelp | BlockId::Seagrass)
                                        } else {
                                            above_neighbor == block
                                        };
                                        if !above_same_fluid {
                                            let my_lvl = cache.fluid_level(IVec3::new(wx, wy, wz));
                                            let neighbor_lvl = cache.fluid_level(IVec3::new(wx + n.x, wy, wz + n.z));
                                            my_lvl > neighbor_lvl
                                        } else {
                                            false
                                        }
                                    }
                                }
                            }
                        }
                    } else if face == Face::Bottom && wy == 0 {
                        false
                    } else {
                        neighbor == BlockId::Air
                        || (neighbor.is_transparent() && neighbor != block)
                    };

                    if visible {
                        if is_trans {
                            emit_face(&mut trans_tris, wx, wy, wz, face, block, cache);
                        } else {
                            emit_face(&mut solid_tris, wx, wy, wz, face, block, cache);
                        }
                    }
                }
            }
        }
    }

    (solid_tris, trans_tris)
}

pub fn build_subchunk_mesh_internal(
    _chunk: &Chunk,
    _world: &World,
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    build_subchunk_mesh_from_cache(sub_y, cache)
}

pub fn build_subchunk_mesh(chunk: &Chunk, world: &World, sub_y: usize) -> Vec<Triangle> {
    if sub_y >= NUM_SUBCHUNKS {
        return Vec::new();
    }
    let cache = ChunkNeighborCache::new(chunk, world);
    let (mut solid, trans) = build_subchunk_mesh_from_cache(sub_y, &cache);
    solid.extend(trans);
    solid
}

fn build_chunk_mesh_from_cache(cache: &ChunkNeighborCache) -> Vec<Triangle> {
    let mut solid_tris = Vec::with_capacity(3072);
    let mut trans_tris = Vec::with_capacity(512);

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_from_cache(sub_y, cache);
        solid_tris.extend(sub_solid);
        trans_tris.extend(sub_trans);
    }

    solid_tris.extend(trans_tris);
    solid_tris
}

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let cache = ChunkNeighborCache::new(chunk, world);
    build_chunk_mesh_from_cache(&cache)
}

pub fn build_chunk_mesh_from_job(job: &ChunkMeshJob) -> Vec<Triangle> {
    let cache = ChunkNeighborCache::from_job(job);
    build_chunk_mesh_from_cache(&cache)
}

pub fn build_chunk_mesh_from_job_limited(job: &ChunkMeshJob, max_triangles: usize) -> Vec<Triangle> {
    if max_triangles == 0 {
        return Vec::new();
    }
    let cache = ChunkNeighborCache::from_job(job);
    let mut solid_tris = Vec::with_capacity(3072.min(max_triangles));
    let mut trans_tris = Vec::with_capacity(512.min(max_triangles));

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_from_cache(sub_y, &cache);
        if solid_tris.len() < max_triangles {
            let remaining = max_triangles - solid_tris.len();
            solid_tris.extend(sub_solid.into_iter().take(remaining));
        }
        if solid_tris.len() >= max_triangles {
            return solid_tris;
        }
        if trans_tris.len() < max_triangles {
            let remaining = max_triangles - trans_tris.len();
            trans_tris.extend(sub_trans.into_iter().take(remaining));
        }
    }

    trans_tris.truncate(max_triangles.saturating_sub(solid_tris.len()));
    solid_tris.extend(trans_tris);
    solid_tris
}

#[inline(always)]
fn push_quad_uv(
    tris: &mut Vec<Triangle>,
    a: (Vec3, (f32, f32)),
    b: (Vec3, (f32, f32)),
    c: (Vec3, (f32, f32)),
    d: (Vec3, (f32, f32)),
    texture: TextureId,
    ao: f32,
    sky_light: f32,
    block_light: f32,
) {
    let va = Vertex::lit(a.0, a.1, ao, sky_light, block_light);
    let vb = Vertex::lit(b.0, b.1, ao, sky_light, block_light);
    let vc = Vertex::lit(c.0, c.1, ao, sky_light, block_light);
    let vd = Vertex::lit(d.0, d.1, ao, sky_light, block_light);
    tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
    tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
}

#[inline(always)]
fn push_quad(
    tris: &mut Vec<Triangle>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    texture: TextureId,
    ao: f32,
    sky_light: f32,
    block_light: f32,
) {
    push_quad_uv(
        tris,
        (a, (0.0, 1.0)),
        (b, (1.0, 1.0)),
        (c, (1.0, 0.0)),
        (d, (0.0, 0.0)),
        texture,
        ao,
        sky_light,
        block_light,
    );
}

fn emit_torch(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    _cache: &ChunkNeighborCache,
) {
    let quad_uv = |tris: &mut Vec<Triangle>,
                   a: (Vec3, (f32, f32)),
                   b: (Vec3, (f32, f32)),
                   c: (Vec3, (f32, f32)),
                   d: (Vec3, (f32, f32)),
                   shade: f32,
                   light: f32| {
        push_quad_uv(tris, a, b, c, d, TextureId::Torch, shade, light, light);
    };

    let x0 = wx as f32 + 0.4375;
    let x1 = wx as f32 + 0.5625;
    let y0 = wy as f32;
    let y1 = wy as f32 + 0.625;
    let z0 = wz as f32 + 0.4375;
    let z1 = wz as f32 + 0.5625;

    let c000 = Vec3::new(x0, y0, z0);
    let c100 = Vec3::new(x1, y0, z0);
    let c010 = Vec3::new(x0, y1, z0);
    let c110 = Vec3::new(x1, y1, z0);
    let c001 = Vec3::new(x0, y0, z1);
    let c101 = Vec3::new(x1, y0, z1);
    let c011 = Vec3::new(x0, y1, z1);
    let c111 = Vec3::new(x1, y1, z1);

    let su0 = 0.4375f32;
    let su1 = 0.5625f32;
    let sv0 = 0.3750f32;
    let sv1 = 1.0000f32;

    quad_uv(tris, (c100, (su0, sv1)), (c000, (su1, sv1)), (c010, (su1, sv0)), (c110, (su0, sv0)), 0.95, 1.0);
    quad_uv(tris, (c001, (su0, sv1)), (c101, (su1, sv1)), (c111, (su1, sv0)), (c011, (su0, sv0)), 0.95, 1.0);
    quad_uv(tris, (c101, (su0, sv1)), (c100, (su1, sv1)), (c110, (su1, sv0)), (c111, (su0, sv0)), 0.90, 1.0);
    quad_uv(tris, (c000, (su0, sv1)), (c001, (su1, sv1)), (c011, (su1, sv0)), (c010, (su0, sv0)), 0.90, 1.0);
    if wy > 0 {
        quad_uv(tris, (c001, (su0, 0.875)), (c000, (su0, 1.0)), (c100, (su1, 1.0)), (c101, (su1, 0.875)), 0.70, 1.0);
    }

    let fx0 = wx as f32 + 0.40625;
    let fx1 = wx as f32 + 0.59375;
    let fy0 = wy as f32 + 0.50000;
    let fy1 = wy as f32 + 0.71875;
    let fz0 = wz as f32 + 0.40625;
    let fz1 = wz as f32 + 0.59375;

    let fc000 = Vec3::new(fx0, fy0, fz0);
    let fc100 = Vec3::new(fx1, fy0, fz0);
    let fc010 = Vec3::new(fx0, fy1, fz0);
    let fc110 = Vec3::new(fx1, fy1, fz0);
    let fc001 = Vec3::new(fx0, fy0, fz1);
    let fc101 = Vec3::new(fx1, fy0, fz1);
    let fc011 = Vec3::new(fx0, fy1, fz1);
    let fc111 = Vec3::new(fx1, fy1, fz1);

    let fu0 = 0.3750f32;
    let fu1 = 0.6250f32;
    let fv0 = 0.0000f32;
    let fv1 = 0.3125f32;

    quad_uv(tris, (fc010, (fu0, 0.06)), (fc011, (fu0, 0.25)), (fc111, (fu1, 0.25)), (fc110, (fu1, 0.06)), 1.0, 1.0);
    quad_uv(tris, (fc100, (fu0, fv1)), (fc000, (fu1, fv1)), (fc010, (fu1, fv0)), (fc110, (fu0, fv0)), 1.0, 1.0);
    quad_uv(tris, (fc001, (fu0, fv1)), (fc101, (fu1, fv1)), (fc111, (fu1, fv0)), (fc011, (fu0, fv0)), 1.0, 1.0);
    quad_uv(tris, (fc101, (fu0, fv1)), (fc100, (fu1, fv1)), (fc110, (fu1, fv0)), (fc111, (fu0, fv0)), 1.0, 1.0);
    quad_uv(tris, (fc000, (fu0, fv1)), (fc001, (fu1, fv1)), (fc011, (fu1, fv0)), (fc010, (fu0, fv0)), 1.0, 1.0);
}

fn emit_door(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    is_open: bool,
    cache: &ChunkNeighborCache,
) {
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;
    let below = cache.sample_block(lx, wy - 1, lz);
    let is_top = below == BlockId::Door || below == BlockId::DoorOpen;
    let tex = if is_top { TextureId::DoorTop } else { TextureId::Door };
    let side_tex = TextureId::Planks;

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    let thick = 0.1875f32;

    let (x0, x1, z0, z1) = if !is_open {
        (x, x + 1.0, z, z + thick)
    } else {
        (x, x + thick, z, z + 1.0)
    };
    let y0 = y;
    let y1 = y + 1.0;

    let c000 = Vec3::new(x0, y0, z0);
    let c100 = Vec3::new(x1, y0, z0);
    let c010 = Vec3::new(x0, y1, z0);
    let c110 = Vec3::new(x1, y1, z0);
    let c001 = Vec3::new(x0, y0, z1);
    let c101 = Vec3::new(x1, y0, z1);
    let c011 = Vec3::new(x0, y1, z1);
    let c111 = Vec3::new(x1, y1, z1);

    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.5);

    let quad_uv = |tris: &mut Vec<Triangle>,
                   a: (Vec3, (f32, f32)),
                   b: (Vec3, (f32, f32)),
                   c: (Vec3, (f32, f32)),
                   d: (Vec3, (f32, f32)),
                   texture: TextureId,
                   shade: f32| {
        push_quad_uv(tris, a, b, c, d, texture, shade, sky, block_l);
    };

    if !is_open {
        quad_uv(tris, (c100, (1.0, 1.0)), (c000, (0.0, 1.0)), (c010, (0.0, 0.0)), (c110, (1.0, 0.0)), tex, 0.85);
        quad_uv(tris, (c001, (0.0, 1.0)), (c101, (1.0, 1.0)), (c111, (1.0, 0.0)), (c011, (0.0, 0.0)), tex, 0.85);
        quad_uv(tris, (c101, (0.0, 1.0)), (c100, (1.0, 1.0)), (c110, (1.0, 0.0)), (c111, (0.0, 0.0)), side_tex, 0.70);
        quad_uv(tris, (c000, (0.0, 1.0)), (c001, (1.0, 1.0)), (c011, (1.0, 0.0)), (c010, (0.0, 0.0)), side_tex, 0.70);
    } else {
        quad_uv(tris, (c101, (1.0, 1.0)), (c100, (0.0, 1.0)), (c110, (0.0, 0.0)), (c111, (1.0, 0.0)), tex, 0.85);
        quad_uv(tris, (c000, (0.0, 1.0)), (c001, (1.0, 1.0)), (c011, (1.0, 0.0)), (c010, (0.0, 0.0)), tex, 0.85);
        quad_uv(tris, (c100, (0.0, 1.0)), (c000, (1.0, 1.0)), (c010, (1.0, 0.0)), (c110, (0.0, 0.0)), side_tex, 0.70);
        quad_uv(tris, (c001, (0.0, 1.0)), (c101, (1.0, 1.0)), (c111, (1.0, 0.0)), (c011, (0.0, 0.0)), side_tex, 0.70);
    }
    quad_uv(tris, (c010, (0.0, 1.0)), (c011, (1.0, 1.0)), (c111, (1.0, 0.0)), (c110, (0.0, 0.0)), side_tex, 1.0);
    if wy > 0 {
        quad_uv(tris, (c001, (0.0, 1.0)), (c000, (1.0, 1.0)), (c100, (1.0, 0.0)), (c101, (0.0, 0.0)), side_tex, 0.5);
    }
}

fn emit_box(
    tris: &mut Vec<Triangle>,
    x0: f32, y0: f32, z0: f32,
    x1: f32, y1: f32, z1: f32,
    top_tex: TextureId,
    side_tex: TextureId,
    bottom_tex: TextureId,
    sky: f32,
    block_l: f32,
) {
    let c000 = Vec3::new(x0, y0, z0);
    let c100 = Vec3::new(x1, y0, z0);
    let c010 = Vec3::new(x0, y1, z0);
    let c110 = Vec3::new(x1, y1, z0);
    let c001 = Vec3::new(x0, y0, z1);
    let c101 = Vec3::new(x1, y0, z1);
    let c011 = Vec3::new(x0, y1, z1);
    let c111 = Vec3::new(x1, y1, z1);

    push_quad(tris, c010, c011, c111, c110, top_tex, 1.0, sky, block_l);
    push_quad(tris, c100, c000, c010, c110, side_tex, 0.85, sky, block_l);
    push_quad(tris, c001, c101, c111, c011, side_tex, 0.85, sky, block_l);
    push_quad(tris, c101, c100, c110, c111, side_tex, 0.70, sky, block_l);
    push_quad(tris, c000, c001, c011, c010, side_tex, 0.70, sky, block_l);
    push_quad(tris, c001, c000, c100, c101, bottom_tex, 0.50, sky, block_l);
}

fn emit_slab(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    block: BlockId,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.25);
    let tex = if block == BlockId::StoneSlab { TextureId::Stone } else { TextureId::Planks };
    emit_box(
        tris,
        wx as f32, wy as f32, wz as f32,
        wx as f32 + 1.0, wy as f32 + 0.5, wz as f32 + 1.0,
        tex, tex, tex, sky, block_l,
    );
}

fn emit_bed(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.25);
    emit_box(
        tris,
        wx as f32, wy as f32, wz as f32,
        wx as f32 + 1.0, wy as f32 + 0.5625, wz as f32 + 1.0,
        TextureId::BedTop, TextureId::BedSide, TextureId::Planks, sky, block_l,
    );
}

fn emit_stairs(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    block: BlockId,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.5);
    let tex = if block == BlockId::StoneStairs { TextureId::Stone } else { TextureId::Planks };
    emit_box(
        tris,
        wx as f32, wy as f32, wz as f32,
        wx as f32 + 1.0, wy as f32 + 0.5, wz as f32 + 1.0,
        tex, tex, tex, sky, block_l,
    );
    emit_box(
        tris,
        wx as f32, wy as f32 + 0.5, wz as f32,
        wx as f32 + 1.0, wy as f32 + 1.0, wz as f32 + 0.5,
        tex, tex, tex, sky, block_l,
    );
}

fn emit_ladder(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    cache: &ChunkNeighborCache,
) {
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.5);
    let tex = TextureId::Ladder;

    let north_solid = cache.sample_block(lx, wy, lz - 1).is_solid();
    let south_solid = cache.sample_block(lx, wy, lz + 1).is_solid();
    let west_solid = cache.sample_block(lx - 1, wy, lz).is_solid();
    let east_solid = cache.sample_block(lx + 1, wy, lz).is_solid();

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    let mut quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, shade: f32| {
        push_quad(tris, a, b, c, d, tex, shade, sky, block_l);
        let va = Vertex::lit(a, (0.0, 1.0), shade, sky, block_l);
        let vb = Vertex::lit(b, (1.0, 1.0), shade, sky, block_l);
        let vc = Vertex::lit(c, (1.0, 0.0), shade, sky, block_l);
        let vd = Vertex::lit(d, (0.0, 0.0), shade, sky, block_l);
        tris.push(Triangle { v0: va, v1: vc, v2: vb, texture: tex });
        tris.push(Triangle { v0: va, v1: vd, v2: vc, texture: tex });
    };

    if north_solid {
        let z_pos = z + 0.05;
        quad(
            Vec3::new(x, y, z_pos),
            Vec3::new(x + 1.0, y, z_pos),
            Vec3::new(x + 1.0, y + 1.0, z_pos),
            Vec3::new(x, y + 1.0, z_pos),
            0.95,
        );
    } else if south_solid {
        let z_pos = z + 0.95;
        quad(
            Vec3::new(x, y, z_pos),
            Vec3::new(x + 1.0, y, z_pos),
            Vec3::new(x + 1.0, y + 1.0, z_pos),
            Vec3::new(x, y + 1.0, z_pos),
            0.95,
        );
    } else if west_solid {
        let x_pos = x + 0.05;
        quad(
            Vec3::new(x_pos, y, z + 1.0),
            Vec3::new(x_pos, y, z),
            Vec3::new(x_pos, y + 1.0, z),
            Vec3::new(x_pos, y + 1.0, z + 1.0),
            0.90,
        );
    } else {
        let x_pos = x + 0.95;
        let _ = east_solid;
        quad(
            Vec3::new(x_pos, y, z),
            Vec3::new(x_pos, y, z + 1.0),
            Vec3::new(x_pos, y + 1.0, z + 1.0),
            Vec3::new(x_pos, y + 1.0, z),
            0.90,
        );
    }
}

fn emit_cross(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    texture: TextureId,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.5);
    let ao = 1.0;

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    let p0 = Vec3::new(x, y, z);
    let p1 = Vec3::new(x + 1.0, y, z + 1.0);
    let p2 = Vec3::new(x + 1.0, y + 1.0, z + 1.0);
    let p3 = Vec3::new(x, y + 1.0, z);

    let q0 = Vec3::new(x + 1.0, y, z);
    let q1 = Vec3::new(x, y, z + 1.0);
    let q2 = Vec3::new(x, y + 1.0, z + 1.0);
    let q3 = Vec3::new(x + 1.0, y + 1.0, z);

    push_quad(tris, p0, p1, p2, p3, texture, ao, sky, block_l);
    push_quad(tris, p1, p0, p3, p2, texture, ao, sky, block_l);
    push_quad(tris, q0, q1, q2, q3, texture, ao, sky, block_l);
    push_quad(tris, q1, q0, q3, q2, texture, ao, sky, block_l);
}

#[inline]
fn emit_waterlogged_faces(
    trans_tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    lx: i32, ly: i32, lz: i32,
    cache: &ChunkNeighborCache,
) {
    for face in Face::ALL {
        let n = face.normal();
        let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);
        if matches!(neighbor, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
            continue;
        }
        if neighbor.is_solid() && !neighbor.is_transparent() {
            continue;
        }
        if face == Face::Bottom && wy == 0 {
            continue;
        }
        emit_face(trans_tris, wx, wy, wz, face, BlockId::Water, cache);
    }
}

fn emit_ruststone_wire(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    is_lit: bool,
    cache: &ChunkNeighborCache,
) {
    let (sky, raw_bl) = cache.sample_voxel_lights(wx, wy, wz, 0.1);
    let block_l = if is_lit { (raw_bl + 0.6).min(1.0) } else { raw_bl };
    let tex = if is_lit { TextureId::RustStoneWireLit } else { TextureId::RustStoneWire };

    let y = wy as f32 + 0.02;
    let x0 = wx as f32;
    let x1 = wx as f32 + 1.0;
    let z0 = wz as f32;
    let z1 = wz as f32 + 1.0;

    push_quad_uv(
        tris,
        (Vec3::new(x0, y, z0), (0.0, 1.0)),
        (Vec3::new(x0, y, z1), (0.0, 0.0)),
        (Vec3::new(x1, y, z1), (1.0, 0.0)),
        (Vec3::new(x1, y, z0), (1.0, 1.0)),
        tex,
        1.0,
        sky,
        block_l,
    );
}

fn emit_lever(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    is_on: bool,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.25);
    let tex = TextureId::Lever;

    emit_box(
        tris,
        wx as f32 + 0.3125, wy as f32, wz as f32 + 0.375,
        wx as f32 + 0.6875, wy as f32 + 0.125, wz as f32 + 0.625,
        tex, tex, tex, sky, block_l,
    );
    if !is_on {
        emit_box(
            tris,
            wx as f32 + 0.4375, wy as f32 + 0.125, wz as f32 + 0.4375,
            wx as f32 + 0.5625, wy as f32 + 0.55, wz as f32 + 0.5625,
            tex, tex, tex, sky, block_l,
        );
    } else {
        emit_box(
            tris,
            wx as f32 + 0.4375, wy as f32 + 0.125, wz as f32 + 0.375,
            wx as f32 + 0.625, wy as f32 + 0.38, wz as f32 + 0.50,
            tex, tex, tex, sky, block_l,
        );
    }
}

fn emit_button(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    is_pressed: bool,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.1);
    let tex = TextureId::Button;
    let h = if is_pressed { 0.04 } else { 0.12 };
    emit_box(
        tris,
        wx as f32 + 0.35, wy as f32, wz as f32 + 0.35,
        wx as f32 + 0.65, wy as f32 + h, wz as f32 + 0.65,
        tex, tex, tex, sky, block_l,
    );
}

fn emit_pressure_plate(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    is_pressed: bool,
    cache: &ChunkNeighborCache,
) {
    let (sky, block_l) = cache.sample_voxel_lights(wx, wy, wz, 0.1);
    let tex = TextureId::PressurePlate;
    let h = if is_pressed { 0.025 } else { 0.0625 };
    emit_box(
        tris,
        wx as f32 + 0.0625, wy as f32, wz as f32 + 0.0625,
        wx as f32 + 0.9375, wy as f32 + h, wz as f32 + 0.9375,
        tex, tex, tex, sky, block_l,
    );
}

#[inline(always)]
fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let count = side1 as u8 + side2 as u8 + (corner && !(side1 && side2)) as u8;
    match count {
        0 => 1.00,
        1 => 0.80,
        2 => 0.65,
        _ => 0.50,
    }
}

#[inline(always)]
fn vertex_light_and_ao(
    center_sky: f32,
    center_block: f32,
    s1: (bool, f32, f32),
    s2: (bool, f32, f32),
    corner: (bool, f32, f32),
) -> (f32, f32, f32) {
    let ao = vertex_ao(s1.0, s2.0, corner.0);
    if s1.0 && s2.0 {
        (ao, center_sky, center_block)
    } else {
        let mut count = 1.0f32;
        let mut sum_sky = center_sky;
        let mut sum_block = center_block;

        if !s1.0 {
            sum_sky += s1.1;
            sum_block += s1.2;
            count += 1.0;
        }
        if !s2.0 {
            sum_sky += s2.1;
            sum_block += s2.2;
            count += 1.0;
        }
        if !corner.0 && (!s1.0 || !s2.0) {
            sum_sky += corner.1;
            sum_block += corner.2;
            count += 1.0;
        }

        (ao, sum_sky / count, sum_block / count)
    }
}

#[inline]
fn emit_face(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    face: Face,
    block: BlockId,
    cache: &ChunkNeighborCache,
) {
    let shade = face.shade();
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;

    let texture = if block == BlockId::Door {
        let below = cache.sample_block(lx, wy - 1, lz);
        if below == BlockId::Door {
            TextureId::DoorTop
        } else {
            TextureId::Door
        }
    } else if block == BlockId::Grass {
        let biome = cache.biome_at_cached(wx, wz);
        match face {
            Face::Top => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassTopSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassTopSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassTopCherry,
                Biome::AutumnForest => TextureId::GrassTopAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::GrassTopTaiga,
                _ => TextureId::GrassTop,
            },
            Face::Bottom => TextureId::Dirt,
            _ => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassSideSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassSideSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassSideCherry,
                Biome::AutumnForest => TextureId::GrassSideAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::GrassSideTaiga,
                _ => TextureId::GrassSide,
            },
        }
    } else if block == BlockId::Leaves {
        let biome = cache.biome_at_cached(wx, wz);
        match biome {
            Biome::Swamp | Biome::DarkForest => TextureId::LeavesSwamp,
            Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::LeavesSavanna,
            Biome::CherryGrove => TextureId::CherryLeaves,
            Biome::AutumnForest => TextureId::AutumnLeaves,
            Biome::SnowyTaiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::SnowLeaves,
            _ => TextureId::Leaves,
        }
    } else {
        block.texture_id(face)
    };

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;
    let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);

    let top_y = if is_fluid {
        let above = cache.sample_block(lx, wy + 1, lz);
        if above == block || matches!(above, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
            y + 1.0
        } else {
            let lvl = cache.fluid_level(IVec3::new(wx, wy, wz));
            if lvl >= 8 {
                y + 1.0
            } else {
                y + (lvl as f32 / 8.0) * 0.85 + 0.05
            }
        }
    } else {
        y + 1.0
    };

    let c000 = Vec3::new(x,       y,     z);
    let c100 = Vec3::new(x + 1.0, y,     z);
    let c010 = Vec3::new(x,       top_y, z);
    let c110 = Vec3::new(x + 1.0, top_y, z);
    let c001 = Vec3::new(x,       y,     z + 1.0);
    let c101 = Vec3::new(x + 1.0, y,     z + 1.0);
    let c011 = Vec3::new(x,       top_y, z + 1.0);
    let c111 = Vec3::new(x + 1.0, top_y, z + 1.0);

    let sample_cell = |dx: i32, dy: i32, dz: i32| -> (bool, f32, f32) {
        let is_solid = cache.solid_at(lx + dx, wy + dy, lz + dz);
        let n_lx = lx + dx;
        let n_ly = wy + dy;
        let n_lz = lz + dz;
        let sky = if is_solid {
            0.0
        } else {
            cache.sample_sky_light(n_lx, n_ly, n_lz)
        };
        let block_l = if is_solid {
            0.0
        } else {
            cache.sample_block_light(
                wx as f32 + dx as f32 + 0.5,
                wy as f32 + dy as f32 + 0.5,
                wz as f32 + dz as f32 + 0.5,
            )
        };
        (is_solid, sky, block_l)
    };

    let n = face.normal();
    let center_cell = sample_cell(n.x, n.y, n.z);
    let center_sky = center_cell.1;
    let center_block = if block == BlockId::Lava {
        1.0
    } else {
        center_cell.2
    };

    let (a, (ao_a, sky_a, blk_a), b, (ao_b, sky_b, blk_b), c, (ao_c, sky_c, blk_c), d, (ao_d, sky_d, blk_d)) = match face {
        Face::Top => (
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,1,0), sample_cell(0,1,-1), sample_cell(-1,1,-1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,1,0), sample_cell(0,1, 1), sample_cell(-1,1, 1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,1,0), sample_cell(0,1, 1), sample_cell( 1,1, 1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,1,0), sample_cell(0,1,-1), sample_cell( 1,1,-1)),
        ),
        Face::Bottom => (
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,-1,0), sample_cell(0,-1, 1), sample_cell(-1,-1, 1)),
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,-1,0), sample_cell(0,-1,-1), sample_cell(-1,-1,-1)),
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,-1,0), sample_cell(0,-1,-1), sample_cell( 1,-1,-1)),
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,-1,0), sample_cell(0,-1, 1), sample_cell( 1,-1, 1)),
        ),
        Face::North => (
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,-1), sample_cell(0,-1,-1), sample_cell( 1,-1,-1)),
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(0,-1,-1), sample_cell(-1,-1,-1)),
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(0, 1,-1), sample_cell(-1, 1,-1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,-1), sample_cell(0, 1,-1), sample_cell( 1, 1,-1)),
        ),
        Face::South => (
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,1), sample_cell(0,-1,1), sample_cell(-1,-1,1)),
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,1), sample_cell(0,-1,1), sample_cell( 1,-1,1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,1), sample_cell(0, 1,1), sample_cell( 1, 1,1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,1), sample_cell(0, 1,1), sample_cell(-1, 1,1)),
        ),
        Face::East => (
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0, 1), sample_cell(1,-1,0), sample_cell(1,-1, 1)),
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0,-1), sample_cell(1,-1,0), sample_cell(1,-1,-1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0,-1), sample_cell(1, 1,0), sample_cell(1, 1,-1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0, 1), sample_cell(1, 1,0), sample_cell(1, 1, 1)),
        ),
        Face::West => (
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(-1,-1,0), sample_cell(-1,-1,-1)),
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0, 1), sample_cell(-1,-1,0), sample_cell(-1,-1, 1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0, 1), sample_cell(-1, 1,0), sample_cell(-1, 1, 1)),
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(-1, 1,0), sample_cell(-1, 1,-1)),
        ),
    };

    let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade * ao_a, sky_light: sky_a, block_light: blk_a };
    let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade * ao_b, sky_light: sky_b, block_light: blk_b };
    let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade * ao_c, sky_light: sky_c, block_light: blk_c };
    let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade * ao_d, sky_light: sky_d, block_light: blk_d };

    let light_ac = ao_a + ao_c;
    let light_bd = ao_b + ao_d;
    if light_ac < light_bd {
        tris.push(Triangle { v0: vb, v1: vc, v2: vd, texture });
        tris.push(Triangle { v0: vb, v1: vd, v2: va, texture });
    } else {
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
    }
}
