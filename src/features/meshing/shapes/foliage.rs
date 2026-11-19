//! Crossed-quad plant geometry and waterlogged face emitters (crops, saplings, kelp, seagrass).
//! Handles submerged transparent water boundaries alongside underwater flora.

use crate::core::math::Vec3;
use crate::features::blocks::{BlockId, Face};
use crate::features::rendering::atlas::TextureId;

use crate::features::meshing::cache::ChunkNeighborCache;
use crate::features::meshing::emit::{emit_face, push_quad};
use crate::features::meshing::types::Triangle;

pub(in crate::features::meshing) fn emit_cross(
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
pub(in crate::features::meshing) fn emit_waterlogged_faces(
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

