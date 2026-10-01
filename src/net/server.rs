//! Dedicated RustCraft multiplayer server with terminal console command management.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::net::protocol::*;

struct ConnectedClient {
    id: u32,
    stream: TcpStream,
    addr: SocketAddr,
    nickname: String,
    ping_ms: u32,
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    is_sneaking: bool,
    is_sprinting: bool,
    rx_buffer: Vec<u8>,
    #[allow(dead_code)]
    last_ping_sent: Instant,
    #[allow(dead_code)]
    pending_ping_seq: u32,
}

impl ConnectedClient {
    fn send_packet(&mut self, packet: &ServerPacket) -> bool {
        let bytes = packet.serialize();
        self.stream.write_all(&bytes).is_ok()
    }
}

/// Binds to `preferred_port` (default 26656). If occupied, automatically tries next available ports.
pub fn bind_server_listener(preferred_port: u16) -> Result<(TcpListener, u16), String> {
    let mut port = preferred_port;
    let max_port = preferred_port.saturating_add(50);
    while port <= max_port {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => {
                let _ = listener.set_nonblocking(true);
                return Ok((listener, port));
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

/// Dedicated server entrypoint, invoked when rustcraft is started with `--server`.
pub fn run_dedicated_server(preferred_port: u16, seed: u32) {
    println!("============================================================");
    println!("           RustCraft Dedicated Multiplayer Server           ");
    println!("============================================================");

    let (listener, port) = match bind_server_listener(preferred_port) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("[SERVER] FATAL: {}", err);
            std::process::exit(1);
        }
    };

    println!("[SERVER] Successfully bound to 0.0.0.0:{}", port);
    println!("[SERVER] World Seed: {}", seed);
    println!("[SERVER] Default Spawn: (8.5, 65.0, 8.5)");
    println!("[SERVER] Type 'help' for a list of terminal console commands.");
    println!("------------------------------------------------------------");

    // Spawn terminal input reader thread
    let (cmd_tx, cmd_rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();

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

    let mut clients: HashMap<u32, ConnectedClient> = HashMap::new();
    let mut next_client_id: u32 = 1;
    let mut modified_blocks: HashMap<(i32, i32, i32), u8> = HashMap::new();
    let mut time_of_day: f32 = 120.0;
    let spawn_pos = (8.5_f32, 65.0_f32, 8.5_f32);

    let start_time = Instant::now();
    let tick_duration = Duration::from_millis(50); // 20 TPS
    let mut tick_counter: u64 = 0;

    let mut last_fps_check = Instant::now();
    let mut ticks_in_second = 0;
    let mut current_tps = 20.0;

    while running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        // 1. Accept new incoming client TCP connections
        while let Ok((stream, addr)) = listener.accept() {
            let _ = stream.set_nonblocking(true);
            let id = next_client_id;
            next_client_id += 1;
            println!("[SERVER] Incoming connection from {} (Assigned Player ID: {})", addr, id);
            clients.insert(
                id,
                ConnectedClient {
                    id,
                    stream,
                    addr,
                    nickname: format!("Player{}", id),
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
                    pending_ping_seq: 0,
                },
            );
        }

        // 2. Read incoming packets from connected clients
        let mut disconnected_clients = Vec::new();
        let mut broadcast_packets: Vec<ServerPacket> = Vec::new();
        let mut player_pos_updates: Vec<PlayerPosUpdate> = Vec::new();

        let client_ids: Vec<u32> = clients.keys().copied().collect();
        for id in client_ids {
            let client = clients.get_mut(&id).unwrap();
            let mut buf = [0u8; 4096];
            match client.stream.read(&mut buf) {
                Ok(0) => {
                    disconnected_clients.push((id, "Connection closed by remote host".to_string()));
                }
                Ok(n) => {
                    client.rx_buffer.extend_from_slice(&buf[..n]);
                    let raw_packets = extract_packets(&mut client.rx_buffer);
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

                                client.nickname = nickname.clone();
                                println!("[SERVER] Player '{}' (ID {}) logged in from {}", nickname, id, client.addr);

                                // Send LoginSuccess with seed, time, and all modified blocks
                                let block_list: Vec<(i32, i32, i32, u8)> = modified_blocks
                                    .iter()
                                    .map(|(&(bx, by, bz), &blk)| (bx, by, bz, blk))
                                    .collect();

                                let login_success = ServerPacket::LoginSuccess {
                                    player_id: id,
                                    seed,
                                    spawn_x: spawn_pos.0,
                                    spawn_y: spawn_pos.1,
                                    spawn_z: spawn_pos.2,
                                    time_of_day,
                                    modified_blocks: block_list,
                                };
                                let _ = client.send_packet(&login_success);

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
                                client.x = x;
                                client.y = y;
                                client.z = z;
                                client.yaw = yaw;
                                client.pitch = pitch;
                                client.is_sneaking = is_sneaking;
                                client.is_sprinting = is_sprinting;
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
                                modified_blocks.insert((x, y, z), block);
                                println!(
                                    "[SERVER] Block change from '{}' at ({}, {}, {}) -> BlockId {}",
                                    client.nickname, x, y, z, block
                                );
                                broadcast_packets.push(ServerPacket::BlockChange { x, y, z, block });
                            }
                            Ok(ClientPacket::ChatMessage { text }) => {
                                println!("[CHAT] <{}> {}", client.nickname, text);
                                broadcast_packets.push(ServerPacket::ChatMessage {
                                    sender: client.nickname.clone(),
                                    text,
                                    is_system: false,
                                });
                            }
                            Ok(ClientPacket::Ping { seq }) => {
                                let _ = client.send_packet(&ServerPacket::Pong { seq });
                            }
                            Ok(ClientPacket::Disconnect { reason }) => {
                                disconnected_clients.push((id, reason));
                                break;
                            }
                            Err(err) => {
                                eprintln!("[SERVER] Corrupted packet from ID {}: {}", id, err);
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

        // 3. Handle disconnected clients
        for (id, reason) in disconnected_clients {
            if let Some(client) = clients.remove(&id) {
                println!("[SERVER] Player '{}' (ID {}) disconnected: {}", client.nickname, id, reason);
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

        // 4. Broadcast queued packets
        if !broadcast_packets.is_empty() {
            for packet in &broadcast_packets {
                for client in clients.values_mut() {
                    let _ = client.send_packet(packet);
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

        // 6. Periodic heartbeat & player list update (every 20 ticks / ~1 second)
        tick_counter += 1;
        if tick_counter % 20 == 0 {
            time_of_day = (time_of_day + 1.0).rem_euclid(480.0);
            let time_sync = ServerPacket::TimeSync { time_of_day };

            let player_list: Vec<PlayerInfo> = clients
                .values()
                .map(|c| PlayerInfo {
                    id: c.id,
                    nickname: c.nickname.clone(),
                    ping_ms: c.ping_ms,
                })
                .collect();
            let list_packet = ServerPacket::PlayerList {
                players: player_list,
            };

            for client in clients.values_mut() {
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
    }
    println!("[SERVER] Goodbye!");
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
                    let mut client = clients.remove(&id).unwrap();
                    let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                        reason: reason.clone(),
                    });
                    println!("[SERVER] Kicked '{}' (ID {}): {}", client.nickname, id, reason);
                    let left_packet = ServerPacket::PlayerLeft {
                        id,
                        nickname: client.nickname.clone(),
                        reason,
                    };
                    for other in clients.values_mut() {
                        let _ = other.send_packet(&left_packet);
                    }
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
