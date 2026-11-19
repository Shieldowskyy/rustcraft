//! Procedural 3D box geometry builders for passive livestock and aquatic mob species.

use crate::core::math::Vec3;
use crate::core::BlockFace;
use crate::features::blocks::BlockId;
use crate::features::rendering::atlas::TextureId;

use super::models::MobBox;
use super::state::Mob;

pub(super) fn emit_pig<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.45 * scale, 0.0),
        Vec3::new(0.65 * scale, 0.45 * scale, 0.90 * scale),
        yaw, 0.0, roll_angle, base_pos, TextureId::PigSkin,
    ));

    let head_pos = base_pos + Vec3::new(yaw.sin() * 0.48 * scale, 0.52 * scale, yaw.cos() * 0.48 * scale);
    let head_yaw = yaw + mob.head_yaw_offset;
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.48 * scale, 0.48 * scale, 0.48 * scale),
        head_yaw, 0.0, roll_angle, head_pos, TextureId::PigSkin, TextureId::PigFace,
    ));

    let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.28 * scale, -0.06 * scale, head_yaw.cos() * 0.28 * scale);
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.24 * scale, 0.18 * scale, 0.12 * scale),
        head_yaw, 0.0, roll_angle, snout_pos, TextureId::PigSkin, TextureId::PigSnout,
    ));

    let leg_offsets = [
        (-0.22 * scale, -0.28 * scale, leg_swing),
        (0.22 * scale, -0.28 * scale, -leg_swing),
        (-0.22 * scale, 0.28 * scale, -leg_swing),
        (0.22 * scale, 0.28 * scale, leg_swing),
    ];
    for &(lx, lz, swing) in &leg_offsets {
        let rotated_x = lx * yaw.cos() + lz * yaw.sin();
        let rotated_z = -lx * yaw.sin() + lz * yaw.cos();
        let leg_origin = base_pos + Vec3::new(rotated_x, 0.30 * scale, rotated_z);
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.15 * scale, 0.0),
            Vec3::new(0.20 * scale, 0.30 * scale, 0.20 * scale),
            yaw, swing, roll_angle, leg_origin, TextureId::PigSkin,
        ));
    }
}

pub(super) fn emit_cow<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.78 * scale, 0.0),
        Vec3::new(0.75 * scale, 0.65 * scale, 1.15 * scale),
        yaw, 0.0, roll_angle, base_pos, TextureId::CowSkin,
    ));

    if !mob.is_baby {
        let udder_pos = base_pos + Vec3::new(0.0, 0.44 * scale, 0.0);
        emit(MobBox::uniform(
            Vec3::zero(), Vec3::new(0.28 * scale, 0.12 * scale, 0.36 * scale),
            yaw, 0.0, roll_angle, udder_pos, TextureId::CowUdder,
        ));
    }

    let head_pos = base_pos + Vec3::new(yaw.sin() * 0.62 * scale, 0.95 * scale, yaw.cos() * 0.62 * scale);
    let head_yaw = yaw + mob.head_yaw_offset;
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.48 * scale, 0.48 * scale, 0.46 * scale),
        head_yaw, 0.0, roll_angle, head_pos, TextureId::CowSkin, TextureId::CowFace,
    ));

    let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.26 * scale, -0.10 * scale, head_yaw.cos() * 0.26 * scale);
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.34 * scale, 0.22 * scale, 0.16 * scale),
        head_yaw, 0.0, roll_angle, snout_pos, TextureId::CowSkin, TextureId::CowSnout,
    ));

    if !mob.is_baby {
        for side in [-1.0f32, 1.0f32] {
            let horn_pos = head_pos
                + Vec3::new(side * 0.28 * scale * head_yaw.cos(), 0.26 * scale, -side * 0.28 * scale * head_yaw.sin());
            emit(MobBox::uniform(
                Vec3::zero(), Vec3::new(0.10 * scale, 0.18 * scale, 0.10 * scale),
                head_yaw, 0.0, roll_angle, horn_pos, TextureId::CowHorns,
            ));
        }
    }

    let leg_offsets = [
        (-0.25 * scale, -0.38 * scale, leg_swing),
        (0.25 * scale, -0.38 * scale, -leg_swing),
        (-0.25 * scale, 0.38 * scale, -leg_swing),
        (0.25 * scale, 0.38 * scale, leg_swing),
    ];
    for &(lx, lz, swing) in &leg_offsets {
        let leg_origin = base_pos
            + Vec3::new(lx * yaw.cos() + lz * yaw.sin(), 0.50 * scale, -lx * yaw.sin() + lz * yaw.cos());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.25 * scale, 0.0),
            Vec3::new(0.22 * scale, 0.50 * scale, 0.22 * scale),
            yaw, swing, roll_angle, leg_origin, TextureId::CowSkin,
        ));
    }
}

