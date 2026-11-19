//! GXM 3D box, tool/item, player/armor, remote player, and first-person main/offhand emitters.
//! Mirrors `software/held_items.rs`, `software/items_3d.rs`, and `player_parts.rs` with 1:1 proportions.

#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::core::BlockFace;
use crate::features::entities::item::{
    emit_arrow_boxes, emit_feather_boxes, emit_meat_boxes, emit_procedural_tool_boxes,
    emit_torch_boxes,
};
use crate::features::physics::STANDING_EYE_OFFSET;
use crate::features::player::ItemStack;
use crate::features::rendering::{emit_player_model_parts, PlayerBoxPart};
use crate::graphics::atlas::TextureId;
use crate::graphics::camera::Camera;
use crate::world::BlockId;

use super::entities::{entity_lit_color, hurt_tint_bytes, EntityBatch};

/// Emits a 6-face textured box specified directly by its `half` extents (matching `software/items_3d.rs`).
#[allow(clippy::too_many_arguments)]
pub(super) fn append_cube_half(
    batch: &mut EntityBatch,
    center: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    half: [f32; 3],
    top_tex: TextureId,
    bot_tex: TextureId,
    side_tex: TextureId,
    daylight: f32,
    brightness: f32,
    tint: [u8; 4],
) {
    let corner = |x: f32, y: f32, z: f32| -> Vec3 {
        center + right * (x * half[0]) + up * (y * half[1]) + forward * (z * half[2])
    };
    let faces = [
        (corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0), top_tex, 1.00),
        (corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0), bot_tex, 0.55),
        (corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0), side_tex, 0.85),
        (corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0), side_tex, 0.85),
        (corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0), side_tex, 0.70),
        (corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0), side_tex, 0.70),
    ];
    for (c0, c1, c2, c3, tex, shade) in faces {
        let color = entity_lit_color(shade, daylight, brightness);
        batch.push_textured_quad(c0, c1, c2, c3, tex, color, tint);
    }
}

