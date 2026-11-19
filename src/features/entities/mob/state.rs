//! Core `Mob` struct, constructor, AABB queries, damage/knockback, and stepped voxel movement.
//! Mutates `Mob` physical state and triggers `World::box_collides` checks.

use std::f32::consts::PI;

use crate::audio::SoundId;
use crate::core::math::{IVec3, Vec3};
use crate::features::world::World;
use crate::features::player::trading::{TradeOffer, VillagerProfession};

use super::kind::MobKind;

#[derive(Clone, Debug)]
pub struct Mob {
    #[allow(dead_code)]
    pub id: u32,
    pub kind: MobKind,
    pub pos: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub head_yaw_offset: f32,
    pub health: f32,
    pub hurt_timer: f32,
    pub death_timer: f32,
    pub is_dead: bool,
    pub walk_phase: f32,
    pub on_ground: bool,
    pub ai_timer: f32,
    pub ai_state_is_moving: bool,
    pub target_yaw: f32,
    pub panic_timer: f32,
    pub despawn_pending: bool,
    pub ambient_sound_timer: f32,
    pub pending_sound: Option<SoundId>,
    pub fuse: f32,
    pub is_ignited: bool,
    pub in_love: f32,
    pub breed_cooldown: f32,
    pub is_baby: bool,
    pub age: f32,
    pub love_particle_timer: f32,
    pub wool_color: u8,
    pub is_sheared: bool,
    pub eat_grass_timer: f32,
    pub is_puffed: bool,
    pub puff_timer: f32,
    pub swim_pitch: f32,
    pub jump_timer: f32,
    pub out_of_water_timer: f32,
    pub fish_variant: u8,
    pub villager_profession: u8,
    pub trades: Vec<TradeOffer>,
    pub attack_cooldown: f32,
    pub home_pos: Option<Vec3>,
    pub player_anger_timer: f32,
}

