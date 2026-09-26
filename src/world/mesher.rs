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
    pub world: &'a World,
    pub base_x: i32,
    pub base_z: i32,
    pub heightmap: [u8; 16 * 16],
    pub leaf_heightmap: [u8; 16 * 16],
    pub lights: Vec<LightSource>,
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

        // 1. Column heights for opaque blocks (Opaque Heightmap) and tree foliage (Leaf Heightmap)
        let mut heightmap = [0u8; 16 * 16];
        let mut leaf_heightmap = [0u8; 16 * 16];
        let blocks = chunk.blocks.as_slice();
        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let mut h = 0u8;
                let mut leaf_h = 0u8;
                for ly in (0..CHUNK_SIZE_Y).rev() {
                    let b = BlockId::from_u8(blocks[((lx << 10) | (ly << 4) | lz) as usize]);
                    if b.is_leaf() && leaf_h == 0 {
                        leaf_h = (ly + 1) as u8;
                    }
                    if b.is_opaque() {
                        h = (ly + 1) as u8;
                        break;
                    }
                }
                let idx = ((lx << 4) | lz) as usize;
                heightmap[idx] = h;
                leaf_heightmap[idx] = leaf_h;
            }
        }

        // 2. Light sources query only from registered nearby torches (O(torch_count) instead of 160k loops!)
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

        Self {
            center_blocks: blocks,
            east_blocks,
            west_blocks,
            south_blocks,
            north_blocks,
            se_blocks,
            sw_blocks,
            ne_blocks,
            nw_blocks,
            world,
            base_x,
            base_z,
            heightmap,
            leaf_heightmap,
            lights,
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
    pub fn sample_sky_light(&self, lx: i32, ly: i32, lz: i32) -> f32 {
        if (lx as u32) < 16 && (lz as u32) < 16 {
            let idx = ((lx << 4) | lz) as usize;
            let opaque_h = self.heightmap[idx] as i32;
            let leaf_h = self.leaf_heightmap[idx] as i32;

            if ly >= opaque_h {
                if leaf_h > 0 && ly < leaf_h {
                    0.88 // Soft natural shadow under tree canopy
                } else {
                    1.0  // Full direct sunlight
                }
            } else {
                let depth = (opaque_h - ly).max(1);
                (0.55 * (0.82f32).powi(depth)).clamp(0.08, 0.55)
            }
        } else {
            1.0
        }
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
                let l = (1.0 - dist / r) * light.intensity;
                if l > max_l {
                    max_l = l;
                }
            }
        }
        max_l
    }
}

pub const SUBCHUNK_SIZE: i32 = 16;
pub const NUM_SUBCHUNKS: usize = (CHUNK_SIZE_Y / SUBCHUNK_SIZE) as usize; // 4

#[derive(Clone, Copy, PartialEq)]
struct FaceCell {
    texture: TextureId,
    ao: [u8; 4],
    sky_light: u8,
    block_light: u8,
    is_trans: bool,
}

