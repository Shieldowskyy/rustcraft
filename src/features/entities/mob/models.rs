//! Oriented 3D box primitives (`MobBox`) and species-dispatched procedural mob geometry emitter.
//! Converts dynamic `Mob` pose state into textured 3D box parts via a zero-allocation callback.

use std::f32::consts::FRAC_PI_2;

use crate::core::math::Vec3;
use crate::features::rendering::atlas::TextureId;

use super::kind::MobKind;
use super::models_hostile::{
    emit_iron_golem, emit_reaper, emit_skeleton, emit_spider, emit_villager, emit_zombie,
};
use super::models_passive::{
    emit_chicken, emit_cod, emit_cow, emit_dolphin, emit_pig, emit_pufferfish, emit_salmon,
    emit_sheep, emit_tropical_fish,
};
use super::state::Mob;

#[derive(Copy, Clone, Debug)]
pub struct MobBox {
    pub center: Vec3,
    pub size: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub origin: Vec3,
    pub tex_top: TextureId,
    pub tex_bottom: TextureId,
    pub tex_side: TextureId,
    pub tex_front: TextureId,
}

impl MobBox {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        center: Vec3,
        size: Vec3,
        yaw: f32,
        pitch: f32,
        roll: f32,
        origin: Vec3,
        tex_top: TextureId,
        tex_bottom: TextureId,
        tex_side: TextureId,
        tex_front: TextureId,
    ) -> Self {
        Self { center, size, yaw, pitch, roll, origin, tex_top, tex_bottom, tex_side, tex_front }
    }

    #[inline(always)]
    pub const fn uniform(
        center: Vec3,
        size: Vec3,
        yaw: f32,
        pitch: f32,
        roll: f32,
        origin: Vec3,
        tex: TextureId,
    ) -> Self {
        Self::new(center, size, yaw, pitch, roll, origin, tex, tex, tex, tex)
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub const fn with_front(
        center: Vec3,
        size: Vec3,
        yaw: f32,
        pitch: f32,
        roll: f32,
        origin: Vec3,
        tex_body: TextureId,
        tex_front: TextureId,
    ) -> Self {
        Self::new(center, size, yaw, pitch, roll, origin, tex_body, tex_body, tex_body, tex_front)
    }

    #[inline]
    pub fn world_corners(&self) -> [Vec3; 8] {
        let half = Vec3::new(self.size.x * 0.5, self.size.y * 0.5, self.size.z * 0.5);
        let local_corners = [
            self.center + Vec3::new(-half.x, -half.y, -half.z),
            self.center + Vec3::new(half.x, -half.y, -half.z),
            self.center + Vec3::new(half.x, half.y, -half.z),
            self.center + Vec3::new(-half.x, half.y, -half.z),
            self.center + Vec3::new(-half.x, -half.y, half.z),
            self.center + Vec3::new(half.x, -half.y, half.z),
            self.center + Vec3::new(half.x, half.y, half.z),
            self.center + Vec3::new(-half.x, half.y, half.z),
        ];
        let (sp, cp) = if self.pitch != 0.0 { self.pitch.sin_cos() } else { (0.0, 1.0) };
        let (sr, cr) = if self.roll != 0.0 { self.roll.sin_cos() } else { (0.0, 1.0) };
        let (sy, cy) = self.yaw.sin_cos();
        local_corners.map(|p| {
            let p_pitch = if self.pitch != 0.0 {
                Vec3::new(p.x, p.y * cp - p.z * sp, p.y * sp + p.z * cp)
            } else {
                p
            };
            let p_roll = if self.roll != 0.0 {
                Vec3::new(p_pitch.x * cr - p_pitch.y * sr, p_pitch.x * sr + p_pitch.y * cr, p_pitch.z)
            } else {
                p_pitch
            };
            let p_yaw = Vec3::new(p_roll.x * cy + p_roll.z * sy, p_roll.y, -p_roll.x * sy + p_roll.z * cy);
            self.origin + p_yaw
        })
    }

    #[inline]
    pub fn emit_visible_faces<F>(&self, camera_pos: Vec3, mut on_face: F)
    where
        F: FnMut(Vec3, Vec3, Vec3, Vec3, TextureId, f32),
    {
        let world_pts = self.world_corners();
        let mut check_face = |i0: usize, i1: usize, i2: usize, i3: usize, tex: TextureId, shade: f32| {
            let (p0, p1, p2, p3) = (world_pts[i0], world_pts[i1], world_pts[i2], world_pts[i3]);
            let normal = (p1 - p0).cross(p3 - p0);
            let center = (p0 + p1 + p2 + p3) * 0.25;
            if (center - camera_pos).dot(normal) < 0.0 {
                on_face(p0, p1, p2, p3, tex, shade);
            }
        };
        check_face(7, 6, 2, 3, self.tex_top, 1.00);
        check_face(4, 0, 1, 5, self.tex_bottom, 0.50);
        check_face(4, 5, 6, 7, self.tex_front, 0.85);
        check_face(1, 0, 3, 2, self.tex_side, 0.85);
        check_face(0, 4, 7, 3, self.tex_side, 0.70);
        check_face(5, 1, 2, 6, self.tex_side, 0.70);
    }
}

pub fn emit_mob_boxes<F: FnMut(MobBox)>(mob: &Mob, mut emit: F) {
    let death_t = (mob.death_timer / 0.8).clamp(0.0, 1.0);
    let roll_angle = death_t * FRAC_PI_2;
    let leg_swing = mob.walk_phase.sin() * 0.65;

    match mob.kind {
        MobKind::Pig => emit_pig(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Cow => emit_cow(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Chicken => emit_chicken(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Sheep => emit_sheep(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Zombie => emit_zombie(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Skeleton => emit_skeleton(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Reaper => emit_reaper(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Dolphin => emit_dolphin(mob, roll_angle, &mut emit),
        MobKind::Cod => emit_cod(mob, roll_angle, &mut emit),
        MobKind::Salmon => emit_salmon(mob, roll_angle, &mut emit),
        MobKind::Pufferfish => emit_pufferfish(mob, roll_angle, &mut emit),
        MobKind::TropicalFish => emit_tropical_fish(mob, roll_angle, &mut emit),
        MobKind::Villager => emit_villager(mob, roll_angle, leg_swing, &mut emit),
        MobKind::IronGolem => emit_iron_golem(mob, roll_angle, leg_swing, &mut emit),
        MobKind::Spider => emit_spider(mob, roll_angle, leg_swing, &mut emit),
    }
}
