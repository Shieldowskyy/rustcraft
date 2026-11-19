//! Konfiguracja serwera (`server.properties`), bindowanie gniazda TCP oraz zapis/odczyt `playerdata` i `ops.txt`.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use crate::features::net::protocol::*;

pub(super) struct ConnectedClient {
    pub(super) id: u32,
    pub(super) stream: TcpStream,
    pub(super) tx_buffer: Vec<u8>,
    pub(super) tx_offset: usize,
    pub(super) output_failed: bool,
    pub(super) addr: SocketAddr,
    pub(super) nickname: String,
    pub(super) authenticated: bool,
    pub(super) player_data: Option<PlayerData>,
    pub(super) ping_ms: u32,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) z: f32,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) is_sneaking: bool,
    pub(super) is_sprinting: bool,
    pub(super) rx_buffer: Vec<u8>,
    pub(super) last_ping_sent: Instant,
    pub(super) pending_ping_seq: Option<u32>,
    pub(super) next_ping_seq: u32,
}

impl ConnectedClient {
    pub(super) fn send_packet(&mut self, packet: &ServerPacket) -> bool {
        let bytes = packet.serialize();
        let pending_bytes = self.tx_buffer.len().saturating_sub(self.tx_offset);
        if bytes.len().saturating_sub(4) > MAX_PACKET_SIZE
            || pending_bytes.saturating_add(bytes.len()) > MAX_QUEUED_BYTES
        {
            self.output_failed = true;
            return false;
        }
        if self.tx_offset >= 64 * 1024 && self.tx_offset * 2 >= self.tx_buffer.len() {
            self.tx_buffer.drain(..self.tx_offset);
            self.tx_offset = 0;
        }
        self.tx_buffer.extend_from_slice(&bytes);
        true
    }

    pub(super) fn flush_outgoing(&mut self) -> bool {
        if self.output_failed {
            return false;
        }
        while self.tx_offset < self.tx_buffer.len() {
            match self.stream.write(&self.tx_buffer[self.tx_offset..]) {
                Ok(0) => return false,
                Ok(written) => self.tx_offset += written,
                Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => return true,
                Err(ref error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return false,
            }
        }
        self.tx_buffer.clear();
        self.tx_offset = 0;
        true
    }
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub server_port: u16,
    pub level_seed: u32,
    pub motd: String,
    pub max_players: usize,
    pub pvp: bool,
    pub gamemode: String,
    pub difficulty: String,
    pub view_distance: u32,
    pub spawn_pos: (f32, f32, f32),
    pub logs_dir: String,
    pub max_log_size_mb: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_port: DEFAULT_SERVER_PORT,
            level_seed: 133742,
            motd: "Welcome to RustCraft Dedicated Server!".to_string(),
            max_players: 20,
            pvp: true,
            gamemode: "survival".to_string(),
            difficulty: "normal".to_string(),
            view_distance: 8,
            spawn_pos: (8.5, 65.0, 8.5),
            logs_dir: "logs".to_string(),
            max_log_size_mb: 20,
        }
    }
}

