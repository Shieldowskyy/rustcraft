use std::f32::consts::PI;
use crate::core::math::{IVec3, Vec3};
use crate::world::block::BlockId;
use crate::world::World;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MobKind {
    Pig,
    Cow,
    Chicken,
}

impl MobKind {
    pub fn max_health(&self) -> f32 {
        match self {
            MobKind::Pig => 10.0,
            MobKind::Cow => 10.0,
            MobKind::Chicken => 4.0,
        }
    }

    pub fn half_width(&self) -> f32 {
        match self {
            MobKind::Pig => 0.4,
            MobKind::Cow => 0.45,
            MobKind::Chicken => 0.25,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            MobKind::Pig => 0.85,
            MobKind::Cow => 1.35,
            MobKind::Chicken => 0.70,
        }
    }

    #[allow(dead_code)]
    pub fn is_hostile(&self) -> bool {
        match self {
            MobKind::Pig | MobKind::Cow | MobKind::Chicken => false,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            MobKind::Pig => "Pig",
            MobKind::Cow => "Cow",
            MobKind::Chicken => "Chicken",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mob {
    #[allow(dead_code)]
    pub id: u32,
    pub kind: MobKind,
    pub pos: Vec3,          // Feet position
    pub velocity: Vec3,
    pub yaw: f32,           // Facing angle in radians
    pub head_yaw_offset: f32, // Head relative look angle
    pub health: f32,
    pub hurt_timer: f32,    // Flashes red when > 0.0
    pub death_timer: f32,   // Ticks up when dying (0.0 to 0.8s)
    pub is_dead: bool,
    pub walk_phase: f32,
    pub on_ground: bool,
    pub ai_timer: f32,
    pub ai_state_is_moving: bool,
    pub target_yaw: f32,
    pub panic_timer: f32,
    pub despawn_pending: bool,
}

impl Mob {
    pub fn new(id: u32, kind: MobKind, pos: Vec3) -> Self {
        let max_hp = kind.max_health();
        Mob {
            id,
            kind,
            pos,
            velocity: Vec3::zero(),
            yaw: 0.0,
            head_yaw_offset: 0.0,
            health: max_hp,
            hurt_timer: 0.0,
            death_timer: 0.0,
            is_dead: false,
            walk_phase: 0.0,
            on_ground: false,
            ai_timer: 1.0,
            ai_state_is_moving: false,
            target_yaw: 0.0,
            panic_timer: 0.0,
            despawn_pending: false,
        }
    }

    pub fn aabb(&self) -> (Vec3, Vec3) {
        let hw = self.kind.half_width();
        let h = self.kind.height();
        (
            Vec3::new(self.pos.x - hw, self.pos.y, self.pos.z - hw),
            Vec3::new(self.pos.x + hw, self.pos.y + h, self.pos.z + hw),
        )
    }

    /// Center of mass / torso for particle effects and targeting
    pub fn center_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + self.kind.height() * 0.5, self.pos.z)
    }

    pub fn take_damage(&mut self, damage: f32, knockback_dir: Vec3) {
        if self.is_dead {
            return;
        }
        self.health -= damage;
        self.hurt_timer = 0.35;
        self.panic_timer = 3.0;

        let kb = if knockback_dir.length_sq() > 1e-4 {
            knockback_dir.normalized()
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        self.velocity.x += kb.x * 5.0;
        self.velocity.z += kb.z * 5.0;
        self.velocity.y = 3.8;
        self.on_ground = false;

        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_dead = true;
            self.death_timer = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, rng: &mut u32) {
        let rand_f32 = |state: &mut u32| -> f32 {
            *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (*state as f32) / (u32::MAX as f32)
        };

        if self.hurt_timer > 0.0 {
            self.hurt_timer = (self.hurt_timer - dt).max(0.0);
        }

        if self.is_dead {
            self.death_timer += dt;
            self.velocity.y -= 22.0 * dt;
            self.apply_movement(dt, world);
            if self.death_timer >= 0.8 {
                self.despawn_pending = true;
            }
            return;
        }

        let (min, max) = self.aabb();
        let min_x = min.x.floor() as i32;
        let max_x = max.x.floor() as i32;
        let min_y = min.y.floor() as i32;
        let max_y = max.y.floor() as i32;
        let min_z = min.z.floor() as i32;
        let max_z = max.z.floor() as i32;

        let mut in_lava = false;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.get_block(IVec3::new(x, y, z)) == BlockId::Lava {
                        in_lava = true;
                        break;
                    }
                }
                if in_lava { break; }
            }
            if in_lava { break; }
        }

