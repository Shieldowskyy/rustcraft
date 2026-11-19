//! Ruststone logic geometry emitters (wire dust, levers, buttons, pressure plates).
//! Adjusts visual height/pose and emitted glow based on energized state.

use crate::core::math::Vec3;
use crate::features::rendering::atlas::TextureId;

use crate::features::meshing::cache::ChunkNeighborCache;
use crate::features::meshing::emit::push_quad_uv;
use crate::features::meshing::types::Triangle;

use super::blocks::emit_box;

pub(in crate::features::meshing) fn emit_ruststone_wire(
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

pub(in crate::features::meshing) fn emit_lever(
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

pub(in crate::features::meshing) fn emit_button(
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

pub(in crate::features::meshing) fn emit_pressure_plate(
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
