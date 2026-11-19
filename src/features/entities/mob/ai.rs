//! Per-tick terrestrial mob AI state machine, timers, target pursuit, and locomotion.
//! Updates a single `Mob`'s orientation, velocity, timers, and position via `World` collision queries.

use std::f32::consts::PI;

use crate::audio::SoundId;
use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::world::World;

use super::constants::{
    DEATH_DESPAWN_DELAY, HOSTILE_TARGET_RANGE_SQ, HOSTILE_TARGET_Y_RANGE, MOB_GRAVITY,
    REAPER_FUSE_DURATION,
};
use super::kind::MobKind;
use super::state::Mob;

#[inline]
pub(super) fn next_rng_f32(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    (*state as f32) / (u32::MAX as f32)
}

#[inline]
pub(super) fn wrap_angle_delta(delta: f32) -> f32 {
    let mut diff = delta;
    while diff > PI {
        diff -= PI * 2.0;
    }
    while diff < -PI {
        diff += PI * 2.0;
    }
    diff
}

impl Mob {
    pub fn update(
        &mut self,
        dt: f32,
        world: &World,
        rng: &mut u32,
        player_pos: Option<Vec3>,
        player_held: Option<BlockId>,
        villager_pos: Option<Vec3>,
    ) {
        self.tick_timers(dt, rng);

        if self.is_dead {
            self.death_timer += dt;
            self.velocity.y -= MOB_GRAVITY * dt;
            self.apply_movement(dt, world);
            if self.death_timer >= DEATH_DESPAWN_DELAY {
                self.despawn_pending = true;
            }
            return;
        }

        let in_water = self.apply_environment_hazards(world, rng);

        if self.kind.is_aquatic() {
            if in_water {
                self.update_aquatic_in_water(dt, world, rng, player_pos);
            } else {
                self.update_aquatic_on_land(dt, world, rng);
            }
            return;
        }

        if self.panic_timer > 0.0 {
            self.panic_timer = (self.panic_timer - dt).max(0.0);
        }

        let pursuing_player = self.update_hostile_ai(dt, player_pos, villager_pos);
        let (following_player, returning_home, skittish_avoiding) =
            self.update_passive_social_ai(player_pos, player_held);

        if !pursuing_player && !following_player && !skittish_avoiding && !returning_home {
            self.update_idle_or_panic_wander(dt, rng, player_pos);
        }

        self.advance_ground_locomotion(dt, world, in_water, pursuing_player, skittish_avoiding);
    }

    fn tick_timers(&mut self, dt: f32, rng: &mut u32) {
        if self.in_love > 0.0 {
            self.in_love = (self.in_love - dt).max(0.0);
        }
        if self.breed_cooldown > 0.0 {
            self.breed_cooldown = (self.breed_cooldown - dt).max(0.0);
        }
        if self.is_baby {
            self.age += dt;
            if self.age >= 0.0 {
                self.is_baby = false;
            }
        }
        if self.eat_grass_timer > 0.0 {
            self.eat_grass_timer = (self.eat_grass_timer - dt).max(0.0);
            if self.eat_grass_timer == 0.0 {
                self.is_sheared = false;
            }
        }

        if !self.is_dead {
            self.ambient_sound_timer -= dt;
            if self.ambient_sound_timer <= 0.0 {
                self.ambient_sound_timer = 7.0 + next_rng_f32(rng) * 9.0;
                if self.pending_sound.is_none() {
                    self.pending_sound = self.kind.idle_sound();
                }
            }
        }

        if self.hurt_timer > 0.0 {
            self.hurt_timer = (self.hurt_timer - dt).max(0.0);
        }
    }

