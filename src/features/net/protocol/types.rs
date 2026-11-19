//! Definicje typów i pakietów protokołu binarnego (`ClientPacket`, `ServerPacket`, `WorldSnapshot`, `PlayerData`).

pub const PROTOCOL_VERSION: u32 = 4;
pub const DEFAULT_SERVER_PORT: u16 = 26656;
pub const MAX_PACKET_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_QUEUED_BYTES: usize = 32 * 1024 * 1024;

pub(super) const MAX_SYNCED_MOBS: usize = 512;
pub const MAX_SYNCED_WORLD_ITEMS: usize = 4096;
pub const MAX_SYNCED_TILE_ENTITIES: usize = 4096;
pub const MAX_SYNCED_PROJECTILES: usize = 2048;

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
