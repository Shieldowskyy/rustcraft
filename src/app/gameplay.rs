use std::fmt::Write;

use crate::core::controls::Action;
use crate::core::math::{IVec3, Vec3};
#[cfg(not(target_os = "vita"))]
use crate::core::math::Mat4;
use crate::core::profiler::{FrameProfiler, MemoryBreakdown};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::Mob;
use crate::gameplay::physics::{
    block_intersects_player, move_player_with_collision, player_touches_ladder,
    CROUCH_EYE_OFFSET, CROUCH_HEIGHT, FLY_SPEED, GRAVITY, JUMP_SPEED, PLAYER_HALF_WIDTH,
    SNEAK_SPEED, SPRINT_JUMP_BOOST, SPRINT_SPEED, STANDING_EYE_OFFSET, STANDING_HEIGHT,
    SWIM_SPEED, TERMINAL_VELOCITY, WALK_SPEED,
};
use crate::gameplay::player::GameMode;
use crate::world::block::{BlockId, Face};
use crate::world::storage::{get_save_metadata, save_path};
use super::{AppOutput, CameraPerspective, GameState, InputState, RumbleRequest, RustCraftApp};

fn can_place_block_in(existing: BlockId) -> bool {
    matches!(existing, BlockId::Air | BlockId::Water)
}

fn block_intersects_live_mob(place_pos: IVec3, mobs: &[Mob]) -> bool {
    mobs.iter()
        .any(|mob| !mob.is_dead && mob.intersects_block(place_pos))
}

impl RustCraftApp {
    fn network_world_snapshot(&self) -> crate::net::protocol::WorldSnapshot {
        use crate::net::protocol::*;

        let network_item = |item: Option<ItemStack>| {
            item.map(|item| PlayerItemStack {
                block: item.block.to_u8(),
                count: item.count,
                durability: item.durability,
            })
        };
        let weather = &self.world.weather;
        let lightning = weather
            .active_lightning
            .iter()
            .map(|bolt| LightningEntry {
                x: bolt.pos.x,
                y: bolt.pos.y,
                z: bolt.pos.z,
                timer: bolt.timer,
                max_duration: bolt.max_duration,
                seed: bolt.seed,
            })
            .collect();

        WorldSnapshot {
            items: self
                .items
                .iter()
                .map(|item| WorldItemEntry {
                    x: item.position.x,
                    y: item.position.y,
                    z: item.position.z,
                    velocity_x: item.velocity.x,
                    velocity_y: item.velocity.y,
                    velocity_z: item.velocity.z,
                    block: item.block.to_u8(),
                    count: item.count,
                    pickup_delay: item.pickup_delay,
                    age: item.age,
                })
                .collect(),
            chests: self
                .world
                .chests
                .iter()
                .map(|(pos, chest)| ChestSnapshot {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    items: chest.items.map(network_item),
                })
                .collect(),
            furnaces: self
                .world
                .furnaces
                .iter()
                .map(|(pos, furnace)| FurnaceSnapshot {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    input: network_item(furnace.input),
                    fuel: network_item(furnace.fuel),
                    output: network_item(furnace.output),
                    burn_time_remaining: furnace.burn_time_remaining,
                    burn_time_total: furnace.burn_time_total,
                    cook_progress: furnace.cook_progress,
                })
                .collect(),
            primed_tnts: self
                .world
                .primed_tnts
                .iter()
                .map(|tnt| PrimedTntEntry {
                    x: tnt.pos.x,
                    y: tnt.pos.y,
                    z: tnt.pos.z,
                    velocity_x: tnt.velocity.x,
                    velocity_y: tnt.velocity.y,
                    velocity_z: tnt.velocity.z,
                    fuse: tnt.fuse,
                })
                .collect(),
            arrows: self
                .arrows
                .iter()
                .map(|arrow| ArrowEntry {
                    x: arrow.position.x,
                    y: arrow.position.y,
                    z: arrow.position.z,
                    velocity_x: arrow.velocity.x,
                    velocity_y: arrow.velocity.y,
                    velocity_z: arrow.velocity.z,
                    shooter_is_player: arrow.shooter_is_player,
                    in_ground: arrow.in_ground,
                    lifetime: arrow.lifetime,
                    damage: arrow.damage,
                })
                .collect(),
            weather: WeatherSnapshot {
                current_type: match weather.current_type {
                    crate::world::WeatherType::Clear => 0,
                    crate::world::WeatherType::Rain => 1,
                    crate::world::WeatherType::Thunder => 2,
                },
                timer: weather.timer,
                rain_strength: weather.rain_strength,
                thunder_strength: weather.thunder_strength,
                lightning_timer: weather.lightning_timer,
                lightning_flash: weather.lightning_flash,
                rng_state: weather.network_rng_state(),
                active_lightning: lightning,
            },
        }
    }

    fn apply_network_world_snapshot(&mut self, snapshot: crate::net::protocol::WorldSnapshot) {
        use crate::net::protocol::PlayerItemStack;

        let world_item = |item: Option<PlayerItemStack>| {
            item.map(|item| ItemStack {
                block: BlockId::from_u8(item.block),
                count: item.count,
                durability: item.durability,
            })
        };
        self.items = snapshot
            .items
            .into_iter()
            .map(|item| {
                let mut entity = ItemEntity::new(
                    Vec3::new(item.x, item.y, item.z),
                    Vec3::new(item.velocity_x, item.velocity_y, item.velocity_z),
                    BlockId::from_u8(item.block),
                    item.count,
                );
                entity.pickup_delay = item.pickup_delay;
                entity.age = item.age;
                entity
            })
            .collect();
        self.world.chests = snapshot
            .chests
            .into_iter()
            .map(|chest| {
                (
                    IVec3::new(chest.x, chest.y, chest.z),
                    crate::world::ChestData {
                        items: chest.items.map(world_item),
                    },
                )
            })
            .collect();
        self.world.furnaces = snapshot
            .furnaces
            .into_iter()
            .map(|furnace| {
                (
                    IVec3::new(furnace.x, furnace.y, furnace.z),
                    crate::world::FurnaceData {
                        input: world_item(furnace.input),
                        fuel: world_item(furnace.fuel),
                        output: world_item(furnace.output),
                        burn_time_remaining: furnace.burn_time_remaining,
                        burn_time_total: furnace.burn_time_total,
                        cook_progress: furnace.cook_progress,
                    },
                )
            })
            .collect();
        self.world.primed_tnts = snapshot
            .primed_tnts
            .into_iter()
            .map(|tnt| crate::world::PrimedTnt {
                pos: Vec3::new(tnt.x, tnt.y, tnt.z),
                velocity: Vec3::new(tnt.velocity_x, tnt.velocity_y, tnt.velocity_z),
                fuse: tnt.fuse,
            })
            .collect();
        self.arrows = snapshot
            .arrows
            .into_iter()
            .map(|arrow| {
                let mut entity = crate::gameplay::ArrowEntity::new(
                    Vec3::new(arrow.x, arrow.y, arrow.z),
                    Vec3::new(arrow.velocity_x, arrow.velocity_y, arrow.velocity_z),
                    arrow.shooter_is_player,
                    arrow.damage,
                );
                entity.in_ground = arrow.in_ground;
                entity.lifetime = arrow.lifetime;
                entity
            })
            .collect();
        let weather = snapshot.weather;
        self.world.weather.set_network_state(
            match weather.current_type {
                0 => crate::world::WeatherType::Clear,
                1 => crate::world::WeatherType::Rain,
                _ => crate::world::WeatherType::Thunder,
            },
            weather.timer,
            weather.rain_strength,
            weather.thunder_strength,
            weather.lightning_timer,
            weather.lightning_flash,
            weather.rng_state,
            weather
                .active_lightning
                .into_iter()
                .map(|bolt| {
                    let mut strike = crate::world::LightningStrike::new(
                        Vec3::new(bolt.x, bolt.y, bolt.z),
                        bolt.max_duration,
                        bolt.seed,
                    );
                    strike.timer = bolt.timer;
                    strike
                })
                .collect(),
        );
    }

    pub(crate) fn update_and_render_playing(
        &mut self,
        dt: f32,
        input: &InputState,
        out: &mut AppOutput,
    ) {
        let left_down = input.mouse_left;
        let right_down = input.mouse_right;
        let f5_down = input.key_f5;
                self.chat.update(dt);

                let player_data = self.net_client.is_some().then(|| self.multiplayer_player_data());
                let network_block_changes = self.world.take_network_changes().collect::<Vec<_>>();
                let mut received_player_data = None;
                let mut received_world_snapshot = None;
                let outgoing_world_snapshot = self
                    .net_client
                    .as_ref()
                    .filter(|client| client.can_send_world_snapshot())
                    .map(|_| self.network_world_snapshot());
                if let Some(client) = &mut self.net_client {
                    if client.last_move_sent.elapsed() >= std::time::Duration::from_millis(50) {
                        client.send_position(
                            self.camera.position,
                            self.camera.yaw,
                            self.camera.pitch,
                            self.crouch_toggled,
                            self.sprint_toggled,
                        );
                        client.last_move_sent = std::time::Instant::now();
                    }
                    let mut events = std::mem::take(&mut self.pending_network_events);
                    events.extend(client.update(dt));
                    let mut should_disconnect = None;
                    let mut pending_mods = Vec::new();
                    let mut spawned_authoritative_mob = false;
                    for ev in events {
                        match ev {
                            crate::net::NetworkEvent::BlockChanged { x, y, z, block } => {
                                self.world.set_network_block(IVec3::new(x, y, z), BlockId::from_u8(block));
                            }
                            crate::net::NetworkEvent::ChatMessage { sender, text, is_system } => {
                                if is_system {
                                    self.chat.add_info(format!("[{}] {}", sender, text));
                                } else {
                                    self.chat.add_message(format!("<{}> {}", sender, text), crate::ui::pack_color(1.0, 1.0, 1.0));
                                }
                            }
                            crate::net::NetworkEvent::TimeSync { time_of_day } => {
                                self.world.time_of_day = time_of_day;
                            }
                            crate::net::NetworkEvent::Teleport { x, y, z } => {
                                self.camera.position = Vec3::new(x, y, z);
                            }
                            crate::net::NetworkEvent::PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z } => {
                                let is_local = client.local_id == Some(victim_id);
                                if is_local {
                                    if self.player.take_damage(damage) {
                                        self.damage_tilt_timer = 0.4;
                                        self.damage_tilt_dir = if knockback_x > 0.0 { -1.0 } else { 1.0 };
                                        self.horiz_velocity.x += knockback_x * 8.0;
                                        self.horiz_velocity.z += knockback_z * 8.0;
                                        self.vertical_velocity = 3.5;
                                        self.particles.spawn_mob_hit(self.camera.position);
                                        if self.settings.vibration {
                                            self.pending_rumble = Some(RumbleRequest { strong: 0.85, weak: 0.6, duration_ms: 220 });
                                        }
                                        crate::log_info!("PVP", "Player took {:.1} damage from player ID {}", damage, attacker_id);
                                    }
                                } else if let Some(rp) = client.remote_players.get(&victim_id) {
                                    self.particles.spawn_mob_hit(rp.pos);
                                }
                            }
                            crate::net::NetworkEvent::MobSnapshot { mobs } => {
                                if let Err(error) = self.mob_manager.apply_network_snapshot(&mobs) {
                                    crate::log_warn!("NETWORK", "Rejected mob snapshot: {}", error);
                                }
                            }
                            crate::net::NetworkEvent::MobAttack {
                                attacker_id,
                                mob_id,
                                damage,
                                direction,
                            } => {
                                if client.local_id == client.mob_authority_id
                                    && client.local_id != Some(attacker_id)
                                {
                                    if let Some(mob) = self
                                        .mob_manager
                                        .mobs
                                        .iter_mut()
                                        .find(|mob| mob.id == mob_id)
                                    {
                                        let hit_pos = mob.center_pos();
                                        mob.take_damage(damage, direction);
                                        self.particles.spawn_mob_hit(hit_pos);
                                    }
                                }
                            }
                            crate::net::NetworkEvent::MobSpawn { kind, pos } => {
                                if client.local_id == client.mob_authority_id {
                                    if let Some(kind) =
                                        crate::gameplay::mob::MobKind::ALL.get(kind as usize)
                                    {
                                        self.mob_manager.spawn_mob(*kind, pos);
                                        spawned_authoritative_mob = true;
                                    } else {
                                        crate::log_warn!(
                                            "NETWORK",
                                            "Rejected mob spawn with unknown kind {}",
                                            kind
                                        );
                                    }
                                }
                            }
                            crate::net::NetworkEvent::WorldSnapshot { snapshot } => {
                                received_world_snapshot = Some(snapshot);
                            }
                            crate::net::NetworkEvent::PlayerDataLoaded { data } => {
                                received_player_data = Some(data);
                            }
                            crate::net::NetworkEvent::ModsSynchronized { mods } => {
                                pending_mods.extend(mods);
                            }
                            crate::net::NetworkEvent::Disconnected { reason } => {
                                should_disconnect = Some(reason);
                            }
                            _ => {}
                        }
                    }
                    if spawned_authoritative_mob {
                        client.send_mob_snapshot_now(&self.mob_manager);
                    } else {
                        client.send_mob_snapshot(&self.mob_manager);
                    }
                    if let Some(snapshot) = outgoing_world_snapshot {
                        client.send_world_snapshot(snapshot);
                    }
                    for (pos, block) in network_block_changes {
                        client.send_block_change(pos.x, pos.y, pos.z, block);
                    }
                    if let Some(data) = player_data.clone() {
                        client.send_player_data(data, false);
                    }
                    if !pending_mods.is_empty() {
                        self.sync_server_mods(pending_mods);
                    }
                    if let Some(reason) = should_disconnect {
                        self.handle_multiplayer_disconnect(reason);
                        out.request_release_mouse = true;
                    }
                }
                if let Some(snapshot) = received_world_snapshot {
                    self.apply_network_world_snapshot(snapshot);
                }
                if self.is_multiplayer {
                    if let Some(data) = received_player_data {
                        match data {
                            Some(data) => self.apply_multiplayer_player_data(data),
                            None => {
                                if let (Some(client), Some(data)) = (&mut self.net_client, player_data) {
                                    client.send_player_data(data, true);
                                }
                            }
                        }
                    }
                }

                self.telemetry_timer += dt;
                if self.telemetry_timer >= 5.0 {
                    self.telemetry_timer = 0.0;
                    crate::log_debug!(
                        "TELEMETRY",
                        "FPS: {} | Pos: ({:.1}, {:.1}, {:.1}) | Chunks: {} (pending: {}) | Mobs: {} | Items: {} | HP: {:.1}/{:.1} | Hunger: {:.1}/{:.1}",
                        self.fps_display,
                        self.camera.position.x,
                        self.camera.position.y,
                        self.camera.position.z,
                        self.world.chunks.len(),
                        self.world.pending_chunks_count(),
                        self.mob_manager.mobs.len(),
                        self.items.len(),
                        self.player.health,
                        self.player.max_health,
                        self.player.hunger,
                        self.player.max_hunger,
                    );
                }

                let open_chat_requested = (input.key_t && !self.prev_t_down) || self.controls.is_just_pressed(Action::OpenChat);
                let open_cmd_requested = (input.key_slash && !self.prev_slash_down) || self.controls.is_just_pressed(Action::OpenCommand);
                let chat_was_open = self.chat.is_open;