    fn update_hostile_ai(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        villager_pos: Option<Vec3>,
    ) -> bool {
        if !(self.kind.is_hostile()
            || (self.kind == MobKind::IronGolem && self.player_anger_timer > 0.0))
        {
            return false;
        }
        let target = if self.kind == MobKind::Zombie {
            villager_pos.or(player_pos)
        } else {
            player_pos
        };
        let Some(target) = target else {
            return false;
        };

        let dx = target.x - self.pos.x;
        let dz = target.z - self.pos.z;
        let dy = (target.y - self.pos.y).abs();
        let dist_sq = dx * dx + dz * dz;

        if self.kind == MobKind::Zombie && villager_pos.is_some() {
            if dist_sq < HOSTILE_TARGET_RANGE_SQ && dy < HOSTILE_TARGET_Y_RANGE {
                self.target_yaw = dx.atan2(dz);
                self.ai_state_is_moving = dist_sq > 1.5 * 1.5;
                self.head_yaw_offset = 0.0;
                return true;
            }
            return false;
        }

        if self.kind == MobKind::Reaper {
            let total_dist_sq = dist_sq + (target.y - self.pos.y) * (target.y - self.pos.y);
            if total_dist_sq < 3.2 * 3.2 {
                if !self.is_ignited {
                    self.is_ignited = true;
                    self.pending_sound = Some(SoundId::ReaperHiss);
                }
                self.fuse += dt;
                if self.fuse >= REAPER_FUSE_DURATION {
                    self.is_dead = true;
                    self.despawn_pending = true;
                }
            } else if total_dist_sq > 5.5 * 5.5 {
                self.is_ignited = false;
                self.fuse = (self.fuse - dt * 1.5).max(0.0);
            } else if self.is_ignited {
                self.fuse += dt;
                if self.fuse >= REAPER_FUSE_DURATION {
                    self.is_dead = true;
                    self.despawn_pending = true;
                }
            }
            if dist_sq < HOSTILE_TARGET_RANGE_SQ && dy < HOSTILE_TARGET_Y_RANGE && !self.is_dead {
                self.target_yaw = dx.atan2(dz);
                self.ai_state_is_moving = self.fuse <= 0.0 || total_dist_sq > 2.8 * 2.8;
                self.head_yaw_offset = 0.0;
                return true;
            }
            return false;
        }

        if dist_sq >= HOSTILE_TARGET_RANGE_SQ || dy >= HOSTILE_TARGET_Y_RANGE {
            return false;
        }

        self.target_yaw = dx.atan2(dz);
        self.head_yaw_offset = 0.0;
        self.ai_state_is_moving = match self.kind {
            MobKind::Skeleton => dist_sq > 9.0 * 9.0,
            MobKind::Spider => {
                if dist_sq < 6.0 * 6.0 && dist_sq > 1.8 * 1.8 && self.on_ground {
                    let to_p = Vec3::new(dx, 0.0, dz).normalized();
                    self.velocity.y = 4.8;
                    self.velocity.x = to_p.x * 5.5;
                    self.velocity.z = to_p.z * 5.5;
                    self.on_ground = false;
                }
                true
            }
            _ => true,
        };
        true
    }

    fn update_passive_social_ai(
        &mut self,
        player_pos: Option<Vec3>,
        player_held: Option<BlockId>,
    ) -> (bool, bool, bool) {
        let mut following_player = false;
        if !self.kind.is_hostile() && self.panic_timer <= 0.0 {
            if let (Some(target), Some(held)) = (player_pos, player_held) {
                if self.kind.is_tempting_item(held) {
                    let (dx, dz, dy) =
                        (target.x - self.pos.x, target.z - self.pos.z, (target.y - self.pos.y).abs());
                    let dist_sq = dx * dx + dz * dz;
                    if dist_sq < 10.0 * 10.0 && dy < 6.0 {
                        self.target_yaw = dx.atan2(dz);
                        self.head_yaw_offset = 0.0;
                        self.ai_state_is_moving = dist_sq > 2.2 * 2.2;
                        following_player = true;
                    }
                }
            }
        }

        let mut returning_home = false;
        if self.kind == MobKind::Villager {
            if let Some(home) = self.home_pos {
                let (dx, dz) = (home.x - self.pos.x, home.z - self.pos.z);
                if dx * dx + dz * dz > 14.0 * 14.0 {
                    self.target_yaw = dx.atan2(dz);
                    self.ai_state_is_moving = true;
                    self.ai_timer = 0.5;
                    following_player = false;
                    returning_home = true;
                }
            }
        }

        let mut skittish_avoiding = false;
        if matches!(self.kind, MobKind::Pig | MobKind::Cow | MobKind::Chicken | MobKind::Sheep)
            && !following_player
            && self.panic_timer <= 0.0
        {
            if let Some(target) = player_pos {
                let (dx, dz, dy) =
                    (self.pos.x - target.x, self.pos.z - target.z, (target.y - self.pos.y).abs());
                if dx * dx + dz * dz < 3.8 * 3.8 && dy < 2.5 {
                    self.target_yaw = dx.atan2(dz);
                    self.head_yaw_offset = 0.0;
                    self.ai_state_is_moving = true;
                    skittish_avoiding = true;
                }
            }
        }

        if self.kind == MobKind::Villager
            && !following_player
            && !returning_home
            && self.panic_timer <= 0.0
        {
            if let Some(target) = player_pos {
                let (dx, dz, dy) =
                    (target.x - self.pos.x, target.z - self.pos.z, (target.y - self.pos.y).abs());
                if dx * dx + dz * dz < 3.5 * 3.5 && dy < 2.5 {
                    self.target_yaw = dx.atan2(dz);
                    self.head_yaw_offset = 0.0;
                    self.ai_state_is_moving = false;
                }
            }
        }

        (following_player, returning_home, skittish_avoiding)
    }

