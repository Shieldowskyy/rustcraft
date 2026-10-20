// Converts a chunk's voxel grid into a list of world-space, textured
// triangles. Uses simple face-culling (skip faces touching a solid, opaque
// neighbor) rather than full greedy meshing, to keep the code easy to
// follow. Still efficient enough for modest view distances at
// software-render scale.

use crate::core::math::{IVec3, Vec3};
use crate::graphics::atlas::TextureId;
use crate::world::block::{BlockId, Face};
use crate::world::{get_biome_at, Biome, Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

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
    pub lights: Vec<LightSource>,
    pub fluid_map: std::collections::HashMap<IVec3, u8>,
}

pub struct AsyncMesher {
    #[allow(dead_code)]
    req_tx: std::sync::mpsc::Sender<ChunkMeshJob>,
    res_rx: std::sync::mpsc::Receiver<((i32, i32), Vec<Triangle>)>,
    #[allow(dead_code)]
    res_tx: std::sync::mpsc::Sender<((i32, i32), Vec<Triangle>)>,
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

        for i in 0..num_threads {
            let req_rx_clone = std::sync::Arc::clone(&req_rx);
            let res_tx_clone = res_tx.clone();
            let _ = std::thread::Builder::new()
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
                        sceKernelChangeThreadCpuAffinityMask(thid, mask);
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
                                let mesh = build_chunk_mesh_from_job(&job);
                                if res_tx_clone.send((key, mesh)).is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                });
        }

        Self { req_tx, res_rx, res_tx }
    }

    pub fn submit_job(&self, job: ChunkMeshJob) {
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        {
            let key = job.key;
            let mesh = build_chunk_mesh_from_job(&job);
            let _ = self.res_tx.send((key, mesh));
        }
        #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
        {
            let _ = self.req_tx.send(job);
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
    pub heightmap: [u8; 18 * 18],
    pub leaf_heightmap: [u8; 18 * 18],
    pub lights: Vec<LightSource>,
    pub fluid_map: Option<&'a std::collections::HashMap<IVec3, u8>>,
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
                    radius: 12.0,
                    intensity: 1.0,
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
            heightmap: [0u8; 18 * 18],
            leaf_heightmap: [0u8; 18 * 18],
            lights,
            fluid_map: None,
        };

        for lx in -1..=16 {
            for lz in -1..=16 {
                let mut h = 0u8;
                let mut leaf_h = 0u8;
                for ly in (0..CHUNK_SIZE_Y).rev() {
                    let b = cache.sample_block(lx, ly, lz);
                    if b.is_leaf() && leaf_h == 0 {
                        leaf_h = (ly + 1) as u8;
                    }
                    if b.is_opaque() {
                        h = (ly + 1) as u8;
                        break;
                    }
                }
                let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
                cache.heightmap[idx] = h;
                cache.leaf_heightmap[idx] = leaf_h;
            }
        }

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
            heightmap: [0u8; 18 * 18],
            leaf_heightmap: [0u8; 18 * 18],
            lights: job.lights.clone(),
            fluid_map: Some(&job.fluid_map),
        };

        for lx in -1..=16 {
            for lz in -1..=16 {
                let mut h = 0u8;
                let mut leaf_h = 0u8;
                for ly in (0..CHUNK_SIZE_Y).rev() {
                    let b = cache.sample_block(lx, ly, lz);
                    if b.is_leaf() && leaf_h == 0 {
                        leaf_h = (ly + 1) as u8;
                    }
                    if b.is_opaque() {
                        h = (ly + 1) as u8;
                        break;
                    }
                }
                let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
                cache.heightmap[idx] = h;
                cache.leaf_heightmap[idx] = leaf_h;
            }
        }

        cache
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
                0.85 // Soft natural shadow under tree canopy
            } else {
                1.0  // Full direct sunlight
            }
        } else {
            let depth = (opaque_h - ly).max(1);
            if depth >= 8 {
                0.0 // Pitch black in deep caves without torches
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
    pub fn sample_block_light(&self, wx: f32, wy: f32, wz: f32) -> f32 {
        if self.lights.is_empty() {
            return 0.0;
        }
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
                // Cubic Hermite smoothstep curve for natural soft falloff
                let falloff = t * t * (3.0 - 2.0 * t);
                let l = falloff * light.intensity;
                if l > max_l {
                    // Check if light is occluded by solid obstacle
                    if !self.is_light_occluded(light.pos, wx.floor() as i32, wy.floor() as i32, wz.floor() as i32) {
                        max_l = l;
                    }
                }
            }
        }
        max_l
    }
}

