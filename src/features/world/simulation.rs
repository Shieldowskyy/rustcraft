//! World tick systems: furnace smelting, weather, leaf decay, primed TNT, crop growth, explosions, fluids/sand.
//! Advances world simulation state deterministically on game/server ticks.

use std::collections::{HashSet, VecDeque};

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;

use super::chunk::Chunk;
use super::constants::{CARDINAL_OFFSETS, CHUNK_SIZE_Y};
use super::weather::LightningStrike;
use super::world_state::{PrimedTnt, World};

impl World {
    pub fn update_furnaces(&mut self, dt: f32) {
        let state_changes: Vec<_> = self.furnaces.iter_mut().filter_map(|(&pos, f)| {
            f.tick(dt).then_some((pos, if f.is_burning() { BlockId::FurnaceLit } else { BlockId::Furnace }))
        }).collect();

        for (pos, new_block) in state_changes {
            if matches!(self.get_block(pos), BlockId::Furnace | BlockId::FurnaceLit) {
                self.set_block(pos, new_block);
            }
        }
    }


    pub fn update_weather(&mut self, dt: f32, player_pos: Vec3) -> Option<LightningStrike> {
        let default_y = player_pos.y.max(60.0);
        let chunk_map = &self.chunks;
        self.weather.update(dt, player_pos, |x, z| {
            let (cx, cz, lx, lz) = World::world_to_chunk(x.floor() as i32, z.floor() as i32);
            if let Some(chunk) = chunk_map.get(&(cx, cz)) {
                for y in (0..CHUNK_SIZE_Y).rev() {
                    if chunk.get_local(lx, y, lz) != BlockId::Air {
                        return y as f32 + 1.0;
                    }
                }
            }
            default_y
        })
    }


    pub fn is_leaf_supported(&self, start: IVec3, max_dist: i32) -> bool {
        let mut visited = HashSet::from([start]);
        let mut queue = VecDeque::from([(start, 0)]);

        while let Some((pos, dist)) = queue.pop_front() {
            if self.get_block(pos).is_wood() {
                return true;
            }
            if dist < max_dist {
                for offset in CARDINAL_OFFSETS {
                    let neighbor = pos + offset;
                    if visited.insert(neighbor) {
                        let n_block = self.get_block(neighbor);
                        if n_block.is_wood() { return true; }
                        if n_block.is_leaves() { queue.push_back((neighbor, dist + 1)); }
                    }
                }
            }
        }
        false
    }

    pub fn find_unsupported_leaves_around(&self, center: IVec3) -> Vec<IVec3> {
        let mut unsupported = Vec::new();
        for dx in -5..=5 {
            for dz in -5..=5 {
                for dy in -5..=10 {
                    let leaf_pos = center + IVec3::new(dx, dy, dz);
                    if self.get_block(leaf_pos).is_leaves() && !self.is_leaf_supported(leaf_pos, 5) {
                        unsupported.push(leaf_pos);
                    }
                }
            }
        }
        unsupported
    }


    pub fn prime_tnt(&mut self, pos: IVec3) {
        if self.get_block(pos) == BlockId::Tnt {
            self.set_block(pos, BlockId::Air);
            self.primed_tnts.push(PrimedTnt {
                pos: Vec3::new(pos.x as f32 + 0.5, pos.y as f32, pos.z as f32 + 0.5),
                                  velocity: Vec3::new(0.0, 2.0, 0.0),
                                  fuse: 3.5,
            });
        }
    }


    pub fn update_primed_tnts(&mut self, dt: f32) {
        let mut exploded_positions = Vec::new();
        for i in 0..self.primed_tnts.len() {
            self.primed_tnts[i].fuse -= dt;
            self.primed_tnts[i].velocity.y -= 20.0 * dt;
            let mut new_pos = self.primed_tnts[i].pos;
            new_pos.y += self.primed_tnts[i].velocity.y * dt;

            if self.box_collides(Vec3::new(new_pos.x - 0.49, new_pos.y, new_pos.z - 0.49), Vec3::new(new_pos.x + 0.49, new_pos.y + 0.99, new_pos.z + 0.49)) {
                self.primed_tnts[i].velocity = Vec3::zero();
                self.primed_tnts[i].pos.y = self.primed_tnts[i].pos.y.floor();
            } else {
                self.primed_tnts[i].pos = new_pos;
            }

            if self.primed_tnts[i].fuse <= 0.0 {
                exploded_positions.push(self.primed_tnts[i].pos.floor_to_ivec());
            }
        }
        self.primed_tnts.retain(|tnt| tnt.fuse > 0.0);
        for center in exploded_positions {
            self.explode(center, 3.5);
        }
    }

