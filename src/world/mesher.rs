// Converts a chunk's voxel grid into a list of world-space, textured
// triangles. Uses simple face-culling (skip faces touching a solid, opaque
// neighbor) rather than full greedy meshing, to keep the code easy to
// follow. Still efficient enough for modest view distances at
// software-render scale.

use crate::core::math::{IVec3, Vec3};
use crate::graphics::atlas::TextureId;
use crate::world::block::{BlockId, Face};
use crate::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: (f32, f32),
    pub ao: f32,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
    pub texture: TextureId,
}

pub struct ChunkNeighborCache<'a> {
    pub center: &'a Chunk,
    pub east: Option<&'a Chunk>,   // +X
    pub west: Option<&'a Chunk>,   // -X
    pub south: Option<&'a Chunk>,  // +Z
    pub north: Option<&'a Chunk>,  // -Z
    pub se: Option<&'a Chunk>,     // +X, +Z
    pub sw: Option<&'a Chunk>,     // -X, +Z
    pub ne: Option<&'a Chunk>,     // +X, -Z
    pub nw: Option<&'a Chunk>,     // -X, -Z
    pub world: &'a World,
    pub base_x: i32,
    pub base_z: i32,
}

impl<'a> ChunkNeighborCache<'a> {
    pub fn new(chunk: &'a Chunk, world: &'a World) -> Self {
        let cx = chunk.cx;
        let cz = chunk.cz;
        Self {
            center: chunk,
            east: world.chunks.get(&(cx + 1, cz)),
            west: world.chunks.get(&(cx - 1, cz)),
            south: world.chunks.get(&(cx, cz + 1)),
            north: world.chunks.get(&(cx, cz - 1)),
            se: world.chunks.get(&(cx + 1, cz + 1)),
            sw: world.chunks.get(&(cx - 1, cz + 1)),
            ne: world.chunks.get(&(cx + 1, cz - 1)),
            nw: world.chunks.get(&(cx - 1, cz - 1)),
            world,
            base_x: chunk.cx * CHUNK_SIZE_X,
            base_z: chunk.cz * CHUNK_SIZE_Z,
        }
    }

    #[inline(always)]
    pub fn sample_block(&self, lx: i32, ly: i32, lz: i32) -> BlockId {
        if (ly as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        if (lx as u32) < CHUNK_SIZE_X as u32 && (lz as u32) < CHUNK_SIZE_Z as u32 {
            return self.center.get_local(lx, ly, lz);
        }
        if lx >= CHUNK_SIZE_X {
            if lz >= CHUNK_SIZE_Z {
                self.se.map(|c| c.get_local(lx - CHUNK_SIZE_X, ly, lz - CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
            } else if lz < 0 {
                self.ne.map(|c| c.get_local(lx - CHUNK_SIZE_X, ly, lz + CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
            } else {
                self.east.map(|c| c.get_local(lx - CHUNK_SIZE_X, ly, lz)).unwrap_or(BlockId::Air)
            }
        } else if lx < 0 {
            if lz >= CHUNK_SIZE_Z {
                self.sw.map(|c| c.get_local(lx + CHUNK_SIZE_X, ly, lz - CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
            } else if lz < 0 {
                self.nw.map(|c| c.get_local(lx + CHUNK_SIZE_X, ly, lz + CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
            } else {
                self.west.map(|c| c.get_local(lx + CHUNK_SIZE_X, ly, lz)).unwrap_or(BlockId::Air)
            }
        } else if lz >= CHUNK_SIZE_Z {
            self.south.map(|c| c.get_local(lx, ly, lz - CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
        } else if lz < 0 {
            self.north.map(|c| c.get_local(lx, ly, lz + CHUNK_SIZE_Z)).unwrap_or(BlockId::Air)
        } else {
            BlockId::Air
        }
    }

    #[inline(always)]
    pub fn solid_at(&self, lx: i32, ly: i32, lz: i32) -> bool {
        self.sample_block(lx, ly, lz).is_solid()
    }
}

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let mut solid_tris = Vec::new();
    let mut trans_tris = Vec::new();
    let cache = ChunkNeighborCache::new(chunk, world);

    let base_x = cache.base_x;
    let base_z = cache.base_z;

    for lx in 0..CHUNK_SIZE_X {
        for ly in 0..CHUNK_SIZE_Y {
            for lz in 0..CHUNK_SIZE_Z {
                let block = chunk.get_local(lx, ly, lz);
                if block == BlockId::Air {
                    continue;
                }

                let wx = base_x + lx;
                let wy = ly;
                let wz = base_z + lz;
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

    let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade * ao_a };
    let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade * ao_b };
    let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade * ao_c };
    let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade * ao_d };

    if (ao_a + ao_c) > (ao_b + ao_d) {
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
    } else {
        tris.push(Triangle { v0: va, v1: vb, v2: vd, texture });
        tris.push(Triangle { v0: vb, v1: vc, v2: vd, texture });
    }
}
