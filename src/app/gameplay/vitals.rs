//! Biome tinting, sparse ambient audio probing, PvP separation, and survival vitals (fire, oxygen, hunger, fall, death).
//! Mutates player health/air/hunger/fire timers, biome tint filter, cached ambient audio context, and death transitions.
//! Called from `movement.rs` after non-flying collision resolution.

use crate::core::math::{IVec3, Vec3};
use crate::gameplay::player::GameMode;
use crate::world::block::BlockId;

use super::super::{AppOutput, GameState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn update_ambient_and_biome(&mut self, dt: f32, is_sprinting: bool, in_water: bool) {
        let horiz_speed = self.horiz_velocity.length();
        let block_below_pos = IVec3::new(
            self.camera.position.x.floor() as i32,
            (self.camera.position.y - self.current_eye_offset - 0.2).floor() as i32,
            self.camera.position.z.floor() as i32,
        );
        let block_below = self.world.get_block(block_below_pos);

        let p_block = self.camera.position.floor_to_ivec();
        let (current_biome, _) = crate::world::get_biome_at(p_block.x, p_block.z, self.world.seed);
        let unlocked = self.achievements.visit_biome(current_biome);
        self.process_achievement_unlocks(unlocked);
        let q_bio = self
            .quests
            .sync_biomes(self.achievements.visited_biome_count());
        self.process_quest_completions(q_bio);
        let target_tint = current_biome.color_filter();
        let factor = (1.0 - (-1.8 * dt).exp()).clamp(0.0, 1.0);
        for i in 0..3 {
            self.biome_filter_tint[i] += (target_tint[i] - self.biome_filter_tint[i]) * factor;
        }

        // Low-frequency sparse environmental probe for ambient soundscapes (~3 Hz)
        self.ambient_probe_timer -= dt;
        if self.ambient_probe_timer <= 0.0 {
            self.ambient_probe_timer = 0.35;
            let (biome, _) = crate::world::get_biome_at(p_block.x, p_block.z, self.world.seed);
            let is_desert = matches!(
                biome,
                crate::world::Biome::Desert | crate::world::Biome::Badlands
            );

            let mut solid_above = 0;
            for &dy in &[2, 4, 8, 14, 22] {
                let test_pos = IVec3::new(p_block.x, p_block.y + dy, p_block.z);
                if self.world.get_block(test_pos).is_solid() {
                    solid_above += 1;
                }
            }
            let is_underground = p_block.y < 55 && solid_above >= 2;
            let sky_exposed = solid_above == 0;

            let mut near_water = in_water;
            let mut water_pos = if in_water {
                Some(self.camera.position)
            } else {
                None
            };
            let mut near_leaves = false;
            let mut leaves_pos = None;
            let mut near_sand = block_below == BlockId::Sand;
            let mut sand_pos = if near_sand {
                Some(self.camera.position)
            } else {
                None
            };

            const PROBE_OFFSETS: [(i32, i32, i32); 16] = [
                (3, 0, 0), (-3, 0, 0), (0, 0, 3), (0, 0, -3),
                (5, 1, 2), (-5, 1, -2), (2, 2, 5), (-2, 2, -5),
                (0, 3, 0), (0, -2, 0), (4, -1, 4), (-4, -1, -4),
                (6, 0, -3), (-6, 0, 3), (2, 4, 2), (-2, 4, -2),
            ];

            for &(ox, oy, oz) in &PROBE_OFFSETS {
                let probe = IVec3::new(p_block.x + ox, p_block.y + oy, p_block.z + oz);
                let b = self.world.get_block(probe);
                if matches!(b, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) && !near_water {
                    near_water = true;
                    water_pos = Some(Vec3::new(
                        probe.x as f32 + 0.5,
                        probe.y as f32 + 0.5,
                        probe.z as f32 + 0.5,
                    ));
                } else if b.is_leaf() && !near_leaves {
                    near_leaves = true;
                    leaves_pos = Some(Vec3::new(
                        probe.x as f32 + 0.5,
                        probe.y as f32 + 0.5,
                        probe.z as f32 + 0.5,
                    ));
                } else if b == BlockId::Sand && !near_sand {
                    near_sand = true;
                    sand_pos = Some(Vec3::new(
                        probe.x as f32 + 0.5,
                        probe.y as f32 + 0.5,
                        probe.z as f32 + 0.5,
                    ));
                }
            }

            let (reverb_wet, reverb_decay_seconds) =
                self.world.calculate_acoustic_reverb(self.camera.position);
            self.cached_ambient_ctx = crate::audio::AmbientContext {
                is_underground,
                sky_exposed,
                near_water,
                water_pos,
                near_leaves,
                leaves_pos,
                is_desert,
                sand_pos,
                rain_intensity: self.world.weather.rain_strength,
                reverb_wet,
                reverb_decay_seconds,
            };
        }

        self.audio.update(
            dt,
            self.camera.position,
            horiz_speed,
            self.on_ground,
            is_sprinting,
            block_below,
            self.cached_ambient_ctx,
        );
        self.audio.update_listener(
            self.camera.position,
            self.camera.forward(),
            self.camera.right(),
            self.camera.up(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_player_vitals(
        &mut self,
        dt: f32,
        wish_dir: Vec3,
        is_sprinting: bool,
        feet_pos: Vec3,
        eye_block: BlockId,
        in_water: bool,
        in_lava: bool,
        out: &mut AppOutput,
    ) {
        // Multiplayer player-to-player collision pushing
        if let Some(client) = &self.net_client {
            for rp in client.remote_players.values() {
                let dy = (self.camera.position.y - rp.pos.y).abs();
                if dy < self.current_player_height {
                    let dx = self.camera.position.x - rp.pos.x;
                    let dz = self.camera.position.z - rp.pos.z;
                    let dist_sq = dx * dx + dz * dz;
                    let min_dist = crate::gameplay::physics::PLAYER_HALF_WIDTH * 2.0;
                    if dist_sq < min_dist * min_dist && dist_sq > 0.00001 {
                        let dist = dist_sq.sqrt();
                        let push = (min_dist - dist) * 0.5;
                        self.camera.position.x += (dx / dist) * push;
                        self.camera.position.z += (dz / dist) * push;
                    }
                }
            }
        }

        // Lava contact & fire burning mechanics (~5s burn after contact)
        if in_lava {
            self.player.fire_timer = 5.0;
            if self.player.gamemode == GameMode::Survival {
                self.lava_damage_timer += dt;
                if self.lava_damage_timer >= 0.5 {
                    self.lava_damage_timer = 0.0;
                    if self.player.take_damage(4.0) {
                        self.audio
                            .play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                        self.damage_tilt_timer = 0.4;
                        self.damage_tilt_dir = 1.0;
                        self.particles.spawn_lava_burn(feet_pos);
                        self.request_rumble(0.85, 0.6, 220);
                        crate::log_info!(
                            "PLAYER",
                            "Player took 4.0 lava direct damage (health: {:.1}/{:.1})",
                            self.player.health,
                            self.player.max_health
                        );
                    }
                }
            }
        } else {
            self.lava_damage_timer = 0.45;
            if in_water && self.player.fire_timer > 0.0 {
                self.player.fire_timer = 0.0;
                self.player.fire_damage_timer = 0.0;
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.6, 0.5);
                crate::log_info!("PLAYER", "Player fire extinguished by water");
            } else if self.player.fire_timer > 0.0 {
                self.player.fire_timer = (self.player.fire_timer - dt).max(0.0);
                if self.player.gamemode == GameMode::Survival {
                    self.player.fire_damage_timer += dt;
                    if self.player.fire_damage_timer >= 1.0 {
                        self.player.fire_damage_timer = 0.0;
                        if self.player.take_damage(1.0) {
                            self.audio
                                .play_sound(crate::audio::SoundId::PlayerHurt, 0.8, 1.0);
                            self.damage_tilt_timer = 0.3;
                            self.damage_tilt_dir =
                                if (self.player.fire_timer as i32 % 2) == 0 {
                                    1.0
                                } else {
                                    -1.0
                                };
                            self.particles.spawn_lava_burn(feet_pos);
                            self.request_rumble(0.5, 0.3, 150);
                            crate::log_info!(
                                "PLAYER",
                                "Player took 1.0 fire burn damage (remaining burn: {:.1}s, health: {:.1}/{:.1})",
                                self.player.fire_timer,
                                self.player.health,
                                self.player.max_health
                            );
                        }
                    }
                }
            } else {
                self.player.fire_damage_timer = 0.0;
            }
        }

        // Oxygen and underwater breathing mechanics
        let is_head_underwater =
            matches!(eye_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass);
        self.player.head_underwater = is_head_underwater;

        if self.player.gamemode == GameMode::Survival {
            if is_head_underwater {
                self.player.air -= (self.player.max_air / 15.0) * dt;
                if self.player.air <= 0.0 {
                    self.player.air = 0.0;
                    self.drown_damage_timer += dt;
                    if self.drown_damage_timer >= 1.0 {
                        self.drown_damage_timer = 0.0;
                        if self.player.take_damage(2.0) {
                            self.audio
                                .play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                            self.damage_tilt_timer = 0.4;
                            self.damage_tilt_dir = 1.0;
                            self.particles.spawn_drown_bubbles(self.camera.position);
                            self.request_rumble(0.7, 0.5, 200);
                            crate::log_info!(
                                "PLAYER",
                                "Player took 2.0 drowning damage (health: {:.1}/{:.1})",
                                self.player.health,
                                self.player.max_health
                            );
                        }
                    }
                } else {
                    self.drown_damage_timer = 0.0;
                }
            } else {
                self.drown_damage_timer = 0.0;
                self.player.air = (self.player.air + 20.0 * dt).min(self.player.max_air);
            }

            if is_sprinting {
                self.player.exhaustion += 0.18 * dt;
            } else if wish_dir.length_sq() > 0.01 {
                self.player.exhaustion += 0.04 * dt;
            } else {
                self.player.exhaustion += 0.01 * dt;
            }

            while self.player.exhaustion >= 4.0 {
                self.player.exhaustion -= 4.0;
                if self.player.saturation > 0.0 {
                    self.player.saturation = (self.player.saturation - 1.0).max(0.0);
                } else {
                    self.player.hunger = (self.player.hunger - 1.0).max(0.0);
                }
            }

            if self.player.hunger >= 18.0 && self.player.health < self.player.max_health {
                self.player.heal_timer += dt;
                if self.player.heal_timer >= 4.0 {
                    self.player.heal_timer = 0.0;
                    self.player.heal(1.0);
                    self.player.exhaustion += 4.0;
                }
            } else {
                self.player.heal_timer = 0.0;
            }

            if self.player.hunger <= 0.0 {
                self.player.starvation_timer += dt;
                if self.player.starvation_timer >= 4.0 {
                    self.player.starvation_timer = 0.0;
                    if self.player.take_damage(1.0) {
                        self.audio
                            .play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                        self.damage_tilt_timer = 0.4;
                        self.damage_tilt_dir = 1.0;
                        self.request_rumble(0.6, 0.4, 180);
                        crate::log_info!(
                            "PLAYER",
                            "Player took 1.0 starvation damage (health: {:.1}/{:.1})",
                            self.player.health,
                            self.player.max_health
                        );
                    }
                }
            } else {
                self.player.starvation_timer = 0.0;
            }
        } else {
            self.player.air = self.player.max_air;
            self.drown_damage_timer = 0.0;
            self.player.hunger = self.player.max_hunger;
            self.player.saturation = 5.0;
            self.player.exhaustion = 0.0;
            self.player.starvation_timer = 0.0;
            self.player.heal_timer = 0.0;
        }

        if self.on_ground {
            if !self.player.has_landed {
                self.player.has_landed = true;
                self.player.fall_distance = 0.0;
            } else if self.player.fall_distance > 3.0 {
                let damage = self.player.fall_distance - 3.0;
                if self.player.take_damage(damage) {
                    self.audio
                        .play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                    self.damage_tilt_timer = 0.4;
                    self.damage_tilt_dir = 1.0;
                    self.request_rumble(0.85, 0.7, 250);
                    crate::log_info!(
                        "PLAYER",
                        "Player took {:.1} fall damage (health: {:.1}/{:.1})",
                        damage,
                        self.player.health,
                        self.player.max_health
                    );
                }
            }
            self.player.fall_distance = 0.0;
        }

        if self.player.is_dead() {
            self.announce_player_death();
            self.state = GameState::DeathScreen;
            out.request_release_mouse = true;
            self.death_hovered_btn = Some(0);
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            self.request_rumble(1.0, 0.9, 500);
            crate::log_info!("PLAYER", "Player died! Transitioning to death screen");
        }
    }
}
