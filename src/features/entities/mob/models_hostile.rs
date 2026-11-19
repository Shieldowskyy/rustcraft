//! Procedural 3D box geometry builders for humanoid, villager, golem, reaper, and spider mobs.

use crate::core::math::Vec3;
use crate::features::rendering::atlas::TextureId;

use super::models::MobBox;
use super::state::Mob;

pub(super) fn emit_zombie<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    emit(MobBox::new(
        Vec3::new(0.0, 1.07, 0.0), Vec3::new(0.48, 0.70, 0.26), yaw, 0.0, roll_angle, base_pos,
        TextureId::ZombieShirt, TextureId::ZombiePants, TextureId::ZombieShirt, TextureId::ZombieShirt,
    ));

    let head_pos = base_pos + Vec3::new(0.0, 1.44, 0.0);
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.24, 0.0), Vec3::new(0.48, 0.48, 0.48),
        yaw + mob.head_yaw_offset, 0.0, roll_angle, head_pos, TextureId::ZombieSkin, TextureId::ZombieFace,
    ));

    for &(ax, ay) in &[(-0.36f32, 1.36f32), (0.36f32, 1.36f32)] {
        let arm_origin = base_pos + Vec3::new(ax * yaw.cos(), ay, -ax * yaw.sin());
        emit(MobBox::new(
            Vec3::new(0.0, -0.32, 0.0), Vec3::new(0.24, 0.64, 0.24), yaw, -1.45, roll_angle, arm_origin,
            TextureId::ZombieShirt, TextureId::ZombieSkin, TextureId::ZombieSkin, TextureId::ZombieSkin,
        ));
    }
    for &(lx, swing) in &[(-0.14f32, leg_swing), (0.14f32, -leg_swing)] {
        let leg_origin = base_pos + Vec3::new(lx * yaw.cos(), 0.72, -lx * yaw.sin());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.36, 0.0), Vec3::new(0.24, 0.72, 0.24), yaw, swing, roll_angle, leg_origin,
            TextureId::ZombiePants,
        ));
    }
}

pub(super) fn emit_skeleton<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    emit(MobBox::with_front(
        Vec3::new(0.0, 1.07, 0.0), Vec3::new(0.44, 0.70, 0.24), yaw, 0.0, roll_angle, base_pos,
        TextureId::SkeletonBone, TextureId::SkeletonBody,
    ));

    let head_pos = base_pos + Vec3::new(0.0, 1.44, 0.0);
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.23, 0.0), Vec3::new(0.46, 0.46, 0.46),
        yaw + mob.head_yaw_offset, 0.0, roll_angle, head_pos, TextureId::SkeletonBone, TextureId::SkeletonFace,
    ));

    for &(ax, ay) in &[(-0.32f32, 1.36f32), (0.32f32, 1.36f32)] {
        let arm_origin = base_pos + Vec3::new(ax * yaw.cos(), ay, -ax * yaw.sin());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.32, 0.0), Vec3::new(0.18, 0.64, 0.18), yaw, -1.35, roll_angle, arm_origin,
            TextureId::SkeletonBone,
        ));
    }
    for &(lx, swing) in &[(-0.13f32, leg_swing), (0.13f32, -leg_swing)] {
        let leg_origin = base_pos + Vec3::new(lx * yaw.cos(), 0.72, -lx * yaw.sin());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.36, 0.0), Vec3::new(0.18, 0.72, 0.18), yaw, swing, roll_angle, leg_origin,
            TextureId::SkeletonBone,
        ));
    }
}

