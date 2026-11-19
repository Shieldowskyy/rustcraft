//! Shared AI, combat, spawning, and physics balance constants for mob entities.

pub(super) const MOB_GRAVITY: f32 = 22.0;
pub(super) const DEATH_DESPAWN_DELAY: f32 = 0.8;
pub(super) const HOSTILE_TARGET_RANGE_SQ: f32 = 24.0 * 24.0;
pub(super) const HOSTILE_TARGET_Y_RANGE: f32 = 12.0;
pub(super) const REAPER_FUSE_DURATION: f32 = 1.5;
pub(super) const DESPAWN_DISTANCE_SQ: f32 = 75.0 * 75.0;
pub(super) const MAX_HOSTILE_NEARBY: usize = 14;
pub(super) const MAX_AQUATIC_NEARBY: usize = 14;
pub(super) const MOB_SNAPSHOT_CAP: usize = 512;
pub(super) const GOLEM_VILLAGER_ANGER_RANGE_SQ: f32 = 16.0 * 16.0;
pub(super) const GOLEM_ENEMY_RANGE_SQ: f32 = 16.0 * 16.0;
pub(super) const SKELETON_BOW_RANGE_SQ: f32 = 18.0 * 18.0;
pub(super) const SKELETON_FIRE_INTERVAL: f32 = 2.2;
