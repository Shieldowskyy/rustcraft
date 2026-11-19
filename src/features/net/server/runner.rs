//! Główna pętla ticków serwera dedykowanego i wbudowanego (`run_dedicated_server`, `EmbeddedServer`, `EmbeddedServerState`).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, channel, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::features::net::discovery::LanDiscoveryHost;
use crate::features::net::protocol::*;
use crate::features::net::server_log::{init_server_logger_with_max_size, shutdown_server_logger};
use crate::features::player::RecipeBook;

use super::client_io::read_client_packets;
use super::commands_chat::handle_in_game_command;
use super::commands_console::handle_console_command;
use super::config::{
    bind_server_listener, load_ops, persist_client_player_data, ConnectedClient, ServerConfig,
};

#[derive(Clone)]
pub struct EmbeddedServerState {
    pub seed: u32,
    pub time_of_day: f32,
    pub spawn_pos: (f32, f32, f32),
    pub blocks: Vec<(i32, i32, i32, u8)>,
    pub mobs: Vec<MobSyncEntry>,
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
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("rustcraft-embedded-server".to_string())
            .spawn(move || {
                run_server(
                    preferred_port,
                    state.seed,
                    Some(state),
                    true,
                    thread_running,
                    Some(ready_tx),
                );
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
        let logs_dir =
            std::env::var("RUSTCRAFT_LOGS_DIR").unwrap_or_else(|_| config.logs_dir.clone());
        let max_log_size_bytes = config.max_log_size_mb * 1024 * 1024;
        let _ = init_server_logger_with_max_size(&logs_dir, max_log_size_bytes);
    }

    crate::server_info!("============================================================");
    crate::server_info!("           RustCraft Dedicated Multiplayer Server           ");
    crate::server_info!("============================================================");

    let (listener, port) = match bind_server_listener(config.server_port) {
        Ok(res) => res,
        Err(err) => {
            crate::server_error!("[SERVER] FATAL: {}", err);
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

    crate::server_info!("[SERVER] Successfully bound to 0.0.0.0:{}", port);
    crate::server_info!("[SERVER] World Seed: {}", config.level_seed);
    crate::server_info!("[SERVER] MOTD: \"{}\"", config.motd);
    crate::server_info!("[SERVER] Max Players: {}", config.max_players);
    crate::server_info!(
        "[SERVER] Default Spawn: ({:.1}, {:.1}, {:.1})",
        config.spawn_pos.0,
        config.spawn_pos.1,
        config.spawn_pos.2
    );
    crate::server_info!(
        "[SERVER] Gamemode: {}, PVP: {}, Difficulty: {}",
        config.gamemode,
        config.pvp,
        config.difficulty
    );
    crate::server_info!("[SERVER] Type 'help' for a list of terminal console commands.");
    crate::server_info!("------------------------------------------------------------");

    let (cmd_tx, cmd_rx): (mpsc::Sender<String>, Receiver<String>) = channel();
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
        .map(|state| {
            state
                .blocks
                .iter()
                .map(|&(x, y, z, block)| ((x, y, z), block))
                .collect()
        })
        .unwrap_or_default();
    let mut mob_authority_id = None;
    let mut embedded_host_id = None;
    let persist_ops = !embedded;
    let mut ops = load_ops(persist_ops);
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

    let mut recipe_book = RecipeBook::new();
    crate::engine::modding::ModManager::load_mods(&mut recipe_book);
    let active_server_mods = crate::engine::modding::ModManager::get_active_mod_files();
    if !active_server_mods.is_empty() {
        crate::server_info!(
            "[SERVER] Active mod(s) enabled for auto-download (Source engine style):"
        );
        for m in &active_server_mods {
            crate::server_info!("         - {}", m.mod_id);
        }
    }

    let start_time = Instant::now();
    let tick_duration = Duration::from_millis(50);
    let mut tick_counter: u64 = 0;

    let mut lan_discovery = LanDiscoveryHost::new(port, &config.motd, config.max_players as u32);

    let mut last_fps_check = Instant::now();
    let mut ticks_in_second = 0;
    let mut current_tps = 20.0;
    let mut tick_work_ms_total = 0.0;
    let mut current_mspt = 0.0;

    while running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        while let Ok((stream, addr)) = listener.accept() {
            if let Err(error) = stream.set_nonblocking(true) {
                crate::log_warn!("SERVER", "Could not configure client socket {addr}: {error}");
                continue;
            }
            let id = next_client_id;
            next_client_id += 1;
            crate::server_info!(
                "[SERVER] Incoming connection from {} (Assigned Player ID: {})",
                addr,
                id
            );
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

        let mut inputs = read_client_packets(
            &mut clients,
            &mut modified_blocks,
            &config,
            embedded,
            &mut embedded_host_id,
            &mut mob_authority_id,
            &active_server_mods,
            spawn_pos,
            time_of_day,
            &mob_snapshot,
            world_snapshot_initialized,
            &world_snapshot,
        );

        let mut pending_mob_broadcasts = Vec::new();
        let mut pending_world_broadcasts = Vec::new();

        for (sender_id, snapshot) in inputs.pending_mob_snapshots {
            if mob_authority_id == Some(sender_id) {
                mob_snapshot = snapshot.clone();
                pending_mob_broadcasts
                    .push((sender_id, ServerPacket::MobSnapshot { mobs: snapshot }));
            }
        }
        for (attacker_id, mob_id, damage, yaw, pitch) in inputs.pending_mob_attacks {
            let Some(authority_id) =
                mob_authority_id.filter(|authority_id| *authority_id != attacker_id)
            else {
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
        for (requester_id, kind, x, y, z) in inputs.pending_mob_spawns {
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
        for (sender_id, snapshot) in inputs.pending_world_snapshots {
            if mob_authority_id == Some(sender_id) {
                world_snapshot = snapshot.clone();
                world_snapshot_initialized = true;
                pending_world_broadcasts
                    .push((sender_id, ServerPacket::WorldSnapshot { snapshot }));
            }
        }

        for (attacker_id, attacker_name, (ax, ay, az), target_id, damage) in inputs.pending_attacks
        {
            if let Some(target_client) = clients.get_mut(&target_id) {
                let dx = target_client.x - ax;
                let dy = target_client.y - ay;
                let dz = target_client.z - az;
                if dx * dx + dy * dy + dz * dz <= 36.0 {
                    let h_len = (dx * dx + dz * dz).sqrt();
                    let (kb_x, kb_z) = if h_len > 0.001 {
                        (dx / h_len * 0.45, dz / h_len * 0.45)
                    } else {
                        (0.0, 0.45)
                    };
                    crate::server_info!(
                        "[SERVER PVP] Player '{}' (ID {}) attacked '{}' (ID {}) for {:.1} dmg",
                        attacker_name,
                        attacker_id,
                        target_client.nickname,
                        target_id,
                        damage
                    );
                    inputs.broadcast_packets.push(ServerPacket::PlayerHurt {
                        victim_id: target_id,
                        attacker_id,
                        damage,
                        knockback_x: kb_x,
                        knockback_z: kb_z,
                    });
                }
            }
        }

        for (sender_id, cmd_text) in inputs.in_game_commands {
            handle_in_game_command(
                &cmd_text,
                sender_id,
                embedded_host_id,
                &mut ops,
                persist_ops,
                &mut clients,
                &mut inputs.broadcast_packets,
                &mut time_of_day,
                &config,
                current_tps,
                current_mspt,
            );
        }

        for (id, reason) in inputs.disconnected_clients {
            if let Some(mut client) = clients.remove(&id) {
                let _ = client.flush_outgoing();
                let was_authenticated = client.authenticated;
                if was_authenticated {
                    persist_client_player_data(&mut client);
                }
                if mob_authority_id == Some(id) {
                    mob_authority_id = None;
                    if embedded {
                        running.store(false, Ordering::Release);
                    }
                }
                crate::server_info!(
                    "[SERVER] Player '{}' (ID {}) disconnected: {}",
                    client.nickname,
                    id,
                    reason
                );
                if was_authenticated {
                    inputs.broadcast_packets.push(ServerPacket::PlayerLeft {
                        id,
                        nickname: client.nickname.clone(),
                        reason,
                    });
                    inputs.broadcast_packets.push(ServerPacket::ChatMessage {
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
                inputs
                    .broadcast_packets
                    .push(ServerPacket::MobAuthority { id });
            }
        }

        for packet in &inputs.broadcast_packets {
            for client in clients.values_mut() {
                let _ = client.send_packet(packet);
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

        if !inputs.player_pos_updates.is_empty() && clients.len() > 1 {
            let pos_packet = ServerPacket::PlayerPositions {
                updates: inputs.player_pos_updates,
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

        while let Ok(cmd_line) = cmd_rx.try_recv() {
            handle_console_command(
                &cmd_line,
                &mut clients,
                &mut modified_blocks,
                &mut time_of_day,
                &mut ops,
                persist_ops,
                seed,
                current_tps,
                current_mspt,
                start_time.elapsed(),
                &running,
            );
        }

        lan_discovery.update(
            clients
                .values()
                .filter(|client| client.authenticated)
                .count() as u32,
        );

        ticks_in_second += 1;
        tick_work_ms_total += frame_start.elapsed().as_secs_f32() * 1000.0;
        if last_fps_check.elapsed() >= Duration::from_secs(1) {
            let sample_seconds = last_fps_check.elapsed().as_secs_f32();
            current_tps = ticks_in_second as f32 / sample_seconds;
            current_mspt = tick_work_ms_total / ticks_in_second.max(1) as f32;
            ticks_in_second = 0;
            tick_work_ms_total = 0.0;
            last_fps_check = Instant::now();
        }

        let elapsed = frame_start.elapsed();
        if elapsed < tick_duration {
            std::thread::sleep(tick_duration - elapsed);
        }
    }

    crate::server_info!("[SERVER] Server shutting down... Disconnecting all active players.");
    for client in clients.values_mut() {
        let _ = client.send_packet(&ServerPacket::LoginDisconnect {
            reason: "Server shut down by host console".to_string(),
        });
        let _ = client.flush_outgoing();
    }
    crate::server_info!("[SERVER] Goodbye!");
    if !embedded {
        shutdown_server_logger();
    }
}
