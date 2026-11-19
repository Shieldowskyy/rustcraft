//! Right-click combat/tool/mob/food interactions: bow, shield, armor equip, fishing rod, villagers, animals, eating.
//! Mutates player inventory, bow draw/blocking state, fishing bobber, arrows, mobs, and `GameState`.
//! Delegates farming and world-block right-click/placement to `world_interact.rs`.

use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::player::GameMode;
use crate::world::block::{BlockId, Face};

use super::super::{AppOutput, GameState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn consume_selected_hotbar_one(&mut self) {
        if self.player.gamemode == GameMode::Survival {
            let slot = self.player.selected_hotbar_slot as usize;
            if let Some(stack) = &mut self.player.inventory.hotbar[slot] {
                stack.count -= 1;
                if stack.count == 0 {
                    self.player.inventory.hotbar[slot] = None;
                }
            }
        }
    }

    pub(super) fn handle_right_click_interactions(
        &mut self,
        dt: f32,
        is_crouching: bool,
        selected_block_id: u8,
        out: &mut AppOutput,
    ) {
        let held_item = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
            .map(|s| s.block);
        let offhand_item = self.player.inventory.offhand.map(|s| s.block);
        let has_shield = held_item.map_or(false, |b| b.is_shield())
            || offhand_item.map_or(false, |b| b.is_shield());
        let has_bow = held_item == Some(BlockId::Bow);
        let use_active = !self.chat.is_open && self.controls.is_active(Action::UsePlace);

        // Bow drawing & release
        if has_bow {
            if use_active {
                self.player.bow_draw_time += dt;
                self.horiz_velocity.x *= 0.70;
                self.horiz_velocity.z *= 0.70;
            } else if self.player.bow_draw_time > 0.0 {
                if self.player.bow_draw_time >= 0.25 {
                    let is_creative = self.player.gamemode == GameMode::Creative;
                    if is_creative || self.player.inventory.consume_item(BlockId::Arrow) {
                        let charge = (self.player.bow_draw_time / 1.0).clamp(0.1, 1.0);
                        let speed = 14.0 + charge * 22.0;
                        let damage = 2.0 + charge * 7.0 + if charge >= 0.95 { 2.5 } else { 0.0 };
                        let forward = self.camera.forward();
                        let spawn_pos =
                            self.camera.position + forward * 0.4 + Vec3::new(0.0, -0.08, 0.0);
                        self.arrows.push(crate::gameplay::ArrowEntity::new(
                            spawn_pos,
                            forward * speed,
                            true,
                            damage,
                        ));
                        self.audio.play_sound(
                            crate::audio::SoundId::BowShoot,
                            0.9,
                            0.9 + charge * 0.25,
                        );
                        self.hand_animation = 0.8;
                        crate::log_info!(
                            "COMBAT",
                            "Player fired arrow with charge {:.2} (speed: {:.1}, dmg: {:.1})",
                            charge,
                            speed,
                            damage
                        );
                        if self.player.gamemode == GameMode::Survival {
                            let selected = self.player.selected_hotbar_slot as usize;
                            if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                                if stack.block == BlockId::Bow && stack.damage(1) {
                                    self.player.inventory.hotbar[selected] = None;
                                    self.audio
                                        .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                }
                            }
                        }
                    }
                }
                self.player.bow_draw_time = 0.0;
            }
        } else {
            self.player.bow_draw_time = 0.0;
        }

        // Shield blocking
        if has_shield && !has_bow && use_active {
            self.player.is_blocking = true;
            self.horiz_velocity.x *= 0.55;
            self.horiz_velocity.z *= 0.55;
        } else {
            self.player.is_blocking = false;
        }

        let interact_right = !self.chat.is_open
            && !has_bow
            && !self.player.is_blocking
            && self.controls.is_just_pressed(Action::UsePlace);
        if !interact_right {
            return;
        }

        let mut handled = false;

        // Quick auto-equip armor / shield from hotbar on right-click
        if let Some(item) = held_item {
            if item.is_armor() || item.is_shield() {
                let curr_slot = self.player.selected_hotbar_slot as usize;
                if self.player.inventory.try_auto_equip_hotbar(curr_slot) {
                    self.audio
                        .play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                    self.hand_animation = 0.5;
                    handled = true;
                }
            }
        }

        // 0. Fishing Rod casting / reeling
        if !handled && held_item == Some(BlockId::FishingRod) {
            self.handle_fishing_rod_use();
            handled = true;
        }

        // 1. Villager trading & animal interaction (shearing, dyeing, feeding/breeding)
        if !handled {
            handled = self.try_interact_mob(held_item, out);
        }

        // 2. Farming tool and block interactions (Hoe, BoneMeal, Planting)
        if !handled {
            handled = self.try_farming_interaction(held_item);
        }

        // 3. Food consumption
        if !handled {
            handled = self.try_eat_held_food(held_item);
        }

        // 4. World block interaction & placement
        if !handled {
            self.try_world_block_use_or_place(is_crouching, selected_block_id, out);
        }
    }

    fn handle_fishing_rod_use(&mut self) {
        if let Some(bobber) = self.fishing_bobber.take() {
            self.audio
                .play_sound(crate::audio::SoundId::FishingReel, 0.9, 1.0);
            self.hand_animation = 1.0;
            if self.player.gamemode == GameMode::Survival {
                let selected = self.player.selected_hotbar_slot as usize;
                if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                    if stack.block == BlockId::FishingRod && stack.damage(1) {
                        self.player.inventory.hotbar[selected] = None;
                        self.audio
                            .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                    }
                }
            }
            if bobber.in_water && bobber.bite_timer <= 0.0 && bobber.bite_window > 0.0 {
                let seed = (self.total_time * 1000.0) as u32;
                let roll = seed % 100;
                let caught_item = if roll < 55 {
                    BlockId::RawCod
                } else if roll < 80 {
                    BlockId::RawSalmon
                } else if roll < 92 {
                    BlockId::TropicalFish
                } else if roll < 97 {
                    BlockId::Pufferfish
                } else if roll % 2 == 0 {
                    BlockId::GoldOunce
                } else {
                    BlockId::Bone
                };

                let pull_dir = (self.camera.position - bobber.pos).normalized();
                let pull_vel = pull_dir * 9.0 + Vec3::new(0.0, 4.5, 0.0);
                self.items.push(ItemEntity::new(
                    bobber.pos + Vec3::new(0.0, 0.2, 0.0),
                    pull_vel,
                    caught_item,
                    1,
                ));
                self.audio
                    .play_sound(crate::audio::SoundId::FishCatch, 1.0, 1.0);
                self.particles
                    .spawn_block_break(bobber.pos, [0.2, 0.5, 0.9]);
                crate::log_info!("GAMEPLAY", "Caught fish {:?} via fishing rod!", caught_item);
            }
        } else {
            let eye_pos = self.camera.position;
            let forward = self.camera.forward();
            let spawn_pos = eye_pos + forward * 0.6;
            let cast_vel = forward * 13.5 + Vec3::new(0.0, 3.2, 0.0);
            let seed = (self.total_time * 1000.0) as u32;
            let wait_time = 4.0 + (seed % 6) as f32;
            self.fishing_bobber = Some(crate::app::FishingBobber {
                pos: spawn_pos,
                vel: cast_vel,
                in_water: false,
                bite_timer: wait_time,
                bite_window: 2.5,
                splash_timer: 0.0,
            });
            self.audio
                .play_sound(crate::audio::SoundId::FishingCast, 0.9, 1.0);
            self.hand_animation = 1.0;
        }
    }

    fn try_interact_mob(&mut self, held_item: Option<BlockId>, out: &mut AppOutput) -> bool {
        let Some((mob_idx, _)) =
            self.mob_manager
                .raycast_mob(self.camera.position, self.camera.forward(), 4.2)
        else {
            return false;
        };

        let mob = &mut self.mob_manager.mobs[mob_idx];
        if mob.kind == crate::gameplay::mob::MobKind::Villager && !mob.is_dead {
            self.state = GameState::VillagerTrade {
                mob_id: mob.id,
                selected_trade: 0,
            };
            out.request_release_mouse = true;
            out.cursor_visible = Some(true);
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            self.audio
                .play_sound(crate::audio::SoundId::VillagerIdle, 1.0, 1.0);
            return true;
        }

        let Some(item) = held_item else {
            return false;
        };

        if item == BlockId::Shears
            && mob.kind == crate::gameplay::mob::MobKind::Sheep
            && !mob.is_sheared
            && !mob.is_baby
        {
            mob.is_sheared = true;
            let wool_block = BlockId::wool_block_from_color(mob.wool_color);
            let wool_count = 1 + (mob.id % 3) as u8;
            let center = mob.center_pos();
            let rx = ((mob.id as f32 * 11.0).sin()) * 1.2;
            let rz = ((mob.id as f32 * 19.0).cos()) * 1.2;
            self.items.push(ItemEntity::new(
                center,
                Vec3::new(rx, 3.2, rz),
                wool_block,
                wool_count,
            ));
            self.audio
                .play_sound(crate::audio::SoundId::SheepShear, 0.9, 1.0);
            self.particles.spawn_growth(center);
            self.hand_animation = 1.0;
            if self.player.gamemode == GameMode::Survival {
                let selected = self.player.selected_hotbar_slot as usize;
                if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                    if stack.block == BlockId::Shears && stack.damage(1) {
                        self.player.inventory.hotbar[selected] = None;
                        self.audio
                            .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                    }
                }
            }
            return true;
        }

        if item.is_wool() && mob.kind == crate::gameplay::mob::MobKind::Sheep && !mob.is_baby {
            let dye_color = item.wool_color_index().unwrap();
            if mob.wool_color != dye_color || mob.is_sheared {
                mob.wool_color = dye_color;
                mob.is_sheared = false;
                let sound_pos = mob.center_pos();
                self.audio
                    .play_sound(crate::audio::SoundId::DigGrass, 0.75, 1.2);
                self.particles.spawn_growth(sound_pos);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            }
        } else if mob.kind.is_tempting_item(item) {
            let center = mob.center_pos();
            if mob.is_baby {
                mob.age += 30.0;
                self.particles.spawn_growth(center);
                self.audio
                    .play_sound(crate::audio::SoundId::PlayerEat, 0.7, 1.3);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            } else if mob.breed_cooldown <= 0.0 && mob.in_love <= 0.0 {
                mob.in_love = 30.0;
                self.particles.spawn_hearts(center);
                self.audio
                    .play_sound(crate::audio::SoundId::PlayerEat, 0.8, 1.2);
                self.consume_selected_hotbar_one();
                self.hand_animation = 1.0;
                return true;
            }
        }

        false
    }

    fn try_eat_held_food(&mut self, held_item: Option<BlockId>) -> bool {
        let Some(food_id) = held_item else {
            return false;
        };
        if !self.player.can_eat(food_id) || !self.player.eat(food_id) {
            return false;
        }

        self.audio
            .play_sound(crate::audio::SoundId::PlayerEat, 0.8, 1.0);
        self.hand_animation = 1.0;
        let color = food_id.face_color(Face::Top);
        let mouth_pos =
            self.camera.position + self.camera.forward() * 0.4 - Vec3::new(0.0, 0.2, 0.0);
        self.particles
            .spawn_block_break(mouth_pos - Vec3::new(0.5, 0.5, 0.5), color);
        if food_id == BlockId::Pufferfish {
            self.player.take_damage(3.0);
            self.audio
                .play_sound(crate::audio::SoundId::PlayerHurt, 0.8, 1.0);
            self.damage_tilt_timer = 0.25;
            crate::log_info!("PLAYER", "Player pricked/poisoned by eating Pufferfish!");
        }
        crate::log_info!(
            "PLAYER",
            "Player consumed {:?} (health: {:.1}/{:.1}, hunger: {:.1}/{:.1})",
            food_id,
            self.player.health,
            self.player.max_health,
            self.player.hunger,
            self.player.max_hunger
        );
        self.consume_selected_hotbar_one();
        true
    }
}
