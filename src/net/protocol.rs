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

trait WireCodec: Sized {
    fn encode(&self, pw: &mut PacketWriter);
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str>;
}

macro_rules! impl_primitive_codec {
    ($($ty:ty => ($w:ident, $r:ident)),* $(,)?) => {
        $(
            impl WireCodec for $ty {
                #[inline(always)]
                fn encode(&self, pw: &mut PacketWriter) { pw.$w(*self); }
                #[inline(always)]
                fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> { pr.$r() }
            }
        )*
    };
}

impl_primitive_codec! {
    u8 => (write_u8, read_u8),
    bool => (write_bool, read_bool),
    u16 => (write_u16, read_u16),
    u32 => (write_u32, read_u32),
    u64 => (write_u64, read_u64),
    i32 => (write_i32, read_i32),
    f32 => (write_f32, read_f32),
}

impl WireCodec for String {
    #[inline(always)]
    fn encode(&self, pw: &mut PacketWriter) { pw.write_string(self); }
    #[inline(always)]
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> { pr.read_string() }
}

impl<T: WireCodec> WireCodec for Option<T> {
    #[inline(always)]
    fn encode(&self, pw: &mut PacketWriter) {
        pw.write_bool(self.is_some());
        if let Some(v) = self {
            v.encode(pw);
        }
    }
    #[inline(always)]
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        if pr.read_bool()? {
            Ok(Some(T::decode(pr)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: WireCodec + Copy, const N: usize> WireCodec for [Option<T>; N] {
    fn encode(&self, pw: &mut PacketWriter) {
        for item in self {
            item.encode(pw);
        }
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let mut arr = [None; N];
        for item in &mut arr {
            *item = Option::<T>::decode(pr)?;
        }
        Ok(arr)
    }
}

impl WireCodec for (i32, i32, i32) {
    fn encode(&self, pw: &mut PacketWriter) {
        self.0.encode(pw);
        self.1.encode(pw);
        self.2.encode(pw);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        Ok((i32::decode(pr)?, i32::decode(pr)?, i32::decode(pr)?))
    }
}

impl WireCodec for (i32, i32, i32, u8) {
    fn encode(&self, pw: &mut PacketWriter) {
        self.0.encode(pw);
        self.1.encode(pw);
        self.2.encode(pw);
        self.3.encode(pw);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        Ok((i32::decode(pr)?, i32::decode(pr)?, i32::decode(pr)?, u8::decode(pr)?))
    }
}

macro_rules! impl_struct_codec {
    ($($ty:ident { $($field:ident),+ $(,)? }),* $(,)?) => {
        $(
            impl WireCodec for $ty {
                fn encode(&self, pw: &mut PacketWriter) {
                    $(self.$field.encode(pw);)+
                }
                fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
                    Ok(Self {
                        $($field: WireCodec::decode(pr)?,)+
                    })
                }
            }
        )*
    };
}

impl_struct_codec! {
    PlayerInfo { id, nickname, ping_ms },
    PlayerPosUpdate { id, x, y, z, yaw, pitch, is_sneaking, is_sprinting },
    MobSyncEntry { id, kind, x, y, z, yaw, health, is_dead, walk_phase, is_baby, wool_color, is_sheared, is_puffed, fuse, is_ignited },
    PlayerItemStack { block, count, durability },
    PlayerData { x, y, z, health, max_health, hunger, max_hunger, saturation, exhaustion, air, selected_hotbar_slot, hotbar, main, armor, offhand, spawn_point },
    WorldItemEntry { x, y, z, velocity_x, velocity_y, velocity_z, block, count, pickup_delay, age },
    ChestSnapshot { x, y, z, items },
    FurnaceSnapshot { x, y, z, input, fuel, output, burn_time_remaining, burn_time_total, cook_progress },
    PrimedTntEntry { x, y, z, velocity_x, velocity_y, velocity_z, fuse },
    ArrowEntry { x, y, z, velocity_x, velocity_y, velocity_z, shooter_is_player, in_ground, lifetime, damage },
    LightningEntry { x, y, z, timer, max_duration, seed },
}

fn write_vec_u16<T: WireCodec>(pw: &mut PacketWriter, slice: &[T], max_len: usize) {
    let count = slice.len().min(max_len);
    pw.write_u16(count as u16);
    for item in slice.iter().take(count) {
        item.encode(pw);
    }
}

fn read_vec_u16_checked<T: WireCodec>(pr: &mut PacketReader<'_>, max_len: usize, err: &'static str) -> Result<Vec<T>, &'static str> {
    let count = pr.read_u16()? as usize;
    if count > max_len {
        return Err(err);
    }
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        vec.push(T::decode(pr)?);
    }
    Ok(vec)
}

fn read_vec_u16_capped<T: WireCodec>(pr: &mut PacketReader<'_>, cap_hint: usize) -> Result<Vec<T>, &'static str> {
    let count = pr.read_u16()? as usize;
    let mut vec = Vec::with_capacity(count.min(cap_hint));
    for _ in 0..count {
        vec.push(T::decode(pr)?);
    }
    Ok(vec)
}

impl WireCodec for Vec<MobSyncEntry> {
    fn encode(&self, pw: &mut PacketWriter) {
        write_vec_u16(pw, self, MAX_SYNCED_MOBS);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        read_vec_u16_checked(pr, MAX_SYNCED_MOBS, "Mob snapshot exceeds maximum entity count")
    }
}

impl WireCodec for Vec<PlayerInfo> {
    fn encode(&self, pw: &mut PacketWriter) {
        write_vec_u16(pw, self, u16::MAX as usize);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        read_vec_u16_capped(pr, 1024)
    }
}

impl WireCodec for Vec<PlayerPosUpdate> {
    fn encode(&self, pw: &mut PacketWriter) {
        write_vec_u16(pw, self, u16::MAX as usize);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        read_vec_u16_capped(pr, 1024)
    }
}

impl WireCodec for Vec<(i32, i32, i32, u8)> {
    fn encode(&self, pw: &mut PacketWriter) {
        let count = self.len().min(u32::MAX as usize) as u32;
        pw.write_u32(count);
        for item in self {
            item.encode(pw);
        }
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let count = pr.read_u32()? as usize;
        let mut vec = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            vec.push(WireCodec::decode(pr)?);
        }
        Ok(vec)
    }
}

impl WireCodec for Vec<ModSyncEntry> {
    fn encode(&self, pw: &mut PacketWriter) {
        pw.write_u16(self.len().min(u16::MAX as usize) as u16);
        for m in self {
            pw.write_string(&m.mod_id);
            let bytes = m.content.as_bytes();
            pw.write_u32(bytes.len().min(u32::MAX as usize) as u32);
            pw.buf.extend_from_slice(bytes);
        }
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let count = pr.read_u16()? as usize;
        let mut mods = Vec::with_capacity(count.min(256));
        for _ in 0..count {
            let mod_id = pr.read_string()?;
            let len = pr.read_u32()? as usize;
            if pr.remaining() < len {
                return Err("Unexpected EOF reading mod payload content");
            }
            let content_bytes = &pr.data[pr.cursor..pr.cursor + len];
            pr.cursor += len;
            let content = String::from_utf8_lossy(content_bytes).into_owned();
            mods.push(ModSyncEntry { mod_id, content });
        }
        Ok(mods)
    }
}

impl WireCodec for WeatherSnapshot {
    fn encode(&self, pw: &mut PacketWriter) {
        self.current_type.encode(pw);
        self.timer.encode(pw);
        self.rain_strength.encode(pw);
        self.thunder_strength.encode(pw);
        self.lightning_timer.encode(pw);
        self.lightning_flash.encode(pw);
        self.rng_state.encode(pw);
        let lightning = self.active_lightning.len().min(64);
        pw.write_u8(lightning as u8);
        for bolt in self.active_lightning.iter().take(lightning) {
            bolt.encode(pw);
        }
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let current_type = pr.read_u8()?;
        if current_type > 2 {
            return Err("World snapshot contains unknown weather type");
        }
        let timer = pr.read_f32()?;
        let rain_strength = pr.read_f32()?;
        let thunder_strength = pr.read_f32()?;
        let lightning_timer = pr.read_f32()?;
        let lightning_flash = pr.read_f32()?;
        let rng_state = pr.read_u64()?;
        let lightning_count = pr.read_u8()? as usize;
        if lightning_count > 64 {
            return Err("World snapshot exceeds maximum lightning count");
        }
        let mut active_lightning = Vec::with_capacity(lightning_count);
        for _ in 0..lightning_count {
            active_lightning.push(LightningEntry::decode(pr)?);
        }
        Ok(Self {
            current_type,
            timer,
            rain_strength,
            thunder_strength,
            lightning_timer,
            lightning_flash,
            rng_state,
            active_lightning,
        })
    }
}

impl WireCodec for WorldSnapshot {
    fn encode(&self, pw: &mut PacketWriter) {
        write_vec_u16(pw, &self.items, MAX_SYNCED_WORLD_ITEMS);
        write_vec_u16(pw, &self.chests, MAX_SYNCED_TILE_ENTITIES);
        write_vec_u16(pw, &self.furnaces, MAX_SYNCED_TILE_ENTITIES);
        write_vec_u16(pw, &self.primed_tnts, MAX_SYNCED_PROJECTILES);
        write_vec_u16(pw, &self.arrows, MAX_SYNCED_PROJECTILES);
        self.weather.encode(pw);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let snapshot = Self {
            items: read_vec_u16_checked(pr, MAX_SYNCED_WORLD_ITEMS, "World snapshot exceeds maximum item count")?,
            chests: read_vec_u16_checked(pr, MAX_SYNCED_TILE_ENTITIES, "World snapshot exceeds maximum chest count")?,
            furnaces: read_vec_u16_checked(pr, MAX_SYNCED_TILE_ENTITIES, "World snapshot exceeds maximum furnace count")?,
            primed_tnts: read_vec_u16_checked(pr, MAX_SYNCED_PROJECTILES, "World snapshot exceeds maximum primed TNT count")?,
            arrows: read_vec_u16_checked(pr, MAX_SYNCED_PROJECTILES, "World snapshot exceeds maximum arrow count")?,
            weather: WeatherSnapshot::decode(pr)?,
        };
        if !world_snapshot_is_finite(&snapshot) {
            return Err("World snapshot contains non-finite values");
        }
        Ok(snapshot)
    }
}

fn world_snapshot_is_finite(snapshot: &WorldSnapshot) -> bool {
    let w = &snapshot.weather;
    [w.timer, w.rain_strength, w.thunder_strength, w.lightning_timer, w.lightning_flash].iter().all(|v| v.is_finite())
        && snapshot.items.iter().all(|i| [i.x, i.y, i.z, i.velocity_x, i.velocity_y, i.velocity_z, i.pickup_delay, i.age].iter().all(|v| v.is_finite()))
        && snapshot.furnaces.iter().all(|f| [f.burn_time_remaining, f.burn_time_total, f.cook_progress].iter().all(|v| v.is_finite()))
        && snapshot.primed_tnts.iter().all(|t| [t.x, t.y, t.z, t.velocity_x, t.velocity_y, t.velocity_z, t.fuse].iter().all(|v| v.is_finite()))
        && snapshot.arrows.iter().all(|a| [a.x, a.y, a.z, a.velocity_x, a.velocity_y, a.velocity_z, a.lifetime, a.damage].iter().all(|v| v.is_finite()))
        && w.active_lightning.iter().all(|b| [b.x, b.y, b.z, b.timer, b.max_duration].iter().all(|v| v.is_finite()))
}

macro_rules! impl_packet_codec {
    ($enum_ty:ident, $unknown_err:literal, $( $opcode:literal => $variant:ident $( { $($field:ident),+ } )? ),* $(,)?) => {
        impl $enum_ty {
            pub fn serialize(&self) -> Vec<u8> {
                match self {
                    $(
                        Self::$variant $( { $($field),+ } )? => {
                            #[allow(unused_mut)]
                            let mut pw = PacketWriter::new($opcode);
                            $( $( $field.encode(&mut pw); )+ )?
                            pw.finish()
                        }
                    )*
                }
            }

            pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
                let mut pr = PacketReader::new(data);
                let id = pr.read_u8()?;
                match id {
                    $(
                        $opcode => Ok(Self::$variant $( { $($field: WireCodec::decode(&mut pr)?),+ } )?),
                    )*
                    _ => Err($unknown_err),
                }
            }
        }
    };
}