#[inline]
fn determine_texture(
    block: BlockId,
    face: Face,
    wx: i32, wy: i32, wz: i32,
    lx: i32, lz: i32,
    cache: &ChunkNeighborCache,
) -> TextureId {
    if block == BlockId::Door {
        let below = cache.sample_block(lx, wy - 1, lz);
        if below == BlockId::Door {
            TextureId::DoorTop
        } else {
            TextureId::Door
        }
    } else if block == BlockId::Grass {
        let (biome, _) = get_biome_at(wx, wz, cache.world.seed);
        match face {
            Face::Top => match biome {
                Biome::Swamp => TextureId::GrassTopSwamp,
                Biome::Savanna | Biome::Desert => TextureId::GrassTopSavanna,
                Biome::CherryGrove => TextureId::GrassTopCherry,
                Biome::AutumnForest => TextureId::GrassTopAutumn,
                Biome::SnowyTaiga | Biome::Mountain => TextureId::GrassTopTaiga,
                _ => TextureId::GrassTop,
            },
            Face::Bottom => TextureId::Dirt,
            _ => match biome {
                Biome::Swamp => TextureId::GrassSideSwamp,
                Biome::Savanna | Biome::Desert => TextureId::GrassSideSavanna,
                Biome::CherryGrove => TextureId::GrassSideCherry,
                Biome::AutumnForest => TextureId::GrassSideAutumn,
                Biome::SnowyTaiga | Biome::Mountain => TextureId::GrassSideTaiga,
                _ => TextureId::GrassSide,
            },
        }
    } else if block == BlockId::Leaves {
        let (biome, _) = get_biome_at(wx, wz, cache.world.seed);
        match biome {
            Biome::Swamp => TextureId::LeavesSwamp,
            Biome::Savanna | Biome::Desert => TextureId::LeavesSavanna,
            Biome::CherryGrove => TextureId::CherryLeaves,
            Biome::AutumnForest => TextureId::AutumnLeaves,
            Biome::SnowyTaiga => TextureId::SnowLeaves,
            _ => TextureId::Leaves,
        }
    } else {
        block.texture_id(face)
    }
}

#[inline]
fn compute_face_ao(
    face: Face,
    lx: i32, wy: i32, lz: i32,
    cache: &ChunkNeighborCache,
) -> [f32; 4] {
    let s = |dx: i32, dy: i32, dz: i32| cache.solid_at(lx + dx, wy + dy, lz + dz);
    match face {
        Face::Top => [
            vertex_ao(s(-1,1,0), s(0,1,-1), s(-1,1,-1)),
            vertex_ao(s(-1,1,0), s(0,1, 1), s(-1,1, 1)),
            vertex_ao(s( 1,1,0), s(0,1, 1), s( 1,1, 1)),
            vertex_ao(s( 1,1,0), s(0,1,-1), s( 1,1,-1)),
        ],
        Face::Bottom => [
            vertex_ao(s(-1,-1,0), s(0,-1, 1), s(-1,-1, 1)),
            vertex_ao(s(-1,-1,0), s(0,-1,-1), s(-1,-1,-1)),
            vertex_ao(s( 1,-1,0), s(0,-1,-1), s( 1,-1,-1)),
            vertex_ao(s( 1,-1,0), s(0,-1, 1), s( 1,-1, 1)),
        ],
        Face::North => [
            vertex_ao(s( 1,0,-1), s(0,-1,-1), s( 1,-1,-1)),
            vertex_ao(s(-1,0,-1), s(0,-1,-1), s(-1,-1,-1)),
            vertex_ao(s(-1,0,-1), s(0, 1,-1), s(-1, 1,-1)),
            vertex_ao(s( 1,0,-1), s(0, 1,-1), s( 1, 1,-1)),
        ],
        Face::South => [
            vertex_ao(s(-1,0,1), s(0,-1,1), s(-1,-1,1)),
            vertex_ao(s( 1,0,1), s(0,-1,1), s( 1,-1,1)),
            vertex_ao(s( 1,0,1), s(0, 1,1), s( 1, 1,1)),
            vertex_ao(s(-1,0,1), s(0, 1,1), s(-1, 1,1)),
        ],
        Face::East => [
            vertex_ao(s(1,0, 1), s(1,-1,0), s(1,-1, 1)),
            vertex_ao(s(1,0,-1), s(1,-1,0), s(1,-1,-1)),
            vertex_ao(s(1,0,-1), s(1, 1,0), s(1, 1,-1)),
            vertex_ao(s(1,0, 1), s(1, 1,0), s(1, 1, 1)),
        ],
        Face::West => [
            vertex_ao(s(-1,0,-1), s(-1,-1,0), s(-1,-1,-1)),
            vertex_ao(s(-1,0, 1), s(-1,-1,0), s(-1,-1, 1)),
            vertex_ao(s(-1,0, 1), s(-1, 1,0), s(-1, 1, 1)),
            vertex_ao(s(-1,0,-1), s(-1, 1,0), s(-1, 1,-1)),
        ],
    }
}

