//! Binarny protokół sieciowy multiplayer (`PROTOCOL_VERSION`, `ClientPacket`, `ServerPacket`, ramkowanie i serializacja).

mod codec;
#[cfg(test)]
mod tests;
mod types;
mod wire;

pub use codec::extract_packets;
pub use types::{
    ArrowEntry, ChestSnapshot, ClientPacket, FurnaceSnapshot, LightningEntry, MobSyncEntry,
    ModSyncEntry, PlayerData, PlayerInfo, PlayerItemStack, PlayerPosUpdate, PrimedTntEntry,
    ServerPacket, WeatherSnapshot, WorldItemEntry, WorldSnapshot, DEFAULT_SERVER_PORT,
    MAX_PACKET_SIZE, MAX_QUEUED_BYTES, MAX_SYNCED_PROJECTILES, MAX_SYNCED_TILE_ENTITIES,
    MAX_SYNCED_WORLD_ITEMS, PROTOCOL_VERSION,
};
pub use wire::{PacketReader, PacketWriter};
