//! Dropped-item physics/pickup/lava-burn, arrow ballistics/collisions, and fishing bobber simulation.
//! Mutates `items`, `arrows`, `fishing_bobber`, player inventory/health, and mob damage.
//! Called by `simulation.rs` inside the entity simulation phase.

use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::gameplay::item::ItemEntity;
use crate::world::block::BlockId;

use super::super::{RumbleRequest, RustCraftApp};

impl RustCraftApp {
    pub(super) fn tick_items_arrows_and_bobber(&mut self, dt: f32, is_creative: bool) {
        if !self.chat.is_open && self.controls.is_just_pressed(Action::DropItem) {
            let slot = self.player.selected_hotbar_slot as usize;
            if let Some(stack) = &mut self.player.inventory.hotbar[slot] {
                crate::log_info!("INVENTORY", "Player dropped 1x {:?}", stack.block);
                self.items.push(ItemEntity::new(
                    self.camera.position + self.camera.forward() * 0.5,
                    self.camera.forward() * 5.0,
                    stack.block,
                    1,
                ));
                stack.count -= 1;
                if stack.count == 0 {
                    self.player.inventory.hotbar[slot] = None;
                }
            }
        }

        let player_center = self.camera.position - Vec3::new(0.0, 0.8, 0.0);
        let mut i = 0;
        while i < self.items.len() {
            self.items[i].update(dt, &self.world, Some(player_center));

            let item_pos = self.items[i].position;
            if self.world.get_block(item_pos.floor_to_ivec()) == BlockId::Lava {
                self.particles.spawn_lava_burn(item_pos);
                self.items.remove(i);
                continue;
            }

            let dist_sq = (self.items[i].position - player_center).length_sq();
            if self.items[i].pickup_delay <= 0.0 && dist_sq < 3.24 {
                let remaining = self
                    .player
                    .inventory
                    .add_item(self.items[i].block, self.items[i].count);
                let picked_up = self.items[i].count - remaining;
                if picked_up > 0 {
                    self.audio
                        .play_sound(crate::audio::SoundId::ItemPop, 0.7, 1.0);
                    crate::log_info!(
                        "INVENTORY",
                        "Player picked up {}x {:?}",
                        picked_up,
                        self.items[i].block
                    );
                    self.record_collected_item(self.items[i].block);
                }
                if remaining == 0 {
                    self.items.remove(i);
                    continue;
                } else {
                    self.items[i].count = remaining;
                }
            }
            i += 1;
        }

        // Drain skeleton-fired arrows from mob manager
        self.arrows.extend(self.mob_manager.pending_arrows.drain(..));

        let mut a_idx = 0;
        while a_idx < self.arrows.len() {
            let struck_block = self.arrows[a_idx].update(dt, &self.world);
            if struck_block {
                self.audio
                    .play_sound(crate::audio::SoundId::BowHit, 0.85, 1.0);
            }

            let arrow = &self.arrows[a_idx];
            if arrow.lifetime > 60.0 {
                self.arrows.remove(a_idx);
                continue;
            }

            let mut remove_arrow = false;
            if !arrow.in_ground {
                if arrow.shooter_is_player {
                    let mut hit_mob = None;
                    for (midx, mob) in self.mob_manager.mobs.iter().enumerate() {
                        if mob.is_dead {
                            continue;
                        }
                        let mob_center = mob.pos + Vec3::new(0.0, mob.kind.height() * 0.5, 0.0);
                        if (arrow.position - mob_center).length() < 0.6 + mob.kind.half_width() {
                            hit_mob = Some(midx);
                            break;
                        }
                    }
                    if let Some(midx) = hit_mob {
                        let (kind, mob_pos) = {
                            let mob = &mut self.mob_manager.mobs[midx];
                            let kb = arrow.velocity.normalized() * 5.0;
                            mob.take_damage(arrow.damage, Vec3::new(kb.x, 2.5, kb.z));
                            (mob.kind, mob.pos)
                        };
                        if kind == crate::gameplay::mob::MobKind::Villager {
                            self.mob_manager.anger_golems_at_villager_attack(mob_pos);
                        }
                        self.audio
                            .play_sound(crate::audio::SoundId::BowHit, 1.0, 1.0);
                        self.particles.spawn_mob_hit(arrow.position);
                        remove_arrow = true;
                    }
                } else if !is_creative {
                    let player_bb_min = self.camera.position - Vec3::new(0.4, 1.6, 0.4);
                    let player_bb_max = self.camera.position + Vec3::new(0.4, 0.2, 0.4);
                    if arrow.position.x >= player_bb_min.x
                        && arrow.position.x <= player_bb_max.x
                        && arrow.position.y >= player_bb_min.y
                        && arrow.position.y <= player_bb_max.y
                        && arrow.position.z >= player_bb_min.z
                        && arrow.position.z <= player_bb_max.z
                    {
                        let to_arrow = (arrow.position - self.camera.position).normalized();
                        let is_front = to_arrow.dot(self.camera.forward()) > 0.35;
                        let (took_damage, blocked, item_broke) =
                            self.player.take_physical_damage(arrow.damage, is_front);
                        if item_broke {
                            self.audio
                                .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                        }
                        if blocked {
                            self.audio
                                .play_sound(crate::audio::SoundId::ShieldBlock, 1.0, 1.0);
                            self.damage_tilt_timer = 0.12;
                            crate::log_info!("COMBAT", "Player blocked skeleton arrow with shield!");
                        } else if took_damage {
                            self.damage_tilt_timer = 0.4;
                            self.damage_tilt_dir = if arrow.position.x > self.camera.position.x {
                                -1.0
                            } else {
                                1.0
                            };
                            if self.settings.vibration {
                                self.pending_rumble = Some(RumbleRequest {
                                    strong: 0.8,
                                    weak: 0.5,
                                    duration_ms: 200,
                                });
                            }
                            self.audio
                                .play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                            crate::log_info!(
                                "PLAYER",
                                "Player struck by arrow for {:.1} damage (health: {:.1}/{:.1})",
                                arrow.damage,
                                self.player.health,
                                self.player.max_health
                            );
                        }
                        self.audio
                            .play_sound(crate::audio::SoundId::BowHit, 1.0, 1.0);
                        remove_arrow = true;
                    }
                }
            } else if !is_creative {
                let dist_sq = (arrow.position - player_center).length_sq();
                if dist_sq < 2.5 && self.player.inventory.add_item(BlockId::Arrow, 1) == 0 {
                    self.audio
                        .play_sound(crate::audio::SoundId::ItemPop, 0.7, 1.0);
                    remove_arrow = true;
                }
            }

            if remove_arrow {
                self.arrows.remove(a_idx);
            } else {
                a_idx += 1;
            }
        }

        // Update fishing bobber physics and bite timer
        if let Some(bobber) = &mut self.fishing_bobber {
            let held_rod = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                .map(|s| s.block)
                == Some(BlockId::FishingRod);
            let dist_sq = (bobber.pos - self.camera.position).length_sq();
            if !held_rod || dist_sq > 1024.0 {
                self.fishing_bobber = None;
            } else {
                let block_at_bobber = self.world.get_block(bobber.pos.floor_to_ivec());
                if matches!(
                    block_at_bobber,
                    BlockId::Water | BlockId::Kelp | BlockId::Seagrass
                ) {
                    bobber.in_water = true;
                    let surface_y = bobber.pos.y.floor() + 0.85;
                    bobber.vel.y += (surface_y - bobber.pos.y) * 12.0 * dt;
                    bobber.vel.y *= 0.82;
                    bobber.vel.x *= 0.88;
                    bobber.vel.z *= 0.88;
                    bobber.pos += bobber.vel * dt;

                    bobber.bite_timer -= dt;
                    if bobber.bite_timer <= 0.0 {
                        bobber.bite_window -= dt;
                        bobber.splash_timer += dt;
                        if bobber.splash_timer > 0.35 {
                            bobber.splash_timer = 0.0;
                            self.audio
                                .play_sound(crate::audio::SoundId::FishFlop, 0.65, 1.2);
                            self.particles
                                .spawn_block_break(bobber.pos, [0.3, 0.6, 1.0]);
                        }
                        bobber.pos.y = surface_y - 0.18;
                        if bobber.bite_window <= 0.0 {
                            let seed = (self.total_time * 1000.0) as u32;
                            bobber.bite_timer = 5.0 + (seed % 6) as f32;
                            bobber.bite_window = 2.5;
                        }
                    }
                } else {
                    bobber.in_water = false;
                    bobber.vel.y -= 18.0 * dt;
                    bobber.pos += bobber.vel * dt;
                    if self.world.get_block(bobber.pos.floor_to_ivec()).is_solid() {
                        bobber.vel = Vec3::zero();
                    }
                }
            }
        }
    }
}