pub(super) fn emit_reaper<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let swell = if mob.fuse > 0.0 { 1.0 + (mob.fuse / 1.5).min(1.0) * 0.22 } else { 1.0 };

    emit(MobBox::uniform(
        Vec3::new(0.0, 0.72 * swell, 0.0), Vec3::new(0.44 * swell, 0.72 * swell, 0.24 * swell),
        yaw, 0.0, roll_angle, base_pos, TextureId::ReaperSkin,
    ));

    let head_pos = base_pos + Vec3::new(0.0, 1.08 * swell, 0.0);
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.24 * swell, 0.0), Vec3::new(0.48 * swell, 0.48 * swell, 0.48 * swell),
        yaw + mob.head_yaw_offset, 0.0, roll_angle, head_pos, TextureId::ReaperSkin, TextureId::ReaperFace,
    ));

    let leg_offsets = [
        (-0.13f32, 0.14f32, leg_swing),
        (0.13f32, 0.14f32, -leg_swing),
        (-0.13f32, -0.14f32, -leg_swing),
        (0.13f32, -0.14f32, leg_swing),
    ];
    for &(lx, lz, swing) in &leg_offsets {
        let leg_origin = base_pos
            + Vec3::new((lx * yaw.cos() + lz * yaw.sin()) * swell, 0.36 * swell, (-lx * yaw.sin() + lz * yaw.cos()) * swell);
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.18 * swell, 0.0), Vec3::new(0.22 * swell, 0.36 * swell, 0.22 * swell),
            yaw, swing, roll_angle, leg_origin, TextureId::ReaperSkin,
        ));
    }
}

pub(super) fn emit_villager<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let robe_tex = match mob.villager_profession {
        1 => TextureId::VillagerRobeLibrarian,
        2 => TextureId::VillagerRobeBlacksmith,
        3 => TextureId::VillagerRobeCleric,
        4 => TextureId::VillagerRobeButcher,
        _ => TextureId::VillagerRobe,
    };

    emit(MobBox::uniform(
        Vec3::new(0.0, 0.88, 0.0), Vec3::new(0.50, 0.88, 0.38), yaw, 0.0, roll_angle, base_pos, robe_tex,
    ));
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.98, 0.16), Vec3::new(0.54, 0.24, 0.22), yaw, 0.0, roll_angle, base_pos,
        robe_tex, TextureId::VillagerArms,
    ));

    let head_pos = base_pos + Vec3::new(0.0, 1.34, 0.0);
    let head_yaw = yaw + mob.head_yaw_offset;
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.26, 0.0), Vec3::new(0.48, 0.52, 0.48), head_yaw, 0.0, roll_angle, head_pos,
        TextureId::VillagerSkin, TextureId::VillagerFace,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.16, 0.28), Vec3::new(0.12, 0.22, 0.14), head_yaw, 0.0, roll_angle, head_pos,
        TextureId::VillagerSkin,
    ));

    for &(lx, swing) in &[(-0.13f32, leg_swing * 0.6), (0.13f32, -leg_swing * 0.6)] {
        let leg_origin = base_pos + Vec3::new(lx * yaw.cos(), 0.44, -lx * yaw.sin());
        emit(MobBox::new(
            Vec3::new(0.0, -0.22, 0.0), Vec3::new(0.20, 0.44, 0.20), yaw, swing, roll_angle, leg_origin,
            TextureId::VillagerRobe, TextureId::VillagerSkin, TextureId::VillagerRobe, TextureId::VillagerRobe,
        ));
    }
}

