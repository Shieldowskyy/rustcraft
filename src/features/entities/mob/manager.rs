//! Swarm coordinator (`MobManager`): lifecycle, per-player tick dispatch, network snapshots, and grazing.
//! Combat, breeding, explosions, and raycasting live in `combat.rs`; natural spawning in `spawning.rs`.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::World;

use super::super::item::ArrowEntity;
use super::combat::try_skeleton_shoot;
use super::constants::{
    DESPAWN_DISTANCE_SQ, HOSTILE_TARGET_RANGE_SQ, HOSTILE_TARGET_Y_RANGE, MOB_SNAPSHOT_CAP,
    REAPER_FUSE_DURATION,
};
use super::drops::collect_mob_drops;
use super::kind::MobKind;
use super::state::Mob;

#[derive(Clone, Copy, Debug)]
pub struct MobPlayerContext {
    pub position: Vec3,
    pub held_item: Option<BlockId>,
    pub is_creative: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MobDrop {
    pub pos: Vec3,
    pub block: BlockId,
    pub count: u8,
}

pub struct MobManager {
    pub mobs: Vec<Mob>,
    pub(super) next_mob_id: u32,
    spawn_timer: f32,
    spawn_center_cursor: usize,
    pub rng_state: u32,
    pub dying_poofs: Vec<Vec3>,
    pub dying_drops: Vec<MobDrop>,
    pub pending_explosions: Vec<Vec3>,
    pub breeding_hearts: Vec<Vec3>,
    pub love_hearts: Vec<Vec3>,
    pub pending_block_changes: Vec<(IVec3, BlockId)>,
    pub pending_arrows: Vec<ArrowEntity>,
}

impl Default for MobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MobManager {
    pub fn new() -> Self {
        MobManager {
            mobs: Vec::new(),
            next_mob_id: 1,
            spawn_timer: 1.0,
            spawn_center_cursor: 0,
            rng_state: 987654321,
            dying_poofs: Vec::new(),
            dying_drops: Vec::new(),
            pending_explosions: Vec::new(),
            breeding_hearts: Vec::new(),
            love_hearts: Vec::new(),
            pending_block_changes: Vec::new(),
            pending_arrows: Vec::new(),
        }
    }

    pub fn spawn_mob(&mut self, kind: MobKind, pos: Vec3) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id = self.next_mob_id.wrapping_add(1);
        self.mobs.push(Mob::new(id, kind, pos));
        id
    }

    pub fn network_snapshot(&self) -> Vec<crate::net::protocol::MobSyncEntry> {
        self.mobs
            .iter()
            .map(|mob| crate::net::protocol::MobSyncEntry {
                id: mob.id,
                kind: mob.kind as u8,
                x: mob.pos.x,
                y: mob.pos.y,
                z: mob.pos.z,
                yaw: mob.yaw,
                health: mob.health,
                is_dead: mob.is_dead,
                walk_phase: mob.walk_phase,
                is_baby: mob.is_baby,
                wool_color: mob.wool_color,
                is_sheared: mob.is_sheared,
                is_puffed: mob.is_puffed,
                fuse: mob.fuse,
                is_ignited: mob.is_ignited,
            })
            .take(MOB_SNAPSHOT_CAP)
            .collect()
    }

    pub fn apply_network_snapshot(
        &mut self,
        snapshot: &[crate::net::protocol::MobSyncEntry],
    ) -> Result<(), String> {
        let mut mobs = Vec::with_capacity(snapshot.len());
        for entry in snapshot {
            let kind = MobKind::ALL
                .get(entry.kind as usize)
                .copied()
                .ok_or_else(|| format!("Unknown mob kind {} in server snapshot", entry.kind))?;
            let mut mob = Mob::new(entry.id, kind, Vec3::new(entry.x, entry.y, entry.z));
            mob.yaw = entry.yaw;
            mob.health = entry.health;
            mob.is_dead = entry.is_dead;
            mob.walk_phase = entry.walk_phase;
            mob.is_baby = entry.is_baby;
            mob.wool_color = entry.wool_color;
            mob.is_sheared = entry.is_sheared;
            mob.is_puffed = entry.is_puffed;
            mob.fuse = entry.fuse;
            mob.is_ignited = entry.is_ignited;
            mobs.push(mob);
        }
        self.next_mob_id = mobs.iter().map(|m| m.id).max().unwrap_or(0).wrapping_add(1).max(1);
        self.mobs = mobs;
        Ok(())
    }

