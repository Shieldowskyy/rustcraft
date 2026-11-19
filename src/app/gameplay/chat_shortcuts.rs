//! Playing-state chat input, slash commands, telemetry logging, and keyboard shortcuts.
//! Handles opening/closing chat, executing commands, F3+A chunk reload, save, pause, inventory, questbook, and map keys.
//! Mutates `GameState`, `chat`, `minimap`, and mouse capture requests in `AppOutput`.

use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::world::storage::{get_save_metadata, save_path};

use super::super::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn log_playing_telemetry(&mut self, dt: f32) {
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
    }

    pub(super) fn handle_chat_and_shortcuts(&mut self, input: &InputState, out: &mut AppOutput) {
        let open_chat_requested =
            (input.key_t && !self.prev_t_down) || self.controls.is_just_pressed(Action::OpenChat);
        let open_cmd_requested = (input.key_slash && !self.prev_slash_down)
            || self.controls.is_just_pressed(Action::OpenCommand);
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
                .map(|client| {
                    client
                        .player_list
                        .iter()
                        .map(|player| player.nickname.clone())
                        .collect()
                })
                .unwrap_or_default();
            if !player_names
                .iter()
                .any(|name| name == &self.settings.nickname)
            {
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
                            && matches!(
                                command_parts[0].to_ascii_lowercase().as_str(),
                                "/tp" | "/teleport"
                            );
                        let kick_player = command_parts.len() >= 2
                            && command_parts[0].eq_ignore_ascii_case("/kick");
                        let time_command = command_parts
                            .first()
                            .is_some_and(|command| command.eq_ignore_ascii_case("/time"));
                        let tps_command = command_parts
                            .first()
                            .is_some_and(|command| command.eq_ignore_ascii_case("/tps"));
                        if kick_player && !self.is_hosting_multiplayer {
                            self.chat
                                .add_info("Only the multiplayer host can kick players.");
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
                                crate::log_info!(
                                    "WORLD",
                                    "Cleared mesh cache following chunk reload command"
                                );
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
            crate::log_info!(
                "WORLD",
                "Reloaded all chunks via F3+A (chunk count: {})",
                self.world.chunks.len()
            );
        }

        if !self.chat.is_open && self.controls.is_just_pressed(Action::Save) {
            let path = save_path(self.save_slot);
            match self.save_world_progress() {
                Ok(()) => {
                    println!("Saved world to {path}");
                    crate::log_info!(
                        "SAVE",
                        "World successfully saved to slot {} ({})",
                        self.save_slot,
                        path
                    );
                }
                Err(error) => eprintln!("{error}"),
            }
            self.save_metadata_cache =
                std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
        }

        if !self.chat.is_open && !chat_was_open && self.controls.is_just_pressed(Action::Pause) {
            self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(
                self.renderer.pixel_buffer(),
                self.width,
                self.height,
            ));
            self.state = GameState::Paused;
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!("STATE", "Game paused (opened pause menu)");
        }

        if !self.chat.is_open && self.controls.is_just_pressed(Action::Inventory) {
            self.crafting_table_open = false;
            self.state = GameState::Inventory;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!("MENU", "Opened player inventory (crafting table: false)");
        }

        if !self.chat.is_open
            && self.quests.enabled
            && self.controls.is_just_pressed(Action::Questbook)
        {
            let q_inv = self.quests.sync_inventory(&self.player.inventory);
            self.process_quest_completions(q_inv);
            let q_bio = self
                .quests
                .sync_biomes(self.achievements.visited_biome_count());
            self.process_quest_completions(q_bio);
            self.state = GameState::Questbook;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_info!("MENU", "Opened questbook");
        }

        if !self.chat.is_open && input.key_m && !self.prev_m_down {
            self.worldmap.center_x = self.camera.position.x;
            self.worldmap.center_z = self.camera.position.z;
            self.state = GameState::WorldMap;
            out.request_release_mouse = true;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
        }

        if !self.chat.is_open && input.key_n && !self.prev_n_down {
            self.minimap.mode = self.minimap.mode.next();
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
        }

        if !self.chat.is_open && input.key_b && !self.prev_b_down {
            let bpos = self.camera.position.floor_to_ivec();
            let count = self.minimap.waypoints.len() + 1;
            let name = format!("Waypoint {}", count);
            let color = crate::ui::minimap::WAYPOINT_COLORS
                [(count - 1) % crate::ui::minimap::WAYPOINT_COLORS.len()]
            .0;
            self.minimap.waypoints.push(crate::ui::minimap::Waypoint {
                name: name.clone(),
                x: bpos.x,
                y: bpos.y,
                z: bpos.z,
                color,
                enabled: true,
            });
            crate::ui::minimap::save_waypoints(
                self.save_slot,
                self.world.seed,
                &self.minimap.waypoints,
            );
            self.chat.add_message(
                format!(
                    "Dodano punkt nawigacyjny: {} ({}, {}, {})",
                    name, bpos.x, bpos.y, bpos.z
                ),
                crate::ui::pack_color(0.4, 1.0, 0.4),
            );
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
        }
    }
}
