//! Per-frame world, material, ruststone, mob, explosion, furnace, particle, and weather simulation tick.
//! Mutates world chunks/weather/explosions, mobs, player hurt/death state, and `FrameProfiler` tick timings.
//! Delegates dropped items, arrows, and fishing bobber physics to `projectiles.rs`.

use crate::core::math::Vec3;
use crate::core::profiler::FrameProfiler;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::player::GameMode;

use super::super::{AppOutput, GameState, RumbleRequest, RustCraftApp};

impl RustCraftApp {
    pub(super) fn tick_world_and_entities(
        &mut self,
        dt: f32,
        collect_profiling: bool,
        cur_profiler: &mut FrameProfiler,
        out: &mut AppOutput,
    ) -> f32 {
        let world_time_start = collect_profiling.then(std::time::Instant::now);
        self.world.advance_time(dt);
        let world_update_start = collect_profiling.then(std::time::Instant::now);

        let local_mob_player = crate::gameplay::mob::MobPlayerContext {
            position: self.camera.position,
            held_item: self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                .map(|stack| stack.block),
            is_creative: self.player.gamemode == GameMode::Creative,
        };
        let mut mob_players = vec![local_mob_player];
        let mut remote_mob_centers = Vec::new();
        if let Some(client) = self.net_client.as_ref() {
            if client.is_connected && client.local_id == client.mob_authority_id {
                let mut remote_positions: Vec<_> = client
                    .remote_players
                    .values()
                    .map(|remote| remote.target_pos)
                    .collect();
                remote_positions.sort_by(|a, b| {
                    a.x.total_cmp(&b.x)
                        .then_with(|| a.z.total_cmp(&b.z))
                        .then_with(|| a.y.total_cmp(&b.y))
                });
                for position in remote_positions {
                    mob_players.push(crate::gameplay::mob::MobPlayerContext {
                        position,
                        held_item: None,
                        is_creative: false,
                    });
                    remote_mob_centers.push(position);
                }
            }
        }
        self.world.update_with_simulation_centers(
            self.camera.position,
            self.settings.render_distance,
            &remote_mob_centers,
        );
        let world_io_dur = world_update_start.map(|t| t.elapsed()).unwrap_or_default();

        let hand_speed = if self.fly_mode {
            self.fly_velocity.length()
        } else {
            self.horiz_velocity.length()
        };
        if hand_speed > 0.05 {
            self.walk_phase += dt * (5.0 + hand_speed * 0.9);
        }
        self.material_timer += dt;
        if self.material_timer >= 0.25 {
            self.material_timer = 0.0;
            let fluid_authority = self
                .net_client
                .as_ref()
                .map_or(true, |c| !c.is_connected || c.local_id == c.mob_authority_id);
            self.world
                .simulate_materials(self.camera.position, fluid_authority);
            self.world
                .tick_farming(self.camera.position, &mut self.mob_manager.rng_state);
        }

        self.world.update_primed_tnts(dt);
        let mob_positions: Vec<Vec3> = self
            .mob_manager
            .mobs
            .iter()
            .filter(|m| !m.is_dead)
            .map(|m| m.pos)
            .collect();
        self.world
            .update_ruststone(dt, self.camera.position, &mob_positions);
        let sound_events = std::mem::take(&mut self.world.ruststone_sound_events);
        for (s_pos, _is_on) in sound_events {
            let sound_pos = Vec3::new(
                s_pos.x as f32 + 0.5,
                s_pos.y as f32 + 0.5,
                s_pos.z as f32 + 0.5,
            );
            self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.7);
        }
        if let Some(w_start) = world_time_start {
            cur_profiler.tick_world_ms =
                w_start.elapsed().saturating_sub(world_io_dur).as_secs_f32() * 1000.0;
        }

        let mobs_start = collect_profiling.then(std::time::Instant::now);
        let is_creative = self.player.gamemode == GameMode::Creative;
        let simulates_mobs = self.net_client.as_ref().map_or(true, |client| {
            !client.is_connected || client.local_id == client.mob_authority_id
        });
        if simulates_mobs {
            self.mob_manager
                .update_for_players(dt, &self.world, &mob_players);
        }
        let mut discovered_mob = false;
        for mob in &self.mob_manager.mobs {
            if !mob.is_dead && (mob.pos - self.camera.position).length_sq() <= 24.0 * 24.0 {
                discovered_mob |= self.achievements.record_mob_seen(mob.kind);
            }
        }
        if discovered_mob {
            self.persist_achievement_progress();
        }
        for (b_pos, b_id) in self.mob_manager.pending_block_changes.drain(..) {
            self.world.set_block(b_pos, b_id);
        }
        for pos in self.mob_manager.breeding_hearts.drain(..) {
            self.particles.spawn_hearts(pos);
        }
        for pos in self.mob_manager.love_hearts.drain(..) {
            self.particles.spawn_hearts(pos);
        }
        self.process_mob_sounds();
        self.resolve_mob_player_collisions_and_melee(is_creative);

