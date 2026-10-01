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
}

pub struct NetworkClient {
    stream: TcpStream,
    rx_buffer: Vec<u8>,
    pub local_id: Option<u32>,
    pub remote_players: HashMap<u32, RemotePlayer>,
    pub player_list: Vec<PlayerInfo>,
    pub is_connected: bool,
    pub disconnect_reason: Option<String>,
    pub last_move_sent: Instant,
    pub ping_ms: u32,
}

impl NetworkClient {
    pub fn connect(server_addr_str: &str, nickname: &str) -> Result<Self, String> {
        let addr = server_addr_str
            .to_socket_addrs()
            .map_err(|e| format!("Invalid server address '{}': {}", server_addr_str, e))?
            .next()
            .ok_or_else(|| format!("Could not resolve address '{}'", server_addr_str))?;

        let stream = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(4))
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        let _ = stream.set_nonblocking(true);
        let _ = stream.set_nodelay(true);

        let mut client = Self {
            stream,
            rx_buffer: Vec::with_capacity(2048),
            local_id: None,
            remote_players: HashMap::new(),
            player_list: Vec::new(),
            is_connected: false,
            disconnect_reason: None,
            last_move_sent: Instant::now(),
            ping_ms: 0,
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
        self.stream
            .write_all(&bytes)
            .map_err(|e| format!("Network write failed: {}", e))
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

    pub fn update(&mut self, dt: f32) -> Vec<NetworkEvent> {
        let mut events = Vec::new();

        // Interpolate remote player positions
        for rp in self.remote_players.values_mut() {
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
        let raw_packets = extract_packets(&mut self.rx_buffer);
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
                Ok(ServerPacket::Pong { seq: _ }) => {}
                Err(err) => {
                    eprintln!("[NET CLIENT] Error decoding packet: {}", err);
                }
            }
        }

        events
    }

    pub fn disconnect(&mut self) {
        if self.is_connected {
            let _ = self.send_packet(&ClientPacket::Disconnect {
                reason: "Client disconnected".to_string(),
            });
            self.is_connected = false;
        }
    }
}
