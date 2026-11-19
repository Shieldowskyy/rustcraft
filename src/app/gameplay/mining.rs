//! Left-click melee combat, block mining/drops/cascades, and Creative middle-click pick-block.
//! Mutates world blocks, chests/furnaces, mob health, dropped items, tool durability, and break progress.
//! Triggered per-frame during `GameState::Playing` after movement and hotbar updates.

use crate::core::controls::Action;
use crate::core::math::{IVec3, Vec3};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::player::GameMode;
use crate::world::block::{BlockId, Face};

use super::super::{InputState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn handle_mining_and_combat(&mut self, dt: f32, _input: &InputState) {
        if self.creative_break_cooldown > 0.0 {
            self.creative_break_cooldown = (self.creative_break_cooldown - dt).max(0.0);
        }

        let interact_left = !self.chat.is_open && self.controls.is_active(Action::AttackBreak);
        let interact_left_just =
            !self.chat.is_open && self.controls.is_just_pressed(Action::AttackBreak);

        if !interact_left {
            self.break_target = None;
            self.break_progress = 0.0;
            self.dig_sound_timer = 0.0;
            self.creative_break_cooldown = 0.0;
            return;
        }

        if self.hand_animation < 0.1 {
            self.hand_animation = 1.0;
        }
        let forward = self.camera.forward();
        let held_tool = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
            .map(|s| s.block);
        let dmg = held_tool.map(|b| b.attack_damage()).unwrap_or(1.5);
        let mut entity_hit = false;
        let mut entity_attacked = false;
        let block_hit = self.world.raycast(self.camera.position, forward, 8.0);
        if let Some(hit) = &block_hit {
            self.record_discovered_block(self.world.get_block(hit.block_pos));
        }
        let block_distance = block_hit.as_ref().map_or(8.0, |hit| hit.distance);
        let entity_reach = 4.2_f32.min((block_distance - 0.001).max(0.0));

        if let Some((target_id, hit_pos)) =
            self.raycast_remote_player(self.camera.position, forward, entity_reach)
        {
            entity_hit = true;
            if interact_left_just {
                if let Some(client) = &mut self.net_client {
                    client.send_attack_player(target_id, dmg);
                }
                self.particles.spawn_mob_hit(hit_pos);
                self.break_target = None;
                self.break_progress = 0.0;
                self.request_rumble(0.65, 0.65, 120);
                crate::log_info!(
                    "COMBAT",
                    "Player attacked remote player ID {} dealing {:.1} damage",
                    target_id,
                    dmg
                );
                entity_attacked = true;
            }
        } else if let Some((mob_idx, _)) =
            self.mob_manager
                .raycast_mob(self.camera.position, forward, entity_reach)
        {
            entity_hit = true;
            if interact_left_just {
                let (mob_id, kind, remaining_hp, mob_pos, was_alive) = {
                    let mob = &mut self.mob_manager.mobs[mob_idx];
                    let was_alive = !mob.is_dead;
                    let hit_pos = mob.center_pos();
                    mob.take_damage(dmg, forward);
                    self.particles.spawn_mob_hit(hit_pos);
                    (mob.id, mob.kind, mob.health, mob.pos, was_alive)
                };
                if let Some(client) = &mut self.net_client {
                    client.send_attack_mob(mob_id, dmg);
                }
                if was_alive && remaining_hp <= 0.0 {
                    let unlocked = self.achievements.record_defeated_mob();
                    self.process_achievement_unlocks(unlocked);
                    let q_mob = self.quests.record_mob_killed(kind);
                    self.process_quest_completions(q_mob);
                }
                if kind == crate::gameplay::mob::MobKind::Villager {
                    self.mob_manager.anger_golems_at_villager_attack(mob_pos);
                }
                self.break_target = None;
                self.break_progress = 0.0;
                self.request_rumble(0.65, 0.65, 120);
                crate::log_info!(
                    "COMBAT",
                    "Player attacked mob {:?} dealing {:.1} damage (remaining HP: {:.1})",
                    kind,
                    dmg,
                    remaining_hp
                );
                entity_attacked = true;
                self.process_mob_sounds();
            }
        }

        if entity_attacked && self.player.gamemode == GameMode::Survival {
            let selected = self.player.selected_hotbar_slot as usize;
            if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                if stack.block.is_damageable() {
                    let cost = if stack.block.is_sword() { 1 } else { 2 };
                    if stack.damage(cost) {
                        self.player.inventory.hotbar[selected] = None;
                        self.audio
                            .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                    }
                }
            }
        }

        if entity_hit {
            return;
        }

        if self.player.gamemode == GameMode::Creative && self.creative_break_cooldown > 0.0 {
            self.break_target = None;
            self.break_progress = 0.0;
            return;
        }

        let Some(hit) = block_hit else {
            self.break_target = None;
            self.break_progress = 0.0;
            self.dig_sound_timer = 0.0;
            return;
        };

        let target = hit.block_pos;
        let broken_block = self.world.get_block(target);

        if broken_block == BlockId::Tnt && interact_left_just {
            self.world.prime_tnt(target);
            self.break_target = None;
            self.break_progress = 0.0;
            self.dig_sound_timer = 0.0;
            crate::log_info!(
                "WORLD",
                "TNT primed at ({}, {}, {})",
                target.x,
                target.y,
                target.z
            );
            return;
        }

        if self.break_target != Some(target) {
            self.break_target = Some(target);
            self.break_progress = 0.0;
            self.dig_sound_timer = 0.0;
        }

        let sound_pos = Vec3::new(
            target.x as f32 + 0.5,
            target.y as f32 + 0.5,
            target.z as f32 + 0.5,
        );
        self.dig_sound_timer -= dt;
        if interact_left_just || self.dig_sound_timer <= 0.0 {
            self.dig_sound_timer = 0.22;
            let dig_sound = crate::audio::SoundId::from_block_dig(broken_block);
            self.play_world_sound_varied(dig_sound, sound_pos, 0.78);
        }

        let break_time = if self.player.gamemode == GameMode::Creative {
            0.0
        } else {
            broken_block.break_time_with_tool(held_tool)
        };

        if break_time == 0.0 {
            self.break_progress = 1.0;
        } else {
            self.break_progress += dt / break_time;
        }

        if self.break_progress >= 1.0 {
            self.finish_breaking_block(target, broken_block, held_tool);
        }
    }

    fn finish_breaking_block(
        &mut self,
        target: IVec3,
        broken_block: BlockId,
        held_tool: Option<BlockId>,
    ) {
        self.break_progress = 0.0;
        self.break_target = None;
        self.dig_sound_timer = 0.0;
        self.hand_animation = 1.0;
        if self.player.gamemode == GameMode::Creative {
            self.creative_break_cooldown = 0.25;
        }

        let bx = target.x as f32 + 0.5;
        let by = target.y as f32 + 0.5;
        let bz = target.z as f32 + 0.5;
        let color = broken_block.face_color(Face::Top);
        self.particles
            .spawn_block_break(Vec3::new(bx - 0.5, by - 0.5, bz - 0.5), color);

        let chest_data = if broken_block == BlockId::Chest {
            self.world.chests.remove(&target)
        } else {
            None
        };
        let furnace_data = if matches!(broken_block, BlockId::Furnace | BlockId::FurnaceLit) {
            self.world.furnaces.remove(&target)
        } else {
            None
        };

        let bed_mate = if broken_block == BlockId::Bed {
            self.world
                .bed_pair_at(target)
                .map(|(first, second)| if first == target { second } else { first })
        } else {
            None
        };
        let replace_block =
            if matches!(broken_block, BlockId::Kelp | BlockId::Seagrass) && target.y <= 20 {
                BlockId::Water
            } else {
                BlockId::Air
            };
        self.world.set_block(target, replace_block);
        self.record_discovered_block(broken_block);
        let unlocked = self.achievements.record_mined(broken_block);
        self.process_achievement_unlocks(unlocked);
        let q_mine = self.quests.record_mined(broken_block);
        self.process_quest_completions(q_mine);
        if let Some(mate) = bed_mate {
            self.world.set_block(mate, BlockId::Air);
        }

        // Breaking Kelp breaks all kelp blocks directly above it
        if broken_block == BlockId::Kelp {
            let mut check_pos = target + IVec3::new(0, 1, 0);
            while self.world.get_block(check_pos) == BlockId::Kelp {
                let rep = if check_pos.y <= 20 {
                    BlockId::Water
                } else {
                    BlockId::Air
                };
                self.world.set_block(check_pos, rep);
                if self.player.gamemode == GameMode::Survival {
                    let drop_pos = Vec3::new(
                        check_pos.x as f32 + 0.5,
                        check_pos.y as f32 + 0.5,
                        check_pos.z as f32 + 0.5,
                    );
                    let r_drop = Vec3::new(
                        ((check_pos.y as f32 * 13.0).sin()) * 1.2,
                        2.8,
                        ((check_pos.y as f32 * 17.0).cos()) * 1.2,
                    );
                    self.items
                        .push(ItemEntity::new(drop_pos, r_drop, BlockId::Kelp, 1));
                }
                check_pos.y += 1;
            }
        }

        let break_sound = crate::audio::SoundId::from_block_break(broken_block);
        self.play_world_sound_varied(break_sound, Vec3::new(bx, by, bz), 0.95);
        self.request_rumble(0.4, 0.25, 75);
        crate::log_info!(
            "WORLD",
            "Block broken: {:?} at ({}, {}, {})",
            broken_block,
            target.x,
            target.y,
            target.z
        );

        if self.player.gamemode == GameMode::Survival {
            self.spawn_survival_break_drops(target, broken_block, held_tool, chest_data, furnace_data);
        }

        if matches!(broken_block, BlockId::Door | BlockId::DoorOpen) {
            let above = target + IVec3::new(0, 1, 0);
            let below = target - IVec3::new(0, 1, 0);
            if matches!(self.world.get_block(above), BlockId::Door | BlockId::DoorOpen) {
                self.world.set_block(above, BlockId::Air);
            }
            if matches!(self.world.get_block(below), BlockId::Door | BlockId::DoorOpen) {
                self.world.set_block(below, BlockId::Air);
            }
        }

        if matches!(
            broken_block,
            BlockId::RustStone
                | BlockId::RustStoneWireLit
                | BlockId::RustStoneBlock
                | BlockId::RustStoneLamp
                | BlockId::RustStoneLampLit
                | BlockId::Lever
                | BlockId::LeverOn
                | BlockId::Button
                | BlockId::ButtonPressed
                | BlockId::PressurePlate
                | BlockId::PressurePlatePressed
        ) {
            self.world.button_timers.remove(&target);
            self.world.update_ruststone_network(target, 16);
        }

        if broken_block.is_wood() || broken_block.is_leaves() {
            let unsupported_leaves = self.world.find_unsupported_leaves_around(target);
            for leaf_pos in unsupported_leaves {
                let leaf_block = self.world.get_block(leaf_pos);
                self.world.set_block(leaf_pos, BlockId::Air);

                let lx = leaf_pos.x as f32 + 0.5;
                let ly = leaf_pos.y as f32 + 0.5;
                let lz = leaf_pos.z as f32 + 0.5;
                let leaf_color = leaf_block.face_color(Face::Top);
                self.particles
                    .spawn_block_break(Vec3::new(lx - 0.5, ly - 0.5, lz - 0.5), leaf_color);

                if self.player.gamemode == GameMode::Survival
                    && matches!(
                        leaf_block,
                        BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves
                    )
                {
                    let l_seed = ((leaf_pos.x as u32).wrapping_mul(73856093)
                        ^ (leaf_pos.y as u32).wrapping_mul(19349663)
                        ^ (leaf_pos.z as u32).wrapping_mul(83492791))
                        % 1000;
                    let drop_id = if l_seed % 20 == 0 {
                        Some(BlockId::Apple)
                    } else if l_seed % 10 == 1 {
                        Some(BlockId::Sapling)
                    } else {
                        None
                    };
                    if let Some(id) = drop_id {
                        let l_rx = ((l_seed % 100) as f32 / 100.0 - 0.5) * 1.5;
                        let l_rz = (((l_seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.5;
                        self.items.push(ItemEntity::new(
                            Vec3::new(lx, ly, lz),
                            Vec3::new(l_rx, 2.0, l_rz),
                            id,
                            1,
                        ));
                    }
                }
            }
        }
    }

    fn spawn_survival_break_drops(
        &mut self,
        target: IVec3,
        broken_block: BlockId,
        held_tool: Option<BlockId>,
        chest_data: Option<crate::world::ChestData>,
        furnace_data: Option<crate::world::FurnaceData>,
    ) {
        let bx = target.x as f32 + 0.5;
        let by = target.y as f32 + 0.5;
        let bz = target.z as f32 + 0.5;

        self.player.exhaustion += 0.025;
        let selected = self.player.selected_hotbar_slot as usize;
        if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
            if stack.block.is_damageable() && stack.damage(1) {
                self.player.inventory.hotbar[selected] = None;
                self.audio
                    .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
            }
        }

        let seed = ((target.x as u32).wrapping_mul(73856093)
            ^ (target.y as u32).wrapping_mul(19349663)
            ^ (target.z as u32).wrapping_mul(83492791))
            % 1000;
        let rx = ((seed % 100) as f32 / 100.0 - 0.5) * 1.8;
        let rz = (((seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.8;
        let drop_vel = Vec3::new(rx, 3.2, rz);
        let can_harvest = broken_block.can_harvest_with_tool(held_tool);
        let custom_drop = if can_harvest {
            crate::engine::modding::get_custom_block_drop(broken_block)
        } else {
            None
        };

        if let Some((drop_id, count)) = custom_drop {
            self.items
                .push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, drop_id, count));
        } else if can_harvest && broken_block != BlockId::Glass {
            let current_held = self.player.inventory.hotbar
                [self.player.selected_hotbar_slot as usize]
                .as_ref()
                .map(|s| s.block);
            let drop_block = match broken_block {
                BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves => {
                    if seed % 20 == 0 {
                        Some(BlockId::Apple)
                    } else if seed % 10 == 1 {
                        Some(BlockId::Sapling)
                    } else {
                        None
                    }
                }
                BlockId::Grass => {
                    if seed % 8 == 0 {
                        Some(BlockId::WheatSeeds)
                    } else {
                        Some(BlockId::Dirt)
                    }
                }
                BlockId::WheatCrop => Some(BlockId::WheatSeeds),
                BlockId::CarrotCrop => Some(BlockId::Carrot),
                BlockId::PotatoCrop => Some(BlockId::Potato),
                BlockId::RustStoneOre => {
                    let count = 4 + (seed % 2) as u8;
                    self.items.push(ItemEntity::new(
                        Vec3::new(bx, by, bz),
                        drop_vel,
                        BlockId::RustStone,
                        count,
                    ));
                    None
                }
                BlockId::RustStoneWireLit => Some(BlockId::RustStone),
                BlockId::RustStoneLampLit => Some(BlockId::RustStoneLamp),
                BlockId::LeverOn => Some(BlockId::Lever),
                BlockId::ButtonPressed => Some(BlockId::Button),
                BlockId::PressurePlatePressed => Some(BlockId::PressurePlate),
                BlockId::Farmland | BlockId::FarmlandWet => Some(BlockId::Dirt),
                BlockId::Door | BlockId::DoorOpen => Some(BlockId::Door),
                BlockId::FurnaceLit => Some(BlockId::Furnace),
                BlockId::Seagrass => (current_held == Some(BlockId::Shears)).then_some(BlockId::Seagrass),
                BlockId::DiamondOre => Some(BlockId::Diamond),
                BlockId::EmeraldOre => Some(BlockId::Emerald),
                BlockId::Clay => {
                    self.items.push(ItemEntity::new(
                        Vec3::new(bx, by, bz),
                        drop_vel,
                        BlockId::ClayBall,
                        4,
                    ));
                    None
                }
                BlockId::Gravel => {
                    if seed % 10 == 0 {
                        Some(BlockId::Flint)
                    } else {
                        Some(BlockId::Gravel)
                    }
                }
                BlockId::Stone => Some(BlockId::Cobblestone),
                BlockId::Obsidian => {
                    (current_held == Some(BlockId::DiamondPickaxe)).then_some(BlockId::Obsidian)
                }
                other => Some(other),
            };
            if let Some(drop_id) = drop_block {
                self.items
                    .push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, drop_id, 1));
            }
        }

        if let Some(chest) = chest_data {
            for item in chest.items.into_iter().flatten() {
                self.items.push(ItemEntity::new(
                    Vec3::new(bx, by, bz),
                    drop_vel,
                    item.block,
                    item.count,
                ));
            }
        }
        if let Some(furnace) = furnace_data {
            for st in [furnace.input, furnace.fuel, furnace.output]
                .into_iter()
                .flatten()
            {
                self.items.push(ItemEntity::new(
                    Vec3::new(bx, by, bz),
                    drop_vel,
                    st.block,
                    st.count,
                ));
            }
        }
    }

    pub(super) fn handle_creative_pick_block(&mut self, input: &InputState) {
        let pick_block_just = !self.chat.is_open
            && (self.controls.is_just_pressed(Action::PickBlock)
                || (input.mouse_middle && !self.prev_middle_down));
        if !pick_block_just || self.player.gamemode != GameMode::Creative {
            return;
        }

        let forward = self.camera.forward();
        let picked_item = self
            .mob_manager
            .raycast_mob(self.camera.position, forward, 4.2)
            .and_then(|(mob_idx, _)| match self.mob_manager.mobs[mob_idx].kind {
                crate::gameplay::mob::MobKind::Pig => Some(BlockId::PigSpawnEgg),
                crate::gameplay::mob::MobKind::Cow => Some(BlockId::CowSpawnEgg),
                crate::gameplay::mob::MobKind::Chicken => Some(BlockId::ChickenSpawnEgg),
                crate::gameplay::mob::MobKind::Sheep => Some(BlockId::SheepSpawnEgg),
                _ => None,
            })
            .or_else(|| {
                self.world
                    .raycast(self.camera.position, forward, 8.0)
                    .and_then(|hit| self.world.get_block(hit.block_pos).pick_item())
            });

        let Some(item_block) = picked_item else {
            return;
        };

        let curr_slot = self.player.selected_hotbar_slot as usize;
        if let Some(existing_slot) = self
            .player
            .inventory
            .hotbar
            .iter()
            .position(|slot| slot.map(|s| s.block) == Some(item_block))
        {
            self.player.selected_hotbar_slot = existing_slot as u8;
            if let Some(stack) = &mut self.player.inventory.hotbar[existing_slot] {
                stack.count = 64;
            }
        } else if self.player.inventory.hotbar[curr_slot].is_none() {
            self.player.inventory.hotbar[curr_slot] = Some(ItemStack::new(item_block, 64));
        } else if let Some(free_slot) = self
            .player
            .inventory
            .hotbar
            .iter()
            .position(|slot| slot.is_none())
        {
            self.player.inventory.hotbar[free_slot] = Some(ItemStack::new(item_block, 64));
            self.player.selected_hotbar_slot = free_slot as u8;
        } else {
            self.player.inventory.hotbar[curr_slot] = Some(ItemStack::new(item_block, 64));
        }

        let new_selected_slot = self.player.selected_hotbar_slot as usize;
        if let Some(stack) = self.player.inventory.hotbar[new_selected_slot] {
            self.renderer.set_selected_block(stack.block.to_u8());
        }
        self.hand_animation = 0.5;
        self.audio
            .play_sound(crate::audio::SoundId::ItemPop, 0.45, 1.2);
        crate::log_info!(
            "GAMEPLAY",
            "Creative pick block: {:?} -> slot {}",
            item_block,
            self.player.selected_hotbar_slot
        );
    }
}