impl_packet_codec! {
    ClientPacket, "Unknown client packet ID",
    0x01 => Handshake { protocol_version, nickname },
    0x02 => PlayerMove { x, y, z, yaw, pitch, is_sneaking, is_sprinting },
    0x03 => BlockChange { x, y, z, block },
    0x04 => ChatMessage { text },
    0x05 => Ping { seq },
    0x06 => Disconnect { reason },
    0x07 => AttackPlayer { target_id, damage },
    0x08 => MobSnapshot { mobs },
    0x09 => PlayerDeath,
    0x0A => PlayerData { data },
    0x0B => Pong { seq },
    0x0C => WorldSnapshot { snapshot },
    0x0D => AttackMob { mob_id, damage },
    0x0E => SpawnMob { kind, x, y, z },
}

impl_packet_codec! {
    ServerPacket, "Unknown server packet ID",
    0x81 => LoginSuccess { player_id, seed, spawn_x, spawn_y, spawn_z, time_of_day, modified_blocks },
    0x82 => LoginDisconnect { reason },
    0x83 => PlayerList { players },
    0x84 => PlayerJoined { id, nickname },
    0x85 => PlayerLeft { id, nickname, reason },
    0x86 => PlayerPositions { updates },
    0x87 => BlockChange { x, y, z, block },
    0x88 => ChatMessage { sender, text, is_system },
    0x89 => TimeSync { time_of_day },
    0x8A => Pong { seq },
    0x8B => Teleport { x, y, z },
    0x8C => PlayerHurt { victim_id, attacker_id, damage, knockback_x, knockback_z },
    0x8D => ModSync { mods },
    0x8E => MobAuthority { id },
    0x8F => MobSnapshot { mobs },
    0x90 => PlayerData { data },
    0x91 => Ping { seq },
    0x92 => WorldSnapshot { snapshot },
    0x93 => MobAttack { attacker_id, mob_id, damage, direction_x, direction_y, direction_z },
    0x94 => MobSpawn { kind, x, y, z },
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