    fn update_idle_or_panic_wander(&mut self, dt: f32, rng: &mut u32, player_pos: Option<Vec3>) {
        self.ai_timer -= dt;
        if self.ai_timer > 0.0 {
            return;
        }
        if self.panic_timer > 0.0 {
            self.ai_state_is_moving = true;
            if let Some(target) = player_pos {
                let base_yaw = (self.pos.x - target.x).atan2(self.pos.z - target.z);
                self.target_yaw = base_yaw + (next_rng_f32(rng) - 0.5) * 0.75;
            } else {
                self.target_yaw += (next_rng_f32(rng) - 0.5) * 1.6;
            }
            self.ai_timer = 0.3 + next_rng_f32(rng) * 0.35;
        } else if next_rng_f32(rng) < 0.45 {
            self.ai_state_is_moving = false;
            self.ai_timer = 2.0 + next_rng_f32(rng) * 3.5;
            self.head_yaw_offset = (next_rng_f32(rng) - 0.5) * 0.8;
        } else {
            self.ai_state_is_moving = true;
            self.target_yaw = next_rng_f32(rng) * PI * 2.0;
            self.ai_timer = 1.5 + next_rng_f32(rng) * 3.0;
            self.head_yaw_offset = (next_rng_f32(rng) - 0.5) * 0.2;
        }
    }

    fn advance_ground_locomotion(
        &mut self,
        dt: f32,
        world: &World,
        in_water: bool,
        pursuing_player: bool,
        skittish_avoiding: bool,
    ) {
        let mut navigation_blocked = false;
        if self.on_ground && self.ai_state_is_moving && self.eat_grass_timer <= 0.0 {
            if let Some(yaw) = self.navigation_yaw(world, self.target_yaw) {
                self.target_yaw = yaw;
            } else {
                navigation_blocked = true;
            }
        }

        let turn_speed = if self.panic_timer > 0.0 || pursuing_player { 10.0 } else { 4.0 };
        self.yaw += wrap_angle_delta(self.target_yaw - self.yaw) * (turn_speed * dt).min(1.0);

        let move_speed = if navigation_blocked || self.eat_grass_timer > 0.0 {
            0.0
        } else if self.panic_timer > 0.0 {
            5.4
        } else if pursuing_player {
            2.6
        } else if skittish_avoiding {
            2.4
        } else if self.ai_state_is_moving {
            1.4
        } else {
            0.0
        };

        if self.on_ground {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let accel = if self.ai_state_is_moving || pursuing_player { 12.0 } else { 16.0 };
            self.velocity.x += (forward.x * move_speed - self.velocity.x) * (accel * dt).min(1.0);
            self.velocity.z += (forward.z * move_speed - self.velocity.z) * (accel * dt).min(1.0);
        } else {
            self.velocity.x *= (1.0 - 2.0 * dt).max(0.0);
            self.velocity.z *= (1.0 - 2.0 * dt).max(0.0);
        }

        if in_water {
            self.velocity.y = (self.velocity.y + 12.0 * dt).min(1.8);
            self.velocity.x *= (1.0 - 4.0 * dt).max(0.0);
            self.velocity.z *= (1.0 - 4.0 * dt).max(0.0);
        } else {
            self.velocity.y -= MOB_GRAVITY * dt;
            let max_fall = if self.kind == MobKind::Chicken { -3.2 } else { -30.0 };
            if self.velocity.y < max_fall {
                self.velocity.y = max_fall;
            }
        }

        if self.on_ground && (self.ai_state_is_moving || self.panic_timer > 0.0 || pursuing_player) {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let check_dist = self.kind.half_width() + 0.35;
            let check_pos = Vec3::new(
                self.pos.x + forward.x * check_dist,
                self.pos.y + 0.4,
                self.pos.z + forward.z * check_dist,
            );
            if world.get_block(check_pos.floor_to_ivec()).is_solid() {
                let above_pos =
                    Vec3::new(check_pos.x, self.pos.y + self.kind.height() + 0.8, check_pos.z);
                if !world.get_block(above_pos.floor_to_ivec()).is_solid() {
                    self.velocity.y = 6.8;
                    self.on_ground = false;
                }
            }
        }

        let hit_wall = self.apply_movement(dt, world);
        if self.kind == MobKind::Spider && hit_wall && !self.is_dead {
            self.velocity.y = 3.8;
            self.on_ground = false;
        }

        let horizontal_speed = self.velocity.x.hypot(self.velocity.z);
        if self.on_ground && horizontal_speed > 0.05 {
            self.walk_phase += dt * (horizontal_speed * 4.5);
        } else if !self.on_ground {
            self.walk_phase += dt * if self.kind == MobKind::Chicken { 16.0 } else { 3.0 };
        } else {
            self.walk_phase *= (1.0 - 6.0 * dt).max(0.0);
        }
    }
}
