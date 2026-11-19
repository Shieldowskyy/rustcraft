//! Farming right-click actions, interactive block usage (doors/levers/buttons/containers/beds/eggs), and block placement.
//! Mutates world blocks, tile entities, player spawn point, mobs, achievements/quests, and hotbar counts.
//! Called by `interact.rs` when no higher-priority item or mob interaction consumed the right-click.

use crate::core::math::{IVec3, Vec3};
use crate::gameplay::item::ItemEntity;
use crate::gameplay::physics::block_intersects_player;
use crate::gameplay::player::GameMode;
use crate::world::block::BlockId;

use super::super::{AppOutput, GameState, RustCraftApp};
use super::{block_intersects_live_mob, can_place_block_in};

impl RustCraftApp {
    pub(super) fn try_farming_interaction(&mut self, held_item: Option<BlockId>) -> bool {
        let Some(item) = held_item else {
            return false;
        };
        let Some(hit) = self
            .world
            .raycast(self.camera.position, self.camera.forward(), 8.0)
        else {
            return false;
        };
        let targeted_block = self.world.get_block(hit.block_pos);

        if item.is_hoe()
            && matches!(targeted_block, BlockId::Grass | BlockId::Dirt)
            && hit.normal.y > 0
        {
            self.world.set_block(hit.block_pos, BlockId::Farmland);
            let sound_pos = Vec3::new(
                hit.block_pos.x as f32 + 0.5,
                hit.block_pos.y as f32 + 0.5,
                hit.block_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(crate::audio::SoundId::DigDirt, sound_pos, 0.85);
            self.hand_animation = 1.0;
            return true;
        }

        if item == BlockId::BoneMeal {
            let center = Vec3::new(
                hit.block_pos.x as f32 + 0.5,
                hit.block_pos.y as f32 + 0.5,
                hit.block_pos.z as f32 + 0.5,
            );
            if targeted_block == BlockId::Sapling {
                self.world.grow_tree_at(hit.block_pos);
                self.particles.spawn_growth(center);
                self.play_world_sound_varied(crate::audio::SoundId::DigGrass, center, 0.85);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            }
            if matches!(
                targeted_block,
                BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop
            ) {
                self.particles.spawn_growth(center);
                let drop_vel = Vec3::new(0.0, 2.5, 0.0);
                match targeted_block {
                    BlockId::WheatCrop => {
                        self.items
                            .push(ItemEntity::new(center, drop_vel, BlockId::Wheat, 1));
                        self.items
                            .push(ItemEntity::new(center, drop_vel, BlockId::WheatSeeds, 2));
                    }
                    BlockId::CarrotCrop => {
                        self.items
                            .push(ItemEntity::new(center, drop_vel, BlockId::Carrot, 2));
                    }
                    BlockId::PotatoCrop => {
                        self.items
                            .push(ItemEntity::new(center, drop_vel, BlockId::Potato, 2));
                    }
                    _ => {}
                }
                self.world.set_block(hit.block_pos, BlockId::Air);
                self.consume_selected_hotbar_one();
                self.play_world_sound_varied(crate::audio::SoundId::DigGrass, center, 0.85);
                self.hand_animation = 1.0;
                return true;
            }
        }

        if matches!(item, BlockId::WheatSeeds | BlockId::Carrot | BlockId::Potato)
            && matches!(targeted_block, BlockId::Farmland | BlockId::FarmlandWet)
            && hit.normal.y > 0
        {
            let plant_pos = hit.block_pos + hit.normal;
            if self.world.get_block(plant_pos) == BlockId::Air {
                let crop = match item {
                    BlockId::WheatSeeds => BlockId::WheatCrop,
                    BlockId::Carrot => BlockId::CarrotCrop,
                    BlockId::Potato => BlockId::PotatoCrop,
                    _ => unreachable!(),
                };
                self.world.set_block(plant_pos, crop);
                let sound_pos = Vec3::new(
                    plant_pos.x as f32 + 0.5,
                    plant_pos.y as f32 + 0.5,
                    plant_pos.z as f32 + 0.5,
                );
                self.play_world_sound_varied(crate::audio::SoundId::DigGrass, sound_pos, 0.85);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            }
        }

        if item == BlockId::Sapling
            && matches!(targeted_block, BlockId::Grass | BlockId::Dirt)
            && hit.normal.y > 0
        {
            let plant_pos = hit.block_pos + hit.normal;
            if self.world.get_block(plant_pos) == BlockId::Air {
                self.world.set_block(plant_pos, BlockId::Sapling);
                let sound_pos = Vec3::new(
                    plant_pos.x as f32 + 0.5,
                    plant_pos.y as f32 + 0.5,
                    plant_pos.z as f32 + 0.5,
                );
                self.play_world_sound_varied(crate::audio::SoundId::DigGrass, sound_pos, 0.85);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            }
        }

        false
    }

    pub(super) fn try_world_block_use_or_place(
        &mut self,
        is_crouching: bool,
        selected_block_id: u8,
        out: &mut AppOutput,
    ) {
        let Some(hit) = self
            .world
            .raycast(self.camera.position, self.camera.forward(), 8.0)
        else {
            return;
        };
        let targeted_block = self.world.get_block(hit.block_pos);

        if matches!(targeted_block, BlockId::Door | BlockId::DoorOpen) && !is_crouching {
            let new_block = if targeted_block == BlockId::Door {
                BlockId::DoorOpen
            } else {
                BlockId::Door
            };
            self.world.set_block(hit.block_pos, new_block);
            let door_sound = if new_block == BlockId::DoorOpen {
                crate::audio::SoundId::DoorOpen
            } else {
                crate::audio::SoundId::DoorClose
            };
            let sound_pos = Vec3::new(
                hit.block_pos.x as f32 + 0.5,
                hit.block_pos.y as f32 + 0.5,
                hit.block_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(door_sound, sound_pos, 0.85);
            let above = hit.block_pos + IVec3::new(0, 1, 0);
            let below = hit.block_pos - IVec3::new(0, 1, 0);
            if matches!(self.world.get_block(above), BlockId::Door | BlockId::DoorOpen) {
                self.world.set_block(above, new_block);
            } else if matches!(self.world.get_block(below), BlockId::Door | BlockId::DoorOpen) {
                self.world.set_block(below, new_block);
            }
            self.hand_animation = 1.0;
            crate::log_info!(
                "WORLD",
                "Toggled door at ({}, {}, {}) to {:?}",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z,
                new_block
            );
        } else if matches!(targeted_block, BlockId::Lever | BlockId::LeverOn) && !is_crouching {
            let new_block = if targeted_block == BlockId::Lever {
                BlockId::LeverOn
            } else {
                BlockId::Lever
            };
            self.world.set_block(hit.block_pos, new_block);
            let sound_pos = Vec3::new(
                hit.block_pos.x as f32 + 0.5,
                hit.block_pos.y as f32 + 0.5,
                hit.block_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.85);
            self.hand_animation = 1.0;
            self.world.update_ruststone_network(hit.block_pos, 16);
            crate::log_info!(
                "WORLD",
                "Toggled lever at ({}, {}, {}) to {:?}",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z,
                new_block
            );
        } else if targeted_block == BlockId::Button && !is_crouching {
            self.world.set_block(hit.block_pos, BlockId::ButtonPressed);
            let sound_pos = Vec3::new(
                hit.block_pos.x as f32 + 0.5,
                hit.block_pos.y as f32 + 0.5,
                hit.block_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.9);
            self.world.button_timers.insert(hit.block_pos, 1.0);
            self.hand_animation = 1.0;
            self.world.update_ruststone_network(hit.block_pos, 16);
            crate::log_info!(
                "WORLD",
                "Pressed button at ({}, {}, {})",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z
            );
        } else if targeted_block == BlockId::CraftingTable && !is_crouching {
            self.crafting_table_open = true;
            self.recipe_book_open = false;
            self.recipe_book_scroll = 0;
            self.recipe_book_status = None;
            if self.player.gamemode != GameMode::Creative {
                self.record_inventory_items_collected();
            }
            self.state = GameState::Inventory;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!(
                "MENU",
                "Opened crafting table at ({}, {}, {})",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z
            );
        } else if targeted_block == BlockId::Chest && !is_crouching {
            let _ = self.world.get_or_create_chest(hit.block_pos);
            self.state = GameState::Chest { pos: hit.block_pos };
            self.audio
                .play_sound(crate::audio::SoundId::ChestOpen, 0.8, 1.0);
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!(
                "MENU",
                "Opened chest container at ({}, {}, {})",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z
            );
        } else if matches!(targeted_block, BlockId::Furnace | BlockId::FurnaceLit) && !is_crouching
        {
            let _ = self.world.get_or_create_furnace(hit.block_pos);
            self.state = GameState::Furnace { pos: hit.block_pos };
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!(
                "MENU",
                "Opened furnace container at ({}, {}, {})",
                hit.block_pos.x,
                hit.block_pos.y,
                hit.block_pos.z
            );
        } else if targeted_block == BlockId::Bed && !is_crouching {
            let spawn_pos = self
                .world
                .bed_pair_at(hit.block_pos)
                .map(|(foot, _)| foot)
                .unwrap_or(hit.block_pos);
            self.player.spawn_point = Some(spawn_pos);
            let is_night = self.world.time_of_day > 260.0 || self.world.time_of_day < 90.0;
            let is_storm_or_rain = self.world.weather.is_raining();
            if is_night || is_storm_or_rain {
                self.world.time_of_day = 120.0;
                self.world.weather.clear();
                self.player.health = (self.player.health + 4.0).min(self.player.max_health);
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.8, 1.0);
                if is_storm_or_rain && !is_night {
                    self.chat.add_message(
                        "Przespano burze. Pogoda sie rozpogodzila!".to_string(),
                        crate::ui::pack_color(0.9, 0.9, 0.3),
                    );
                } else {
                    self.chat.add_message(
                        "Przespano noc. Punkt odrodzenia ustawiony!".to_string(),
                        crate::ui::pack_color(0.9, 0.9, 0.3),
                    );
                }
                crate::log_info!(
                    "PLAYER",
                    "Player slept in bed at ({}, {}, {}). Time set to morning, weather cleared.",
                    spawn_pos.x,
                    spawn_pos.y,
                    spawn_pos.z
                );
            } else {
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                self.chat.add_message(
                    "Punkt odrodzenia ustawiony! Mozesz spac tylko w nocy lub podczas burzy."
                        .to_string(),
                    crate::ui::pack_color(0.9, 0.9, 0.3),
                );
                crate::log_info!(
                    "PLAYER",
                    "Spawn point set to bed at ({}, {}, {}). Day time, cannot sleep.",
                    spawn_pos.x,
                    spawn_pos.y,
                    spawn_pos.z
                );
            }
        } else if let Some(mob_kind) = BlockId::from_u8(selected_block_id).spawn_mob_kind() {
            let spawn_pos = Vec3::new(
                hit.block_pos.x as f32 + 0.5 + hit.normal.x as f32 * 0.5,
                hit.block_pos.y as f32 + if hit.normal.y > 0 { 1.0 } else { 0.0 },
                hit.block_pos.z as f32 + 0.5 + hit.normal.z as f32 * 0.5,
            );
            let delegated_to_authority = self.net_client.as_mut().is_some_and(|client| {
                if client.is_connected && client.local_id != client.mob_authority_id {
                    client.send_spawn_mob(mob_kind, spawn_pos);
                    true
                } else {
                    false
                }
            });
            if !delegated_to_authority {
                self.mob_manager.spawn_mob(mob_kind, spawn_pos);
            }
            self.particles.spawn_mob_hit(spawn_pos);
            if let Some(snd) = mob_kind.idle_sound() {
                self.play_world_sound_varied(snd, spawn_pos, 0.9);
            }
            self.hand_animation = 1.0;
            crate::log_info!(
                "WORLD",
                "Spawned mob {:?} via egg at ({:.1}, {:.1}, {:.1})",
                mob_kind,
                spawn_pos.x,
                spawn_pos.y,
                spawn_pos.z
            );
            self.consume_selected_hotbar_one();
        } else {
            self.try_place_selected_block(hit.block_pos + hit.normal, selected_block_id);
        }
    }

    fn try_place_selected_block(&mut self, place_pos: IVec3, selected_block_id: u8) {
        let place_id = BlockId::from_u8(selected_block_id);
        let existing = self.world.get_block(place_pos);
        if place_id == BlockId::Air || !place_id.is_placeable() || !can_place_block_in(existing) {
            return;
        }

        self.hand_animation = 1.0;
        if place_id == BlockId::Bed {
            let forward = self.camera.forward_flat();
            let offset = if forward.x.abs() > forward.z.abs() {
                IVec3::new(if forward.x > 0.0 { 1 } else { -1 }, 0, 0)
            } else {
                IVec3::new(0, 0, if forward.z > 0.0 { 1 } else { -1 })
            };
            let head_pos = place_pos + offset;
            let foot_support = self.world.get_block(place_pos - IVec3::new(0, 1, 0));
            let head_support = self.world.get_block(head_pos - IVec3::new(0, 1, 0));
            if self.world.get_block(head_pos) == BlockId::Air
                && foot_support.is_solid()
                && head_support.is_solid()
                && !block_intersects_player(
                    place_pos,
                    self.camera.position,
                    self.current_eye_offset,
                    self.current_player_height,
                )
                && !block_intersects_player(
                    head_pos,
                    self.camera.position,
                    self.current_eye_offset,
                    self.current_player_height,
                )
                && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
                && !block_intersects_live_mob(head_pos, &self.mob_manager.mobs)
            {
                self.world.set_block(place_pos, BlockId::Bed);
                self.world.set_block(head_pos, BlockId::Bed);
                let unlocked = self.achievements.record_placed();
                self.process_achievement_unlocks(unlocked);
                let q_place = self.quests.record_placed(BlockId::Bed);
                self.process_quest_completions(q_place);
                let sound_pos = Vec3::new(
                    place_pos.x as f32 + 0.5,
                    place_pos.y as f32 + 0.5,
                    place_pos.z as f32 + 0.5,
                );
                self.play_world_sound_varied(crate::audio::SoundId::PlaceWood, sound_pos, 0.85);
                self.request_rumble(0.2, 0.1, 45);
                self.consume_selected_hotbar_one();
            }
        } else if place_id == BlockId::Door {
            let top_pos = place_pos + IVec3::new(0, 1, 0);
            if !block_intersects_player(
                place_pos,
                self.camera.position,
                self.current_eye_offset,
                self.current_player_height,
            ) && !block_intersects_player(
                top_pos,
                self.camera.position,
                self.current_eye_offset,
                self.current_player_height,
            ) && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
                && !block_intersects_live_mob(top_pos, &self.mob_manager.mobs)
                && self.world.get_block(top_pos) == BlockId::Air
            {
                self.world.set_block(place_pos, BlockId::Door);
                self.world.set_block(top_pos, BlockId::Door);
                let unlocked = self.achievements.record_placed();
                self.process_achievement_unlocks(unlocked);
                let q_place = self.quests.record_placed(BlockId::Door);
                self.process_quest_completions(q_place);
                let sound_pos = Vec3::new(
                    place_pos.x as f32 + 0.5,
                    place_pos.y as f32 + 0.5,
                    place_pos.z as f32 + 0.5,
                );
                self.play_world_sound_varied(crate::audio::SoundId::PlaceWood, sound_pos, 0.85);
                self.request_rumble(0.2, 0.1, 45);
                crate::log_info!(
                    "WORLD",
                    "Placed door at ({}, {}, {})",
                    place_pos.x,
                    place_pos.y,
                    place_pos.z
                );
                self.consume_selected_hotbar_one();
            }
        } else if !block_intersects_player(
            place_pos,
            self.camera.position,
            self.current_eye_offset,
            self.current_player_height,
        ) && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
        {
            self.world.set_block(place_pos, place_id);
            let unlocked = self.achievements.record_placed();
            self.process_achievement_unlocks(unlocked);
            let q_place = self.quests.record_placed(place_id);
            self.process_quest_completions(q_place);
            let place_sound = crate::audio::SoundId::from_block_place(place_id);
            let sound_pos = Vec3::new(
                place_pos.x as f32 + 0.5,
                place_pos.y as f32 + 0.5,
                place_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(place_sound, sound_pos, 0.85);
            self.request_rumble(0.2, 0.1, 45);
            if matches!(
                place_id,
                BlockId::Tnt
                    | BlockId::RustStone
                    | BlockId::RustStoneBlock
                    | BlockId::RustStoneLamp
                    | BlockId::Lever
                    | BlockId::Button
                    | BlockId::PressurePlate
            ) {
                self.world.update_ruststone_network(place_pos, 16);
            }
            crate::log_info!(
                "WORLD",
                "Placed block {:?} at ({}, {}, {})",
                place_id,
                place_pos.x,
                place_pos.y,
                place_pos.z
            );
            self.consume_selected_hotbar_one();
        }
    }
}