    pub fn update(
        &mut self,
        dt: f32,
        world: &World,
        player_pos: Vec3,
        player_held: Option<BlockId>,
        is_creative: bool,
    ) {
        self.update_for_players(
            dt,
            world,
            &[MobPlayerContext { position: player_pos, held_item: player_held, is_creative }],
        );
    }

    pub fn update_for_players(&mut self, dt: f32, world: &World, players: &[MobPlayerContext]) {
        let fallback_player =
            MobPlayerContext { position: Vec3::zero(), held_item: None, is_creative: false };
        let players = if players.is_empty() { std::slice::from_ref(&fallback_player) } else { players };
        let mut rng = self.rng_state;
        self.breeding_hearts.clear();
        self.love_hearts.clear();

        let villagers: Vec<(usize, Vec3)> = self
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, mob)| mob.kind == MobKind::Villager && !mob.is_dead)
            .map(|(index, mob)| (index, mob.pos))
            .collect();
        let mut zombie_attacks = Vec::new();

        for (mob_index, mob) in self.mobs.iter_mut().enumerate() {
            let target_player = nearest_target_player(mob, players).copied().unwrap_or(fallback_player);
            let (player_pos, player_held, is_creative) =
                (target_player.position, target_player.held_item, target_player.is_creative);

            if mob.kind == MobKind::IronGolem {
                mob.player_anger_timer = (mob.player_anger_timer - dt).max(0.0);
            }
            if mob.kind == MobKind::IronGolem || mob.kind == MobKind::Zombie {
                mob.attack_cooldown = (mob.attack_cooldown - dt).max(0.0);
            }

            let villager_target = find_nearest_villager_target(mob, &villagers);
            let villager_pos = villager_target.map(|(_, pos, _)| pos);
            if let Some((villager_index, _, distance_sq)) = villager_target {
                if distance_sq <= 1.5 * 1.5 && mob.attack_cooldown <= 0.0 {
                    zombie_attacks.push((mob_index, villager_index));
                    mob.attack_cooldown = 1.0;
                }
            }

            let was_exploding = mob.kind == MobKind::Reaper && mob.fuse >= REAPER_FUSE_DURATION;
            let target_pos = resolve_visible_target(mob, player_pos, player_held, is_creative);
            if is_creative && mob.kind == MobKind::Reaper && mob.is_ignited {
                mob.is_ignited = false;
                mob.fuse = 0.0;
            }

            if try_sheep_graze(mob, world, rng) {
                self.pending_block_changes.push((
                    IVec3::new(mob.pos.x.floor() as i32, (mob.pos.y - 0.2).floor() as i32, mob.pos.z.floor() as i32),
                    BlockId::Dirt,
                ));
            }

            mob.update(dt, world, &mut rng, target_pos, player_held, villager_pos);
            if mob.kind == MobKind::Reaper && mob.fuse >= REAPER_FUSE_DURATION && !was_exploding {
                self.pending_explosions.push(mob.center_pos());
            }

            if mob.in_love > 0.0 && !mob.is_dead {
                mob.love_particle_timer -= dt;
                if mob.love_particle_timer <= 0.0 {
                    mob.love_particle_timer = 0.45;
                    self.love_hearts.push(mob.center_pos() + Vec3::new(0.0, mob.kind.height() * 0.45, 0.0));
                }
            }

            if !is_creative && mob.kind == MobKind::Skeleton && !mob.is_dead {
                if let Some(arrow) = try_skeleton_shoot(mob, dt, world, player_pos) {
                    self.pending_arrows.push(arrow);
                }
            }

            let nearest_sq = players
                .iter()
                .map(|p| {
                    let (dx, dz) = (mob.pos.x - p.position.x, mob.pos.z - p.position.z);
                    dx * dx + dz * dz
                })
                .fold(f32::INFINITY, f32::min);
            if nearest_sq > DESPAWN_DISTANCE_SQ && !mob.is_dead {
                mob.despawn_pending = true;
            }
        }

        for (zombie_index, villager_index) in zombie_attacks {
            let dir = self.mobs[villager_index].pos - self.mobs[zombie_index].pos;
            self.mobs[villager_index].take_damage(3.0, Vec3::new(dir.x, 1.0, dir.z));
        }

        self.resolve_golem_defense();
        self.resolve_breeding(&mut rng);
        self.cull_despawned_mobs();

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = if world.sunlight_factor() < 0.50 { 2.0 } else { 4.0 };
            let spawn_player = players[self.spawn_center_cursor % players.len()];
            self.spawn_center_cursor = self.spawn_center_cursor.wrapping_add(1);
            self.try_natural_spawn(world, spawn_player.position, &mut rng);
        }

        self.rng_state = rng;
    }

    fn cull_despawned_mobs(&mut self) {
        self.dying_poofs.clear();
        self.dying_drops.clear();
        let dying_poofs = &mut self.dying_poofs;
        let dying_drops = &mut self.dying_drops;
        self.mobs.retain(|m| {
            if m.despawn_pending {
                if m.is_dead {
                    dying_poofs.push(m.center_pos());
                    collect_mob_drops(m, dying_drops);
                }
                false
            } else {
                true
            }
        });
    }
}