pub(super) fn emit_chicken<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.34 * scale, 0.0),
        Vec3::new(0.36 * scale, 0.30 * scale, 0.44 * scale),
        yaw, 0.0, roll_angle, base_pos, TextureId::ChickenFeathers,
    ));

    let head_pos = base_pos + Vec3::new(yaw.sin() * 0.22 * scale, 0.48 * scale, yaw.cos() * 0.22 * scale);
    let head_yaw = yaw + mob.head_yaw_offset;
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.28 * scale, 0.28 * scale, 0.24 * scale),
        head_yaw, 0.0, roll_angle, head_pos, TextureId::ChickenFeathers, TextureId::ChickenFace,
    ));

    let beak_pos = head_pos + Vec3::new(head_yaw.sin() * 0.14 * scale, -0.03 * scale, head_yaw.cos() * 0.14 * scale);
    emit(MobBox::uniform(
        Vec3::zero(), Vec3::new(0.12 * scale, 0.06 * scale, 0.08 * scale),
        head_yaw, 0.0, roll_angle, beak_pos, TextureId::ChickenBeak,
    ));

    if !mob.is_baby {
        let wattle_pos = head_pos + Vec3::new(head_yaw.sin() * 0.13 * scale, -0.08 * scale, head_yaw.cos() * 0.13 * scale);
        emit(MobBox::uniform(
            Vec3::zero(), Vec3::new(0.06 * scale, 0.08 * scale, 0.06 * scale),
            head_yaw, 0.0, roll_angle, wattle_pos, TextureId::ChickenWattle,
        ));
    }

    let flap = (mob.walk_phase.sin() * 0.45).abs();
    let left_wing = base_pos + Vec3::new(-0.20 * scale * yaw.cos(), 0.38 * scale, 0.20 * scale * yaw.sin());
    emit(MobBox::uniform(
        Vec3::new(-0.02 * scale, -0.06 * scale, 0.0),
        Vec3::new(0.06 * scale, 0.20 * scale, 0.30 * scale),
        yaw, 0.0, roll_angle - flap, left_wing, TextureId::ChickenFeathers,
    ));

    let right_wing = base_pos + Vec3::new(0.20 * scale * yaw.cos(), 0.38 * scale, -0.20 * scale * yaw.sin());
    emit(MobBox::uniform(
        Vec3::new(0.02 * scale, -0.06 * scale, 0.0),
        Vec3::new(0.06 * scale, 0.20 * scale, 0.30 * scale),
        yaw, 0.0, roll_angle + flap, right_wing, TextureId::ChickenFeathers,
    ));

    let leg_offsets = [(-0.08 * scale, -0.02 * scale, leg_swing), (0.08 * scale, -0.02 * scale, -leg_swing)];
    for &(lx, lz, swing) in &leg_offsets {
        let leg_origin = base_pos
            + Vec3::new(lx * yaw.cos() + lz * yaw.sin(), 0.20 * scale, -lx * yaw.sin() + lz * yaw.cos());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.10 * scale, 0.0), Vec3::new(0.08 * scale, 0.20 * scale, 0.08 * scale),
            yaw, swing, roll_angle, leg_origin, TextureId::ChickenLeg,
        ));
    }
}