pub const SUBCHUNK_SIZE: i32 = 16;
pub const NUM_SUBCHUNKS: usize = (CHUNK_SIZE_Y / SUBCHUNK_SIZE) as usize; // 4


pub fn build_subchunk_mesh_internal(
    chunk: &Chunk,
    _world: &World,
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    let mut solid_tris = Vec::with_capacity(768);
    let mut trans_tris = Vec::with_capacity(128);

    let base_x = cache.base_x;
    let base_z = cache.base_z;
    let blocks = chunk.blocks.as_slice();

    let y_start = (sub_y as i32) * SUBCHUNK_SIZE;
    let y_end = y_start + SUBCHUNK_SIZE;

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
                            match face {
                                Face::Top => neighbor != block,
                                Face::Bottom => wy > 0 && neighbor != block && (neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)),
                                _ => {
                                    if neighbor != block {
                                        neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)
                                    } else {
                                        let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                                        if above_neighbor != block {
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
                    } else {
                        if face == Face::Bottom && wy == 0 {
                            false
                        } else {
                            neighbor == BlockId::Air
                            || (neighbor.is_transparent() && neighbor != block)
                        }
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

pub fn build_subchunk_mesh(chunk: &Chunk, world: &World, sub_y: usize) -> Vec<Triangle> {
    if sub_y >= NUM_SUBCHUNKS {
        return Vec::new();
    }
    let cache = ChunkNeighborCache::new(chunk, world);
    let (mut solid, trans) = build_subchunk_mesh_internal(chunk, world, sub_y, &cache);
    solid.extend(trans);
    solid
}

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let mut solid_tris = Vec::with_capacity(3072);
    let mut trans_tris = Vec::with_capacity(512);
    let cache = ChunkNeighborCache::new(chunk, world);

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_internal(chunk, world, sub_y, &cache);
        solid_tris.extend(sub_solid);
        trans_tris.extend(sub_trans);
    }

    solid_tris.extend(trans_tris);
    solid_tris
}

pub fn build_chunk_mesh_from_job(job: &ChunkMeshJob) -> Vec<Triangle> {
    let mut solid_tris = Vec::with_capacity(3072);
    let mut trans_tris = Vec::with_capacity(512);
    let cache = ChunkNeighborCache::from_job(job);
    let dummy_chunk = Chunk {
        cx: job.key.0,
        cz: job.key.1,
        blocks: job.center_blocks.clone(),
        dirty: false,
    };

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = {
            let mut sub_solid = Vec::with_capacity(768);
            let mut sub_trans = Vec::with_capacity(128);
            let base_x = cache.base_x;
            let base_z = cache.base_z;
            let blocks = dummy_chunk.blocks.as_slice();

            let y_start = (sub_y as i32) * SUBCHUNK_SIZE;
            let y_end = y_start + SUBCHUNK_SIZE;

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
                            emit_torch(&mut sub_solid, wx, wy, wz, &cache);
                            continue;
                        }
                        if block == BlockId::Door {
                            emit_door(&mut sub_solid, wx, wy, wz, false, &cache);
                            continue;
                        }
                        if block == BlockId::DoorOpen {
                            emit_door(&mut sub_solid, wx, wy, wz, true, &cache);
                            continue;
                        }
                        if block == BlockId::Bed {
                            emit_bed(&mut sub_solid, wx, wy, wz, &cache);
                            continue;
                        }
                        if block == BlockId::WoodenSlab || block == BlockId::StoneSlab {
                            emit_slab(&mut sub_solid, wx, wy, wz, block, &cache);
                            continue;
                        }
                        if block == BlockId::WoodenStairs || block == BlockId::StoneStairs {
                            emit_stairs(&mut sub_solid, wx, wy, wz, block, &cache);
                            continue;
                        }
                        if block == BlockId::Ladder {
                            emit_ladder(&mut sub_solid, wx, wy, wz, &cache);
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
                                    match face {
                                        Face::Top => neighbor != block,
                                        Face::Bottom => wy > 0 && neighbor != block && (neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)),
                                        _ => {
                                            if neighbor != block {
                                                neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)
                                            } else {
                                                let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                                                if above_neighbor != block {
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
                            } else {
                                if face == Face::Bottom && wy == 0 {
                                    false
                                } else {
                                    neighbor == BlockId::Air
                                    || (neighbor.is_transparent() && neighbor != block)
                                }
                            };

                            if visible {
                                if is_trans {
                                    emit_face(&mut sub_trans, wx, wy, wz, face, block, &cache);
                                } else {
                                    emit_face(&mut sub_solid, wx, wy, wz, face, block, &cache);
                                }
                            }
                        }
                    }
                }
            }
            (sub_solid, sub_trans)
        };
        solid_tris.extend(sub_solid);
        trans_tris.extend(sub_trans);
    }

    solid_tris.extend(trans_tris);
    solid_tris
}