impl ServerConfig {
    pub fn load_or_create(path: &str) -> Self {
        if std::path::Path::new(path).exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let mut cfg = ServerConfig::default();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = trimmed.split_once('=') {
                        let key = k.trim().to_lowercase();
                        let val = v.trim();
                        match key.as_str() {
                            "server-port" => {
                                if let Ok(p) = val.parse::<u16>() {
                                    cfg.server_port = p;
                                }
                            }
                            "level-seed" => {
                                if let Ok(s) = val.parse::<u32>() {
                                    cfg.level_seed = s;
                                }
                            }
                            "motd" => cfg.motd = val.to_string(),
                            "max-players" => {
                                if let Ok(m) = val.parse::<usize>() {
                                    cfg.max_players = m;
                                }
                            }
                            "pvp" => cfg.pvp = val.eq_ignore_ascii_case("true"),
                            "gamemode" => cfg.gamemode = val.to_string(),
                            "difficulty" => cfg.difficulty = val.to_string(),
                            "view-distance" => {
                                if let Ok(vd) = val.parse::<u32>() {
                                    cfg.view_distance = vd;
                                }
                            }
                            "spawn-x" => {
                                if let Ok(x) = val.parse::<f32>() {
                                    cfg.spawn_pos.0 = x;
                                }
                            }
                            "spawn-y" => {
                                if let Ok(y) = val.parse::<f32>() {
                                    cfg.spawn_pos.1 = y;
                                }
                            }
                            "spawn-z" => {
                                if let Ok(z) = val.parse::<f32>() {
                                    cfg.spawn_pos.2 = z;
                                }
                            }
                            "logs-dir" => cfg.logs_dir = val.to_string(),
                            "max-log-size-mb" => {
                                if let Ok(mb) = val.parse::<u64>() {
                                    cfg.max_log_size_mb = mb;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                crate::server_info!("[SERVER] Loaded configuration from '{}'", path);
                return cfg;
            }
        }

        let default_cfg = ServerConfig::default();
        let props = format!(
            "# Minecraft server properties\n\
             # Auto-generated by RustCraft Dedicated Server\n\
             server-port={}\n\
             level-seed={}\n\
             motd={}\n\
             max-players={}\n\
             pvp={}\n\
             gamemode={}\n\
             difficulty={}\n\
             view-distance={}\n\
             spawn-x={:.1}\n\
             spawn-y={:.1}\n\
             spawn-z={:.1}\n\
             logs-dir={}\n\
             max-log-size-mb={}\n",
            default_cfg.server_port,
            default_cfg.level_seed,
            default_cfg.motd,
            default_cfg.max_players,
            default_cfg.pvp,
            default_cfg.gamemode,
            default_cfg.difficulty,
            default_cfg.view_distance,
            default_cfg.spawn_pos.0,
            default_cfg.spawn_pos.1,
            default_cfg.spawn_pos.2,
            default_cfg.logs_dir,
            default_cfg.max_log_size_mb,
        );
        let _ = std::fs::write(path, props);
        crate::server_info!("[SERVER] Created default configuration file at '{}'", path);
        default_cfg
    }
}

pub fn bind_server_listener(preferred_port: u16) -> Result<(TcpListener, u16), String> {
    let mut port = preferred_port;
    let max_port = preferred_port.saturating_add(50);
    while port <= max_port {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => {
                listener
                    .set_nonblocking(true)
                    .map_err(|error| format!("Could not configure server listener: {error}"))?;
                let bound_port = listener
                    .local_addr()
                    .map_err(|e| format!("Could not determine bound server port: {e}"))?
                    .port();
                return Ok((listener, bound_port));
            }
            Err(err) => {
                if port == preferred_port {
                    crate::server_info!(
                        "[SERVER] Port {} is in use ({}), checking next available port...",
                        preferred_port,
                        err
                    );
                }
                port += 1;
            }
        }
    }
    Err(format!(
        "Could not bind to any port in range {}-{}",
        preferred_port, max_port
    ))
}

pub(super) fn player_data_path(data_dir: &std::path::Path, nickname: &str) -> std::path::PathBuf {
    let key: String = nickname
        .to_ascii_lowercase()
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    data_dir.join(format!("{key}.dat"))
}

pub(super) fn is_valid_nickname(nickname: &str) -> bool {
    let nickname = nickname.trim();
    !nickname.is_empty()
        && nickname.len() <= 24
        && nickname
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
}

pub(super) fn load_player_data(
    data_dir: &std::path::Path,
    nickname: &str,
) -> Result<Option<PlayerData>, String> {
    let path = player_data_path(data_dir, nickname);
    let mut bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let packets = extract_packets(&mut bytes)
        .map_err(|error| format!("Invalid player data frame: {error}"))?;
    if !bytes.is_empty() || packets.len() != 1 {
        return Err(format!("Invalid player data file '{}'", path.display()));
    }
    match ClientPacket::deserialize(&packets[0]) {
        Ok(ClientPacket::PlayerData { data }) => Ok(Some(data)),
        Ok(_) => Err(format!(
            "Unexpected packet in player data file '{}'",
            path.display()
        )),
        Err(error) => Err(format!("Could not decode '{}': {error}", path.display())),
    }
}

pub(super) fn save_player_data(
    data_dir: &std::path::Path,
    nickname: &str,
    data: &PlayerData,
) -> Result<(), String> {
    std::fs::create_dir_all(data_dir)
        .map_err(|error| format!("Could not create '{}': {error}", data_dir.display()))?;
    let path = player_data_path(data_dir, nickname);
    std::fs::write(
        &path,
        ClientPacket::PlayerData { data: data.clone() }.serialize(),
    )
    .map_err(|error| format!("Could not write '{}': {error}", path.display()))
}

pub(super) fn persist_client_player_data(client: &mut ConnectedClient) {
    if let Some(mut data) = client.player_data.take() {
        data.x = client.x;
        data.y = client.y;
        data.z = client.z;
        if let Err(error) =
            save_player_data(std::path::Path::new("playerdata"), &client.nickname, &data)
        {
            crate::log_error!(
                "SERVER",
                "Could not save player data for '{}': {}",
                client.nickname,
                error
            );
        }
    }
}

pub(super) fn kick_client(
    clients: &mut HashMap<u32, ConnectedClient>,
    id: u32,
    reason: &str,
) -> Option<String> {
    let mut client = clients.remove(&id)?;
    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
        reason: reason.to_string(),
    });
    let _ = client.flush_outgoing();
    persist_client_player_data(&mut client);
    let nickname = client.nickname.clone();
    crate::server_info!("[SERVER] Kicked '{}' (ID {}): {}", nickname, id, reason);

    let left_packet = ServerPacket::PlayerLeft {
        id,
        nickname: nickname.clone(),
        reason: reason.to_string(),
    };
    let chat_packet = ServerPacket::ChatMessage {
        sender: "Server".to_string(),
        text: format!("{nickname} was kicked: {reason}"),
        is_system: true,
    };
    for other in clients.values_mut() {
        let _ = other.send_packet(&left_packet);
        let _ = other.send_packet(&chat_packet);
    }
    Some(nickname)
}

const OPS_FILE: &str = "ops.txt";

pub(super) fn load_ops(persist_to_disk: bool) -> HashSet<String> {
    if !persist_to_disk {
        return Default::default();
    }
    std::fs::read_to_string(OPS_FILE)
        .map(|text| {
            text.lines()
                .map(|line| line.trim().to_lowercase())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn save_ops(ops: &HashSet<String>, persist_to_disk: bool) {
    if !persist_to_disk {
        return;
    }
    let mut names: Vec<&String> = ops.iter().collect();
    names.sort();
    let text = names
        .iter()
        .map(|n| n.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if let Err(error) = std::fs::write(OPS_FILE, text) {
        crate::server_info!("[SERVER] Could not save {}: {}", OPS_FILE, error);
    }
}

pub(super) fn set_op(
    ops: &mut HashSet<String>,
    name: &str,
    grant: bool,
    persist_to_disk: bool,
) -> bool {
    let key = name.to_lowercase();
    let changed = if grant {
        ops.insert(key)
    } else {
        ops.remove(&key)
    };
    if changed {
        save_ops(ops, persist_to_disk);
    }
    changed
}
