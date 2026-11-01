//! Non-blocking multiplayer network client for connecting to a RustCraft server.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Instant;

use crate::core::math::Vec3;
use crate::net::protocol::*;

#[derive(Clone, Debug)]
pub struct RemotePlayer {
    pub id: u32,
    pub nickname: String,
    pub pos: Vec3,
    pub target_pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub walk_phase: f32,
    pub hurt_timer: f32,
}

#[derive(Clone, Debug)]
pub enum NetworkEvent {
    Connected {
        player_id: u32,
        seed: u32,
        spawn_x: f32,
        spawn_y: f32,
        spawn_z: f32,
        time_of_day: f32,
        modified_blocks: Vec<(i32, i32, i32, u8)>,
    },
    ModsSynchronized {
        mods: Vec<ModSyncEntry>,
    },
    Disconnected {
        reason: String,
    },
    BlockChanged {
        x: i32,
        y: i32,
        z: i32,
        block: u8,
    },
    ChatMessage {
        sender: String,
        text: String,
        is_system: bool,
    },
    TimeSync {
        time_of_day: f32,
    },
    Teleport {
        x: f32,
        y: f32,
        z: f32,
    },
    PlayerHurt {
        victim_id: u32,
        attacker_id: u32,
        damage: f32,
        knockback_x: f32,
        knockback_z: f32,
    },
    MobSnapshot {
        mobs: Vec<MobSyncEntry>,
    },
    MobAttack {
        attacker_id: u32,
        mob_id: u32,
        damage: f32,
        direction: Vec3,
    },
    MobSpawn {
        kind: u8,
        pos: Vec3,
    },
    WorldSnapshot {
        snapshot: WorldSnapshot,
    },
    PlayerDataLoaded {
        data: Option<PlayerData>,
    },
}

pub struct NetworkClient {
    stream: TcpStream,
    rx_buffer: Vec<u8>,
    tx_buffer: Vec<u8>,
    tx_offset: usize,
    output_failed: bool,
    pub local_id: Option<u32>,
    pub mob_authority_id: Option<u32>,
    pub remote_players: HashMap<u32, RemotePlayer>,
    pub player_list: Vec<PlayerInfo>,
    pub is_connected: bool,
    pub disconnect_reason: Option<String>,
    pub last_move_sent: Instant,
    pub ping_ms: u32,
    last_mob_snapshot_sent: Instant,
    last_world_snapshot_sent: Instant,
    last_player_data_sent: Instant,
}

impl NetworkClient {
    pub fn connect(server_addr_str: &str, nickname: &str) -> Result<Self, String> {
        let stream = if let Some(invite) = server_addr_str.strip_prefix("relay://") {
            let (relay_addr, room_code) = invite
                .rsplit_once('/')
                .ok_or_else(|| "Relay address must be relay://host:port/invitation-code".to_string())?;
            crate::net::connect_relay_guest(relay_addr, room_code)?
        } else {
            let addr = server_addr_str
                .to_socket_addrs()
                .map_err(|e| format!("Invalid server address '{}': {}", server_addr_str, e))?
                .next()
                .ok_or_else(|| format!("Could not resolve address '{}'", server_addr_str))?;

            TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(4))
                .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?
        };

        stream
            .set_nonblocking(true)
            .map_err(|error| format!("Could not configure non-blocking socket: {error}"))?;
        stream
            .set_nodelay(true)
            .map_err(|error| format!("Could not enable TCP_NODELAY: {error}"))?;

        let mut client = Self {
            stream,
            rx_buffer: Vec::with_capacity(2048),
            tx_buffer: Vec::new(),
            tx_offset: 0,
            output_failed: false,
            local_id: None,
            mob_authority_id: None,
            remote_players: HashMap::new(),
            player_list: Vec::new(),
            is_connected: false,
            disconnect_reason: None,
            last_move_sent: Instant::now(),
            ping_ms: 0,
            last_mob_snapshot_sent: Instant::now(),
            last_world_snapshot_sent: Instant::now(),
            last_player_data_sent: Instant::now(),
        };