fn emit_torch(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    _cache: &ChunkNeighborCache,
) {
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

    let quad = |tris: &mut Vec<Triangle>, a: Vec3, b: Vec3, c: Vec3, d: Vec3, tex: TextureId, shade: f32| {
        let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade, sky_light: 1.0, block_light: 1.0 };
        let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade, sky_light: 1.0, block_light: 1.0 };
        let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade, sky_light: 1.0, block_light: 1.0 };
        let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade, sky_light: 1.0, block_light: 1.0 };
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture: tex });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture: tex });
    };

    let tex = TextureId::Torch;
    // Top
    quad(tris, c010, c011, c111, c110, tex, 1.0);
    // North
    quad(tris, c100, c000, c010, c110, tex, 0.95);
    // South
    quad(tris, c001, c101, c111, c011, tex, 0.95);
    // East
    quad(tris, c101, c100, c110, c111, tex, 0.90);
    // West
    quad(tris, c000, c001, c011, c010, tex, 0.90);
    // Bottom
    if wy > 0 {
        quad(tris, c001, c000, c100, c101, tex, 0.70);
    }
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

    // Door thickness: 3/16 = 0.1875
    let thick = 0.1875f32;

    let (x0, x1, z0, z1) = if !is_open {
        // Closed door: positioned along front edge (Z in [0, thick], X in [0, 1])
        // Hinge is at x0 (x=0), handle is at x1 (x=1)
        (x, x + 1.0, z, z + thick)
    } else {
        // Open door: swung 90 degrees against the wall (X in [0, thick], Z in [0, 1])
        // Hinge is at z0 (z=0), handle is at z1 (z=1)
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

    let sky = cache.sample_sky_light(lx, wy, lz);
    let block_l = cache.sample_block_light(x + 0.5, y + 0.5, z + 0.5);

    let quad_uv = |tris: &mut Vec<Triangle>,
                   a: (Vec3, (f32, f32)),
                   b: (Vec3, (f32, f32)),
                   c: (Vec3, (f32, f32)),
                   d: (Vec3, (f32, f32)),
                   texture: TextureId,
                   shade: f32| {
        let va = Vertex { pos: a.0, uv: a.1, ao: shade, sky_light: sky, block_light: block_l };
        let vb = Vertex { pos: b.0, uv: b.1, ao: shade, sky_light: sky, block_light: block_l };
        let vc = Vertex { pos: c.0, uv: c.1, ao: shade, sky_light: sky, block_light: block_l };
        let vd = Vertex { pos: d.0, uv: d.1, ao: shade, sky_light: sky, block_light: block_l };
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
    };

    if !is_open {
        // Closed door: Hinge is at x0 (U=0), Handle is at x1 (U=1)
        // North (Front, facing -Z)
        quad_uv(
            tris,
            (c100, (1.0, 1.0)),
            (c000, (0.0, 1.0)),
            (c010, (0.0, 0.0)),
            (c110, (1.0, 0.0)),
            tex,
            0.85,
        );
        // South (Back, facing +Z)
        quad_uv(
            tris,
            (c001, (0.0, 1.0)),
            (c101, (1.0, 1.0)),
            (c111, (1.0, 0.0)),
            (c011, (0.0, 0.0)),
            tex,
            0.85,
        );
        // East (Outer side edge at x1)
        quad_uv(
            tris,
            (c101, (0.0, 1.0)),
            (c100, (1.0, 1.0)),
            (c110, (1.0, 0.0)),
            (c111, (0.0, 0.0)),
            side_tex,
            0.70,
        );
        // West (Hinge side edge at x0)
        quad_uv(
            tris,
            (c000, (0.0, 1.0)),
            (c001, (1.0, 1.0)),
            (c011, (1.0, 0.0)),
            (c010, (0.0, 0.0)),
            side_tex,
            0.70,
        );
    } else {
        // Open door: Hinge is at z0 (U=0), Handle is at z1 (U=1)
        // East (facing +X)
        quad_uv(
            tris,
            (c101, (1.0, 1.0)),
            (c100, (0.0, 1.0)),
            (c110, (0.0, 0.0)),
            (c111, (1.0, 0.0)),
            tex,
            0.85,
        );
        // West (facing -X)
        quad_uv(
            tris,
            (c000, (0.0, 1.0)),
            (c001, (1.0, 1.0)),
            (c011, (1.0, 0.0)),
            (c010, (0.0, 0.0)),
            tex,
            0.85,
        );
        // North (Hinge side edge at z0)
        quad_uv(
            tris,
            (c100, (0.0, 1.0)),
            (c000, (1.0, 1.0)),
            (c010, (1.0, 0.0)),
            (c110, (0.0, 0.0)),
            side_tex,
            0.70,
        );
        // South (Outer side edge at z1)
        quad_uv(
            tris,
            (c001, (0.0, 1.0)),
            (c101, (1.0, 1.0)),
            (c111, (1.0, 0.0)),
            (c011, (0.0, 0.0)),
            side_tex,
            0.70,
        );
    }
    // Top & Bottom
    quad_uv(
        tris,
        (c010, (0.0, 1.0)),
        (c011, (1.0, 1.0)),
        (c111, (1.0, 0.0)),
        (c110, (0.0, 0.0)),
        side_tex,
        1.0,
    );
    if wy > 0 {
        quad_uv(
            tris,
            (c001, (0.0, 1.0)),
            (c000, (1.0, 1.0)),
            (c100, (1.0, 0.0)),
            (c101, (0.0, 0.0)),
            side_tex,
            0.5,
        );
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

    let mut quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, tex: TextureId, shade: f32| {
        let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade, sky_light: sky, block_light: block_l };
        let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade, sky_light: sky, block_light: block_l };
        let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade, sky_light: sky, block_light: block_l };
        let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade, sky_light: sky, block_light: block_l };
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture: tex });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture: tex });
    };

    // Top
    quad(c010, c011, c111, c110, top_tex, 1.0);
    // North (-Z)
    quad(c100, c000, c010, c110, side_tex, 0.85);
    // South (+Z)
    quad(c001, c101, c111, c011, side_tex, 0.85);
    // East (+X)
    quad(c101, c100, c110, c111, side_tex, 0.70);
    // West (-X)
    quad(c000, c001, c011, c010, side_tex, 0.70);
    // Bottom
    quad(c001, c000, c100, c101, bottom_tex, 0.50);
}