fn emit_greedy_quad(
    solid_tris: &mut Vec<Triangle>,
    trans_tris: &mut Vec<Triangle>,
    wx: f32, wy: f32, wz: f32,
    w: f32, h: f32,
    face: Face,
    cell: &FaceCell,
) {
    let shade = face.shade();
    let ao_a = cell.ao[0] as f32 / 100.0;
    let ao_b = cell.ao[1] as f32 / 100.0;
    let ao_c = cell.ao[2] as f32 / 100.0;
    let ao_d = cell.ao[3] as f32 / 100.0;
    let sky = cell.sky_light as f32 / 100.0;
    let block_l = cell.block_light as f32 / 100.0;

    let (p_a, uv_a, p_b, uv_b, p_c, uv_c, p_d, uv_d) = match face {
        Face::Top => (
            Vec3::new(wx,     wy + 1.0, wz),     (0.0, w),
            Vec3::new(wx,     wy + 1.0, wz + h), (h,   w),
            Vec3::new(wx + w, wy + 1.0, wz + h), (h,   0.0),
            Vec3::new(wx + w, wy + 1.0, wz),     (0.0, 0.0),
        ),
        Face::Bottom => (
            Vec3::new(wx,     wy, wz + h), (0.0, w),
            Vec3::new(wx,     wy, wz),     (h,   w),
            Vec3::new(wx + w, wy, wz),     (h,   0.0),
            Vec3::new(wx + w, wy, wz + h), (0.0, 0.0),
        ),
        Face::North => (
            Vec3::new(wx + w, wy,     wz), (0.0, h),
            Vec3::new(wx,     wy,     wz), (w,   h),
            Vec3::new(wx,     wy + h, wz), (w,   0.0),
            Vec3::new(wx + w, wy + h, wz), (0.0, 0.0),
        ),
        Face::South => (
            Vec3::new(wx,     wy,     wz + 1.0), (0.0, h),
            Vec3::new(wx + w, wy,     wz + 1.0), (w,   h),
            Vec3::new(wx + w, wy + h, wz + 1.0), (w,   0.0),
            Vec3::new(wx,     wy + h, wz + 1.0), (0.0, 0.0),
        ),
        Face::West => (
            Vec3::new(wx, wy,     wz),     (0.0, h),
            Vec3::new(wx, wy,     wz + w), (w,   h),
            Vec3::new(wx, wy + h, wz + w), (w,   0.0),
            Vec3::new(wx, wy + h, wz),     (0.0, 0.0),
        ),
        Face::East => (
            Vec3::new(wx + 1.0, wy,     wz + w), (0.0, h),
            Vec3::new(wx + 1.0, wy,     wz),     (w,   h),
            Vec3::new(wx + 1.0, wy + h, wz),     (w,   0.0),
            Vec3::new(wx + 1.0, wy + h, wz + w), (0.0, 0.0),
        ),
    };

    let va = Vertex { pos: p_a, uv: uv_a, ao: shade * ao_a, sky_light: sky, block_light: block_l };
    let vb = Vertex { pos: p_b, uv: uv_b, ao: shade * ao_b, sky_light: sky, block_light: block_l };
    let vc = Vertex { pos: p_c, uv: uv_c, ao: shade * ao_c, sky_light: sky, block_light: block_l };
    let vd = Vertex { pos: p_d, uv: uv_d, ao: shade * ao_d, sky_light: sky, block_light: block_l };

    let target = if cell.is_trans { trans_tris } else { solid_tris };
    target.push(Triangle { v0: va, v1: vb, v2: vc, texture: cell.texture });
    target.push(Triangle { v0: va, v1: vc, v2: vd, texture: cell.texture });
}

