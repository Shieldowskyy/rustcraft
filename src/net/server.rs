//! Dedicated RustCraft multiplayer server with terminal console command management.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, channel, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::net::protocol::*;

#[allow(unused_macros)]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::server_info!($($arg)*)
    };
}

#[allow(unused_macros)]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::server_error!($($arg)*)
    };
}

struct ConnectedClient {
    id: u32,
    stream: TcpStream,
    tx_buffer: Vec<u8>,
    tx_offset: usize,
    output_failed: bool,
    addr: SocketAddr,
    nickname: String,
    authenticated: bool,
    player_data: Option<PlayerData>,
    ping_ms: u32,
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    is_sneaking: bool,
    is_sprinting: bool,
    rx_buffer: Vec<u8>,
    last_ping_sent: Instant,
    pending_ping_seq: Option<u32>,
    next_ping_seq: u32,
}

impl ConnectedClient {
    fn send_packet(&mut self, packet: &ServerPacket) -> bool {
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

    fn flush_outgoing(&mut self) -> bool {
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

/// Configuration for the dedicated server loaded from `server.properties` (or created with defaults).
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
                            "motd" => {
                                cfg.motd = val.to_string();
                            }
                            "max-players" => {
                                if let Ok(m) = val.parse::<usize>() {
                                    cfg.max_players = m;
                                }
                            }
                            "pvp" => {
                                cfg.pvp = val.eq_ignore_ascii_case("true");
                            }
                            "gamemode" => {
                                cfg.gamemode = val.to_string();
                            }
                            "difficulty" => {
                                cfg.difficulty = val.to_string();
                            }
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
                            "logs-dir" => {
                                cfg.logs_dir = val.to_string();
                            }
                            "max-log-size-mb" => {
                                if let Ok(mb) = val.parse::<u64>() {
                                    cfg.max_log_size_mb = mb;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                println!("[SERVER] Loaded configuration from '{}'", path);
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
        println!("[SERVER] Created default configuration file at '{}'", path);
        default_cfg
    }
}

/// Binds to `preferred_port` (default 26656). If occupied, automatically tries next available ports.
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
                    println!("[SERVER] Port {} is in use ({}), checking next available port...", preferred_port, err);
                }
                port += 1;
            }
        }
    }
    Err(format!("Could not bind to any port in range {}-{}", preferred_port, max_port))
}

#[derive(Clone)]
pub struct EmbeddedServerState {
    pub seed: u32,
    pub time_of_day: f32,
    pub spawn_pos: (f32, f32, f32),
    pub blocks: Vec<(i32, i32, i32, u8)>,
    pub mobs: Vec<MobSyncEntry>,
}

fn player_data_path(data_dir: &std::path::Path, nickname: &str) -> std::path::PathBuf {
    let key: String = nickname
        .to_ascii_lowercase()
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    data_dir.join(format!("{key}.dat"))
}

fn is_valid_nickname(nickname: &str) -> bool {
    let nickname = nickname.trim();
    !nickname.is_empty()
        && nickname.len() <= 24
        && nickname
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
}

fn load_player_data(
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
        Ok(_) => Err(format!("Unexpected packet in player data file '{}'", path.display())),
        Err(error) => Err(format!("Could not decode '{}': {error}", path.display())),
    }
}

fn save_player_data(
    data_dir: &std::path::Path,
    nickname: &str,
    data: &PlayerData,
) -> Result<(), String> {
    std::fs::create_dir_all(data_dir)
        .map_err(|error| format!("Could not create '{}': {error}", data_dir.display()))?;
    let path = player_data_path(data_dir, nickname);
    std::fs::write(&path, ClientPacket::PlayerData { data: data.clone() }.serialize())
        .map_err(|error| format!("Could not write '{}': {error}", path.display()))
}

