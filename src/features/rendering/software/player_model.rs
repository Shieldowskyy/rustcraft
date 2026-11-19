//! Articulated 3rd-person player model, armor overlays, remote player, and floating nametag rendering.
//! Zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::blocks::BlockId;
use crate::features::physics::STANDING_EYE_OFFSET;
use crate::features::player::{Camera, ItemStack};
use crate::features::rendering::atlas::TextureId;
use crate::features::rendering::font;

use super::types::Renderer;

impl Renderer {
    fn armor_material_texture(block: BlockId) -> Option<TextureId> {
        match block {
            BlockId::LeatherHelmet
            | BlockId::LeatherChestplate
            | BlockId::LeatherLeggings
            | BlockId::LeatherBoots => Some(TextureId::LeatherArmor),
            BlockId::IronHelmet
            | BlockId::IronChestplate
            | BlockId::IronLeggings
            | BlockId::IronBoots => Some(TextureId::IronArmor),
            BlockId::GoldHelmet
            | BlockId::GoldChestplate
            | BlockId::GoldLeggings
            | BlockId::GoldBoots => Some(TextureId::GoldArmor),
            BlockId::DiamondHelmet
            | BlockId::DiamondChestplate
            | BlockId::DiamondLeggings
            | BlockId::DiamondBoots => Some(TextureId::DiamondArmor),
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_player_model(
        &mut self,
        pos: Vec3,
        yaw: f32,
        pitch: f32,
        walk_phase: f32,
        swing_progress: f32,
        is_sneaking: bool,
        hurt_tint: f32,
        nickname: Option<&str>,
        held_block: Option<BlockId>,
        armor: &[Option<ItemStack>; 4],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let camera_pos = camera.position;
        let player_forward = Vec3::new(yaw.cos(), 0.0, yaw.sin()).normalized();
        let player_right = Vec3::new(-yaw.sin(), 0.0, yaw.cos()).normalized();
        let player_up = Vec3::new(0.0, 1.0, 0.0);

        let leg_swing = walk_phase.sin() * 0.65;
        let sneak_y_drop: f32 = if is_sneaking { 0.22 } else { 0.0 };
        let sneak_torso_pitch: f32 = if is_sneaking { 0.28 } else { 0.0 };
        let sneak_forward = if is_sneaking {
            player_forward * 0.16
        } else {
            Vec3::zero()
        };

        // 1. Head
        let head_origin = pos + player_up * (1.44 - sneak_y_drop) + sneak_forward;
        let head_forward = (player_forward * pitch.cos() + player_up * pitch.sin()).normalized();
        let head_up = (player_up * pitch.cos() - player_forward * pitch.sin()).normalized();
        self.draw_oriented_box(
            head_origin,
            Vec3::new(0.0, 0.24, 0.0),
            Vec3::new(0.48, 0.48, 0.48),
            head_forward,
            player_right,
            head_up,
            TextureId::PlayerHair,
            TextureId::PlayerSkin,
            TextureId::PlayerFace,
            TextureId::PlayerHair,
            TextureId::PlayerHair,
            TextureId::PlayerHair,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        if let Some(helmet) = &armor[0] {
            if let Some(tex) = Self::armor_material_texture(helmet.block) {
                self.draw_uniform_oriented_box(
                    head_origin,
                    Vec3::new(0.0, 0.25, 0.0),
                    Vec3::new(0.54, 0.54, 0.54),
                    head_forward,
                    player_right,
                    head_up,
                    tex,
                    hurt_tint,
                    camera_pos,
                    view_proj,
                );
            }
        }

        // 2. Torso
        let body_origin = pos + player_up * 0.72;
        let torso_forward = (player_forward * sneak_torso_pitch.cos()
            - player_up * sneak_torso_pitch.sin())
        .normalized();
        let torso_up = (player_up * sneak_torso_pitch.cos()
            + player_forward * sneak_torso_pitch.sin())
        .normalized();
        self.draw_oriented_box(
            body_origin,
            Vec3::new(0.0, 0.35 - sneak_y_drop * 0.5, 0.0) + sneak_forward * 0.5,
            Vec3::new(0.48, 0.70, 0.26),
            torso_forward,
            player_right,
            torso_up,
            TextureId::PlayerShirt,
            TextureId::PlayerPants,
            TextureId::PlayerShirtFront,
            TextureId::PlayerShirt,
            TextureId::PlayerShirt,
            TextureId::PlayerShirt,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        // 3. Left arm
        let left_shoulder_origin =
            pos - player_right * 0.36 + player_up * (1.36 - sneak_y_drop) + sneak_forward * 0.5;
        let left_arm_pitch = -leg_swing;
        let left_arm_forward =
            (player_forward * left_arm_pitch.cos() + player_up * left_arm_pitch.sin()).normalized();
        let left_arm_up =
            (player_up * left_arm_pitch.cos() - player_forward * left_arm_pitch.sin()).normalized();
        self.draw_oriented_box(
            left_shoulder_origin,
            Vec3::new(0.0, -0.32, 0.0),
            Vec3::new(0.24, 0.64, 0.24),
            left_arm_forward,
            player_right,
            left_arm_up,
            TextureId::PlayerShirt,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        // 4. Right arm
        let arm_swing_angle = if swing_progress > 0.0 {
            let swing = (swing_progress * std::f32::consts::PI).sin() * 1.3;
            leg_swing * 0.3 - swing
        } else {
            leg_swing
        };
        let right_shoulder_origin =
            pos + player_right * 0.36 + player_up * (1.36 - sneak_y_drop) + sneak_forward * 0.5;
        let right_arm_forward = (player_forward * arm_swing_angle.cos()
            + player_up * arm_swing_angle.sin())
        .normalized();
        let right_arm_up = (player_up * arm_swing_angle.cos()
            - player_forward * arm_swing_angle.sin())
        .normalized();
        self.draw_oriented_box(
            right_shoulder_origin,
            Vec3::new(0.0, -0.32, 0.0),
            Vec3::new(0.24, 0.64, 0.24),
            right_arm_forward,
            player_right,
            right_arm_up,
            TextureId::PlayerShirt,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            TextureId::PlayerSkin,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        if let Some(chest) = &armor[1] {
            if let Some(tex) = Self::armor_material_texture(chest.block) {
                self.draw_uniform_oriented_box(
                    body_origin,
                    Vec3::new(0.0, 0.35 - sneak_y_drop * 0.5, 0.0) + sneak_forward * 0.5,
                    Vec3::new(0.54, 0.72, 0.32),
                    torso_forward,
                    player_right,
                    torso_up,
                    tex,
                    hurt_tint,
                    camera_pos,
                    view_proj,
                );
                for &(shoulder_origin, arm_forward, arm_up) in &[
                    (left_shoulder_origin, left_arm_forward, left_arm_up),
                    (right_shoulder_origin, right_arm_forward, right_arm_up),
                ] {
                    self.draw_uniform_oriented_box(
                        shoulder_origin,
                        Vec3::new(0.0, -0.16, 0.0),
                        Vec3::new(0.28, 0.36, 0.28),
                        arm_forward,
                        player_right,
                        arm_up,
                        tex,
                        hurt_tint,
                        camera_pos,
                        view_proj,
                    );
                }
            }
        }

        if let Some(blk) = held_block {
            if blk != BlockId::Air {
                let hand_pos =
                    right_shoulder_origin + right_arm_forward * 0.22 - right_arm_up * 0.50;
                self.draw_item_3d_oriented(
                    blk,
                    hand_pos,
                    player_right,
                    right_arm_up,
                    right_arm_forward,
                    0.85,
                    view_proj,
                );
            }
        }

        // 5. Left leg
        let left_hip_origin = pos - player_right * 0.14 + player_up * 0.72;
        let left_leg_pitch = leg_swing;
        let left_leg_forward =
            (player_forward * left_leg_pitch.cos() + player_up * left_leg_pitch.sin()).normalized();
        let left_leg_up =
            (player_up * left_leg_pitch.cos() - player_forward * left_leg_pitch.sin()).normalized();
        self.draw_oriented_box(
            left_hip_origin,
            Vec3::new(0.0, -0.36, 0.0),
            Vec3::new(0.24, 0.72, 0.24),
            left_leg_forward,
            player_right,
            left_leg_up,
            TextureId::PlayerPants,
            TextureId::PlayerShoes,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        // 6. Right leg
        let right_hip_origin = pos + player_right * 0.14 + player_up * 0.72;
        let right_leg_pitch = -leg_swing;
        let right_leg_forward = (player_forward * right_leg_pitch.cos()
            + player_up * right_leg_pitch.sin())
        .normalized();
        let right_leg_up = (player_up * right_leg_pitch.cos()
            - player_forward * right_leg_pitch.sin())
        .normalized();
        self.draw_oriented_box(
            right_hip_origin,
            Vec3::new(0.0, -0.36, 0.0),
            Vec3::new(0.24, 0.72, 0.24),
            right_leg_forward,
            player_right,
            right_leg_up,
            TextureId::PlayerPants,
            TextureId::PlayerShoes,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            TextureId::PlayerPants,
            hurt_tint,
            camera_pos,
            view_proj,
        );

        if let Some(legs) = &armor[2] {
            if let Some(tex) = Self::armor_material_texture(legs.block) {
                self.draw_uniform_oriented_box(
                    body_origin,
                    Vec3::new(0.0, 0.12 - sneak_y_drop * 0.5, 0.0) + sneak_forward * 0.5,
                    Vec3::new(0.52, 0.24, 0.30),
                    torso_forward,
                    player_right,
                    torso_up,
                    tex,
                    hurt_tint,
                    camera_pos,
                    view_proj,
                );
                for &(hip_origin, leg_forward, leg_up) in &[
                    (left_hip_origin, left_leg_forward, left_leg_up),
                    (right_hip_origin, right_leg_forward, right_leg_up),
                ] {
                    self.draw_uniform_oriented_box(
                        hip_origin,
                        Vec3::new(0.0, -0.24, 0.0),
                        Vec3::new(0.27, 0.48, 0.27),
                        leg_forward,
                        player_right,
                        leg_up,
                        tex,
                        hurt_tint,
                        camera_pos,
                        view_proj,
                    );
                }
            }
        }

        if let Some(boots) = &armor[3] {
            if let Some(tex) = Self::armor_material_texture(boots.block) {
                for &(hip_origin, leg_forward, leg_up) in &[
                    (left_hip_origin, left_leg_forward, left_leg_up),
                    (right_hip_origin, right_leg_forward, right_leg_up),
                ] {
                    self.draw_uniform_oriented_box(
                        hip_origin,
                        Vec3::new(0.0, -0.58, 0.0),
                        Vec3::new(0.28, 0.28, 0.28),
                        leg_forward,
                        player_right,
                        leg_up,
                        tex,
                        hurt_tint,
                        camera_pos,
                        view_proj,
                    );
                }
            }
        }

        // 7. Floating nametag above player head
        if let Some(nick) = nickname {
            let tag_pos = pos + Vec3::new(0.0, 2.22, 0.0);
            let clip = view_proj.transform_point(tag_pos);
            if clip[3] > 0.2 {
                let ndc_x = clip[0] / clip[3];
                let ndc_y = clip[1] / clip[3];
                if (-1.1..=1.1).contains(&ndc_x) && (-1.1..=1.1).contains(&ndc_y) {
                    let sx = ((ndc_x * 0.5 + 0.5) * self.width as f32) as i32;
                    let sy = ((1.0 - (ndc_y * 0.5 + 0.5)) * self.height as f32) as i32;
                    let ntw = font::measure_text(nick, 1, 1);
                    crate::ui::draw::fill_rect_alpha(
                        &mut self.color_buffer,
                        self.width,
                        self.height,
                        sx - ntw / 2 - 4,
                        sy - 10,
                        ntw + 8,
                        14,
                        crate::ui::draw::pack_color(0.0, 0.0, 0.0),
                        0.6,
                    );
                    crate::ui::draw::draw_text_colored_shadowed(
                        &mut self.color_buffer,
                        self.width,
                        self.height,
                        nick,
                        sx - ntw / 2,
                        sy - 8,
                        1,
                        1,
                        crate::ui::draw::pack_color(1.0, 1.0, 1.0),
                    );
                }
            }
        }
    }

    pub fn draw_remote_player(
        &mut self,
        player: &crate::net::RemotePlayer,
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let feet_pos = Vec3::new(
            player.pos.x,
            player.pos.y - STANDING_EYE_OFFSET,
            player.pos.z,
        );
        let hurt_tint = if player.hurt_timer > 0.0 { 0.65 } else { 0.0 };
        self.draw_player_model(
            feet_pos,
            player.yaw,
            player.pitch,
            player.walk_phase,
            0.0,
            player.is_sneaking,
            hurt_tint,
            Some(&player.nickname),
            None,
            &[None, None, None, None],
            camera,
            view_proj,
        );
    }
}