/// Emits visible faces of a full-extent `PlayerBoxPart` (`part.size` is full width/height/depth).
pub(super) fn append_oriented_box(
    batch: &mut EntityBatch,
    part: &PlayerBoxPart,
    hurt_tint: f32,
    camera_pos: Vec3,
    daylight: f32,
    brightness: f32,
) {
    let half = Vec3::new(part.size.x * 0.5, part.size.y * 0.5, part.size.z * 0.5);
    let tint = hurt_tint_bytes(hurt_tint);
    let get_pt = |sx: f32, sy: f32, sz: f32| -> Vec3 {
        part.origin
            + part.right * (part.center.x + sx * half.x)
            + part.up * (part.center.y + sy * half.y)
            + part.forward * (part.center.z + sz * half.z)
    };
    let faces = [
        (get_pt(-1.0, -1.0, 1.0), get_pt(1.0, -1.0, 1.0), get_pt(1.0, 1.0, 1.0), get_pt(-1.0, 1.0, 1.0), part.tex_front, 0.85),
        (get_pt(1.0, -1.0, -1.0), get_pt(-1.0, -1.0, -1.0), get_pt(-1.0, 1.0, -1.0), get_pt(1.0, 1.0, -1.0), part.tex_back, 0.85),
        (get_pt(1.0, -1.0, 1.0), get_pt(1.0, -1.0, -1.0), get_pt(1.0, 1.0, -1.0), get_pt(1.0, 1.0, 1.0), part.tex_right, 0.70),
        (get_pt(-1.0, -1.0, -1.0), get_pt(-1.0, -1.0, 1.0), get_pt(-1.0, 1.0, 1.0), get_pt(-1.0, 1.0, -1.0), part.tex_left, 0.70),
        (get_pt(-1.0, 1.0, 1.0), get_pt(1.0, 1.0, 1.0), get_pt(1.0, 1.0, -1.0), get_pt(-1.0, 1.0, -1.0), part.tex_top, 1.00),
        (get_pt(-1.0, -1.0, -1.0), get_pt(1.0, -1.0, -1.0), get_pt(1.0, -1.0, 1.0), get_pt(-1.0, -1.0, 1.0), part.tex_bottom, 0.50),
    ];
    for (p0, p1, p2, p3, tex, shade) in faces {
        let normal = (p3 - p0).cross(p1 - p0);
        let center = (p0 + p1 + p2 + p3) * 0.25;
        if (center - camera_pos).dot(normal) >= 0.0 {
            continue;
        }
        let color = entity_lit_color(shade, daylight, brightness);
        batch.push_textured_quad(p0, p1, p2, p3, tex, color, tint);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_item_3d_oriented(
    batch: &mut EntityBatch,
    block: BlockId,
    pos: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    scale: f32,
    daylight: f32,
    brightness: f32,
) {
    let mut emit_part = |part_pos: Vec3, r: Vec3, u: Vec3, f: Vec3, half: [f32; 3], tex: TextureId| {
        append_cube_half(batch, part_pos, r, u, f, half, tex, tex, tex, daylight, brightness, [0; 4]);
    };

    if block == BlockId::Torch {
        emit_torch_boxes(pos, up, scale, |p, half, tex| emit_part(p, right, up, forward, half, tex));
    } else if block.is_tool() {
        let tilt = 0.38f32;
        let tool_up = (up * tilt.cos() + right * tilt.sin()).normalized();
        let tool_right = (right * tilt.cos() - up * tilt.sin()).normalized();
        emit_procedural_tool_boxes(block, pos, tool_right, tool_up, scale, |p, half, tex| {
            emit_part(p, tool_right, tool_up, forward, half, tex);
        });
    } else if block.is_meat() {
        let tilt = 0.32f32;
        let meat_up = (up * tilt.cos() + right * tilt.sin()).normalized();
        let meat_right = (right * tilt.cos() - up * tilt.sin()).normalized();
        emit_meat_boxes(block, pos, meat_right, meat_up, scale, |p, half, tex| {
            emit_part(p, meat_right, meat_up, forward, half, tex);
        });
    } else if block == BlockId::Feather {
        let tilt = 0.35f32;
        let f_up = (up * tilt.cos() + right * tilt.sin()).normalized();
        let f_right = (right * tilt.cos() - up * tilt.sin()).normalized();
        emit_feather_boxes(pos, f_right, f_up, scale * 1.1, |p, half, tex| {
            emit_part(p, f_right, f_up, forward, half, tex);
        });
    } else if matches!(
        block,
        BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop
            | BlockId::Sapling | BlockId::Kelp | BlockId::Seagrass
    ) {
        let half = 0.35 * scale * 0.5;
        let tex = block.texture_id(BlockFace::Top);
        let color = entity_lit_color(0.95, daylight, brightness);
        let p0 = pos + (right - forward) * half - up * half;
        let p1 = pos + (-right + forward) * half - up * half;
        let p2 = pos + (-right + forward) * half + up * half;
        let p3 = pos + (right - forward) * half + up * half;
        let q0 = pos + (right + forward) * half - up * half;
        let q1 = pos + (-right - forward) * half - up * half;
        let q2 = pos + (-right - forward) * half + up * half;
        let q3 = pos + (right + forward) * half + up * half;
        batch.push_textured_quad(p0, p1, p2, p3, tex, color, [0; 4]);
        batch.push_textured_quad(q0, q1, q2, q3, tex, color, [0; 4]);
    } else {
        let half = 0.28 * scale * 0.5;
        let top_tex = block.texture_id(BlockFace::Top);
        let bot_tex = block.texture_id(BlockFace::Bottom);
        let side_tex = block.texture_id(BlockFace::North);
        append_cube_half(batch, pos, right, up, forward, [half, half, half], top_tex, bot_tex, side_tex, daylight, brightness, [0; 4]);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_player_model(
    batch: &mut EntityBatch,
    feet_pos: Vec3,
    yaw: f32,
    pitch: f32,
    walk_phase: f32,
    swing_progress: f32,
    is_sneaking: bool,
    hurt_tint: f32,
    held_block: Option<BlockId>,
    armor: &[Option<ItemStack>; 4],
    camera_pos: Vec3,
    daylight: f32,
    brightness: f32,
) {
    let held_pose = emit_player_model_parts(
        feet_pos, yaw, pitch, walk_phase, swing_progress, is_sneaking, held_block, armor,
        |part| append_oriented_box(batch, &part, hurt_tint, camera_pos, daylight, brightness),
    );
    if let Some(held) = held_pose {
        append_item_3d_oriented(batch, held.block, held.pos, held.right, held.up, held.forward, 0.85, daylight, brightness);
    }
}

pub(super) fn append_remote_player(
    batch: &mut EntityBatch,
    player: &crate::net::RemotePlayer,
    camera_pos: Vec3,
    daylight: f32,
    brightness: f32,
) {
    let feet_pos = Vec3::new(player.pos.x, player.pos.y - STANDING_EYE_OFFSET, player.pos.z);
    let hurt_tint = if player.hurt_timer > 0.0 { 0.65 } else { 0.0 };
    append_player_model(
        batch, feet_pos, player.yaw, player.pitch, player.walk_phase, 0.0,
        player.is_sneaking, hurt_tint, None, &[None, None, None, None], camera_pos, daylight, brightness,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_hand_3d(
    batch: &mut EntityBatch,
    camera: &Camera,
    block_id: Option<u8>,
    walk_phase: f32,
    speed: f32,
    attack_animation: f32,
    daylight: f32,
    brightness: f32,
) {
    let forward = camera.forward();
    let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let movement = (speed / 8.5).clamp(0.0, 1.0);
    let swing = walk_phase.sin() * 0.08 * movement;
    let attack = (attack_animation * core::f32::consts::PI).sin();
    let swing = swing + attack * 0.12;
    let bob = (walk_phase * 2.0).cos() * 0.025 * movement - attack.abs() * 0.06;
    let hand_pos = camera.position + forward * 0.72 + right * (0.48 + swing) + up * (-0.36 + bob);

    let Some(id) = block_id else {
        let arm_pos = hand_pos - right * 0.08 - up * 0.10 + forward * 0.03;
        append_cube_half(batch, arm_pos, right, up, forward, [0.085, 0.085, 0.12], TextureId::PlayerSkin, TextureId::PlayerSkin, TextureId::PlayerSkin, daylight, brightness, [0; 4]);
        return;
    };

    let block = BlockId::from_u8(id);
    if block == BlockId::Torch {
        let sp = attack * 0.45;
        let t_up = (up * (1.0 - sp * 0.3) + forward * (0.3 + sp)).normalized();
        let t_fwd = t_up.cross(right).normalized();
        let origin = hand_pos + forward * 0.12 + up * 0.10;
        emit_torch_boxes(origin, t_up, 1.0, |p, half, tex| {
            append_cube_half(batch, p, right, t_up, t_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block.is_tool() {
        let sp = attack * 0.85;
        let t_up = (up * (1.0 - sp * 0.5) + forward * (0.85 + sp) - right * 0.20).normalized();
        let t_right = (right * 0.95 + up * 0.15).normalized();
        let t_fwd = t_up.cross(t_right).normalized();
        let origin = hand_pos + forward * (0.10 + attack * 0.15) + up * (0.12 - attack * 0.12) - right * 0.02;
        emit_procedural_tool_boxes(block, origin, t_right, t_up, 1.0, |p, half, tex| {
            append_cube_half(batch, p, t_right, t_up, t_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block.is_meat() {
        let sp = attack * 0.65;
        let m_up = (up * (1.0 - sp * 0.4) + forward * (0.5 + sp) - right * 0.15).normalized();
        let m_right = (right * 0.95 + up * 0.15).normalized();
        let m_fwd = m_up.cross(m_right).normalized();
        let origin = hand_pos + forward * (0.08 + attack * 0.10) + up * (0.08 - attack * 0.08);
        emit_meat_boxes(block, origin, m_right, m_up, 1.0, |p, half, tex| {
            append_cube_half(batch, p, m_right, m_up, m_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block == BlockId::Feather {
        let sp = attack * 0.55;
        let f_up = (up * (1.0 - sp * 0.3) + forward * (0.6 + sp) - right * 0.15).normalized();
        let f_right = (right * 0.95 + up * 0.15).normalized();
        let f_fwd = f_up.cross(f_right).normalized();
        let origin = hand_pos + forward * (0.10 + attack * 0.10) + up * (0.10 - attack * 0.08);
        emit_feather_boxes(origin, f_right, f_up, 1.0, |p, half, tex| {
            append_cube_half(batch, p, f_right, f_up, f_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block == BlockId::Arrow {
        let a_up = (up * 0.65 + forward * 0.76 - right * 0.08).normalized();
        let a_right = (right + a_up * 0.08).normalized();
        let a_fwd = a_right.cross(a_up).normalized();
        emit_arrow_boxes(hand_pos + forward * 0.10 + up * 0.08, a_up, 1.0, |p, half, tex| {
            append_cube_half(batch, p, a_right, a_up, a_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block == BlockId::Shield {
        let s_pos = hand_pos + forward * (0.10 + attack * 0.12) - right * 0.08 + up * 0.05;
        let s_right = (right * 0.95 - forward * 0.2).normalized();
        let s_fwd = s_right.cross(up).normalized();
        append_cube_half(batch, s_pos, s_right, up, s_fwd, [0.09, 0.13, 0.015], TextureId::Planks, TextureId::Planks, TextureId::Planks, daylight, brightness, [0; 4]);
        append_cube_half(batch, s_pos + s_fwd * 0.012, s_right, up, s_fwd, [0.025, 0.025, 0.01], TextureId::IronOre, TextureId::IronOre, TextureId::IronOre, daylight, brightness, [0; 4]);
    } else if block == BlockId::Bow {
        let b_pos = hand_pos + forward * (0.12 + attack * 0.08) + up * 0.05;
        let b_right = (right * 0.9 + forward * 0.2).normalized();
        let b_fwd = b_right.cross(up).normalized();
        append_cube_half(batch, b_pos, b_right, up, b_fwd, [0.015, 0.16, 0.015], TextureId::WoodSide, TextureId::WoodSide, TextureId::WoodSide, daylight, brightness, [0; 4]);
        append_cube_half(batch, b_pos - b_fwd * 0.04, b_right, up, b_fwd, [0.005, 0.14, 0.005], TextureId::WhiteWool, TextureId::WhiteWool, TextureId::WhiteWool, daylight, brightness, [0; 4]);
    } else {
        let b_pos = camera.position + forward * (0.92 + attack * 0.06) + right * (0.44 + swing * 0.4) + up * (-0.35 + bob * 0.4);
        let (by, bp) = (-0.62f32, 0.38f32);
        let bf = (forward * bp.cos() - up * bp.sin()).normalized();
        let bu = (up * bp.cos() + forward * bp.sin()).normalized();
        let br = (right * by.cos() + bf * by.sin()).normalized();
        let b_fwd = (bf * by.cos() - right * by.sin()).normalized();
        append_cube_half(
            batch, b_pos, br, bu, b_fwd, [0.095, 0.095, 0.095],
            block.texture_id(BlockFace::Top), block.texture_id(BlockFace::Bottom), block.texture_id(BlockFace::North),
            daylight, brightness, [0; 4],
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_offhand_3d(
    batch: &mut EntityBatch,
    camera: &Camera,
    block_id: Option<u8>,
    walk_phase: f32,
    speed: f32,
    is_blocking: bool,
    daylight: f32,
    brightness: f32,
) {
    let Some(id) = block_id else { return };
    let block = BlockId::from_u8(id);
    let forward = camera.forward();
    let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let movement = (speed / 8.5).clamp(0.0, 1.0);
    let swing = -walk_phase.sin() * 0.06 * movement;
    let bob = (walk_phase * 2.0).cos() * 0.02 * movement;

    if is_blocking {
        let s_pos = camera.position + forward * 0.55 - right * 0.05 + up * -0.15;
        append_cube_half(batch, s_pos, right, up, forward, [0.12, 0.16, 0.018], TextureId::Planks, TextureId::Planks, TextureId::Planks, daylight, brightness, [0; 4]);
        append_cube_half(batch, s_pos + forward * 0.014, right, up, forward, [0.035, 0.035, 0.012], TextureId::IronOre, TextureId::IronOre, TextureId::IronOre, daylight, brightness, [0; 4]);
        return;
    }

    let hand_pos = camera.position + forward * 0.65 - right * (0.42 + swing) + up * (-0.38 + bob);
    if block == BlockId::Shield {
        let s_pos = hand_pos + forward * 0.06 + right * 0.04;
        let s_right = (right * 0.95 + forward * 0.2).normalized();
        let s_fwd = s_right.cross(up).normalized();
        append_cube_half(batch, s_pos, s_right, up, s_fwd, [0.08, 0.12, 0.015], TextureId::Planks, TextureId::Planks, TextureId::Planks, daylight, brightness, [0; 4]);
        append_cube_half(batch, s_pos + s_fwd * 0.012, s_right, up, s_fwd, [0.025, 0.025, 0.01], TextureId::IronOre, TextureId::IronOre, TextureId::IronOre, daylight, brightness, [0; 4]);
    } else if block == BlockId::Arrow {
        let a_up = (up * 0.65 + forward * 0.76 + right * 0.08).normalized();
        let a_right = (right - a_up * 0.08).normalized();
        let a_fwd = a_right.cross(a_up).normalized();
        emit_arrow_boxes(hand_pos + forward * 0.08 + up * 0.08, a_up, 0.9, |p, half, tex| {
            append_cube_half(batch, p, a_right, a_up, a_fwd, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    } else if block == BlockId::Torch {
        emit_torch_boxes(hand_pos + forward * 0.08 + up * 0.08, up, 0.9, |p, half, tex| {
            append_cube_half(batch, p, right, up, forward, half, tex, tex, tex, daylight, brightness, [0; 4]);
        });
    }
}
