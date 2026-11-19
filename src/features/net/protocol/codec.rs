//! Kodeki strukturalne pakietów, walidacja skończonych wartości `WorldSnapshot` oraz ekstrakcja ramek z bufora strumienia TCP.

use super::types::*;
use super::wire::{PacketReader, PacketWriter, WireCodec};

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

fn read_vec_u16_checked<T: WireCodec>(
    pr: &mut PacketReader<'_>,
    max_len: usize,
    err: &'static str,
) -> Result<Vec<T>, &'static str> {
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

fn read_vec_u16_capped<T: WireCodec>(
    pr: &mut PacketReader<'_>,
    cap_hint: usize,
) -> Result<Vec<T>, &'static str> {
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
            items: read_vec_u16_checked(
                pr,
                MAX_SYNCED_WORLD_ITEMS,
                "World snapshot exceeds maximum item count",
            )?,
            chests: read_vec_u16_checked(
                pr,
                MAX_SYNCED_TILE_ENTITIES,
                "World snapshot exceeds maximum chest count",
            )?,
            furnaces: read_vec_u16_checked(
                pr,
                MAX_SYNCED_TILE_ENTITIES,
                "World snapshot exceeds maximum furnace count",
            )?,
            primed_tnts: read_vec_u16_checked(
                pr,
                MAX_SYNCED_PROJECTILES,
                "World snapshot exceeds maximum primed TNT count",
            )?,
            arrows: read_vec_u16_checked(
                pr,
                MAX_SYNCED_PROJECTILES,
                "World snapshot exceeds maximum arrow count",
            )?,
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
    [
        w.timer,
        w.rain_strength,
        w.thunder_strength,
        w.lightning_timer,
        w.lightning_flash,
    ]
    .iter()
    .all(|v| v.is_finite())
        && snapshot.items.iter().all(|i| {
            [
                i.x,
                i.y,
                i.z,
                i.velocity_x,
                i.velocity_y,
                i.velocity_z,
                i.pickup_delay,
                i.age,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
        && snapshot.furnaces.iter().all(|f| {
            [f.burn_time_remaining, f.burn_time_total, f.cook_progress]
                .iter()
                .all(|v| v.is_finite())
        })
        && snapshot.primed_tnts.iter().all(|t| {
            [
                t.x,
                t.y,
                t.z,
                t.velocity_x,
                t.velocity_y,
                t.velocity_z,
                t.fuse,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
        && snapshot.arrows.iter().all(|a| {
            [
                a.x,
                a.y,
                a.z,
                a.velocity_x,
                a.velocity_y,
                a.velocity_z,
                a.lifetime,
                a.damage,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
        && w.active_lightning
            .iter()
            .all(|b| [b.x, b.y, b.z, b.timer, b.max_duration].iter().all(|v| v.is_finite()))
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