fn nearest_target_player<'a>(
    mob: &Mob,
    players: &'a [MobPlayerContext],
) -> Option<&'a MobPlayerContext> {
    let dist_sq = |player: &&MobPlayerContext| (player.position - mob.pos).length_sq();
    players
        .iter()
        .filter(|player| !mob.kind.is_hostile() || !player.is_creative)
        .min_by(|a, b| dist_sq(a).total_cmp(&dist_sq(b)))
        .or_else(|| players.iter().min_by(|a, b| dist_sq(a).total_cmp(&dist_sq(b))))
}

fn find_nearest_villager_target(mob: &Mob, villagers: &[(usize, Vec3)]) -> Option<(usize, Vec3, f32)> {
    if mob.kind != MobKind::Zombie || mob.is_dead {
        return None;
    }
    villagers
        .iter()
        .filter_map(|&(villager_index, pos)| {
            let (dx, dz, dy) = (pos.x - mob.pos.x, pos.z - mob.pos.z, (pos.y - mob.pos.y).abs());
            let dist_sq = dx * dx + dz * dz;
            (dist_sq < HOSTILE_TARGET_RANGE_SQ && dy < HOSTILE_TARGET_Y_RANGE)
                .then_some((villager_index, pos, dist_sq))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
}

fn resolve_visible_target(
    mob: &Mob,
    player_pos: Vec3,
    player_held: Option<BlockId>,
    is_creative: bool,
) -> Option<Vec3> {
    if !is_creative {
        return Some(player_pos);
    }
    if mob.kind.is_hostile() {
        None
    } else if mob.kind == MobKind::IronGolem && mob.player_anger_timer > 0.0 {
        Some(player_pos)
    } else if player_held.map(|h| mob.kind.is_tempting_item(h)) == Some(true) {
        Some(player_pos)
    } else {
        None
    }
}

fn try_sheep_graze(mob: &mut Mob, world: &World, rng: u32) -> bool {
    if mob.kind != MobKind::Sheep || mob.is_dead || !mob.on_ground || mob.eat_grass_timer > 0.0 {
        return false;
    }
    let graze_roll = (rng.wrapping_mul(1664525) % 1000) as f32 / 1000.0;
    let threshold = if mob.is_sheared { 0.008 } else { 0.002 };
    if graze_roll >= threshold {
        return false;
    }
    let under_pos =
        IVec3::new(mob.pos.x.floor() as i32, (mob.pos.y - 0.2).floor() as i32, mob.pos.z.floor() as i32);
    if world.get_block(under_pos) == BlockId::Grass {
        mob.eat_grass_timer = 2.0;
        mob.ai_state_is_moving = false;
        true
    } else {
        false
    }
}