impl Mob {
    pub fn new(id: u32, kind: MobKind, pos: Vec3) -> Self {
        let max_hp = kind.max_health();
        let initial_ambient = 3.0 + ((id.wrapping_mul(2654435761) % 1000) as f32 / 1000.0) * 8.0;
        let wool_color = if kind == MobKind::Sheep {
            let roll = (id.wrapping_mul(1664525) % 100) as u8;
            if roll < 80 {
                0
            } else if roll < 85 {
                8
            } else if roll < 90 {
                7
            } else if roll < 95 {
                15
            } else if roll < 98 {
                12
            } else {
                6
            }
        } else {
            0
        };
        let villager_profession = if kind == MobKind::Villager { (id % 5) as u8 } else { 0 };
        let trades = if kind == MobKind::Villager {
            VillagerProfession::from_u8(villager_profession).default_trades()
        } else {
            Vec::new()
        };
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
            ambient_sound_timer: initial_ambient,
            pending_sound: None,
            fuse: 0.0,
            is_ignited: false,
            in_love: 0.0,
            breed_cooldown: 0.0,
            is_baby: false,
            age: 0.0,
            love_particle_timer: 0.0,
            wool_color,
            is_sheared: false,
            eat_grass_timer: 0.0,
            is_puffed: false,
            puff_timer: 0.0,
            swim_pitch: 0.0,
            jump_timer: 3.0 + ((id % 7) as f32),
            out_of_water_timer: 0.0,
            fish_variant: (id % 4) as u8,
            villager_profession,
            trades,
            attack_cooldown: 0.0,
            home_pos: (kind == MobKind::Villager).then_some(pos),
            player_anger_timer: 0.0,
        }
    }

    pub fn aabb(&self) -> (Vec3, Vec3) {
        let scale = if self.is_baby { 0.5 } else { 1.0 };
        let (hw, h) = if self.kind == MobKind::Pufferfish && self.is_puffed {
            (0.46 * scale, 0.65 * scale)
        } else {
            (self.kind.half_width() * scale, self.kind.height() * scale)
        };
        (
            Vec3::new(self.pos.x - hw, self.pos.y, self.pos.z - hw),
            Vec3::new(self.pos.x + hw, self.pos.y + h, self.pos.z + hw),
        )
    }

    pub fn intersects_block(&self, block_pos: IVec3) -> bool {
        let (min, max) = self.aabb();
        min.x < block_pos.x as f32 + 1.0
            && max.x > block_pos.x as f32
            && min.y < block_pos.y as f32 + 1.0
            && max.y > block_pos.y as f32
            && min.z < block_pos.z as f32 + 1.0
            && max.z > block_pos.z as f32
    }

    /// Center of mass / torso for particle effects and targeting.
    pub fn center_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + self.kind.height() * 0.5, self.pos.z)
    }

    pub(super) fn navigation_yaw(&self, world: &World, desired_yaw: f32) -> Option<f32> {
        let half_width = self.kind.half_width() * if self.is_baby { 0.5 } else { 1.0 };
        let height = self.kind.height() * if self.is_baby { 0.5 } else { 1.0 };
        let look_ahead = (half_width + 0.55).max(0.9);
        let offsets = [0.0, 0.55, -0.55, 1.1, -1.1, 1.65, -1.65, 2.2, -2.2, PI];
        let mut best: Option<(f32, f32)> = None;

        for offset in offsets {
            let yaw = desired_yaw + offset;
            let next_x = self.pos.x + yaw.sin() * look_ahead;
            let next_z = self.pos.z + yaw.cos() * look_ahead;
            let min = Vec3::new(next_x - half_width, self.pos.y, next_z - half_width);
            let max = Vec3::new(next_x + half_width, self.pos.y + height, next_z + half_width);
            if world.box_collides(min, max) {
                continue;
            }

            let support_y = (self.pos.y - 0.1).floor() as i32;
            let support = [
                (next_x - half_width + 0.05, next_z - half_width + 0.05),
                (next_x + half_width - 0.05, next_z - half_width + 0.05),
                (next_x - half_width + 0.05, next_z + half_width - 0.05),
                (next_x + half_width - 0.05, next_z + half_width - 0.05),
            ];
            if !support.iter().any(|&(x, z)| {
                world.get_block(IVec3::new(x.floor() as i32, support_y, z.floor() as i32)).is_solid()
            }) {
                continue;
            }

            let score = offset.cos();
            if best.map_or(true, |(best_score, _)| score > best_score) {
                best = Some((score, yaw));
            }
        }

        best.map(|(_, yaw)| yaw)
    }

    pub fn take_damage(&mut self, damage: f32, knockback_dir: Vec3) {
        if self.is_dead {
            return;
        }
        self.health -= damage;
        self.hurt_timer = 0.35;
        self.panic_timer = 5.0;

        let kb = if knockback_dir.length_sq() > 1e-4 {
            knockback_dir.normalized()
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let (kb_horiz, kb_vert) =
            if self.kind == MobKind::IronGolem { (0.4, 0.4) } else { (2.2, 1.6) };
        self.velocity.x += kb.x * kb_horiz;
        self.velocity.z += kb.z * kb_horiz;
        self.velocity.y = kb_vert;
        self.on_ground = false;

        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_dead = true;
            self.death_timer = 0.0;
            self.pending_sound = Some(self.kind.death_sound());
        } else {
            self.pending_sound = Some(self.kind.hurt_sound());
        }
    }

    pub(super) fn apply_movement(&mut self, dt: f32, world: &World) -> bool {
        let hw = self.kind.half_width();
        let h = self.kind.height();
        let aabb_at = |p: Vec3| -> (Vec3, Vec3) {
            (Vec3::new(p.x - hw, p.y, p.z - hw), Vec3::new(p.x + hw, p.y + h, p.z + hw))
        };

        let mut hit_wall = false;
        let step_height = 1.05;

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