                if !self.chat.is_open {
                    if open_cmd_requested {
                        self.chat.open("/");
                        self.horiz_velocity = Vec3::zero();
                        self.fly_velocity = Vec3::zero();
                        out.request_release_mouse = true;
                        crate::log_debug!("CHAT", "Opened command prompt");
                    } else if open_chat_requested {
                        self.chat.open("");
                        self.horiz_velocity = Vec3::zero();
                        self.fly_velocity = Vec3::zero();
                        out.request_release_mouse = true;
                        crate::log_debug!("CHAT", "Opened chat prompt");
                    }
                } else {
                    let mut player_names: Vec<String> = self
                        .net_client
                        .as_ref()
                        .map(|client| client.player_list.iter().map(|player| player.nickname.clone()).collect())
                        .unwrap_or_default();
                    if !player_names.iter().any(|name| name == &self.settings.nickname) {
                        player_names.push(self.settings.nickname.clone());
                    }
                    if let Some(action) = self.chat.handle_input(input, &player_names) {
                        match action {
                            crate::gameplay::chat::ChatAction::Close => {
                                out.request_capture_mouse = true;
                                crate::log_debug!("CHAT", "Closed chat prompt");
                            }
                            crate::gameplay::chat::ChatAction::Execute(text) => {
                                out.request_capture_mouse = true;
                                let command_parts: Vec<&str> = text.split_whitespace().collect();
                                let tp_to_player = command_parts.len() == 2
                                    && matches!(command_parts[0].to_ascii_lowercase().as_str(), "/tp" | "/teleport");
                                let kick_player = command_parts.len() >= 2
                                    && command_parts[0].eq_ignore_ascii_case("/kick");
                                let time_command = command_parts
                                    .first()
                                    .is_some_and(|command| command.eq_ignore_ascii_case("/time"));
                                let tps_command = command_parts
                                    .first()
                                    .is_some_and(|command| command.eq_ignore_ascii_case("/tps"));
                                if kick_player && !self.is_hosting_multiplayer {
                                    self.chat.add_info("Only the multiplayer host can kick players.");
                                } else if (tp_to_player || kick_player || time_command || tps_command)
                                    && self.net_client.is_some()
                                {
                                    if let Some(client) = &mut self.net_client {
                                        client.send_chat(&text);
                                    }
                                } else if text.starts_with('/') {
                                    crate::log_info!("COMMAND", "Executing: {}", text);
                                    let mut chunks_reloaded = false;
                                    let mut ctx = crate::gameplay::chat::CommandContext {
                                        player: &mut self.player,
                                        camera: &mut self.camera,
                                        world: &mut self.world,
                                        mob_manager: &mut self.mob_manager,
                                        items: &mut self.items,
                                        particles: &mut self.particles,
                                        is_flying: &mut self.fly_mode,
                                        fly_velocity: &mut self.fly_velocity,
                                        horiz_velocity: &mut self.horiz_velocity,
                                        reload_chunks_requested: &mut chunks_reloaded,
                                    };
                                    self.chat.execute_command(&text, &mut ctx);
                                    if chunks_reloaded {
                                        self.renderer.clear_mesh_cache();
                                        crate::log_info!("WORLD", "Cleared mesh cache following chunk reload command");
                                    }
                                } else {
                                    crate::log_info!("CHAT", "<{}> {}", self.settings.nickname, text);
                                    if let Some(client) = &mut self.net_client {
                                        client.send_chat(&text);
                                    } else {
                                        self.chat.add_player_chat(&self.settings.nickname, &text);
                                    }
                                }
                            }
                        }
                    }
                }
                self.chat
                    .update_text_selection(input, self.width, self.height);

                // F3+A: Reload all chunks shortcut
                if input.key_f3 && input.key_a && !self.prev_a_down && !self.chat.is_open {
                    self.world.mark_all_chunks_dirty();
                    self.renderer.clear_mesh_cache();
                    crate::log_info!("WORLD", "Reloaded all chunks via F3+A (chunk count: {})", self.world.chunks.len());
                }