pub fn build_subchunk_mesh_internal(
    chunk: &Chunk,
    world: &World,
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    let mut solid_tris = Vec::with_capacity(512);
    let mut trans_tris = Vec::with_capacity(128);

    let base_x = cache.base_x;
    let base_z = cache.base_z;
    let blocks = chunk.blocks.as_slice();

    let y_start = (sub_y as i32) * SUBCHUNK_SIZE;
    let y_end = y_start + SUBCHUNK_SIZE;

    // 1. Top & Bottom Greedy Meshing (slicing along Y)
    for ly in y_start..y_end {
        let wy = ly;
        for face in [Face::Top, Face::Bottom] {
            if face == Face::Bottom && wy == 0 {
                continue;
            }
            let n = face.normal();
            let mut mask = [[None; CHUNK_SIZE_Z as usize]; CHUNK_SIZE_X as usize];

            for lx in 0..CHUNK_SIZE_X {
                for lz in 0..CHUNK_SIZE_Z {
                    let block_u8 = blocks[((lx << 10) | (ly << 4) | lz) as usize];
                    if block_u8 == 0 {
                        continue;
                    }
                    let block = BlockId::from_u8(block_u8);
                    let wx = base_x + lx;
                    let wz = base_z + lz;

                    if matches!(block, BlockId::Torch | BlockId::Door | BlockId::DoorOpen) {
                        continue;
                    }

                    let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                    let is_trans = matches!(block, BlockId::Water | BlockId::Glass);
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    if is_fluid {
                        let visible = match face {
                            Face::Top => neighbor != block,
                            Face::Bottom => wy > 0 && neighbor != block && (neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)),
                            _ => false,
                        };
                        if visible {
                            if is_trans {
                                emit_face(&mut trans_tris, wx, wy, wz, face, block, cache);
                            } else {
                                emit_face(&mut solid_tris, wx, wy, wz, face, block, cache);
                            }
                        }
                        continue;
                    }

                    let visible = neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block);
                    if visible {
                        let tex = determine_texture(block, face, wx, wy, wz, lx, lz, cache);
                        let aos = compute_face_ao(face, lx, wy, lz, cache);
                        let sky = (cache.sample_sky_light(lx + n.x, wy + n.y, lz + n.z) * 100.0) as u8;
                        let block_l = (cache.sample_block_light(wx as f32 + 0.5 + n.x as f32 * 0.5, wy as f32 + 0.5 + n.y as f32 * 0.5, wz as f32 + 0.5 + n.z as f32 * 0.5) * 100.0) as u8;

                        mask[lx as usize][lz as usize] = Some(FaceCell {
                            texture: tex,
                            ao: [(aos[0] * 100.0) as u8, (aos[1] * 100.0) as u8, (aos[2] * 100.0) as u8, (aos[3] * 100.0) as u8],
                            sky_light: sky,
                            block_light: block_l,
                            is_trans,
                        });
                    }
                }
            }

            // Greedy merge rectangles on (lx, lz)
            for lz in 0..CHUNK_SIZE_Z as usize {
                for lx in 0..CHUNK_SIZE_X as usize {
                    if let Some(cell) = mask[lx][lz] {
                        let mut w = 1;
                        while lx + w < CHUNK_SIZE_X as usize && mask[lx + w][lz] == Some(cell) {
                            w += 1;
                        }
                        let mut h = 1;
                        'outer_h: while lz + h < CHUNK_SIZE_Z as usize {
                            for dx in 0..w {
                                if mask[lx + dx][lz + h] != Some(cell) {
                                    break 'outer_h;
                                }
                            }
                            h += 1;
                        }
                        for dz in 0..h {
                            for dx in 0..w {
                                mask[lx + dx][lz + dz] = None;
                            }
                        }
                        emit_greedy_quad(
                            &mut solid_tris,
                            &mut trans_tris,
                            (base_x + lx as i32) as f32,
                            wy as f32,
                            (base_z + lz as i32) as f32,
                            w as f32,
                            h as f32,
                            face,
                            &cell,
                        );
                    }
                }
            }
        }
    }

    // 2. North & South Greedy Meshing (slicing along Z)
    for lz in 0..CHUNK_SIZE_Z {
        let wz = base_z + lz;
        for face in [Face::North, Face::South] {
            let n = face.normal();
            let mut mask = [[None; SUBCHUNK_SIZE as usize]; CHUNK_SIZE_X as usize];

            for lx in 0..CHUNK_SIZE_X {
                for dy in 0..SUBCHUNK_SIZE {
                    let ly = y_start + dy;
                    let wy = ly;
                    let block_u8 = blocks[((lx << 10) | (ly << 4) | lz) as usize];
                    if block_u8 == 0 {
                        continue;
                    }
                    let block = BlockId::from_u8(block_u8);
                    let wx = base_x + lx;

                    if matches!(block, BlockId::Torch | BlockId::Door | BlockId::DoorOpen) {
                        continue;
                    }

                    let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                    let is_trans = matches!(block, BlockId::Water | BlockId::Glass);
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    if is_fluid {
                        let neighbor_loaded = match face {
                            Face::South => lz < CHUNK_SIZE_Z - 1 || cache.south_blocks.is_some(),
                            Face::North => lz > 0 || cache.north_blocks.is_some(),
                            _ => true,
                        };
                        let visible = neighbor_loaded && if neighbor != block {
                            neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)
                        } else {
                            let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                            if above_neighbor != block {
                                let my_lvl = world.fluid_level(IVec3::new(wx, wy, wz));
                                let neighbor_lvl = world.fluid_level(IVec3::new(wx + n.x, wy, wz + n.z));
                                my_lvl > neighbor_lvl
                            } else {
                                false
                            }
                        };
                        if visible {
                            if is_trans {
                                emit_face(&mut trans_tris, wx, wy, wz, face, block, cache);
                            } else {
                                emit_face(&mut solid_tris, wx, wy, wz, face, block, cache);
                            }
                        }
                        continue;
                    }

                    let visible = neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block);
                    if visible {
                        let tex = determine_texture(block, face, wx, wy, wz, lx, lz, cache);
                        let aos = compute_face_ao(face, lx, wy, lz, cache);
                        let sky = (cache.sample_sky_light(lx + n.x, wy + n.y, lz + n.z) * 100.0) as u8;
                        let block_l = (cache.sample_block_light(wx as f32 + 0.5 + n.x as f32 * 0.5, wy as f32 + 0.5 + n.y as f32 * 0.5, wz as f32 + 0.5 + n.z as f32 * 0.5) * 100.0) as u8;

                        mask[lx as usize][dy as usize] = Some(FaceCell {
                            texture: tex,
                            ao: [(aos[0] * 100.0) as u8, (aos[1] * 100.0) as u8, (aos[2] * 100.0) as u8, (aos[3] * 100.0) as u8],
                            sky_light: sky,
                            block_light: block_l,
                            is_trans,
                        });
                    }
                }
            }

            // Greedy merge rectangles on (lx, dy)
            for dy in 0..SUBCHUNK_SIZE as usize {
                for lx in 0..CHUNK_SIZE_X as usize {
                    if let Some(cell) = mask[lx][dy] {
                        let mut w = 1;
                        while lx + w < CHUNK_SIZE_X as usize && mask[lx + w][dy] == Some(cell) {
                            w += 1;
                        }
                        let mut h = 1;
                        'outer_h: while dy + h < SUBCHUNK_SIZE as usize {
                            for dx in 0..w {
                                if mask[lx + dx][dy + h] != Some(cell) {
                                    break 'outer_h;
                                }
                            }
                            h += 1;
                        }
                        for dh in 0..h {
                            for dx in 0..w {
                                mask[lx + dx][dy + dh] = None;
                            }
                        }
                        emit_greedy_quad(
                            &mut solid_tris,
                            &mut trans_tris,
                            (base_x + lx as i32) as f32,
                            (y_start + dy as i32) as f32,
                            wz as f32,
                            w as f32,
                            h as f32,
                            face,
                            &cell,
                        );
                    }
                }
            }
        }
    }

    // 3. West & East Greedy Meshing (slicing along X)
    for lx in 0..CHUNK_SIZE_X {
        let wx = base_x + lx;
        for face in [Face::West, Face::East] {
            let n = face.normal();
            let mut mask = [[None; SUBCHUNK_SIZE as usize]; CHUNK_SIZE_Z as usize];

            for lz in 0..CHUNK_SIZE_Z {
                for dy in 0..SUBCHUNK_SIZE {
                    let ly = y_start + dy;
                    let wy = ly;
                    let block_u8 = blocks[((lx << 10) | (ly << 4) | lz) as usize];
                    if block_u8 == 0 {
                        continue;
                    }
                    let block = BlockId::from_u8(block_u8);
                    let wz = base_z + lz;

                    if matches!(block, BlockId::Torch | BlockId::Door | BlockId::DoorOpen) {
                        continue;
                    }

                    let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                    let is_trans = matches!(block, BlockId::Water | BlockId::Glass);
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    if is_fluid {
                        let neighbor_loaded = match face {
                            Face::East => lx < CHUNK_SIZE_X - 1 || cache.east_blocks.is_some(),
                            Face::West => lx > 0 || cache.west_blocks.is_some(),
                            _ => true,
                        };
                        let visible = neighbor_loaded && if neighbor != block {
                            neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)
                        } else {
                            let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                            if above_neighbor != block {
                                let my_lvl = world.fluid_level(IVec3::new(wx, wy, wz));
                                let neighbor_lvl = world.fluid_level(IVec3::new(wx + n.x, wy, wz + n.z));
                                my_lvl > neighbor_lvl
                            } else {
                                false
                            }
                        };
                        if visible {
                            if is_trans {
                                emit_face(&mut trans_tris, wx, wy, wz, face, block, cache);
                            } else {
                                emit_face(&mut solid_tris, wx, wy, wz, face, block, cache);
                            }
                        }
                        continue;
                    }

                    let visible = neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block);
                    if visible {
                        let tex = determine_texture(block, face, wx, wy, wz, lx, lz, cache);
                        let aos = compute_face_ao(face, lx, wy, lz, cache);
                        let sky = (cache.sample_sky_light(lx + n.x, wy + n.y, lz + n.z) * 100.0) as u8;
                        let block_l = (cache.sample_block_light(wx as f32 + 0.5 + n.x as f32 * 0.5, wy as f32 + 0.5 + n.y as f32 * 0.5, wz as f32 + 0.5 + n.z as f32 * 0.5) * 100.0) as u8;

                        mask[lz as usize][dy as usize] = Some(FaceCell {
                            texture: tex,
                            ao: [(aos[0] * 100.0) as u8, (aos[1] * 100.0) as u8, (aos[2] * 100.0) as u8, (aos[3] * 100.0) as u8],
                            sky_light: sky,
                            block_light: block_l,
                            is_trans,
                        });
                    }
                }
            }

            // Greedy merge rectangles on (lz, dy)
            for dy in 0..SUBCHUNK_SIZE as usize {
                for lz in 0..CHUNK_SIZE_Z as usize {
                    if let Some(cell) = mask[lz][dy] {
                        let mut w = 1;
                        while lz + w < CHUNK_SIZE_Z as usize && mask[lz + w][dy] == Some(cell) {
                            w += 1;
                        }
                        let mut h = 1;
                        'outer_h: while dy + h < SUBCHUNK_SIZE as usize {
                            for dz in 0..w {
                                if mask[lz + dz][dy + h] != Some(cell) {
                                    break 'outer_h;
                                }
                            }
                            h += 1;
                        }
                        for dh in 0..h {
                            for dz in 0..w {
                                mask[lz + dz][dy + dh] = None;
                            }
                        }
                        emit_greedy_quad(
                            &mut solid_tris,
                            &mut trans_tris,
                            wx as f32,
                            (y_start + dy as i32) as f32,
                            (base_z + lz as i32) as f32,
                            w as f32,
                            h as f32,
                            face,
                            &cell,
                        );
                    }
                }
            }
        }
    }

    // 4. Special non-merged objects (Torches and Doors)
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
                } else if block == BlockId::Door {
                    emit_door(&mut solid_tris, wx, wy, wz, false, cache);
                } else if block == BlockId::DoorOpen {
                    emit_door(&mut solid_tris, wx, wy, wz, true, cache);
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
    let mut solid_tris = Vec::with_capacity(2048);
    let mut trans_tris = Vec::with_capacity(256);
    let cache = ChunkNeighborCache::new(chunk, world);

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_internal(chunk, world, sub_y, &cache);
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
        let (biome, _) = get_biome_at(wx, wz, cache.world.seed);
        match face {
            Face::Top => match biome {
                Biome::Swamp => TextureId::GrassTopSwamp,
                Biome::Savanna | Biome::Desert => TextureId::GrassTopSavanna,
                Biome::CherryGrove => TextureId::GrassTopCherry,
                Biome::AutumnForest => TextureId::GrassTopAutumn,
                Biome::SnowyTaiga | Biome::Mountain => TextureId::GrassTopTaiga,
                _ => TextureId::GrassTop,
            },
            Face::Bottom => TextureId::Dirt,
            _ => match biome {
                Biome::Swamp => TextureId::GrassSideSwamp,
                Biome::Savanna | Biome::Desert => TextureId::GrassSideSavanna,
                Biome::CherryGrove => TextureId::GrassSideCherry,
                Biome::AutumnForest => TextureId::GrassSideAutumn,
                Biome::SnowyTaiga | Biome::Mountain => TextureId::GrassSideTaiga,
                _ => TextureId::GrassSide,
            },
        }
    } else if block == BlockId::Leaves {
        let (biome, _) = get_biome_at(wx, wz, cache.world.seed);
        match biome {
            Biome::Swamp => TextureId::LeavesSwamp,
            Biome::Savanna | Biome::Desert => TextureId::LeavesSavanna,
            Biome::CherryGrove => TextureId::CherryLeaves,
            Biome::AutumnForest => TextureId::AutumnLeaves,
            Biome::SnowyTaiga => TextureId::SnowLeaves,
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
            let lvl = cache.world.fluid_level(IVec3::new(wx, wy, wz));
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

    let s = |dx: i32, dy: i32, dz: i32| cache.solid_at(lx + dx, wy + dy, lz + dz);

    let (a, ao_a, b, ao_b, c, ao_c, d, ao_d) = match face {
        Face::Top => (
            c010, vertex_ao(s(-1,1,0), s(0,1,-1), s(-1,1,-1)),
            c011, vertex_ao(s(-1,1,0), s(0,1, 1), s(-1,1, 1)),
            c111, vertex_ao(s( 1,1,0), s(0,1, 1), s( 1,1, 1)),
            c110, vertex_ao(s( 1,1,0), s(0,1,-1), s( 1,1,-1)),
        ),
        Face::Bottom => (
            c001, vertex_ao(s(-1,-1,0), s(0,-1, 1), s(-1,-1, 1)),
            c000, vertex_ao(s(-1,-1,0), s(0,-1,-1), s(-1,-1,-1)),
            c100, vertex_ao(s( 1,-1,0), s(0,-1,-1), s( 1,-1,-1)),
            c101, vertex_ao(s( 1,-1,0), s(0,-1, 1), s( 1,-1, 1)),
        ),
        Face::North => (
            c100, vertex_ao(s( 1,0,-1), s(0,-1,-1), s( 1,-1,-1)),
            c000, vertex_ao(s(-1,0,-1), s(0,-1,-1), s(-1,-1,-1)),
            c010, vertex_ao(s(-1,0,-1), s(0, 1,-1), s(-1, 1,-1)),
            c110, vertex_ao(s( 1,0,-1), s(0, 1,-1), s( 1, 1,-1)),
        ),
        Face::South => (
            c001, vertex_ao(s(-1,0,1), s(0,-1,1), s(-1,-1,1)),
            c101, vertex_ao(s( 1,0,1), s(0,-1,1), s( 1,-1,1)),
            c111, vertex_ao(s( 1,0,1), s(0, 1,1), s( 1, 1,1)),
            c011, vertex_ao(s(-1,0,1), s(0, 1,1), s(-1, 1,1)),
        ),
        Face::East => (
            c101, vertex_ao(s(1,0, 1), s(1,-1,0), s(1,-1, 1)),
            c100, vertex_ao(s(1,0,-1), s(1,-1,0), s(1,-1,-1)),
            c110, vertex_ao(s(1,0,-1), s(1, 1,0), s(1, 1,-1)),
            c111, vertex_ao(s(1,0, 1), s(1, 1,0), s(1, 1, 1)),
        ),
        Face::West => (
            c000, vertex_ao(s(-1,0,-1), s(-1,-1,0), s(-1,-1,-1)),
            c001, vertex_ao(s(-1,0, 1), s(-1,-1,0), s(-1,-1, 1)),
            c011, vertex_ao(s(-1,0, 1), s(-1, 1,0), s(-1, 1, 1)),
            c010, vertex_ao(s(-1,0,-1), s(-1, 1,0), s(-1, 1,-1)),
        ),
    };

    let n = face.normal();
    let neighbor_lx = lx + n.x;
    let neighbor_ly = wy + n.y;
    let neighbor_lz = lz + n.z;
    let face_sky_light = cache.sample_sky_light(neighbor_lx, neighbor_ly, neighbor_lz);
    let face_block_light = if block == BlockId::Lava {
        1.0
    } else {
        cache.sample_block_light(
            wx as f32 + 0.5 + n.x as f32 * 0.5,
            wy as f32 + 0.5 + n.y as f32 * 0.5,
            wz as f32 + 0.5 + n.z as f32 * 0.5,
        )
    };

    let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade * ao_a, sky_light: face_sky_light, block_light: face_block_light };
    let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade * ao_b, sky_light: face_sky_light, block_light: face_block_light };
    let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade * ao_c, sky_light: face_sky_light, block_light: face_block_light };
    let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade * ao_d, sky_light: face_sky_light, block_light: face_block_light };

    tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
    tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
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
    fn test_greedy_meshing_merges_coplanar_quads() {
        let mut world = World::new();
        let mut chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
            dirty: false,
        };

        // Create a 4x4 flat stone floor at y=20
        for lx in 0..4 {
            for lz in 0..4 {
                chunk.set_local(lx, 20, lz, BlockId::Stone);
            }
        }

        world.chunks.insert((0, 0), chunk);
        let mesh = build_chunk_mesh(world.chunks.get(&(0, 0)).unwrap(), &world);

        // Top face of a 4x4 area with uniform AO & light:
        // Without greedy meshing, it would be 4 * 4 * 2 = 32 triangles.
        // With greedy meshing, the top face is merged into exactly 2 triangles!
        let top_faces: Vec<_> = mesh.iter().filter(|tri| {
            tri.v0.pos.y == 21.0 && tri.v1.pos.y == 21.0 && tri.v2.pos.y == 21.0
        }).collect();

        assert_eq!(top_faces.len(), 2, "Greedy meshing should merge 4x4 top faces into 2 triangles (1 quad)");
    }
}
