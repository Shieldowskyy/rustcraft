//! Mob environmental hazards (lava, undead sunlight burning) and aquatic swimming/flopping AI.
//! Mutates `Mob` health, velocity, timers, and orientation inside `World`.

use std::f32::consts::PI;

use crate::audio::SoundId;
use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::World;

use super::ai::{next_rng_f32, wrap_angle_delta};
use super::constants::MOB_GRAVITY;
use super::kind::MobKind;
use super::state::Mob;

impl Mob {
    pub(super) fn apply_environment_hazards(&mut self, world: &World, rng: &mut u32) -> bool {
        let (min, max) = self.aabb();
        let (min_x, max_x) = (min.x.floor() as i32, max.x.floor() as i32);
        let (min_y, max_y) = (min.y.floor() as i32, max.y.floor() as i32);
        let (min_z, max_z) = (min.z.floor() as i32, max.z.floor() as i32);

        let mut in_lava = false;
        let mut in_water = false;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let b = world.get_block(IVec3::new(x, y, z));
                    if b == BlockId::Lava {
                        in_lava = true;
                    } else if matches!(b, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
                        in_water = true;
                    }
                }
            }
        }

        if in_lava {
            self.velocity.x *= 0.8;
            self.velocity.z *= 0.8;
            if self.velocity.y < -3.0 {
                self.velocity.y = -3.0;
            }
            if self.hurt_timer <= 0.0 {
                let rx = (next_rng_f32(rng) - 0.5) * 0.4;
                let rz = (next_rng_f32(rng) - 0.5) * 0.4;
                self.take_damage(4.0, Vec3::new(rx, 0.8, rz));
            }
        }

        if (self.kind == MobKind::Zombie || self.kind == MobKind::Skeleton)
            && world.sunlight_factor() > 0.65
        {
            let ix = self.pos.x.floor() as i32;
            let iy = (self.pos.y + self.kind.height()).ceil() as i32;
            let iz = self.pos.z.floor() as i32;
            let covered = (iy..=64).any(|y| {
                let blk = world.get_block(IVec3::new(ix, y, iz));
                blk.is_solid() || blk == BlockId::Leaves || blk == BlockId::SnowLeaves
            });
            if !covered && self.hurt_timer <= 0.0 {
                self.take_damage(2.0, Vec3::new(0.0, 0.5, 0.0));
            }
        }

        in_water
    }

    pub(super) fn update_aquatic_in_water(
        &mut self,
        dt: f32,
        world: &World,
        rng: &mut u32,
        player_pos: Option<Vec3>,
    ) {
        self.out_of_water_timer = 0.0;
        self.ai_timer -= dt;
        if self.ai_timer <= 0.0 {
            if next_rng_f32(rng) < 0.22 {
                self.ai_state_is_moving = false;
                self.ai_timer = 1.0 + next_rng_f32(rng) * 2.0;
            } else {
                self.ai_state_is_moving = true;
                self.target_yaw = next_rng_f32(rng) * PI * 2.0;
                self.ai_timer = 2.0 + next_rng_f32(rng) * 4.0;
            }
        }

        if let Some(target) = player_pos {
            let (dx, dy, dz) = (target.x - self.pos.x, target.y - self.pos.y, target.z - self.pos.z);
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if self.kind == MobKind::Pufferfish && dist_sq < 3.2 * 3.2 {
                self.is_puffed = true;
                self.puff_timer = 2.5;
            }
            if self.kind == MobKind::Dolphin && dist_sq < 14.0 * 14.0 && dist_sq > 2.5 * 2.5 {
                self.target_yaw = dx.atan2(dz) + (next_rng_f32(rng) - 0.5) * 0.4;
                self.ai_state_is_moving = true;
            }
        }

        if self.puff_timer > 0.0 {
            self.puff_timer -= dt;
            if self.puff_timer <= 0.0 {
                self.is_puffed = false;
            }
        }

        if self.kind == MobKind::Dolphin {
            self.jump_timer -= dt;
            let above_pos = (self.pos + Vec3::new(0.0, 1.2, 0.0)).floor_to_ivec();
            if self.jump_timer <= 0.0
                && world.get_block(above_pos) == BlockId::Air
                && self.ai_state_is_moving
            {
                self.jump_timer = 4.5 + next_rng_f32(rng) * 5.0;
                self.velocity.y = 5.2;
                self.pending_sound = Some(SoundId::DolphinJump);
            }
        }

        self.yaw += wrap_angle_delta(self.target_yaw - self.yaw) * (4.5 * dt).min(1.0);

        let swim_speed = match self.kind {
            MobKind::Dolphin => if self.ai_state_is_moving { 4.2 } else { 1.5 },
            MobKind::Salmon => if self.ai_state_is_moving { 2.4 } else { 0.9 },
            _ => if self.ai_state_is_moving { 1.8 } else { 0.6 },
        };
        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        let surface_above = world.get_block((self.pos + Vec3::new(0.0, 0.6, 0.0)).floor_to_ivec());
        let floor_below = world.get_block((self.pos - Vec3::new(0.0, 0.4, 0.0)).floor_to_ivec());
        let target_vy = if floor_below.is_solid() {
            1.2
        } else if surface_above == BlockId::Air && self.velocity.y <= 0.5 {
            -0.8
        } else {
            (self.id.wrapping_mul(101) % 5) as f32 * 0.3 - 0.6
        };

        self.velocity.x += (forward.x * swim_speed - self.velocity.x) * (6.0 * dt).min(1.0);
        self.velocity.z += (forward.z * swim_speed - self.velocity.z) * (6.0 * dt).min(1.0);
        if self.velocity.y < 3.0 {
            self.velocity.y += (target_vy - self.velocity.y) * (4.0 * dt).min(1.0);
        } else {
            self.velocity.y -= 18.0 * dt;
        }

        let current_pitch = (-self.velocity.y * 0.22).clamp(-0.55, 0.55);
        self.swim_pitch += (current_pitch - self.swim_pitch) * (6.0 * dt).min(1.0);

        let _ = self.apply_movement(dt, world);
        self.walk_phase += dt * (self.velocity.x.hypot(self.velocity.z) * 5.0 + 3.0);
    }

    pub(super) fn update_aquatic_on_land(&mut self, dt: f32, world: &World, rng: &mut u32) {
        self.out_of_water_timer += dt;
        self.velocity.y -= MOB_GRAVITY * dt;
        self.velocity.x *= (1.0 - 5.0 * dt).max(0.0);
        self.velocity.z *= (1.0 - 5.0 * dt).max(0.0);

        self.walk_phase += dt * 16.0;
        if self.on_ground {
            self.ai_timer -= dt;
            if self.ai_timer <= 0.0 {
                self.ai_timer = 0.5 + next_rng_f32(rng) * 0.8;
                self.velocity.y = 3.0 + next_rng_f32(rng) * 1.5;
                self.velocity.x = (next_rng_f32(rng) - 0.5) * 2.8;
                self.velocity.z = (next_rng_f32(rng) - 0.5) * 2.8;
                self.on_ground = false;
                self.pending_sound = Some(SoundId::FishFlop);
            }
        }

        if self.out_of_water_timer >= 12.0 && self.hurt_timer <= 0.0 {
            self.take_damage(2.0, Vec3::new(0.0, 1.0, 0.0));
        }
        let _ = self.apply_movement(dt, world);
    }
}