        for exp_pos in self.mob_manager.pending_explosions.drain(..) {
            self.world.explode(exp_pos.floor_to_ivec(), 3.5);
        }
        for poof_pos in self.mob_manager.dying_poofs.drain(..) {
            self.particles.spawn_mob_death_poof(poof_pos);
        }
        for drop in self.mob_manager.dying_drops.drain(..) {
            crate::log_info!(
                "COMBAT",
                "Mob killed, dropped {}x {:?}",
                drop.count,
                drop.block
            );
            let rx = ((self.mob_manager.mobs.len() as f32 * 17.0).sin()) * 1.5;
            let rz = ((self.mob_manager.mobs.len() as f32 * 23.0).cos()) * 1.5;
            self.items.push(ItemEntity::new(
                drop.pos,
                Vec3::new(rx, 3.0, rz),
                drop.block,
                drop.count,
            ));
        }
        if let Some(m_start) = mobs_start {
            cur_profiler.tick_mobs_ms = m_start.elapsed().as_secs_f32() * 1000.0;
        }

        let items_start = collect_profiling.then(std::time::Instant::now);

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

        self.tick_items_arrows_and_bobber(dt, is_creative);
        self.process_world_explosions_and_weather(dt);

        if let Some(i_start) = items_start {
            cur_profiler.tick_entities_ms = i_start.elapsed().as_secs_f32() * 1000.0;
        }
        if collect_profiling {
            cur_profiler.world_io_ms = world_io_dur.as_secs_f32() * 1000.0;
            if crate::platform::capabilities().uses_hardware_gxm {
                cur_profiler.world_gen_ms = cur_profiler.world_io_ms;
                cur_profiler.world_mesh_ms = 0.0;
            } else {
                cur_profiler.world_mesh_ms = cur_profiler.world_io_ms * 0.72;
                cur_profiler.world_gen_ms = cur_profiler.world_io_ms * 0.28;
            }
            cur_profiler.tick_ms = cur_profiler.tick_physics_ms
                + cur_profiler.tick_world_ms
                + cur_profiler.tick_mobs_ms
                + cur_profiler.tick_entities_ms;
        }

