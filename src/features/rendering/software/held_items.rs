//! First-person main-hand and off-hand 3D item/tool/shield rendering.
//! Zero platform `#[cfg]`.

use crate::core::BlockFace;
use crate::core::math::{Mat4, Vec3};
use crate::features::blocks::BlockId;
use crate::features::player::Camera;
use crate::features::rendering::atlas::TextureId;

use super::types::Renderer;

impl Renderer {
    pub fn draw_hand_3d(
        &mut self,
        camera: &Camera,
        view_proj: &Mat4,
        block_id: Option<u8>,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
    ) {
        self.clear_depth_buffer();

        let forward = camera.forward();
        let right = camera.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        let movement = (speed / 8.5).clamp(0.0, 1.0);
        let swing = walk_phase.sin() * 0.08 * movement;
        let attack = (attack_animation * std::f32::consts::PI).sin();
        let swing = swing + attack * 0.12;
        let bob = (walk_phase * 2.0).cos() * 0.025 * movement - attack.abs() * 0.06;
        let hand_pos = camera.position
            + forward * 0.72
            + right * (0.48 + swing)
            + up * (-0.36 + bob);

        if block_id.is_none() {
            let arm_pos = hand_pos - right * 0.08 - up * 0.10 + forward * 0.03;
            self.draw_cube_3d(
                arm_pos,
                right,
                up,
                forward,
                [0.085, 0.085, 0.12],
                TextureId::PlayerSkin,
                view_proj,
            );
        } else if let Some(id) = block_id {
            let block = BlockId::from_u8(id);
            if block == BlockId::Torch {
                let swing_pitch = attack * 0.45;
                let torch_up =
                    (up * (1.0 - swing_pitch * 0.3) + forward * (0.3 + swing_pitch)).normalized();
                let torch_right = right;
                let torch_forward = torch_up.cross(torch_right).normalized();
                let torch_origin = hand_pos + forward * 0.12 + up * 0.10;
                self.draw_torch_3d(
                    torch_origin,
                    torch_right,
                    torch_up,
                    torch_forward,
                    1.0,
                    view_proj,
                );
            } else if block.is_tool() {
                let swing_pitch = attack * 0.85;
                let tool_up = (up * (1.0 - swing_pitch * 0.5)
                    + forward * (0.85 + swing_pitch)
                    - right * 0.20)
                    .normalized();
                let tool_right = (right * 0.95 + up * 0.15).normalized();
                let tool_forward = tool_up.cross(tool_right).normalized();
                let tool_origin = hand_pos
                    + forward * (0.10 + attack * 0.15)
                    + up * (0.12 - attack * 0.12)
                    - right * 0.02;

                self.draw_procedural_tool_3d(
                    block,
                    tool_origin,
                    tool_right,
                    tool_up,
                    tool_forward,
                    1.0,
                    view_proj,
                );
            } else if block.is_meat() {
                let swing_pitch = attack * 0.65;
                let meat_up = (up * (1.0 - swing_pitch * 0.4)
                    + forward * (0.5 + swing_pitch)
                    - right * 0.15)
                    .normalized();
                let meat_right = (right * 0.95 + up * 0.15).normalized();
                let meat_forward = meat_up.cross(meat_right).normalized();
                let meat_origin =
                    hand_pos + forward * (0.08 + attack * 0.10) + up * (0.08 - attack * 0.08);

                self.draw_meat_3d(
                    block,
                    meat_origin,
                    meat_right,
                    meat_up,
                    meat_forward,
                    1.0,
                    view_proj,
                );
            } else if block == BlockId::Feather {
                let swing_pitch = attack * 0.55;
                let feather_up = (up * (1.0 - swing_pitch * 0.3)
                    + forward * (0.6 + swing_pitch)
                    - right * 0.15)
                    .normalized();
                let feather_right = (right * 0.95 + up * 0.15).normalized();
                let feather_forward = feather_up.cross(feather_right).normalized();
                let feather_origin =
                    hand_pos + forward * (0.10 + attack * 0.10) + up * (0.10 - attack * 0.08);

                self.draw_feather_3d(
                    feather_origin,
                    feather_right,
                    feather_up,
                    feather_forward,
                    1.0,
                    view_proj,
                );
            } else if block == BlockId::Arrow {
                let arrow_up = (up * 0.65 + forward * 0.76 - right * 0.08).normalized();
                let arrow_right = (right + arrow_up * 0.08).normalized();
                let arrow_forward = arrow_right.cross(arrow_up).normalized();
                let arrow_pos = hand_pos + forward * 0.10 + up * 0.08;
                self.draw_arrow_3d(
                    arrow_pos,
                    arrow_right,
                    arrow_up,
                    arrow_forward,
                    1.0,
                    view_proj,
                );
            } else if block == BlockId::Shield {
                let shield_pos =
                    hand_pos + forward * (0.10 + attack * 0.12) - right * 0.08 + up * 0.05;
                let s_right = (right * 0.95 - forward * 0.2).normalized();
                let s_up = up;
                let s_fwd = s_right.cross(s_up).normalized();
                self.draw_cube_3d(
                    shield_pos,
                    s_right,
                    s_up,
                    s_fwd,
                    [0.09, 0.13, 0.015],
                    TextureId::Planks,
                    view_proj,
                );
                self.draw_cube_3d(
                    shield_pos + s_fwd * 0.012,
                    s_right,
                    s_up,
                    s_fwd,
                    [0.025, 0.025, 0.01],
                    TextureId::IronOre,
                    view_proj,
                );
            } else if block == BlockId::Bow {
                let bow_pos = hand_pos + forward * (0.12 + attack * 0.08) + up * 0.05;
                let b_right = (right * 0.9 + forward * 0.2).normalized();
                let b_up = up;
                let b_fwd = b_right.cross(b_up).normalized();
                self.draw_cube_3d(
                    bow_pos,
                    b_right,
                    b_up,
                    b_fwd,
                    [0.015, 0.16, 0.015],
                    TextureId::WoodSide,
                    view_proj,
                );
                self.draw_cube_3d(
                    bow_pos - b_fwd * 0.04,
                    b_right,
                    b_up,
                    b_fwd,
                    [0.005, 0.14, 0.005],
                    TextureId::WhiteWool,
                    view_proj,
                );
            } else {
                let block_forward_dist = 0.92 + attack * 0.06;
                let block_pos = camera.position
                    + forward * block_forward_dist
                    + right * (0.44 + swing * 0.4)
                    + up * (-0.35 + bob * 0.4);

                let block_yaw = -0.62f32;
                let block_pitch = 0.38f32;
                let b_forward =
                    (forward * block_pitch.cos() - up * block_pitch.sin()).normalized();
                let b_up = (up * block_pitch.cos() + forward * block_pitch.sin()).normalized();
                let b_right =
                    (right * block_yaw.cos() + b_forward * block_yaw.sin()).normalized();
                let b_fwd =
                    (b_forward * block_yaw.cos() - right * block_yaw.sin()).normalized();

                let top_tex = block.texture_id(BlockFace::Top);
                let side_tex = block.texture_id(BlockFace::North);
                let bot_tex = block.texture_id(BlockFace::Bottom);
                self.draw_multitex_cube_3d(
                    block_pos,
                    b_right,
                    b_up,
                    b_fwd,
                    [0.095, 0.095, 0.095],
                    top_tex,
                    bot_tex,
                    side_tex,
                    view_proj,
                );
            }
        }
    }