pub(super) fn emit_sheep<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };
    let wool_block = BlockId::wool_block_from_color(mob.wool_color);
    let wool_tex = wool_block.texture_id(BlockFace::Top);

    if !mob.is_sheared {
        emit(MobBox::uniform(
            Vec3::new(0.0, 0.72 * scale, 0.0),
            Vec3::new(0.72 * scale, 0.62 * scale, 1.02 * scale),
            yaw, 0.0, roll_angle, base_pos, wool_tex,
        ));
    } else {
        emit(MobBox::uniform(
            Vec3::new(0.0, 0.72 * scale, 0.0),
            Vec3::new(0.56 * scale, 0.52 * scale, 0.92 * scale),
            yaw, 0.0, roll_angle, base_pos, TextureId::SheepSkin,
        ));
    }

    let (head_pitch, head_y, head_fwd) = if mob.eat_grass_timer > 0.0 {
        (0.75, 0.44 * scale, 0.64 * scale)
    } else {
        (0.0, 0.86 * scale, 0.54 * scale)
    };
    let head_pos = base_pos + Vec3::new(yaw.sin() * head_fwd, head_y, yaw.cos() * head_fwd);
    let head_yaw = yaw + mob.head_yaw_offset;

    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.42 * scale, 0.42 * scale, 0.44 * scale),
        head_yaw, head_pitch, roll_angle, head_pos, TextureId::SheepSkin, TextureId::SheepFace,
    ));
    if !mob.is_sheared {
        emit(MobBox::uniform(
            Vec3::new(0.0, 0.12 * scale, -0.04 * scale),
            Vec3::new(0.44 * scale, 0.22 * scale, 0.36 * scale),
            head_yaw, head_pitch, roll_angle, head_pos, wool_tex,
        ));
    }

    let leg_offsets = [
        (-0.20, -0.32, leg_swing),
        (0.20, -0.32, -leg_swing),
        (-0.20, 0.32, -leg_swing),
        (0.20, 0.32, leg_swing),
    ];
    for &(lx, lz, swing) in &leg_offsets {
        let leg_origin = base_pos + Vec3::new(
            (lx * scale) * yaw.cos() + (lz * scale) * yaw.sin(),
            0.48 * scale,
            -(lx * scale) * yaw.sin() + (lz * scale) * yaw.cos(),
        );
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.24 * scale, 0.0), Vec3::new(0.18 * scale, 0.48 * scale, 0.18 * scale),
            yaw, swing, roll_angle, leg_origin, TextureId::SheepSkin,
        ));
        if !mob.is_sheared {
            emit(MobBox::uniform(
                Vec3::new(0.0, -0.12 * scale, 0.0), Vec3::new(0.24 * scale, 0.24 * scale, 0.24 * scale),
                yaw, swing, roll_angle, leg_origin, wool_tex,
            ));
        }
    }
}

pub(super) fn emit_dolphin<F: FnMut(MobBox)>(mob: &Mob, roll_angle: f32, emit: &mut F) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };
    let pitch = mob.swim_pitch;

    emit(MobBox::new(
        Vec3::new(0.0, 0.35 * scale, 0.0), Vec3::new(0.48 * scale, 0.42 * scale, 0.95 * scale),
        yaw, pitch, roll_angle, base_pos,
        TextureId::DolphinSkin, TextureId::DolphinBelly, TextureId::DolphinSkin, TextureId::DolphinSkin,
    ));
    emit(MobBox::new(
        Vec3::new(0.0, 0.28 * scale, 0.60 * scale), Vec3::new(0.24 * scale, 0.18 * scale, 0.32 * scale),
        yaw, pitch, roll_angle, base_pos,
        TextureId::DolphinSkin, TextureId::DolphinBelly, TextureId::DolphinSkin, TextureId::DolphinSkin,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.64 * scale, -0.06 * scale), Vec3::new(0.08 * scale, 0.28 * scale, 0.24 * scale),
        yaw, pitch, roll_angle, base_pos, TextureId::DolphinSkin,
    ));
    for side in &[-1.0f32, 1.0f32] {
        emit(MobBox::new(
            Vec3::new(side * 0.32 * scale, 0.24 * scale, 0.20 * scale),
            Vec3::new(0.24 * scale, 0.06 * scale, 0.22 * scale),
            yaw + side * 0.25, pitch, roll_angle + side * 0.3, base_pos,
            TextureId::DolphinSkin, TextureId::DolphinBelly, TextureId::DolphinSkin, TextureId::DolphinSkin,
        ));
    }
    let fluke_pitch = pitch + mob.walk_phase.sin() * 0.38;
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.35 * scale, -0.80 * scale), Vec3::new(0.55 * scale, 0.06 * scale, 0.28 * scale),
        yaw, fluke_pitch, roll_angle, base_pos, TextureId::DolphinSkin,
    ));
}