        if in_lava {
            self.velocity.x *= 0.8;
            self.velocity.z *= 0.8;
            if self.velocity.y < -3.0 {
                self.velocity.y = -3.0;
            }

            if self.hurt_timer <= 0.0 {
                let rx = (rand_f32(rng) - 0.5) * 0.4;
                let rz = (rand_f32(rng) - 0.5) * 0.4;
                self.take_damage(4.0, Vec3::new(rx, 0.8, rz));
            }
        }

        if self.panic_timer > 0.0 {
            self.panic_timer = (self.panic_timer - dt).max(0.0);
        }

        self.ai_timer -= dt;
        if self.ai_timer <= 0.0 {
            if self.panic_timer > 0.0 {
                self.ai_state_is_moving = true;
                self.target_yaw += (rand_f32(rng) - 0.5) * 1.6;
                self.ai_timer = 0.4 + rand_f32(rng) * 0.5;
            } else {
                let roll = rand_f32(rng);
                if roll < 0.45 {
                    self.ai_state_is_moving = false;
                    self.ai_timer = 2.0 + rand_f32(rng) * 3.5;
                    self.head_yaw_offset = (rand_f32(rng) - 0.5) * 0.8;
                } else {
                    self.ai_state_is_moving = true;
                    self.target_yaw = rand_f32(rng) * PI * 2.0;
                    self.ai_timer = 1.5 + rand_f32(rng) * 3.0;
                    self.head_yaw_offset = (rand_f32(rng) - 0.5) * 0.2;
                }
            }
        }

        let mut diff = self.target_yaw - self.yaw;
        while diff > PI { diff -= PI * 2.0; }
        while diff < -PI { diff += PI * 2.0; }
        let turn_speed = if self.panic_timer > 0.0 { 10.0 } else { 4.0 };
        self.yaw += diff * (turn_speed * dt).min(1.0);

        let move_speed = if self.panic_timer > 0.0 {
            3.8
        } else if self.ai_state_is_moving {
            1.4
        } else {
            0.0
        };

        if self.on_ground {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let target_vel_x = forward.x * move_speed;
            let target_vel_z = forward.z * move_speed;

            let accel = if self.ai_state_is_moving { 12.0 } else { 16.0 };
            self.velocity.x += (target_vel_x - self.velocity.x) * (accel * dt).min(1.0);
            self.velocity.z += (target_vel_z - self.velocity.z) * (accel * dt).min(1.0);
        } else {
            self.velocity.x *= (1.0 - 2.0 * dt).max(0.0);
            self.velocity.z *= (1.0 - 2.0 * dt).max(0.0);
        }

        self.velocity.y -= 22.0 * dt;
        let max_fall_speed = if self.kind == MobKind::Chicken { -3.2 } else { -30.0 };
        if self.velocity.y < max_fall_speed {
            self.velocity.y = max_fall_speed;
        }

        if self.on_ground && (self.ai_state_is_moving || self.panic_timer > 0.0) {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let hw = self.kind.half_width();
            let check_dist = hw + 0.35;
            let check_pos = Vec3::new(self.pos.x + forward.x * check_dist, self.pos.y + 0.4, self.pos.z + forward.z * check_dist);
            let block_ahead = world.get_block(check_pos.floor_to_ivec());
            if block_ahead.is_solid() {
                let above_pos = Vec3::new(check_pos.x, self.pos.y + self.kind.height() + 0.8, check_pos.z);
                let block_above = world.get_block(above_pos.floor_to_ivec());
                if !block_above.is_solid() {
                    self.velocity.y = 6.8;
                    self.on_ground = false;
                }
            }
        }

        let _hit_wall = self.apply_movement(dt, world);

        let horizontal_speed = (self.velocity.x * self.velocity.x + self.velocity.z * self.velocity.z).sqrt();