    pub fn grow_tree_at(&mut self, pos: IVec3) -> bool {
        let trunk_height = 4 + (pos.x.wrapping_mul(7) ^ pos.z.wrapping_mul(13)).rem_euclid(2);
        for i in 0..trunk_height {
            self.set_block(IVec3::new(pos.x, pos.y + i, pos.z), BlockId::Wood);
        }
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..=2 {
                    if dx * dx + dz * dz + (dy - 1) * (dy - 1) * 2 <= 6 {
                        let p = IVec3::new(pos.x + dx, pos.y + trunk_height - 1 + dy, pos.z + dz);
                        if self.get_block(p) == BlockId::Air {
                            self.set_block(p, BlockId::Leaves);
                        }
                    }
                }
            }
        }
        true
    }

    pub fn tick_farming(&mut self, player_pos: Vec3, rng_seed: &mut u32) {
        let (pcx, pcz) = ((player_pos.x.floor() as i32) >> 4, (player_pos.z.floor() as i32) >> 4);
        let mut next_rand = || -> u32 {
            *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *rng_seed
        };

        for _ in 0..4 {
            let cx = pcx + (next_rand() % 5) as i32 - 2;
            let cz = pcz + (next_rand() % 5) as i32 - 2;
            if self.chunks.contains_key(&(cx, cz)) {
                for _ in 0..3 {
                    let pos = IVec3::new(cx * 16 + (next_rand() % 16) as i32, 20 + (next_rand() % 35) as i32, cz * 16 + (next_rand() % 16) as i32);
                    match self.get_block(pos) {
                        BlockId::Farmland => {
                            let mut has_water = false;
                            'water_search: for wx2 in -4..=4 {
                                for wz2 in -4..=4 {
                                    for wy2 in 0..=1 {
                                        if self.get_block(pos + IVec3::new(wx2, wy2, wz2)) == BlockId::Water {
                                            has_water = true;
                                            break 'water_search;
                                        }
                                    }
                                }
                            }
                            if has_water {
                                self.set_block(pos, BlockId::FarmlandWet);
                            } else if !matches!(self.get_block(pos + IVec3::new(0, 1, 0)), BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop) && next_rand() % 6 == 0 {
                                self.set_block(pos, BlockId::Dirt);
                            }
                        }
                        BlockId::Sapling if next_rand() % 35 == 0 => {
                            self.set_block(pos, BlockId::Air);
                            self.grow_tree_at(pos);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn explode(&mut self, center: IVec3, radius: f32) {
        let r_int = radius.ceil() as i32;
        let r_sq = radius * radius;
        let mut tnts_to_prime = Vec::new();

        for dx in -r_int..=r_int {
            for dy in -r_int..=r_int {
                for dz in -r_int..=r_int {
                    if (dx * dx + dy * dy + dz * dz) as f32 <= r_sq {
                        let target_pos = center + IVec3::new(dx, dy, dz);
                        let b = self.get_block(target_pos);
                        if b != BlockId::Air && !b.is_bedrock() {
                            if b == BlockId::Tnt { tnts_to_prime.push(target_pos); }
                            else { self.set_block(target_pos, BlockId::Air); }
                        }
                    }
                }
            }
        }
        self.pending_explosions.push(Vec3::new(center.x as f32 + 0.5, center.y as f32 + 0.5, center.z as f32 + 0.5));
        for tnt_pos in tnts_to_prime { self.prime_tnt(tnt_pos); }
    }


    pub fn simulate_materials(&mut self, player_pos: Vec3, authoritative: bool) {
        self.sim_tick = self.sim_tick.wrapping_add(1);
        let (px, py, pz) = (player_pos.x.floor() as i32, player_pos.y.floor() as i32, player_pos.z.floor() as i32);
        let (min_y, max_y) = ((py - 12).max(1), (py + 8).min(CHUNK_SIZE_Y - 1));
        let sim_lava = self.sim_tick % 3 == 0;

        let mut fluid_spawns = Vec::new();
        let mut fluid_despawns = Vec::new();
        let mut solidifications = Vec::new();
        let mut tnts_to_prime = Vec::new();
        let mut hint: Option<((i32, i32), &Chunk)> = None;

        for y in (min_y..=max_y).rev() {
            for z in (pz - 10)..=(pz + 10) {
                for x in (px - 10)..=(px + 10) {
                    let pos = IVec3::new(x, y, z);
                    let (block, new_hint) = self.get_block_cached(pos, hint);
                    hint = new_hint;

                    if block == BlockId::Tnt {
                        for off in CARDINAL_OFFSETS {
                            let (nblock, nh) = self.get_block_cached(pos + off, hint);
                            hint = nh;
                            if nblock == BlockId::Lava {
                                tnts_to_prime.push(pos);
                                break;
                            }
                        }
                        continue;
                    }

                    if !authoritative || !matches!(block, BlockId::Water | BlockId::Lava) || (block == BlockId::Lava && !sim_lava) {
                        continue;
                    }

                    let level = self.fluid_level(pos);
                    let below = IVec3::new(x, y - 1, z);
                    let below_block = self.get_block(below);
                    let cardinals = [
                        IVec3::new(x + 1, y, z), IVec3::new(x - 1, y, z),
                        IVec3::new(x, y, z + 1), IVec3::new(x, y, z - 1),
                        IVec3::new(x, y + 1, z),
                    ];

                    if block == BlockId::Lava && (below_block == BlockId::Water || cardinals.iter().any(|&p| self.get_block(p) == BlockId::Water)) {
                        solidifications.push((pos, BlockId::Stone));
                        continue;
                    }

                    if below.y >= 0 && below_block == BlockId::Air {
                        fluid_spawns.push((below, block, level));
                    } else if block == BlockId::Water && below_block == BlockId::Lava {
                        solidifications.push((below, BlockId::Stone));
                    } else {
                        let step = if block == BlockId::Lava { 4 } else { 2 };
                        if level > step {
                            for &npos in &cardinals[0..4] {
                                let nblock = self.get_block(npos);
                                if nblock == BlockId::Air { fluid_spawns.push((npos, block, level - step)); }
                                else if block == BlockId::Water && nblock == BlockId::Lava { solidifications.push((npos, BlockId::Stone)); }
                            }
                        }
                    }

                    if level < 8 && self.get_block(cardinals[4]) != block && !cardinals[0..4].iter().any(|&p| self.get_block(p) == block && self.fluid_level(p) > level) {
                        fluid_despawns.push(pos);
                    }
                }
            }
        }

        for tnt_pos in tnts_to_prime { self.prime_tnt(tnt_pos); }
        for (pos, stone) in solidifications { self.set_block(pos, stone); }
        for (pos, block, level) in fluid_spawns {
            if self.get_block(pos) == BlockId::Air {
                self.set_block(pos, block);
                self.set_fluid_level(pos, level);
            }
        }
        for pos in fluid_despawns {
            if matches!(self.get_block(pos), BlockId::Water | BlockId::Lava) && self.fluid_level(pos) < 8 {
                self.set_block(pos, BlockId::Air);
            }
        }

        if !self.active_sand.is_empty() {
            let mut active_list: Vec<IVec3> = self.active_sand.iter().copied().collect();
            active_list.sort_unstable_by_key(|p| p.y);
            let mut sand_moves = Vec::new();

            for pos in active_list {
                let below = IVec3::new(pos.x, pos.y - 1, pos.z);
                if self.get_block(pos) == BlockId::Sand && below.y >= 0 && self.get_block(below) == BlockId::Air {
                    sand_moves.push((pos, below));
                } else {
                    self.active_sand.remove(&pos);
                }
            }
            for (from, to) in sand_moves {
                if self.get_block(from) == BlockId::Sand && self.get_block(to) == BlockId::Air {
                    self.set_block(from, BlockId::Air);
                    self.set_block(to, BlockId::Sand);
                }
            }
        }
    }

}
