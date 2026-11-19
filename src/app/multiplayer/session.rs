//! Zarządzanie sesją multiplayer: hostowanie przez relay, synchronizacja danych gracza i modów oraz raycast PvP.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::modding::ModManager;
use crate::features::net::protocol::{ModSyncEntry, PlayerData, PlayerItemStack};
use crate::features::net::{
    generate_room_code, invite_address, redact_invite_address, EmbeddedServer,
    EmbeddedServerState, NetworkClient, ReverseTunnelHost, DEFAULT_SERVER_PORT,
};
use crate::features::physics::{ray_aabb_intersect, STANDING_EYE_OFFSET, STANDING_HEIGHT};
use crate::features::player::ItemStack;
use crate::features::world::World;

use crate::app::{GameState, RustCraftApp};

const DEFAULT_RELAY_ADDRESS: &str = "relay.rustcraft.dsh.yt:26657";

impl RustCraftApp {
    pub(crate) fn multiplayer_player_data(&self) -> PlayerData {
        let to_network_item = |stack: Option<ItemStack>| {
            stack.map(|stack| PlayerItemStack {
                block: stack.block.to_u8(),
                count: stack.count,
                durability: stack.durability,
            })
        };
        PlayerData {
            x: self.camera.position.x,
            y: self.camera.position.y,
            z: self.camera.position.z,
            health: self.player.health,
            max_health: self.player.max_health,
            hunger: self.player.hunger,
            max_hunger: self.player.max_hunger,
            saturation: self.player.saturation,
            exhaustion: self.player.exhaustion,
            air: self.player.air,
            selected_hotbar_slot: self.player.selected_hotbar_slot,
            hotbar: self.player.inventory.hotbar.map(to_network_item),
            main: self.player.inventory.main.map(to_network_item),
            armor: self.player.inventory.armor.map(to_network_item),
            offhand: to_network_item(self.player.inventory.offhand),
            spawn_point: self.player.spawn_point.map(|pos| (pos.x, pos.y, pos.z)),
        }
    }

    pub(crate) fn apply_multiplayer_player_data(&mut self, data: PlayerData) {
        let from_network_item = |item: Option<PlayerItemStack>| {
            item.map(|item| {
                ItemStack::with_durability(
                    BlockId::from_u8(item.block),
                    item.count,
                    item.durability,
                )
            })
        };
        self.camera.position = Vec3::new(data.x, data.y, data.z);
        self.world.player_position = self.camera.position;
        self.player.health = data.health;
        self.player.max_health = data.max_health;
        self.player.hunger = data.hunger;
        self.player.max_hunger = data.max_hunger;
        self.player.saturation = data.saturation;
        self.player.exhaustion = data.exhaustion;
        self.player.air = data.air;
        self.player.selected_hotbar_slot = data.selected_hotbar_slot.min(9);
        self.player.inventory.hotbar = data.hotbar.map(from_network_item);
        self.player.inventory.main = data.main.map(from_network_item);
        self.player.inventory.armor = data.armor.map(from_network_item);
        self.player.inventory.offhand = from_network_item(data.offhand);
        self.player.spawn_point = data.spawn_point.map(|(x, y, z)| IVec3::new(x, y, z));
    }

    pub(crate) fn announce_player_death(&mut self) {
        if self.death_message_sent {
            return;
        }
        if let Some(client) = &mut self.net_client {
            client.send_player_death();
            self.death_message_sent = true;
        }
    }

