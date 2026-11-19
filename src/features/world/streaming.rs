//! World chunk streaming, async generation worker coordination, and mesh job assembly.
//! Uses PAL threading & capabilities for zero platform `#[cfg]` inside `features::world`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::math::Vec3;
use crate::features::meshing::{ChunkMeshJob, LightSource};
use crate::features::player::GameMode;
use crate::platform::threading::{configure_worker_thread_affinity, worker_thread_count};

use super::biome::WorldGeneratorType;
use super::chunk::Chunk;
use super::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Z};
use super::weather::WeatherState;
use super::world_state::World;

impl World {
    pub fn new_with_seed(seed: u32) -> Self {
        Self::new_with_generator(seed, WorldGeneratorType::Default)
    }

    pub fn new_with_generator(seed: u32, generator_type: WorldGeneratorType) -> Self {
        crate::log_info!("WORLD", "Initializing world with seed {} (type: {:?})", seed, generator_type);
        let num_threads = worker_thread_count(2, 6, 1);
        let (chunk_req_tx, chunk_res_rx) = if num_threads > 0 {
            let (req_tx, req_rx) = std::sync::mpsc::channel::<(i32, i32, u32, WorldGeneratorType)>();
            let (res_tx, res_rx) = std::sync::mpsc::channel::<Chunk>();
            let req_rx = std::sync::Arc::new(std::sync::Mutex::new(req_rx));
            crate::log_debug!("WORLD", "Spawned {} asynchronous chunk generation worker threads", num_threads);

            for i in 0..num_threads {
                let req_rx_clone = std::sync::Arc::clone(&req_rx);
                let res_tx_clone = res_tx.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("chunk_gen_{}", i))
                    .spawn(move || {
                        configure_worker_thread_affinity(i, false);
                        while let Some((cx, cz, chunk_seed, gen_type)) =
                            req_rx_clone.lock().ok().and_then(|rx| rx.recv().ok())
                        {
                            crate::log_debug!(
                                "CHUNK",
                                "Worker generating chunk ({}, {}) [seed={}, type={:?}]",
                                cx,
                                cz,
                                chunk_seed,
                                gen_type
                            );
                            if res_tx_clone
                                .send(Chunk::generate_with_type(cx, cz, chunk_seed, gen_type))
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
            }
            (Some(req_tx), Some(std::sync::Mutex::new(res_rx)))
        } else {
            (None, None)
        };

        let spawn_pos = Self::find_spawn_position_for_type(seed, generator_type);
        let (spawn_cx, spawn_cz) = ((spawn_pos.x.floor() as i32) >> 4, (spawn_pos.z.floor() as i32) >> 4);
        crate::log_info!(
            "WORLD",
            "Spawn point found at ({:.1}, {:.1}, {:.1}) [chunk ({}, {})]",
            spawn_pos.x,
            spawn_pos.y,
            spawn_pos.z,
            spawn_cx,
            spawn_cz
        );

        let radius: i32 = crate::platform::capabilities().initial_spawn_radius;
        let total_chunks = (radius * 2 + 1) * (radius * 2 + 1);
        crate::log_info!(
            "WORLD",
            "Pregenerating initial {}x{} spawn chunks around ({}, {})",
            radius * 2 + 1,
            radius * 2 + 1,
            spawn_cx,
            spawn_cz
        );

        let mut world = World {
            name: "New World".to_string(),
            gamemode: GameMode::Survival,
            generator_type,
            chunks: HashMap::new(),
            block_overrides: HashMap::new(),
            pending_network_changes: HashMap::new(),
            seed,
            player_position: spawn_pos,
            fluid_levels: HashMap::new(),
            active_sand: HashSet::new(),
            chests: HashMap::new(),
            furnaces: HashMap::new(),
            saved_player: None,
            sim_tick: 0,
            time_of_day: 120.0,
            torches: HashSet::new(),
            ruststone_lights: HashMap::new(),
            button_timers: HashMap::new(),
            active_pressure_plates: HashSet::new(),
            ruststone_sound_events: Vec::new(),
            pending_explosions: Vec::new(),
            primed_tnts: Vec::new(),
            weather: WeatherState::new(seed),
            chunk_req_tx,
            chunk_res_rx,
            pending_chunks: HashSet::new(),
            last_stream_regions: Vec::new(),
        };

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let (cx, cz) = (spawn_cx + dx, spawn_cz + dz);
                let chunk = Chunk::generate_with_type(cx, cz, seed, generator_type);
                world.index_chunk_lights(&chunk);
                world.chunks.insert((cx, cz), chunk);
                crate::log_info!(
                    "WORLD",
                    "Spawn chunk ({}, {}) ready [{}/{}]",
                    cx,
                    cz,
                    world.chunks.len(),
                    total_chunks
                );
            }
        }
        crate::log_info!(
            "WORLD",
            "Pregeneration completed ({} chunks loaded in memory)",
            world.chunks.len()
        );
        if let Some(safe_spawn) = world.find_clear_spawn_position(world.player_position) {
            if safe_spawn.x != world.player_position.x || safe_spawn.z != world.player_position.z {
                crate::log_warn!(
                    "WORLD",
                    "Spawn position was obstructed; moved player to ({:.1}, {:.1}, {:.1})",
                    safe_spawn.x,
                    safe_spawn.y,
                    safe_spawn.z
                );
                world.player_position = safe_spawn;
            }
        } else {
            crate::log_warn!(
                "WORLD",
                "Could not find a clear spawn position near ({:.1}, {:.1}, {:.1})",
                world.player_position.x,
                world.player_position.y,
                world.player_position.z
            );
        }
        world
    }