pub(super) fn emit_iron_golem<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;

    emit(MobBox::uniform(
        Vec3::new(0.0, 1.10, 0.0), Vec3::new(0.72, 0.52, 0.46), yaw, 0.0, roll_angle, base_pos,
        TextureId::IronGolemBody,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 1.70, 0.0), Vec3::new(1.16, 0.76, 0.66), yaw, 0.0, roll_angle, base_pos,
        TextureId::IronGolemBody,
    ));

    let head_pos = base_pos + Vec3::new(0.0, 2.08, 0.12);
    let head_yaw = yaw + mob.head_yaw_offset;
    emit(MobBox::with_front(
        Vec3::new(0.0, 0.26, 0.0), Vec3::new(0.52, 0.56, 0.48), head_yaw, 0.0, roll_angle, head_pos,
        TextureId::IronGolemBody, TextureId::IronGolemHead,
    ));
    emit(MobBox::uniform(
        Vec3::new(0.0, 0.16, 0.28), Vec3::new(0.12, 0.24, 0.14), head_yaw, 0.0, roll_angle, head_pos,
        TextureId::IronGolemBody,
    ));

    for &(lx, swing) in &[(-0.25f32, leg_swing * 0.45), (0.25f32, -leg_swing * 0.45)] {
        let leg_origin = base_pos + Vec3::new(lx * yaw.cos(), 0.84, -lx * yaw.sin());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.42, 0.0), Vec3::new(0.32, 0.84, 0.32), yaw, swing, roll_angle, leg_origin,
            TextureId::IronGolemBody,
        ));
    }
    for &(ax, swing) in &[(-0.70f32, -leg_swing * 0.5), (0.70f32, leg_swing * 0.5)] {
        let arm_origin = base_pos + Vec3::new(ax * yaw.cos(), 1.95, -ax * yaw.sin());
        emit(MobBox::uniform(
            Vec3::new(0.0, -0.70, 0.0), Vec3::new(0.28, 1.45, 0.34), yaw, swing, roll_angle, arm_origin,
            TextureId::IronGolemBody,
        ));
    }
}

pub(super) fn emit_spider<F: FnMut(MobBox)>(
    mob: &Mob,
    roll_angle: f32,
    leg_swing: f32,
    emit: &mut F,
) {
    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let scale = if mob.is_baby { 0.5f32 } else { 1.0f32 };

    emit(MobBox::uniform(
        Vec3::new(0.0, 0.32 * scale, 0.0), Vec3::new(0.38 * scale, 0.32 * scale, 0.44 * scale),
        yaw, 0.0, roll_angle, base_pos, TextureId::SpiderSkin,
    ));

    let head_pos = base_pos + Vec3::new(yaw.sin() * 0.38 * scale, 0.34 * scale, yaw.cos() * 0.38 * scale);
    emit(MobBox::with_front(
        Vec3::zero(), Vec3::new(0.46 * scale, 0.38 * scale, 0.44 * scale),
        yaw + mob.head_yaw_offset, 0.0, roll_angle, head_pos, TextureId::SpiderSkin, TextureId::SpiderFace,
    ));

    let abdomen_pos = base_pos - Vec3::new(yaw.sin() * 0.52 * scale, -0.06 * scale, yaw.cos() * 0.52 * scale);
    emit(MobBox::uniform(
        Vec3::zero(), Vec3::new(0.68 * scale, 0.48 * scale, 0.72 * scale),
        yaw, 0.0, roll_angle, abdomen_pos, TextureId::SpiderSkin,
    ));

    for (i, &lz) in [0.18f32, 0.06, -0.06, -0.18].iter().enumerate() {
        let leg_pitch = leg_swing * if i % 2 == 0 { 1.0 } else { -1.0 } * 0.6;

        let lx_left = -0.22f32 * scale;
        let leg_origin_l = base_pos + Vec3::new(
            lx_left * yaw.cos() + (lz * scale) * yaw.sin(),
            0.32 * scale,
            -lx_left * yaw.sin() + (lz * scale) * yaw.cos(),
        );
        emit(MobBox::uniform(
            Vec3::new(-0.25 * scale, -0.08 * scale, 0.0), Vec3::new(0.55 * scale, 0.10 * scale, 0.10 * scale),
            yaw, leg_pitch, roll_angle + 0.35, leg_origin_l, TextureId::SpiderSkin,
        ));

        let lx_right = 0.22f32 * scale;
        let leg_origin_r = base_pos + Vec3::new(
            lx_right * yaw.cos() + (lz * scale) * yaw.sin(),
            0.32 * scale,
            -lx_right * yaw.sin() + (lz * scale) * yaw.cos(),
        );
        emit(MobBox::uniform(
            Vec3::new(0.25 * scale, -0.08 * scale, 0.0), Vec3::new(0.55 * scale, 0.10 * scale, 0.10 * scale),
            yaw, -leg_pitch, roll_angle - 0.35, leg_origin_r, TextureId::SpiderSkin,
        ));
    }
}
