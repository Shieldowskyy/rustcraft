//! Mob combat interactions: iron golem defense, skeleton archery, animal breeding, explosions, and raycasting.

use crate::audio::SoundId;
use crate::core::math::Vec3;
use crate::features::world::World;

use super::super::item::ArrowEntity;
use super::constants::{
    GOLEM_ENEMY_RANGE_SQ, GOLEM_VILLAGER_ANGER_RANGE_SQ, SKELETON_BOW_RANGE_SQ,
    SKELETON_FIRE_INTERVAL,
};
use super::kind::MobKind;
use super::manager::MobManager;
use super::state::Mob;

impl MobManager {
    pub(super) fn resolve_golem_defense(&mut self) {
        let mut golem_attacks: Vec<(usize, usize, f32, f32)> = Vec::new();
        let mob_count = self.mobs.len();
        for i in 0..mob_count {
            if self.mobs[i].kind != MobKind::IronGolem || self.mobs[i].is_dead {
                continue;
            }
            let golem_pos = self.mobs[i].pos;
            let mut closest: Option<(usize, f32, f32, f32)> = None;
            for j in 0..mob_count {
                if !self.mobs[j].kind.is_hostile() || self.mobs[j].is_dead {
                    continue;
                }
                let (dx, dz, dy) = (
                    self.mobs[j].pos.x - golem_pos.x,
                    self.mobs[j].pos.z - golem_pos.z,
                    (self.mobs[j].pos.y - golem_pos.y).abs(),
                );
                let dist_sq = dx * dx + dz * dz;
                if dist_sq < GOLEM_ENEMY_RANGE_SQ
                    && dy < 6.0
                    && closest.as_ref().map_or(true, |&(_, min_d, _, _)| dist_sq < min_d)
                {
                    closest = Some((j, dist_sq, dx, dz));
                }
            }
            if let Some((target_idx, dist_sq, dx, dz)) = closest {
                self.mobs[i].target_yaw = dx.atan2(dz);
                self.mobs[i].ai_state_is_moving = true;
                if dist_sq <= 2.2 * 2.2 {
                    golem_attacks.push((i, target_idx, dx, dz));
                }
            }
        }

        for (golem_idx, target_idx, dx, dz) in golem_attacks {
            let kb_dir = Vec3::new(dx.signum() * 1.5, 4.2, dz.signum() * 1.5);
            self.mobs[target_idx].take_damage(14.0, kb_dir);
            self.mobs[golem_idx].pending_sound = Some(SoundId::IronGolemAttack);
            self.mobs[golem_idx].walk_phase += 2.0;
        }
    }

    pub(super) fn resolve_breeding(&mut self, rng: &mut u32) {
        let mut babies = Vec::new();
        let len = self.mobs.len();
        for i in 0..len {
            if self.mobs[i].in_love <= 0.0 || self.mobs[i].is_baby || self.mobs[i].is_dead {
                continue;
            }
            for j in (i + 1)..len {
                if self.mobs[j].in_love <= 0.0 || self.mobs[j].is_baby || self.mobs[j].is_dead {
                    continue;
                }
                if self.mobs[i].kind != self.mobs[j].kind {
                    continue;
                }
                let delta = self.mobs[i].pos - self.mobs[j].pos;
                let dist_sq = delta.length_sq();
                if dist_sq < 2.25 {
                    self.mobs[i].in_love = 0.0;
                    self.mobs[j].in_love = 0.0;
                    self.mobs[i].breed_cooldown = 300.0;
                    self.mobs[j].breed_cooldown = 300.0;
                    let mid = (self.mobs[i].pos + self.mobs[j].pos) * 0.5;
                    self.breeding_hearts.push(mid + Vec3::new(0.0, 0.8, 0.0));
                    let mut baby = Mob::new(self.next_mob_id, self.mobs[i].kind, mid);
                    self.next_mob_id = self.next_mob_id.wrapping_add(1);
                    baby.is_baby = true;
                    baby.age = -120.0;
                    baby.health = self.mobs[i].kind.max_health() * 0.5;
                    if self.mobs[i].kind == MobKind::Sheep {
                        baby.wool_color = if (rng.wrapping_mul(22695477) % 2) == 0 {
                            self.mobs[i].wool_color
                        } else {
                            self.mobs[j].wool_color
                        };
                    }
                    babies.push(baby);
                } else if dist_sq < 64.0 {
                    self.mobs[i].target_yaw = (-delta.x).atan2(-delta.z);
                    self.mobs[i].ai_state_is_moving = true;
                    self.mobs[j].target_yaw = delta.x.atan2(delta.z);
                    self.mobs[j].ai_state_is_moving = true;
                }
            }
        }
        self.mobs.extend(babies);
    }

