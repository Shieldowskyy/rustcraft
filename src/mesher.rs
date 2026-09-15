// Converts a chunk's voxel grid into a list of world-space, textured
// triangles. Uses simple face-culling (skip faces touching a solid, opaque
// neighbor) rather than full greedy meshing, to keep the code easy to
// follow. Still efficient enough for modest view distances at
// software-render scale.

use crate::block::{BlockId, Face};
use crate::math::Vec3;
use crate::texture::TextureId;
use crate::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

/// A single world-space triangle vertex: position, texture coordinate, and
/// a per-vertex AO factor baked at mesh-build time so the renderer can
/// interpolate it smoothly without per-pixel lighting.
#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: (f32, f32),
    /// Per-vertex AO * directional shade factor [0.5..1.0]
    pub ao: f32,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
    pub texture: TextureId,
    #[allow(dead_code)]
    pub tint: [f32; 3],
}

/// Builds a mesh (list of triangles, in world space) for a single chunk.
/// `world` is used to sample neighboring chunks so faces on chunk borders
/// are culled correctly against adjacent chunk contents.
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

                for face in Face::ALL {
                    let n = face.normal();
                    let neighbor = sample_block(chunk, world, lx + n.x, ly + n.y, lz + n.z, base_x, base_z);

                    let visible = if block == BlockId::Water {
                        neighbor == BlockId::Air
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

/// Samples a block at local coordinates that may spill into a neighboring
/// chunk (used only for X/Z since chunks aren't stacked vertically).
fn sample_block(
    chunk: &Chunk,
    world: &World,
    lx: i32,
    ly: i32,
    lz: i32,
    base_x: i32,
    base_z: i32,
) -> BlockId {
    if lx >= 0 && lx < CHUNK_SIZE_X && lz >= 0 && lz < CHUNK_SIZE_Z {
        return chunk.get_local(lx, ly, lz);
    }
    if ly < 0 || ly >= CHUNK_SIZE_Y {
        return BlockId::Air;
    }
    let wx = base_x + lx;
    let wz = base_z + lz;
    world.get_block(crate::math::IVec3::new(wx, ly, wz))
}

/// Returns true if the block at the given local offset is solid (used for AO).
fn solid_at(chunk: &Chunk, world: &World, base_x: i32, base_z: i32, lx: i32, ly: i32, lz: i32) -> bool {
    sample_block(chunk, world, lx, ly, lz, base_x, base_z).is_solid()
}

/// Classic Minecraft per-vertex AO from the 3 neighboring blocks at a corner.
fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let count = side1 as u8 + side2 as u8 + (corner && !(side1 && side2)) as u8;
    match count {
        0 => 1.00,
        1 => 0.80,
        2 => 0.65,
        _ => 0.50,
    }
}

#[allow(clippy::too_many_arguments)]
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
    let texture = block.texture_id(face);

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    // Cube corners
    let c000 = Vec3::new(x,       y,       z);
    let c100 = Vec3::new(x + 1.0, y,       z);
    let c010 = Vec3::new(x,       y + 1.0, z);
    let c110 = Vec3::new(x + 1.0, y + 1.0, z);
    let c001 = Vec3::new(x,       y,       z + 1.0);
    let c101 = Vec3::new(x + 1.0, y,       z + 1.0);
    let c011 = Vec3::new(x,       y + 1.0, z + 1.0);
    let c111 = Vec3::new(x + 1.0, y + 1.0, z + 1.0);

    let lx = wx - base_x;
    let lz = wz - base_z;
    let ly = wy;
    let s = |dx: i32, dy: i32, dz: i32| solid_at(chunk, world, base_x, base_z, lx+dx, ly+dy, lz+dz);

    // 4 corners of the face with AO factor each (winding matches original)
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

    // Flip the diagonal based on AO sum to avoid gradient artifacts
    if (ao_a + ao_c) > (ao_b + ao_d) {
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture, tint: [shade * ao_a; 3] });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture, tint: [shade * ao_a; 3] });
    } else {
        tris.push(Triangle { v0: va, v1: vb, v2: vd, texture, tint: [shade * ao_b; 3] });
        tris.push(Triangle { v0: vb, v1: vc, v2: vd, texture, tint: [shade * ao_b; 3] });
    }
}
