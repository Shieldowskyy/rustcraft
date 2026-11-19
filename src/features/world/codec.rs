//! Binary world save/load serialization (`RUSTCRAFT1`..`RUSTCRAFT8`) and checked slice readers.
//! Serializes chunks, overrides, fluids, tile entities, and saved player stats.

use std::collections::HashMap;

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::player::{GameMode, ItemStack, SavedPlayerData};

use super::biome::WorldGeneratorType;
use super::chunk::Chunk;
use super::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use super::tile_entity::{ChestData, FurnaceData};
use super::world_state::World;

impl World {
    pub fn save(&self, path: &str) -> Result<(), String> {
        crate::log_info!("SAVE", "Serializing world '{}' (seed: {}, mode: {:?}, type: {:?}, chunks: {}, chests: {}, furnaces: {}) to '{}'", self.name, self.seed, self.gamemode, self.generator_type, self.chunks.len(), self.chests.len(), self.furnaces.len(), path);
        let mut bytes = Vec::with_capacity(32 + self.chunks.len() * 4);
        bytes.extend_from_slice(b"RUSTCRAFT8");
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        bytes.push(match self.gamemode { GameMode::Survival => 0, GameMode::Creative => 1 });
        bytes.push(match self.generator_type { WorldGeneratorType::Default => 0, WorldGeneratorType::Superflat => 1 });

        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(255) as u8;
        bytes.push(name_len);
        bytes.extend_from_slice(&name_bytes[..name_len as usize]);

        for f in [self.player_position.x, self.player_position.y, self.player_position.z] {
            bytes.extend_from_slice(&f.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());
        for chunk in self.chunks.values() {
            bytes.extend_from_slice(&chunk.cx.to_le_bytes());
            bytes.extend_from_slice(&chunk.cz.to_le_bytes());
            bytes.extend_from_slice(&(chunk.blocks.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&chunk.blocks);
        }

        let push_pos = |b: &mut Vec<u8>, pos: &IVec3| {
            b.extend_from_slice(&pos.x.to_le_bytes());
            b.extend_from_slice(&pos.y.to_le_bytes());
            b.extend_from_slice(&pos.z.to_le_bytes());
        };
        let push_item = |b: &mut Vec<u8>, item: &Option<ItemStack>| {
            if let Some(s) = item { b.extend_from_slice(&[s.block.to_u8(), s.count]); }
            else { b.extend_from_slice(&[0, 0]); }
        };

        bytes.extend_from_slice(&(self.fluid_levels.len() as u32).to_le_bytes());
        for (pos, &level) in &self.fluid_levels {
            push_pos(&mut bytes, pos);
            bytes.push(level);
        }

        bytes.extend_from_slice(&(self.chests.len() as u32).to_le_bytes());
        for (pos, chest) in &self.chests {
            push_pos(&mut bytes, pos);
            for item in &chest.items { push_item(&mut bytes, item); }
        }

        bytes.extend_from_slice(&(self.furnaces.len() as u32).to_le_bytes());
        for (pos, furnace) in &self.furnaces {
            push_pos(&mut bytes, pos);
            for s in [&furnace.input, &furnace.fuel, &furnace.output] { push_item(&mut bytes, s); }
            for f in [furnace.burn_time_remaining, furnace.burn_time_total, furnace.cook_progress] {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
        }

        if let Some(ref p) = self.saved_player {
            bytes.push(1);
            for f in [p.health, p.max_health, p.hunger, p.max_hunger, p.saturation, p.exhaustion, p.air] {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            bytes.push(p.selected_hotbar_slot);
            for item in p.hotbar.iter().chain(p.main.iter()).chain(p.armor.iter()).chain(std::iter::once(&p.offhand)) {
                push_item(&mut bytes, item);
            }
        } else {
            bytes.push(0);
        }

        let override_count: usize = self.block_overrides.values().map(HashMap::len).sum();
        bytes.extend_from_slice(&(override_count as u32).to_le_bytes());
        for overrides in self.block_overrides.values() {
            for (pos, &block) in overrides {
                bytes.extend_from_slice(&pos.x.to_le_bytes());
                bytes.extend_from_slice(&pos.y.to_le_bytes());
                bytes.extend_from_slice(&pos.z.to_le_bytes());
                bytes.push(block);
            }
        }

        std::fs::write(path, bytes).map_err(|e| format!("failed to save world: {e}"))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        crate::log_info!("SAVE", "Loading world from file '{}'...", path);
        let bytes = std::fs::read(path).map_err(|e| format!("failed to read world file: {e}"))?;
        let mut cursor = 0usize;
        let format = read_bytes(&bytes, &mut cursor, 10)?;
        let version = match format {
            b"RUSTCRAFT1" => 1, b"RUSTCRAFT2" => 2, b"RUSTCRAFT3" => 3,
            b"RUSTCRAFT4" => 4, b"RUSTCRAFT5" => 5, b"RUSTCRAFT6" => 6,
            b"RUSTCRAFT7" => 7, b"RUSTCRAFT8" => 8,
            _ => return Err("unknown world file format".to_owned()),
        };

        let read_u32 = |c: &mut usize| -> Result<u32, String> { Ok(u32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_i32 = |c: &mut usize| -> Result<i32, String> { Ok(i32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_f32 = |c: &mut usize| -> Result<f32, String> { Ok(f32::from_le_bytes(read_bytes(&bytes, c, 4)?.try_into().unwrap())) };
        let read_item = |c: &mut usize| -> Option<ItemStack> {
            read_bytes(&bytes, c, 2).ok().and_then(|ib| (ib[0] > 0 && ib[1] > 0).then(|| ItemStack::new(BlockId::from_u8(ib[0]), ib[1])))
        };

        let seed = read_u32(&mut cursor)?;
        let (gamemode, generator_type, name) = if version >= 8 {
            let gm = if read_bytes(&bytes, &mut cursor, 1)?[0] == 1 { GameMode::Creative } else { GameMode::Survival };
            let gen = if read_bytes(&bytes, &mut cursor, 1)?[0] == 1 { WorldGeneratorType::Superflat } else { WorldGeneratorType::Default };
            let name_len = read_bytes(&bytes, &mut cursor, 1)?[0] as usize;
            let name_str = if name_len > 0 { String::from_utf8_lossy(read_bytes(&bytes, &mut cursor, name_len)?).to_string() } else { format!("World {}", seed) };
            (gm, gen, name_str)
        } else if version >= 4 {
            let gm = if read_bytes(&bytes, &mut cursor, 1)?[0] == 1 { GameMode::Creative } else { GameMode::Survival };
            let name_len = read_bytes(&bytes, &mut cursor, 1)?[0] as usize;
            let name_str = if name_len > 0 { String::from_utf8_lossy(read_bytes(&bytes, &mut cursor, name_len)?).to_string() } else { format!("World {}", seed) };
            (gm, WorldGeneratorType::Default, name_str)
        } else {
            (GameMode::Survival, WorldGeneratorType::Default, format!("World {}", seed))
        };

        let player_position = if version >= 2 {
            Vec3::new(read_f32(&mut cursor)?, read_f32(&mut cursor)?, read_f32(&mut cursor)?)
        } else {
            Self::find_spawn_position_for_type(seed, generator_type)
        };

        let chunk_count = read_u32(&mut cursor)? as usize;
        let expected_len = (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize;
        let mut world = Self::new_with_generator(seed, generator_type);
        world.name = name;
        world.gamemode = gamemode;
        world.generator_type = generator_type;
        world.player_position = player_position;

        for _ in 0..chunk_count {
            let (cx, cz, block_len) = (read_i32(&mut cursor)?, read_i32(&mut cursor)?, read_u32(&mut cursor)? as usize);
            if block_len != expected_len {
                return Err("invalid chunk data length in world file".to_owned());
            }
            let blocks = read_bytes(&bytes, &mut cursor, block_len)?.to_vec();
            world.chunks.insert((cx, cz), Chunk::new(cx, cz, blocks, true));
        }

        if version >= 3 {
            let fluid_count = read_u32(&mut cursor)? as usize;
            for _ in 0..fluid_count {
                let pos = IVec3::new(read_i32(&mut cursor)?, read_i32(&mut cursor)?, read_i32(&mut cursor)?);
                let level = read_bytes(&bytes, &mut cursor, 1)?[0].clamp(1, 8);
                world.fluid_levels.insert(pos, level);
            }
        }

        if version >= 5 && bytes.len() > cursor {
            if let Ok(chest_count) = read_u32(&mut cursor) {
                for _ in 0..chest_count {
                    if let (Ok(x), Ok(y), Ok(z)) = (read_i32(&mut cursor), read_i32(&mut cursor), read_i32(&mut cursor)) {
                        let mut chest = ChestData::new();
                        for slot in &mut chest.items { *slot = read_item(&mut cursor); }
                        world.chests.insert(IVec3::new(x, y, z), chest);
                    }
                }
            }
            if let Ok(furnace_count) = read_u32(&mut cursor) {
                for _ in 0..furnace_count {
                    if let (Ok(x), Ok(y), Ok(z)) = (read_i32(&mut cursor), read_i32(&mut cursor), read_i32(&mut cursor)) {
                        let mut furnace = FurnaceData::new();
                        for slot in [&mut furnace.input, &mut furnace.fuel, &mut furnace.output] { *slot = read_item(&mut cursor); }
                        if let (Ok(rem), Ok(tot), Ok(prog)) = (read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor)) {
                            furnace.burn_time_remaining = rem;
                            furnace.burn_time_total = tot;
                            furnace.cook_progress = prog;
                        }
                        world.furnaces.insert(IVec3::new(x, y, z), furnace);
                    }
                }
            }
        }

        if version >= 6 && bytes.len() > cursor {
            if let Ok(has_p) = read_bytes(&bytes, &mut cursor, 1) {
                if has_p[0] == 1 {
                    if let (Ok(health), Ok(max_health), Ok(hunger), Ok(max_hunger), Ok(saturation), Ok(exhaustion), Ok(air), Ok(slot)) = (
                        read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor),
                                                                                                                                          read_f32(&mut cursor), read_f32(&mut cursor), read_f32(&mut cursor), read_bytes(&bytes, &mut cursor, 1)
                    ) {
                        let mut hotbar = [None; 10];
                        for s in &mut hotbar { *s = read_item(&mut cursor); }
                        let mut main = [None; 27];
                        for s in &mut main { *s = read_item(&mut cursor); }
                        let mut armor = [None; 4];
                        for s in &mut armor { *s = read_item(&mut cursor); }
                        let offhand = read_item(&mut cursor);

                        world.saved_player = Some(SavedPlayerData {
                            health, max_health, hunger, max_hunger, saturation, exhaustion, air,
                            selected_hotbar_slot: slot[0], hotbar, main, armor, offhand,
                        });
                    }
                }
            }
        }

        if version >= 7 {
            let override_count = read_u32(&mut cursor)? as usize;
            let max_count = bytes.len().saturating_sub(cursor) / 13;
            if override_count > max_count {
                return Err("invalid block override count in world file".to_owned());
            }
            for _ in 0..override_count {
                let pos = IVec3::new(
                    read_i32(&mut cursor)?,
                                     read_i32(&mut cursor)?,
                                     read_i32(&mut cursor)?,
                );
                let block = read_bytes(&bytes, &mut cursor, 1)?[0];
                let (cx, cz, _, _) = Self::world_to_chunk(pos.x, pos.z);
                world
                .block_overrides
                .entry((cx, cz))
                .or_default()
                .insert(pos, block);
            }
        } else {
            for (&(cx, cz), chunk) in &world.chunks {
                let generated = Chunk::generate(cx, cz, world.seed);
                for y in 0..CHUNK_SIZE_Y {
                    for z in 0..CHUNK_SIZE_Z {
                        for x in 0..CHUNK_SIZE_X {
                            let index = Chunk::index(x, y, z);
                            if chunk.blocks[index] != generated.blocks[index] {
                                world
                                .block_overrides
                                .entry((cx, cz))
                                .or_default()
                                .insert(
                                    IVec3::new(cx * CHUNK_SIZE_X + x, y, cz * CHUNK_SIZE_Z + z),
                                        chunk.blocks[index],
                                );
                            }
                        }
                    }
                }
            }
        }
        crate::log_info!("SAVE", "Successfully loaded world '{}' (seed: {}, mode: {:?}, chunks: {}, chests: {}, furnaces: {})", world.name, world.seed, world.gamemode, world.chunks.len(), world.chests.len(), world.furnaces.len());
        Ok(world)
    }
}

pub(crate) fn read_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, count: usize) -> Result<&'a [u8], String> {
    if bytes.len().saturating_sub(*cursor) < count {
        return Err("corrupted world file".to_owned());
    }
    let slice = &bytes[*cursor..*cursor + count];
    *cursor += count;
    Ok(slice)
}