    pub fn draw_offhand_3d(
        &mut self,
        camera: &Camera,
        view_proj: &Mat4,
        block_id: Option<u8>,
        walk_phase: f32,
        speed: f32,
        is_blocking: bool,
    ) {
        let Some(id) = block_id else { return };
        let block = BlockId::from_u8(id);

        let forward = camera.forward();
        let right = camera.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        let movement = (speed / 8.5).clamp(0.0, 1.0);
        let swing = -walk_phase.sin() * 0.06 * movement;
        let bob = (walk_phase * 2.0).cos() * 0.02 * movement;

        if is_blocking {
            let shield_pos = camera.position + forward * 0.55 - right * 0.05 + up * -0.15;
            self.draw_cube_3d(
                shield_pos,
                right,
                up,
                forward,
                [0.12, 0.16, 0.018],
                TextureId::Planks,
                view_proj,
            );
            self.draw_cube_3d(
                shield_pos + forward * 0.014,
                right,
                up,
                forward,
                [0.035, 0.035, 0.012],
                TextureId::IronOre,
                view_proj,
            );
        } else {
            let hand_pos = camera.position
                + forward * 0.65
                - right * (0.42 + swing)
                + up * (-0.38 + bob);
            if block == BlockId::Shield {
                let shield_pos = hand_pos + forward * 0.06 + right * 0.04;
                let s_right = (right * 0.95 + forward * 0.2).normalized();
                let s_up = up;
                let s_fwd = s_right.cross(s_up).normalized();
                self.draw_cube_3d(
                    shield_pos,
                    s_right,
                    s_up,
                    s_fwd,
                    [0.08, 0.12, 0.015],
                    TextureId::Planks,
                    view_proj,
                );
                self.draw_cube_3d(
                    shield_pos + s_fwd * 0.012,
                    s_right,
                    s_up,
                    s_fwd,
                    [0.025, 0.025, 0.01],
                    TextureId::IronOre,
                    view_proj,
                );
            } else if block == BlockId::Arrow {
                let arrow_up = (up * 0.65 + forward * 0.76 + right * 0.08).normalized();
                let arrow_right = (right - arrow_up * 0.08).normalized();
                let arrow_forward = arrow_right.cross(arrow_up).normalized();
                self.draw_arrow_3d(
                    hand_pos + forward * 0.08 + up * 0.08,
                    arrow_right,
                    arrow_up,
                    arrow_forward,
                    0.9,
                    view_proj,
                );
            } else if block == BlockId::Torch {
                self.draw_torch_3d(
                    hand_pos + forward * 0.08 + up * 0.08,
                    right,
                    up,
                    forward,
                    0.9,
                    view_proj,
                );
            }
        }
    }
}
