//! Backend-independent 3rd-person articulated player model, armor overlays, and held-item pose emitter.
//! Shared between the PC software renderer and PS Vita `libgxm` with zero platform `#[cfg]`.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::player::ItemStack;
use crate::features::rendering::atlas::TextureId;

/// Oriented box primitive emitted for a player body part or armor piece.
#[derive(Copy, Clone, Debug)]
pub struct PlayerBoxPart {
    pub origin: Vec3,
    pub center: Vec3,
    pub size: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub tex_top: TextureId,
    pub tex_bottom: TextureId,
    pub tex_front: TextureId,
    pub tex_back: TextureId,
    pub tex_left: TextureId,
    pub tex_right: TextureId,
}

impl PlayerBoxPart {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub const fn new(
        origin: Vec3,
        center: Vec3,
        size: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        tex_top: TextureId,
        tex_bottom: TextureId,
        tex_front: TextureId,
        tex_back: TextureId,
        tex_left: TextureId,
        tex_right: TextureId,
    ) -> Self {
        Self {
            origin,
            center,
            size,
            forward,
            right,
            up,
            tex_top,
            tex_bottom,
            tex_front,
            tex_back,
            tex_left,
            tex_right,
        }
    }

    #[inline]
    pub const fn uniform(
        origin: Vec3,
        center: Vec3,
        size: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        tex: TextureId,
    ) -> Self {
        Self::new(origin, center, size, forward, right, up, tex, tex, tex, tex, tex, tex)
    }
}

/// World-space hand position and basis vectors for a 3rd-person held block/item.
#[derive(Copy, Clone, Debug)]
pub struct PlayerHeldItemPose {
    pub block: BlockId,
    pub pos: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
}

