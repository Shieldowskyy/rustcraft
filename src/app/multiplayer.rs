use crate::core::math::{IVec3, Vec3};
use crate::gameplay::physics::ray_aabb_intersect;
use crate::gameplay::player::GameMode;
use crate::world::block::BlockId;
use crate::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use super::{AppOutput, GameState, InputState, RustCraftApp};

const DEFAULT_RELAY_ADDRESS: &str = "relay.rustcraft.dsh.yt:26657";

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

        // Poll LAN server discovery announcements and probe pings
        self.lan_discovery.update();

        // F5 or Gamepad Triangle/North refreshes LAN discovery
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
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                match idx {
                    0 => {
                        self.multiplayer_focus = 0;
                        let nick = self.settings.nickname.clone();
                        self.request_ime("Multiplayer Nickname", &nick);
                    }
                    1 => {
                        self.settings.nickname = crate::core::settings::generate_default_nickname();
                        let _ = self.settings.save();
                        crate::log_info!("SETTINGS", "Nickname randomized to {}", self.settings.nickname);
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
                            crate::log_info!("LAN", "Selected LAN server {}: {}", s.motd, s.addr());
                        }
                    }
                    _ => {}
                }
            }
        }

        if (tab_down && !self.prev_tab_down) || (up_arrow_down && !self.prev_up_arrow_down) || (down_arrow_down && !self.prev_down_arrow_down) {
            self.multiplayer_focus = 1 - self.multiplayer_focus;
        }
        if (escape_down && !self.prev_escape_down) || (q_down && !self.prev_q_down) || input.gp_east_just_pressed {
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            let _ = self.settings.save();
            self.state = GameState::MainMenu;
            crate::log_info!("MENU", "Returned to Main Menu from Multiplayer menu");
        }
        if enter_down && !self.prev_enter_down {
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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
            crate::ui::clipboard::set_clipboard(copied);
        }
        if ctrl_down && v_down && !self.prev_v_down {
            let pasted = crate::ui::clipboard::get_clipboard();
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
                    if self.settings.nickname.len() < 24 && (ch.is_alphanumeric() || ch == '_' || ch == '-') {
                        self.settings.nickname.push(ch);
                    }
                } else {
                    if self.multiplayer_server_addr.len() < 256
                        && ch.is_ascii()
                        && !ch.is_whitespace()
                    {
                        self.multiplayer_server_addr.push(ch);
                    }
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
            GameState::MultiplayerConnecting { server_addr, start_time } => (server_addr.clone(), *start_time),
            _ => unreachable!(),
        };
        if start_time.elapsed() > std::time::Duration::from_secs(6) {
            let was_hosting = self.is_hosting_multiplayer;
            let error = "Connection timed out (no response from server)";
            self.multiplayer_status = Some(error.to_string());
            self.multiplayer_is_connecting = false;
            self.net_client = None;
            self.stop_host_multiplayer();
            self.show_multiplayer_toast(error, true);
            self.state = if was_hosting { GameState::Paused } else { GameState::MultiplayerMenu };
            crate::log_warn!("NETWORK", "Connection to {} timed out", server_addr);
        } else if let Some(client) = &mut self.net_client {
            let events = client.update(dt);
            let mut connected_result = None;
            let mut disconnected_err = None;
            let mut pending_mods = Vec::new();

            for ev in events {
                match ev {
                    crate::net::NetworkEvent::ModsSynchronized { mods } => {
                        pending_mods.extend(mods);
                    }
                    crate::net::NetworkEvent::Connected {
                        player_id,
                        seed,
                        spawn_x,
                        spawn_y,
                        spawn_z,
                        time_of_day,
                        modified_blocks,
                    } => {
                        connected_result = Some((player_id, seed, spawn_x, spawn_y, spawn_z, time_of_day, modified_blocks));
                    }
                    crate::net::NetworkEvent::Disconnected { reason } => {
                        disconnected_err = Some(reason);
                    }
                    _ => {}
                }
            }

            if !pending_mods.is_empty() {
                let downloaded = self.sync_server_mods(pending_mods);
                self.multiplayer_status = Some(format!("Synchronized {} mod(s) from server...", downloaded));
            }

            if let Some((player_id, seed, spawn_x, spawn_y, spawn_z, time_of_day, modified_blocks)) = connected_result {
                crate::log_info!(
                    "NETWORK",
                    "Connected! Assigned Player ID: {}, World Seed: {}, Spawn: ({:.1}, {:.1}, {:.1})",
                    player_id, seed, spawn_x, spawn_y, spawn_z
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
                        self.world.set_network_block(IVec3::new(bx, by, bz), BlockId::from_u8(blk));
                    }
                    self.player.reset(GameMode::Survival);
                }
                self.is_multiplayer = true;
                self.multiplayer_is_connecting = false;
                let was_hosting = self.is_hosting_multiplayer;
                if was_hosting {
                    if let Some(invite) = &self.multiplayer_invite {
                        crate::ui::clipboard::set_clipboard(invite);
                    }
                    self.multiplayer_status = Some(crate::core::i18n::t("multiplayer.hosting_status").to_string());
                    self.show_multiplayer_toast(crate::core::i18n::t("multiplayer.host_ready_toast"), false);
                } else {
                    self.multiplayer_status = Some(crate::core::i18n::t("multiplayer.joined_toast").to_string());
                    self.show_multiplayer_toast(crate::core::i18n::t("multiplayer.joined_toast"), false);
                }
                self.chat.add_message(format!("[Server] Connected to multiplayer server as {}", self.settings.nickname), crate::ui::pack_color(1.0, 1.0, 0.4));
                if let Some(invite) = &self.multiplayer_invite {
                    self.chat.add_message(format!("[Relay] Share this invite: {invite}"), crate::ui::pack_color(0.35, 0.9, 1.0));
                    crate::ui::clipboard::set_clipboard(invite);
                }
                self.state = GameState::Playing;
                out.request_capture_mouse = true;
            } else if let Some(err) = disconnected_err {
                let was_hosting = self.is_hosting_multiplayer;
                self.multiplayer_status = Some(err.clone());
                self.multiplayer_is_connecting = false;
                self.net_client = None;
                self.stop_host_multiplayer();
                self.show_multiplayer_toast(err, true);
                self.state = if was_hosting { GameState::Paused } else { GameState::MultiplayerMenu };
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

    pub fn start_multiplayer_connect(&mut self) {
        self.multiplayer_status = Some("Connecting to server...".to_string());
        self.multiplayer_is_connecting = true;
        let display_addr = crate::net::redact_invite_address(&self.multiplayer_server_addr);
        match crate::net::NetworkClient::connect(&self.multiplayer_server_addr, &self.settings.nickname) {
            Ok(client) => {
                let _ = self.settings.save();
                self.net_client = Some(client);
                self.state = GameState::MultiplayerConnecting {
                    server_addr: display_addr.clone(),
                    start_time: std::time::Instant::now(),
                };
                crate::log_info!(
                    "NETWORK",
                    "Connecting to {} as '{}'...",
                    display_addr,
                    self.settings.nickname
                );
            }
            Err(err) => {
                self.multiplayer_status = Some(err.clone());
                self.multiplayer_is_connecting = false;
                self.show_multiplayer_toast(err, true);
                crate::log_warn!(
                    "NETWORK",
                    "Connection failed: {}",
                    self.multiplayer_status.as_ref().unwrap()
                );
            }

        }
    }

    pub(crate) fn show_multiplayer_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.multiplayer_toast = Some((message.into(), 6.0, is_error));
    }

    /// Sets the multiplayer server address and initiates connection immediately.
    pub fn connect_to_server(&mut self, addr: &str) {
        self.multiplayer_server_addr = addr.to_string();
        self.start_multiplayer_connect();
    }

    pub fn start_host_multiplayer(&mut self) -> Result<(), String> {
        if self.is_multiplayer || self.embedded_server.is_some() {
            return Err("This world is already in a multiplayer session".to_string());
        }
        let relay_addr = configured_relay_address()?;
        let room_code = crate::net::generate_room_code()?;
        let spawn = self.camera.position;
        let server = crate::net::EmbeddedServer::start(
            crate::net::DEFAULT_SERVER_PORT,
            crate::net::EmbeddedServerState {
                seed: self.world.seed,
                time_of_day: self.world.time_of_day,
                spawn_pos: (spawn.x, spawn.y, spawn.z),
                blocks: snapshot_world_changes(&self.world),
            },
        )?;
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server.port);
        let relay = crate::net::ReverseTunnelHost::connect(
            &relay_addr,
            room_code.clone(),
            local_addr,
        )?;
        let invite = crate::net::invite_address(&relay_addr, &room_code)?;
        let server_addr = format!("127.0.0.1:{}", server.port);
        let client = crate::net::NetworkClient::connect(&server_addr, &self.settings.nickname)?;

        self.net_client = Some(client);
        self.embedded_server = Some(server);
        self.relay_host = Some(relay);
        self.multiplayer_invite = Some(invite);
        if let Some(invite) = &self.multiplayer_invite {
            crate::ui::clipboard::set_clipboard(invite);
        }
        self.is_hosting_multiplayer = true;
        self.multiplayer_status = Some("Starting hosted multiplayer session...".to_string());
        self.show_multiplayer_toast(crate::core::i18n::t("multiplayer.host_start_toast"), false);
        self.multiplayer_is_connecting = true;
        self.state = GameState::MultiplayerConnecting {
            server_addr,
            start_time: std::time::Instant::now(),
        };
        Ok(())
    }

    pub(crate) fn stop_host_multiplayer(&mut self) {
        self.net_client = None;
        self.relay_host = None;
        self.embedded_server = None;
        self.is_hosting_multiplayer = false;
        self.multiplayer_invite = None;
    }

    /// Automatically synchronizes and downloads server mods (Source engine style).
    ///
    /// Saves mod manifests to `./mods/<mod_id>/mod.json`, parses and registers custom blocks/items,
    /// updates the recipe book, and refreshes the GPU/software texture atlas.
    pub fn sync_server_mods(&mut self, mods: Vec<crate::net::protocol::ModSyncEntry>) -> usize {
        let mut installed_count = 0;
        let mods_dir = std::path::Path::new("mods");
        let _ = std::fs::create_dir_all(&mods_dir);

        for m in &mods {
            let mod_dir = mods_dir.join(&m.mod_id);
            let _ = std::fs::create_dir_all(&mod_dir);
            let manifest_path = mod_dir.join("mod.json");

            // Compare with existing file on disk
            let needs_write = match std::fs::read_to_string(&manifest_path) {
                Ok(existing) => existing != m.content,
                Err(_) => true,
            };

            if needs_write {
                if let Err(err) = std::fs::write(&manifest_path, &m.content) {
                    crate::log_error!("MODS", "Failed to save downloaded mod '{}': {}", m.mod_id, err);
                    continue;
                }

                crate::log_info!("MODS", "Saved downloaded server mod '{}' to '{}'", m.mod_id, manifest_path.display());
            }

            // Register mod into game engine
            match crate::engine::modding::ModManager::register_mod_from_str(
                &m.content,
                &mod_dir,
                &mut self.recipe_book,
            ) {
                Ok(manifest) => {
                    crate::log_info!("MODS", "Synchronized server mod '{}' [{}] v{}", manifest.name, manifest.id, manifest.version);
                    installed_count += 1;
                }
                Err(err) => {
                    crate::log_warn!("MODS", "Failed to register synchronized mod '{}': {}", m.mod_id, err);
                }
            }

        }

        if installed_count > 0 {
            // Refresh texture atlas to upload new custom textures to GPU and software renderer
            self.renderer.update_textures(self.settings.fancy_leaves);
            self.chat.add_message(
                format!("[Mods] Synchronized {} mod(s) from server", installed_count),
                crate::ui::pack_color(0.4, 1.0, 0.4),
            );
        }

        installed_count
    }

    pub(crate) fn raycast_remote_player(&self, origin: Vec3, dir: Vec3, max_reach: f32) -> Option<(u32, Vec3)> {
        let client = self.net_client.as_ref()?;
        let dir = dir.normalized();
        let mut closest_hit: Option<(u32, f32)> = None;

        for (&id, rp) in &client.remote_players {
            let feet_y = rp.pos.y - crate::gameplay::physics::STANDING_EYE_OFFSET;
            let half = 0.40f32;
            let min = Vec3::new(rp.pos.x - half, feet_y, rp.pos.z - half);
            let max = Vec3::new(rp.pos.x + half, feet_y + crate::gameplay::physics::STANDING_HEIGHT, rp.pos.z + half);

            if let Some(t) = ray_aabb_intersect(origin, dir, min, max) {
                if t >= 0.0 && t <= max_reach {
                    match closest_hit {
                        Some((_, closest_t)) if t >= closest_t => {}
                        _ => closest_hit = Some((id, t)),
                    }
                }
            }
        }

        closest_hit.map(|(id, t)| (id, origin + dir * t))
    }
}