pub(super) fn emit_cod<F: FnMut(MobBox)>(mob: &Mob, roll_angle: f32, emit: &mut F) {
    let tail_swish = mob.walk_phase.sin() * 0.42;
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.16, 0.0), Vec3::new(0.18, 0.24, 0.48),
        mob.yaw, mob.swim_pitch, roll_angle, mob.pos, TextureId::CodSkin,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.16, -0.32), Vec3::new(0.04, 0.22, 0.20),
        mob.yaw + tail_swish, mob.swim_pitch, roll_angle, mob.pos, TextureId::CodSkin,
    ));
}

pub(super) fn emit_salmon<F: FnMut(MobBox)>(mob: &Mob, roll_angle: f32, emit: &mut F) {
    let tail_swish = mob.walk_phase.sin() * 0.45;
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.20, -0.05), Vec3::new(0.22, 0.30, 0.55),
        mob.yaw, mob.swim_pitch, roll_angle, mob.pos, TextureId::SalmonSkin,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.20, 0.30), Vec3::new(0.20, 0.26, 0.22),
        mob.yaw, mob.swim_pitch, roll_angle, mob.pos, TextureId::CodSkin,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.20, -0.42), Vec3::new(0.04, 0.28, 0.26),
        mob.yaw + tail_swish, mob.swim_pitch, roll_angle, mob.pos, TextureId::SalmonSkin,
    ));
}

pub(super) fn emit_pufferfish<F: FnMut(MobBox)>(mob: &Mob, roll_angle: f32, emit: &mut F) {
    let (size, y_center) = if mob.is_puffed {
        (Vec3::new(0.65, 0.65, 0.65), 0.35)
    } else {
        (Vec3::new(0.28, 0.28, 0.28), 0.16)
    };
    emit(MobBox::with_front(
        Vec3::new(0.0, y_center, 0.0), size, mob.yaw, mob.swim_pitch, roll_angle, mob.pos,
        TextureId::PufferfishSkin, TextureId::Pufferfish,
    ));
    let fin_flap = (mob.walk_phase * 2.0).sin() * 0.4;
    for side in &[-1.0f32, 1.0f32] {
        emit(MobBox::uniform(
            Vec3::new(side * (size.x * 0.55), y_center, 0.0), Vec3::new(0.04, 0.14, 0.14),
            mob.yaw + side * 0.2, mob.swim_pitch + fin_flap, roll_angle + side * 0.3, mob.pos,
            TextureId::PufferfishSkin,
        ));
    }
}

pub(super) fn emit_tropical_fish<F: FnMut(MobBox)>(mob: &Mob, roll_angle: f32, emit: &mut F) {
    let tail_swish = mob.walk_phase.sin() * 0.50;
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.16, 0.0), Vec3::new(0.14, 0.26, 0.36), mob.yaw, mob.swim_pitch, roll_angle, mob.pos,
        TextureId::TropicalFishSkin, TextureId::TropicalFish,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.16, -0.24), Vec3::new(0.04, 0.24, 0.18),
        mob.yaw + tail_swish, mob.swim_pitch, roll_angle, mob.pos, TextureId::TropicalFishSkin,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.34, 0.0), Vec3::new(0.04, 0.14, 0.18),
        mob.yaw, mob.swim_pitch, roll_angle, mob.pos, TextureId::TropicalFishSkin,
    ));
}