        if self.on_ground && horizontal_speed > 0.05 {
            self.walk_phase += dt * (horizontal_speed * 4.5);
        } else if !self.on_ground {
            if self.kind == MobKind::Chicken {
                self.walk_phase += dt * 16.0; // Rapid wing flapping when in air
            } else {
                self.walk_phase += dt * 3.0;
            }
        } else {
            self.walk_phase *= (1.0 - 6.0 * dt).max(0.0);
        }
    }

    fn apply_movement(&mut self, dt: f32, world: &World) -> bool {
        let hw = self.kind.half_width();
        let h = self.kind.height();

        let aabb_at = |p: Vec3| -> (Vec3, Vec3) {
            (
                Vec3::new(p.x - hw, p.y, p.z - hw),
                Vec3::new(p.x + hw, p.y + h, p.z + hw),
            )
        };

        let mut hit_wall = false;
        let step_height = 1.05;

        // X movement
        let dx = self.velocity.x * dt;
        if dx.abs() > 1e-6 {
            let test_x = Vec3::new(self.pos.x + dx, self.pos.y, self.pos.z);
            let (min_x, max_x) = aabb_at(test_x);
            if world.box_collides(min_x, max_x) {
                let (head_min, head_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z));
                let (step_min, step_max) = aabb_at(Vec3::new(self.pos.x + dx, self.pos.y + step_height, self.pos.z));
                if self.on_ground && !world.box_collides(head_min, head_max) && !world.box_collides(step_min, step_max) {
                    self.pos.x += dx;
                    self.pos.y += 1.0;
                } else {
                    self.velocity.x = 0.0;
                    hit_wall = true;
                }
            } else {
                self.pos.x = test_x.x;
            }
        }

        // Z movement
        let dz = self.velocity.z * dt;
        if dz.abs() > 1e-6 {
            let test_z = Vec3::new(self.pos.x, self.pos.y, self.pos.z + dz);
            let (min_z, max_z) = aabb_at(test_z);
            if world.box_collides(min_z, max_z) {
                let (head_min, head_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z));
                let (step_min, step_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z + dz));
                if self.on_ground && !world.box_collides(head_min, head_max) && !world.box_collides(step_min, step_max) {
                    self.pos.z += dz;
                    self.pos.y += 1.0;
                } else {
                    self.velocity.z = 0.0;
                    hit_wall = true;
                }
            } else {
                self.pos.z = test_z.z;
            }
        }

        // Y movement
        self.on_ground = false;
        let dy = self.velocity.y * dt;
        let test_y = Vec3::new(self.pos.x, self.pos.y + dy, self.pos.z);
        let (min_y, max_y) = aabb_at(test_y);
        if world.box_collides(min_y, max_y) {
            if self.velocity.y <= 0.0 {
                self.on_ground = true;
            }
            self.velocity.y = 0.0;
        } else {
            self.pos.y = test_y.y;
        }

        hit_wall
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MobDrop {
    pub pos: Vec3,
    pub block: BlockId,
    pub count: u8,
}

pub struct MobManager {
    pub mobs: Vec<Mob>,
    next_mob_id: u32,
    spawn_timer: f32,
    rng_state: u32,
    pub dying_poofs: Vec<Vec3>,
    pub dying_drops: Vec<MobDrop>,
}

impl MobManager {
    pub fn new() -> Self {
        MobManager {
            mobs: Vec::new(),
            next_mob_id: 1,
            spawn_timer: 1.0,
            rng_state: 987654321,
            dying_poofs: Vec::new(),
            dying_drops: Vec::new(),
        }
    }