    pub fn pending_chunks_count(&self) -> usize {
        self.pending_chunks.len()
    }

    pub fn create_mesh_job(&self, key: (i32, i32)) -> Option<ChunkMeshJob> {
        let chunk = self.chunks.get(&key)?;
        let (cx, cz) = key;
        let (center_x, center_z) = (cx * CHUNK_SIZE_X + 8, cz * CHUNK_SIZE_Z + 8);

        let mut lights = Vec::new();
        if !self.torches.is_empty() {
            for &pos in &self.torches {
                if (pos.x - center_x).abs() <= 24 && (pos.z - center_z).abs() <= 24 {
                    lights.push(LightSource { pos, radius: 14.0, intensity: 1.0 });
                }
            }
        }
        if !self.ruststone_lights.is_empty() {
            for (&pos, &(radius, intensity)) in &self.ruststone_lights {
                if (pos.x - center_x).abs() <= 24 && (pos.z - center_z).abs() <= 24 {
                    lights.push(LightSource { pos, radius, intensity });
                }
            }
        }

        let get_nb = |dx: i32, dz: i32| self.chunks.get(&(cx + dx, cz + dz)).map(|c| Arc::clone(&c.blocks));
        let fluid_map: HashMap<_, _> = if self.fluid_levels.is_empty() {
            HashMap::new()
        } else {
            self.fluid_levels
                .iter()
                .filter(|(pos, _)| (pos.x - center_x).abs() <= 10 && (pos.z - center_z).abs() <= 10)
                .map(|(&pos, &lvl)| (pos, lvl))
                .collect()
        };

        Some(ChunkMeshJob {
            key,
            center_blocks: Arc::clone(&chunk.blocks),
            east_blocks: get_nb(1, 0),
            west_blocks: get_nb(-1, 0),
            south_blocks: get_nb(0, 1),
            north_blocks: get_nb(0, -1),
            se_blocks: get_nb(1, 1),
            sw_blocks: get_nb(-1, 1),
            ne_blocks: get_nb(1, -1),
            nw_blocks: get_nb(-1, -1),
            seed: self.seed,
            generator_type: self.generator_type,
            lights,
            fluid_map,
        })
    }


    pub fn update(&mut self, player_pos: Vec3, view_distance: i32) {
        self.update_with_simulation_centers(player_pos, view_distance, &[]);
    }