                if !self.chat.is_open && self.controls.is_just_pressed(Action::Save) {
                    let path = save_path(self.save_slot);
                    match self.save_world_progress() {
                        Ok(()) => {
                            println!("Saved world to {path}");
                            crate::log_info!("SAVE", "World successfully saved to slot {} ({})", self.save_slot, path);
                        }
                        Err(error) => eprintln!("{error}"),
                    }
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                }
                if !self.chat.is_open && !chat_was_open && self.controls.is_just_pressed(Action::Pause) {
                    self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(self.renderer.pixel_buffer(), self.width, self.height));
                    self.state = GameState::Paused;
                    out.request_release_mouse = true;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                    crate::log_info!("STATE", "Game paused (opened pause menu)");
                }

                if !self.chat.is_open && self.controls.is_just_pressed(Action::Inventory) {
                    self.crafting_table_open = false;
                    self.state = GameState::Inventory;
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    out.request_release_mouse = true;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                    crate::log_info!("MENU", "Opened player inventory (crafting table: false)");
                }

                if !self.chat.is_open && input.key_m && !self.prev_m_down {
                    self.worldmap.center_x = self.camera.position.x;
                    self.worldmap.center_z = self.camera.position.z;
                    self.state = GameState::WorldMap;
                    out.request_release_mouse = true;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                }

                if !self.chat.is_open && input.key_n && !self.prev_n_down {
                    self.minimap.mode = self.minimap.mode.next();
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                }

                if !self.chat.is_open && input.key_b && !self.prev_b_down {
                    let bpos = self.camera.position.floor_to_ivec();
                    let count = self.minimap.waypoints.len() + 1;
                    let name = format!("Waypoint {}", count);
                    let color = crate::ui::minimap::WAYPOINT_COLORS[(count - 1) % crate::ui::minimap::WAYPOINT_COLORS.len()].0;
                    self.minimap.waypoints.push(crate::ui::minimap::Waypoint {
                        name: name.clone(),
                        x: bpos.x,
                        y: bpos.y,
                        z: bpos.z,
                        color,
                        enabled: true,
                    });
                    crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
                    self.chat.add_message(format!("Dodano punkt nawigacyjny: {} ({}, {}, {})", name, bpos.x, bpos.y, bpos.z), crate::ui::pack_color(0.4, 1.0, 0.4));
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                }

                let any_click = left_down || right_down;
                if any_click && !input.is_mouse_captured && !self.chat.is_open {
                    out.request_capture_mouse = true;
                }

                let look_x_dir = if self.settings.invert_z { -1.0 } else { 1.0 };
                let look_y_dir = if self.settings.invert_y { -1.0 } else { 1.0 };

                if input.is_mouse_captured && !self.chat.is_open {
                    self.camera.rotate(
                        input.mouse_dx * 0.0025 * look_x_dir * self.settings.mouse_sensitivity,
                        -input.mouse_dy * 0.0025 * look_y_dir * self.settings.mouse_sensitivity,
                    );
                }

                let gp_look_speed_x = 3.2 * self.settings.gamepad_sensitivity;
                let gp_look_speed_y = 2.8 * self.settings.gamepad_sensitivity;
                if (input.gp_right_stick_x.abs() > 0.0 || input.gp_right_stick_y.abs() > 0.0) && !self.chat.is_open {
                    self.camera.rotate(
                        input.gp_right_stick_x * gp_look_speed_x * dt * look_x_dir,
                        input.gp_right_stick_y * gp_look_speed_y * dt * look_y_dir,
                    );
                }

                let forward = self.camera.forward_flat();
                let right = self.camera.right_flat();
                let mut wish_dir = Vec3::zero();

                if !self.chat.is_open {
                    if self.controls.is_active(Action::MoveForward) { wish_dir = wish_dir + forward; }
                    if self.controls.is_active(Action::MoveBackward) { wish_dir = wish_dir - forward; }
                    if self.controls.is_active(Action::StrafeLeft) { wish_dir = wish_dir - right; }
                    if self.controls.is_active(Action::StrafeRight) { wish_dir = wish_dir + right; }
                    if input.gp_left_stick_y.abs() > 0.0 || input.gp_left_stick_x.abs() > 0.0 {
                        wish_dir = wish_dir + forward * input.gp_left_stick_y + right * input.gp_left_stick_x;
                    }
                    if wish_dir.length_sq() > 0.0001 {
                        wish_dir = wish_dir.normalized();
                    }
                }

                if self.jump_double_tap_timer > 0.0 {
                    self.jump_double_tap_timer = (self.jump_double_tap_timer - dt).max(0.0);
                }

                if !self.chat.is_open && self.controls.is_just_pressed(Action::Jump) && self.player.gamemode == GameMode::Creative {
                    if self.jump_double_tap_timer > 0.0 {
                        self.fly_mode = !self.fly_mode;
                        self.jump_double_tap_timer = 0.0;
                        self.horiz_velocity = Vec3::zero();
                        self.fly_velocity = Vec3::zero();
                        if self.fly_mode {
                            self.vertical_velocity = 0.0;
                        }
                        crate::log_info!("PLAYER", "Flight mode toggled by double-tap jump: {}", self.fly_mode);
                    } else {
                        self.jump_double_tap_timer = 0.35;
                    }
                }

                if !self.chat.is_open && self.controls.is_just_pressed(Action::ToggleFly) {
                    self.fly_mode = !self.fly_mode;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                    crate::log_debug!("PLAYER", "Flight mode: {}", self.fly_mode);
                }

                if !self.chat.is_open && self.controls.is_just_pressed(Action::ToggleGamemode) {
                    self.player.gamemode = if self.player.gamemode == GameMode::Survival {
                        GameMode::Creative
                    } else {
                        self.fly_mode = false;
                        GameMode::Survival
                    };
                    crate::log_info!("PLAYER", "Gamemode changed to {:?}", self.player.gamemode);
                }

                // F5 Camera Perspective cycling (First Person -> Third Person Back -> Third Person Front)
                if !self.chat.is_open && ((f5_down && !self.prev_f5_down) || input.gp_select_just_pressed) {
                    self.camera_perspective = match self.camera_perspective {
                        CameraPerspective::FirstPerson => CameraPerspective::ThirdPersonBack,
                        CameraPerspective::ThirdPersonBack => CameraPerspective::ThirdPersonFront,
                        CameraPerspective::ThirdPersonFront => CameraPerspective::FirstPerson,
                    };
                    let mode_str = match self.camera_perspective {
                        CameraPerspective::FirstPerson => "Perspective: First Person",
                        CameraPerspective::ThirdPersonBack => "Perspective: Third Person (Back)",
                        CameraPerspective::ThirdPersonFront => "Perspective: Third Person (Front)",
                    };
                    self.chat.add_message(mode_str.to_string(), crate::ui::pack_color(0.4, 0.9, 1.0));
                    crate::log_info!("CAMERA", "{}", mode_str);
                }

                let sneak_btn_down = !self.chat.is_open && self.controls.is_active(Action::Sneak);
                let sneak_just_pressed = !self.chat.is_open && self.controls.is_just_pressed(Action::Sneak);
                self.prev_sneak_btn = sneak_btn_down;

                if self.settings.sneak_mode == crate::core::settings::SneakMode::Toggle {
                    if sneak_just_pressed {
                        self.crouch_toggled = !self.crouch_toggled;
                    }
                } else {
                    self.crouch_toggled = sneak_btn_down;
                }

                let is_crouching = !self.fly_mode && self.crouch_toggled;

                let target_eye = if is_crouching { CROUCH_EYE_OFFSET } else { STANDING_EYE_OFFSET };
                let target_h = if is_crouching { CROUCH_HEIGHT } else { STANDING_HEIGHT };
                let t_eye = 1.0 - (-14.0 * dt).exp();
                let old_eye = self.current_eye_offset;
                self.current_eye_offset += (target_eye - self.current_eye_offset) * t_eye;
                self.current_player_height += (target_h - self.current_player_height) * t_eye;

                if !self.fly_mode {
                    self.camera.position.y += self.current_eye_offset - old_eye;
                }

                let can_sprint = self.player.gamemode == GameMode::Creative || self.player.hunger > 6.0;
                let sprint_btn_down = !self.chat.is_open && (self.controls.is_active(Action::Sprint) || input.gp_left_stick_y > 0.85);
                let sprint_just_pressed = !self.chat.is_open && (self.controls.is_just_pressed(Action::Sprint) || (input.gp_left_stick_y > 0.85 && !self.prev_sprint_btn));
                self.prev_sprint_btn = sprint_btn_down;

                if self.settings.sprint_mode == crate::core::settings::SprintMode::Toggle {
                    if sprint_just_pressed && can_sprint && !is_crouching {
                        self.sprint_toggled = !self.sprint_toggled;
                    }
                    if !can_sprint || is_crouching || wish_dir.length_sq() <= 0.01 {
                        self.sprint_toggled = false;
                    }
                } else {
                    self.sprint_toggled = sprint_btn_down;
                }

                let is_sprinting = !is_crouching
                    && can_sprint
                    && self.sprint_toggled
                    && wish_dir.length_sq() > 0.01;

                let target_fov_deg = if is_sprinting {
                    self.settings.fov_deg + 15.0
                } else if is_crouching {
                    self.settings.fov_deg - 5.0
                } else {
                    self.settings.fov_deg
                };
                let target_fov = target_fov_deg.to_radians();
                let t_fov = 1.0 - (-8.0 * dt).exp();
                self.camera.fov_y += (target_fov - self.camera.fov_y) * t_fov;

                let move_speed = if is_crouching {
                    SNEAK_SPEED
                } else if is_sprinting {
                    SPRINT_SPEED
                } else {
                    WALK_SPEED
                };

                if self.fly_mode {
                    let mut move_dir = wish_dir;
                    if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                        move_dir.y += 1.0;
                    }
                    if !self.chat.is_open && self.controls.is_active(Action::Sneak) {
                        move_dir.y -= 1.0;
                    }
                    if move_dir.length_sq() > 0.0001 {
                        move_dir = move_dir.normalized();
                    }

                    let fly_fast = !self.chat.is_open && self.controls.is_active(Action::Sprint);
                    let fly_mult = if fly_fast { 2.0 } else { 1.0 };
                    let target_fly = move_dir * (FLY_SPEED * fly_mult);
                    let fly_damping = 14.0;
                    let t_fly = 1.0 - (-fly_damping * dt).exp();
                    self.fly_velocity = self.fly_velocity.lerp(target_fly, t_fly);

                    self.camera.position = self.camera.position + self.fly_velocity * dt;
                    self.vertical_velocity = 0.0;
                    self.horiz_velocity = Vec3::zero();
                    self.player.head_underwater = matches!(self.world.get_block(self.camera.position.floor_to_ivec()), BlockId::Water | BlockId::Kelp | BlockId::Seagrass);
                    self.player.air = self.player.max_air;
                    self.drown_damage_timer = 0.0;
                } else {
                    let feet_pos = Vec3::new(
                        self.camera.position.x,
                        self.camera.position.y - self.current_eye_offset - 0.08,
                        self.camera.position.z,
                    );
                    let feet_block = self.world.get_block(feet_pos.floor_to_ivec());
                    let eye_block = self.world.get_block(self.camera.position.floor_to_ivec());
                    let in_water = matches!(feet_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) || matches!(eye_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass);
                    let in_lava = feet_block == BlockId::Lava || eye_block == BlockId::Lava;

                    // Dolphin's Grace: swimming near a dolphin grants * 1.85 swim speed boost
                    let near_dolphin = in_water && self.mob_manager.mobs.iter().any(|m| {
                        m.kind == crate::gameplay::mob::MobKind::Dolphin && m.health > 0.0 && (m.pos - self.camera.position).length_sq() < 196.0
                    });
                    let swim_speed = if near_dolphin { SWIM_SPEED * 1.85 } else { SWIM_SPEED };

                    let target_horiz = wish_dir * (if in_lava { SWIM_SPEED * 0.55 } else if in_water { swim_speed } else { move_speed });
                    let on_ice = feet_block == BlockId::Ice;
                    let in_ladder = player_touches_ladder(
                        &self.world,
                        Vec3::new(
                            self.camera.position.x,
                            self.camera.position.y - self.current_eye_offset,
                            self.camera.position.z,
                        ),
                        PLAYER_HALF_WIDTH,
                        self.current_player_height,
                    );
                    let damping = if in_lava {
                        12.0
                    } else if in_water {
                        8.0
                    } else if self.on_ground {
                        if on_ice { 2.0 } else if is_crouching { 18.0 } else { 15.0 }
                    } else {
                        2.0
                    };
                    let t_horiz = 1.0 - (-damping * dt).exp();
                    self.horiz_velocity = self.horiz_velocity.lerp(target_horiz, t_horiz);

                    if in_ladder {
                        self.player.fall_distance = 0.0;
                        if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                            self.vertical_velocity = 3.5;
                        } else if is_crouching {
                            self.vertical_velocity = 0.0;
                        } else if wish_dir.length_sq() > 0.01 {
                            self.vertical_velocity = 2.4;
                        } else {
                            self.vertical_velocity = -1.2;
                        }
                    } else if in_lava {
                        self.vertical_velocity -= 2.5 * dt;
                        if self.vertical_velocity < -1.8 {
                            self.vertical_velocity = -1.8;
                        }
                        if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                            self.vertical_velocity = 2.2;
                        }
                    } else if in_water {
                        self.vertical_velocity -= 4.0 * dt;
                        if self.vertical_velocity < -2.5 {
                            self.vertical_velocity = -2.5;
                        }
                        if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                            self.vertical_velocity = 3.2;
                        }
                    } else {
                        self.vertical_velocity -= GRAVITY * dt;
                        if self.vertical_velocity < -TERMINAL_VELOCITY {
                            self.vertical_velocity = -TERMINAL_VELOCITY;
                        }

                        let mut should_jump = !self.chat.is_open && self.controls.is_active(Action::Jump) && self.on_ground;

                        // Auto-jump logic
                        if !should_jump && self.settings.jump_mode == crate::core::settings::JumpMode::AutoJump && self.on_ground && wish_dir.length_sq() > 0.01 {
                            let check_dir = wish_dir.normalized();
                            let feet_y = self.camera.position.y - self.current_eye_offset;
                            let feet_pos = Vec3::new(self.camera.position.x, feet_y, self.camera.position.z);
                            let probe_dist = 0.65;
                            let step_ahead = check_dir * probe_dist;

                            let probe_feet = (feet_pos + step_ahead + Vec3::new(0.0, 0.2, 0.0)).floor_to_ivec();
                            let probe_head = (feet_pos + step_ahead + Vec3::new(0.0, 1.2, 0.0)).floor_to_ivec();
                            let probe_above = (feet_pos + Vec3::new(0.0, 2.0, 0.0)).floor_to_ivec();

                            let block_feet = self.world.get_block(probe_feet);
                            let block_head = self.world.get_block(probe_head);
                            let block_above = self.world.get_block(probe_above);

                            if block_feet.is_solid() && !block_head.is_solid() && !block_above.is_solid() {
                                should_jump = true;
                            }
                        }

                        if should_jump {
                            self.vertical_velocity = JUMP_SPEED;
                            self.audio.play_sound(crate::audio::SoundId::PlayerJump, 0.75, 1.0);
                            if is_sprinting && wish_dir.length_sq() > 0.01 {
                                self.horiz_velocity = self.horiz_velocity + wish_dir * SPRINT_JUMP_BOOST;
                                self.player.exhaustion += 0.2;
                            } else {
                                self.player.exhaustion += 0.05;
                            }
                        }
                    }

                    let old_y = self.camera.position.y;
                    let motion = Vec3::new(self.horiz_velocity.x, self.vertical_velocity, self.horiz_velocity.z) * dt;
                    let prev_grounded = self.was_on_ground;
                    self.on_ground = move_player_with_collision(
                        &self.world,
                        &mut self.camera.position,
                        motion,
                        &mut self.vertical_velocity,
                        self.current_eye_offset,
                        self.current_player_height,
                        is_crouching,
                    );
                    self.was_on_ground = self.on_ground;
                    if !prev_grounded && self.on_ground && self.player.fall_distance > 0.4 {
                        self.audio.play_sound(crate::audio::SoundId::PlayerLand, 0.85, 1.0);
                    }
                    if self.vertical_velocity < 0.0 && !self.fly_mode && !in_water && !in_lava {
                        self.player.fall_distance += old_y - self.camera.position.y;
                    } else if self.vertical_velocity > 0.0 || self.fly_mode || in_water || in_lava {
                        self.player.fall_distance = 0.0;
                    }

                    // Update audio listener & footstep cadence
                    let horiz_speed = self.horiz_velocity.length();
                    let block_below_pos = IVec3::new(
                        self.camera.position.x.floor() as i32,
                        (self.camera.position.y - self.current_eye_offset - 0.2).floor() as i32,
                        self.camera.position.z.floor() as i32,
                    );
                    let block_below = self.world.get_block(block_below_pos);

                    // Smoothly interpolate postprocess screen color filter towards current biome tint
                    let p_block = self.camera.position.floor_to_ivec();
                    let (current_biome, _) = crate::world::get_biome_at(p_block.x, p_block.z, self.world.seed);
                    let unlocked = self.achievements.visit_biome(current_biome);
                    self.process_achievement_unlocks(unlocked);
                    let target_tint = current_biome.color_filter();
                    let lerp_speed = 1.8;
                    let factor = (1.0 - (-lerp_speed * dt).exp()).clamp(0.0, 1.0);
                    for i in 0..3 {
                        self.biome_filter_tint[i] += (target_tint[i] - self.biome_filter_tint[i]) * factor;
                    }

                    // Low-frequency sparse environmental probe for ambient soundscapes (~3 Hz)
                    self.ambient_probe_timer -= dt;
                    if self.ambient_probe_timer <= 0.0 {
                        self.ambient_probe_timer = 0.35;
                        let p_block = self.camera.position.floor_to_ivec();
                        let (biome, _) = crate::world::get_biome_at(p_block.x, p_block.z, self.world.seed);
                        let is_desert = matches!(biome, crate::world::Biome::Desert | crate::world::Biome::Badlands);

                        // Vertical profile check: solid roof indicates cave / underground
                        let mut solid_above = 0;
                        for &dy in &[2, 4, 8, 14, 22] {
                            let test_pos = IVec3::new(p_block.x, p_block.y + dy, p_block.z);
                            if self.world.get_block(test_pos).is_solid() {
                                solid_above += 1;
                            }
                        }
                        let is_underground = p_block.y < 55 && solid_above >= 2;
                        let sky_exposed = solid_above == 0;

                        // Radial sparse probe around player for water, leaves, sand
                        let mut near_water = in_water;
                        let mut water_pos = if in_water { Some(self.camera.position) } else { None };
                        let mut near_leaves = false;
                        let mut leaves_pos = None;
                        let mut near_sand = block_below == BlockId::Sand;
                        let mut sand_pos = if near_sand { Some(self.camera.position) } else { None };

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
                                water_pos = Some(Vec3::new(probe.x as f32 + 0.5, probe.y as f32 + 0.5, probe.z as f32 + 0.5));
                            } else if b.is_leaf() && !near_leaves {
                                near_leaves = true;
                                leaves_pos = Some(Vec3::new(probe.x as f32 + 0.5, probe.y as f32 + 0.5, probe.z as f32 + 0.5));
                            } else if b == BlockId::Sand && !near_sand {
                                near_sand = true;
                                sand_pos = Some(Vec3::new(probe.x as f32 + 0.5, probe.y as f32 + 0.5, probe.z as f32 + 0.5));
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

                    // Multiplayer player-to-player collision pushing
                    if let Some(client) = &self.net_client {
                        for rp in client.remote_players.values() {
                            let dy = (self.camera.position.y - rp.pos.y).abs();
                            if dy < self.current_player_height {
                                let dx = self.camera.position.x - rp.pos.x;
                                let dz = self.camera.position.z - rp.pos.z;
                                let dist_sq = dx * dx + dz * dz;
                                let min_dist = crate::gameplay::physics::PLAYER_HALF_WIDTH * 2.0; // 0.60m
                                if dist_sq < min_dist * min_dist && dist_sq > 0.00001 {
                                    let dist = dist_sq.sqrt();
                                    let push = (min_dist - dist) * 0.5;
                                    self.camera.position.x += (dx / dist) * push;
                                    self.camera.position.z += (dz / dist) * push;
                                }
                            }
                        }
                    }

                    // Lava contact & fire burning mechanics (burns for ~5 seconds after contact)
                    if in_lava {
                        self.player.fire_timer = 5.0;
                        if self.player.gamemode == GameMode::Survival {
                            self.lava_damage_timer += dt;
                            if self.lava_damage_timer >= 0.5 {
                                self.lava_damage_timer = 0.0;
                                if self.player.take_damage(4.0) {
                                    self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                                    self.damage_tilt_timer = 0.4;
                                    self.damage_tilt_dir = 1.0;
                                    self.particles.spawn_lava_burn(feet_pos);
                                    self.request_rumble(0.85, 0.6, 220);
                                    crate::log_info!("PLAYER", "Player took 4.0 lava direct damage (health: {:.1}/{:.1})", self.player.health, self.player.max_health);
                                }
                            }
                        }
                    } else {
                        self.lava_damage_timer = 0.45;
                        if in_water && self.player.fire_timer > 0.0 {
                            // Water immediately extinguishes burning
                            self.player.fire_timer = 0.0;
                            self.player.fire_damage_timer = 0.0;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 0.5);
                            crate::log_info!("PLAYER", "Player fire extinguished by water");
                        } else if self.player.fire_timer > 0.0 {
                            self.player.fire_timer = (self.player.fire_timer - dt).max(0.0);
                            if self.player.gamemode == GameMode::Survival {
                                self.player.fire_damage_timer += dt;
                                // Fire deals 1.0 damage (~half a heart) every 1.0 second
                                if self.player.fire_damage_timer >= 1.0 {
                                    self.player.fire_damage_timer = 0.0;
                                    if self.player.take_damage(1.0) {
                                        self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.8, 1.0);
                                        self.damage_tilt_timer = 0.3;
                                        self.damage_tilt_dir = if (self.player.fire_timer as i32 % 2) == 0 { 1.0 } else { -1.0 };
                                        self.particles.spawn_lava_burn(feet_pos);
                                        self.request_rumble(0.5, 0.3, 150);
                                        crate::log_info!("PLAYER", "Player took 1.0 fire burn damage (remaining burn: {:.1}s, health: {:.1}/{:.1})", self.player.fire_timer, self.player.health, self.player.max_health);
                                    }
                                }
                            }
                        } else {
                            self.player.fire_damage_timer = 0.0;
                        }
                    }

                    // Oxygen and underwater breathing mechanics
                    let is_head_underwater = matches!(eye_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass);
                    self.player.head_underwater = is_head_underwater;

                    if self.player.gamemode == GameMode::Survival {
                        if is_head_underwater {
                            // 15 seconds to fully exhaust 20 oxygen points (~1.33 pt/s)
                            self.player.air -= (self.player.max_air / 15.0) * dt;
                            if self.player.air <= 0.0 {
                                self.player.air = 0.0;
                                self.drown_damage_timer += dt;
                                if self.drown_damage_timer >= 1.0 {
                                    self.drown_damage_timer = 0.0;
                                    if self.player.take_damage(2.0) {
                                        self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                                        self.damage_tilt_timer = 0.4;
                                        self.damage_tilt_dir = 1.0;
                                        self.particles.spawn_drown_bubbles(self.camera.position);
                                        self.request_rumble(0.7, 0.5, 200);
                                        crate::log_info!("PLAYER", "Player took 2.0 drowning damage (health: {:.1}/{:.1})", self.player.health, self.player.max_health);
                                    }
                                }
                            } else {
                                self.drown_damage_timer = 0.0;
                            }
                        } else {
                            self.drown_damage_timer = 0.0;
                            self.player.air = (self.player.air + 20.0 * dt).min(self.player.max_air);
                        }

                        // Hunger / Saturation / Exhaustion mechanics
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

                        // Natural health regeneration (when hunger >= 18)
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

                        // Starvation damage (when hunger == 0)
                        if self.player.hunger <= 0.0 {
                            self.player.starvation_timer += dt;
                            if self.player.starvation_timer >= 4.0 {
                                self.player.starvation_timer = 0.0;
                                if self.player.take_damage(1.0) {
                                    self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                                    self.damage_tilt_timer = 0.4;
                                    self.damage_tilt_dir = 1.0;
                                    self.request_rumble(0.6, 0.4, 180);
                                    crate::log_info!("PLAYER", "Player took 1.0 starvation damage (health: {:.1}/{:.1})", self.player.health, self.player.max_health);
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
                                self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = 1.0;
                                self.request_rumble(0.85, 0.7, 250);
                                crate::log_info!("PLAYER", "Player took {:.1} fall damage (health: {:.1}/{:.1})", damage, self.player.health, self.player.max_health);
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

                if self.player.invulnerability_timer > 0.0 {
                    self.player.invulnerability_timer -= dt;
                }
                if self.damage_tilt_timer > 0.0 {
                    self.damage_tilt_timer = (self.damage_tilt_timer - dt).max(0.0);
                }

                if self.camera.position.y < -32.0 {
                    if self.player.take_damage(20.0) {
                        self.damage_tilt_timer = 0.4;
                        self.request_rumble(1.0, 1.0, 300);
                        crate::log_info!("PLAYER", "Player took 20.0 void damage at y={:.1} (health: {:.1}/{:.1})", self.camera.position.y, self.player.health, self.player.max_health);
                    }
                    if !self.player.is_dead() {
                        self.camera.position.y = 80.0;
                        self.vertical_velocity = 0.0;
                    }
                }
                if self.camera.position.y > 250.0 {
                    self.camera.position.y = 250.0;
                    self.vertical_velocity = self.vertical_velocity.min(0.0);
                }

                if !self.chat.is_open {
                    if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
                        self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
                    }
                    if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
                        self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
                    }

                    for i in 0..10 {
                        if input.key_digits[i] {
                            self.player.selected_hotbar_slot = i as u8;
                        }
                    }
                }

                self.player.update_held_item_tracker(dt);

                let mut selected_block_id = 0u8;
                if let Some(stack) = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                    selected_block_id = stack.block.to_u8();
                }
                self.renderer.set_selected_block(selected_block_id);

                if self.creative_break_cooldown > 0.0 {
                    self.creative_break_cooldown = (self.creative_break_cooldown - dt).max(0.0);
                }

                let interact_left = !self.chat.is_open && self.controls.is_active(Action::AttackBreak);
                let interact_left_just = !self.chat.is_open && self.controls.is_just_pressed(Action::AttackBreak);

                if interact_left {
                    if self.hand_animation < 0.1 {
                        self.hand_animation = 1.0;
                    }
                    let forward = self.camera.forward();
                    let held_tool = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block);
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
                            crate::log_info!("COMBAT", "Player attacked remote player ID {} dealing {:.1} damage", target_id, dmg);
                            entity_attacked = true;
                        }
                    } else if let Some((mob_idx, _)) =
                        self.mob_manager.raycast_mob(self.camera.position, forward, entity_reach)
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
                            }
                            if kind == crate::gameplay::mob::MobKind::Villager {
                                self.mob_manager.anger_golems_at_villager_attack(mob_pos);
                            }
                            self.break_target = None;
                            self.break_progress = 0.0;
                            self.request_rumble(0.65, 0.65, 120);
                            crate::log_info!("COMBAT", "Player attacked mob {:?} dealing {:.1} damage (remaining HP: {:.1})", kind, dmg, remaining_hp);
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
                                    self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                }
                            }
                        }
                    }

                    if !entity_hit {
                        if self.player.gamemode == GameMode::Creative && self.creative_break_cooldown > 0.0 {
                            self.break_target = None;
                            self.break_progress = 0.0;
                        } else if let Some(hit) = block_hit {
                        let target = hit.block_pos;
                        let broken_block = self.world.get_block(target);

                        if broken_block == BlockId::Tnt && interact_left_just {
                            self.world.prime_tnt(target);
                            self.break_target = None;
                            self.break_progress = 0.0;
                            self.dig_sound_timer = 0.0;
                            crate::log_info!("WORLD", "TNT primed at ({}, {}, {})", target.x, target.y, target.z);
                        } else {
                            if self.break_target != Some(target) {
                                self.break_target = Some(target);
                                self.break_progress = 0.0;
                                self.dig_sound_timer = 0.0;
                            }

                            let sound_pos = Vec3::new(target.x as f32 + 0.5, target.y as f32 + 0.5, target.z as f32 + 0.5);
                            self.dig_sound_timer -= dt;
                            if interact_left_just || self.dig_sound_timer <= 0.0 {
                                self.dig_sound_timer = 0.22; // Minecraft hit cadence
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
                                self.particles.spawn_block_break(Vec3::new(bx - 0.5, by - 0.5, bz - 0.5), color);

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
                                    self.world.bed_pair_at(target).map(|(first, second)| {
                                        if first == target { second } else { first }
                                    })
                                } else {
                                    None
                                };
                                let replace_block = if matches!(broken_block, BlockId::Kelp | BlockId::Seagrass) && target.y <= 20 {
                                    BlockId::Water
                                } else {
                                    BlockId::Air
                                };
                                self.world.set_block(target, replace_block);
                                self.record_discovered_block(broken_block);
                                let unlocked = self.achievements.record_mined(broken_block);
                                self.process_achievement_unlocks(unlocked);
                                if let Some(mate) = bed_mate {
                                    self.world.set_block(mate, BlockId::Air);
                                }

                                // Breaking Kelp breaks all kelp blocks directly above it
                                if broken_block == BlockId::Kelp {
                                    let mut check_pos = target + IVec3::new(0, 1, 0);
                                    while self.world.get_block(check_pos) == BlockId::Kelp {
                                        let rep = if check_pos.y <= 20 { BlockId::Water } else { BlockId::Air };
                                        self.world.set_block(check_pos, rep);
                                        if self.player.gamemode == GameMode::Survival {
                                            let drop_pos = Vec3::new(check_pos.x as f32 + 0.5, check_pos.y as f32 + 0.5, check_pos.z as f32 + 0.5);
                                            let r_drop = Vec3::new(((check_pos.y as f32 * 13.0).sin()) * 1.2, 2.8, ((check_pos.y as f32 * 17.0).cos()) * 1.2);
                                            self.items.push(ItemEntity::new(drop_pos, r_drop, BlockId::Kelp, 1));
                                        }
                                        check_pos.y += 1;
                                    }
                                }

                                let break_sound = crate::audio::SoundId::from_block_break(broken_block);
                                let sound_pos = Vec3::new(bx, by, bz);
                                self.play_world_sound_varied(break_sound, sound_pos, 0.95);
                                self.request_rumble(0.4, 0.25, 75);
                                crate::log_info!("WORLD", "Block broken: {:?} at ({}, {}, {})", broken_block, target.x, target.y, target.z);

                                if self.player.gamemode == GameMode::Survival {
                                    self.player.exhaustion += 0.025;
                                    let selected = self.player.selected_hotbar_slot as usize;
                                    if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                                        if stack.block.is_damageable() {
                                            if stack.damage(1) {
                                                self.player.inventory.hotbar[selected] = None;
                                                self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                            }
                                        }
                                    }
                                    let seed = ((target.x as u32).wrapping_mul(73856093) ^ (target.y as u32).wrapping_mul(19349663) ^ (target.z as u32).wrapping_mul(83492791)) % 1000;
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
                                        self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, drop_id, count));
                                    } else if can_harvest && broken_block != BlockId::Glass {
                                        let drop_block = if matches!(broken_block, BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves) {
                                            if seed % 20 == 0 {
                                                Some(BlockId::Apple)
                                            } else if seed % 10 == 1 {
                                                Some(BlockId::Sapling)
                                            } else {
                                                None
                                            }
                                        } else if broken_block == BlockId::Grass {
                                            if seed % 8 == 0 {
                                                Some(BlockId::WheatSeeds)
                                            } else {
                                                Some(BlockId::Dirt)
                                            }
                                        } else if broken_block == BlockId::WheatCrop {
                                            Some(BlockId::WheatSeeds)
                                        } else if broken_block == BlockId::CarrotCrop {
                                            Some(BlockId::Carrot)
                                        } else if broken_block == BlockId::PotatoCrop {
                                            Some(BlockId::Potato)
                                        } else if broken_block == BlockId::RustStoneOre {
                                            let count = 4 + (seed % 2) as u8;
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, BlockId::RustStone, count));
                                            None
                                        } else if matches!(broken_block, BlockId::RustStoneWireLit) {
                                            Some(BlockId::RustStone)
                                        } else if matches!(broken_block, BlockId::RustStoneLampLit) {
                                            Some(BlockId::RustStoneLamp)
                                        } else if matches!(broken_block, BlockId::LeverOn) {
                                            Some(BlockId::Lever)
                                        } else if matches!(broken_block, BlockId::ButtonPressed) {
                                            Some(BlockId::Button)
                                        } else if matches!(broken_block, BlockId::PressurePlatePressed) {
                                            Some(BlockId::PressurePlate)
                                        } else if matches!(broken_block, BlockId::Farmland | BlockId::FarmlandWet) {
                                            Some(BlockId::Dirt)
                                        } else if matches!(broken_block, BlockId::Door | BlockId::DoorOpen) {
                                            Some(BlockId::Door)
                                        } else if matches!(broken_block, BlockId::FurnaceLit) {
                                            Some(BlockId::Furnace)
                                        } else if broken_block == BlockId::Seagrass {
                                            let held_tool = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].as_ref().map(|s| s.block);
                                            if held_tool == Some(BlockId::Shears) {
                                                Some(BlockId::Seagrass)
                                            } else {
                                                None
                                            }
                                        } else if matches!(broken_block, BlockId::DiamondOre) {
                                            Some(BlockId::Diamond)
                                        } else if matches!(broken_block, BlockId::EmeraldOre) {
                                            Some(BlockId::Emerald)
                                        } else if broken_block == BlockId::Clay {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, BlockId::ClayBall, 4));
                                            None
                                        } else if broken_block == BlockId::Gravel {
                                            if seed % 10 == 0 {
                                                Some(BlockId::Flint)
                                            } else {
                                                Some(BlockId::Gravel)
                                            }
                                        } else if broken_block == BlockId::Stone {
                                            Some(BlockId::Cobblestone)
                                        } else if broken_block == BlockId::Obsidian {
                                            let held_tool = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].as_ref().map(|s| s.block);
                                            if held_tool == Some(BlockId::DiamondPickaxe) {
                                                Some(BlockId::Obsidian)
                                            } else {
                                                None
                                            }
                                        } else {
                                            Some(broken_block)
                                        };
                                        if let Some(drop_id) = drop_block {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, drop_id, 1));
                                        }
                                    }

                                    if let Some(chest) = chest_data {
                                        for item in chest.items.into_iter().flatten() {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, item.block, item.count));
                                        }
                                    }
                                    if let Some(furnace) = furnace_data {
                                        if let Some(st) = furnace.input {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, st.block, st.count));
                                        }
                                        if let Some(st) = furnace.fuel {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, st.block, st.count));
                                        }
                                        if let Some(st) = furnace.output {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, st.block, st.count));
                                        }
                                    }
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

                                if matches!(broken_block, BlockId::RustStone | BlockId::RustStoneWireLit | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::RustStoneLampLit | BlockId::Lever | BlockId::LeverOn | BlockId::Button | BlockId::ButtonPressed | BlockId::PressurePlate | BlockId::PressurePlatePressed) {
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
                                        self.particles.spawn_block_break(Vec3::new(lx - 0.5, ly - 0.5, lz - 0.5), leaf_color);

                                        if self.player.gamemode == GameMode::Survival {
                                            let l_seed = ((leaf_pos.x as u32).wrapping_mul(73856093)
                                                ^ (leaf_pos.y as u32).wrapping_mul(19349663)
                                                ^ (leaf_pos.z as u32).wrapping_mul(83492791))
                                                % 1000;
                                            if matches!(leaf_block, BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves) {
                                                if l_seed % 20 == 0 {
                                                    let l_rx = ((l_seed % 100) as f32 / 100.0 - 0.5) * 1.5;
                                                    let l_rz = (((l_seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.5;
                                                    let l_drop_vel = Vec3::new(l_rx, 2.0, l_rz);
                                                    self.items.push(ItemEntity::new(Vec3::new(lx, ly, lz), l_drop_vel, BlockId::Apple, 1));
                                                } else if l_seed % 10 == 1 {
                                                    let l_rx = ((l_seed % 100) as f32 / 100.0 - 0.5) * 1.5;
                                                    let l_rz = (((l_seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.5;
                                                    let l_drop_vel = Vec3::new(l_rx, 2.0, l_rz);
                                                    self.items.push(ItemEntity::new(Vec3::new(lx, ly, lz), l_drop_vel, BlockId::Sapling, 1));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        self.break_target = None;
                        self.break_progress = 0.0;
                        self.dig_sound_timer = 0.0;
                    }
                    }
                } else {
                    self.break_target = None;
                    self.break_progress = 0.0;
                    self.dig_sound_timer = 0.0;
                    self.creative_break_cooldown = 0.0;
                }

                // Creative pick block: middle-click on a block (or mob) gives it to the current / free hotbar slot
                let pick_block_just = !self.chat.is_open && (
                    self.controls.is_just_pressed(Action::PickBlock)
                    || (input.mouse_middle && !self.prev_middle_down)
                );
                if pick_block_just && self.player.gamemode == GameMode::Creative {
                    let forward = self.camera.forward();
                    let picked_item = self.mob_manager.raycast_mob(self.camera.position, forward, 4.2).and_then(|(mob_idx, _)| {
                        match self.mob_manager.mobs[mob_idx].kind {
                            crate::gameplay::mob::MobKind::Pig => Some(BlockId::PigSpawnEgg),
                            crate::gameplay::mob::MobKind::Cow => Some(BlockId::CowSpawnEgg),
                            crate::gameplay::mob::MobKind::Chicken => Some(BlockId::ChickenSpawnEgg),
                            crate::gameplay::mob::MobKind::Sheep => Some(BlockId::SheepSpawnEgg),
                            _ => None,
                        }
                    }).or_else(|| {
                        self.world.raycast(self.camera.position, forward, 8.0)
                            .and_then(|hit| self.world.get_block(hit.block_pos).pick_item())
                    });

                    if let Some(item_block) = picked_item {
                        let curr_slot = self.player.selected_hotbar_slot as usize;
                        if let Some(existing_slot) = self.player.inventory.hotbar.iter().position(|slot| slot.map(|s| s.block) == Some(item_block)) {
                            // Block is already in hotbar: switch to that slot and refresh stack
                            self.player.selected_hotbar_slot = existing_slot as u8;
                            if let Some(stack) = &mut self.player.inventory.hotbar[existing_slot] {
                                stack.count = 64;
                            }
                        } else if self.player.inventory.hotbar[curr_slot].is_none() {
                            // Current selected slot is free/empty: place it in the current slot
                            self.player.inventory.hotbar[curr_slot] = Some(ItemStack::new(item_block, 64));
                        } else if let Some(free_slot) = self.player.inventory.hotbar.iter().position(|slot| slot.is_none()) {
                            // Current slot is occupied, but another hotbar slot is free: place in free slot and select it
                            self.player.inventory.hotbar[free_slot] = Some(ItemStack::new(item_block, 64));
                            self.player.selected_hotbar_slot = free_slot as u8;
                        } else {
                            // All hotbar slots are full: replace current slot
                            self.player.inventory.hotbar[curr_slot] = Some(ItemStack::new(item_block, 64));
                        }

                        let new_selected_slot = self.player.selected_hotbar_slot as usize;
                        if let Some(stack) = self.player.inventory.hotbar[new_selected_slot] {
                            self.renderer.set_selected_block(stack.block.to_u8());
                        }
                        self.hand_animation = 0.5;
                        self.audio.play_sound(crate::audio::SoundId::ItemPop, 0.45, 1.2);
                        crate::log_info!("GAMEPLAY", "Creative pick block: {:?} -> slot {}", item_block, self.player.selected_hotbar_slot);
                    }
                }

                let held_item = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block);
                let offhand_item = self.player.inventory.offhand.map(|s| s.block);
                let has_shield = held_item.map_or(false, |b| b.is_shield()) || offhand_item.map_or(false, |b| b.is_shield());
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
                                let spawn_pos = self.camera.position + forward * 0.4 + Vec3::new(0.0, -0.08, 0.0);
                                let arrow = crate::gameplay::ArrowEntity::new(
                                    spawn_pos,
                                    forward * speed,
                                    true,
                                    damage,
                                );
                                self.arrows.push(arrow);
                                self.audio.play_sound(crate::audio::SoundId::BowShoot, 0.9, 0.9 + charge * 0.25);
                                self.hand_animation = 0.8;
                                crate::log_info!("COMBAT", "Player fired arrow with charge {:.2} (speed: {:.1}, dmg: {:.1})", charge, speed, damage);
                                if self.player.gamemode == GameMode::Survival {
                                    let selected = self.player.selected_hotbar_slot as usize;
                                    if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                                        if stack.block == BlockId::Bow {
                                            if stack.damage(1) {
                                                self.player.inventory.hotbar[selected] = None;
                                                self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                            }
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
                if has_shield && !has_bow {
                    if use_active {
                        self.player.is_blocking = true;
                        self.horiz_velocity.x *= 0.55;
                        self.horiz_velocity.z *= 0.55;
                    } else {
                        self.player.is_blocking = false;
                    }
                } else {
                    self.player.is_blocking = false;
                }

                let interact_right = !self.chat.is_open && !has_bow && !self.player.is_blocking && self.controls.is_just_pressed(Action::UsePlace);
                if interact_right {
                    let mut handled = false;

                    // Quick auto-equip armor / shield from hotbar on right-click
                    if let Some(item) = held_item {
                        if item.is_armor() || item.is_shield() {
                            let curr_slot = self.player.selected_hotbar_slot as usize;
                            if self.player.inventory.try_auto_equip_hotbar(curr_slot) {
                                self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                                self.hand_animation = 0.5;
                                handled = true;
                            }
                        }
                    }

                    // 0. Fishing Rod casting / reeling
                    if !handled && held_item == Some(BlockId::FishingRod) {
                        if let Some(bobber) = self.fishing_bobber.take() {
                            self.audio.play_sound(crate::audio::SoundId::FishingReel, 0.9, 1.0);
                            self.hand_animation = 1.0;
                            if self.player.gamemode == GameMode::Survival {
                                let selected = self.player.selected_hotbar_slot as usize;
                                if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                                    if stack.block == BlockId::FishingRod {
                                        if stack.damage(1) {
                                            self.player.inventory.hotbar[selected] = None;
                                            self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                        }
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
                                self.items.push(ItemEntity::new(bobber.pos + Vec3::new(0.0, 0.2, 0.0), pull_vel, caught_item, 1));
                                self.audio.play_sound(crate::audio::SoundId::FishCatch, 1.0, 1.0);
                                self.particles.spawn_block_break(bobber.pos, [0.2, 0.5, 0.9]);
                                crate::log_info!("GAMEPLAY", "Caught fish {:?} via fishing rod!", caught_item);
                            }
                            handled = true;
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
                            self.audio.play_sound(crate::audio::SoundId::FishingCast, 0.9, 1.0);
                            self.hand_animation = 1.0;
                            handled = true;
                        }
                    }

                    // 1. Villager trading & animal interaction (shearing, dyeing, feeding/breeding)
                    if !handled {
                        if let Some((mob_idx, _)) = self.mob_manager.raycast_mob(self.camera.position, self.camera.forward(), 4.2) {
                            let mob = &mut self.mob_manager.mobs[mob_idx];
                            if mob.kind == crate::gameplay::mob::MobKind::Villager && !mob.is_dead {
                                self.state = GameState::VillagerTrade { mob_id: mob.id, selected_trade: 0 };
                                out.request_release_mouse = true;
                                out.cursor_visible = Some(true);
                                self.horiz_velocity = Vec3::zero();
                                self.fly_velocity = Vec3::zero();
                                self.audio.play_sound(crate::audio::SoundId::VillagerIdle, 1.0, 1.0);
                                handled = true;
                            } else if let Some(item) = held_item {
                                if item == BlockId::Shears && mob.kind == crate::gameplay::mob::MobKind::Sheep && !mob.is_sheared && !mob.is_baby {
                                mob.is_sheared = true;
                                let wool_block = BlockId::wool_block_from_color(mob.wool_color);
                                let wool_count = 1 + (mob.id % 3) as u8; // 1-3 wool drops
                                let center = mob.center_pos();
                                let rx = ((mob.id as f32 * 11.0).sin()) * 1.2;
                                let rz = ((mob.id as f32 * 19.0).cos()) * 1.2;
                                self.items.push(ItemEntity::new(
                                    center,
                                    Vec3::new(rx, 3.2, rz),
                                    wool_block,
                                    wool_count,
                                ));
                                self.audio.play_sound(crate::audio::SoundId::SheepShear, 0.9, 1.0);
                                self.particles.spawn_growth(center);
                                self.hand_animation = 1.0;
                                if self.player.gamemode == GameMode::Survival {
                                    let selected = self.player.selected_hotbar_slot as usize;
                                    if let Some(stack) = &mut self.player.inventory.hotbar[selected] {
                                        if stack.block == BlockId::Shears {
                                            if stack.damage(1) {
                                                self.player.inventory.hotbar[selected] = None;
                                                self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                            }
                                        }
                                    }
                                }
                                handled = true;
                            } else if item.is_wool() && mob.kind == crate::gameplay::mob::MobKind::Sheep && !mob.is_baby {
                                let dye_color = item.wool_color_index().unwrap();
                                if mob.wool_color != dye_color || mob.is_sheared {
                                    mob.wool_color = dye_color;
                                    mob.is_sheared = false;
                                    let sound_pos = mob.center_pos();
                                    self.audio.play_sound(crate::audio::SoundId::DigGrass, 0.75, 1.2);
                                    self.particles.spawn_growth(sound_pos);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 {
                                                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                            }
                                        }
                                    }
                                    self.hand_animation = 1.0;
                                    handled = true;
                                }
                            } else if mob.kind.is_tempting_item(item) {
                                let center = mob.center_pos();
                                if mob.is_baby {
                                    mob.age += 30.0;
                                    self.particles.spawn_growth(center);
                                    self.audio.play_sound(crate::audio::SoundId::PlayerEat, 0.7, 1.3);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 {
                                                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                            }
                                        }
                                    }
                                    self.hand_animation = 1.0;
                                    handled = true;
                                } else if mob.breed_cooldown <= 0.0 && mob.in_love <= 0.0 {
                                    mob.in_love = 30.0;
                                    self.particles.spawn_hearts(center);
                                    self.audio.play_sound(crate::audio::SoundId::PlayerEat, 0.8, 1.2);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 {
                                                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                            }
                                        }
                                    }
                                    self.hand_animation = 1.0;
                                    handled = true;
                                }
                            }
                        }
                    }
                    }

                    // 2. Farming tool and block interactions (Hoe, BoneMeal, Planting)
                    if !handled {
                        if let Some(hit) = self.world.raycast(self.camera.position, self.camera.forward(), 8.0) {
                            let targeted_block = self.world.get_block(hit.block_pos);
                            if let Some(item) = held_item {
                                if item.is_hoe() && matches!(targeted_block, BlockId::Grass | BlockId::Dirt) && hit.normal.y > 0 {
                                    self.world.set_block(hit.block_pos, BlockId::Farmland);
                                    let sound_pos = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                                    self.play_world_sound_varied(crate::audio::SoundId::DigDirt, sound_pos, 0.85);
                                    self.hand_animation = 1.0;
                                    handled = true;
                                } else if item == BlockId::BoneMeal {
                                    if targeted_block == BlockId::Sapling {
                                        self.world.grow_tree_at(hit.block_pos);
                                        let center = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                                        self.particles.spawn_growth(center);
                                        self.play_world_sound_varied(crate::audio::SoundId::DigGrass, center, 0.85);
                                        if self.player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 {
                                                    self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                                }
                                            }
                                        }
                                        self.hand_animation = 1.0;
                                        handled = true;
                                    } else if matches!(targeted_block, BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop) {
                                        let center = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                                        self.particles.spawn_growth(center);
                                        let drop_vel = Vec3::new(0.0, 2.5, 0.0);
                                        match targeted_block {
                                            BlockId::WheatCrop => {
                                                self.items.push(ItemEntity::new(center, drop_vel, BlockId::Wheat, 1));
                                                self.items.push(ItemEntity::new(center, drop_vel, BlockId::WheatSeeds, 2));
                                            }
                                            BlockId::CarrotCrop => {
                                                self.items.push(ItemEntity::new(center, drop_vel, BlockId::Carrot, 2));
                                            }
                                            BlockId::PotatoCrop => {
                                                self.items.push(ItemEntity::new(center, drop_vel, BlockId::Potato, 2));
                                            }
                                            _ => {}
                                        }
                                        self.world.set_block(hit.block_pos, BlockId::Air);
                                        if self.player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 {
                                                    self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                                }
                                            }
                                        }
                                        self.play_world_sound_varied(crate::audio::SoundId::DigGrass, center, 0.85);
                                        self.hand_animation = 1.0;
                                        handled = true;
                                    }
                                } else if matches!(item, BlockId::WheatSeeds | BlockId::Carrot | BlockId::Potato) {
                                    if matches!(targeted_block, BlockId::Farmland | BlockId::FarmlandWet) && hit.normal.y > 0 {
                                        let plant_pos = hit.block_pos + hit.normal;
                                        if self.world.get_block(plant_pos) == BlockId::Air {
                                            let crop = match item {
                                                BlockId::WheatSeeds => BlockId::WheatCrop,
                                                BlockId::Carrot => BlockId::CarrotCrop,
                                                BlockId::Potato => BlockId::PotatoCrop,
                                                _ => unreachable!(),
                                            };
                                            self.world.set_block(plant_pos, crop);
                                            let sound_pos = Vec3::new(plant_pos.x as f32 + 0.5, plant_pos.y as f32 + 0.5, plant_pos.z as f32 + 0.5);
                                            self.play_world_sound_varied(crate::audio::SoundId::DigGrass, sound_pos, 0.85);
                                            if self.player.gamemode == GameMode::Survival {
                                                if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                    stack.count -= 1;
                                                    if stack.count == 0 {
                                                        self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                                    }
                                                }
                                            }
                                            self.hand_animation = 1.0;
                                            handled = true;
                                        }
                                    }
                                } else if item == BlockId::Sapling {
                                    if matches!(targeted_block, BlockId::Grass | BlockId::Dirt) && hit.normal.y > 0 {
                                        let plant_pos = hit.block_pos + hit.normal;
                                        if self.world.get_block(plant_pos) == BlockId::Air {
                                            self.world.set_block(plant_pos, BlockId::Sapling);
                                            let sound_pos = Vec3::new(plant_pos.x as f32 + 0.5, plant_pos.y as f32 + 0.5, plant_pos.z as f32 + 0.5);
                                            self.play_world_sound_varied(crate::audio::SoundId::DigGrass, sound_pos, 0.85);
                                            if self.player.gamemode == GameMode::Survival {
                                                if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                    stack.count -= 1;
                                                    if stack.count == 0 {
                                                        self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                                    }
                                                }
                                            }
                                            self.hand_animation = 1.0;
                                            handled = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 3. Food consumption (if player can eat held food)
                    if !handled {
                        if let Some(food_id) = held_item {
                            if self.player.can_eat(food_id) {
                                if self.player.eat(food_id) {
                                    handled = true;
                                    self.audio.play_sound(crate::audio::SoundId::PlayerEat, 0.8, 1.0);
                                    self.hand_animation = 1.0;
                                    let color = food_id.face_color(Face::Top);
                                    let mouth_pos = self.camera.position + self.camera.forward() * 0.4 - Vec3::new(0.0, 0.2, 0.0);
                                    self.particles.spawn_block_break(mouth_pos - Vec3::new(0.5, 0.5, 0.5), color);
                                    if food_id == BlockId::Pufferfish {
                                        self.player.take_damage(3.0);
                                        self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.8, 1.0);
                                        self.damage_tilt_timer = 0.25;
                                        crate::log_info!("PLAYER", "Player pricked/poisoned by eating Pufferfish!");
                                    }
                                    crate::log_info!("PLAYER", "Player consumed {:?} (health: {:.1}/{:.1}, hunger: {:.1}/{:.1})", food_id, self.player.health, self.player.max_health, self.player.hunger, self.player.max_hunger);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 {
                                                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !handled {
                        if let Some(hit) = self.world.raycast(self.camera.position, self.camera.forward(), 8.0) {
                        let targeted_block = self.world.get_block(hit.block_pos);
                        if matches!(targeted_block, BlockId::Door | BlockId::DoorOpen) && !is_crouching {
                            let new_block = if targeted_block == BlockId::Door { BlockId::DoorOpen } else { BlockId::Door };
                            self.world.set_block(hit.block_pos, new_block);
                            let door_sound = if new_block == BlockId::DoorOpen { crate::audio::SoundId::DoorOpen } else { crate::audio::SoundId::DoorClose };
                            let sound_pos = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                            self.play_world_sound_varied(door_sound, sound_pos, 0.85);
                            let above = hit.block_pos + IVec3::new(0, 1, 0);
                            let below = hit.block_pos - IVec3::new(0, 1, 0);
                            if matches!(self.world.get_block(above), BlockId::Door | BlockId::DoorOpen) {
                                self.world.set_block(above, new_block);
                            } else if matches!(self.world.get_block(below), BlockId::Door | BlockId::DoorOpen) {
                                self.world.set_block(below, new_block);
                            }
                            self.hand_animation = 1.0;
                            crate::log_info!("WORLD", "Toggled door at ({}, {}, {}) to {:?}", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z, new_block);
                        } else if matches!(targeted_block, BlockId::Lever | BlockId::LeverOn) && !is_crouching {
                            let new_block = if targeted_block == BlockId::Lever { BlockId::LeverOn } else { BlockId::Lever };
                            self.world.set_block(hit.block_pos, new_block);
                            let sound_pos = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                            self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.85);
                            self.hand_animation = 1.0;
                            self.world.update_ruststone_network(hit.block_pos, 16);
                            crate::log_info!("WORLD", "Toggled lever at ({}, {}, {}) to {:?}", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z, new_block);
                        } else if targeted_block == BlockId::Button && !is_crouching {
                            self.world.set_block(hit.block_pos, BlockId::ButtonPressed);
                            let sound_pos = Vec3::new(hit.block_pos.x as f32 + 0.5, hit.block_pos.y as f32 + 0.5, hit.block_pos.z as f32 + 0.5);
                            self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.9);
                            self.world.button_timers.insert(hit.block_pos, 1.0);
                            self.hand_animation = 1.0;
                            self.world.update_ruststone_network(hit.block_pos, 16);
                            crate::log_info!("WORLD", "Pressed button at ({}, {}, {})", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
                        } else if targeted_block == BlockId::CraftingTable && !is_crouching {
                            self.crafting_table_open = true;
                            self.recipe_book_open = false;
                            self.recipe_book_scroll = 0;
                            self.recipe_book_status = None;
                            if self.player.gamemode != crate::gameplay::player::GameMode::Creative {
                                self.record_inventory_items_collected();
                            }
                            self.state = GameState::Inventory;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            out.request_release_mouse = true;
                            self.horiz_velocity = Vec3::zero();
                            self.fly_velocity = Vec3::zero();
                            crate::log_info!("MENU", "Opened crafting table at ({}, {}, {})", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
                        } else if targeted_block == BlockId::Chest && !is_crouching {
                            let _ = self.world.get_or_create_chest(hit.block_pos);
                            self.state = GameState::Chest { pos: hit.block_pos };
                            self.audio.play_sound(crate::audio::SoundId::ChestOpen, 0.8, 1.0);
                            out.request_release_mouse = true;
                            self.horiz_velocity = Vec3::zero();
                            self.fly_velocity = Vec3::zero();
                            crate::log_info!("MENU", "Opened chest container at ({}, {}, {})", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
                        } else if (targeted_block == BlockId::Furnace || targeted_block == BlockId::FurnaceLit) && !is_crouching {
                            let _ = self.world.get_or_create_furnace(hit.block_pos);
                            self.state = GameState::Furnace { pos: hit.block_pos };
                            out.request_release_mouse = true;
                            self.horiz_velocity = Vec3::zero();
                            self.fly_velocity = Vec3::zero();
                            crate::log_info!("MENU", "Opened furnace container at ({}, {}, {})", hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
                        } else if targeted_block == BlockId::Bed && !is_crouching {
                            let spawn_pos = self.world.bed_pair_at(hit.block_pos)
                                .map(|(foot, _)| foot)
                                .unwrap_or(hit.block_pos);
                            self.player.spawn_point = Some(spawn_pos);
                            let is_night = self.world.time_of_day > 260.0 || self.world.time_of_day < 90.0;
                            let is_storm_or_rain = self.world.weather.is_raining();
                            if is_night || is_storm_or_rain {
                                self.world.time_of_day = 120.0;
                                self.world.weather.clear();
                                self.player.health = (self.player.health + 4.0).min(self.player.max_health);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.0);
                                if is_storm_or_rain && !is_night {
                                    self.chat.add_message("Przespano burze. Pogoda sie rozpogodzila!".to_string(), crate::ui::pack_color(0.9, 0.9, 0.3));
                                } else {
                                    self.chat.add_message("Przespano noc. Punkt odrodzenia ustawiony!".to_string(), crate::ui::pack_color(0.9, 0.9, 0.3));
                                }
                                crate::log_info!("PLAYER", "Player slept in bed at ({}, {}, {}). Time set to morning, weather cleared.", spawn_pos.x, spawn_pos.y, spawn_pos.z);
                            } else {
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                                self.chat.add_message("Punkt odrodzenia ustawiony! Mozesz spac tylko w nocy lub podczas burzy.".to_string(), crate::ui::pack_color(0.9, 0.9, 0.3));
                                crate::log_info!("PLAYER", "Spawn point set to bed at ({}, {}, {}). Day time, cannot sleep.", spawn_pos.x, spawn_pos.y, spawn_pos.z);
                            }
                        } else if let Some(mob_kind) = BlockId::from_u8(selected_block_id).spawn_mob_kind() {
                            let spawn_pos = Vec3::new(
                                hit.block_pos.x as f32 + 0.5 + hit.normal.x as f32 * 0.5,
                                hit.block_pos.y as f32 + (if hit.normal.y > 0 { 1.0 } else { 0.0 }),
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
                            crate::log_info!("WORLD", "Spawned mob {:?} via egg at ({:.1}, {:.1}, {:.1})", mob_kind, spawn_pos.x, spawn_pos.y, spawn_pos.z);
                            if self.player.gamemode == GameMode::Survival {
                                if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                    stack.count -= 1;
                                    if stack.count == 0 {
                                        self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                    }
                                }
                            }
                        } else {
                            let place_pos = hit.block_pos + hit.normal;
                            let place_id = BlockId::from_u8(selected_block_id);
                            let existing = self.world.get_block(place_pos);
                            if place_id != BlockId::Air
                                && place_id.is_placeable()
                                && can_place_block_in(existing)
                            {
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
                                        && !block_intersects_player(place_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && !block_intersects_player(head_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
                                        && !block_intersects_live_mob(head_pos, &self.mob_manager.mobs)
                                    {
                                        self.world.set_block(place_pos, BlockId::Bed);
                                        self.world.set_block(head_pos, BlockId::Bed);
                                        let unlocked = self.achievements.record_placed();
                                        self.process_achievement_unlocks(unlocked);
                                        let sound_pos = Vec3::new(place_pos.x as f32 + 0.5, place_pos.y as f32 + 0.5, place_pos.z as f32 + 0.5);
                                        self.play_world_sound_varied(crate::audio::SoundId::PlaceWood, sound_pos, 0.85);
                                        self.request_rumble(0.2, 0.1, 45);
                                        if self.player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 { self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None; }
                                            }
                                        }
                                    }
                                } else if place_id == BlockId::Door {
                                    let top_pos = place_pos + IVec3::new(0, 1, 0);
                                    if !block_intersects_player(place_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && !block_intersects_player(top_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
                                        && !block_intersects_live_mob(top_pos, &self.mob_manager.mobs)
                                        && self.world.get_block(top_pos) == BlockId::Air
                                    {
                                        self.world.set_block(place_pos, BlockId::Door);
                                        self.world.set_block(top_pos, BlockId::Door);
                                        let unlocked = self.achievements.record_placed();
                                        self.process_achievement_unlocks(unlocked);
                                        let sound_pos = Vec3::new(place_pos.x as f32 + 0.5, place_pos.y as f32 + 0.5, place_pos.z as f32 + 0.5);
                                        self.play_world_sound_varied(crate::audio::SoundId::PlaceWood, sound_pos, 0.85);
                                        self.request_rumble(0.2, 0.1, 45);
                                        crate::log_info!("WORLD", "Placed door at ({}, {}, {})", place_pos.x, place_pos.y, place_pos.z);
                                        if self.player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 { self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None; }
                                            }
                                        }
                                    }
                                } else if !block_intersects_player(place_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                    && !block_intersects_live_mob(place_pos, &self.mob_manager.mobs)
                                {
                                    self.world.set_block(place_pos, place_id);
                                    let unlocked = self.achievements.record_placed();
                                    self.process_achievement_unlocks(unlocked);
                                    let place_sound = crate::audio::SoundId::from_block_place(place_id);
                                    let sound_pos = Vec3::new(place_pos.x as f32 + 0.5, place_pos.y as f32 + 0.5, place_pos.z as f32 + 0.5);
                                    self.play_world_sound_varied(place_sound, sound_pos, 0.85);
                                    self.request_rumble(0.2, 0.1, 45);
                                    if matches!(place_id, BlockId::Tnt | BlockId::RustStone | BlockId::RustStoneBlock | BlockId::RustStoneLamp | BlockId::Lever | BlockId::Button | BlockId::PressurePlate) {
                                        self.world.update_ruststone_network(place_pos, 16);
                                    }
                                    crate::log_info!("WORLD", "Placed block {:?} at ({}, {}, {})", place_id, place_pos.x, place_pos.y, place_pos.z);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 { self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

                self.profiler_frame_count = self.profiler_frame_count.wrapping_add(1);
                #[cfg(target_os = "vita")]
                let should_sample_detail = self.profiler_frame_count % 3 == 0;
                #[cfg(not(target_os = "vita"))]
                let should_sample_detail = true;

                let collect_profiling = ((self.profiler_enabled && self.settings.debug_overlay) || self.profiler_server.is_some()) && should_sample_detail;
                let frame_start_time = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                let mut cur_profiler = FrameProfiler::default();

                let tick_start_time = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                let world_time_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                self.world.advance_time(dt);
                let world_update_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
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
                    self.world.simulate_materials(self.camera.position);
                    self.world.tick_farming(self.camera.position, &mut self.mob_manager.rng_state);
                }

                self.world.update_primed_tnts(dt);
                let mob_positions: Vec<Vec3> = self.mob_manager.mobs.iter().filter(|m| !m.is_dead).map(|m| m.pos).collect();
                self.world.update_ruststone(dt, self.camera.position, &mob_positions);
                let sound_events = std::mem::take(&mut self.world.ruststone_sound_events);
                for (s_pos, _is_on) in sound_events {
                    let sound_pos = Vec3::new(s_pos.x as f32 + 0.5, s_pos.y as f32 + 0.5, s_pos.z as f32 + 0.5);
                    self.play_world_sound_varied(crate::audio::SoundId::UiClick, sound_pos, 0.7);
                }
                if let Some(w_start) = world_time_start {
                    cur_profiler.tick_world_ms = w_start.elapsed().saturating_sub(world_io_dur).as_secs_f32() * 1000.0;
                }

                let mobs_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                let is_creative = self.player.gamemode == GameMode::Creative;
                let simulates_mobs = self.net_client.as_ref().map_or(true, |client| {
                    !client.is_connected || client.local_id == client.mob_authority_id
                });
                if simulates_mobs {
                    self.mob_manager.update_for_players(dt, &self.world, &mob_players);
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
                for mob in &mut self.mob_manager.mobs {
                    if !mob.is_dead {
                        let dx = self.camera.position.x - mob.pos.x;
                        let dz = self.camera.position.z - mob.pos.z;
                        let dist_sq = dx * dx + dz * dz;

                        // Soft push separation between player and mob
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

                        // Hostile mob attack (reapers do not melee, they explode; ignored in creative mode)
                        let is_angry_golem = mob.kind == crate::gameplay::mob::MobKind::IronGolem
                            && mob.player_anger_timer > 0.0
                            && mob.attack_cooldown <= 0.0;
                        let is_hostile_melee = mob.kind.is_hostile()
                            && mob.kind != crate::gameplay::mob::MobKind::Reaper;
                        let attack_range = if is_angry_golem { 2.2 } else { 1.4 };
                        if !is_creative
                            && (is_hostile_melee || is_angry_golem)
                            && dist_sq < attack_range * attack_range
                            && (mob.pos.y - self.camera.position.y).abs() < 1.8
                        {
                            if is_angry_golem {
                                mob.attack_cooldown = 1.0;
                                self.audio.play_sound(crate::audio::SoundId::IronGolemAttack, 1.0, 1.0);
                            }
                            let to_mob = (mob.pos - self.camera.position).normalized();
                            let is_front = to_mob.dot(self.camera.forward()) > 0.35;
                            let damage = if is_angry_golem { 7.0 } else { 2.0 };
                            let (took_damage, blocked, item_broke) = self.player.take_physical_damage(damage, is_front);
                            if item_broke {
                                self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                            }
                            if blocked {
                                self.audio.play_sound(crate::audio::SoundId::ShieldBlock, 1.0, 1.0);
                                mob.velocity.x -= to_mob.x * 6.0;
                                mob.velocity.z -= to_mob.z * 6.0;
                                mob.velocity.y += 2.0;
                                self.damage_tilt_timer = 0.15;
                                crate::log_info!("COMBAT", "Player blocked attack from {:?} with shield!", mob.kind);
                            } else if took_damage {
                                if is_angry_golem {
                                    self.horiz_velocity.x -= to_mob.x * 5.0;
                                    self.horiz_velocity.z -= to_mob.z * 5.0;
                                    self.vertical_velocity = self.vertical_velocity.max(3.0);
                                }
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = if mob.pos.x > self.camera.position.x { -1.0 } else { 1.0 };
                                if self.settings.vibration {
                                    self.pending_rumble = Some(RumbleRequest { strong: 0.85, weak: 0.6, duration_ms: 220 });
                                }
                                self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                                crate::log_info!("PLAYER", "Player took damage from {:?} (health: {:.1}/{:.1})", mob.kind, self.player.health, self.player.max_health);
                            }
                        }
                    }
                }
                for exp_pos in self.mob_manager.pending_explosions.drain(..) {
                    self.world.explode(exp_pos.floor_to_ivec(), 3.5);
                }
                for poof_pos in self.mob_manager.dying_poofs.drain(..) {
                    self.particles.spawn_mob_death_poof(poof_pos);
                }
                for drop in self.mob_manager.dying_drops.drain(..) {
                    crate::log_info!("COMBAT", "Mob killed, dropped {}x {:?}", drop.count, drop.block);
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

                let items_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };

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

                if !self.chat.is_open && self.controls.is_just_pressed(Action::DropItem) {
                    if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                        crate::log_info!("INVENTORY", "Player dropped 1x {:?}", stack.block);
                        self.items.push(ItemEntity::new(
                            self.camera.position + self.camera.forward() * 0.5,
                            self.camera.forward() * 5.0,
                            stack.block,
                            1,
                        ));
                        stack.count -= 1;
                        if stack.count == 0 {
                            self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                        }
                    }
                }

                let player_center = self.camera.position - Vec3::new(0.0, 0.8, 0.0);
                let mut i = 0;
                while i < self.items.len() {
                    self.items[i].update(dt, &self.world, Some(player_center));

                    // Lava burns dropped items
                    let item_pos = self.items[i].position;
                    let item_block = self.world.get_block(item_pos.floor_to_ivec());
                    if item_block == BlockId::Lava {
                        self.particles.spawn_lava_burn(item_pos);
                        self.items.remove(i);
                        continue;
                    }

                    let dist_sq = (self.items[i].position - player_center).length_sq();
                    if self.items[i].pickup_delay <= 0.0 && dist_sq < 3.24 {
                        let remaining = self.player.inventory.add_item(self.items[i].block, self.items[i].count);
                        let picked_up = self.items[i].count - remaining;
                        if picked_up > 0 {
                            self.audio.play_sound(crate::audio::SoundId::ItemPop, 0.7, 1.0);
                            crate::log_info!("INVENTORY", "Player picked up {}x {:?}", picked_up, self.items[i].block);
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

                // Drain any skeleton-fired arrows from mob manager
                self.arrows.extend(self.mob_manager.pending_arrows.drain(..));

                // Update physical arrows (ballistics, block collision, entity collision)
                let mut a_idx = 0;
                while a_idx < self.arrows.len() {
                    let struck_block = self.arrows[a_idx].update(dt, &self.world);
                    if struck_block {
                        self.audio.play_sound(crate::audio::SoundId::BowHit, 0.85, 1.0);
                    }

                    let arrow = &self.arrows[a_idx];
                    let mut remove_arrow = false;

                    // Despawn old arrows
                    if arrow.lifetime > 60.0 {
                        self.arrows.remove(a_idx);
                        continue;
                    }

                    if !arrow.in_ground {
                        if arrow.shooter_is_player {
                            // Check collision with mobs
                            let mut hit_mob = None;
                            for (midx, mob) in self.mob_manager.mobs.iter().enumerate() {
                                if mob.is_dead {
                                    continue;
                                }
                                let mob_center = mob.pos + Vec3::new(0.0, mob.kind.height() * 0.5, 0.0);
                                let d = (arrow.position - mob_center).length();
                                if d < 0.6 + mob.kind.half_width() {
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
                                self.audio.play_sound(crate::audio::SoundId::BowHit, 1.0, 1.0);
                                self.particles.spawn_mob_hit(arrow.position);
                                remove_arrow = true;
                            }
                        } else if !is_creative {
                            // Skeleton arrow colliding with player
                            let player_bb_min = self.camera.position - Vec3::new(0.4, 1.6, 0.4);
                            let player_bb_max = self.camera.position + Vec3::new(0.4, 0.2, 0.4);
                            if arrow.position.x >= player_bb_min.x && arrow.position.x <= player_bb_max.x &&
                               arrow.position.y >= player_bb_min.y && arrow.position.y <= player_bb_max.y &&
                               arrow.position.z >= player_bb_min.z && arrow.position.z <= player_bb_max.z {
                                let to_arrow = (arrow.position - self.camera.position).normalized();
                                let is_front = to_arrow.dot(self.camera.forward()) > 0.35;
                                let (took_damage, blocked, item_broke) = self.player.take_physical_damage(arrow.damage, is_front);
                                if item_broke {
                                    self.audio.play_sound(crate::audio::SoundId::ToolBreak, 1.0, 1.0);
                                }
                                if blocked {
                                    self.audio.play_sound(crate::audio::SoundId::ShieldBlock, 1.0, 1.0);
                                    self.damage_tilt_timer = 0.12;
                                    crate::log_info!("COMBAT", "Player blocked skeleton arrow with shield!");
                                } else if took_damage {
                                    self.damage_tilt_timer = 0.4;
                                    self.damage_tilt_dir = if arrow.position.x > self.camera.position.x { -1.0 } else { 1.0 };
                                    if self.settings.vibration {
                                        self.pending_rumble = Some(RumbleRequest { strong: 0.8, weak: 0.5, duration_ms: 200 });
                                    }
                                    self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                                    crate::log_info!("PLAYER", "Player struck by arrow for {:.1} damage (health: {:.1}/{:.1})", arrow.damage, self.player.health, self.player.max_health);
                                }
                                self.audio.play_sound(crate::audio::SoundId::BowHit, 1.0, 1.0);
                                remove_arrow = true;
                            }
                        }
                    } else if !is_creative {
                        // Arrow stuck in ground can be picked up by player
                        let dist_sq = (arrow.position - player_center).length_sq();
                        if dist_sq < 2.5 {
                            if self.player.inventory.add_item(BlockId::Arrow, 1) == 0 {
                                self.audio.play_sound(crate::audio::SoundId::ItemPop, 0.7, 1.0);
                                remove_arrow = true;
                            }
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
                    let held_rod = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block) == Some(BlockId::FishingRod);
                    let dist_sq = (bobber.pos - self.camera.position).length_sq();
                    if !held_rod || dist_sq > 1024.0 {
                        self.fishing_bobber = None;
                    } else {
                        let block_at_bobber = self.world.get_block(bobber.pos.floor_to_ivec());
                        if matches!(block_at_bobber, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
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
                                    self.audio.play_sound(crate::audio::SoundId::FishFlop, 0.65, 1.2);
                                    self.particles.spawn_block_break(bobber.pos, [0.3, 0.6, 1.0]);
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
                            let ground_block = self.world.get_block(bobber.pos.floor_to_ivec());
                            if ground_block.is_solid() {
                                bobber.vel = Vec3::zero();
                            }
                        }
                    }
                }

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
                            self.damage_tilt_dir = if exp_pos.x > self.camera.position.x { -1.0 } else { 1.0 };
                            self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 0.9, 1.0);
                            self.request_rumble(1.0, 0.9, 450);
                            crate::log_info!("PLAYER", "Player took {:.1} explosion damage (health: {:.1}/{:.1})", dmg, self.player.health, self.player.max_health);
                        }
                    }
                }

                self.world.update_furnaces(dt);
                self.particles.update(dt);

                // Update weather simulation (rain, thunder, lightning)
                if let Some(strike) = self.world.update_weather(dt, self.camera.position) {
                    let strike_pos = strike.pos;
                    let dist = (strike_pos - self.camera.position).length();
                    // Audio: Thunder crack / rumble
                    if dist < 45.0 {
                        self.audio.play_sound(crate::audio::SoundId::ThunderClose, 1.0, 1.0);
                        self.request_rumble(0.9, 1.0, 600);
                    } else {
                        self.audio.play_sound(crate::audio::SoundId::ThunderDistant, 0.85, 0.95);
                    }
                    // Lightning blast damage to nearby mobs
                    for mob in &mut self.mob_manager.mobs {
                        let d = (mob.pos - strike_pos).length();
                        if d < 4.0 {
                            let kb = (mob.pos - strike_pos).normalized();
                            mob.take_damage(15.0, kb);
                            self.particles.spawn_mob_hit(mob.pos);
                        }
                    }
                    // Blast damage to player if caught in strike
                    if dist < 4.0 {
                        let dmg = (1.0 - dist / 4.0) * 15.0;
                        if self.player.take_damage(dmg) {
                            self.damage_tilt_timer = 0.4;
                            self.damage_tilt_dir = 1.0;
                            self.audio.play_sound(crate::audio::SoundId::PlayerHurt, 1.0, 1.0);
                            self.request_rumble(1.0, 1.0, 500);
                        }
                    }
                    self.particles.spawn_explosion(strike_pos);
                    crate::log_info!("WEATHER", "Lightning struck at ({:.1}, {:.1}, {:.1}), distance: {:.1}m", strike_pos.x, strike_pos.y, strike_pos.z, dist);
                }
                if let Some(i_start) = items_start {
                    cur_profiler.tick_entities_ms = i_start.elapsed().as_secs_f32() * 1000.0;
                }
                if let Some(t_start) = tick_start_time {
                    cur_profiler.tick_ms = t_start.elapsed().saturating_sub(world_io_dur).as_secs_f32() * 1000.0;
                    cur_profiler.world_io_ms = world_io_dur.as_secs_f32() * 1000.0;
                    cur_profiler.world_mesh_ms = cur_profiler.world_io_ms * 0.72;
                    cur_profiler.world_gen_ms = cur_profiler.world_io_ms * 0.28;
                    cur_profiler.tick_physics_ms = (cur_profiler.tick_ms - cur_profiler.tick_mobs_ms - cur_profiler.tick_entities_ms - cur_profiler.tick_world_ms).max(0.0);
                }

                let aspect = self.width as f32 / self.height as f32;

                if (self.camera.fov_y - self.last_fov).abs() > 0.0001 || (aspect - self.last_aspect).abs() > 0.0001 {
                    self.last_fov = self.camera.fov_y;
                    self.last_aspect = aspect;
                    self.proj_matrix = self.camera.projection_matrix(aspect);
                }
                let (shake_offset, roll) = if self.settings.damage_tilt && self.damage_tilt_timer > 0.0 {
                    let progress = self.damage_tilt_timer / 0.4;
                    let roll = (progress * std::f32::consts::PI).sin() * 0.14 * self.damage_tilt_dir;
                    let sx = (self.total_time * 50.0).sin() * 0.08 * progress;
                    let sy = (self.total_time * 60.0).cos() * 0.05 * progress;
                    (self.camera.right_flat() * sx + Vec3::new(0.0, sy, 0.0), roll)
                } else {
                    (Vec3::zero(), 0.0)
                };
                let mut effective_camera = match self.camera_perspective {
                    CameraPerspective::FirstPerson => self.camera.clone(),
                    CameraPerspective::ThirdPersonBack => {
                        let eye_pos = self.camera.position;
                        let forward = self.camera.forward();
                        let max_dist = 3.8;
                        let actual_dist = self.world.raycast_camera_collision(eye_pos, -forward, max_dist);
                        let mut cam = self.camera.clone();
                        cam.position = eye_pos - forward * actual_dist;
                        cam
                    }
                    CameraPerspective::ThirdPersonFront => {
                        let eye_pos = self.camera.position;
                        let forward = self.camera.forward();
                        let max_dist = 3.8;
                        let actual_dist = self.world.raycast_camera_collision(eye_pos, forward, max_dist);
                        let mut cam = self.camera.clone();
                        cam.position = eye_pos + forward * actual_dist;
                        cam.yaw = self.camera.yaw + std::f32::consts::PI;
                        cam.pitch = -self.camera.pitch;
                        cam
                    }
                };
                effective_camera.position += shake_offset;
                effective_camera.roll += roll;
                #[cfg(not(target_os = "vita"))]
                let view_proj = self.proj_matrix.mul(&effective_camera.view_matrix());
                #[cfg(not(target_os = "vita"))]
                {
                    let render_terrain_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                    self.renderer.fog_enabled = self.settings.fog;
                    self.renderer.render_frame(&mut self.world, &effective_camera);
                    if let Some(t_start) = render_terrain_start {
                        cur_profiler.render_terrain_ms = t_start.elapsed().as_secs_f32() * 1000.0;
                    }

                    let render_entities_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                    let is_occluded = self.world.is_view_fully_occluded(effective_camera.position, effective_camera.yaw, effective_camera.pitch);

                    if !is_occluded {
                        // In 3rd person perspective, render local player model
                        if self.camera_perspective != CameraPerspective::FirstPerson {
                            let feet_pos = Vec3::new(
                                self.camera.position.x,
                                self.camera.position.y - self.current_eye_offset,
                                self.camera.position.z,
                            );
                            let held = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block);
                            let swing = self.break_progress.max(self.hand_animation);
                            let hurt_tint = if self.damage_tilt_timer > 0.0 { 0.65 } else { 0.0 };
                            self.renderer.draw_player_model(
                                feet_pos,
                                self.camera.yaw,
                                self.camera.pitch,
                                self.walk_phase,
                                swing,
                                self.crouch_toggled,
                                hurt_tint,
                                None,
                                held,
                                &self.player.inventory.armor,
                                &effective_camera,
                                &view_proj,
                            );
                        }

                        for mob in &self.mob_manager.mobs {
                            self.renderer.draw_mob(mob, &effective_camera, &view_proj);
                        }

                        for tnt in &self.world.primed_tnts {
                            self.renderer.draw_tnt_cube(tnt.pos, tnt.fuse, &effective_camera, &view_proj);
                        }

                        if self.settings.clouds {
                            let clouds_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                            self.renderer.draw_clouds(
                                effective_camera.position,
                                &view_proj,
                                self.total_time,
                                self.world.weather.rain_strength,
                                self.world.weather.thunder_strength,
                            );
                            if let Some(c_start) = clouds_start {
                                cur_profiler.render_clouds_ms = c_start.elapsed().as_secs_f32() * 1000.0;
                            }
                        }

                        for item in &self.items {
                            // Ground drop shadow below floating block (Minecraft style)
                            let ipos_x = item.position.x.floor() as i32;
                            let ipos_z = item.position.z.floor() as i32;
                            let item_y = item.position.y;
                            for dy in 0..=3 {
                                let test_y = (item_y.floor() as i32) - dy;
                                let b = self.world.get_block(crate::core::math::IVec3::new(ipos_x, test_y, ipos_z));
                                if b.is_solid() {
                                    let top_y = test_y as f32 + 1.0 + 0.015;
                                    if item_y >= top_y - 0.2 {
                                        self.renderer.draw_item_shadow(item.position, top_y, &view_proj);
                                        break;
                                    }
                                }
                            }

                            let y_offset = (item.age * 3.0).sin() * 0.08;
                            let mut pos = item.position;
                            pos.y += y_offset;
                            let yaw = item.age * 2.5;
                            self.renderer.draw_item_3d(item.block, pos, yaw, &view_proj);
                        }

                        if let Some(target) = self.break_target {
                            if self.break_progress > 0.0 {
                                self.renderer.draw_block_break_cracks(target, self.break_progress, &view_proj);
                            }
                        }

                        if self.settings.particles {
                            for p in &self.particles.particles {
                                self.renderer.draw_billboard(p.pos, p.size, p.color, &effective_camera, &view_proj);
                            }
                        }

                        if let Some(bobber) = &self.fishing_bobber {
                            let bob_color = if bobber.in_water && bobber.bite_timer <= 0.0 {
                                [1.0, 0.3, 0.1]
                            } else {
                                [0.9, 0.15, 0.15]
                            };
                            self.renderer.draw_billboard(bobber.pos, 0.16, bob_color, &effective_camera, &view_proj);
                            self.renderer.draw_billboard(bobber.pos + Vec3::new(0.0, 0.08, 0.0), 0.10, [0.95, 0.95, 0.95], &effective_camera, &view_proj);
                        }

                        // Draw flying and lodged physical arrows
                        for arrow in &self.arrows {
                            let dir = if arrow.velocity.length() > 0.001 {
                                arrow.velocity.normalized()
                            } else {
                                Vec3::new(0.0, -1.0, 0.0)
                            };
                            self.renderer.draw_billboard(arrow.position, 0.12, [0.35, 0.35, 0.38], &effective_camera, &view_proj);
                            self.renderer.draw_billboard(arrow.position - dir * 0.18, 0.09, [0.65, 0.45, 0.25], &effective_camera, &view_proj);
                            self.renderer.draw_billboard(arrow.position - dir * 0.36, 0.11, [0.95, 0.95, 0.95], &effective_camera, &view_proj);
                        }

                        // Weather Precipitation (Rain & Snow) and Lightning Bolts
                        self.renderer.draw_precipitation(&self.world, &effective_camera, &view_proj, self.total_time);
                        self.renderer.draw_lightning_bolts(&self.world.weather.active_lightning, &view_proj);
                    }

                    if let Some(client) = &self.net_client {
                        for rp in client.remote_players.values() {
                            self.renderer.draw_remote_player(rp, &effective_camera, &view_proj);
                        }
                    }

                    let block_at_camera = self.world.get_block(self.camera.position.floor_to_ivec());
                    if block_at_camera == BlockId::Lava {
                        self.renderer.draw_lava_overlay(self.total_time);
                    } else {
                        if self.settings.underwater_fx && matches!(block_at_camera, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
                            self.renderer.draw_underwater_overlay(self.total_time);
                        }
                        if self.player.fire_timer > 0.0 {
                            self.renderer.draw_burning_fire_overlay(self.total_time);
                        }
                    }

                    // Apply biome color filter postprocess pass
                    self.renderer.apply_biome_color_filter(self.biome_filter_tint);

                    if self.damage_tilt_timer > 0.0 {
                        self.renderer.draw_damage_flash(self.damage_tilt_timer / 0.4);
                    }
                    if self.player.gamemode == GameMode::Survival && self.player.health <= 2.0 && self.player.health > 0.0 {
                        self.renderer.draw_low_health_vignette(self.total_time);
                    }
                    if let Some(e_start) = render_entities_start {
                        cur_profiler.render_entities_ms = e_start.elapsed().as_secs_f32() * 1000.0;
                    }
                }

                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0);
                }

                let render_ui_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                if self.camera_perspective != CameraPerspective::ThirdPersonFront {
                    self.renderer.draw_crosshair();
                }
                self.renderer.draw_break_progress(self.break_progress);
                #[cfg(not(target_os = "vita"))]
                {
                    if self.camera_perspective == CameraPerspective::FirstPerson {
                        let hand_start = if collect_profiling { Some(std::time::Instant::now()) } else { None };
                        let hand_block: Option<u8> = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                            .map(|s| s.block.to_u8());
                        let aspect = (self.width as f32) / (self.height as f32);
                        let hand_proj = Mat4::perspective(68.0_f32.to_radians(), aspect, 0.01, 20.0);
                        let hand_view_proj = hand_proj.mul(&effective_camera.view_matrix());
                        let offhand_block: Option<u8> = self.player.inventory.offhand.as_ref().map(|s| s.block.to_u8());
                        self.renderer.draw_hand_3d(&effective_camera, &hand_view_proj, hand_block, self.walk_phase, hand_speed, self.hand_animation);
                        self.renderer.draw_offhand_3d(&effective_camera, &hand_view_proj, offhand_block, self.walk_phase, hand_speed, self.player.is_blocking);
                        if let Some(h_start) = hand_start {
                            cur_profiler.render_hand_ms = h_start.elapsed().as_secs_f32() * 1000.0;
                        }
                    }
                }
                self.renderer.draw_hotbar(&self.player);
                crate::ui::minimap::draw_world_waypoint_markers(
                    &mut self.renderer.color_buffer,
                    self.width,
                    self.height,
                    &effective_camera,
                    &self.minimap.waypoints,
                );
                crate::ui::minimap::draw_minimap(
                    &mut self.renderer.color_buffer,
                    self.width,
                    self.height,
                    &self.world,
                    self.camera.position,
                    self.camera.yaw,
                    &self.mob_manager.mobs,
                    &self.minimap,
                );
                self.chat.draw(&mut self.renderer.color_buffer, self.width, self.height);

                if input.key_tab && !self.chat.is_open {
                    let p_list = if let Some(client) = &self.net_client {
                        &client.player_list[..]
                    } else {
                        &[]
                    };
                    self.renderer.draw_player_list_tab(p_list, &self.settings.nickname, self.net_client.is_some());
                }
                if let Some(u_start) = render_ui_start {
                    cur_profiler.render_ui_ms = u_start.elapsed().as_secs_f32() * 1000.0;
                }

                if collect_profiling {
                    if let Some(f_start) = frame_start_time {
                        cur_profiler.total_frame_ms = f_start.elapsed().as_secs_f32() * 1000.0;
                    }
                    cur_profiler.fps = (1.0 / dt.max(0.001)).round() as u32;

                    let (draw_calls, tris_rendered, tris_culled, subchunks_rendered, subchunks_culled) = self.renderer.last_render_telemetry();
                    #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
                    let raster_threads = rayon::current_num_threads();
                    #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
                    let raster_threads = 1;

                    let (renderer_name, gpu_device, driver_info) = self.renderer.renderer_info();
                    cur_profiler.render_stats = crate::core::profiler::RenderStats {
                        pipeline_name: self.renderer.backend_name(),
                        renderer_name,
                        gpu_device,
                        driver_info,
                        draw_calls,
                        subchunks_rendered,
                        subchunks_culled_frustum: subchunks_culled,
                        subchunks_culled_occlusion: 0,
                        subchunks_empty_skipped: 0,
                        triangles_rendered: tris_rendered,
                        triangles_culled: tris_culled,
                        raster_threads,
                    };

                    let chunk_count = self.world.chunks.len();
                    #[cfg(target_os = "vita")]
                    let chunk_voxels_bytes = chunk_count * 16 * 16 * 64;
                    #[cfg(not(target_os = "vita"))]
                    let chunk_voxels_bytes = chunk_count * 16 * 16 * 128;

                    #[cfg(target_os = "vita")]
                    let mesh_cache_bytes = self.chunk_slots_info.0 * 64 * 1024;
                    #[cfg(not(target_os = "vita"))]
                    let mesh_cache_bytes = self.renderer.mesh_cache_triangles_count() * std::mem::size_of::<crate::world::Triangle>();

                    let textures_and_fonts_bytes = 256 * 256 * 4 + 256 * 1024;
                    let framebuffers_bytes = (self.width * self.height * 4) * 2;
                    let entities_bytes = self.mob_manager.mobs.len() * 256 + self.items.len() * 128 + self.particles.particles.len() * 64;
                    let ui_and_misc_bytes = 256 * 1024;
                    let total_ram_bytes = chunk_voxels_bytes + mesh_cache_bytes + textures_and_fonts_bytes + framebuffers_bytes + entities_bytes + ui_and_misc_bytes;

                    #[cfg(target_os = "vita")]
                    let cdram_bytes = (self.cdram_mb * 1024.0 * 1024.0) as usize;
                    #[cfg(not(target_os = "vita"))]
                    let cdram_bytes = 0;

                    cur_profiler.memory = MemoryBreakdown {
                        total_ram_bytes,
                        chunk_voxels_bytes,
                        mesh_cache_bytes,
                        textures_and_fonts_bytes,
                        framebuffers_bytes,
                        entities_bytes,
                        ui_and_misc_bytes,
                        cdram_bytes,
                    };

                    self.profiler.smooth_update(&cur_profiler, 0.15);

                    if let Some(ref server) = self.profiler_server {
                        let snap = self.build_profiler_snapshot(&cur_profiler);
                        server.update_snapshot(snap);
                    }
                }

                if self.settings.debug_overlay {
                    self.debug_timer += dt;
                    if self.debug_timer >= 0.15 {
                        self.debug_timer = 0.0;
                        self.cpu_time_ms = dt * 1000.0;
                        self.fps_display = (1.0 / dt.max(0.001)).round() as u32;

                        let mode_str = if self.fly_mode {
                            "FLY"
                        } else if is_crouching {
                            "SNEAK"
                        } else if is_sprinting {
                            "SPRINT"
                        } else {
                            "WALK"
                        };

                        let yaw_deg = (self.camera.yaw.to_degrees() % 360.0 + 360.0) % 360.0;
                        let pitch_deg = self.camera.pitch.to_degrees();
                        let (facing_str, axis_str) = if yaw_deg >= 45.0 && yaw_deg < 135.0 {
                            ("East", "+X")
                        } else if yaw_deg >= 135.0 && yaw_deg < 225.0 {
                            ("South", "+Z")
                        } else if yaw_deg >= 225.0 && yaw_deg < 315.0 {
                            ("West", "-X")
                        } else {
                            ("North", "-Z")
                        };

                        let px = self.camera.position.x;
                        let py = self.camera.position.y;
                        let pz = self.camera.position.z;
                        let bpos = self.camera.position.floor_to_ivec();
                        let cx = bpos.x.div_euclid(16);
                        let cz = bpos.z.div_euclid(16);
                        let in_cx = bpos.x.rem_euclid(16);
                        let in_cz = bpos.z.rem_euclid(16);

                        let biome = self.world.biome_name_at(bpos.x, bpos.z);
                        let biome_key = match biome {
                            "MOUNTAINS" => "biome.mountains",
                            "SNOW" => "biome.snow",
                            "SNOWY TAIGA" => "biome.snowy_taiga",
                            "SNOWY TUNDRA" => "biome.snowy_tundra",
                            "ICE SPIKES" => "biome.ice_spikes",
                            "TAIGA" => "biome.taiga",
                            "DESERT" => "biome.desert",
                            "BADLANDS" => "biome.badlands",
                            "OCEAN" => "biome.ocean",
                            "DEEP OCEAN" => "biome.deep_ocean",
                            "MUSHROOM ISLAND" => "biome.mushroom_island",
                            "CHERRY GROVE" => "biome.cherry_grove",
                            "AUTUMN FOREST" => "biome.autumn_forest",
                            "DARK FOREST" => "biome.dark_forest",
                            "BIRCH FOREST" => "biome.birch_forest",
                            "JUNGLE" => "biome.jungle",
                            "SAVANNA" => "biome.savanna",
                            "SWAMP" => "biome.swamp",
                            "MEADOW" => "biome.meadow",
                            "FOREST" => "biome.forest",
                            _ => "biome.plains",
                        };

                        let time_hrs = (self.world.day_fraction() * 24.0 + 6.0) % 24.0;
                        let hours = time_hrs.floor() as u32;
                        let minutes = (time_hrs.fract() * 60.0) as u32;

                        // Left Lines
                        let (ren_name, _, _) = self.renderer.renderer_info();
                        self.debug_lines[0].clear();
                        let _ = write!(&mut self.debug_lines[0], "{}", crate::core::build_info::get_build_info());

                        self.debug_lines[1].clear();
                        let _ = write!(&mut self.debug_lines[1], "FPS: {} ({:.1}ms) | Renderer: {} ({} thr)", self.fps_display, dt * 1000.0, ren_name, self.profiler.render_stats.raster_threads);

                        self.debug_lines[2].clear();
                        let _ = write!(&mut self.debug_lines[2], "Chunks: {} loaded | Subchunks: {} ren, {} cul", self.world.chunks.len(), self.profiler.render_stats.subchunks_rendered, self.profiler.render_stats.subchunks_culled_frustum);

                        self.debug_lines[3].clear();
                        let _ = write!(&mut self.debug_lines[3], "XYZ: {:.3} / {:.3} / {:.3}", px, py, pz);

                        self.debug_lines[4].clear();
                        let _ = write!(&mut self.debug_lines[4], "Block: {} {} {} [{} {} in chunk {}, {}]", bpos.x, bpos.y, bpos.z, in_cx, in_cz, cx, cz);

                        self.debug_lines[5].clear();
                        let _ = write!(&mut self.debug_lines[5], "Facing: {} ({}) (Yaw: {:.1} / Pitch: {:.1})", facing_str, axis_str, yaw_deg, pitch_deg);

                        self.debug_lines[6].clear();
                        let weather_str = match self.world.weather.current_type {
                            crate::world::weather::WeatherType::Clear => "Clear",
                            crate::world::weather::WeatherType::Rain => "Rain",
                            crate::world::weather::WeatherType::Thunder => "Thunder",
                        };
                        let _ = write!(
                            &mut self.debug_lines[6],
                            "Biome: {} | Light: {:.0}% ({:02}:{:02}) | Weather: {}",
                            crate::core::i18n::t(biome_key),
                            self.world.sunlight_factor() * 100.0,
                            hours,
                            minutes,
                            weather_str
                        );

                        self.debug_lines[7].clear();
                        if let Some(target) = self.break_target {
                            let target_block = self.world.get_block(target);
                            let _ = write!(&mut self.debug_lines[7], "Target: {} [{}, {}, {}]", target_block.display_name().to_uppercase(), target.x, target.y, target.z);
                        } else {
                            let sel_name = BlockId::from_u8(self.renderer.selected_block()).display_name().to_uppercase();
                            let _ = write!(&mut self.debug_lines[7], "Holding: {}", sel_name);
                        }

                        self.debug_lines[8].clear();
                        let _ = write!(&mut self.debug_lines[8], "Entities: {} mobs, {} drops, {} fx | Mode: {}", self.mob_manager.mobs.len(), self.items.len(), self.particles.particles.len(), mode_str);

                        // Right Lines (Memory breakdown & Render telemetry)
                        if self.profiler_enabled {
                            while self.debug_right_lines.len() < 14 {
                                self.debug_right_lines.push(String::with_capacity(96));
                            }
                            let mem = &self.profiler.memory;
                            let stats = &self.profiler.render_stats;
                            let mesh_tris = self.renderer.mesh_cache_triangles_count();
                            self.debug_right_lines[0].clear();
                            #[cfg(target_os = "vita")]
                            let _ = write!(&mut self.debug_right_lines[0], "Platform: Sony PlayStation Vita (ARM Cortex-A9)");
                            #[cfg(not(target_os = "vita"))]
                            let _ = write!(&mut self.debug_right_lines[0], "Engine: Rust 1.85 (x86_64)");

                            self.debug_right_lines[1].clear();
                            let _ = write!(&mut self.debug_right_lines[1], "Renderer: {}", stats.renderer_name);

                            self.debug_right_lines[2].clear();
                            let _ = write!(&mut self.debug_right_lines[2], "GPU: {}", stats.gpu_device);

                            self.debug_right_lines[3].clear();
                            let _ = write!(&mut self.debug_right_lines[3], "Driver: {}", stats.driver_info);

                            self.debug_right_lines[4].clear();
                            let _ = write!(&mut self.debug_right_lines[4], "RAM (EST): ~{:.2} MB", MemoryBreakdown::to_mb(mem.total_ram_bytes));

                            self.debug_right_lines[5].clear();
                            let _ = write!(&mut self.debug_right_lines[5], " - Chunk Voxels: {:.2} MB ({} ch)", MemoryBreakdown::to_mb(mem.chunk_voxels_bytes), self.world.chunks.len());

                            self.debug_right_lines[6].clear();
                            let _ = write!(&mut self.debug_right_lines[6], " - Mesh Cache:   {:.2} MB ({} tris)", MemoryBreakdown::to_mb(mem.mesh_cache_bytes), mesh_tris);

                            self.debug_right_lines[7].clear();
                            let _ = write!(&mut self.debug_right_lines[7], " - Framebuffers: {:.2} MB (Color+Depth)", MemoryBreakdown::to_mb(mem.framebuffers_bytes));

                            self.debug_right_lines[8].clear();
                            let _ = write!(&mut self.debug_right_lines[8], " - Textures/Font:{:.2} MB (256x256 RGBA)", MemoryBreakdown::to_mb(mem.textures_and_fonts_bytes));

                            self.debug_right_lines[9].clear();
                            let _ = write!(&mut self.debug_right_lines[9], " - Entities/Misc:{:.1} KB", MemoryBreakdown::to_kb(mem.entities_bytes + mem.ui_and_misc_bytes));

                            self.debug_right_lines[10].clear();
                            let _ = write!(&mut self.debug_right_lines[10], "Display: {}x{} ({})", self.width, self.height, stats.pipeline_name);

                            self.debug_right_lines[11].clear();
                            let _ = write!(&mut self.debug_right_lines[11], "Draw Calls: {}", stats.draw_calls);

                            self.debug_right_lines[12].clear();
                            let _ = write!(&mut self.debug_right_lines[12], "Triangles: {} ren / {} cul", stats.triangles_rendered, stats.triangles_culled);

                            self.debug_right_lines[13].clear();
                            let _ = write!(&mut self.debug_right_lines[13], "Subchunks: {} ren / {} cul", stats.subchunks_rendered, stats.subchunks_culled_frustum);
                        }
                    }

                    if self.profiler_enabled {
                        self.renderer.draw_advanced_debug_overlay(&self.debug_lines, &self.debug_right_lines, &self.profiler);
                    } else {
                        self.renderer.draw_debug_panel(&self.debug_lines);
                    }
                }

                if !input.is_mouse_captured && !self.chat.is_open {
                    self.renderer.draw_capture_prompt(true);
                }

                self.thumbnail_timer += dt;
                if self.thumbnail_timer >= 0.5 {
                    self.thumbnail_timer = 0.0;
                    self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(self.renderer.pixel_buffer(), self.width, self.height));
                }

    }
}

#[cfg(test)]
mod tests {
    use crate::app::{RustCraftApp, InputState, GameState};
    use crate::gameplay::player::GameMode;
    use crate::gameplay::inventory::ItemStack;
    use crate::core::math::{Vec3, IVec3};

    #[test]
    fn blocks_can_be_placed_into_water_but_not_solid_blocks() {
        assert!(super::can_place_block_in(crate::world::BlockId::Air));
        assert!(super::can_place_block_in(crate::world::BlockId::Water));
        assert!(!super::can_place_block_in(crate::world::BlockId::Stone));
        assert!(!super::can_place_block_in(crate::world::BlockId::Lava));
    }

    #[test]
    fn block_placement_is_blocked_by_live_mobs_but_not_dead_ones() {
        use crate::gameplay::mob::{Mob, MobKind};

        let place_pos = IVec3::new(4, 10, 4);
        let mob = Mob::new(1, MobKind::Pig, Vec3::new(4.5, 10.0, 4.5));
        assert!(super::block_intersects_live_mob(place_pos, std::slice::from_ref(&mob)));
        assert!(!super::block_intersects_live_mob(
            IVec3::new(6, 10, 4),
            std::slice::from_ref(&mob)
        ));

        let mut dead_mob = mob;
        dead_mob.is_dead = true;
        assert!(!super::block_intersects_live_mob(place_pos, &[dead_mob]));
    }

    #[test]
    fn opening_chat_releases_mouse_and_closing_it_recaptures() {
        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        let mut input = InputState {
            key_t: true,
            is_mouse_captured: true,
            ..Default::default()
        };

        let opened = app.update_and_render(0.016, &input);
        assert!(app.chat.is_open);
        assert!(opened.request_release_mouse);
        assert!(!opened.is_playing);

        input.key_t = false;
        input.is_mouse_captured = false;
        input.key_escape = true;
        let closed = app.update_and_render(0.016, &input);
        assert!(!app.chat.is_open);
        assert!(closed.request_capture_mouse);
        assert!(closed.is_playing);
        assert!(matches!(app.state, GameState::Playing));
    }

    #[test]
    fn test_creative_flight_double_tap_space() {
        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        app.player.gamemode = GameMode::Creative;
        app.fly_mode = false;
        app.jump_double_tap_timer = 0.0;

        let mut input = InputState::default();
        input.is_mouse_captured = true;

        // 1. First Space press
        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        assert!(!app.fly_mode, "First press should not trigger flying");
        assert!(app.jump_double_tap_timer > 0.0, "Timer should be set after first press");

        // 2. Release Space
        input.key_space = false;
        let _ = app.update_and_render(0.05, &input);
        assert!(!app.fly_mode);
        assert!(app.jump_double_tap_timer > 0.0, "Timer should still be active");

        // 3. Second Space press within window -> should enable flight!
        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        assert!(app.fly_mode, "Double-tapping Space in creative mode should enable flight");
        assert_eq!(app.jump_double_tap_timer, 0.0, "Timer should reset after double tap");

        // 4. Release Space while flying
        input.key_space = false;
        let _ = app.update_and_render(0.05, &input);
        assert!(app.fly_mode, "Should remain in flight mode");

        // 5. First Space press to exit flight
        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        assert!(app.fly_mode, "First press while flying should not immediately exit flight");
        assert!(app.jump_double_tap_timer > 0.0);

        // 6. Release Space
        input.key_space = false;
        let _ = app.update_and_render(0.05, &input);
        assert!(app.fly_mode);

        // 7. Second Space press within window -> should exit flight!
        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        assert!(!app.fly_mode, "Double-tapping Space in creative mode should exit flight");
        assert_eq!(app.jump_double_tap_timer, 0.0);

        // 8. In Survival mode, double tap space should NEVER enable flight
        app.player.gamemode = GameMode::Survival;
        app.fly_mode = false;
        app.jump_double_tap_timer = 0.0;

        input.key_space = false;
        let _ = app.update_and_render(0.05, &input);

        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        input.key_space = false;
        let _ = app.update_and_render(0.05, &input);
        input.key_space = true;
        let _ = app.update_and_render(0.016, &input);
        assert!(!app.fly_mode, "Survival mode must never allow flight via double-tap space");
    }

    #[test]
    fn test_creative_break_cooldown_holding_mouse() {
        use crate::world::BlockId;

        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        app.player.gamemode = GameMode::Creative;
        app.fly_mode = true;
        app.camera.position = Vec3::new(0.5, 60.0, 0.5);
        app.camera.yaw = 0.0;
        app.camera.pitch = 0.0;

        // With yaw = 0.0, pitch = 0.0, forward direction is along +X
        let block1_pos = IVec3::new(2, 60, 0);
        let block2_pos = IVec3::new(3, 60, 0);
        app.world.set_block(block1_pos, BlockId::Stone);
        app.world.set_block(block2_pos, BlockId::Stone);

        let mut input = InputState::default();
        input.is_mouse_captured = true;
        input.mouse_left = true;

        // Frame 1: breaks block 1 immediately
        let _ = app.update_and_render(0.016, &input);
        assert_eq!(app.world.get_block(block1_pos), BlockId::Air, "Block 1 should break on first frame");
        assert_eq!(app.world.get_block(block2_pos), BlockId::Stone, "Block 2 should NOT break immediately");
        assert!(app.creative_break_cooldown > 0.0, "Cooldown should be set after breaking in creative");

        // Simulate 100ms of holding mouse (cooldown still active)
        let _ = app.update_and_render(0.10, &input);
        assert_eq!(app.world.get_block(block2_pos), BlockId::Stone, "Block 2 should still exist after 100ms");
        assert!(app.creative_break_cooldown > 0.0);

        // Simulate another 100ms (total 200ms < 250ms)
        let _ = app.update_and_render(0.10, &input);
        assert_eq!(app.world.get_block(block2_pos), BlockId::Stone, "Block 2 should still exist after 200ms");

        // Simulate remaining 60ms (total > 250ms)
        let _ = app.update_and_render(0.06, &input);
        assert_eq!(app.world.get_block(block2_pos), BlockId::Air, "Block 2 should break after 250ms cooldown elapsed");
    }

    #[test]
    fn test_pick_block_item_mappings() {
        use crate::world::BlockId;

        assert_eq!(BlockId::Grass.pick_item(), Some(BlockId::Grass));
        assert_eq!(BlockId::Stone.pick_item(), Some(BlockId::Stone));
        assert_eq!(BlockId::Air.pick_item(), None);
        assert_eq!(BlockId::Water.pick_item(), None);
        assert_eq!(BlockId::Lava.pick_item(), None);
        assert_eq!(BlockId::RustStoneWireLit.pick_item(), Some(BlockId::RustStone));
        assert_eq!(BlockId::RustStoneLampLit.pick_item(), Some(BlockId::RustStoneLamp));
        assert_eq!(BlockId::LeverOn.pick_item(), Some(BlockId::Lever));
        assert_eq!(BlockId::ButtonPressed.pick_item(), Some(BlockId::Button));
        assert_eq!(BlockId::PressurePlatePressed.pick_item(), Some(BlockId::PressurePlate));
        assert_eq!(BlockId::DoorOpen.pick_item(), Some(BlockId::Door));
        assert_eq!(BlockId::FurnaceLit.pick_item(), Some(BlockId::Furnace));
        assert_eq!(BlockId::Farmland.pick_item(), Some(BlockId::Dirt));
        assert_eq!(BlockId::FarmlandWet.pick_item(), Some(BlockId::Dirt));
    }

    #[test]
    fn test_creative_pick_block_middle_click() {
        use crate::world::BlockId;

        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        app.player.gamemode = GameMode::Creative;
        app.fly_mode = true;
        app.camera.position = Vec3::new(0.5, 60.0, 0.5);
        app.camera.yaw = 0.0;
        app.camera.pitch = 0.0;

        let target_pos = IVec3::new(3, 60, 0);
        app.world.set_block(target_pos, BlockId::RustStoneBlock);

        // Ensure current hotbar slot 0 is empty
        app.player.selected_hotbar_slot = 0;
        app.player.inventory.hotbar[0] = None;

        let mut input = InputState::default();
        input.is_mouse_captured = true;
        input.mouse_middle = true;

        // Middle click in Creative should put 64x RustStoneBlock into current empty slot 0
        let _ = app.update_and_render(0.016, &input);
        assert_eq!(
            app.player.inventory.hotbar[0],
            Some(ItemStack::new(BlockId::RustStoneBlock, 64)),
            "Middle click in creative should place 64x of picked block into current free slot"
        );
        assert_eq!(app.player.selected_hotbar_slot, 0);

        // Now place a different block ahead: DiamondBlock
        app.world.set_block(target_pos, BlockId::DiamondBlock);
        input.mouse_middle = false;
        let _ = app.update_and_render(0.016, &input);

        // Current slot 0 has RustStoneBlock; slot 1 is free
        app.player.inventory.hotbar[1] = None;
        input.mouse_middle = true;
        let _ = app.update_and_render(0.016, &input);

        // DiamondBlock should be placed into free slot 1 and selected!
        assert_eq!(
            app.player.inventory.hotbar[1],
            Some(ItemStack::new(BlockId::DiamondBlock, 64)),
            "DiamondBlock should go into the next free slot"
        );
        assert_eq!(app.player.selected_hotbar_slot, 1, "Selected slot should switch to the free slot where item was placed");

        // Middle click looking at RustStoneBlock should switch back to slot 0
        app.world.set_block(target_pos, BlockId::RustStoneBlock);
        input.mouse_middle = false;
        let _ = app.update_and_render(0.016, &input);
        input.mouse_middle = true;
        let _ = app.update_and_render(0.016, &input);
        assert_eq!(app.player.selected_hotbar_slot, 0, "Middle click should select existing hotbar slot if already in hotbar");

        // Middle click looking at RustStone wire should pick RustStone
        app.world.set_block(target_pos, BlockId::RustStoneWireLit);
        input.mouse_middle = false;
        let _ = app.update_and_render(0.016, &input);
        app.player.inventory.hotbar[2] = None;
        app.player.selected_hotbar_slot = 2;
        input.mouse_middle = true;
        let _ = app.update_and_render(0.016, &input);
        assert_eq!(
            app.player.inventory.hotbar[2],
            Some(ItemStack::new(BlockId::RustStone, 64)),
            "Middle click on RustStoneWireLit should pick RustStone item"
        );
    }

    #[test]
    fn test_break_ruststone_wire() {
        use crate::world::BlockId;

        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        app.player.gamemode = GameMode::Survival;
        app.fly_mode = true;
        app.camera.position = Vec3::new(0.5, 60.0, 0.5);
        app.camera.yaw = 0.0;
        app.camera.pitch = 0.0;

        let target_pos = IVec3::new(3, 60, 0);
        app.world.set_block(target_pos, BlockId::RustStoneWireLit);

        let mut input = InputState::default();
        input.is_mouse_captured = true;
        input.mouse_left = true;

        // Break wire (break_time is 0.05s)
        let _ = app.update_and_render(0.1, &input);
        assert_eq!(app.world.get_block(target_pos), BlockId::Air, "Breaking wire should clear the block");
        assert!(app.items.iter().any(|item| item.block == BlockId::RustStone), "Breaking wire should drop RustStone item");
    }

    #[test]
    fn test_aquatic_update_mechanics() {
        use crate::world::BlockId;
        use crate::gameplay::mob::MobKind;

        let mut app = RustCraftApp::new(160, 120, "software", 12345);
        app.state = GameState::Playing;
        app.player.gamemode = GameMode::Survival;
        app.fly_mode = false;
        app.camera.position = Vec3::new(10.5, 15.0, 10.5);

        // 1. Dolphin's Grace: swimming in water near alive dolphin increases speed
        app.world.set_block(IVec3::new(10, 15, 10), BlockId::Water);
        app.world.set_block(IVec3::new(10, 14, 10), BlockId::Water);
        app.mob_manager.spawn_mob(MobKind::Dolphin, Vec3::new(11.0, 15.0, 11.0));

        let feet_pos = Vec3::new(app.camera.position.x, app.camera.position.y - app.current_eye_offset - 0.08, app.camera.position.z);
        let feet_block = app.world.get_block(feet_pos.floor_to_ivec());
        let eye_block = app.world.get_block(app.camera.position.floor_to_ivec());
        let in_water = feet_block == BlockId::Water || eye_block == BlockId::Water;
        assert!(in_water, "Player should be in water");

        let near_dolphin = in_water && app.mob_manager.mobs.iter().any(|m| {
            m.kind == MobKind::Dolphin && m.health > 0.0 && (m.pos - app.camera.position).length_sq() < 196.0
        });
        assert!(near_dolphin, "Player should receive Dolphin's Grace near alive dolphin");

        // 2. Eating Pufferfish deals damage to player
        app.player.invulnerability_timer = 0.0;
        let initial_health = app.player.health;
        app.player.hunger = 10.0;
        assert!(app.player.eat(BlockId::Pufferfish));
        // Simulate eating effect
        app.player.take_damage(3.0);
        assert!(app.player.health < initial_health, "Eating pufferfish should deal damage");

        // 3. Kelp column breaking: breaking bottom kelp block should break blocks above
        app.world.set_block(IVec3::new(20, 10, 20), BlockId::Sand);
        app.world.set_block(IVec3::new(20, 11, 20), BlockId::Kelp);
        app.world.set_block(IVec3::new(20, 12, 20), BlockId::Kelp);
        app.world.set_block(IVec3::new(20, 13, 20), BlockId::Kelp);
        assert_eq!(app.world.get_block(IVec3::new(20, 11, 20)), BlockId::Kelp);
        assert_eq!(app.world.get_block(IVec3::new(20, 12, 20)), BlockId::Kelp);
        assert_eq!(app.world.get_block(IVec3::new(20, 13, 20)), BlockId::Kelp);

        // 4. Fishing Bobber: cast and reel in
        app.fishing_bobber = Some(crate::app::FishingBobber {
            pos: Vec3::new(20.0, 15.0, 20.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            in_water: true,
            bite_timer: 0.0,
            bite_window: 2.0,
            splash_timer: 0.0,
        });
        assert!(app.fishing_bobber.is_some());
        let bobber = app.fishing_bobber.take().unwrap();
        assert!(bobber.in_water && bobber.bite_timer <= 0.0 && bobber.bite_window > 0.0);
    }
}
