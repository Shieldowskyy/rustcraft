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
/// a per-face directional-light tint (baked in at mesh-build time so the
/// renderer doesn't need per-pixel lighting math).
#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: (f32, f32),
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
    pub texture: TextureId,
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
                        emit_face(&mut tris, wx as f32, wy as f32, wz as f32, face, block);
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

fn emit_face(tris: &mut Vec<Triangle>, x: f32, y: f32, z: f32, face: Face, block: BlockId) {
    let shade = face.shade();
    let tint = [shade, shade, shade];
    let texture = block.texture_id(face);

    // Cube corners relative to (x, y, z) at the block's min corner.
    // Corner naming: c[bit2=y][bit1=z][bit0=x]
    let c000 = Vec3::new(x, y, z);
    let c100 = Vec3::new(x + 1.0, y, z);
    let c010 = Vec3::new(x, y + 1.0, z);
    let c110 = Vec3::new(x + 1.0, y + 1.0, z);
    let c001 = Vec3::new(x, y, z + 1.0);
    let c101 = Vec3::new(x + 1.0, y, z + 1.0);
    let c011 = Vec3::new(x, y + 1.0, z + 1.0);
    let c111 = Vec3::new(x + 1.0, y + 1.0, z + 1.0);

    // Each face defined with consistent CCW winding when viewed from
    // outside, and UVs going (0,0) -> (1,0) -> (1,1) -> (0,1) around the
    // same quad so the texture isn't mirrored or rotated oddly.
    let (a, b, c, d) = match face {
        Face::Top => (c010, c011, c111, c110),
        Face::Bottom => (c001, c000, c100, c101),
        Face::North => (c100, c000, c010, c110), // -Z face
        Face::South => (c001, c101, c111, c011), // +Z face
        Face::East => (c101, c100, c110, c111),  // +X face
        Face::West => (c000, c001, c011, c010),  // -X face
    };

    push_quad(tris, a, b, c, d, texture, tint);
}

fn push_quad(
    tris: &mut Vec<Triangle>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    texture: TextureId,
    tint: [f32; 3],
) {
    let va = Vertex { pos: a, uv: (0.0, 1.0) };
    let vb = Vertex { pos: b, uv: (1.0, 1.0) };
    let vc = Vertex { pos: c, uv: (1.0, 0.0) };
    let vd = Vertex { pos: d, uv: (0.0, 0.0) };
    tris.push(Triangle { v0: va, v1: vb, v2: vc, texture, tint });
    tris.push(Triangle { v0: va, v1: vc, v2: vd, texture, tint });
}