fn configured_relay_address() -> Result<String, String> {
    if let Ok(address) = std::env::var("RUSTCRAFT_RELAY_ADDR") {
        if !address.trim().is_empty() {
            return Ok(address.trim().to_string());
        }
    }
    if let Some(address) = option_env!("RUSTCRAFT_RELAY_ADDR") {
        if !address.trim().is_empty() {
            return Ok(address.trim().to_string());
        }
    }
    #[cfg(target_os = "android")]
    let config_path = crate::world::storage::app_files_dir().join("relay.properties");
    #[cfg(not(target_os = "android"))]
    let config_path = std::path::PathBuf::from("relay.properties");

    let configured_address = match std::fs::read_to_string(&config_path) {
        Ok(contents) => contents
            .lines()
            .find_map(|line| {
                let line = line.trim();
                (!line.is_empty() && !line.starts_with('#'))
                    .then(|| line.strip_prefix("relay-address="))
                    .flatten()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "Could not read relay configuration {}: {}",
                config_path.display(),
                error
            ));
        }
    };
    Ok(configured_address.unwrap_or_else(|| DEFAULT_RELAY_ADDRESS.to_string()))
}

fn snapshot_world_changes(world: &World) -> Vec<(i32, i32, i32, u8)> {
    let mut changes = Vec::new();
    for (&(cx, cz), chunk) in &world.chunks {
        let generated = Chunk::generate(cx, cz, world.seed);
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                for x in 0..CHUNK_SIZE_X {
                    let index = Chunk::index(x, y, z);
                    let block = chunk.blocks[index];
                    if block != generated.blocks[index] {
                        changes.push((cx * CHUNK_SIZE_X + x, y, cz * CHUNK_SIZE_Z + z, block));
                    }
                }
            }
        }
    }
    changes
}
