//! Stan klienta sieciowego (`NetworkClient`), kolejka nadawcza TCP, połączenie bezpośrednie/relay i wysyłanie pakietów.

use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Instant;

use crate::core::math::Vec3;
use crate::features::entities::{MobKind, MobManager};
use crate::features::net::protocol::*;

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
    pub(super) stream: TcpStream,
    pub(super) rx_buffer: Vec<u8>,
    pub(super) tx_buffer: Vec<u8>,
    pub(super) tx_offset: usize,
    pub(super) output_failed: bool,
    pub local_id: Option<u32>,
    pub mob_authority_id: Option<u32>,
    pub remote_players: HashMap<u32, RemotePlayer>,
    pub player_list: Vec<PlayerInfo>,
    pub is_connected: bool,
    pub disconnect_reason: Option<String>,
    pub last_move_sent: Instant,
    pub ping_ms: u32,
    pub(super) last_mob_snapshot_sent: Instant,
    pub(super) last_world_snapshot_sent: Instant,
    pub(super) last_player_data_sent: Instant,
}

impl NetworkClient {
    pub fn connect(server_addr_str: &str, nickname: &str) -> Result<Self, String> {
        let stream = if let Some(invite) = server_addr_str.strip_prefix("relay://") {
            let (relay_addr, room_code) = invite.rsplit_once('/').ok_or_else(|| {
                "Relay address must be relay://host:port/invitation-code".to_string()
            })?;
            crate::features::net::relay::connect_guest(relay_addr, room_code)?
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

    pub(super) fn flush_outgoing(&mut self) -> Result<(), String> {
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
        let _ = self.send_packet(&ClientPacket::BlockChange { x, y, z, block });
    }

    pub fn send_chat(&mut self, text: &str) {
        if !self.is_connected {
            return;
        }
        let _ = self.send_packet(&ClientPacket::ChatMessage {
            text: text.to_string(),
        });
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
        let _ = self.send_packet(&ClientPacket::AttackPlayer { target_id, damage });
    }

    pub fn send_attack_mob(&mut self, mob_id: u32, damage: f32) {
        if !self.is_connected || self.local_id == self.mob_authority_id {
            return;
        }
        if let Err(error) = self.send_packet(&ClientPacket::AttackMob { mob_id, damage }) {
            crate::log_warn!("NETWORK", "Failed to send mob attack: {}", error);
        }
    }

    pub fn send_spawn_mob(&mut self, kind: MobKind, pos: Vec3) {
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

    pub fn send_mob_snapshot(&mut self, mob_manager: &MobManager) {
        self.send_mob_snapshot_with_force(mob_manager, false);
    }

    pub fn send_mob_snapshot_now(&mut self, mob_manager: &MobManager) {
        self.send_mob_snapshot_with_force(mob_manager, true);
    }

    fn send_mob_snapshot_with_force(&mut self, mob_manager: &MobManager, force: bool) {
        if !self.is_connected
            || self.local_id != self.mob_authority_id
            || (!force
                && self.last_mob_snapshot_sent.elapsed() < std::time::Duration::from_millis(100))
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
