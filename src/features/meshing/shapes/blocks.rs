//! Structural & fixture geometry emitters (torches, doors, generic boxes, slabs, beds, stairs, ladders).
//! Emits non-full-cube block geometry into CPU triangle slices.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::rendering::atlas::TextureId;

use crate::features::meshing::cache::ChunkNeighborCache;
use crate::features::meshing::emit::{push_quad, push_quad_uv};
use crate::features::meshing::types::{Triangle, Vertex};

pub(in crate::features::meshing) fn emit_torch(
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

pub(in crate::features::meshing) fn emit_door(
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

pub(in crate::features::meshing) fn emit_box(
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

pub(in crate::features::meshing) fn emit_slab(
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

pub(in crate::features::meshing) fn emit_bed(
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

pub(in crate::features::meshing) fn emit_stairs(
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

pub(in crate::features::meshing) fn emit_ladder(
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