        // Send initial handshake
        let handshake = ClientPacket::Handshake {
            protocol_version: PROTOCOL_VERSION,
            nickname: nickname.to_string(),
        };
        client.send_packet(&handshake)?;

        Ok(client)
    }

    pub fn send_packet(&mut self, packet: &ClientPacket) -> Result<(), String> {
        let bytes = packet.serialize();
        let pending_bytes = self.tx_buffer.len().saturating_sub(self.tx_offset);
        if bytes.len().saturating_sub(4) > MAX_PACKET_SIZE
            || pending_bytes.saturating_add(bytes.len()) > MAX_QUEUED_BYTES
        {
            self.output_failed = true;
            return Err("Network send queue exceeded its size limit".to_string());
        }
        if self.tx_offset >= 64 * 1024 && self.tx_offset * 2 >= self.tx_buffer.len() {
            self.tx_buffer.drain(..self.tx_offset);
            self.tx_offset = 0;
        }
        self.tx_buffer.extend_from_slice(&bytes);
        self.flush_outgoing()
    }

    fn flush_outgoing(&mut self) -> Result<(), String> {
        if self.output_failed {
            return Err("Network send queue exceeded its size limit".to_string());
        }
        while self.tx_offset < self.tx_buffer.len() {
            match self.stream.write(&self.tx_buffer[self.tx_offset..]) {
                Ok(0) => return Err("Network write returned zero bytes".to_string()),
                Ok(written) => self.tx_offset += written,
                Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(ref error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(format!("Network write failed: {error}")),
            }
        }
        self.tx_buffer.clear();
        self.tx_offset = 0;
        Ok(())
    }

    pub fn send_position(
        &mut self,
        pos: Vec3,
        yaw: f32,
        pitch: f32,
        is_sneaking: bool,
        is_sprinting: bool,
    ) {
        if !self.is_connected {
            return;
        }
        let packet = ClientPacket::PlayerMove {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            yaw,
            pitch,
            is_sneaking,
            is_sprinting,
        };
        let _ = self.send_packet(&packet);
    }

    pub fn send_block_change(&mut self, x: i32, y: i32, z: i32, block: u8) {
        if !self.is_connected {
            return;
        }
        let packet = ClientPacket::BlockChange { x, y, z, block };
        let _ = self.send_packet(&packet);
    }

    pub fn send_chat(&mut self, text: &str) {
        if !self.is_connected {
            return;
        }
        let packet = ClientPacket::ChatMessage {
            text: text.to_string(),
        };
        let _ = self.send_packet(&packet);
    }

    pub fn send_player_death(&mut self) {
        if self.is_connected {
            let _ = self.send_packet(&ClientPacket::PlayerDeath);
        }
    }

    pub fn send_player_data(&mut self, data: PlayerData, force: bool) {
        if !self.is_connected
            || (!force && self.last_player_data_sent.elapsed() < std::time::Duration::from_secs(1))
        {
            return;
        }
        match self.send_packet(&ClientPacket::PlayerData { data }) {
            Ok(()) => self.last_player_data_sent = Instant::now(),
            Err(error) => {
                crate::log_warn!("NETWORK", "Failed to send player data: {}", error);
            }
        }
    }

    pub fn send_attack_player(&mut self, target_id: u32, damage: f32) {
        if !self.is_connected {
            return;
        }
        let packet = ClientPacket::AttackPlayer { target_id, damage };
        let _ = self.send_packet(&packet);
    }

    pub fn send_attack_mob(&mut self, mob_id: u32, damage: f32) {
        if !self.is_connected || self.local_id == self.mob_authority_id {
            return;
        }
        let packet = ClientPacket::AttackMob { mob_id, damage };
        if let Err(error) = self.send_packet(&packet) {
            crate::log_warn!("NETWORK", "Failed to send mob attack: {}", error);
        }
    }

    pub fn send_spawn_mob(&mut self, kind: crate::gameplay::mob::MobKind, pos: Vec3) {
        if !self.is_connected || self.local_id == self.mob_authority_id {
            return;
        }
        let packet = ClientPacket::SpawnMob {
            kind: kind as u8,
            x: pos.x,
            y: pos.y,
            z: pos.z,
        };
        if let Err(error) = self.send_packet(&packet) {
            crate::log_warn!("NETWORK", "Failed to send mob spawn request: {}", error);
        }
    }

    pub fn send_mob_snapshot(&mut self, mob_manager: &crate::gameplay::mob::MobManager) {
        if !self.is_connected
            || self.local_id != self.mob_authority_id
            || self.last_mob_snapshot_sent.elapsed() < std::time::Duration::from_millis(100)
        {
            return;
        }
        let packet = ClientPacket::MobSnapshot {
            mobs: mob_manager.network_snapshot(),
        };
        match self.send_packet(&packet) {
            Ok(()) => self.last_mob_snapshot_sent = Instant::now(),
            Err(error) => {
                crate::log_warn!("NETWORK", "Failed to send mob snapshot: {}", error);
            }
        }
    }

    pub fn send_world_snapshot(&mut self, snapshot: WorldSnapshot) {
        if !self.can_send_world_snapshot() {
            return;
        }
        match self.send_packet(&ClientPacket::WorldSnapshot { snapshot }) {
            Ok(()) => self.last_world_snapshot_sent = Instant::now(),
            Err(error) => {
                crate::log_warn!("NETWORK", "Failed to send world snapshot: {}", error);
            }
        }
    }

    pub fn can_send_world_snapshot(&self) -> bool {
        self.is_connected
            && self.local_id == self.mob_authority_id
            && self.last_world_snapshot_sent.elapsed() >= std::time::Duration::from_millis(100)
    }

    pub fn update(&mut self, dt: f32) -> Vec<NetworkEvent> {
        let mut events = Vec::new();
        if let Err(error) = self.flush_outgoing() {
            self.is_connected = false;
            let reason = format!("Network write failed: {error}");
            self.disconnect_reason = Some(reason.clone());
            events.push(NetworkEvent::Disconnected { reason });
            return events;
        }

        // Interpolate remote player positions & decrement hurt timers
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

        // Read socket
        let mut buf = [0u8; 4096];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => {
                    self.is_connected = false;
                    let reason = self.disconnect_reason.clone().unwrap_or_else(|| "Server closed connection".to_string());
                    events.push(NetworkEvent::Disconnected { reason });
                    break;
                }
                Ok(n) => {
                    self.rx_buffer.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    self.is_connected = false;
                    let reason = format!("Network error: {}", e);
                    events.push(NetworkEvent::Disconnected { reason });
                    break;
                }
            }
        }

        // Parse extracted packets
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
                        if let Some(local_player) = players.iter().find(|player| player.id == local_id) {
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
                Ok(ServerPacket::PlayerLeft { id, nickname: _, reason: _ }) => {
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
                            // Discover remote player if not yet spawned
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
                Ok(ServerPacket::ChatMessage { sender, text, is_system }) => {
                    events.push(NetworkEvent::ChatMessage { sender, text, is_system });
                }
                Ok(ServerPacket::TimeSync { time_of_day }) => {
                    events.push(NetworkEvent::TimeSync { time_of_day });
                }
                Ok(ServerPacket::Teleport { x, y, z }) => {
                    events.push(NetworkEvent::Teleport { x, y, z });
                }
                Ok(ServerPacket::PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z }) => {
                    if let Some(rp) = self.remote_players.get_mut(&victim_id) {
                        rp.hurt_timer = 0.4;
                    }
                    events.push(NetworkEvent::PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z });
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
                Ok(ServerPacket::Pong { seq: _ }) => {}
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

    pub fn disconnect(&mut self) {
        if self.is_connected {
            let _ = self.send_packet(&ClientPacket::Disconnect {
                reason: "Client disconnected".to_string(),
            });
            let _ = self.flush_outgoing();
            self.is_connected = false;
        }
    }
}