    pub fn spawn_mob(&mut self, kind: MobKind, pos: Vec3) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id = self.next_mob_id.wrapping_add(1);
        self.mobs.push(Mob::new(id, kind, pos));
        id
    }

    pub fn update(&mut self, dt: f32, world: &World, player_pos: Vec3) {
        let mut rng = self.rng_state;

        for mob in &mut self.mobs {
            mob.update(dt, world, &mut rng);

            let dist_sq = (mob.pos.x - player_pos.x) * (mob.pos.x - player_pos.x)
                + (mob.pos.z - player_pos.z) * (mob.pos.z - player_pos.z);
            if dist_sq > 75.0 * 75.0 && !mob.is_dead {
                mob.despawn_pending = true;
            }
        }

        self.dying_poofs.clear();
        self.dying_drops.clear();
        self.mobs.retain(|m| {
            if m.despawn_pending {
                if m.is_dead {
                    self.dying_poofs.push(m.center_pos());
                    match m.kind {
                        MobKind::Pig => {
                            let count = 1 + (m.id % 3) as u8; // 1-3 meat drops
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawPorkchop,
                                count,
                            });
                        }
                        MobKind::Cow => {
                            let count = 1 + (m.id % 3) as u8; // 1-3 meat drops
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawBeef,
                                count,
                            });
                        }
                        MobKind::Chicken => {
                            let meat_count = 1 + (m.id % 2) as u8; // 1-2 raw chicken
                            let feather_count = 1 + ((m.id >> 2) % 3) as u8; // 1-3 feathers
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawChicken,
                                count: meat_count,
                            });
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos() + Vec3::new(0.1, 0.1, 0.1),
                                block: BlockId::Feather,
                                count: feather_count,
                            });
                        }
                    }
                }
                false
            } else {
                true
            }
        });

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = 4.0;
            self.try_natural_spawn(world, player_pos, &mut rng);
        }

        self.rng_state = rng;
    }

    pub fn apply_explosion(&mut self, explosion_pos: Vec3, radius: f32, max_damage: f32) {
        for mob in &mut self.mobs {
            if mob.is_dead {
                continue;
            }
            let center = mob.center_pos();
            let to_mob = center - explosion_pos;
            let dist = to_mob.length();
            if dist <= radius {
                let factor = (1.0 - dist / radius).clamp(0.0, 1.0);
                let damage = (factor * max_damage).max(4.0);
                let kb_dir = if dist > 0.01 {
                    to_mob.normalized()
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                mob.take_damage(damage, kb_dir);
                let kb_strength = factor * 12.0;
                mob.velocity.x += kb_dir.x * kb_strength;
                mob.velocity.y = (mob.velocity.y + kb_dir.y * kb_strength + 4.0).min(14.0);
                mob.velocity.z += kb_dir.z * kb_strength;
                mob.on_ground = false;
            }
        }
    }

    fn try_natural_spawn(&mut self, world: &World, player_pos: Vec3, rng: &mut u32) {
        const MAX_NEARBY_MOBS: usize = 8;
        if self.mobs.len() >= MAX_NEARBY_MOBS {
            return;
        }

        let rand_f32 = |state: &mut u32| -> f32 {
            *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (*state as f32) / (u32::MAX as f32)
        };

        if rand_f32(rng) > 0.40 {
            return;
        }

        let angle = rand_f32(rng) * PI * 2.0;
        let dist = 18.0 + rand_f32(rng) * 24.0;
        let target_x = (player_pos.x + angle.cos() * dist).floor() as i32;
        let target_z = (player_pos.z + angle.sin() * dist).floor() as i32;

        let mut surface_y = None;
        for y in (1..62).rev() {
            let block = world.get_block(IVec3::new(target_x, y, target_z));
            if block == BlockId::Grass {
                let above1 = world.get_block(IVec3::new(target_x, y + 1, target_z));
                let above2 = world.get_block(IVec3::new(target_x, y + 2, target_z));
                if above1 == BlockId::Air && above2 == BlockId::Air {
                    surface_y = Some(y + 1);
                    break;
                }
            } else if block != BlockId::Air && block != BlockId::Leaves && block != BlockId::SnowLeaves {
                break;
            }
        }

        if let Some(sy) = surface_y {
            let roll = rand_f32(rng);
            let kind = if roll < 0.33 {
                MobKind::Pig
            } else if roll < 0.66 {
                MobKind::Cow
            } else {
                MobKind::Chicken
            };

            let spawn_pos = Vec3::new(target_x as f32 + 0.5, sy as f32, target_z as f32 + 0.5);
            self.spawn_mob(kind, spawn_pos);
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
                if t >= 0.0 && t <= max_reach {
                    match closest_hit {
                        Some((_, closest_t)) => {
                            if t < closest_t {
                                closest_hit = Some((idx, t));
                            }
                        }
                        None => {
                            closest_hit = Some((idx, t));
                        }
                    }
                }
            }
        }

        closest_hit
    }
}

fn ray_aabb_intersect(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = (min.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    let mut t_max = (max.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    if t_min > t_max { std::mem::swap(&mut t_min, &mut t_max); }

    let mut ty_min = (min.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    let mut ty_max = (max.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    if ty_min > ty_max { std::mem::swap(&mut ty_min, &mut ty_max); }

    if (t_min > ty_max) || (ty_min > t_max) {
        return None;
    }
    if ty_min > t_min { t_min = ty_min; }
    if ty_max < t_max { t_max = ty_max; }

    let mut tz_min = (min.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    let mut tz_max = (max.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    if tz_min > tz_max { std::mem::swap(&mut tz_min, &mut tz_max); }

    if (t_min > tz_max) || (tz_min > t_max) {
        return None;
    }
    if tz_min > t_min { t_min = tz_min; }
    if tz_max < t_max { t_max = tz_max; }

    if t_max < 0.0 {
        return None;
    }

    Some(if t_min >= 0.0 { t_min } else { t_max })
}