        hand_speed
    }

    fn resolve_mob_player_collisions_and_melee(&mut self, is_creative: bool) {
        for mob in &mut self.mob_manager.mobs {
            if mob.is_dead {
                continue;
            }
            let dx = self.camera.position.x - mob.pos.x;
            let dz = self.camera.position.z - mob.pos.z;
            let dist_sq = dx * dx + dz * dz;

            let min_dist = mob.kind.half_width() + 0.35;
            if dist_sq < min_dist * min_dist && (mob.pos.y - self.camera.position.y).abs() < 1.6 {
                let dist = dist_sq.sqrt().max(0.001);
                let push = (min_dist - dist) * 0.5;
                let push_x = (dx / dist) * push;
                let push_z = (dz / dist) * push;
                mob.velocity.x -= push_x * 3.0;
                mob.velocity.z -= push_z * 3.0;
                self.horiz_velocity.x += push_x * 3.0;
                self.horiz_velocity.z += push_z * 3.0;
            }

            let is_angry_golem = mob.kind == crate::gameplay::mob::MobKind::IronGolem
                && mob.player_anger_timer > 0.0
                && mob.attack_cooldown <= 0.0;
            let is_hostile_melee =
                mob.kind.is_hostile() && mob.kind != crate::gameplay::mob::MobKind::Reaper;
            let attack_range = if is_angry_golem { 2.2 } else { 1.4 };
            if !is_creative
                && (is_hostile_melee || is_angry_golem)
                && dist_sq < attack_range * attack_range
                && (mob.pos.y - self.camera.position.y).abs() < 1.8
            {
                if is_angry_golem {
                    mob.attack_cooldown = 1.0;
                    self.audio
                        .play_sound(crate::audio::SoundId::IronGolemAttack, 1.0, 1.0);
                }
                let to_mob = (mob.pos - self.camera.position).normalized();
                let is_front = to_mob.dot(self.camera.forward()) > 0.35;
                let damage = if is_angry_golem { 7.0 } else { 2.0 };
                let (took_damage, blocked, item_broke) =
                    self.player.take_physical_damage(damage, is_front);
                if item_broke {
                    self.audio
                        .play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                }
                if blocked {
                    self.audio
                        .play_sound(crate::audio::SoundId::ShieldBlock, 1.0, 1.0);
                    mob.velocity.x -= to_mob.x * 6.0;
                    mob.velocity.z -= to_mob.z * 6.0;
                    mob.velocity.y += 2.0;
                    self.damage_tilt_timer = 0.15;
                    crate::log_info!(
                        "COMBAT",
                        "Player blocked attack from {:?} with shield!",
                        mob.kind
                    );
                } else if took_damage {
                    if is_angry_golem {
                        self.horiz_velocity.x -= to_mob.x * 5.0;
                        self.horiz_velocity.z -= to_mob.z * 5.0;
                        self.vertical_velocity = self.vertical_velocity.max(3.0);
                    }
                    self.damage_tilt_timer = 0.4;
                    self.damage_tilt_dir = if mob.pos.x > self.camera.position.x {
                        -1.0
                    } else {
                        1.0
                    };
                    if self.settings.vibration {
                        self.pending_rumble = Some(RumbleRequest {
                            strong: 0.85,
                            weak: 0.6,
                            duration_ms: 220,
                        });
                    }
                    self.audio
                        .play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                    crate::log_info!(
                        "PLAYER",
                        "Player took damage from {:?} (health: {:.1}/{:.1})",
                        mob.kind,
                        self.player.health,
                        self.player.max_health
                    );
                }
            }
        }
    }

    fn process_world_explosions_and_weather(&mut self, dt: f32) {
        let pending_explosions: Vec<_> = self.world.pending_explosions.drain(..).collect();
        for exp_pos in pending_explosions {
            self.particles.spawn_explosion(exp_pos);
            self.play_world_sound(crate::audio::SoundId::Explosion, exp_pos, 1.0, 1.0);
            self.mob_manager.apply_explosion(exp_pos, 5.0, 30.0);
            self.process_mob_sounds();
            let dist = (self.camera.position - exp_pos).length();
            if dist < 12.0 {
                let factor = (1.0 - dist / 12.0).clamp(0.2, 1.0);
                self.request_rumble(factor, factor * 0.9, (factor * 450.0) as u32);
            }
            if dist < 6.0 {
                let dmg = (1.0 - dist / 6.0) * 18.0;
                if self.player.take_damage(dmg) {
                    self.damage_tilt_timer = 0.4;
                    self.damage_tilt_dir = if exp_pos.x > self.camera.position.x {
                        -1.0
                    } else {
                        1.0
                    };
                    self.audio
                        .play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                    self.request_rumble(1.0, 0.9, 450);
                    crate::log_info!(
                        "PLAYER",
                        "Player took {:.1} explosion damage (health: {:.1}/{:.1})",
                        dmg,
                        self.player.health,
                        self.player.max_health
                    );
                }
            }
        }

        self.world.update_furnaces(dt);
        self.particles.update(dt);

        if let Some(strike) = self.world.update_weather(dt, self.camera.position) {
            let strike_pos = strike.pos;
            let dist = (strike_pos - self.camera.position).length();
            if dist < 45.0 {
                self.audio
                    .play_sound(crate::audio::SoundId::ThunderClose, 1.0, 1.0);
                self.request_rumble(0.9, 1.0, 600);
            } else {
                self.audio
                    .play_sound(crate::audio::SoundId::ThunderDistant, 0.85, 0.95);
            }
            for mob in &mut self.mob_manager.mobs {
                let d = (mob.pos - strike_pos).length();
                if d < 4.0 {
                    let kb = (mob.pos - strike_pos).normalized();
                    mob.take_damage(15.0, kb);
                    self.particles.spawn_mob_hit(mob.pos);
                }
            }
            if dist < 4.0 {
                let dmg = (1.0 - dist / 4.0) * 15.0;
                if self.player.take_damage(dmg) {
                    self.damage_tilt_timer = 0.4;
                    self.damage_tilt_dir = 1.0;
                    self.audio
                        .play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                    self.request_rumble(1.0, 1.0, 500);
                }
            }
            self.particles.spawn_explosion(strike_pos);
            crate::log_info!(
                "WEATHER",
                "Lightning struck at ({:.1}, {:.1}, {:.1}), distance: {:.1}m",
                strike_pos.x,
                strike_pos.y,
                strike_pos.z,
                dist
            );
        }
    }
}
