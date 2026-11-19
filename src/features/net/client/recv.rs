//! Odbieranie ramek TCP od serwera, interpolacja pozycji zdalnych graczy i emitowanie `NetworkEvent`.

use std::io::Read;

use crate::core::math::Vec3;
use crate::features::net::protocol::{extract_packets, ClientPacket, ServerPacket};

use super::state::{NetworkClient, NetworkEvent, RemotePlayer};

impl NetworkClient {
    pub fn update(&mut self, dt: f32) -> Vec<NetworkEvent> {
        let mut events = Vec::new();
        if let Err(error) = self.flush_outgoing() {
            self.is_connected = false;
            let reason = format!("Network write failed: {error}");
            self.disconnect_reason = Some(reason.clone());
            events.push(NetworkEvent::Disconnected { reason });
            return events;
        }

        for rp in self.remote_players.values_mut() {
            if rp.hurt_timer > 0.0 {
                rp.hurt_timer = (rp.hurt_timer - dt).max(0.0);
            }
            let diff = rp.target_pos - rp.pos;
            let len = diff.length();
            if len > 0.001 {
                let speed = (len * 12.0).clamp(2.0, 30.0);
                let step = speed * dt;
                if step >= len {
                    rp.pos = rp.target_pos;
                } else {
                    rp.pos = rp.pos + diff * (step / len);
                }
                rp.walk_phase += dt * 10.0;
            }
        }

        let mut buf = [0u8; 4096];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => {
                    self.is_connected = false;
                    let reason = self
                        .disconnect_reason
                        .clone()
                        .unwrap_or_else(|| "Server closed connection".to_string());
                    events.push(NetworkEvent::Disconnected { reason });
                    break;
                }
                Ok(n) => {
                    self.rx_buffer.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    self.is_connected = false;
                    let reason = format!("Network error: {}", e);
                    events.push(NetworkEvent::Disconnected { reason });
                    break;
                }
            }
        }

        let raw_packets = match extract_packets(&mut self.rx_buffer) {
            Ok(packets) => packets,
            Err(error) => {
                self.is_connected = false;
                let reason = format!("Invalid response from server: {error}");
                self.disconnect_reason = Some(reason.clone());
                events.push(NetworkEvent::Disconnected { reason });
                return events;
            }
        };

        for raw in raw_packets {
            match ServerPacket::deserialize(&raw) {
                Ok(ServerPacket::LoginSuccess {
                    player_id,
                    seed,
                    spawn_x,
                    spawn_y,
                    spawn_z,
                    time_of_day,
                    modified_blocks,
                }) => {
                    self.local_id = Some(player_id);
                    self.is_connected = true;
                    events.push(NetworkEvent::Connected {
                        player_id,
                        seed,
                        spawn_x,
                        spawn_y,
                        spawn_z,
                        time_of_day,
                        modified_blocks,
                    });
                }
                Ok(ServerPacket::LoginDisconnect { reason }) => {
                    self.is_connected = false;
                    self.disconnect_reason = Some(reason.clone());
                    events.push(NetworkEvent::Disconnected { reason });
                }
                Ok(ServerPacket::PlayerList { players }) => {
                    if let Some(local_id) = self.local_id {
                        if let Some(local_player) =
                            players.iter().find(|player| player.id == local_id)
                        {
                            self.ping_ms = local_player.ping_ms;
                        }
                    }
                    self.player_list = players;
                }
                Ok(ServerPacket::PlayerJoined { id, nickname }) => {
                    if Some(id) != self.local_id && !self.remote_players.contains_key(&id) {
                        self.remote_players.insert(
                            id,
                            RemotePlayer {
                                id,
                                nickname,
                                pos: Vec3::new(8.5, 65.0, 8.5),
                                target_pos: Vec3::new(8.5, 65.0, 8.5),
                                yaw: 0.0,
                                pitch: 0.0,
                                is_sneaking: false,
                                is_sprinting: false,
                                walk_phase: 0.0,
                                hurt_timer: 0.0,
                            },
                        );
                    }
                }
                Ok(ServerPacket::PlayerLeft { id, .. }) => {
                    self.remote_players.remove(&id);
                }
                Ok(ServerPacket::PlayerPositions { updates }) => {
                    for u in updates {
                        if Some(u.id) == self.local_id {
                            continue;
                        }
                        if let Some(rp) = self.remote_players.get_mut(&u.id) {
                            rp.target_pos = Vec3::new(u.x, u.y, u.z);
                            rp.yaw = u.yaw;
                            rp.pitch = u.pitch;
                            rp.is_sneaking = u.is_sneaking;
                            rp.is_sprinting = u.is_sprinting;
                        } else {
                            let name = self
                                .player_list
                                .iter()
                                .find(|p| p.id == u.id)
                                .map(|p| p.nickname.clone())
                                .unwrap_or_else(|| format!("Player{}", u.id));
                            self.remote_players.insert(
                                u.id,
                                RemotePlayer {
                                    id: u.id,
                                    nickname: name,
                                    pos: Vec3::new(u.x, u.y, u.z),
                                    target_pos: Vec3::new(u.x, u.y, u.z),
                                    yaw: u.yaw,
                                    pitch: u.pitch,
                                    is_sneaking: u.is_sneaking,
                                    is_sprinting: u.is_sprinting,
                                    walk_phase: 0.0,
                                    hurt_timer: 0.0,
                                },
                            );
                        }
                    }
                }
                Ok(ServerPacket::BlockChange { x, y, z, block }) => {
                    events.push(NetworkEvent::BlockChanged { x, y, z, block });
                }
                Ok(ServerPacket::ChatMessage {
                    sender,
                    text,
                    is_system,
                }) => {
                    events.push(NetworkEvent::ChatMessage {
                        sender,
                        text,
                        is_system,
                    });
                }
                Ok(ServerPacket::TimeSync { time_of_day }) => {
                    events.push(NetworkEvent::TimeSync { time_of_day });
                }
                Ok(ServerPacket::Teleport { x, y, z }) => {
                    events.push(NetworkEvent::Teleport { x, y, z });
                }
                Ok(ServerPacket::PlayerHurt {
                    victim_id,
                    attacker_id,
                    damage,
                    knockback_x,
                    knockback_z,
                }) => {
                    if let Some(rp) = self.remote_players.get_mut(&victim_id) {
                        rp.hurt_timer = 0.4;
                    }
                    events.push(NetworkEvent::PlayerHurt {
                        victim_id,
                        attacker_id,
                        damage,
                        knockback_x,
                        knockback_z,
                    });
                }
                Ok(ServerPacket::MobAuthority { id }) => {
                    self.mob_authority_id = Some(id);
                }
                Ok(ServerPacket::MobSnapshot { mobs }) => {
                    events.push(NetworkEvent::MobSnapshot { mobs });
                }
                Ok(ServerPacket::MobAttack {
                    attacker_id,
                    mob_id,
                    damage,
                    direction_x,
                    direction_y,
                    direction_z,
                }) => {
                    events.push(NetworkEvent::MobAttack {
                        attacker_id,
                        mob_id,
                        damage,
                        direction: Vec3::new(direction_x, direction_y, direction_z),
                    });
                }
                Ok(ServerPacket::MobSpawn { kind, x, y, z }) => {
                    events.push(NetworkEvent::MobSpawn {
                        kind,
                        pos: Vec3::new(x, y, z),
                    });
                }
                Ok(ServerPacket::WorldSnapshot { snapshot }) => {
                    events.push(NetworkEvent::WorldSnapshot { snapshot });
                }
                Ok(ServerPacket::PlayerData { data }) => {
                    events.push(NetworkEvent::PlayerDataLoaded { data });
                }
                Ok(ServerPacket::ModSync { mods }) => {
                    events.push(NetworkEvent::ModsSynchronized { mods });
                }
                Ok(ServerPacket::Pong { .. }) => {}
                Ok(ServerPacket::Ping { seq }) => {
                    if let Err(error) = self.send_packet(&ClientPacket::Pong { seq }) {
                        crate::log_warn!("NETWORK", "Failed to reply to server ping: {}", error);
                    }
                }
                Err(err) => {
                    let reason = format!("Invalid response from server: {err}");
                    crate::log_warn!("NETWORK", "{}", reason);
                    self.is_connected = false;
                    self.disconnect_reason = Some(reason.clone());
                    events.push(NetworkEvent::Disconnected { reason });
                    break;
                }
            }
        }

        if let Err(error) = self.flush_outgoing() {
            self.is_connected = false;
            let reason = format!("Network write failed: {error}");
            self.disconnect_reason = Some(reason.clone());
            events.push(NetworkEvent::Disconnected { reason });
        }
        events
    }
}
