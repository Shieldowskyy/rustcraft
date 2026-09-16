// Converts a chunk's voxel grid into a list of world-space, textured
// triangles. Uses simple face-culling (skip faces touching a solid, opaque
// neighbor) rather than full greedy meshing, to keep the code easy to
// follow. Still efficient enough for modest view distances at
// software-render scale.

use crate::block::{BlockId, Face};
use crate::math::Vec3;
use crate::texture::TextureId;
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

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let mut tris = Vec::new();

    let base_x = chunk.cx * CHUNK_SIZE_X;
    let base_z = chunk.cz * CHUNK_SIZE_Z;

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

                for face in Face::ALL {
                    let n = face.normal();
                    let neighbor = sample_block(chunk, world, lx + n.x, ly + n.y, lz + n.z, base_x, base_z);

                    let visible = if is_fluid {
                        match face {
                            Face::Top => neighbor != block,
                            Face::Bottom => neighbor != block && (neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)),
                            _ => {
                                if neighbor != block {
                                    neighbor == BlockId::Air || (neighbor.is_transparent() && neighbor != block)
                                } else {
                                    let above_neighbor = sample_block(chunk, world, lx + n.x, ly + 1, lz + n.z, base_x, base_z);
                                    if above_neighbor != block {
                                        let my_lvl = world.fluid_level(crate::math::IVec3::new(wx, wy, wz));
                                        let neighbor_lvl = world.fluid_level(crate::math::IVec3::new(wx + n.x, wy, wz + n.z));
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
                        emit_face(&mut tris, wx, wy, wz, face, block, chunk, world, base_x, base_z);
                    }
                }
            }
        }
    }

    tris
}

#[inline(always)]
fn sample_block(
    chunk: &Chunk,
    world: &World,
    lx: i32,
    ly: i32,
    lz: i32,
    base_x: i32,
    base_z: i32,
) -> BlockId {
    if (lx as u32) < CHUNK_SIZE_X as u32 && (lz as u32) < CHUNK_SIZE_Z as u32 && (ly as u32) < CHUNK_SIZE_Y as u32 {
        return chunk.get_local(lx, ly, lz);
    }
    if (ly as u32) >= CHUNK_SIZE_Y as u32 {
        return BlockId::Air;
    }
    let wx = base_x + lx;
    let wz = base_z + lz;
    world.get_block(crate::math::IVec3::new(wx, ly, wz))
}

#[inline(always)]
fn solid_at(chunk: &Chunk, world: &World, base_x: i32, base_z: i32, lx: i32, ly: i32, lz: i32) -> bool {
    sample_block(chunk, world, lx, ly, lz, base_x, base_z).is_solid()
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
    chunk: &Chunk,
    world: &World,
    base_x: i32,
    base_z: i32,
) {
    let shade = face.shade();
    let lx = wx - base_x;
    let lz = wz - base_z;

    let texture = if block == BlockId::Door {
        let below = sample_block(chunk, world, lx, wy - 1, lz, base_x, base_z);
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
        let above = sample_block(chunk, world, lx, wy + 1, lz, base_x, base_z);
        if above == block {
            y + 1.0
        } else {
            let lvl = world.fluid_level(crate::math::IVec3::new(wx, wy, wz));
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

    let s = |dx: i32, dy: i32, dz: i32| solid_at(chunk, world, base_x, base_z, lx+dx, wy+dy, lz+dz);

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
