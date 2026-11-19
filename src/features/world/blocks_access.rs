//! World voxel read/write accessors, block overrides, spawn locator, and tile entity lookup.
//! Maintains fluid levels, torch/ruststone indices, dirty chunk marks, and multiplayer deltas.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::physics::{
    block_intersects_player, PLAYER_HALF_WIDTH, STANDING_EYE_OFFSET, STANDING_HEIGHT,
};

use super::biome::{Biome, WorldGeneratorType};
use super::chunk::Chunk;
use super::constants::{CARDINAL_OFFSETS, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use super::gen::{get_biome_at, get_height_at};
use super::tile_entity::{ChestData, FurnaceData};
use super::world_state::{ruststone_light_params, World};

impl World {
    pub fn get_or_create_chest(&mut self, pos: IVec3) -> &mut ChestData {
        let seed = self.seed;
        let is_blacksmith = self.get_block(pos + IVec3::new(-1, 0, 0)) == BlockId::StoneSlab
        || self.get_block(pos + IVec3::new(-2, 0, 0)) == BlockId::Furnace
        || self.get_block(pos + IVec3::new(1, 0, 0)) == BlockId::Furnace;
        self.chests.entry(pos).or_insert_with(|| {
            if is_blacksmith { ChestData::generate_blacksmith_loot(seed, pos) }
            else { ChestData::generate_house_loot(seed, pos) }
        })
    }

    pub fn get_or_create_furnace(&mut self, pos: IVec3) -> &mut FurnaceData {
        self.furnaces.entry(pos).or_insert_with(FurnaceData::new)
    }


    pub fn find_spawn_position(seed: u32) -> Vec3 {
        Self::find_spawn_position_for_type(seed, WorldGeneratorType::Default)
    }

    pub fn find_spawn_position_for_type(seed: u32, generator_type: WorldGeneratorType) -> Vec3 {
        if generator_type == WorldGeneratorType::Superflat {
            return Vec3::new(8.5, 4.0 + STANDING_EYE_OFFSET + 0.1, 8.5);
        }
        let (mut x, mut z, mut dx, mut dz) = (0i32, 0i32, 0i32, -1i32);
        for _ in 0..1500 {
            let (wx, wz) = (x + 8, z + 8);
            let (biome, cont) = get_biome_at(wx, wz, seed);
            if !matches!(biome, Biome::DeepOcean | Biome::Ocean | Biome::Swamp | Biome::MushroomIsland) {
                let height = get_height_at(wx, wz, seed, biome, cont);
                if height >= 24 {
                    return Vec3::new(wx as f32 + 0.5, (height + 3) as f32 + 0.1, wz as f32 + 0.5);
                }
            }
            if x == z || (x < 0 && x == -z) || (x > 0 && x == 1 - z) {
                (dx, dz) = (-dz, dx);
            }
            x += dx * 16;
            z += dz * 16;
        }
        Vec3::new(8.5, 92.0, 8.5)
    }

    pub(super) fn player_position_is_clear(&self, position: Vec3) -> bool {
        let feet_y = position.y - STANDING_EYE_OFFSET;
        let min_x = (position.x - PLAYER_HALF_WIDTH).floor() as i32;
        let max_x = (position.x + PLAYER_HALF_WIDTH).floor() as i32;
        let min_y = feet_y.floor() as i32;
        let max_y = (feet_y + STANDING_HEIGHT).floor() as i32;
        let min_z = (position.z - PLAYER_HALF_WIDTH).floor() as i32;
        let max_z = (position.z + PLAYER_HALF_WIDTH).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let block_pos = IVec3::new(x, y, z);
                    if self.get_block(block_pos).is_solid()
                        && block_intersects_player(
                            block_pos,
                            position,
                            STANDING_EYE_OFFSET,
                            STANDING_HEIGHT,
                        )
                        {
                            return false;
                        }
                }
            }
        }

        true
    }

    pub(super) fn find_clear_spawn_position(&self, position: Vec3) -> Option<Vec3> {
        if self.player_position_is_clear(position) {
            return Some(position);
        }

        for radius in 1i32..=8 {
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx.abs() != radius && dz.abs() != radius {
                        continue;
                    }
                    let candidate =
                    Vec3::new(position.x + dx as f32, position.y, position.z + dz as f32);
                    if self.player_position_is_clear(candidate) {
                        return Some(candidate);
                    }
                }
            }
        }

        None
    }


    pub fn get_biome_at(&self, wx: i32, wz: i32) -> Biome {
        if self.generator_type == WorldGeneratorType::Superflat {
            Biome::Plains
        } else {
            get_biome_at(wx, wz, self.seed).0
        }
    }


    pub fn world_to_chunk(wx: i32, wz: i32) -> (i32, i32, i32, i32) {
        (wx >> 4, wz >> 4, wx & 15, wz & 15)
    }

    #[inline(always)]
    pub fn get_block(&self, pos: IVec3) -> BlockId {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return BlockId::Air;
        }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        self.chunks.get(&(cx, cz)).map_or(BlockId::Air, |c| c.get_local(lx, pos.y, lz))
    }


    pub fn biome_name_at(&self, wx: i32, wz: i32) -> &'static str {
        self.get_biome_at(wx, wz).name()
    }

    #[inline]

    pub fn fluid_level(&self, pos: IVec3) -> u8 {
        self.fluid_levels.get(&pos).copied().unwrap_or(8)
    }

    pub(super) fn set_fluid_level(&mut self, pos: IVec3, level: u8) {
        self.fluid_levels.insert(pos, level.clamp(1, 8));
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockId) {
        self.set_block_inner(pos, block, true);
    }

    pub(super) fn set_block_inner(&mut self, pos: IVec3, block: BlockId, track_network_change: bool) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return;
        }

        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            let previous_block = chunk.get_local(lx, pos.y, lz);
            let changed = previous_block != block;
            chunk.set_local(lx, pos.y, lz, block);
            self.block_overrides
            .entry((cx, cz))
            .or_default()
            .insert(pos, block.to_u8());
            if changed && track_network_change {
                self.pending_network_changes.insert(pos, block.to_u8());
            }

            if matches!(block, BlockId::Water | BlockId::Lava) { self.set_fluid_level(pos, 8); } else { self.fluid_levels.remove(&pos); }
            if block == BlockId::Sand { self.active_sand.insert(pos); } else { self.active_sand.remove(&pos); }
            if block == BlockId::Torch { self.torches.insert(pos); } else { self.torches.remove(&pos); }
            if let Some(params) = ruststone_light_params(block) { self.ruststone_lights.insert(pos, params); } else { self.ruststone_lights.remove(&pos); }
            if block == BlockId::PressurePlatePressed { self.active_pressure_plates.insert(pos); } else { self.active_pressure_plates.remove(&pos); }
            if block == BlockId::Chest { self.chests.entry(pos).or_insert_with(ChestData::new); } else { self.chests.remove(&pos); }
            if matches!(block, BlockId::Furnace | BlockId::FurnaceLit) { self.furnaces.entry(pos).or_insert_with(FurnaceData::new); } else { self.furnaces.remove(&pos); }

            for offset in CARDINAL_OFFSETS {
                let neighbor = pos + offset;
                if self.get_block(neighbor) == BlockId::Sand {
                    self.active_sand.insert(neighbor);
                }
            }

            if lx == 0 { self.mark_dirty(cx - 1, cz); }
            if lx == CHUNK_SIZE_X - 1 { self.mark_dirty(cx + 1, cz); }
            if lz == 0 { self.mark_dirty(cx, cz - 1); }
            if lz == CHUNK_SIZE_Z - 1 { self.mark_dirty(cx, cz + 1); }

            let light_source_changed = previous_block == BlockId::Torch
            || block == BlockId::Torch
            || ruststone_light_params(previous_block).is_some()
            || ruststone_light_params(block).is_some();
            let light_occlusion_changed = previous_block.is_opaque() != block.is_opaque();
            if changed
                && (light_source_changed
                || (light_occlusion_changed && self.has_nearby_light_source(pos)))
                {
                    self.mark_light_affected_chunks_dirty(pos);
                }
        }
    }

    /// Applies a server-provided block, generating its chunk if it is not loaded yet.
    pub fn set_network_block(&mut self, pos: IVec3, block: BlockId) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 {
            return;
        }
        let (cx, cz, _, _) = Self::world_to_chunk(pos.x, pos.z);
        if !self.chunks.contains_key(&(cx, cz)) {
            let mut chunk = Chunk::generate(cx, cz, self.seed);
            self.apply_block_overrides(&mut chunk);
            self.index_chunk_lights(&chunk);
            self.chunks.insert((cx, cz), chunk);
        }
        self.set_block_inner(pos, block, false);
    }

    pub fn take_network_changes(
        &mut self,
    ) -> std::collections::hash_map::IntoIter<IVec3, u8> {
        std::mem::take(&mut self.pending_network_changes).into_iter()
    }

    pub fn block_overrides(&self) -> impl Iterator<Item = (IVec3, u8)> + '_ {
        self.block_overrides
        .values()
        .flat_map(|overrides| overrides.iter().map(|(&pos, &block)| (pos, block)))
    }

    pub(super) fn apply_block_overrides(&self, chunk: &mut Chunk) {
        if let Some(overrides) = self.block_overrides.get(&(chunk.cx, chunk.cz)) {
            for (&pos, &block) in overrides {
                let (_, _, lx, lz) = Self::world_to_chunk(pos.x, pos.z);
                if (pos.y as u32) < CHUNK_SIZE_Y as u32 {
                    chunk.set_local(lx, pos.y, lz, BlockId::from_u8(block));
                }
            }
        }
    }


    pub(super) fn get_block_cached<'a>(&'a self, pos: IVec3, hint: Option<((i32, i32), &'a Chunk)>) -> (BlockId, Option<((i32, i32), &'a Chunk)>) {
        if (pos.y as u32) >= CHUNK_SIZE_Y as u32 { return (BlockId::Air, hint); }
        let (cx, cz, lx, lz) = World::world_to_chunk(pos.x, pos.z);
        if let Some((key, chunk)) = hint {
            if key == (cx, cz) { return (chunk.get_local(lx, pos.y, lz), Some((key, chunk))); }
        }
        match self.chunks.get(&(cx, cz)) {
            Some(chunk) => (chunk.get_local(lx, pos.y, lz), Some(((cx, cz), chunk))),
            None => (BlockId::Air, hint),
        }
    }

    /// `authoritative` is false on remote clients, which only receive fluid changes from the host.
    pub fn bed_pair_at(&self, pos: IVec3) -> Option<(IVec3, IVec3)> {
        if self.get_block(pos) != BlockId::Bed {
            return None;
        }

        for offset in [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ] {
            let other = pos + offset;
            if self.get_block(other) == BlockId::Bed {
                return Some((pos, other));
            }
        }
        None
    }

}
