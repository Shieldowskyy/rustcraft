//! Core `World` container, `PrimedTnt` entity state, and physics/acoustic delegate wrappers.
//! Holds chunk storage, block overrides, lighting caches, tile entities, and async gen channels.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::physics::RaycastHit;
use crate::features::player::{GameMode, SavedPlayerData};

use super::biome::WorldGeneratorType;
use super::chunk::Chunk;
use super::constants::DEFAULT_WORLD_SEED;
use super::tile_entity::{ChestData, FurnaceData};
use super::weather::WeatherState;

#[derive(Clone)]
pub struct PrimedTnt {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
}

pub struct World {
    pub name: String,
    pub gamemode: GameMode,
    pub generator_type: WorldGeneratorType,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub(super) block_overrides: HashMap<(i32, i32), HashMap<IVec3, u8>>,
    pub(super) pending_network_changes: HashMap<IVec3, u8>,
    pub seed: u32,
    pub player_position: Vec3,
    pub(super) fluid_levels: HashMap<IVec3, u8>,
    pub active_sand: HashSet<IVec3>,
    pub chests: HashMap<IVec3, ChestData>,
    pub furnaces: HashMap<IVec3, FurnaceData>,
    pub saved_player: Option<SavedPlayerData>,
    pub sim_tick: u64,
    pub time_of_day: f32,
    pub torches: HashSet<IVec3>,
    pub ruststone_lights: HashMap<IVec3, (f32, f32)>,
    pub button_timers: HashMap<IVec3, f32>,
    pub active_pressure_plates: HashSet<IVec3>,
    pub ruststone_sound_events: Vec<(IVec3, bool)>,
    pub pending_explosions: Vec<Vec3>,
    pub primed_tnts: Vec<PrimedTnt>,
    pub weather: WeatherState,
    #[allow(dead_code)]
    pub(super) chunk_req_tx: Option<Sender<(i32, i32, u32, WorldGeneratorType)>>,
    pub(super) chunk_res_rx: Option<Mutex<Receiver<Chunk>>>,
    pub(super) pending_chunks: HashSet<(i32, i32)>,
    pub(super) last_stream_regions: Vec<(i32, i32, i32, i32)>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub(super) fn ruststone_light_params(b: BlockId) -> Option<(f32, f32)> {
    match b {
        BlockId::RustStoneOre => Some((8.0, 0.75)),
        BlockId::RustStoneBlock => Some((9.0, 0.8)),
        BlockId::RustStoneLampLit => Some((14.0, 1.0)),
        BlockId::RustStoneWireLit => Some((7.0, 0.65)),
        _ => None,
    }
}


impl World {
    pub const DAY_DURATION: f32 = 480.0;

    pub fn new() -> Self {
        Self::new_with_seed(DEFAULT_WORLD_SEED)
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RaycastHit> {
        crate::features::physics::raycast_world(self, origin, dir, max_dist)
    }

    pub fn raycast_camera_collision(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> f32 {
        crate::features::physics::raycast_camera_collision(self, origin, dir, max_dist)
    }

    pub fn calculate_acoustic_occlusion(&self, listener: Vec3, emitter: Vec3) -> f32 {
        crate::features::physics::calculate_acoustic_occlusion(self, listener, emitter)
    }

    pub fn calculate_acoustic_reverb(&self, listener: Vec3) -> (f32, f32) {
        crate::features::physics::calculate_acoustic_reverb(self, listener)
    }

    pub fn box_collides(&self, min: Vec3, max: Vec3) -> bool {
        crate::features::physics::box_collides(self, min, max)
    }
}
