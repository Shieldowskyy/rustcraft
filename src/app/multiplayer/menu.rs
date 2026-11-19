//! Menu gry wieloosobowej, odkrywanie serwerów LAN i obsługa stanu łączenia (`MultiplayerMenu`, `MultiplayerConnecting`).

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::net::NetworkEvent;
use crate::features::player::GameMode;
use crate::features::world::World;

use crate::app::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    pub(crate) fn update_and_render_multiplayer_menu(
        &mut self,
        _dt: f32,
        input: &InputState,
        _out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let f5_down = input.key_f5;
        let tab_down = input.key_tab;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
        let escape_down = input.key_escape;
        let q_down = input.key_q;
        let enter_down = input.key_enter;
        let backspace_down = input.key_backspace;
        let ctrl_down = input.key_ctrl;
        let a_down = input.key_a;
        let c_down = input.key_c;
        let v_down = input.key_v;

        self.lan_discovery.update();

        if (f5_down && !self.prev_f5_down) || input.gp_north_just_pressed {
            self.lan_discovery.refresh();
            crate::log_info!("LAN", "LAN server discovery refreshed via hotkey");
        }

        let rects = self.multiplayer_menu_rects;
        let mut hovered_idx = None;
        if let Some((mx, my)) = mouse_pos {
            let px = mx as i32;
            let py = my as i32;
            for (i, &(bx, by, bw, bh)) in rects.iter().enumerate() {
                if px >= bx && px < bx + bw && py >= by && py < by + bh {
                    hovered_idx = Some(i);
                    break;
                }
            }
        }

        let click = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        if click {
            if let Some(idx) = hovered_idx {
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                match idx {
                    0 => {
                        self.multiplayer_focus = 0;
                        let nick = self.settings.nickname.clone();
                        self.request_ime("Multiplayer Nickname", &nick);
                    }
                    1 => {
                        self.settings.nickname = crate::core::settings::generate_default_nickname();
                        let _ = self.settings.save();
                        crate::log_info!(
                            "SETTINGS",
                            "Nickname randomized to {}",
                            self.settings.nickname
                        );
                    }
                    2 => self.multiplayer_focus = 1,
                    3 => {
                        self.start_multiplayer_connect();
                    }
                    4 => {
                        let _ = self.settings.save();
                        self.state = GameState::MainMenu;
                        crate::log_info!("MENU", "Returned to Main Menu from Multiplayer menu");
                    }
                    5 => {
                        self.lan_discovery.refresh();
                        crate::log_info!("LAN", "LAN server discovery refreshed via button click");
                    }
                    6..=9 => {
                        let slot = idx - 6;
                        if slot < self.lan_discovery.servers.len() {
                            self.selected_lan_server = Some(slot);
                            let s = &self.lan_discovery.servers[slot];
                            self.multiplayer_server_addr = s.addr();
                            crate::log_info!(
                                "LAN",
                                "Selected LAN server {}: {}",
                                s.motd,
                                s.addr()
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        if (tab_down && !self.prev_tab_down)
            || (up_arrow_down && !self.prev_up_arrow_down)
            || (down_arrow_down && !self.prev_down_arrow_down)
        {
            self.multiplayer_focus = 1 - self.multiplayer_focus;
        }
        if (escape_down && !self.prev_escape_down)
            || (q_down && !self.prev_q_down)
            || input.gp_east_just_pressed
        {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            let _ = self.settings.save();
            self.state = GameState::MainMenu;
            crate::log_info!("MENU", "Returned to Main Menu from Multiplayer menu");
        }
        if enter_down && !self.prev_enter_down {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            self.start_multiplayer_connect();
        }

        if backspace_down && !self.prev_backspace_down {
            if self.multiplayer_focus == 0 {
                self.settings.nickname.pop();
            } else {
                self.multiplayer_server_addr.pop();
            }
        }
        if ctrl_down && a_down && !self.prev_a_down {
            if self.multiplayer_focus == 0 {
                self.settings.nickname.clear();
            } else {
                self.multiplayer_server_addr.clear();
            }
        }
        if ctrl_down && c_down && !self.prev_c_down {
            let copied = if self.multiplayer_focus == 0 {
                self.settings.nickname.as_str()
            } else {
                self.multiplayer_server_addr.as_str()
            };
            crate::platform::clipboard::set_clipboard(copied);
        }
        if ctrl_down && v_down && !self.prev_v_down {
            let pasted = crate::platform::clipboard::get_clipboard();
            if self.multiplayer_focus == 0 {
                self.settings.nickname = pasted
                    .chars()
                    .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
                    .take(24)
                    .collect();
            } else {
                self.multiplayer_server_addr = pasted
                    .chars()
                    .filter(|ch| !ch.is_control() && !ch.is_whitespace())
                    .take(256)
                    .collect();
            }
        }
        if !ctrl_down {
            for &ch in &input.typed_chars {
                if ch.is_control() {
                    continue;
                }
                if self.multiplayer_focus == 0 {
                    if self.settings.nickname.len() < 24
                        && (ch.is_alphanumeric() || ch == '_' || ch == '-')
                    {
                        self.settings.nickname.push(ch);
                    }
                } else if self.multiplayer_server_addr.len() < 256
                    && ch.is_ascii()
                    && !ch.is_whitespace()
                {
                    self.multiplayer_server_addr.push(ch);
                }
            }
        }

        self.renderer.draw_multiplayer_menu(
            &self.settings.nickname,
            &self.multiplayer_server_addr,
            self.multiplayer_focus,
            hovered_idx,
            self.multiplayer_status.as_deref(),
            self.multiplayer_is_connecting,
            &self.lan_discovery.servers,
            self.selected_lan_server,
            self.lan_discovery.is_refreshing,
        );
    }

    pub(crate) fn update_and_render_multiplayer_connecting(
        &mut self,
        dt: f32,
        _input: &InputState,
        out: &mut AppOutput,
    ) {
        let (server_addr, start_time) = match &self.state {
            GameState::MultiplayerConnecting {
                server_addr,
                start_time,
            } => (server_addr.clone(), *start_time),
            _ => unreachable!(),
        };
        if start_time.elapsed() > std::time::Duration::from_secs(6) {
            let was_hosting = self.is_hosting_multiplayer;
            let error = "Connection timed out (no response from server)";
            self.multiplayer_status = Some(error.to_string());
            self.multiplayer_is_connecting = false;
            self.net_client = None;
            self.pending_network_events.clear();
            self.stop_host_multiplayer();
            self.show_multiplayer_toast(error, true);
            self.state = if was_hosting {
                GameState::Paused
            } else {
                GameState::MultiplayerMenu
            };
            crate::log_warn!("NETWORK", "Connection to {} timed out", server_addr);
        } else if let Some(client) = &mut self.net_client {
            let events = client.update(dt);
            let mut connected_result = None;
            let mut disconnected_err = None;
            let mut pending_mods = Vec::new();
            let mut player_data_result = None;

            for ev in events {
                match ev {
                    NetworkEvent::ModsSynchronized { mods } => {
                        pending_mods.extend(mods);
                    }
                    NetworkEvent::Connected {
                        player_id,
                        seed,
                        spawn_x,
                        spawn_y,
                        spawn_z,
                        time_of_day,
                        modified_blocks,
                    } => {
                        connected_result = Some((
                            player_id,
                            seed,
                            spawn_x,
                            spawn_y,
                            spawn_z,
                            time_of_day,
                            modified_blocks,
                        ));
                    }
                    NetworkEvent::Disconnected { reason } => {
                        disconnected_err = Some(reason);
                    }
                    NetworkEvent::PlayerDataLoaded { data } => {
                        player_data_result = Some(data);
                    }
                    other => self.pending_network_events.push(other),
                }
            }

            if !pending_mods.is_empty() {
                let downloaded = self.sync_server_mods(pending_mods);
                self.multiplayer_status = Some(format!(
                    "Synchronized {} mod(s) from server...",
                    downloaded
                ));
            }

            if let Some((player_id, seed, spawn_x, spawn_y, spawn_z, time_of_day, modified_blocks)) =
                connected_result
            {
                crate::log_info!(
                    "NETWORK",
                    "Connected! Assigned Player ID: {}, World Seed: {}, Spawn: ({:.1}, {:.1}, {:.1})",
                    player_id,
                    seed,
                    spawn_x,
                    spawn_y,
                    spawn_z
                );
                if self.is_hosting_multiplayer {
                    self.world.time_of_day = time_of_day;
                } else {
                    self.seed = seed;
                    self.world = World::new_with_seed(seed);
                    self.world.player_position = Vec3::new(spawn_x, spawn_y, spawn_z);
                    self.camera.position = Vec3::new(spawn_x, spawn_y, spawn_z);
                    self.world.time_of_day = time_of_day;
                    for (bx, by, bz, blk) in modified_blocks {
                        self.world
                            .set_network_block(IVec3::new(bx, by, bz), BlockId::from_u8(blk));
                    }
                    self.player.reset(GameMode::Survival);
                }
                match player_data_result {
                    Some(Some(data)) => self.apply_multiplayer_player_data(data),
                    Some(None) => {
                        let data = self.multiplayer_player_data();
                        if let Some(client) = &mut self.net_client {
                            client.send_player_data(data, true);
                        }
                    }
                    None => {}
                }
                self.death_message_sent = false;
                self.is_multiplayer = true;
                self.multiplayer_is_connecting = false;
                if self.is_hosting_multiplayer {
                    if let Some(invite) = &self.multiplayer_invite {
                        crate::platform::clipboard::set_clipboard(invite);
                    }
                    self.multiplayer_status =
                        Some(crate::core::i18n::t("multiplayer.hosting_status").to_string());
                    self.show_multiplayer_toast(
                        crate::core::i18n::t("multiplayer.host_ready_toast"),
                        false,
                    );
                } else {
                    self.multiplayer_status =
                        Some(crate::core::i18n::t("multiplayer.joined_toast").to_string());
                    self.show_multiplayer_toast(
                        crate::core::i18n::t("multiplayer.joined_toast"),
                        false,
                    );
                }
                self.chat.add_message(
                    format!(
                        "[Server] Connected to multiplayer server as {}",
                        self.settings.nickname
                    ),
                    crate::ui::pack_color(1.0, 1.0, 0.4),
                );
                if let Some(invite) = &self.multiplayer_invite {
                    self.chat.add_message(
                        format!("[Relay] Share this invite: {invite}"),
                        crate::ui::pack_color(0.35, 0.9, 1.0),
                    );
                    crate::platform::clipboard::set_clipboard(invite);
                }
                self.state = GameState::Playing;
                out.request_capture_mouse = true;
            } else if let Some(err) = disconnected_err {
                let was_hosting = self.is_hosting_multiplayer;
                crate::log_warn!("NETWORK", "Disconnected while connecting: {}", err);
                self.multiplayer_status = Some(err.clone());
                self.multiplayer_is_connecting = false;
                self.net_client = None;
                self.pending_network_events.clear();
                self.stop_host_multiplayer();
                self.show_multiplayer_toast(err, true);
                self.state = if was_hosting {
                    GameState::Paused
                } else {
                    GameState::MultiplayerMenu
                };
            }
        }

        self.renderer.draw_multiplayer_menu(
            &self.settings.nickname,
            &self.multiplayer_server_addr,
            self.multiplayer_focus,
            None,
            Some("Connecting to server..."),
            true,
            &self.lan_discovery.servers,
            self.selected_lan_server,
            false,
        );
    }
}