    pub fn update_with_simulation_centers(
        &mut self,
        player_pos: Vec3,
        view_distance: i32,
        simulation_centers: &[Vec3],
    ) {
        let view_distance = view_distance.clamp(2, 16);
        let mut regions = Vec::with_capacity(simulation_centers.len() + 1);
        let mut add_region = |pos: Vec3, load_radius: i32, retain_radius: i32| {
            regions.push((
                (pos.x.floor() as i32) >> 4,
                          (pos.z.floor() as i32) >> 4,
                          load_radius,
                          retain_radius,
            ));
        };
        add_region(player_pos, view_distance, view_distance + 3);
        for &center in simulation_centers {
            add_region(center, 6, 7);
        }
        drop(add_region);
        regions.sort_unstable();
        regions.dedup();

        let is_within_region = |cx: i32, cz: i32, retain: bool| {
            regions.iter().any(|&(center_x, center_z, load_radius, retain_radius)| {
                let radius = if retain { retain_radius } else { load_radius };
                let dx = cx - center_x;
                let dz = cz - center_z;
                dx * dx + dz * dz <= radius * radius
            })
        };
        let stream_key: Vec<_> = regions.clone();

        let max_integrate = crate::platform::capabilities().max_chunks_integrated_per_frame.max(1);
        let mut received_chunks = Vec::with_capacity(max_integrate);
        if let Some(rx_mutex) = &self.chunk_res_rx {
            if let Ok(rx) = rx_mutex.lock() {
                while received_chunks.len() < max_integrate {
                    match rx.try_recv() {
                        Ok(chunk) => received_chunks.push(chunk),
                        Err(_) => break,
                    }
                }
            }
        }
        for chunk in received_chunks {
            let (cx, cz) = (chunk.cx, chunk.cz);
            self.pending_chunks.remove(&(cx, cz));
            if is_within_region(cx, cz, true) {
                let mut chunk = chunk;
                self.apply_block_overrides(&mut chunk);
                self.index_chunk_lights(&chunk);
                self.chunks.insert((cx, cz), chunk);
                crate::log_debug!(
                    "CHUNK",
                    "Integrated chunk ({}, {}) into active world (loaded: {})",
                    cx,
                    cz,
                    self.chunks.len()
                );
                self.mark_neighbors_dirty(cx, cz);
            }
        }

        if self.last_stream_regions == stream_key && (self.chunk_req_tx.is_some() || self.pending_chunks.is_empty()) {
            return;
        }

        let mut missing_chunks = HashSet::new();
        for &(center_x, center_z, radius, _) in &regions {
            let view_dist_sq = radius * radius;
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dz * dz <= view_dist_sq {
                        let key = (center_x + dx, center_z + dz);
                        if !self.chunks.contains_key(&key) && !self.pending_chunks.contains(&key) {
                            missing_chunks.insert(key);
                        }
                    }
                }
            }
        }

        let has_missing_chunks = !missing_chunks.is_empty();
        if has_missing_chunks {
            let mut missing_chunks: Vec<_> = missing_chunks.into_iter().collect();
            crate::log_debug!("CHUNK", "Requesting generation for {} missing chunks across {} player regions", missing_chunks.len(), regions.len());
            missing_chunks.sort_unstable_by_key(|&(cx, cz)| {
                regions
                .iter()
                .map(|&(center_x, center_z, _, _)| {
                    (cx - center_x) * (cx - center_x) + (cz - center_z) * (cz - center_z)
                })
                .min()
                .unwrap_or(i32::MAX)
            });

            if let Some(tx) = &self.chunk_req_tx {
                for (cx, cz) in missing_chunks {
                    self.pending_chunks.insert((cx, cz));
                    let _ = tx.send((cx, cz, self.seed, self.generator_type));
                }
            } else {
                for (cx, cz) in missing_chunks.into_iter().take(1) {
                    let mut chunk = Chunk::generate_with_type(cx, cz, self.seed, self.generator_type);
                    self.apply_block_overrides(&mut chunk);
                    self.index_chunk_lights(&chunk);
                    self.chunks.insert((cx, cz), chunk);
                    self.mark_neighbors_dirty(cx, cz);
                }
            }
        }

        self.last_stream_regions = if self.chunk_req_tx.is_some() || (self.pending_chunks.is_empty() && !has_missing_chunks) {
            stream_key
        } else {
            Vec::new()
        };

        let count_before = self.chunks.len();
        self.chunks
        .retain(|&(cx, cz), _| is_within_region(cx, cz, true));
        self.pending_chunks
        .retain(|&(cx, cz)| is_within_region(cx, cz, true));
        if self.chunks.len() < count_before {
            crate::log_debug!("CHUNK", "Unloaded {} distant chunks (remaining active: {})", count_before - self.chunks.len(), self.chunks.len());
        }
    }

    pub fn mark_all_chunks_dirty(&mut self) {
        for chunk in self.chunks.values_mut() { chunk.dirty = true; }
        crate::log_info!("WORLD", "Marked all {} chunks dirty for mesh regeneration", self.chunks.len());
    }

    pub fn purge_distant_chunks(&mut self, player_pos: Vec3, view_distance: i32) {
        let view_distance = view_distance.clamp(2, 16);
        let (pcx, pcz) = ((player_pos.x.floor() as i32) >> 4, (player_pos.z.floor() as i32) >> 4);
        let unload_dist_sq = (view_distance + 2) * (view_distance + 2);
        let before_count = self.chunks.len();

        self.chunks.retain(|&(cx, cz), _| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        self.pending_chunks.retain(|&(cx, cz)| (cx - pcx) * (cx - pcx) + (cz - pcz) * (cz - pcz) <= unload_dist_sq);
        for chunk in self.chunks.values_mut() { chunk.dirty = true; }

        self.last_stream_regions.clear();
        crate::log_info!("CHUNK", "Purged distant chunks: {} -> {} active (view distance: {} chunks)", before_count, self.chunks.len(), view_distance);
    }
}