fn persist_client_player_data(client: &mut ConnectedClient) {
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

fn kick_client(
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
    println!("[SERVER] Kicked '{}' (ID {}): {}", nickname, id, reason);

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

pub struct EmbeddedServer {
    pub port: u16,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl EmbeddedServer {
    pub fn start(preferred_port: u16, state: EmbeddedServerState) -> Result<Self, String> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = Arc::clone(&running);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("rustcraft-embedded-server".to_string())
            .spawn(move || {
                run_server(preferred_port, state.seed, Some(state), true, thread_running, Some(ready_tx));
            })
            .map_err(|e| format!("Could not start the multiplayer server: {e}"))?;

        match ready_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(port)) => Ok(Self {
                port,
                running,
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(error) => {
                running.store(false, Ordering::Release);
                let _ = thread.join();
                Err(format!("Multiplayer server did not start: {error}"))
            }
        }
    }
}

impl Drop for EmbeddedServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Dedicated server entrypoint, invoked when rustcraft is started with `--server`.
pub fn run_dedicated_server(preferred_port: u16, seed: u32) {
    let running = Arc::new(AtomicBool::new(true));
    run_server(preferred_port, seed, None, false, running, None);
}

fn run_server(
    preferred_port: u16,
    seed: u32,
    embedded_state: Option<EmbeddedServerState>,
    embedded: bool,
    running: Arc<AtomicBool>,
    ready: Option<mpsc::SyncSender<Result<u16, String>>>,
) {
    let mut config = ServerConfig::load_or_create("server.properties");
    if preferred_port != DEFAULT_SERVER_PORT {
        config.server_port = preferred_port;
    }
    if seed != 0 {
        config.level_seed = seed;
    }
    if let Some(state) = embedded_state.as_ref() {
        config.level_seed = state.seed;
        config.spawn_pos = state.spawn_pos;
    }

    if !embedded {
        let logs_dir = std::env::var("RUSTCRAFT_LOGS_DIR").unwrap_or_else(|_| config.logs_dir.clone());
        let max_log_size_bytes = config.max_log_size_mb * 1024 * 1024;
        let _ = crate::net::server_log::init_server_logger_with_max_size(&logs_dir, max_log_size_bytes);
    }

    println!("============================================================");
    println!("           RustCraft Dedicated Multiplayer Server           ");
    println!("============================================================");

    let (listener, port) = match bind_server_listener(config.server_port) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("[SERVER] FATAL: {}", err);
            if let Some(ready) = ready {
                let _ = ready.send(Err(err));
                return;
            }
            std::process::exit(1);
        }
    };

    if let Some(ready) = ready {
        if ready.send(Ok(port)).is_err() {
            return;
        }
    }

    println!("[SERVER] Successfully bound to 0.0.0.0:{}", port);
    println!("[SERVER] World Seed: {}", config.level_seed);
    println!("[SERVER] MOTD: \"{}\"", config.motd);
    println!("[SERVER] Max Players: {}", config.max_players);
    println!("[SERVER] Default Spawn: ({:.1}, {:.1}, {:.1})", config.spawn_pos.0, config.spawn_pos.1, config.spawn_pos.2);
    println!("[SERVER] Gamemode: {}, PVP: {}, Difficulty: {}", config.gamemode, config.pvp, config.difficulty);
    println!("[SERVER] Type 'help' for a list of terminal console commands.");
    println!("------------------------------------------------------------");

    // Spawn terminal input reader thread
    let (cmd_tx, cmd_rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    if !embedded {
        let r_clone = Arc::clone(&running);
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut line = String::new();
            while r_clone.load(Ordering::Relaxed) {
                line.clear();
                if stdin.read_line(&mut line).is_ok() {
                    let trimmed = line.trim().to_string();
                    if !trimmed.is_empty() {
                        let _ = cmd_tx.send(trimmed);
                    }
                } else {
                    break;
                }
            }
        });
    } else {
        drop(cmd_tx);
    }

    let mut clients: HashMap<u32, ConnectedClient> = HashMap::new();
    let mut next_client_id: u32 = 1;
    let mut modified_blocks: HashMap<(i32, i32, i32), u8> = embedded_state
        .as_ref()
        .map(|state| state.blocks.iter().map(|&(x, y, z, block)| ((x, y, z), block)).collect())
        .unwrap_or_default();
    let mut mob_authority_id = None;
    let mut embedded_host_id = None;
    let mut mob_snapshot = embedded_state
        .as_ref()
        .map(|state| state.mobs.clone())
        .unwrap_or_default();
    let mut world_snapshot = WorldSnapshot::default();
    let mut world_snapshot_initialized = false;
    let mut time_of_day = embedded_state
        .as_ref()
        .map_or(120.0, |state| state.time_of_day);
    let spawn_pos = config.spawn_pos;

    let mut recipe_book = crate::gameplay::recipe::RecipeBook::new();
    crate::engine::modding::ModManager::load_mods(&mut recipe_book);
    let active_server_mods = crate::engine::modding::ModManager::get_active_mod_files();
    if !active_server_mods.is_empty() {
        println!("[SERVER] Active mod(s) enabled for auto-download (Source engine style):");
        for m in &active_server_mods {
            println!("         - {}", m.mod_id);
        }
    }

    let start_time = Instant::now();
    let tick_duration = Duration::from_millis(50); // 20 TPS
    let mut tick_counter: u64 = 0;

    let mut lan_discovery = crate::net::discovery::LanDiscoveryHost::new(
        port,
        &config.motd,
        config.max_players as u32,
    );

    let mut last_fps_check = Instant::now();
    let mut ticks_in_second = 0;
    let mut current_tps = 20.0;

    while running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        // 1. Accept new incoming client TCP connections
        while let Ok((stream, addr)) = listener.accept() {
            if let Err(error) = stream.set_nonblocking(true) {
                crate::log_warn!("SERVER", "Could not configure client socket {addr}: {error}");
                continue;
            }
            let id = next_client_id;
            next_client_id += 1;
            println!("[SERVER] Incoming connection from {} (Assigned Player ID: {})", addr, id);
            clients.insert(
                id,
                ConnectedClient {
                    id,
                    stream,
                    tx_buffer: Vec::new(),
                    tx_offset: 0,
                    output_failed: false,
                    addr,
                    nickname: format!("Player{}", id),
                    authenticated: false,
                    player_data: None,
                    ping_ms: 0,
                    x: spawn_pos.0,
                    y: spawn_pos.1,
                    z: spawn_pos.2,
                    yaw: 0.0,
                    pitch: 0.0,
                    is_sneaking: false,
                    is_sprinting: false,
                    rx_buffer: Vec::with_capacity(1024),
                    last_ping_sent: Instant::now(),
                    pending_ping_seq: None,
                    next_ping_seq: 0,
                },
            );
        }

        // 2. Read incoming packets from connected clients
        let mut disconnected_clients = Vec::new();
        let mut broadcast_packets: Vec<ServerPacket> = Vec::new();
        let mut player_pos_updates: Vec<PlayerPosUpdate> = Vec::new();
        let mut in_game_commands: Vec<(u32, String)> = Vec::new();
        let mut pending_attacks: Vec<(u32, String, (f32, f32, f32), u32, f32)> = Vec::new();
        let mut pending_mob_attacks = Vec::new();
        let mut pending_mob_spawns = Vec::new();
        let mut pending_mob_snapshots = Vec::new();
        let mut pending_mob_broadcasts = Vec::new();
        let mut pending_world_snapshots = Vec::new();
        let mut pending_world_broadcasts = Vec::new();
        let mut authenticated_client_count = clients
            .values()
            .filter(|client| client.authenticated)
            .count();
        let mut authenticated_nicknames: HashSet<String> = clients
            .values()
            .filter(|client| client.authenticated)
            .map(|client| client.nickname.to_lowercase())
            .collect();

        let client_ids: Vec<u32> = clients.keys().copied().collect();
        for id in client_ids {
            let existing_players: Vec<PlayerInfo> = clients
                .values()
                .filter(|client| client.authenticated)
                .map(|client| PlayerInfo {
                    id: client.id,
                    nickname: client.nickname.clone(),
                    ping_ms: client.ping_ms,
                })
                .collect();
            let existing_positions: Vec<PlayerPosUpdate> = clients
                .values()
                .filter(|client| client.authenticated)
                .map(|client| PlayerPosUpdate {
                    id: client.id,
                    x: client.x,
                    y: client.y,
                    z: client.z,
                    yaw: client.yaw,
                    pitch: client.pitch,
                    is_sneaking: client.is_sneaking,
                    is_sprinting: client.is_sprinting,
                })
                .collect();
            let client = clients.get_mut(&id).unwrap();
            if !client.flush_outgoing() {
                disconnected_clients.push((id, "Network write failed".to_string()));
                continue;
            }
            let mut buf = [0u8; 4096];
            match client.stream.read(&mut buf) {
                Ok(0) => {
                    disconnected_clients.push((id, "Connection closed by remote host".to_string()));
                }
                Ok(n) => {
                    client.rx_buffer.extend_from_slice(&buf[..n]);
                    let raw_packets = match extract_packets(&mut client.rx_buffer) {
                        Ok(packets) => packets,
                        Err(error) => {
                            disconnected_clients.push((id, format!("Invalid packet frame: {error}")));
                            continue;
                        }
                    };
                    for raw in raw_packets {
                        match ClientPacket::deserialize(&raw) {
                            Ok(ClientPacket::Handshake { protocol_version, nickname }) => {
                                if protocol_version != PROTOCOL_VERSION {
                                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                        reason: format!(
                                            "Protocol mismatch: server={}, client={}",
                                            PROTOCOL_VERSION, protocol_version
                                        ),
                                    });
                                    disconnected_clients.push((id, "Protocol version mismatch".to_string()));
                                    break;
                                }

                                if client.authenticated {
                                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                        reason: "Handshake already completed".to_string(),
                                    });
                                    disconnected_clients.push((id, "Repeated handshake".to_string()));
                                    break;
                                }
                                if !is_valid_nickname(&nickname) {
                                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                        reason: "Nickname must contain 1-24 letters, numbers, '_' or '-'".to_string(),
                                    });
                                    disconnected_clients.push((id, "Invalid nickname".to_string()));
                                    break;
                                }
                                let nickname = nickname.trim().to_string();
                                if authenticated_nicknames.contains(&nickname.to_lowercase()) {
                                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                        reason: "That nickname is already in use".to_string(),
                                    });
                                    disconnected_clients.push((id, "Nickname already in use".to_string()));
                                    break;
                                }
                                if authenticated_client_count >= config.max_players {
                                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                        reason: format!("Server is full! (max players: {})", config.max_players),
                                    });
                                    disconnected_clients.push((id, "Server full".to_string()));
                                    break;
                                }

                                client.nickname = nickname.clone();
                                client.authenticated = true;
                                authenticated_client_count += 1;
                                authenticated_nicknames.insert(nickname.to_lowercase());
                                if embedded && embedded_host_id.is_none() {
                                    embedded_host_id = Some(id);
                                }
                                if embedded && embedded_host_id == Some(id) {
                                    client.player_data = None;
                                } else {
                                    match load_player_data(std::path::Path::new("playerdata"), &nickname) {
                                        Ok(data) => {
                                            if let Some(data) = &data {
                                                client.x = data.x;
                                                client.y = data.y;
                                                client.z = data.z;
                                            }
                                            client.player_data = data;
                                        }
                                        Err(error) => {
                                            crate::log_warn!("SERVER", "Could not load player data for '{}': {}", nickname, error);
                                            client.player_data = None;
                                        }
                                    }
                                }
                                if mob_authority_id.is_none() {
                                    mob_authority_id = Some(id);
                                }
                                println!("[SERVER] Player '{}' (ID {}) logged in from {}", nickname, id, client.addr);

                                // 1. Synchronize active server mods (Source engine style auto-download)
                                let _ = client.send_packet(&ServerPacket::ModSync {
                                    mods: active_server_mods.clone(),
                                });

                                // 2. Send LoginSuccess with seed, time, and all modified blocks
                                let block_list: Vec<(i32, i32, i32, u8)> = modified_blocks
                                    .iter()
                                    .map(|(&(bx, by, bz), &blk)| (bx, by, bz, blk))
                                    .collect();

                                let login_success = ServerPacket::LoginSuccess {
                                    player_id: id,
                                    seed: config.level_seed,
                                    spawn_x: spawn_pos.0,
                                    spawn_y: spawn_pos.1,
                                    spawn_z: spawn_pos.2,
                                    time_of_day,
                                    modified_blocks: block_list,
                                };
                                let _ = client.send_packet(&login_success);
                                let mut players = existing_players.clone();
                                players.push(PlayerInfo {
                                    id,
                                    nickname: nickname.clone(),
                                    ping_ms: client.ping_ms,
                                });
                                let _ = client.send_packet(&ServerPacket::PlayerList { players });
                                let mut positions = existing_positions.clone();
                                positions.push(PlayerPosUpdate {
                                    id,
                                    x: client.x,
                                    y: client.y,
                                    z: client.z,
                                    yaw: client.yaw,
                                    pitch: client.pitch,
                                    is_sneaking: client.is_sneaking,
                                    is_sprinting: client.is_sprinting,
                                });
                                let _ = client.send_packet(&ServerPacket::PlayerPositions {
                                    updates: positions,
                                });
                                if let Some(authority_id) = mob_authority_id {
                                    let _ = client.send_packet(&ServerPacket::MobAuthority { id: authority_id });
                                }
                                if mob_authority_id != Some(id) {
                                    let _ = client.send_packet(&ServerPacket::MobSnapshot {
                                        mobs: mob_snapshot.clone(),
                                    });
                                }
                                if world_snapshot_initialized {
                                    let _ = client.send_packet(&ServerPacket::WorldSnapshot {
                                        snapshot: world_snapshot.clone(),
                                    });
                                }
                                let _ = client.send_packet(&ServerPacket::PlayerData {
                                    data: client.player_data.clone(),
                                });

                                // Send MOTD
                                if !config.motd.is_empty() {
                                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                                        sender: "Server".to_string(),
                                        text: config.motd.clone(),
                                        is_system: true,
                                    });
                                }

                                broadcast_packets.push(ServerPacket::PlayerJoined {
                                    id,
                                    nickname: nickname.clone(),
                                });

                                broadcast_packets.push(ServerPacket::ChatMessage {
                                    sender: "Server".to_string(),
                                    text: format!("{} joined the game", nickname),
                                    is_system: true,
                                });
                            }
                            Ok(ClientPacket::PlayerMove { x, y, z, yaw, pitch, is_sneaking, is_sprinting }) => {
                                if !client.authenticated
                                    || ![x, y, z, yaw, pitch].iter().all(|value| value.is_finite())
                                {
                                    disconnected_clients.push((id, "Invalid player movement".to_string()));
                                    break;
                                }
                                client.x = x;
                                client.y = y;
                                client.z = z;
                                client.yaw = yaw;
                                client.pitch = pitch;
                                client.is_sneaking = is_sneaking;
                                client.is_sprinting = is_sprinting;
                                if let Some(data) = &mut client.player_data {
                                    data.x = x;
                                    data.y = y;
                                    data.z = z;
                                }
                                player_pos_updates.push(PlayerPosUpdate {
                                    id,
                                    x,
                                    y,
                                    z,
                                    yaw,
                                    pitch,
                                    is_sneaking,
                                    is_sprinting,
                                });
                            }
                            Ok(ClientPacket::BlockChange { x, y, z, block }) => {
                                if !client.authenticated || !(0..crate::world::CHUNK_SIZE_Y).contains(&y) {
                                    disconnected_clients.push((id, "Invalid block change".to_string()));
                                    break;
                                }
                                modified_blocks.insert((x, y, z), block);
                                println!(
                                    "[SERVER] Block change from '{}' at ({}, {}, {}) -> BlockId {}",
                                    client.nickname, x, y, z, block
                                );
                                broadcast_packets.push(ServerPacket::BlockChange { x, y, z, block });
                            }
                            Ok(ClientPacket::ChatMessage { text }) => {
                                if !client.authenticated {
                                    disconnected_clients.push((id, "Login required".to_string()));
                                    break;
                                }
                                if text.starts_with('/') {
                                    in_game_commands.push((id, text));
                                } else {
                                    println!("[CHAT] <{}> {}", client.nickname, text);
                                    broadcast_packets.push(ServerPacket::ChatMessage {
                                        sender: client.nickname.clone(),
                                        text,
                                        is_system: false,
                                    });
                                }
                            }
                            Ok(ClientPacket::Ping { seq }) => {
                                let _ = client.send_packet(&ServerPacket::Pong { seq });
                            }
                            Ok(ClientPacket::Pong { seq }) => {
                                if client.pending_ping_seq == Some(seq) {
                                    client.ping_ms = client
                                        .last_ping_sent
                                        .elapsed()
                                        .as_millis()
                                        .clamp(1, u32::MAX as u128)
                                        as u32;
                                    client.pending_ping_seq = None;
                                }
                            }
                            Ok(ClientPacket::Disconnect { reason }) => {
                                disconnected_clients.push((id, reason));
                                break;
                            }
                            Ok(ClientPacket::AttackPlayer { target_id, damage }) => {
                                if !client.authenticated {
                                    disconnected_clients.push((id, "Login required".to_string()));
                                    break;
                                } else if !config.pvp {
                                    println!("[SERVER] Attack from '{}' on player ID {} ignored (pvp=false)", client.nickname, target_id);
                                } else {
                                    let attacker_pos = (client.x, client.y, client.z);
                                    let attacker_name = client.nickname.clone();
                                    pending_attacks.push((id, attacker_name, attacker_pos, target_id, damage));
                                }
                            }
                            Ok(ClientPacket::AttackMob { mob_id, damage }) => {
                                if !client.authenticated
                                    || !damage.is_finite()
                                    || damage <= 0.0
                                    || damage > 50.0
                                {
                                    disconnected_clients.push((id, "Invalid mob attack".to_string()));
                                    break;
                                }
                                pending_mob_attacks.push((id, mob_id, damage, client.yaw, client.pitch));
                            }
                            Ok(ClientPacket::SpawnMob { kind, x, y, z }) => {
                                if !client.authenticated
                                    || crate::gameplay::mob::MobKind::ALL.get(kind as usize).is_none()
                                    || ![x, y, z].iter().all(|coordinate| coordinate.is_finite())
                                {
                                    disconnected_clients.push((id, "Invalid mob spawn request".to_string()));
                                    break;
                                }
                                pending_mob_spawns.push((id, kind, x, y, z));
                            }
                            Ok(ClientPacket::MobSnapshot { mobs }) => {
                                if client.authenticated {
                                    pending_mob_snapshots.push((id, mobs));
                                } else {
                                    disconnected_clients.push((id, "Login required".to_string()));
                                    break;
                                }
                            }
                            Ok(ClientPacket::WorldSnapshot { snapshot }) => {
                                if client.authenticated {
                                    pending_world_snapshots.push((id, snapshot));
                                } else {
                                    disconnected_clients.push((id, "Login required".to_string()));
                                    break;
                                }
                            }
                            Ok(ClientPacket::PlayerDeath) => {
                                if client.authenticated {
                                    broadcast_packets.push(ServerPacket::ChatMessage {
                                        sender: "Server".to_string(),
                                        text: format!("{} died", client.nickname),
                                        is_system: true,
                                    });
                                }
                            }
                            Ok(ClientPacket::PlayerData { data }) => {
                                if !client.authenticated
                                    || ![data.x, data.y, data.z].iter().all(|value| value.is_finite())
                                {
                                    disconnected_clients.push((id, "Invalid player data".to_string()));
                                    break;
                                }
                                client.x = data.x;
                                client.y = data.y;
                                client.z = data.z;
                                client.player_data = Some(data.clone());
                                if let Err(error) =
                                    save_player_data(std::path::Path::new("playerdata"), &client.nickname, &data)
                                {
                                    crate::log_error!("SERVER", "Could not save player data for '{}': {}", client.nickname, error);
                                }
                            }
                            Err(err) => {
                                eprintln!("[SERVER] Corrupted packet from ID {}: {}", id, err);
                                disconnected_clients.push((id, format!("Invalid packet: {err}")));
                                break;
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data pending on non-blocking socket
                }
                Err(e) => {
                    disconnected_clients.push((id, format!("I/O Error: {}", e)));
                }
            }
        }

        for (sender_id, snapshot) in pending_mob_snapshots {
            if mob_authority_id == Some(sender_id) {
                mob_snapshot = snapshot.clone();
                pending_mob_broadcasts.push((
                    sender_id,
                    ServerPacket::MobSnapshot { mobs: snapshot },
                ));
            }
        }
        for (attacker_id, mob_id, damage, yaw, pitch) in pending_mob_attacks {
            let Some(authority_id) = mob_authority_id.filter(|authority_id| *authority_id != attacker_id) else {
                continue;
            };
            let Some(mob) = mob_snapshot.iter().find(|mob| mob.id == mob_id) else {
                continue;
            };
            let Some(attacker) = clients.get(&attacker_id) else {
                continue;
            };
            let dx = mob.x - attacker.x;
            let dy = mob.y - attacker.y;
            let dz = mob.z - attacker.z;
            if dx * dx + dy * dy + dz * dz > 36.0 {
                continue;
            }
            let (sin_yaw, cos_yaw) = yaw.sin_cos();
            let (sin_pitch, cos_pitch) = pitch.sin_cos();
            if let Some(authority) = clients.get_mut(&authority_id) {
                let _ = authority.send_packet(&ServerPacket::MobAttack {
                    attacker_id,
                    mob_id,
                    damage,
                    direction_x: sin_yaw * cos_pitch,
                    direction_y: -sin_pitch,
                    direction_z: cos_yaw * cos_pitch,
                });
            }
        }
        for (requester_id, kind, x, y, z) in pending_mob_spawns {
            let Some(authority_id) =
                mob_authority_id.filter(|authority_id| *authority_id != requester_id)
            else {
                continue;
            };
            let Some(requester) = clients.get(&requester_id) else {
                continue;
            };
            let dx = x - requester.x;
            let dy = y - requester.y;
            let dz = z - requester.z;
            if dx * dx + dy * dy + dz * dz > 144.0 {
                continue;
            }
            if let Some(authority) = clients.get_mut(&authority_id) {
                let _ = authority.send_packet(&ServerPacket::MobSpawn { kind, x, y, z });
            }
        }
        for (sender_id, snapshot) in pending_world_snapshots {
            if mob_authority_id == Some(sender_id) {
                world_snapshot = snapshot.clone();
                world_snapshot_initialized = true;
                pending_world_broadcasts.push((
                    sender_id,
                    ServerPacket::WorldSnapshot { snapshot },
                ));
            }
        }

        // 2a. Process PvP attacks
        for (attacker_id, attacker_name, (ax, ay, az), target_id, damage) in pending_attacks {
            if let Some(target_client) = clients.get_mut(&target_id) {
                let dx = target_client.x - ax;
                let dy = target_client.y - ay;
                let dz = target_client.z - az;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if dist_sq <= 36.0 {
                    let h_len = (dx * dx + dz * dz).sqrt();
                    let (kb_x, kb_z) = if h_len > 0.001 {
                        (dx / h_len * 0.45, dz / h_len * 0.45)
                    } else {
                        (0.0, 0.45)
                    };
                    println!(
                        "[SERVER PVP] Player '{}' (ID {}) attacked '{}' (ID {}) for {:.1} dmg",
                        attacker_name, attacker_id, target_client.nickname, target_id, damage
                    );
                    broadcast_packets.push(ServerPacket::PlayerHurt {
                        victim_id: target_id,
                        attacker_id,
                        damage,
                        knockback_x: kb_x,
                        knockback_z: kb_z,
                    });
                }
            }
        }

        // 2b. Process queued in-game chat commands
        for (sender_id, cmd_text) in in_game_commands {
            handle_in_game_command(
                &cmd_text,
                sender_id,
                embedded_host_id,
                &mut clients,
                &mut broadcast_packets,
                &mut time_of_day,
                &config,
            );
        }

        // 3. Handle disconnected clients
        for (id, reason) in disconnected_clients {
            if let Some(mut client) = clients.remove(&id) {
                let _ = client.flush_outgoing();
                let was_authenticated = client.authenticated;
                if was_authenticated {
                    persist_client_player_data(&mut client);
                }
                if mob_authority_id == Some(id) {
                    mob_authority_id = None;
                }
                println!("[SERVER] Player '{}' (ID {}) disconnected: {}", client.nickname, id, reason);
                if was_authenticated {
                    broadcast_packets.push(ServerPacket::PlayerLeft {
                        id,
                        nickname: client.nickname.clone(),
                        reason,
                    });
                    broadcast_packets.push(ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: format!("{} left the game", client.nickname),
                        is_system: true,
                    });
                }
            }
        }

        if mob_authority_id.is_none() {
            mob_authority_id = clients
                .values()
                .filter(|client| client.authenticated)
                .map(|client| client.id)
                .min();
            if let Some(id) = mob_authority_id {
                broadcast_packets.push(ServerPacket::MobAuthority { id });
            }
        }

        // 4. Broadcast queued packets
        if !broadcast_packets.is_empty() {
            for packet in &broadcast_packets {
                for client in clients.values_mut() {
                    let _ = client.send_packet(packet);
                }
            }
        }
        for (sender_id, packet) in pending_mob_broadcasts {
            for client in clients.values_mut() {
                if client.id != sender_id {
                    let _ = client.send_packet(&packet);
                }
            }
        }
        for (sender_id, packet) in pending_world_broadcasts {
            for client in clients.values_mut() {
                if client.id != sender_id {
                    let _ = client.send_packet(&packet);
                }
            }
        }

        // 5. Broadcast player positions every tick
        if !player_pos_updates.is_empty() && clients.len() > 1 {
            let pos_packet = ServerPacket::PlayerPositions {
                updates: player_pos_updates,
            };
            for client in clients.values_mut() {
                let _ = client.send_packet(&pos_packet);
            }
        }

        for client in clients.values_mut() {
            if client.pending_ping_seq.is_some()
                && client.last_ping_sent.elapsed() >= Duration::from_secs(3)
            {
                client.pending_ping_seq = None;
            }
            if client.authenticated
                && client.pending_ping_seq.is_none()
                && client.last_ping_sent.elapsed() >= Duration::from_secs(1)
            {
                let seq = client.next_ping_seq;
                client.next_ping_seq = client.next_ping_seq.wrapping_add(1);
                client.pending_ping_seq = Some(seq);
                client.last_ping_sent = Instant::now();
                if !client.send_packet(&ServerPacket::Ping { seq }) {
                    crate::log_warn!("SERVER", "Failed to send ping to '{}'", client.nickname);
                }
            }
        }

        // 6. Periodic heartbeat & player list update (every 20 ticks / ~1 second)
        tick_counter += 1;
        if tick_counter % 20 == 0 {
            time_of_day = (time_of_day + 1.0).rem_euclid(480.0);
            let time_sync = ServerPacket::TimeSync { time_of_day };

            let player_list: Vec<PlayerInfo> = clients
                .values()
                .filter(|client| client.authenticated)
                .map(|c| PlayerInfo {
                    id: c.id,
                    nickname: c.nickname.clone(),
                    ping_ms: c.ping_ms,
                })
                .collect();
            let list_packet = ServerPacket::PlayerList {
                players: player_list,
            };

            for client in clients.values_mut().filter(|client| client.authenticated) {
                let _ = client.send_packet(&time_sync);
                let _ = client.send_packet(&list_packet);
            }
        }

        // 7. Process terminal console commands
        while let Ok(cmd_line) = cmd_rx.try_recv() {
            handle_console_command(
                &cmd_line,
                &mut clients,
                &mut modified_blocks,
                &mut time_of_day,
                seed,
                current_tps,
                start_time.elapsed(),
                &running,
            );
        }

        // 8. LAN local network discovery updates and client probe pings
        lan_discovery.update(
            clients
                .values()
                .filter(|client| client.authenticated)
                .count() as u32,
        );

        // TPS Tracking
        ticks_in_second += 1;
        if last_fps_check.elapsed() >= Duration::from_secs(1) {
            current_tps = ticks_in_second as f32 / last_fps_check.elapsed().as_secs_f32();
            ticks_in_second = 0;
            last_fps_check = Instant::now();
        }

        // Maintain fixed tick rate
        let elapsed = frame_start.elapsed();
        if elapsed < tick_duration {
            std::thread::sleep(tick_duration - elapsed);
        }
    }

    println!("[SERVER] Server shutting down... Disconnecting all active players.");
    for client in clients.values_mut() {
        let _ = client.send_packet(&ServerPacket::LoginDisconnect {
            reason: "Server shut down by host console".to_string(),
        });
        let _ = client.flush_outgoing();
    }
    println!("[SERVER] Goodbye!");
    if !embedded {
        crate::net::server_log::shutdown_server_logger();
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_console_command(
    cmd_line: &str,
    clients: &mut HashMap<u32, ConnectedClient>,
    modified_blocks: &mut HashMap<(i32, i32, i32), u8>,
    time_of_day: &mut f32,
    seed: u32,
    tps: f32,
    uptime: Duration,
    running: &Arc<AtomicBool>,
) {
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "help" => {
            println!("------------------- Server Commands -------------------");
            println!("  help                     - Show this list of console commands");
            println!("  list                     - List all connected players and their ping");
            println!("  say <message>            - Broadcast chat message to all players");
            println!("  tell <player> <message>  - Whisper a private message to a player");
            println!("  kick <player> [reason]   - Disconnect a player from the server");
            println!("  time [day|night|set <v>] - Display or change world time of day");
            println!("  seed                     - Display the world generation seed");
            println!("  status                   - Display server TPS, uptime, and stats");
            println!("  tp <p1> <p2> / tp <p> x y z - Teleport a player");
            println!("  stop | exit              - Gracefully shut down the server");
            println!("-------------------------------------------------------");
        }
        "list" => {
            println!("[SERVER] Connected Players ({}/16):", clients.len());
            if clients.is_empty() {
                println!("  (No players online)");
            } else {
                for client in clients.values() {
                    println!(
                        "  - {} (ID: {}, IP: {}, Pos: ({:.1}, {:.1}, {:.1}), Ping: {}ms)",
                        client.nickname, client.id, client.addr, client.x, client.y, client.z, client.ping_ms
                    );
                }
            }
        }
        "say" => {
            if parts.len() < 2 {
                println!("[SERVER] Usage: say <message>");
            } else {
                let msg = parts[1..].join(" ");
                println!("[SERVER SAY] {}", msg);
                let packet = ServerPacket::ChatMessage {
                    sender: "[Server]".to_string(),
                    text: msg,
                    is_system: true,
                };
                for client in clients.values_mut() {
                    let _ = client.send_packet(&packet);
                }
            }
        }
        "tell" | "msg" | "w" => {
            if parts.len() < 3 {
                println!("[SERVER] Usage: tell <player> <message>");
            } else {
                let target_name = parts[1].to_lowercase();
                let msg = parts[2..].join(" ");
                let mut found = false;
                for client in clients.values_mut() {
                    if client.nickname.to_lowercase() == target_name {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "[Server Whisper]".to_string(),
                            text: msg.clone(),
                            is_system: true,
                        });
                        println!("[SERVER] Whispered to {}: {}", client.nickname, msg);
                        found = true;
                        break;
                    }
                }
                if !found {
                    println!("[SERVER] Player '{}' not found", parts[1]);
                }
            }
        }
        "kick" => {
            if parts.len() < 2 {
                println!("[SERVER] Usage: kick <player> [reason]");
            } else {
                let target_name = parts[1].to_lowercase();
                let reason = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    "Kicked by server operator".to_string()
                };
                let mut kick_id = None;
                for client in clients.values() {
                    if client.nickname.to_lowercase() == target_name {
                        kick_id = Some(client.id);
                        break;
                    }
                }
                if let Some(id) = kick_id {
                    kick_client(clients, id, &reason);
                } else {
                    println!("[SERVER] Player '{}' not found", parts[1]);
                }
            }
        }
        "seed" => {
            println!("[SERVER] World Seed: {}", seed);
        }
        "time" => {
            if parts.len() == 1 {
                println!("[SERVER] Current time of day: {:.1} / 480.0", *time_of_day);
            } else {
                match parts[1].to_lowercase().as_str() {
                    "day" => {
                        *time_of_day = 120.0;
                        println!("[SERVER] Time set to Day (120.0)");
                    }
                    "night" => {
                        *time_of_day = 360.0;
                        println!("[SERVER] Time set to Night (360.0)");
                    }
                    "set" if parts.len() >= 3 => {
                        if let Ok(val) = parts[2].parse::<f32>() {
                            *time_of_day = val.rem_euclid(480.0);
                            println!("[SERVER] Time set to {:.1}", *time_of_day);
                        } else {
                            println!("[SERVER] Invalid time value: {}", parts[2]);
                        }
                    }
                    _ => {
                        println!("[SERVER] Usage: time [day|night|set <number>]");
                        return;
                    }
                }
                let sync = ServerPacket::TimeSync { time_of_day: *time_of_day };
                for client in clients.values_mut() {
                    let _ = client.send_packet(&sync);
                }
            }
        }
        "status" => {
            let secs = uptime.as_secs();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let s = secs % 60;
            println!("------------------- Server Status -------------------");
            println!("  TPS:             {:.1} / 20.0", tps);
            println!("  Uptime:          {:02}:{:02}:{:02}", hours, mins, s);
            println!("  Online Players:  {}/16", clients.len());
            println!("  Modified Blocks: {}", modified_blocks.len());
            println!("  Time of Day:     {:.1}s", *time_of_day);
            println!("-----------------------------------------------------");
        }
        "tp" => {
            if parts.len() == 5 {
                let p_name = parts[1].to_lowercase();
                if let (Ok(x), Ok(y), Ok(z)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>(), parts[4].parse::<f32>()) {
                    let mut found = false;
                    for client in clients.values_mut() {
                        if client.nickname.to_lowercase() == p_name {
                            client.x = x;
                            client.y = y;
                            client.z = z;
                            let _ = client.send_packet(&ServerPacket::Teleport { x, y, z });
                            println!("[SERVER] Teleported {} to ({:.1}, {:.1}, {:.1})", client.nickname, x, y, z);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        println!("[SERVER] Player '{}' not found", parts[1]);
                    }
                } else {
                    println!("[SERVER] Coordinates must be numbers: tp <player> <x> <y> <z>");
                }
            } else if parts.len() == 3 {
                let p1_name = parts[1].to_lowercase();
                let p2_name = parts[2].to_lowercase();
                let target_pos = clients.values().find(|c| c.nickname.to_lowercase() == p2_name).map(|c| (c.x, c.y, c.z));
                if let Some((x, y, z)) = target_pos {
                    if let Some(c) = clients.values_mut().find(|c| c.nickname.to_lowercase() == p1_name) {
                        c.x = x;
                        c.y = y;
                        c.z = z;
                        let _ = c.send_packet(&ServerPacket::Teleport { x, y, z });
                        println!("[SERVER] Teleported {} to {}", c.nickname, parts[2]);
                    } else {
                        println!("[SERVER] Player '{}' not found", parts[1]);
                    }
                } else {
                    println!("[SERVER] Target player '{}' not found", parts[2]);
                }
            } else {
                println!("[SERVER] Usage: tp <player> <x> <y> <z>  OR  tp <player1> <player2>");
            }
        }
        "stop" | "exit" | "quit" => {
            println!("[SERVER] Initiating server shutdown...");
            running.store(false, Ordering::Relaxed);
        }
        unknown => {
            println!("[SERVER] Unknown command: '{}'. Type 'help' for commands.", unknown);
        }
    }
}