fn emit_slab(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    block: BlockId,
    cache: &ChunkNeighborCache,
) {
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;
    let sky = cache.sample_sky_light(lx, wy, lz);
    let block_l = cache.sample_block_light(wx as f32 + 0.5, wy as f32 + 0.25, wz as f32 + 0.5);
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
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;
    let sky = cache.sample_sky_light(lx, wy, lz);
    let block_l = cache.sample_block_light(wx as f32 + 0.5, wy as f32 + 0.25, wz as f32 + 0.5);
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
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;
    let sky = cache.sample_sky_light(lx, wy, lz);
    let block_l = cache.sample_block_light(wx as f32 + 0.5, wy as f32 + 0.5, wz as f32 + 0.5);
    let tex = if block == BlockId::StoneStairs { TextureId::Stone } else { TextureId::Planks };
    // Lower full base step (0.0 .. 0.5)
    emit_box(
        tris,
        wx as f32, wy as f32, wz as f32,
        wx as f32 + 1.0, wy as f32 + 0.5, wz as f32 + 1.0,
        tex, tex, tex, sky, block_l,
    );
    // Upper back step (0.5 .. 1.0, z in 0.0 .. 0.5)
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
    let sky = cache.sample_sky_light(lx, wy, lz);
    let block_l = cache.sample_block_light(wx as f32 + 0.5, wy as f32 + 0.5, wz as f32 + 0.5);
    let tex = TextureId::Ladder;

    let north_solid = cache.sample_block(lx, wy, lz - 1).is_solid();
    let south_solid = cache.sample_block(lx, wy, lz + 1).is_solid();
    let west_solid = cache.sample_block(lx - 1, wy, lz).is_solid();

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    let mut quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, shade: f32| {
        let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade, sky_light: sky, block_light: block_l };
        let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade, sky_light: sky, block_light: block_l };
        let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade, sky_light: sky, block_light: block_l };
        let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade, sky_light: sky, block_light: block_l };
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture: tex });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture: tex });
    };

    if north_solid {
        let z_pos = z + 0.05;
        quad(Vec3::new(x + 1.0, y, z_pos), Vec3::new(x, y, z_pos), Vec3::new(x, y + 1.0, z_pos), Vec3::new(x + 1.0, y + 1.0, z_pos), 0.95);
    } else if south_solid {
        let z_pos = z + 0.95;
        quad(Vec3::new(x, y, z_pos), Vec3::new(x + 1.0, y, z_pos), Vec3::new(x + 1.0, y + 1.0, z_pos), Vec3::new(x, y + 1.0, z_pos), 0.95);
    } else if west_solid {
        let x_pos = x + 0.05;
        quad(Vec3::new(x_pos, y, z), Vec3::new(x_pos, y, z + 1.0), Vec3::new(x_pos, y + 1.0, z + 1.0), Vec3::new(x_pos, y + 1.0, z), 0.90);
    } else {
        let x_pos = x + 0.95;
        quad(Vec3::new(x_pos, y, z + 1.0), Vec3::new(x_pos, y, z), Vec3::new(x_pos, y + 1.0, z), Vec3::new(x_pos, y + 1.0, z + 1.0), 0.90);
    }
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
        let (biome, _) = get_biome_at(wx, wz, cache.seed);
        match face {
            Face::Top => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassTopSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassTopSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassTopCherry,
                Biome::AutumnForest => TextureId::GrassTopAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes => TextureId::GrassTopTaiga,
                _ => TextureId::GrassTop,
            },
            Face::Bottom => TextureId::Dirt,
            _ => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassSideSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassSideSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassSideCherry,
                Biome::AutumnForest => TextureId::GrassSideAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes => TextureId::GrassSideTaiga,
                _ => TextureId::GrassSide,
            },
        }
    } else if block == BlockId::Leaves {
        let (biome, _) = get_biome_at(wx, wz, cache.seed);
        match biome {
            Biome::Swamp | Biome::DarkForest => TextureId::LeavesSwamp,
            Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::LeavesSavanna,
            Biome::CherryGrove => TextureId::CherryLeaves,
            Biome::AutumnForest => TextureId::AutumnLeaves,
            Biome::SnowyTaiga | Biome::IceSpikes => TextureId::SnowLeaves,
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
        if above == block {
            y + 1.0
        } else {
            let lvl = cache.fluid_level(IVec3::new(wx, wy, wz));
            if lvl >= 8 {
                y + 0.90
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Chunk, World};

    #[test]
    fn test_chunk_bottom_face_culling_at_y0() {
        let mut world = World::new();
        let mut chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };
        // Place stone at y = 0
        chunk.set_local(0, 0, 0, BlockId::Stone);
        // Place stone at y = 10 (floating with air below it)
        chunk.set_local(5, 10, 5, BlockId::Stone);

        world.chunks.insert((0, 0), chunk);
        let mesh = build_chunk_mesh(world.chunks.get(&(0, 0)).unwrap(), &world);

        // Verify that NO triangles have all vertex y coordinates equal to 0.0 with bottom face orientation
        let has_y0_bottom_face = mesh.iter().any(|tri| {
            tri.v0.pos.y == 0.0 && tri.v1.pos.y == 0.0 && tri.v2.pos.y == 0.0
                && tri.v0.pos.x <= 1.0 && tri.v0.pos.z <= 1.0
        });
        assert!(!has_y0_bottom_face, "Bottom face at y=0 should be culled");

        // Verify that floating stone at y=10 DOES have a bottom face at y=10.0
        let has_y10_bottom_face = mesh.iter().any(|tri| {
            tri.v0.pos.y == 10.0 && tri.v1.pos.y == 10.0 && tri.v2.pos.y == 10.0
        });
        assert!(has_y10_bottom_face, "Bottom face at y=10 with air below should be generated");
    }

    #[test]
    fn test_subchunk_mesh_building() {
        let mut world = World::new();
        let mut chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };

        // Create a stone at y=5 (subchunk 0) and at y=20 (subchunk 1)
        chunk.set_local(2, 5, 2, BlockId::Stone);
        chunk.set_local(2, 20, 2, BlockId::Stone);

        world.chunks.insert((0, 0), chunk);
        let c = world.chunks.get(&(0, 0)).unwrap();

        let sub0 = build_subchunk_mesh(c, &world, 0);
        let sub1 = build_subchunk_mesh(c, &world, 1);
        let sub2 = build_subchunk_mesh(c, &world, 2);

        assert!(!sub0.is_empty(), "Subchunk 0 (y 0..16) should contain faces for block at y=5");
        assert!(!sub1.is_empty(), "Subchunk 1 (y 16..32) should contain faces for block at y=20");
        assert!(sub2.is_empty(), "Subchunk 2 (y 32..48) should be empty");

        // Verify that all triangles have standard 1:1 UVs within [0.0, 1.0] to prevent texture stretching
        for tri in sub0.iter().chain(sub1.iter()) {
            for v in [&tri.v0, &tri.v1, &tri.v2] {
                assert!(v.uv.0 >= 0.0 && v.uv.0 <= 1.0, "UV X coordinate out of 1:1 range: {}", v.uv.0);
                assert!(v.uv.1 >= 0.0 && v.uv.1 <= 1.0, "UV Y coordinate out of 1:1 range: {}", v.uv.1);
            }
        }
    }

    #[test]
    fn test_light_occlusion_and_smooth_lighting() {
        let mut world = World::new();
        let mut chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };
        // Floor at y=0
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_local(x, 0, z, BlockId::Stone);
            }
        }
        // Wall at x=5 from y=1..5
        for y in 1..=4 {
            for z in 0..16 {
                chunk.set_local(5, y, z, BlockId::Stone);
            }
        }
        world.chunks.insert((0, 0), chunk);

        // Torch at (3, 2, 8) - west side of the wall
        world.torches.insert(IVec3::new(3, 2, 8));

        let cache = ChunkNeighborCache::new(world.chunks.get(&(0, 0)).unwrap(), &world);

        // Light in front of torch (4, 2, 8) should be lit (> 0.5)
        let light_near = cache.sample_block_light(4.5, 2.5, 8.5);
        assert!(light_near > 0.5, "Light near torch should be bright, got {}", light_near);

        // Light on the other side of the wall (6, 2, 8) must be occluded (0.0)
        let light_behind_wall = cache.sample_block_light(6.5, 2.5, 8.5);
        assert_eq!(light_behind_wall, 0.0, "Light behind stone wall should be 0.0 (occluded), got {}", light_behind_wall);
    }

    #[test]
    fn test_chunk_boundary_sky_light() {
        let mut world = World::new();
        let mut chunk_west = Chunk {
            cx: -1,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };
        // In west chunk, column at lx=15 (adjacent to center chunk lx=0) has terrain up to y=20
        for y in 0..=20 {
            chunk_west.set_local(15, y, 5, BlockId::Stone);
        }
        world.chunks.insert((-1, 0), chunk_west);

        let chunk_center = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };
        world.chunks.insert((0, 0), chunk_center);

        let cache = ChunkNeighborCache::new(world.chunks.get(&(0, 0)).unwrap(), &world);

        // At lx = -1, lz = 5, ly = 5 (deep below y=20 terrain), sky light should be dark (0.0), NOT 1.0
        let sky = cache.sample_sky_light(-1, 5, 5);
        assert_eq!(sky, 0.0, "Deep underground at chunk boundary should be 0.0 sky light, got {}", sky);

        // At lx = -1, lz = 5, ly = 25 (above y=20 terrain), sky light should be full (1.0)
        let sky_above = cache.sample_sky_light(-1, 25, 5);
        assert_eq!(sky_above, 1.0, "Above terrain at chunk boundary should be 1.0 sky light, got {}", sky_above);
    }
}
