//! Przetwarzanie pakietów przychodzących od klientów na serwerze w trakcie pojedynczego ticka.

use std::collections::{HashMap, HashSet};
use std::io::Read;

use crate::features::entities::MobKind;
use crate::features::net::protocol::*;
use crate::features::world::CHUNK_SIZE_Y;

use super::config::{
    is_valid_nickname, load_player_data, save_player_data, ConnectedClient, ServerConfig,
};

pub(super) struct TickClientInputs {
    pub(super) disconnected_clients: Vec<(u32, String)>,
    pub(super) broadcast_packets: Vec<ServerPacket>,
    pub(super) player_pos_updates: Vec<PlayerPosUpdate>,
    pub(super) in_game_commands: Vec<(u32, String)>,
    pub(super) pending_attacks: Vec<(u32, String, (f32, f32, f32), u32, f32)>,
    pub(super) pending_mob_attacks: Vec<(u32, u32, f32, f32, f32)>,
    pub(super) pending_mob_spawns: Vec<(u32, u8, f32, f32, f32)>,
    pub(super) pending_mob_snapshots: Vec<(u32, Vec<MobSyncEntry>)>,
    pub(super) pending_world_snapshots: Vec<(u32, WorldSnapshot)>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn read_client_packets(
    clients: &mut HashMap<u32, ConnectedClient>,
    modified_blocks: &mut HashMap<(i32, i32, i32), u8>,
    config: &ServerConfig,
    embedded: bool,
    embedded_host_id: &mut Option<u32>,
    mob_authority_id: &mut Option<u32>,
    active_server_mods: &[ModSyncEntry],
    spawn_pos: (f32, f32, f32),
    time_of_day: f32,
    mob_snapshot: &[MobSyncEntry],
    world_snapshot_initialized: bool,
    world_snapshot: &WorldSnapshot,
) -> TickClientInputs {
    let mut out = TickClientInputs {
        disconnected_clients: Vec::new(),
        broadcast_packets: Vec::new(),
        player_pos_updates: Vec::new(),
        in_game_commands: Vec::new(),
        pending_attacks: Vec::new(),
        pending_mob_attacks: Vec::new(),
        pending_mob_spawns: Vec::new(),
        pending_mob_snapshots: Vec::new(),
        pending_world_snapshots: Vec::new(),
    };

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
            out.disconnected_clients
                .push((id, "Network write failed".to_string()));
            continue;
        }
        let mut buf = [0u8; 4096];
        match client.stream.read(&mut buf) {
            Ok(0) => {
                out.disconnected_clients
                    .push((id, "Connection closed by remote host".to_string()));
            }
            Ok(n) => {
                client.rx_buffer.extend_from_slice(&buf[..n]);
                let raw_packets = match extract_packets(&mut client.rx_buffer) {
                    Ok(packets) => packets,
                    Err(error) => {
                        out.disconnected_clients
                            .push((id, format!("Invalid packet frame: {error}")));
                        continue;
                    }
                };
                for raw in raw_packets {
                    match ClientPacket::deserialize(&raw) {
                        Ok(ClientPacket::Handshake {
                            protocol_version,
                            nickname,
                        }) => {
                            if protocol_version != PROTOCOL_VERSION {
                                let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                    reason: format!(
                                        "Protocol mismatch: server={}, client={}",
                                        PROTOCOL_VERSION, protocol_version
                                    ),
                                });
                                out.disconnected_clients
                                    .push((id, "Protocol version mismatch".to_string()));
                                break;
                            }
                            if client.authenticated {
                                let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                    reason: "Handshake already completed".to_string(),
                                });
                                out.disconnected_clients
                                    .push((id, "Repeated handshake".to_string()));
                                break;
                            }
                            if !is_valid_nickname(&nickname) {
                                let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                    reason: "Nickname must contain 1-24 letters, numbers, '_' or '-'"
                                        .to_string(),
                                });
                                out.disconnected_clients
                                    .push((id, "Invalid nickname".to_string()));
                                break;
                            }
                            let nickname = nickname.trim().to_string();
                            if authenticated_nicknames.contains(&nickname.to_lowercase()) {
                                let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                    reason: "That nickname is already in use".to_string(),
                                });
                                out.disconnected_clients
                                    .push((id, "Nickname already in use".to_string()));
                                break;
                            }
                            if authenticated_client_count >= config.max_players {
                                let _ = client.send_packet(&ServerPacket::LoginDisconnect {
                                    reason: format!(
                                        "Server is full! (max players: {})",
                                        config.max_players
                                    ),
                                });
                                out.disconnected_clients
                                    .push((id, "Server full".to_string()));
                                break;
                            }

                            client.nickname = nickname.clone();
                            client.authenticated = true;
                            authenticated_client_count += 1;
                            authenticated_nicknames.insert(nickname.to_lowercase());
                            if embedded && embedded_host_id.is_none() {
                                *embedded_host_id = Some(id);
                            }
                            if embedded && *embedded_host_id == Some(id) {
                                client.player_data = None;
                            } else {
                                match load_player_data(std::path::Path::new("playerdata"), &nickname)
                                {
                                    Ok(data) => {
                                        if let Some(data) = &data {
                                            client.x = data.x;
                                            client.y = data.y;
                                            client.z = data.z;
                                        }
                                        client.player_data = data;
                                    }
                                    Err(error) => {
                                        crate::log_warn!(
                                            "SERVER",
                                            "Could not load player data for '{}': {}",
                                            nickname,
                                            error
                                        );
                                        client.player_data = None;
                                    }
                                }
                            }
                            if mob_authority_id.is_none() {
                                *mob_authority_id = Some(id);
                            }
                            crate::server_info!(
                                "[SERVER] Player '{}' (ID {}) logged in from {}",
                                nickname,
                                id,
                                client.addr
                            );

                            let _ = client.send_packet(&ServerPacket::ModSync {
                                mods: active_server_mods.to_vec(),
                            });

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
                            if let Some(authority_id) = *mob_authority_id {
                                let _ = client
                                    .send_packet(&ServerPacket::MobAuthority { id: authority_id });
                            }
                            if *mob_authority_id != Some(id) {
                                let _ = client.send_packet(&ServerPacket::MobSnapshot {
                                    mobs: mob_snapshot.to_vec(),
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

                            if !config.motd.is_empty() {
                                let _ = client.send_packet(&ServerPacket::ChatMessage {
                                    sender: "Server".to_string(),
                                    text: config.motd.clone(),
                                    is_system: true,
                                });
                            }

                            out.broadcast_packets.push(ServerPacket::PlayerJoined {
                                id,
                                nickname: nickname.clone(),
                            });
                            out.broadcast_packets.push(ServerPacket::ChatMessage {
                                sender: "Server".to_string(),
                                text: format!("{} joined the game", nickname),
                                is_system: true,
                            });
                        }
                        Ok(ClientPacket::PlayerMove {
                            x,
                            y,
                            z,
                            yaw,
                            pitch,
                            is_sneaking,
                            is_sprinting,
                        }) => {
                            if !client.authenticated
                                || ![x, y, z, yaw, pitch].iter().all(|value| value.is_finite())
                            {
                                out.disconnected_clients
                                    .push((id, "Invalid player movement".to_string()));
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
                            out.player_pos_updates.push(PlayerPosUpdate {
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
                            if !client.authenticated || !(0..CHUNK_SIZE_Y).contains(&y) {
                                out.disconnected_clients
                                    .push((id, "Invalid block change".to_string()));
                                break;
                            }
                            modified_blocks.insert((x, y, z), block);
                            crate::server_info!(
                                "[SERVER] Block change from '{}' at ({}, {}, {}) -> BlockId {}",
                                client.nickname,
                                x,
                                y,
                                z,
                                block
                            );
                            out.broadcast_packets
                                .push(ServerPacket::BlockChange { x, y, z, block });
                        }
                        Ok(ClientPacket::ChatMessage { text }) => {
                            if !client.authenticated {
                                out.disconnected_clients
                                    .push((id, "Login required".to_string()));
                                break;
                            }
                            if text.starts_with('/') {
                                out.in_game_commands.push((id, text));
                            } else {
                                crate::server_info!("[CHAT] <{}> {}", client.nickname, text);
                                out.broadcast_packets.push(ServerPacket::ChatMessage {
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
                            out.disconnected_clients.push((id, reason));
                            break;
                        }
                        Ok(ClientPacket::AttackPlayer { target_id, damage }) => {
                            if !client.authenticated {
                                out.disconnected_clients
                                    .push((id, "Login required".to_string()));
                                break;
                            } else if !config.pvp {
                                crate::server_info!(
                                    "[SERVER] Attack from '{}' on player ID {} ignored (pvp=false)",
                                    client.nickname,
                                    target_id
                                );
                            } else {
                                out.pending_attacks.push((
                                    id,
                                    client.nickname.clone(),
                                    (client.x, client.y, client.z),
                                    target_id,
                                    damage,
                                ));
                            }
                        }
                        Ok(ClientPacket::AttackMob { mob_id, damage }) => {
                            if !client.authenticated
                                || !damage.is_finite()
                                || damage <= 0.0
                                || damage > 50.0
                            {
                                out.disconnected_clients
                                    .push((id, "Invalid mob attack".to_string()));
                                break;
                            }
                            out.pending_mob_attacks.push((
                                id,
                                mob_id,
                                damage,
                                client.yaw,
                                client.pitch,
                            ));
                        }
                        Ok(ClientPacket::SpawnMob { kind, x, y, z }) => {
                            if !client.authenticated
                                || MobKind::ALL.get(kind as usize).is_none()
                                || ![x, y, z].iter().all(|coordinate| coordinate.is_finite())
                            {
                                out.disconnected_clients
                                    .push((id, "Invalid mob spawn request".to_string()));
                                break;
                            }
                            out.pending_mob_spawns.push((id, kind, x, y, z));
                        }
                        Ok(ClientPacket::MobSnapshot { mobs }) => {
                            if client.authenticated {
                                out.pending_mob_snapshots.push((id, mobs));
                            } else {
                                out.disconnected_clients
                                    .push((id, "Login required".to_string()));
                                break;
                            }
                        }
                        Ok(ClientPacket::WorldSnapshot { snapshot }) => {
                            if client.authenticated {
                                out.pending_world_snapshots.push((id, snapshot));
                            } else {
                                out.disconnected_clients
                                    .push((id, "Login required".to_string()));
                                break;
                            }
                        }
                        Ok(ClientPacket::PlayerDeath) => {
                            if client.authenticated {
                                out.broadcast_packets.push(ServerPacket::ChatMessage {
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
                                out.disconnected_clients
                                    .push((id, "Invalid player data".to_string()));
                                break;
                            }
                            client.x = data.x;
                            client.y = data.y;
                            client.z = data.z;
                            client.player_data = Some(data.clone());
                            if let Err(error) = save_player_data(
                                std::path::Path::new("playerdata"),
                                &client.nickname,
                                &data,
                            ) {
                                crate::log_error!(
                                    "SERVER",
                                    "Could not save player data for '{}': {}",
                                    client.nickname,
                                    error
                                );
                            }
                        }
                        Err(err) => {
                            crate::server_error!(
                                "[SERVER] Corrupted packet from ID {}: {}",
                                id,
                                err
                            );
                            out.disconnected_clients
                                .push((id, format!("Invalid packet: {err}")));
                            break;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                out.disconnected_clients
                    .push((id, format!("I/O Error: {}", e)));
            }
        }
    }
    out
}