    pub fn anger_golems_at_villager_attack(&mut self, villager_pos: Vec3) {
        for golem in &mut self.mobs {
            if golem.kind != MobKind::IronGolem || golem.is_dead {
                continue;
            }
            let (dx, dz, dy) = (
                golem.pos.x - villager_pos.x,
                golem.pos.z - villager_pos.z,
                (golem.pos.y - villager_pos.y).abs(),
            );
            if dx * dx + dz * dz <= GOLEM_VILLAGER_ANGER_RANGE_SQ && dy <= 8.0 {
                golem.player_anger_timer = 30.0;
            }
        }
    }

    pub fn apply_explosion(&mut self, explosion_pos: Vec3, radius: f32, max_damage: f32) {
        for mob in &mut self.mobs {
            if mob.is_dead {
                continue;
            }
            let to_mob = mob.center_pos() - explosion_pos;
            let dist = to_mob.length();
            if dist <= radius {
                let factor = (1.0 - dist / radius).clamp(0.0, 1.0);
                let damage = (factor * max_damage).max(4.0);
                let kb_dir =
                    if dist > 0.01 { to_mob.normalized() } else { Vec3::new(0.0, 1.0, 0.0) };
                mob.take_damage(damage, kb_dir);
                let kb_strength = factor * 12.0;
                mob.velocity.x += kb_dir.x * kb_strength;
                mob.velocity.y = (mob.velocity.y + kb_dir.y * kb_strength + 4.0).min(14.0);
                mob.velocity.z += kb_dir.z * kb_strength;
                mob.on_ground = false;
            }
        }
    }

    pub fn raycast_mob(&self, origin: Vec3, dir: Vec3, max_reach: f32) -> Option<(usize, f32)> {
        let dir = dir.normalized();
        let mut closest_hit: Option<(usize, f32)> = None;
        for (idx, mob) in self.mobs.iter().enumerate() {
            if mob.is_dead {
                continue;
            }
            let (min, max) = mob.aabb();
            let padding = 0.15;
            let min = Vec3::new(min.x - padding, min.y, min.z - padding);
            let max = Vec3::new(max.x + padding, max.y + padding, max.z + padding);
            if let Some(t) = ray_aabb_intersect(origin, dir, min, max) {
                if t >= 0.0 && t <= max_reach && closest_hit.map_or(true, |(_, best)| t < best) {
                    closest_hit = Some((idx, t));
                }
            }
        }
        closest_hit
    }
}

pub(super) fn try_skeleton_shoot(
    mob: &mut Mob,
    dt: f32,
    world: &World,
    player_pos: Vec3,
) -> Option<ArrowEntity> {
    let (dx, dz, dy) = (
        player_pos.x - mob.pos.x,
        player_pos.z - mob.pos.z,
        (player_pos.y - mob.pos.y).abs(),
    );
    if dx * dx + dz * dz >= SKELETON_BOW_RANGE_SQ || dy >= 10.0 {
        return None;
    }
    let start_pos = mob.pos + Vec3::new(0.0, 1.4, 0.0);
    let aim_pos = player_pos + Vec3::new(0.0, 0.8, 0.0);
    let shot_delta = aim_pos - start_pos;
    let line_of_sight =
        world.raycast_camera_collision(start_pos, shot_delta, shot_delta.length()) >= shot_delta.length();
    if line_of_sight {
        mob.attack_cooldown += dt;
    }
    if line_of_sight && mob.attack_cooldown >= SKELETON_FIRE_INTERVAL {
        mob.attack_cooldown = 0.0;
        let dir = shot_delta.normalized();
        mob.pending_sound = Some(SoundId::BowShoot);
        Some(ArrowEntity::new(start_pos, (dir + Vec3::new(0.0, 0.08, 0.0)) * 18.0, false, 3.0))
    } else {
        None
    }
}

fn ray_aabb_intersect(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = (min.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    let mut t_max = (max.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    if t_min > t_max {
        std::mem::swap(&mut t_min, &mut t_max);
    }
    let mut ty_min = (min.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    let mut ty_max = (max.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    if ty_min > ty_max {
        std::mem::swap(&mut ty_min, &mut ty_max);
    }
    if (t_min > ty_max) || (ty_min > t_max) {
        return None;
    }
    if ty_min > t_min {
        t_min = ty_min;
    }
    if ty_max < t_max {
        t_max = ty_max;
    }
    let mut tz_min = (min.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    let mut tz_max = (max.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    if tz_min > tz_max {
        std::mem::swap(&mut tz_min, &mut tz_max);
    }
    if (t_min > tz_max) || (tz_min > t_max) {
        return None;
    }
    if tz_min > t_min {
        t_min = tz_min;
    }
    if tz_max < t_max {
        t_max = tz_max;
    }
    if t_max < 0.0 {
        return None;
    }
    Some(if t_min >= 0.0 { t_min } else { t_max })
}
