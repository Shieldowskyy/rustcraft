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

        // 1. Wysokość kolumn nieprzezroczystych (Opaque Heightmap) i liści (Leaf Heightmap)
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

        // 2. Źródła światła tylko z zarejestrowanych pochodni w pobliżu (O(liczba_pochodni) zamiast 160k pętli!)
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
                    0.88 // Delikatny, naturalny cień pod koroną drzewa
                } else {
                    1.0  // Pełne, bezpośrednie światło słoneczne
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

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let mut solid_tris = Vec::with_capacity(3072);
    let mut trans_tris = Vec::with_capacity(512);
    let cache = ChunkNeighborCache::new(chunk, world);

    let base_x = cache.base_x;
    let base_z = cache.base_z;
    let blocks = chunk.blocks.as_slice();

    for lx in 0..CHUNK_SIZE_X {
        for ly in 0..CHUNK_SIZE_Y {
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
                    emit_torch(&mut solid_tris, wx, wy, wz, &cache);
                    continue;
                }

                let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                let is_trans = matches!(block, BlockId::Water | BlockId::Glass);

                for face in Face::ALL {
                    let n = face.normal();
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    let visible = if is_fluid {
                        match face {
                            Face::Top => neighbor != block,
                            Face::Bottom => neighbor != block && (neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)),
                            _ => {
                                if neighbor != block {
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
                                }
                            }
                        }
                    } else {
                        neighbor == BlockId::Air
                        || (neighbor.is_transparent() && neighbor != block)
                    };

                    if visible {
                        if is_trans {
                            emit_face(&mut trans_tris, wx, wy, wz, face, block, &cache);
                        } else {
                            emit_face(&mut solid_tris, wx, wy, wz, face, block, &cache);
                        }
                    }
                }
            }
        }
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
    quad(tris, c001, c000, c100, c101, tex, 0.70);
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