/// Returns the armor overlay atlas texture for an equipped armor piece.
#[inline]
pub fn armor_material_texture(block: BlockId) -> Option<TextureId> {
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

/// Emits articulated player limbs, armor overlays, and returns the optional 3rd-person held item pose.
#[allow(clippy::too_many_arguments)]
pub fn emit_player_model_parts<F>(
    pos: Vec3,
    yaw: f32,
    pitch: f32,
    walk_phase: f32,
    swing_progress: f32,
    is_sneaking: bool,
    held_block: Option<BlockId>,
    armor: &[Option<ItemStack>; 4],
    mut emit: F,
) -> Option<PlayerHeldItemPose>
where
    F: FnMut(PlayerBoxPart),
{
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
    emit(PlayerBoxPart::new(
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
    ));

    if let Some(helmet) = &armor[0] {
        if let Some(tex) = armor_material_texture(helmet.block) {
            emit(PlayerBoxPart::uniform(
                head_origin,
                Vec3::new(0.0, 0.25, 0.0),
                Vec3::new(0.54, 0.54, 0.54),
                head_forward,
                player_right,
                head_up,
                tex,
            ));
        }
    }

    // 2. Torso
    let body_origin = pos + player_up * 0.72;
    let torso_forward =
        (player_forward * sneak_torso_pitch.cos() - player_up * sneak_torso_pitch.sin()).normalized();
    let torso_up =
        (player_up * sneak_torso_pitch.cos() + player_forward * sneak_torso_pitch.sin()).normalized();
    emit(PlayerBoxPart::new(
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
    ));

    // 3. Left arm
    let left_shoulder_origin =
        pos - player_right * 0.36 + player_up * (1.36 - sneak_y_drop) + sneak_forward * 0.5;
    let left_arm_pitch = -leg_swing;
    let left_arm_forward =
        (player_forward * left_arm_pitch.cos() + player_up * left_arm_pitch.sin()).normalized();
    let left_arm_up =
        (player_up * left_arm_pitch.cos() - player_forward * left_arm_pitch.sin()).normalized();
    emit(PlayerBoxPart::new(
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
    ));

    // 4. Right arm
    let arm_swing_angle = if swing_progress > 0.0 {
        let swing = (swing_progress * std::f32::consts::PI).sin() * 1.3;
        leg_swing * 0.3 - swing
    } else {
        leg_swing
    };
    let right_shoulder_origin =
        pos + player_right * 0.36 + player_up * (1.36 - sneak_y_drop) + sneak_forward * 0.5;
    let right_arm_forward =
        (player_forward * arm_swing_angle.cos() + player_up * arm_swing_angle.sin()).normalized();
    let right_arm_up =
        (player_up * arm_swing_angle.cos() - player_forward * arm_swing_angle.sin()).normalized();
    emit(PlayerBoxPart::new(
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
    ));

    if let Some(chest) = &armor[1] {
        if let Some(tex) = armor_material_texture(chest.block) {
            emit(PlayerBoxPart::uniform(
                body_origin,
                Vec3::new(0.0, 0.35 - sneak_y_drop * 0.5, 0.0) + sneak_forward * 0.5,
                Vec3::new(0.54, 0.72, 0.32),
                torso_forward,
                player_right,
                torso_up,
                tex,
            ));
            for &(shoulder_origin, arm_forward, arm_up) in &[
                (left_shoulder_origin, left_arm_forward, left_arm_up),
                (right_shoulder_origin, right_arm_forward, right_arm_up),
            ] {
                emit(PlayerBoxPart::uniform(
                    shoulder_origin,
                    Vec3::new(0.0, -0.16, 0.0),
                    Vec3::new(0.28, 0.36, 0.28),
                    arm_forward,
                    player_right,
                    arm_up,
                    tex,
                ));
            }
        }
    }

    let held_pose = held_block
        .filter(|&blk| blk != BlockId::Air)
        .map(|blk| PlayerHeldItemPose {
            block: blk,
            pos: right_shoulder_origin + right_arm_forward * 0.22 - right_arm_up * 0.50,
            right: player_right,
            up: right_arm_up,
            forward: right_arm_forward,
        });

    // 5. Left leg
    let left_hip_origin = pos - player_right * 0.14 + player_up * 0.72;
    let left_leg_pitch = leg_swing;
    let left_leg_forward =
        (player_forward * left_leg_pitch.cos() + player_up * left_leg_pitch.sin()).normalized();
    let left_leg_up =
        (player_up * left_leg_pitch.cos() - player_forward * left_leg_pitch.sin()).normalized();
    emit(PlayerBoxPart::new(
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
    ));

    // 6. Right leg
    let right_hip_origin = pos + player_right * 0.14 + player_up * 0.72;
    let right_leg_pitch = -leg_swing;
    let right_leg_forward =
        (player_forward * right_leg_pitch.cos() + player_up * right_leg_pitch.sin()).normalized();
    let right_leg_up =
        (player_up * right_leg_pitch.cos() - player_forward * right_leg_pitch.sin()).normalized();
    emit(PlayerBoxPart::new(
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
    ));

    if let Some(legs) = &armor[2] {
        if let Some(tex) = armor_material_texture(legs.block) {
            emit(PlayerBoxPart::uniform(
                body_origin,
                Vec3::new(0.0, 0.12 - sneak_y_drop * 0.5, 0.0) + sneak_forward * 0.5,
                Vec3::new(0.52, 0.24, 0.30),
                torso_forward,
                player_right,
                torso_up,
                tex,
            ));
            for &(hip_origin, leg_forward, leg_up) in &[
                (left_hip_origin, left_leg_forward, left_leg_up),
                (right_hip_origin, right_leg_forward, right_leg_up),
            ] {
                emit(PlayerBoxPart::uniform(
                    hip_origin,
                    Vec3::new(0.0, -0.24, 0.0),
                    Vec3::new(0.27, 0.48, 0.27),
                    leg_forward,
                    player_right,
                    leg_up,
                    tex,
                ));
            }
        }
    }

    if let Some(boots) = &armor[3] {
        if let Some(tex) = armor_material_texture(boots.block) {
            for &(hip_origin, leg_forward, leg_up) in &[
                (left_hip_origin, left_leg_forward, left_leg_up),
                (right_hip_origin, right_leg_forward, right_leg_up),
            ] {
                emit(PlayerBoxPart::uniform(
                    hip_origin,
                    Vec3::new(0.0, -0.58, 0.0),
                    Vec3::new(0.28, 0.28, 0.28),
                    leg_forward,
                    player_right,
                    leg_up,
                    tex,
                ));
            }
        }
    }

    held_pose
}