fn parse_time_value(value: &str) -> Option<f32> {
    match value.to_ascii_lowercase().as_str() {
        "day" | "1000" => Some(120.0),
        "noon" | "6000" => Some(240.0),
        "night" | "13000" => Some(360.0),
        "midnight" | "18000" => Some(0.0),
        ticks => {
            let ticks = ticks.parse::<f32>().ok()?;
            ticks.is_finite().then(|| ticks.rem_euclid(24_000.0) / 24_000.0 * 480.0)
        }
    }
}

fn handle_in_game_command(
    cmd_line: &str,
    sender_id: u32,
    embedded_host_id: Option<u32>,
    clients: &mut HashMap<u32, ConnectedClient>,
    broadcast_packets: &mut Vec<ServerPacket>,
    time_of_day: &mut f32,
    config: &ServerConfig,
) {
    let sender_nick = clients.get(&sender_id).map(|c| c.nickname.clone()).unwrap_or_default();
    let trimmed = cmd_line.trim_start_matches('/');
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "help" => {
            if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: "Available commands: /help, /list, /time set <day|noon|night|midnight|number>, /seed, /spawn, /tp <player>, /tell <player> <msg>, /kick <player> [reason] (host only)".to_string(),
                    is_system: true,
                });
            }
        }
        "kick" => {
            if embedded_host_id != Some(sender_id) {
                if let Some(client) = clients.get_mut(&sender_id) {
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: "Only the multiplayer host can kick players.".to_string(),
                        is_system: true,
                    });
                }
                return;
            }
            if parts.len() < 2 {
                if let Some(client) = clients.get_mut(&sender_id) {
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: "Usage: /kick <player> [reason]".to_string(),
                        is_system: true,
                    });
                }
                return;
            }

            let target = clients
                .values()
                .find(|client| client.nickname.eq_ignore_ascii_case(parts[1]))
                .map(|client| client.id);
            let reason = if parts.len() > 2 {
                parts[2..].join(" ")
            } else {
                format!("Kicked by host ({sender_nick})")
            };
            if let Some(target_id) = target {
                if let Some(target_name) = kick_client(clients, target_id, &reason) {
                    if let Some(client) = clients.get_mut(&sender_id) {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "Server".to_string(),
                            text: format!("Kicked {target_name}: {reason}"),
                            is_system: true,
                        });
                    }
                }
            } else if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: format!("Player '{}' not found", parts[1]),
                    is_system: true,
                });
            }
        }
        "list" => {
            let names: Vec<String> = clients.values().map(|c| format!("{} ({}ms)", c.nickname, c.ping_ms)).collect();
            let msg = format!("Online ({}/{}): {}", clients.len(), config.max_players, names.join(", "));
            if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: msg,
                    is_system: true,
                });
            }
        }
        "seed" => {
            let msg = format!("World Seed: {}", config.level_seed);
            if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: msg,
                    is_system: true,
                });
            }
        }
        "spawn" => {
            if let Some(client) = clients.get_mut(&sender_id) {
                client.x = config.spawn_pos.0;
                client.y = config.spawn_pos.1;
                client.z = config.spawn_pos.2;
                let _ = client.send_packet(&ServerPacket::Teleport {
                    x: config.spawn_pos.0,
                    y: config.spawn_pos.1,
                    z: config.spawn_pos.2,
                });
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: "Teleported to spawn point".to_string(),
                    is_system: true,
                });
            }
        }
        "tp" | "teleport" => {
            if parts.len() != 2 {
                if let Some(client) = clients.get_mut(&sender_id) {
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: "Usage: /tp <player>".to_string(),
                        is_system: true,
                    });
                }
                return;
            }

            let target = clients
                .values()
                .find(|client| client.nickname.eq_ignore_ascii_case(parts[1]))
                .map(|client| (client.x, client.y, client.z, client.nickname.clone()));
            if let Some((x, y, z, target_name)) = target {
                if let Some(client) = clients.get_mut(&sender_id) {
                    client.x = x;
                    client.y = y;
                    client.z = z;
                    let _ = client.send_packet(&ServerPacket::Teleport { x, y, z });
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: format!("Teleported to {}", target_name),
                        is_system: true,
                    });
                }
            } else if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: format!("Player '{}' not found", parts[1]),
                    is_system: true,
                });
            }
        }
        "time" => {
            let args = &parts[1..];
            if args.first().is_some_and(|arg| arg.eq_ignore_ascii_case("query")) {
                if let Some(client) = clients.get_mut(&sender_id) {
                    let ticks = (*time_of_day / 480.0 * 24_000.0).round() as u32;
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: format!("The time is {} ticks", ticks),
                        is_system: true,
                    });
                }
                return;
            }

            let value = match args {
                [value] if !value.eq_ignore_ascii_case("set") => Some(*value),
                [set, value] if set.eq_ignore_ascii_case("set") => Some(*value),
                _ => None,
            };
            if let Some(value) = value.and_then(parse_time_value) {
                *time_of_day = value;
                broadcast_packets.push(ServerPacket::TimeSync { time_of_day: value });
                broadcast_packets.push(ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: format!("{} set the time to {:.0} ticks", sender_nick, value / 480.0 * 24_000.0),
                    is_system: true,
                });
            } else if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: "Usage: /time set <day|noon|night|midnight|number>".to_string(),
                    is_system: true,
                });
            }
        }
        "tell" | "w" | "msg" => {
            if parts.len() >= 3 {
                let target_name = parts[1].to_lowercase();
                let whisper = parts[2..].join(" ");
                let mut found = false;
                for (other_id, other) in clients.iter_mut() {
                    if *other_id != sender_id && other.nickname.to_lowercase() == target_name {
                        let _ = other.send_packet(&ServerPacket::ChatMessage {
                            sender: format!("{} -> you", sender_nick),
                            text: whisper.clone(),
                            is_system: false,
                        });
                        found = true;
                        break;
                    }
                }

                if let Some(client) = clients.get_mut(&sender_id) {
                    if found {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: format!("you -> {}", parts[1]),
                            text: whisper,
                            is_system: false,
                        });
                    } else {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "Server".to_string(),
                            text: format!("Player '{}' not found", parts[1]),
                            is_system: true,
                        });
                    }
                }
            } else if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: "Usage: /tell <player> <message>".to_string(),
                    is_system: true,
                });
            }
        }
        unknown => {
            if let Some(client) = clients.get_mut(&sender_id) {
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: format!("Unknown command: /{}. Type /help for a list of commands.", unknown),
                    is_system: true,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn player_data_is_persisted_by_case_insensitive_nickname() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let data_dir = std::env::temp_dir().join(format!(
            "rustcraft_playerdata_{}_{}",
            std::process::id(),
            unique
        ));
        let data = PlayerData {
            x: 12.5,
            y: 70.0,
            z: -6.0,
            health: 16.0,
            max_health: 20.0,
            hunger: 14.0,
            max_hunger: 20.0,
            saturation: 2.0,
            exhaustion: 1.0,
            air: 18.0,
            selected_hotbar_slot: 2,
            hotbar: [Some(PlayerItemStack {
                block: crate::world::BlockId::Diamond.to_u8(),
                count: 5,
                durability: 42,
            }); 10],
            main: [None; 27],
            armor: [None; 4],
            offhand: None,
            spawn_point: Some((3, 64, 8)),
        };

        save_player_data(&data_dir, "RelayUser", &data).unwrap();
        assert_eq!(load_player_data(&data_dir, "relayuser").unwrap(), Some(data));
        assert_eq!(load_player_data(&data_dir, "AnotherUser").unwrap(), None);
        std::fs::remove_file(player_data_path(&data_dir, "RelayUser")).unwrap();
        std::fs::remove_dir(&data_dir).unwrap();
    }

    #[test]
    fn server_updates_player_list_with_measured_ping() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let mut client = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "PingTest",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && client.ping_ms == 0 {
            client.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }

        assert!(client.is_connected);
        assert!(client.ping_ms > 0);
        client.disconnect();
        drop(server);
    }

    #[test]
    fn non_authority_mob_attacks_are_sent_to_the_mob_authority() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 48.0, 8.5),
                blocks: Vec::new(),
                mobs: vec![MobSyncEntry {
                    id: 7,
                    kind: crate::gameplay::mob::MobKind::Zombie as u8,
                    x: 9.0,
                    y: 48.0,
                    z: 8.5,
                    yaw: 0.0,
                    health: 20.0,
                    is_dead: false,
                    walk_phase: 0.0,
                    is_baby: false,
                    wool_color: 0,
                    is_sheared: false,
                    is_puffed: false,
                    fuse: 0.0,
                    is_ignited: false,
                }],
            },
        )
        .unwrap();
        let address = format!("127.0.0.1:{}", server.port);
        let mut authority = crate::net::NetworkClient::connect(&address, "Authority").unwrap();
        let mut attacker = crate::net::NetworkClient::connect(&address, "Attacker").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline
            && (!authority.is_connected
                || !attacker.is_connected
                || attacker.mob_authority_id != authority.local_id)
        {
            authority.update(0.05);
            attacker.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(authority.is_connected && attacker.is_connected);
        assert_eq!(attacker.mob_authority_id, authority.local_id);

        attacker.send_attack_mob(7, 6.5);
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut replicated = false;
        while Instant::now() < deadline && !replicated {
            replicated |= authority.update(0.05).into_iter().any(|event| {
                matches!(
                    event,
                    crate::net::NetworkEvent::MobAttack {
                        attacker_id,
                        mob_id: 7,
                        damage: 6.5,
                        ..
                    } if Some(attacker_id) == attacker.local_id
                )
            });
            attacker.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(replicated, "mob authority should receive the remote attack");

        authority.disconnect();
        attacker.disconnect();
        drop(server);
    }

    #[test]
    fn non_authority_mob_spawns_are_sent_to_the_mob_authority() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let address = format!("127.0.0.1:{}", server.port);
        let mut authority = crate::net::NetworkClient::connect(&address, "SpawnAuthority").unwrap();
        let mut guest = crate::net::NetworkClient::connect(&address, "SpawnGuest").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline
            && (!authority.is_connected
                || !guest.is_connected
                || guest.mob_authority_id != authority.local_id)
        {
            authority.update(0.05);
            guest.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(authority.is_connected && guest.is_connected);
        assert_eq!(guest.mob_authority_id, authority.local_id);

        let spawn_pos = crate::core::math::Vec3::new(9.0, 65.0, 8.5);
        guest.send_spawn_mob(crate::gameplay::mob::MobKind::Zombie, spawn_pos);
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut received = false;
        while Instant::now() < deadline && !received {
            received |= authority.update(0.05).into_iter().any(|event| {
                matches!(
                    event,
                    crate::net::NetworkEvent::MobSpawn {
                        kind,
                        pos,
                    } if kind == crate::gameplay::mob::MobKind::Zombie as u8 && pos == spawn_pos
                )
            });
            guest.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(received, "mob authority should receive the client's spawn request");

        authority.disconnect();
        guest.disconnect();
        drop(server);
    }

    #[test]
    fn in_game_time_set_command_syncs_all_clients() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 42.0,
                spawn_pos: (8.5, 48.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let address = format!("127.0.0.1:{}", server.port);
        let mut host = crate::net::NetworkClient::connect(&address, "TimeHost").unwrap();
        let mut guest = crate::net::NetworkClient::connect(&address, "TimeGuest").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && (!host.is_connected || !guest.is_connected) {
            host.update(0.05);
            guest.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(host.is_connected && guest.is_connected);

        guest.send_chat("/time set day");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut host_synced = false;
        let mut guest_synced = false;
        while Instant::now() < deadline && !(host_synced && guest_synced) {
            host_synced |= host.update(0.05).into_iter().any(|event| {
                matches!(event, crate::net::NetworkEvent::TimeSync { time_of_day } if time_of_day == 120.0)
            });
            guest_synced |= guest.update(0.05).into_iter().any(|event| {
                matches!(event, crate::net::NetworkEvent::TimeSync { time_of_day } if time_of_day == 120.0)
            });
            thread::sleep(Duration::from_millis(10));
        }
        assert!(host_synced && guest_synced, "server should broadcast the requested time");

        host.disconnect();
        guest.disconnect();
        drop(server);
    }

    #[test]
    fn time_values_accept_named_and_numeric_ticks() {
        assert_eq!(parse_time_value("day"), Some(120.0));
        assert_eq!(parse_time_value("noon"), Some(240.0));
        assert_eq!(parse_time_value("night"), Some(360.0));
        assert_eq!(parse_time_value("midnight"), Some(0.0));
        assert_eq!(parse_time_value("12000"), Some(240.0));
        assert_eq!(parse_time_value("NaN"), None);
    }

    #[test]
    fn authoritative_world_snapshots_reach_other_clients_and_late_joiners() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let mut host =
            crate::net::NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "WorldHost")
                .unwrap();
        let mut observer = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "WorldObserver",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline
            && (!host.is_connected
                || !observer.is_connected
                || host.mob_authority_id.is_none()
                || observer.mob_authority_id.is_none())
        {
            host.update(0.05);
            observer.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(host.is_connected && observer.is_connected);
        assert_eq!(host.mob_authority_id, observer.mob_authority_id);

        let snapshot = WorldSnapshot {
            items: vec![WorldItemEntry {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                velocity_x: 0.0,
                velocity_y: 1.0,
                velocity_z: 0.0,
                block: crate::world::BlockId::Stone.to_u8(),
                count: 2,
                pickup_delay: 0.5,
                age: 1.0,
            }],
            ..WorldSnapshot::default()
        };

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut received = false;
        if host.local_id == host.mob_authority_id {
            host.send_packet(&ClientPacket::WorldSnapshot {
                snapshot: snapshot.clone(),
            })
            .unwrap();
            while Instant::now() < deadline && !received {
                host.update(0.05);
                received |= observer.update(0.05).into_iter().any(|event| {
                    matches!(event, crate::net::NetworkEvent::WorldSnapshot { snapshot: actual } if actual == snapshot)
                });
                thread::sleep(Duration::from_millis(10));
            }
        } else {
            observer
                .send_packet(&ClientPacket::WorldSnapshot {
                    snapshot: snapshot.clone(),
                })
                .unwrap();
            while Instant::now() < deadline && !received {
                observer.update(0.05);
                received |= host.update(0.05).into_iter().any(|event| {
                    matches!(event, crate::net::NetworkEvent::WorldSnapshot { snapshot: actual } if actual == snapshot)
                });
                thread::sleep(Duration::from_millis(10));
            }
        }
        assert!(received, "observer should receive the authority's world snapshot");

        host.disconnect();
        observer.disconnect();
        thread::sleep(Duration::from_millis(200));
        let mut late_joiner = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "WorldLateJoiner",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut late_joiner_received = false;
        while Instant::now() < deadline && !late_joiner_received {
            late_joiner_received |= late_joiner.update(0.05).into_iter().any(|event| {
                matches!(event, crate::net::NetworkEvent::WorldSnapshot { snapshot: actual } if actual == snapshot)
            });
            thread::sleep(Duration::from_millis(10));
        }
        assert!(late_joiner_received, "late joiner should receive the latest snapshot");

        late_joiner.disconnect();
        drop(server);
    }

    #[test]
    fn embedded_server_uses_world_snapshot_and_stops_cleanly() {
        let initial_mob = MobSyncEntry {
            id: 7,
            kind: crate::gameplay::mob::MobKind::Zombie as u8,
            x: 4.0,
            y: 65.0,
            z: -2.0,
            yaw: 0.5,
            health: 20.0,
            is_dead: false,
            walk_phase: 1.0,
            is_baby: false,
            wool_color: 0,
            is_sheared: false,
            is_puffed: false,
            fuse: 0.0,
            is_ignited: false,
        };
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 98765,
                time_of_day: 42.0,
                spawn_pos: (3.5, 65.0, -2.5),
                blocks: vec![(3, 10, -2, crate::world::BlockId::Stone.to_u8())],
                mobs: vec![initial_mob.clone()],
            },
        )
        .unwrap();
        let mut client = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "Host",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut connected = None;
        let mut host_received_mob_snapshot = false;
        while Instant::now() < deadline {
            for event in client.update(0.05) {
                match event {
                    crate::net::NetworkEvent::Connected {
                        seed,
                        time_of_day,
                        modified_blocks,
                        ..
                    } => connected = Some((seed, time_of_day, modified_blocks)),
                    crate::net::NetworkEvent::MobSnapshot { .. } => {
                        host_received_mob_snapshot = true;
                    }
                    _ => {}
                }
            }
            if connected.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let (seed, time_of_day, modified_blocks) = connected.expect("host client should log in");
        assert_eq!(seed, 98765);
        assert_eq!(time_of_day, 42.0);
        assert_eq!(modified_blocks, vec![(3, 10, -2, crate::world::BlockId::Stone.to_u8())]);

        assert!(!host_received_mob_snapshot);
        let host_position = crate::core::math::Vec3::new(31.0, 72.0, -8.0);
        client.send_position(host_position, 0.3, -0.1, false, true);
        thread::sleep(Duration::from_millis(100));
        client.update(0.05);
        let mut observer = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "Observer",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut observer_received_mobs = false;
        while Instant::now() < deadline && !observer_received_mobs {
            client.update(0.05);
            for event in observer.update(0.05) {
                if let crate::net::NetworkEvent::MobSnapshot { mobs } = event {
                    observer_received_mobs = mobs == vec![initial_mob.clone()];
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(observer_received_mobs);
        let host_id = client.local_id.expect("host should have an assigned ID");
        let replicated_host = observer
            .remote_players
            .get(&host_id)
            .expect("late joiner should receive existing players");
        assert_eq!(replicated_host.target_pos, host_position);
        assert_eq!(replicated_host.yaw, 0.3);

        client
            .send_packet(&ClientPacket::MobSnapshot {
                mobs: vec![initial_mob.clone()],
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut observer_received_update = false;
        let mut authority_received_own_update = false;
        while Instant::now() < deadline && !observer_received_update {
            for event in client.update(0.05) {
                authority_received_own_update |=
                    matches!(event, crate::net::NetworkEvent::MobSnapshot { .. });
            }
            for event in observer.update(0.05) {
                if let crate::net::NetworkEvent::MobSnapshot { mobs } = event {
                    observer_received_update = mobs == vec![initial_mob.clone()];
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(observer_received_update);
        assert!(!authority_received_own_update);

        let changed_block = (
            6,
            20,
            -7,
            crate::world::BlockId::Dirt.to_u8(),
        );
        client.send_block_change(
            changed_block.0,
            changed_block.1,
            changed_block.2,
            changed_block.3,
        );
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut observer_received_block = false;
        while Instant::now() < deadline && !observer_received_block {
            client.update(0.05);
            for event in observer.update(0.05) {
                if let crate::net::NetworkEvent::BlockChanged { x, y, z, block } = event {
                    observer_received_block = (x, y, z, block) == changed_block;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(observer_received_block);

        let mut late_joiner = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "LateJoiner",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut late_joiner_blocks = None;
        while Instant::now() < deadline && late_joiner_blocks.is_none() {
            client.update(0.05);
            observer.update(0.05);
            for event in late_joiner.update(0.05) {
                if let crate::net::NetworkEvent::Connected {
                    modified_blocks, ..
                } = event
                {
                    late_joiner_blocks = Some(modified_blocks);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        let late_joiner_blocks = late_joiner_blocks.expect("late joiner should receive world changes");
        assert!(late_joiner_blocks.contains(&changed_block));

        late_joiner.disconnect();
        observer.disconnect();
        client.disconnect();
        drop(server);
    }

    #[test]
    fn unauthenticated_clients_cannot_change_replicated_blocks() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let mut unauthenticated =
            TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        unauthenticated
            .write_all(
                &ClientPacket::BlockChange {
                    x: 4,
                    y: 20,
                    z: 8,
                    block: crate::world::BlockId::DiamondBlock.to_u8(),
                }
                .serialize(),
            )
            .unwrap();
        drop(unauthenticated);
        thread::sleep(Duration::from_millis(100));

        let mut observer = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "Observer",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut modified_blocks = None;
        while Instant::now() < deadline && modified_blocks.is_none() {
            for event in observer.update(0.05) {
                if let crate::net::NetworkEvent::Connected {
                    modified_blocks: blocks,
                    ..
                } = event
                {
                    modified_blocks = Some(blocks);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            modified_blocks.expect("observer should connect"),
            Vec::<(i32, i32, i32, u8)>::new()
        );
        observer.disconnect();
        drop(server);
    }

    #[test]
    fn in_game_teleport_command_moves_player_to_named_target() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let mut host = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "Host",
        )
        .unwrap();
        let mut guest = crate::net::NetworkClient::connect(
            &format!("127.0.0.1:{}", server.port),
            "Guest",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && (!host.is_connected || !guest.is_connected) {
            host.update(0.05);
            guest.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(host.is_connected && guest.is_connected);

        host.send_position(
            crate::core::math::Vec3::new(31.0, 72.0, -8.0),
            0.0,
            0.0,
            false,
            false,
        );
        thread::sleep(Duration::from_millis(100));
        guest.send_chat("/tp Host");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut teleport = None;
        while Instant::now() < deadline && teleport.is_none() {
            for event in guest.update(0.05) {
                if let crate::net::NetworkEvent::Teleport { x, y, z } = event {
                    teleport = Some((x, y, z));
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(teleport, Some((31.0, 72.0, -8.0)));
        host.disconnect();
        guest.disconnect();
        drop(server);
    }

    #[test]
    fn embedded_host_can_kick_a_player_with_a_visible_reason() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let address = format!("127.0.0.1:{}", server.port);
        let mut host = crate::net::NetworkClient::connect(&address, "Host").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !host.is_connected {
            host.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(host.is_connected);

        let mut guest = crate::net::NetworkClient::connect(&address, "Guest").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !guest.is_connected {
            host.update(0.05);
            guest.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(guest.is_connected);

        guest.send_chat("/kick Host unauthorized");
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut permission_denied = false;
        while Instant::now() < deadline && !permission_denied {
            host.update(0.05);
            for event in guest.update(0.05) {
                if let crate::net::NetworkEvent::ChatMessage { text, .. } = event {
                    permission_denied = text.contains("Only the multiplayer host can kick");
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(permission_denied);
        assert!(host.is_connected && guest.is_connected);

        host.send_chat("/kick Guest Testing kick reason");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut disconnect_reason = None;
        while Instant::now() < deadline && disconnect_reason.is_none() {
            host.update(0.05);
            for event in guest.update(0.05) {
                if let crate::net::NetworkEvent::Disconnected { reason } = event {
                    disconnect_reason = Some(reason);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(
            disconnect_reason.as_deref(),
            Some("Testing kick reason")
        );
        host.disconnect();
        drop(server);
    }

    #[test]
    fn player_death_is_broadcast_to_all_connected_clients() {
        let server = EmbeddedServer::start(
            0,
            EmbeddedServerState {
                seed: 1,
                time_of_day: 120.0,
                spawn_pos: (8.5, 65.0, 8.5),
                blocks: Vec::new(),
                mobs: Vec::new(),
            },
        )
        .unwrap();
        let address = format!("127.0.0.1:{}", server.port);
        let mut player = crate::net::NetworkClient::connect(&address, "PlayerOne").unwrap();
        let mut observer = crate::net::NetworkClient::connect(&address, "Observer").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && (!player.is_connected || !observer.is_connected) {
            player.update(0.05);
            observer.update(0.05);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(player.is_connected && observer.is_connected);

        player.send_player_death();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut player_notified = false;
        let mut observer_notified = false;
        while Instant::now() < deadline && !(player_notified && observer_notified) {
            for event in player.update(0.05) {
                player_notified |= matches!(
                    event,
                    crate::net::NetworkEvent::ChatMessage { ref text, is_system: true, .. }
                        if text == "PlayerOne died"
                );
            }
            for event in observer.update(0.05) {
                observer_notified |= matches!(
                    event,
                    crate::net::NetworkEvent::ChatMessage { ref text, is_system: true, .. }
                        if text == "PlayerOne died"
                );
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(player_notified && observer_notified);
        player.disconnect();
        observer.disconnect();
        drop(server);
    }

    #[test]
    fn test_server_config_defaults_and_parse() {
        let default_cfg = ServerConfig::default();
        assert_eq!(default_cfg.server_port, 26656);
        assert_eq!(default_cfg.max_players, 20);
        assert!(default_cfg.pvp);
        assert_eq!(default_cfg.gamemode, "survival");

        let test_props = "\
# Test server properties
server-port=25565
level-seed=424242
motd=Custom Test MOTD
max-players=64
pvp=false
gamemode=creative
difficulty=hard
view-distance=12
spawn-x=10.0
spawn-y=70.0
spawn-z=20.0
";
        let config_path = std::env::temp_dir().join(format!("test_server_props_{}.properties", std::process::id()));
        std::fs::write(&config_path, test_props).unwrap();

        let loaded = ServerConfig::load_or_create(config_path.to_str().unwrap());
        assert_eq!(loaded.server_port, 25565);
        assert_eq!(loaded.level_seed, 424242);
        assert_eq!(loaded.motd, "Custom Test MOTD");
        assert_eq!(loaded.max_players, 64);
        assert!(!loaded.pvp);
        assert_eq!(loaded.gamemode, "creative");
        assert_eq!(loaded.difficulty, "hard");
        assert_eq!(loaded.view_distance, 12);
        assert_eq!(loaded.spawn_pos, (10.0, 70.0, 20.0));

        let _ = std::fs::remove_file(&config_path);
    }

    #[test]
    fn test_server_config_creates_missing_file() {
        let config_path = std::env::temp_dir().join(format!("test_missing_props_{}.properties", std::process::id()));
        let _ = std::fs::remove_file(&config_path);
        assert!(!config_path.exists());

        let created = ServerConfig::load_or_create(config_path.to_str().unwrap());
        assert!(config_path.exists());
        assert_eq!(created.server_port, 26656);
        assert_eq!(created.max_players, 20);

        let _ = std::fs::remove_file(&config_path);
    }
}