    pub fn start_multiplayer_connect(&mut self) {
        self.pending_network_events.clear();
        self.multiplayer_status = Some("Connecting to server...".to_string());
        self.multiplayer_is_connecting = true;
        let display_addr = redact_invite_address(&self.multiplayer_server_addr);
        match NetworkClient::connect(&self.multiplayer_server_addr, &self.settings.nickname) {
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
                crate::log_warn!("NETWORK", "Connection failed: {}", err);
                self.multiplayer_status = Some(err.clone());
                self.multiplayer_is_connecting = false;
                self.show_multiplayer_toast(err, true);
            }
        }
    }

    pub(crate) fn show_multiplayer_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.multiplayer_toast = Some((message.into(), 6.0, is_error));
    }

    pub(crate) fn handle_multiplayer_disconnect(&mut self, reason: String) {
        crate::log_warn!("NETWORK", "Disconnected from multiplayer: {}", reason);
        self.multiplayer_status = Some(format!("Disconnected: {reason}"));
        self.show_multiplayer_toast(format!("Disconnected: {reason}"), true);
        self.pending_network_events.clear();
        self.is_multiplayer = false;
        self.net_client = None;
        self.stop_host_multiplayer();
        self.state = GameState::MainMenu;
    }

    pub fn connect_to_server(&mut self, addr: &str) {
        self.multiplayer_server_addr = addr.to_string();
        self.start_multiplayer_connect();
    }

    pub fn start_host_multiplayer(&mut self) -> Result<(), String> {
        if self.is_multiplayer || self.embedded_server.is_some() {
            return Err("This world is already in a multiplayer session".to_string());
        }
        self.pending_network_events.clear();
        let relay_addr = configured_relay_address()?;
        let room_code = generate_room_code()?;
        let spawn = self.camera.position;
        let server = EmbeddedServer::start(
            DEFAULT_SERVER_PORT,
            EmbeddedServerState {
                seed: self.world.seed,
                time_of_day: self.world.time_of_day,
                spawn_pos: (spawn.x, spawn.y, spawn.z),
                blocks: snapshot_world_changes(&self.world),
                mobs: self.mob_manager.network_snapshot(),
            },
        )?;
        self.world.take_network_changes().for_each(drop);
        let server_addr = format!("127.0.0.1:{}", server.port);
        let client = NetworkClient::connect(&server_addr, &self.settings.nickname)?;
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server.port);
        let relay = ReverseTunnelHost::connect(&relay_addr, room_code.clone(), local_addr)?;
        let invite = invite_address(&relay_addr, &room_code)?;

        self.net_client = Some(client);
        self.embedded_server = Some(server);
        self.relay_host = Some(relay);
        self.multiplayer_invite = Some(invite);
        if let Some(invite) = &self.multiplayer_invite {
            crate::platform::clipboard::set_clipboard(invite);
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

    pub fn sync_server_mods(&mut self, mods: Vec<ModSyncEntry>) -> usize {
        let mut installed_count = 0;
        let mods_dir = std::path::Path::new("mods");
        let _ = std::fs::create_dir_all(mods_dir);

        for m in &mods {
            let mod_dir = mods_dir.join(&m.mod_id);
            let _ = std::fs::create_dir_all(&mod_dir);
            let manifest_path = mod_dir.join("mod.json");

            let needs_write = match std::fs::read_to_string(&manifest_path) {
                Ok(existing) => existing != m.content,
                Err(_) => true,
            };

            if needs_write {
                if let Err(err) = std::fs::write(&manifest_path, &m.content) {
                    crate::log_error!(
                        "MODS",
                        "Failed to save downloaded mod '{}': {}",
                        m.mod_id,
                        err
                    );
                    continue;
                }
                crate::log_info!(
                    "MODS",
                    "Saved downloaded server mod '{}' to '{}'",
                    m.mod_id,
                    manifest_path.display()
                );
            }

            match ModManager::register_mod_from_str(&m.content, &mod_dir, &mut self.recipe_book) {
                Ok(manifest) => {
                    crate::log_info!(
                        "MODS",
                        "Synchronized server mod '{}' [{}] v{}",
                        manifest.name,
                        manifest.id,
                        manifest.version
                    );
                    installed_count += 1;
                }
                Err(err) => {
                    crate::log_warn!(
                        "MODS",
                        "Failed to register synchronized mod '{}': {}",
                        m.mod_id,
                        err
                    );
                }
            }
        }

        if installed_count > 0 {
            self.renderer.update_textures(self.settings.fancy_leaves);
            self.chat.add_message(
                format!("[Mods] Synchronized {} mod(s) from server", installed_count),
                crate::ui::pack_color(0.4, 1.0, 0.4),
            );
        }

        installed_count
    }

    pub(crate) fn raycast_remote_player(
        &self,
        origin: Vec3,
        dir: Vec3,
        max_reach: f32,
    ) -> Option<(u32, Vec3)> {
        let client = self.net_client.as_ref()?;
        let dir = dir.normalized();
        let mut closest_hit: Option<(u32, f32)> = None;

        for (&id, rp) in &client.remote_players {
            let feet_y = rp.pos.y - STANDING_EYE_OFFSET;
            let half = 0.40f32;
            let min = Vec3::new(rp.pos.x - half, feet_y, rp.pos.z - half);
            let max = Vec3::new(rp.pos.x + half, feet_y + STANDING_HEIGHT, rp.pos.z + half);

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
    let config_path = crate::platform::storage::relay_properties_path();

    let configured_address = match std::fs::read_to_string(&config_path) {
        Ok(contents) => contents.lines().find_map(|line| {
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
    world
        .block_overrides()
        .map(|(pos, block)| (pos.x, pos.y, pos.z, block))
        .collect()
}
