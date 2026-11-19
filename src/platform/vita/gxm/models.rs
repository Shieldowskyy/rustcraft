//! GXM 3D oriented box, procedural tool/item, and first-person held-item geometry emitters.
//! Shared between world entities (`entities.rs`) and first-person hand rendering.

#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::graphics::atlas::TextureId;
use crate::graphics::camera::Camera;
use crate::world::BlockId;

use super::entities::append_quad;
use super::types::GxmVertex;

pub(super) fn append_cube_oriented(
    pos: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    size: [f32; 3],
    tex_top: TextureId,
    tex_bottom: TextureId,
    tex_side: TextureId,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let half_x = size[0] * 0.5;
    let half_y = size[1] * 0.5;
    let half_z = size[2] * 0.5;

    let corner = |x: f32, y: f32, z: f32| -> Vec3 {
        pos + right * (x * half_x) + up * (y * half_y) + forward * (z * half_z)
    };

    let c000 = corner(-1.0, -1.0, -1.0);
    let c100 = corner( 1.0, -1.0, -1.0);
    let c010 = corner(-1.0,  1.0, -1.0);
    let c110 = corner( 1.0,  1.0, -1.0);
    let c001 = corner(-1.0, -1.0,  1.0);
    let c101 = corner( 1.0, -1.0,  1.0);
    let c011 = corner(-1.0,  1.0,  1.0);
    let c111 = corner( 1.0,  1.0,  1.0);

    // Top (+Y)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c010, c110, c111, c011, tex_top, 1.00);
    // Bottom (-Y)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c001, c101, c100, c000, tex_bottom, 0.50);
    // Front (+Z)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c001, c011, c111, c101, tex_side, 0.85);
    // Back (-Z)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c100, c110, c010, c000, tex_side, 0.85);
    // Right (+X)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c101, c111, c110, c100, tex_side, 0.70);
    // Left (-X)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c000, c010, c011, c001, tex_side, 0.70);
}

pub(super) fn append_procedural_tool_3d(
    block: BlockId,
    pos: Vec3,
    tool_right: Vec3,
    tool_up: Vec3,
    tool_forward: Vec3,
    scale: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    crate::gameplay::item::emit_procedural_tool_boxes(
        block,
        pos,
        tool_right,
        tool_up,
        scale,
        |part_pos, sz, tex| {
            append_cube_oriented(
                part_pos,
                tool_right,
                tool_up,
                tool_forward,
                sz,
                tex,
                tex,
                tex,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        },
    );
}

pub(super) fn append_meat_3d(
    block: BlockId,
    pos: Vec3,
    meat_right: Vec3,
    meat_up: Vec3,
    meat_forward: Vec3,
    scale: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    crate::gameplay::item::emit_meat_boxes(
        block,
        pos,
        meat_right,
        meat_up,
        scale,
        |part_pos, sz, tex| {
            append_cube_oriented(
                part_pos,
                meat_right,
                meat_up,
                meat_forward,
                sz,
                tex,
                tex,
                tex,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        },
    );
}

pub(super) fn append_feather_3d(
    pos: Vec3,
    feather_right: Vec3,
    feather_up: Vec3,
    feather_forward: Vec3,
    scale: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    crate::gameplay::item::emit_feather_boxes(
        pos,
        feather_right,
        feather_up,
        scale,
        |part_pos, sz, tex| {
            append_cube_oriented(
                part_pos,
                feather_right,
                feather_up,
                feather_forward,
                sz,
                tex,
                tex,
                tex,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        },
    );
}

pub(super) fn append_hand_3d(
    camera: &Camera,
    block_id: Option<u8>,
    walk_phase: f32,
    speed: f32,
    attack_animation: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let forward = camera.forward();
    let right = camera.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let movement = (speed / 8.5).clamp(0.0, 1.0);
    let swing = walk_phase.sin() * 0.10 * movement;
    let attack = (attack_animation * core::f32::consts::PI).sin();
    let swing = swing + attack * 0.16;
    let bob = (walk_phase * 2.0).cos() * 0.035 * movement - attack.abs() * 0.08;
    let hand_pos = camera.position
        + forward * 0.68
        + right * (0.53 + swing)
        + up * (-0.40 + bob);
    let arm_pos = hand_pos - right * 0.10 - up * 0.13;

    // Always draw arm (sleeve/skin)
    append_cube_oriented(
        arm_pos,
        right,
        up,
        forward,
        [0.085, 0.085, 0.12],
        TextureId::Dirt,
        TextureId::Dirt,
        TextureId::Dirt,
        vb,
        ib,
        v_count,
        i_count,
        max_verts,
        max_idx,
    );

    // Draw held tool or block
    if let Some(id) = block_id {
        let block = BlockId::from_u8(id);
        if block.is_tool() {
            let swing_pitch = attack * 0.85;
            let tool_up = (up * (1.0 - swing_pitch * 0.5) + forward * (0.85 + swing_pitch) - right * 0.20).normalized();
            let tool_right = (right * 0.95 + up * 0.15).normalized();
            let tool_forward = tool_up.cross(tool_right).normalized();
            let tool_origin = hand_pos + forward * (0.10 + attack * 0.15) + up * (0.12 - attack * 0.12) - right * 0.02;

            append_procedural_tool_3d(
                block,
                tool_origin,
                tool_right,
                tool_up,
                tool_forward,
                1.0,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        } else if block.is_meat() {
            let swing_pitch = attack * 0.65;
            let meat_up = (up * (1.0 - swing_pitch * 0.4) + forward * (0.5 + swing_pitch) - right * 0.15).normalized();
            let meat_right = (right * 0.95 + up * 0.15).normalized();
            let meat_forward = meat_up.cross(meat_right).normalized();
            let meat_origin = hand_pos + forward * (0.08 + attack * 0.10) + up * (0.08 - attack * 0.08);

            append_meat_3d(
                block,
                meat_origin,
                meat_right,
                meat_up,
                meat_forward,
                1.0,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        } else if block == BlockId::Feather {
            let swing_pitch = attack * 0.55;
            let feather_up = (up * (1.0 - swing_pitch * 0.3) + forward * (0.6 + swing_pitch) - right * 0.15).normalized();
            let feather_right = (right * 0.95 + up * 0.15).normalized();
            let feather_forward = feather_up.cross(feather_right).normalized();
            let feather_origin = hand_pos + forward * (0.10 + attack * 0.10) + up * (0.10 - attack * 0.08);

            append_feather_3d(
                feather_origin,
                feather_right,
                feather_up,
                feather_forward,
                1.0,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        } else {
            let top_tex = block.texture_id(crate::world::block::Face::Top);
            let side_tex = block.texture_id(crate::world::block::Face::North);
            let bot_tex = block.texture_id(crate::world::block::Face::Bottom);
            append_cube_oriented(
                hand_pos,
                right,
                up,
                forward,
                [0.13, 0.13, 0.13],
                top_tex,
                bot_tex,
                side_tex,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        }
    }
}
