//! Binary network protocol definition, framing, packet serialization, and parsing.

pub const PROTOCOL_VERSION: u32 = 4;
pub const DEFAULT_SERVER_PORT: u16 = 26656;
pub const MAX_PACKET_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_QUEUED_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerInfo {
    pub id: u32,
    pub nickname: String,
    pub ping_ms: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerPosUpdate {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MobSyncEntry {
    pub id: u32,
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub health: f32,
    pub is_dead: bool,
    pub walk_phase: f32,
    pub is_baby: bool,
    pub wool_color: u8,
    pub is_sheared: bool,
    pub is_puffed: bool,
    pub fuse: f32,
    pub is_ignited: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerItemStack {
    pub block: u8,
    pub count: u8,
    pub durability: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub air: f32,
    pub selected_hotbar_slot: u8,
    pub hotbar: [Option<PlayerItemStack>; 10],
    pub main: [Option<PlayerItemStack>; 27],
    pub armor: [Option<PlayerItemStack>; 4],
    pub offhand: Option<PlayerItemStack>,
    pub spawn_point: Option<(i32, i32, i32)>,
}

const MAX_SYNCED_MOBS: usize = 512;
pub const MAX_SYNCED_WORLD_ITEMS: usize = 4096;
pub const MAX_SYNCED_TILE_ENTITIES: usize = 4096;
pub const MAX_SYNCED_PROJECTILES: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldItemEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub block: u8,
    pub count: u8,
    pub pickup_delay: f32,
    pub age: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChestSnapshot {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub items: [Option<PlayerItemStack>; 27],
}

#[derive(Clone, Debug, PartialEq)]
pub struct FurnaceSnapshot {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub input: Option<PlayerItemStack>,
    pub fuel: Option<PlayerItemStack>,
    pub output: Option<PlayerItemStack>,
    pub burn_time_remaining: f32,
    pub burn_time_total: f32,
    pub cook_progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrimedTntEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub fuse: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub shooter_is_player: bool,
    pub in_ground: bool,
    pub lifetime: f32,
    pub damage: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightningEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub timer: f32,
    pub max_duration: f32,
    pub seed: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeatherSnapshot {
    pub current_type: u8,
    pub timer: f32,
    pub rain_strength: f32,
    pub thunder_strength: f32,
    pub lightning_timer: f32,
    pub lightning_flash: f32,
    pub rng_state: u64,
    pub active_lightning: Vec<LightningEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldSnapshot {
    pub items: Vec<WorldItemEntry>,
    pub chests: Vec<ChestSnapshot>,
    pub furnaces: Vec<FurnaceSnapshot>,
    pub primed_tnts: Vec<PrimedTntEntry>,
    pub arrows: Vec<ArrowEntry>,
    pub weather: WeatherSnapshot,
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            chests: Vec::new(),
            furnaces: Vec::new(),
            primed_tnts: Vec::new(),
            arrows: Vec::new(),
            weather: WeatherSnapshot {
                current_type: 0,
                timer: 900.0,
                rain_strength: 0.0,
                thunder_strength: 0.0,
                lightning_timer: 8.0,
                lightning_flash: 0.0,
                rng_state: 0,
                active_lightning: Vec::new(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModSyncEntry {
    pub mod_id: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientPacket {
    Handshake {
        protocol_version: u32,
        nickname: String,
    },
    PlayerMove {
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        is_sneaking: bool,
        is_sprinting: bool,
    },
    BlockChange {
        x: i32,
        y: i32,
        z: i32,
        block: u8,
    },
    ChatMessage {
        text: String,
    },
    Ping {
        seq: u32,
    },
    Disconnect {
        reason: String,
    },
    AttackPlayer {
        target_id: u32,
        damage: f32,
    },
    AttackMob {
        mob_id: u32,
        damage: f32,
    },
    SpawnMob {
        kind: u8,
        x: f32,
        y: f32,
        z: f32,
    },
    MobSnapshot {
        mobs: Vec<MobSyncEntry>,
    },
    PlayerDeath,
    PlayerData {
        data: PlayerData,
    },
    Pong {
        seq: u32,
    },
    WorldSnapshot {
        snapshot: WorldSnapshot,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerPacket {
    LoginSuccess {
        player_id: u32,
        seed: u32,
        spawn_x: f32,
        spawn_y: f32,
        spawn_z: f32,
        time_of_day: f32,
        modified_blocks: Vec<(i32, i32, i32, u8)>,
    },
    LoginDisconnect {
        reason: String,
    },
    PlayerList {
        players: Vec<PlayerInfo>,
    },
    PlayerJoined {
        id: u32,
        nickname: String,
    },
    PlayerLeft {
        id: u32,
        nickname: String,
        reason: String,
    },
    PlayerPositions {
        updates: Vec<PlayerPosUpdate>,
    },
    BlockChange {
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
    Pong {
        seq: u32,
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
    ModSync {
        mods: Vec<ModSyncEntry>,
    },
    MobAuthority {
        id: u32,
    },
    MobSnapshot {
        mobs: Vec<MobSyncEntry>,
    },
    PlayerData {
        data: Option<PlayerData>,
    },
    Ping {
        seq: u32,
    },
    WorldSnapshot {
        snapshot: WorldSnapshot,
    },
    MobAttack {
        attacker_id: u32,
        mob_id: u32,
        damage: f32,
        direction_x: f32,
        direction_y: f32,
        direction_z: f32,
    },
    MobSpawn {
        kind: u8,
        x: f32,
        y: f32,
        z: f32,
    },
}

// -----------------------------------------------------------------------------
// Binary Packet Helpers
// -----------------------------------------------------------------------------

pub struct PacketWriter {
    pub buf: Vec<u8>,
}

impl PacketWriter {
    pub fn new(packet_id: u8) -> Self {
        let mut pw = Self {
            buf: Vec::with_capacity(64),
        };
        // Reserve 4 bytes for length prefix, then write packet_id
        pw.buf.extend_from_slice(&[0, 0, 0, 0]);
        pw.write_u8(packet_id);
        pw
    }

    pub fn write_u8(&mut self, val: u8) {
        self.buf.push(val);
    }

    pub fn write_bool(&mut self, val: bool) {
        self.buf.push(if val { 1 } else { 0 });
    }

    pub fn write_u16(&mut self, val: u16) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u32(&mut self, val: u32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_i32(&mut self, val: i32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_f32(&mut self, val: f32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_string(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(u16::MAX as usize) as u16;
        self.write_u16(len);
        self.buf.extend_from_slice(&bytes[..len as usize]);
    }

    pub fn finish(mut self) -> Vec<u8> {
        let payload_len = (self.buf.len() - 4) as u32;
        self.buf[0..4].copy_from_slice(&payload_len.to_le_bytes());
        self.buf
    }
}

pub struct PacketReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    pub fn read_u8(&mut self) -> Result<u8, &'static str> {
        if self.cursor < self.data.len() {
            let val = self.data[self.cursor];
            self.cursor += 1;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u8")
        }
    }

    pub fn read_bool(&mut self) -> Result<bool, &'static str> {
        self.read_u8().map(|v| v != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16, &'static str> {
        if self.cursor + 2 <= self.data.len() {
            let val = u16::from_le_bytes(self.data[self.cursor..self.cursor + 2].try_into().unwrap());
            self.cursor += 2;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u16")
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = u32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u32")
        }
    }

    pub fn read_u64(&mut self) -> Result<u64, &'static str> {
        if self.cursor + 8 <= self.data.len() {
            let val = u64::from_le_bytes(self.data[self.cursor..self.cursor + 8].try_into().unwrap());
            self.cursor += 8;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u64")
        }
    }

    pub fn read_i32(&mut self) -> Result<i32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = i32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading i32")
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = f32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading f32")
        }
    }

    pub fn read_string(&mut self) -> Result<String, &'static str> {
        let len = self.read_u16()? as usize;
        if self.cursor + len <= self.data.len() {
            let s = String::from_utf8_lossy(&self.data[self.cursor..self.cursor + len]).into_owned();
            self.cursor += len;
            Ok(s)
        } else {
            Err("Unexpected EOF reading string")
        }
    }
}

// -----------------------------------------------------------------------------
// Serialization & Deserialization
// -----------------------------------------------------------------------------

impl ClientPacket {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            ClientPacket::Handshake { protocol_version, nickname } => {
                let mut pw = PacketWriter::new(0x01);
                pw.write_u32(*protocol_version);
                pw.write_string(nickname);
                pw.finish()
            }
            ClientPacket::PlayerMove { x, y, z, yaw, pitch, is_sneaking, is_sprinting } => {
                let mut pw = PacketWriter::new(0x02);
                pw.write_f32(*x);
                pw.write_f32(*y);
                pw.write_f32(*z);
                pw.write_f32(*yaw);
                pw.write_f32(*pitch);
                pw.write_bool(*is_sneaking);
                pw.write_bool(*is_sprinting);
                pw.finish()
            }
            ClientPacket::BlockChange { x, y, z, block } => {
                let mut pw = PacketWriter::new(0x03);
                pw.write_i32(*x);
                pw.write_i32(*y);
                pw.write_i32(*z);
                pw.write_u8(*block);
                pw.finish()
            }
            ClientPacket::ChatMessage { text } => {
                let mut pw = PacketWriter::new(0x04);
                pw.write_string(text);
                pw.finish()
            }
            ClientPacket::Ping { seq } => {
                let mut pw = PacketWriter::new(0x05);
                pw.write_u32(*seq);
                pw.finish()
            }
            ClientPacket::Disconnect { reason } => {
                let mut pw = PacketWriter::new(0x06);
                pw.write_string(reason);
                pw.finish()
            }
            ClientPacket::AttackPlayer { target_id, damage } => {
                let mut pw = PacketWriter::new(0x07);
                pw.write_u32(*target_id);
                pw.write_f32(*damage);
                pw.finish()
            }
            ClientPacket::MobSnapshot { mobs } => {
                let mut pw = PacketWriter::new(0x08);
                write_mob_snapshot(&mut pw, mobs);
                pw.finish()
            }
            ClientPacket::PlayerDeath => PacketWriter::new(0x09).finish(),
            ClientPacket::PlayerData { data } => {
                let mut pw = PacketWriter::new(0x0A);
                write_player_data(&mut pw, data);
                pw.finish()
            }
            ClientPacket::Pong { seq } => {
                let mut pw = PacketWriter::new(0x0B);
                pw.write_u32(*seq);
                pw.finish()
            }
            ClientPacket::WorldSnapshot { snapshot } => {
                let mut pw = PacketWriter::new(0x0C);
                write_world_snapshot(&mut pw, snapshot);
                pw.finish()
            }
            ClientPacket::AttackMob { mob_id, damage } => {
                let mut pw = PacketWriter::new(0x0D);
                pw.write_u32(*mob_id);
                pw.write_f32(*damage);
                pw.finish()
            }
            ClientPacket::SpawnMob { kind, x, y, z } => {
                let mut pw = PacketWriter::new(0x0E);
                pw.write_u8(*kind);
                pw.write_f32(*x);
                pw.write_f32(*y);
                pw.write_f32(*z);
                pw.finish()
            }
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
        let mut r = PacketReader::new(data);
        let id = r.read_u8()?;
        match id {
            0x01 => {
                let protocol_version = r.read_u32()?;
                let nickname = r.read_string()?;
                Ok(ClientPacket::Handshake { protocol_version, nickname })
            }
            0x02 => {
                let x = r.read_f32()?;
                let y = r.read_f32()?;
                let z = r.read_f32()?;
                let yaw = r.read_f32()?;
                let pitch = r.read_f32()?;
                let is_sneaking = r.read_bool()?;
                let is_sprinting = r.read_bool()?;
                Ok(ClientPacket::PlayerMove { x, y, z, yaw, pitch, is_sneaking, is_sprinting })
            }
            0x03 => {
                let x = r.read_i32()?;
                let y = r.read_i32()?;
                let z = r.read_i32()?;
                let block = r.read_u8()?;
                Ok(ClientPacket::BlockChange { x, y, z, block })
            }
            0x04 => {
                let text = r.read_string()?;
                Ok(ClientPacket::ChatMessage { text })
            }
            0x05 => {
                let seq = r.read_u32()?;
                Ok(ClientPacket::Ping { seq })
            }
            0x06 => {
                let reason = r.read_string()?;
                Ok(ClientPacket::Disconnect { reason })
            }
            0x07 => {
                let target_id = r.read_u32()?;
                let damage = r.read_f32()?;
                Ok(ClientPacket::AttackPlayer { target_id, damage })
            }
            0x08 => Ok(ClientPacket::MobSnapshot {
                mobs: read_mob_snapshot(&mut r)?,
            }),
            0x09 => Ok(ClientPacket::PlayerDeath),
            0x0A => Ok(ClientPacket::PlayerData {
                data: read_player_data(&mut r)?,
            }),
            0x0B => Ok(ClientPacket::Pong { seq: r.read_u32()? }),
            0x0C => Ok(ClientPacket::WorldSnapshot {
                snapshot: read_world_snapshot(&mut r)?,
            }),
            0x0D => Ok(ClientPacket::AttackMob {
                mob_id: r.read_u32()?,
                damage: r.read_f32()?,
            }),
            0x0E => Ok(ClientPacket::SpawnMob {
                kind: r.read_u8()?,
                x: r.read_f32()?,
                y: r.read_f32()?,
                z: r.read_f32()?,
            }),
            _ => Err("Unknown client packet ID"),
        }
    }
}

impl ServerPacket {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            ServerPacket::LoginSuccess { player_id, seed, spawn_x, spawn_y, spawn_z, time_of_day, modified_blocks } => {
                let mut pw = PacketWriter::new(0x81);
                pw.write_u32(*player_id);
                pw.write_u32(*seed);
                pw.write_f32(*spawn_x);
                pw.write_f32(*spawn_y);
                pw.write_f32(*spawn_z);
                pw.write_f32(*time_of_day);
                let count = modified_blocks.len().min(u32::MAX as usize) as u32;
                pw.write_u32(count);
                for &(bx, by, bz, block) in modified_blocks {
                    pw.write_i32(bx);
                    pw.write_i32(by);
                    pw.write_i32(bz);
                    pw.write_u8(block);
                }
                pw.finish()
            }
            ServerPacket::LoginDisconnect { reason } => {
                let mut pw = PacketWriter::new(0x82);
                pw.write_string(reason);
                pw.finish()
            }
            ServerPacket::PlayerList { players } => {
                let mut pw = PacketWriter::new(0x83);
                pw.write_u16(players.len().min(u16::MAX as usize) as u16);
                for p in players {
                    pw.write_u32(p.id);
                    pw.write_string(&p.nickname);
                    pw.write_u32(p.ping_ms);
                }
                pw.finish()
            }
            ServerPacket::PlayerJoined { id, nickname } => {
                let mut pw = PacketWriter::new(0x84);
                pw.write_u32(*id);
                pw.write_string(nickname);
                pw.finish()
            }
            ServerPacket::PlayerLeft { id, nickname, reason } => {
                let mut pw = PacketWriter::new(0x85);
                pw.write_u32(*id);
                pw.write_string(nickname);
                pw.write_string(reason);
                pw.finish()
            }
            ServerPacket::PlayerPositions { updates } => {
                let mut pw = PacketWriter::new(0x86);
                pw.write_u16(updates.len().min(u16::MAX as usize) as u16);
                for u in updates {
                    pw.write_u32(u.id);
                    pw.write_f32(u.x);
                    pw.write_f32(u.y);
                    pw.write_f32(u.z);
                    pw.write_f32(u.yaw);
                    pw.write_f32(u.pitch);
                    pw.write_bool(u.is_sneaking);
                    pw.write_bool(u.is_sprinting);
                }
                pw.finish()
            }
            ServerPacket::BlockChange { x, y, z, block } => {
                let mut pw = PacketWriter::new(0x87);
                pw.write_i32(*x);
                pw.write_i32(*y);
                pw.write_i32(*z);
                pw.write_u8(*block);
                pw.finish()
            }
            ServerPacket::ChatMessage { sender, text, is_system } => {
                let mut pw = PacketWriter::new(0x88);
                pw.write_string(sender);
                pw.write_string(text);
                pw.write_bool(*is_system);
                pw.finish()
            }
            ServerPacket::TimeSync { time_of_day } => {
                let mut pw = PacketWriter::new(0x89);
                pw.write_f32(*time_of_day);
                pw.finish()
            }
            ServerPacket::Pong { seq } => {
                let mut pw = PacketWriter::new(0x8A);
                pw.write_u32(*seq);
                pw.finish()
            }
            ServerPacket::Teleport { x, y, z } => {
                let mut pw = PacketWriter::new(0x8B);
                pw.write_f32(*x);
                pw.write_f32(*y);
                pw.write_f32(*z);
                pw.finish()
            }
            ServerPacket::PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z } => {
                let mut pw = PacketWriter::new(0x8C);
                pw.write_u32(*victim_id);
                pw.write_u32(*attacker_id);
                pw.write_f32(*damage);
                pw.write_f32(*knockback_x);
                pw.write_f32(*knockback_z);
                pw.finish()
            }
            ServerPacket::ModSync { mods } => {
                let mut pw = PacketWriter::new(0x8D);
                pw.write_u16(mods.len().min(u16::MAX as usize) as u16);
                for m in mods {
                    pw.write_string(&m.mod_id);
                    let bytes = m.content.as_bytes();
                    pw.write_u32(bytes.len().min(u32::MAX as usize) as u32);
                    pw.buf.extend_from_slice(bytes);
                }
                pw.finish()
            }
            ServerPacket::MobAuthority { id } => {
                let mut pw = PacketWriter::new(0x8E);
                pw.write_u32(*id);
                pw.finish()
            }
            ServerPacket::MobSnapshot { mobs } => {
                let mut pw = PacketWriter::new(0x8F);
                write_mob_snapshot(&mut pw, mobs);
                pw.finish()
            }
            ServerPacket::PlayerData { data } => {
                let mut pw = PacketWriter::new(0x90);
                pw.write_bool(data.is_some());
                if let Some(data) = data {
                    write_player_data(&mut pw, data);
                }
                pw.finish()
            }
            ServerPacket::Ping { seq } => {
                let mut pw = PacketWriter::new(0x91);
                pw.write_u32(*seq);
                pw.finish()
            }
            ServerPacket::WorldSnapshot { snapshot } => {
                let mut pw = PacketWriter::new(0x92);
                write_world_snapshot(&mut pw, snapshot);
                pw.finish()
            }
            ServerPacket::MobAttack {
                attacker_id,
                mob_id,
                damage,
                direction_x,
                direction_y,
                direction_z,
            } => {
                let mut pw = PacketWriter::new(0x93);
                pw.write_u32(*attacker_id);
                pw.write_u32(*mob_id);
                pw.write_f32(*damage);
                pw.write_f32(*direction_x);
                pw.write_f32(*direction_y);
                pw.write_f32(*direction_z);
                pw.finish()
            }
            ServerPacket::MobSpawn { kind, x, y, z } => {
                let mut pw = PacketWriter::new(0x94);
                pw.write_u8(*kind);
                pw.write_f32(*x);
                pw.write_f32(*y);
                pw.write_f32(*z);
                pw.finish()
            }
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
        let mut r = PacketReader::new(data);
        let id = r.read_u8()?;
        match id {
            0x81 => {
                let player_id = r.read_u32()?;
                let seed = r.read_u32()?;
                let spawn_x = r.read_f32()?;
                let spawn_y = r.read_f32()?;
                let spawn_z = r.read_f32()?;
                let time_of_day = r.read_f32()?;
                let count = r.read_u32()? as usize;
                let mut modified_blocks = Vec::with_capacity(count.min(4096));
                for _ in 0..count {
                    let bx = r.read_i32()?;
                    let by = r.read_i32()?;
                    let bz = r.read_i32()?;
                    let block = r.read_u8()?;
                    modified_blocks.push((bx, by, bz, block));
                }
                Ok(ServerPacket::LoginSuccess {
                    player_id,
                    seed,
                    spawn_x,
                    spawn_y,
                    spawn_z,
                    time_of_day,
                    modified_blocks,
                })
            }
            0x82 => {
                let reason = r.read_string()?;
                Ok(ServerPacket::LoginDisconnect { reason })
            }
            0x83 => {
                let count = r.read_u16()? as usize;
                let mut players = Vec::with_capacity(count.min(1024));
                for _ in 0..count {
                    let id = r.read_u32()?;
                    let nickname = r.read_string()?;
                    let ping_ms = r.read_u32()?;
                    players.push(PlayerInfo { id, nickname, ping_ms });
                }
                Ok(ServerPacket::PlayerList { players })
            }
            0x84 => {
                let id = r.read_u32()?;
                let nickname = r.read_string()?;
                Ok(ServerPacket::PlayerJoined { id, nickname })
            }
            0x85 => {
                let id = r.read_u32()?;
                let nickname = r.read_string()?;
                let reason = r.read_string()?;
                Ok(ServerPacket::PlayerLeft { id, nickname, reason })
            }
            0x86 => {
                let count = r.read_u16()? as usize;
                let mut updates = Vec::with_capacity(count.min(1024));
                for _ in 0..count {
                    let id = r.read_u32()?;
                    let x = r.read_f32()?;
                    let y = r.read_f32()?;
                    let z = r.read_f32()?;
                    let yaw = r.read_f32()?;
                    let pitch = r.read_f32()?;
                    let is_sneaking = r.read_bool()?;
                    let is_sprinting = r.read_bool()?;
                    updates.push(PlayerPosUpdate { id, x, y, z, yaw, pitch, is_sneaking, is_sprinting });
                }
                Ok(ServerPacket::PlayerPositions { updates })
            }
            0x87 => {
                let x = r.read_i32()?;
                let y = r.read_i32()?;
                let z = r.read_i32()?;
                let block = r.read_u8()?;
                Ok(ServerPacket::BlockChange { x, y, z, block })
            }
            0x88 => {
                let sender = r.read_string()?;
                let text = r.read_string()?;
                let is_system = r.read_bool()?;
                Ok(ServerPacket::ChatMessage { sender, text, is_system })
            }
            0x89 => {
                let time_of_day = r.read_f32()?;
                Ok(ServerPacket::TimeSync { time_of_day })
            }
            0x8A => {
                let seq = r.read_u32()?;
                Ok(ServerPacket::Pong { seq })
            }
            0x8B => {
                let x = r.read_f32()?;
                let y = r.read_f32()?;
                let z = r.read_f32()?;
                Ok(ServerPacket::Teleport { x, y, z })
            }
            0x8C => {
                let victim_id = r.read_u32()?;
                let attacker_id = r.read_u32()?;
                let damage = r.read_f32()?;
                let knockback_x = r.read_f32()?;
                let knockback_z = r.read_f32()?;
                Ok(ServerPacket::PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z })
            }
            0x8D => {
                let count = r.read_u16()? as usize;
                let mut mods = Vec::with_capacity(count.min(256));
                for _ in 0..count {
                    let mod_id = r.read_string()?;
                    let len = r.read_u32()? as usize;
                    if r.remaining() < len {
                        return Err("Unexpected EOF reading mod payload content");
                    }
                    let content_bytes = &r.data[r.cursor..r.cursor + len];
                    r.cursor += len;
                    let content = String::from_utf8_lossy(content_bytes).into_owned();
                    mods.push(ModSyncEntry { mod_id, content });
                }
                Ok(ServerPacket::ModSync { mods })
            }
            0x8E => Ok(ServerPacket::MobAuthority { id: r.read_u32()? }),
            0x8F => Ok(ServerPacket::MobSnapshot {
                mobs: read_mob_snapshot(&mut r)?,
            }),
            0x90 => {
                let data = if r.read_bool()? {
                    Some(read_player_data(&mut r)?)
                } else {
                    None
                };
                Ok(ServerPacket::PlayerData { data })
            }
            0x91 => Ok(ServerPacket::Ping { seq: r.read_u32()? }),
            0x92 => Ok(ServerPacket::WorldSnapshot {
                snapshot: read_world_snapshot(&mut r)?,
            }),
            0x93 => Ok(ServerPacket::MobAttack {
                attacker_id: r.read_u32()?,
                mob_id: r.read_u32()?,
                damage: r.read_f32()?,
                direction_x: r.read_f32()?,
                direction_y: r.read_f32()?,
                direction_z: r.read_f32()?,
            }),
            0x94 => Ok(ServerPacket::MobSpawn {
                kind: r.read_u8()?,
                x: r.read_f32()?,
                y: r.read_f32()?,
                z: r.read_f32()?,
            }),
            _ => Err("Unknown server packet ID"),
        }
    }
}

fn write_mob_snapshot(writer: &mut PacketWriter, mobs: &[MobSyncEntry]) {
    let count = mobs.len().min(MAX_SYNCED_MOBS);
    writer.write_u16(count as u16);
    for mob in mobs.iter().take(count) {
        writer.write_u32(mob.id);
        writer.write_u8(mob.kind);
        writer.write_f32(mob.x);
        writer.write_f32(mob.y);
        writer.write_f32(mob.z);
        writer.write_f32(mob.yaw);
        writer.write_f32(mob.health);
        writer.write_bool(mob.is_dead);
        writer.write_f32(mob.walk_phase);
        writer.write_bool(mob.is_baby);
        writer.write_u8(mob.wool_color);
        writer.write_bool(mob.is_sheared);
        writer.write_bool(mob.is_puffed);
        writer.write_f32(mob.fuse);
        writer.write_bool(mob.is_ignited);
    }
}

fn read_mob_snapshot(reader: &mut PacketReader<'_>) -> Result<Vec<MobSyncEntry>, &'static str> {
    let count = reader.read_u16()? as usize;
    if count > MAX_SYNCED_MOBS {
        return Err("Mob snapshot exceeds maximum entity count");
    }
    let mut mobs = Vec::with_capacity(count);
    for _ in 0..count {
        mobs.push(MobSyncEntry {
            id: reader.read_u32()?,
            kind: reader.read_u8()?,
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            yaw: reader.read_f32()?,
            health: reader.read_f32()?,
            is_dead: reader.read_bool()?,
            walk_phase: reader.read_f32()?,
            is_baby: reader.read_bool()?,
            wool_color: reader.read_u8()?,
            is_sheared: reader.read_bool()?,
            is_puffed: reader.read_bool()?,
            fuse: reader.read_f32()?,
            is_ignited: reader.read_bool()?,
        });
    }
    Ok(mobs)
}

fn write_world_snapshot(writer: &mut PacketWriter, snapshot: &WorldSnapshot) {
    let items = snapshot.items.len().min(MAX_SYNCED_WORLD_ITEMS);
    writer.write_u16(items as u16);
    for item in snapshot.items.iter().take(items) {
        for value in [
            item.x,
            item.y,
            item.z,
            item.velocity_x,
            item.velocity_y,
            item.velocity_z,
        ] {
            writer.write_f32(value);
        }
        writer.write_u8(item.block);
        writer.write_u8(item.count);
        writer.write_f32(item.pickup_delay);
        writer.write_f32(item.age);
    }

    let chests = snapshot.chests.len().min(MAX_SYNCED_TILE_ENTITIES);
    writer.write_u16(chests as u16);
    for chest in snapshot.chests.iter().take(chests) {
        writer.write_i32(chest.x);
        writer.write_i32(chest.y);
        writer.write_i32(chest.z);
        for item in chest.items {
            write_optional_item(writer, item);
        }
    }

    let furnaces = snapshot.furnaces.len().min(MAX_SYNCED_TILE_ENTITIES);
    writer.write_u16(furnaces as u16);
    for furnace in snapshot.furnaces.iter().take(furnaces) {
        writer.write_i32(furnace.x);
        writer.write_i32(furnace.y);
        writer.write_i32(furnace.z);
        write_optional_item(writer, furnace.input);
        write_optional_item(writer, furnace.fuel);
        write_optional_item(writer, furnace.output);
        writer.write_f32(furnace.burn_time_remaining);
        writer.write_f32(furnace.burn_time_total);
        writer.write_f32(furnace.cook_progress);
    }

    let tnts = snapshot.primed_tnts.len().min(MAX_SYNCED_PROJECTILES);
    writer.write_u16(tnts as u16);
    for tnt in snapshot.primed_tnts.iter().take(tnts) {
        for value in [
            tnt.x,
            tnt.y,
            tnt.z,
            tnt.velocity_x,
            tnt.velocity_y,
            tnt.velocity_z,
            tnt.fuse,
        ] {
            writer.write_f32(value);
        }
    }

    let arrows = snapshot.arrows.len().min(MAX_SYNCED_PROJECTILES);
    writer.write_u16(arrows as u16);
    for arrow in snapshot.arrows.iter().take(arrows) {
        for value in [
            arrow.x,
            arrow.y,
            arrow.z,
            arrow.velocity_x,
            arrow.velocity_y,
            arrow.velocity_z,
        ] {
            writer.write_f32(value);
        }
        writer.write_bool(arrow.shooter_is_player);
        writer.write_bool(arrow.in_ground);
        writer.write_f32(arrow.lifetime);
        writer.write_f32(arrow.damage);
    }

    let weather = &snapshot.weather;
    writer.write_u8(weather.current_type);
    writer.write_f32(weather.timer);
    writer.write_f32(weather.rain_strength);
    writer.write_f32(weather.thunder_strength);
    writer.write_f32(weather.lightning_timer);
    writer.write_f32(weather.lightning_flash);
    writer.write_u64(weather.rng_state);
    let lightning = weather.active_lightning.len().min(64);
    writer.write_u8(lightning as u8);
    for bolt in weather.active_lightning.iter().take(lightning) {
        writer.write_f32(bolt.x);
        writer.write_f32(bolt.y);
        writer.write_f32(bolt.z);
        writer.write_f32(bolt.timer);
        writer.write_f32(bolt.max_duration);
        writer.write_u32(bolt.seed);
    }
}

fn read_world_snapshot(reader: &mut PacketReader<'_>) -> Result<WorldSnapshot, &'static str> {
    let item_count = reader.read_u16()? as usize;
    if item_count > MAX_SYNCED_WORLD_ITEMS {
        return Err("World snapshot exceeds maximum item count");
    }
    let mut items = Vec::with_capacity(item_count);
    for _ in 0..item_count {
        items.push(WorldItemEntry {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            velocity_x: reader.read_f32()?,
            velocity_y: reader.read_f32()?,
            velocity_z: reader.read_f32()?,
            block: reader.read_u8()?,
            count: reader.read_u8()?,
            pickup_delay: reader.read_f32()?,
            age: reader.read_f32()?,
        });
    }

    let chest_count = reader.read_u16()? as usize;
    if chest_count > MAX_SYNCED_TILE_ENTITIES {
        return Err("World snapshot exceeds maximum chest count");
    }
    let mut chests = Vec::with_capacity(chest_count);
    for _ in 0..chest_count {
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let z = reader.read_i32()?;
        let mut items = [None; 27];
        for item in &mut items {
            *item = read_optional_item(reader)?;
        }
        chests.push(ChestSnapshot { x, y, z, items });
    }

    let furnace_count = reader.read_u16()? as usize;
    if furnace_count > MAX_SYNCED_TILE_ENTITIES {
        return Err("World snapshot exceeds maximum furnace count");
    }
    let mut furnaces = Vec::with_capacity(furnace_count);
    for _ in 0..furnace_count {
        furnaces.push(FurnaceSnapshot {
            x: reader.read_i32()?,
            y: reader.read_i32()?,
            z: reader.read_i32()?,
            input: read_optional_item(reader)?,
            fuel: read_optional_item(reader)?,
            output: read_optional_item(reader)?,
            burn_time_remaining: reader.read_f32()?,
            burn_time_total: reader.read_f32()?,
            cook_progress: reader.read_f32()?,
        });
    }

    let tnt_count = reader.read_u16()? as usize;
    if tnt_count > MAX_SYNCED_PROJECTILES {
        return Err("World snapshot exceeds maximum primed TNT count");
    }
    let mut primed_tnts = Vec::with_capacity(tnt_count);
    for _ in 0..tnt_count {
        primed_tnts.push(PrimedTntEntry {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            velocity_x: reader.read_f32()?,
            velocity_y: reader.read_f32()?,
            velocity_z: reader.read_f32()?,
            fuse: reader.read_f32()?,
        });
    }

    let arrow_count = reader.read_u16()? as usize;
    if arrow_count > MAX_SYNCED_PROJECTILES {
        return Err("World snapshot exceeds maximum arrow count");
    }
    let mut arrows = Vec::with_capacity(arrow_count);
    for _ in 0..arrow_count {
        arrows.push(ArrowEntry {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            velocity_x: reader.read_f32()?,
            velocity_y: reader.read_f32()?,
            velocity_z: reader.read_f32()?,
            shooter_is_player: reader.read_bool()?,
            in_ground: reader.read_bool()?,
            lifetime: reader.read_f32()?,
            damage: reader.read_f32()?,
        });
    }

    let current_type = reader.read_u8()?;
    if current_type > 2 {
        return Err("World snapshot contains unknown weather type");
    }
    let timer = reader.read_f32()?;
    let rain_strength = reader.read_f32()?;
    let thunder_strength = reader.read_f32()?;
    let lightning_timer = reader.read_f32()?;
    let lightning_flash = reader.read_f32()?;
    let rng_state = reader.read_u64()?;
    let lightning_count = reader.read_u8()? as usize;
    if lightning_count > 64 {
        return Err("World snapshot exceeds maximum lightning count");
    }
    let mut active_lightning = Vec::with_capacity(lightning_count);
    for _ in 0..lightning_count {
        active_lightning.push(LightningEntry {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            timer: reader.read_f32()?,
            max_duration: reader.read_f32()?,
            seed: reader.read_u32()?,
        });
    }

    let snapshot = WorldSnapshot {
        items,
        chests,
        furnaces,
        primed_tnts,
        arrows,
        weather: WeatherSnapshot {
            current_type,
            timer,
            rain_strength,
            thunder_strength,
            lightning_timer,
            lightning_flash,
            rng_state,
            active_lightning,
        },
    };
    if !world_snapshot_is_finite(&snapshot) {
        return Err("World snapshot contains non-finite values");
    }
    Ok(snapshot)
}

fn world_snapshot_is_finite(snapshot: &WorldSnapshot) -> bool {
    let weather = &snapshot.weather;
    [
        weather.timer,
        weather.rain_strength,
        weather.thunder_strength,
        weather.lightning_timer,
        weather.lightning_flash,
    ]
    .iter()
    .all(|value| value.is_finite())
        && snapshot.items.iter().all(|item| {
            [
                item.x,
                item.y,
                item.z,
                item.velocity_x,
                item.velocity_y,
                item.velocity_z,
                item.pickup_delay,
                item.age,
            ]
            .iter()
            .all(|value| value.is_finite())
        })
        && snapshot.furnaces.iter().all(|furnace| {
            [
                furnace.burn_time_remaining,
                furnace.burn_time_total,
                furnace.cook_progress,
            ]
            .iter()
            .all(|value| value.is_finite())
        })
        && snapshot.primed_tnts.iter().all(|tnt| {
            [
                tnt.x,
                tnt.y,
                tnt.z,
                tnt.velocity_x,
                tnt.velocity_y,
                tnt.velocity_z,
                tnt.fuse,
            ]
            .iter()
            .all(|value| value.is_finite())
        })
        && snapshot.arrows.iter().all(|arrow| {
            [
                arrow.x,
                arrow.y,
                arrow.z,
                arrow.velocity_x,
                arrow.velocity_y,
                arrow.velocity_z,
                arrow.lifetime,
                arrow.damage,
            ]
            .iter()
            .all(|value| value.is_finite())
        })
        && weather.active_lightning.iter().all(|bolt| {
            [
                bolt.x,
                bolt.y,
                bolt.z,
                bolt.timer,
                bolt.max_duration,
            ]
            .iter()
            .all(|value| value.is_finite())
        })
}

fn write_optional_item(writer: &mut PacketWriter, item: Option<PlayerItemStack>) {
    writer.write_bool(item.is_some());
    if let Some(item) = item {
        writer.write_u8(item.block);
        writer.write_u8(item.count);
        writer.write_u16(item.durability);
    }
}

fn read_optional_item(reader: &mut PacketReader<'_>) -> Result<Option<PlayerItemStack>, &'static str> {
    if reader.read_bool()? {
        Ok(Some(PlayerItemStack {
            block: reader.read_u8()?,
            count: reader.read_u8()?,
            durability: reader.read_u16()?,
        }))
    } else {
        Ok(None)
    }
}

fn write_player_data(writer: &mut PacketWriter, data: &PlayerData) {
    writer.write_f32(data.x);
    writer.write_f32(data.y);
    writer.write_f32(data.z);
    writer.write_f32(data.health);
    writer.write_f32(data.max_health);
    writer.write_f32(data.hunger);
    writer.write_f32(data.max_hunger);
    writer.write_f32(data.saturation);
    writer.write_f32(data.exhaustion);
    writer.write_f32(data.air);
    writer.write_u8(data.selected_hotbar_slot);
    for item in data.hotbar {
        write_optional_item(writer, item);
    }
    for item in data.main {
        write_optional_item(writer, item);
    }
    for item in data.armor {
        write_optional_item(writer, item);
    }
    write_optional_item(writer, data.offhand);
    writer.write_bool(data.spawn_point.is_some());
    if let Some((x, y, z)) = data.spawn_point {
        writer.write_i32(x);
        writer.write_i32(y);
        writer.write_i32(z);
    }
}

fn read_player_data(reader: &mut PacketReader<'_>) -> Result<PlayerData, &'static str> {
    let x = reader.read_f32()?;
    let y = reader.read_f32()?;
    let z = reader.read_f32()?;
    let health = reader.read_f32()?;
    let max_health = reader.read_f32()?;
    let hunger = reader.read_f32()?;
    let max_hunger = reader.read_f32()?;
    let saturation = reader.read_f32()?;
    let exhaustion = reader.read_f32()?;
    let air = reader.read_f32()?;
    let selected_hotbar_slot = reader.read_u8()?;
    let mut hotbar = [None; 10];
    for item in &mut hotbar {
        *item = read_optional_item(reader)?;
    }
    let mut main = [None; 27];
    for item in &mut main {
        *item = read_optional_item(reader)?;
    }
    let mut armor = [None; 4];
    for item in &mut armor {
        *item = read_optional_item(reader)?;
    }
    let offhand = read_optional_item(reader)?;
    let spawn_point = if reader.read_bool()? {
        Some((reader.read_i32()?, reader.read_i32()?, reader.read_i32()?))
    } else {
        None
    };
    Ok(PlayerData {
        x,
        y,
        z,
        health,
        max_health,
        hunger,
        max_hunger,
        saturation,
        exhaustion,
        air,
        selected_hotbar_slot,
        hotbar,
        main,
        armor,
        offhand,
        spawn_point,
    })
}

/// Helper to decode complete frames from a raw incoming byte stream buffer.
pub fn extract_packets(buffer: &mut Vec<u8>) -> Result<Vec<Vec<u8>>, &'static str> {
    let mut packets = Vec::new();
    loop {
        if buffer.len() < 4 {
            break;
        }
        let payload_len = u32::from_le_bytes(buffer[0..4].try_into().unwrap()) as usize;
        if payload_len == 0 || payload_len > MAX_PACKET_SIZE {
            buffer.clear();
            return Err("Invalid packet frame length");
        }
        if buffer.len() < 4 + payload_len {
            break;
        }
        let packet_data = buffer[4..4 + payload_len].to_vec();
        buffer.drain(0..4 + payload_len);
        packets.push(packet_data);
    }
    Ok(packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_packets_roundtrip() {
        let handshake = ClientPacket::Handshake {
            protocol_version: PROTOCOL_VERSION,
            nickname: "TestMiner123".to_string(),
        };
        let mut framed = handshake.serialize();
        let packets = extract_packets(&mut framed).unwrap();
        assert_eq!(packets.len(), 1);
        let decoded = ClientPacket::deserialize(&packets[0]).unwrap();
        assert_eq!(decoded, handshake);

        let pmove = ClientPacket::PlayerMove {
            x: 10.5,
            y: 64.0,
            z: -20.25,
            yaw: 1.57,
            pitch: -0.2,
            is_sneaking: true,
            is_sprinting: false,
        };
        let mut framed2 = pmove.serialize();
        let packets2 = extract_packets(&mut framed2).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets2[0]).unwrap(), pmove);

        let attack = ClientPacket::AttackPlayer {
            target_id: 101,
            damage: 6.5,
        };
        let mut framed_atk = attack.serialize();
        let packets_atk = extract_packets(&mut framed_atk).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets_atk[0]).unwrap(), attack);

        let attack_mob = ClientPacket::AttackMob {
            mob_id: 8,
            damage: 6.5,
        };
        let mut framed_mob_attack = attack_mob.serialize();
        let packets_mob_attack = extract_packets(&mut framed_mob_attack).unwrap();
        assert_eq!(
            ClientPacket::deserialize(&packets_mob_attack[0]).unwrap(),
            attack_mob
        );

        let spawn_mob = ClientPacket::SpawnMob {
            kind: crate::gameplay::mob::MobKind::Zombie as u8,
            x: 12.5,
            y: 64.0,
            z: -7.25,
        };
        let mut framed_spawn = spawn_mob.serialize();
        let packets_spawn = extract_packets(&mut framed_spawn).unwrap();
        assert_eq!(
            ClientPacket::deserialize(&packets_spawn[0]).unwrap(),
            spawn_mob
        );

        let mob_attack = ServerPacket::MobAttack {
            attacker_id: 2,
            mob_id: 8,
            damage: 6.5,
            direction_x: 0.0,
            direction_y: 0.1,
            direction_z: 1.0,
        };
        let mut framed_server_mob_attack = mob_attack.serialize();
        let packets_server_mob_attack = extract_packets(&mut framed_server_mob_attack).unwrap();
        assert_eq!(
            ServerPacket::deserialize(&packets_server_mob_attack[0]).unwrap(),
            mob_attack
        );

        let mob_spawn = ServerPacket::MobSpawn {
            kind: crate::gameplay::mob::MobKind::Zombie as u8,
            x: 12.5,
            y: 64.0,
            z: -7.25,
        };
        let mut framed_server_spawn = mob_spawn.serialize();
        let packets_server_spawn = extract_packets(&mut framed_server_spawn).unwrap();
        assert_eq!(
            ServerPacket::deserialize(&packets_server_spawn[0]).unwrap(),
            mob_spawn
        );

        let mob_snapshot = ClientPacket::MobSnapshot {
            mobs: vec![MobSyncEntry {
                id: 8,
                kind: 2,
                x: 1.0,
                y: 64.0,
                z: -3.0,
                yaw: 0.5,
                health: 4.0,
                is_dead: false,
                walk_phase: 1.25,
                is_baby: false,
                wool_color: 0,
                is_sheared: false,
                is_puffed: false,
                fuse: 0.0,
                is_ignited: false,
            }],
        };
        let mut framed_mobs = mob_snapshot.serialize();
        let packets_mobs = extract_packets(&mut framed_mobs).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets_mobs[0]).unwrap(), mob_snapshot);

        let mut framed_death = ClientPacket::PlayerDeath.serialize();
        let packets_death = extract_packets(&mut framed_death).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets_death[0]).unwrap(), ClientPacket::PlayerDeath);
        let pong = ClientPacket::Pong { seq: 42 };
        let mut framed_pong = pong.serialize();
        let packets_pong = extract_packets(&mut framed_pong).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets_pong[0]).unwrap(), pong);

        let player_data = PlayerData {
            x: 2.0,
            y: 70.0,
            z: -4.0,
            health: 17.0,
            max_health: 20.0,
            hunger: 13.0,
            max_hunger: 20.0,
            saturation: 2.5,
            exhaustion: 0.5,
            air: 20.0,
            selected_hotbar_slot: 3,
            hotbar: [Some(PlayerItemStack {
                block: 1,
                count: 32,
                durability: 12,
            }); 10],
            main: [None; 27],
            armor: [None; 4],
            offhand: None,
            spawn_point: Some((10, 64, -8)),
        };
        let packet = ClientPacket::PlayerData {
            data: player_data.clone(),
        };
        let mut framed_data = packet.serialize();
        let packets_data = extract_packets(&mut framed_data).unwrap();
        assert_eq!(ClientPacket::deserialize(&packets_data[0]).unwrap(), packet);
    }

    #[test]
    fn test_server_packets_roundtrip() {
        let login = ServerPacket::LoginSuccess {
            player_id: 42,
            seed: 987654321,
            spawn_x: 8.5,
            spawn_y: 65.0,
            spawn_z: 8.5,
            time_of_day: 120.0,
            modified_blocks: vec![(1, 2, 3, 4), (10, 20, 30, 0)],
        };
        let mut framed = login.serialize();
        let packets = extract_packets(&mut framed).unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(ServerPacket::deserialize(&packets[0]).unwrap(), login);

        let hurt = ServerPacket::PlayerHurt {
            victim_id: 7,
            attacker_id: 3,
            damage: 4.5,
            knockback_x: 0.3,
            knockback_z: -0.2,
        };
        let mut framed_hurt = hurt.serialize();
        let packets_hurt = extract_packets(&mut framed_hurt).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_hurt[0]).unwrap(), hurt);

        let chat = ServerPacket::ChatMessage {
            sender: "SwiftMiner".to_string(),
            text: "Hello RustCraft multiplayer!".to_string(),
            is_system: false,
        };
        let mut framed_chat = chat.serialize();
        let packets_chat = extract_packets(&mut framed_chat).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_chat[0]).unwrap(), chat);

        let mod_sync = ServerPacket::ModSync {
            mods: vec![ModSyncEntry {
                mod_id: "emerald_expansion".to_string(),
                content: r#"{"id":"emerald_expansion","name":"Emerald Expansion"}"#.to_string(),
            }],
        };
        let mut framed_mod_sync = mod_sync.serialize();
        let packets_mod_sync = extract_packets(&mut framed_mod_sync).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_mod_sync[0]).unwrap(), mod_sync);

        let authority = ServerPacket::MobAuthority { id: 3 };
        let mut framed_authority = authority.serialize();
        let packets_authority = extract_packets(&mut framed_authority).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_authority[0]).unwrap(), authority);
        let ping = ServerPacket::Ping { seq: 42 };
        let mut framed_ping = ping.serialize();
        let packets_ping = extract_packets(&mut framed_ping).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_ping[0]).unwrap(), ping);

        let mob_snapshot = ServerPacket::MobSnapshot {
            mobs: vec![MobSyncEntry {
                id: 8,
                kind: 2,
                x: 1.0,
                y: 64.0,
                z: -3.0,
                yaw: 0.5,
                health: 4.0,
                is_dead: false,
                walk_phase: 1.25,
                is_baby: false,
                wool_color: 0,
                is_sheared: false,
                is_puffed: false,
                fuse: 0.0,
                is_ignited: false,
            }],
        };
        let mut framed_mobs = mob_snapshot.serialize();
        let packets_mobs = extract_packets(&mut framed_mobs).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_mobs[0]).unwrap(), mob_snapshot);

        let player_data = ServerPacket::PlayerData {
            data: Some(PlayerData {
                x: 2.0,
                y: 70.0,
                z: -4.0,
                health: 17.0,
                max_health: 20.0,
                hunger: 13.0,
                max_hunger: 20.0,
                saturation: 2.5,
                exhaustion: 0.5,
                air: 20.0,
                selected_hotbar_slot: 3,
                hotbar: [Some(PlayerItemStack {
                    block: 1,
                    count: 32,
                    durability: 12,
                }); 10],
                main: [None; 27],
                armor: [None; 4],
                offhand: None,
                spawn_point: Some((10, 64, -8)),
            }),
        };
        let mut framed_data = player_data.serialize();
        let packets_data = extract_packets(&mut framed_data).unwrap();
        assert_eq!(ServerPacket::deserialize(&packets_data[0]).unwrap(), player_data);
    }

    #[test]
    fn world_snapshot_packets_roundtrip_and_reject_non_finite_values() {
        let snapshot = WorldSnapshot {
            items: vec![WorldItemEntry {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                velocity_x: 0.1,
                velocity_y: 0.2,
                velocity_z: 0.3,
                block: 7,
                count: 4,
                pickup_delay: 0.5,
                age: 12.0,
            }],
            chests: vec![ChestSnapshot {
                x: 4,
                y: 5,
                z: 6,
                items: [None; 27],
            }],
            furnaces: vec![FurnaceSnapshot {
                x: 7,
                y: 8,
                z: 9,
                input: Some(PlayerItemStack {
                    block: 10,
                    count: 2,
                    durability: 30,
                }),
                fuel: None,
                output: None,
                burn_time_remaining: 3.0,
                burn_time_total: 4.0,
                cook_progress: 5.0,
            }],
            primed_tnts: vec![PrimedTntEntry {
                x: 1.5,
                y: 2.5,
                z: 3.5,
                velocity_x: 0.0,
                velocity_y: 2.0,
                velocity_z: 0.0,
                fuse: 1.25,
            }],
            arrows: vec![ArrowEntry {
                x: 6.0,
                y: 7.0,
                z: 8.0,
                velocity_x: 1.0,
                velocity_y: 2.0,
                velocity_z: 3.0,
                shooter_is_player: true,
                in_ground: false,
                lifetime: 0.75,
                damage: 6.0,
            }],
            weather: WeatherSnapshot {
                current_type: 2,
                timer: 20.0,
                rain_strength: 0.8,
                thunder_strength: 0.6,
                lightning_timer: 1.5,
                lightning_flash: 0.25,
                rng_state: 123456789,
                active_lightning: vec![LightningEntry {
                    x: 10.0,
                    y: 64.0,
                    z: -5.0,
                    timer: 0.1,
                    max_duration: 0.2,
                    seed: 42,
                }],
            },
        };
        let client_packet = ClientPacket::WorldSnapshot {
            snapshot: snapshot.clone(),
        };
        let mut frame = client_packet.serialize();
        let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
        assert_eq!(ClientPacket::deserialize(&payload).unwrap(), client_packet);

        let server_packet = ServerPacket::WorldSnapshot { snapshot };
        let mut frame = server_packet.serialize();
        let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
        assert_eq!(ServerPacket::deserialize(&payload).unwrap(), server_packet);

        let invalid = ClientPacket::WorldSnapshot {
            snapshot: WorldSnapshot {
                items: vec![WorldItemEntry {
                    x: f32::NAN,
                    y: 0.0,
                    z: 0.0,
                    velocity_x: 0.0,
                    velocity_y: 0.0,
                    velocity_z: 0.0,
                    block: 1,
                    count: 1,
                    pickup_delay: 0.0,
                    age: 0.0,
                }],
                ..WorldSnapshot::default()
            },
        };
        let mut frame = invalid.serialize();
        let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
        assert_eq!(
            ClientPacket::deserialize(&payload),
            Err("World snapshot contains non-finite values")
        );
    }

    #[test]
    fn packet_extraction_handles_fragmented_and_oversized_frames() {
        let mut frame = ClientPacket::Ping { seq: 42 }.serialize();
        let suffix = frame.split_off(6);
        assert!(extract_packets(&mut frame).unwrap().is_empty());
        frame.extend_from_slice(&suffix);
        assert_eq!(
            ClientPacket::deserialize(&extract_packets(&mut frame).unwrap()[0]).unwrap(),
            ClientPacket::Ping { seq: 42 }
        );

        let mut oversized = ((MAX_PACKET_SIZE as u32) + 1).to_le_bytes().to_vec();
        assert_eq!(
            extract_packets(&mut oversized),
            Err("Invalid packet frame length")
        );
        assert!(oversized.is_empty());
    }
}
