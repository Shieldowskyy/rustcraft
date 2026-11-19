//! Natural hostile (night surface / cave) and aquatic mob spawning rules.
//! Evaluates loaded world blocks around a player center and spawns mobs into `MobManager`.

use std::f32::consts::PI;

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::World;

use super::constants::{DESPAWN_DISTANCE_SQ, MAX_AQUATIC_NEARBY, MAX_HOSTILE_NEARBY};
use super::kind::MobKind;
use super::manager::MobManager;

#[inline]
fn next_rng_f32(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    (*state as f32) / (u32::MAX as f32)
}

#[inline]
fn pick_hostile_kind(roll: f32) -> MobKind {
    if roll < 0.35 {
        MobKind::Zombie
    } else if roll < 0.60 {
        MobKind::Skeleton
    } else if roll < 0.80 {
        MobKind::Reaper
    } else {
        MobKind::Spider
    }
}

#[inline]
fn pick_aquatic_kind(roll: f32) -> MobKind {
    if roll < 0.35 {
        MobKind::Cod
    } else if roll < 0.60 {
        MobKind::Salmon
    } else if roll < 0.75 {
        MobKind::TropicalFish
    } else if roll < 0.88 {
        MobKind::Pufferfish
    } else {
        MobKind::Dolphin
    }
}

impl MobManager {
    pub(super) fn try_natural_spawn(&mut self, world: &World, player_pos: Vec3, rng: &mut u32) {
        let is_night = world.sunlight_factor() < 0.50;
        let hostile_count = self
            .mobs
            .iter()
            .filter(|m| {
                let (dx, dz) = (m.pos.x - player_pos.x, m.pos.z - player_pos.z);
                m.kind.is_hostile() && !m.is_dead && dx * dx + dz * dz <= DESPAWN_DISTANCE_SQ
            })
            .count();

        if hostile_count >= MAX_HOSTILE_NEARBY {
            return;
        }
        if !is_night && next_rng_f32(rng) > 0.40 {
            return;
        }

        let attempts = if is_night { 5 } else { 2 };
        for _ in 0..attempts {
            let angle = next_rng_f32(rng) * PI * 2.0;
            let dist = 14.0 + next_rng_f32(rng) * 24.0;
            let target_x = (player_pos.x + angle.cos() * dist).floor() as i32;
            let target_z = (player_pos.z + angle.sin() * dist).floor() as i32;

            if is_night {
                if let Some(sy) = find_night_surface_y(world, target_x, target_z) {
                    let kind = pick_hostile_kind(next_rng_f32(rng));
                    let spawn_pos = Vec3::new(target_x as f32 + 0.5, sy as f32, target_z as f32 + 0.5);
                    self.spawn_mob(kind, spawn_pos);
                    crate::log_info!(
                        "MOB",
                        "Night surface spawn: {:?} at ({:.1}, {:.1}, {:.1})",
                        kind,
                        spawn_pos.x,
                        spawn_pos.y,
                        spawn_pos.z
                    );
                    return;
                }
            }

            if let Some(cy) = find_cave_spawn_y(world, target_x, target_z, player_pos.y.floor() as i32) {
                let kind = pick_hostile_kind(next_rng_f32(rng));
                let spawn_pos = Vec3::new(target_x as f32 + 0.5, cy as f32, target_z as f32 + 0.5);
                self.spawn_mob(kind, spawn_pos);
                return;
            }

            if let Some((bot, top)) = find_water_column(world, target_x, target_z) {
                let aquatic_count = self
                    .mobs
                    .iter()
                    .filter(|mob| {
                        let (dx, dz) = (mob.pos.x - player_pos.x, mob.pos.z - player_pos.z);
                        mob.kind.is_aquatic() && !mob.is_dead && dx * dx + dz * dz <= DESPAWN_DISTANCE_SQ
                    })
                    .count();
                if aquatic_count < MAX_AQUATIC_NEARBY {
                    let spawn_y = bot as f32 + 0.5 + next_rng_f32(rng) * ((top - bot) as f32 - 0.5);
                    let kind = pick_aquatic_kind(next_rng_f32(rng));
                    let spawn_pos = Vec3::new(target_x as f32 + 0.5, spawn_y, target_z as f32 + 0.5);
                    self.spawn_mob(kind, spawn_pos);
                    crate::log_info!(
                        "MOB",
                        "Aquatic spawn: {:?} at ({:.1}, {:.1}, {:.1})",
                        kind,
                        spawn_pos.x,
                        spawn_pos.y,
                        spawn_pos.z
                    );
                    return;
                }
            }
        }
    }
}

fn find_night_surface_y(world: &World, x: i32, z: i32) -> Option<i32> {
    for y in (1..63).rev() {
        let block = world.get_block(IVec3::new(x, y, z));
        if block.is_solid()
            && !matches!(block, BlockId::Leaves | BlockId::SnowLeaves | BlockId::Lava | BlockId::Water)
        {
            let above1 = world.get_block(IVec3::new(x, y + 1, z));
            let above2 = world.get_block(IVec3::new(x, y + 2, z));
            if above1 == BlockId::Air && above2 == BlockId::Air {
                return Some(y + 1);
            }
        }
    }
    None
}

fn find_cave_spawn_y(world: &World, x: i32, z: i32, player_y: i32) -> Option<i32> {
    let y_min = (player_y - 12).clamp(2, 58);
    let y_max = (player_y + 8).clamp(3, 60);
    for y in y_min..=y_max {
        let floor_block = world.get_block(IVec3::new(x, y, z));
        if floor_block.is_solid() && floor_block != BlockId::Lava && floor_block != BlockId::Water {
            let above1 = world.get_block(IVec3::new(x, y + 1, z));
            let above2 = world.get_block(IVec3::new(x, y + 2, z));
            if above1 == BlockId::Air && above2 == BlockId::Air {
                if ((y + 3)..=63).any(|cy| world.get_block(IVec3::new(x, cy, z)).is_solid()) {
                    return Some(y + 1);
                }
            }
        }
    }
    None
}

fn find_water_column(world: &World, x: i32, z: i32) -> Option<(i32, i32)> {
    for y in (2..=21).rev() {
        if world.get_block(IVec3::new(x, y, z)) == BlockId::Water {
            let mut bot_y = y;
            for by in (1..y).rev() {
                let b = world.get_block(IVec3::new(x, by, z));
                if !matches!(b, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
                    bot_y = by + 1;
                    break;
                }
            }
            if y - bot_y >= 2 {
                return Some((bot_y, y));
            }
            return None;
        }
    }
    None
}
