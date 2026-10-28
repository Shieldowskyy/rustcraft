//! Audio sound definitions, categories, samples, and WAV parsing.

use std::sync::Arc;
use crate::world::block::BlockId;

/// Sound category for independent volume adjustments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Sfx,
    Music,
    Ambient,
    Ui,
}

impl SoundCategory {
    pub const ALL: [SoundCategory; 5] = [
        SoundCategory::Master,
        SoundCategory::Sfx,
        SoundCategory::Music,
        SoundCategory::Ambient,
        SoundCategory::Ui,
    ];

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        match self {
            SoundCategory::Master => "Master",
            SoundCategory::Sfx => "SFX",
            SoundCategory::Music => "Music",
            SoundCategory::Ambient => "Ambient",
            SoundCategory::Ui => "UI",
        }
    }
}

/// Unique identifiers for every discrete sound effect in RustCraft.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundId {
    // Footsteps
    StepGrass,
    StepDirt,
    StepStone,
    StepWood,
    StepSand,
    StepGravel,
    StepWater,

    // Block interactions (Hit / Digging)
    DigGrass,
    DigDirt,
    DigStone,
    DigWood,
    DigSand,

    // Block interactions (Break)
    BreakGrass,
    BreakDirt,
    BreakStone,
    BreakWood,
    BreakGlass,
    BreakSand,

    // Block interactions (Place)
    PlaceGrass,
    PlaceDirt,
    PlaceStone,
    PlaceWood,
    PlaceSand,
    PlaceGeneric,

    // Player events
    PlayerJump,
    PlayerLand,
    PlayerHurt,
    PlayerSplash,
    PlayerEat,

    // World & UI events
    ItemPop,
    UiClick,
    UiHover,
    ChestOpen,
    ChestClose,
    DoorOpen,
    DoorClose,
    Explosion,

    // Mob sounds (Idle, Hurt, Death)
    PigIdle,
    PigHurt,
    PigDeath,
    CowIdle,
    CowHurt,
    CowDeath,
    ChickenIdle,
    ChickenHurt,
    ChickenDeath,
    ZombieIdle,
    ZombieHurt,
    ZombieDeath,
    SkeletonIdle,
    SkeletonHurt,
    SkeletonDeath,
    CreeperHiss,
    CreeperHurt,
    CreeperDeath,
    SheepIdle,
    SheepHurt,
    SheepDeath,
    SheepShear,
    VillagerIdle,
    VillagerHurt,
    VillagerDeath,
    VillagerTrade,
    VillagerNo,
    IronGolemIdle,
    IronGolemHurt,
    IronGolemDeath,
    IronGolemAttack,

    // Aquatic & Fishing
    FishFlop,
    DolphinIdle,
    DolphinHurt,
    DolphinDeath,
    DolphinJump,
    FishingCast,
    FishingReel,
    FishCatch,

    // Atmospheric & Ambient
    AmbientWind,
    AmbientCave,
    AmbientWater,
    AmbientLeaves,
    AmbientCaveRocks,
    StepWoodCreak,
    AmbientDesertSand,

    // Weather & Atmosphere
    Rain,
    ThunderClose,
    ThunderDistant,
}

impl SoundId {
    /// Returns the sound category this sound belongs to.
    #[inline(always)]
    pub fn category(&self) -> SoundCategory {
        match self {
            SoundId::StepGrass
            | SoundId::StepDirt
            | SoundId::StepStone
            | SoundId::StepWood
            | SoundId::StepSand
            | SoundId::StepGravel
            | SoundId::StepWater
            | SoundId::DigGrass
            | SoundId::DigDirt
            | SoundId::DigStone
            | SoundId::DigWood
            | SoundId::DigSand
            | SoundId::BreakGrass
            | SoundId::BreakDirt
            | SoundId::BreakStone
            | SoundId::BreakWood
            | SoundId::BreakGlass
            | SoundId::BreakSand
            | SoundId::PlaceGrass
            | SoundId::PlaceDirt
            | SoundId::PlaceStone
            | SoundId::PlaceWood
            | SoundId::PlaceSand
            | SoundId::PlaceGeneric
            | SoundId::PlayerJump
            | SoundId::PlayerLand
            | SoundId::PlayerHurt
            | SoundId::PlayerSplash
            | SoundId::PlayerEat
            | SoundId::ItemPop
            | SoundId::ChestOpen
            | SoundId::ChestClose
            | SoundId::DoorOpen
            | SoundId::DoorClose
            | SoundId::Explosion
            | SoundId::PigIdle
            | SoundId::PigHurt
            | SoundId::PigDeath
            | SoundId::CowIdle
            | SoundId::CowHurt
            | SoundId::CowDeath
            | SoundId::ChickenIdle
            | SoundId::ChickenHurt
            | SoundId::ChickenDeath
            | SoundId::ZombieIdle
            | SoundId::ZombieHurt
            | SoundId::ZombieDeath
            | SoundId::SkeletonIdle
            | SoundId::SkeletonHurt
            | SoundId::SkeletonDeath
            | SoundId::CreeperHiss
            | SoundId::CreeperHurt
            | SoundId::CreeperDeath
            | SoundId::SheepIdle
            | SoundId::SheepHurt
            | SoundId::SheepDeath
            | SoundId::SheepShear
            | SoundId::VillagerIdle
            | SoundId::VillagerHurt
            | SoundId::VillagerDeath
            | SoundId::VillagerTrade
            | SoundId::VillagerNo
            | SoundId::IronGolemIdle
            | SoundId::IronGolemHurt
            | SoundId::IronGolemDeath
            | SoundId::IronGolemAttack
            | SoundId::FishFlop
            | SoundId::DolphinIdle
            | SoundId::DolphinHurt
            | SoundId::DolphinDeath
            | SoundId::DolphinJump
            | SoundId::FishingCast
            | SoundId::FishingReel
            | SoundId::FishCatch
            | SoundId::StepWoodCreak => SoundCategory::Sfx,

            SoundId::UiClick | SoundId::UiHover => SoundCategory::Ui,

            SoundId::AmbientWind
            | SoundId::AmbientCave
            | SoundId::AmbientWater
            | SoundId::AmbientLeaves
            | SoundId::AmbientCaveRocks
            | SoundId::AmbientDesertSand
            | SoundId::Rain
            | SoundId::ThunderClose
            | SoundId::ThunderDistant => SoundCategory::Ambient,
        }
    }

    /// Determines the appropriate footstep sound for walking on a given block.
    #[inline]
    pub fn from_block_step(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::StepGrass;
        }
        match block {
            BlockId::Grass | BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => {
                SoundId::StepGrass
            }
            BlockId::Dirt | BlockId::Mud => {
                SoundId::StepDirt
            }
            BlockId::Stone
            | BlockId::Bedrock
            | BlockId::CoalOre
            | BlockId::IronOre
            | BlockId::GoldOre
            | BlockId::Furnace
            | BlockId::FurnaceLit
            | BlockId::IronBlock
            | BlockId::GoldBlock
            | BlockId::CoalBlock => SoundId::StepStone,
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen => SoundId::StepWood,
            BlockId::Sand => SoundId::StepSand,
            BlockId::Water => SoundId::StepWater,
            _ => SoundId::StepStone,
        }
    }

    /// Determines the block digging/hitting sound.
    #[inline]
    pub fn from_block_dig(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::DigGrass;
        }
        match block {
            BlockId::Grass | BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => {
                SoundId::DigGrass
            }
            BlockId::Dirt | BlockId::Mud => {
                SoundId::DigDirt
            }
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen => SoundId::DigWood,
            BlockId::Glass => SoundId::BreakGlass,
            BlockId::Sand => SoundId::DigSand,
            _ => SoundId::DigStone,
        }
    }

    /// Determines the block breaking sound.
    #[inline]
    pub fn from_block_break(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::BreakGrass;
        }
        match block {
            BlockId::Grass | BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => {
                SoundId::BreakGrass
            }
            BlockId::Dirt | BlockId::Mud => {
                SoundId::BreakDirt
            }
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen => SoundId::BreakWood,
            BlockId::Glass => SoundId::BreakGlass,
            BlockId::Sand => SoundId::BreakSand,
            _ => SoundId::BreakStone,
        }
    }

    /// Determines the block placing sound.
    #[inline]
    pub fn from_block_place(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::PlaceGrass;
        }
        match block {
            BlockId::Grass | BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => {
                SoundId::PlaceGrass
            }
            BlockId::Dirt | BlockId::Mud => {
                SoundId::PlaceDirt
            }
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door => SoundId::PlaceWood,
            BlockId::Sand => SoundId::PlaceSand,
            _ => SoundId::PlaceStone,
        }
    }
}

/// An in-memory audio sample buffer normalized to 32-bit floating point [-1.0, 1.0].
///
/// Kept inside an `Arc<[f32]>` so multiple polyphonic voices can reference the same sound
/// without cloning or copying memory.
#[derive(Clone, Debug)]
pub struct SoundSample {
    /// Interleaved or mono 32-bit float samples.
    pub data: Arc<[f32]>,
    /// Sampling rate in Hz (e.g. 44100 or 48000).
    pub sample_rate: u32,
    /// Channel count (1 for mono, 2 for stereo).
    pub channels: u16,
}

impl SoundSample {
    /// Creates a new sound sample from raw float samples.
    pub fn new(data: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            data: data.into(),
            sample_rate,
            channels: channels.max(1),
        }
    }

    /// Duration in seconds.
    #[inline]
    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            0.0
        } else {
            (self.data.len() / self.channels as usize) as f32 / self.sample_rate as f32
        }
    }

    /// Parses a standard RIFF/WAVE PCM or IEEE Float byte buffer.
    ///
    /// Supports:
    /// - 8-bit unsigned PCM
    /// - 16-bit signed PCM (Little-Endian)
    /// - 24-bit signed PCM (Little-Endian)
    /// - 32-bit signed PCM (Little-Endian)
    /// - 32-bit IEEE float (Little-Endian)
    pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 44 {
            return Err("WAV header too short".to_string());
        }

        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err("Invalid RIFF/WAVE magic header".to_string());
        }

        let mut offset = 12;
        let mut audio_format = 1u16; // 1 = PCM, 3 = IEEE Float
        let mut channels = 1u16;
        let mut sample_rate = 44100u32;
        let mut bits_per_sample = 16u16;
        let mut data_start = 0usize;
        let mut data_len = 0usize;

        while offset + 8 <= bytes.len() {
            let chunk_id = &bytes[offset..offset + 4];
            let chunk_size = u32::from_le_bytes(
                bytes[offset + 4..offset + 8]
                    .try_into()
                    .map_err(|_| "Malformed chunk size".to_string())?,
            ) as usize;
            offset += 8;

            if chunk_id == b"fmt " {
                if chunk_size < 16 || offset + chunk_size > bytes.len() {
                    return Err("Malformed fmt chunk".to_string());
                }
                audio_format = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                channels = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
                sample_rate = u32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                bits_per_sample = u16::from_le_bytes([bytes[offset + 14], bytes[offset + 15]]);
            } else if chunk_id == b"data" {
                data_start = offset;
                data_len = chunk_size.min(bytes.len() - offset);
                break;
            }

            offset += (chunk_size + 1) & !1; // Word align
        }

        if data_start == 0 || data_len == 0 {
            return Err("Missing WAV 'data' chunk".to_string());
        }

        let pcm_slice = &bytes[data_start..data_start + data_len];
        let mut samples = Vec::new();

        match (audio_format, bits_per_sample) {
            (1, 8) => {
                // 8-bit unsigned PCM
                samples.reserve(pcm_slice.len());
                for &b in pcm_slice {
                    samples.push((b as f32 - 128.0) / 128.0);
                }
            }
            (1, 16) => {
                // 16-bit signed PCM
                let count = pcm_slice.len() / 2;
                samples.reserve(count);
                for i in 0..count {
                    let val = i16::from_le_bytes([pcm_slice[i * 2], pcm_slice[i * 2 + 1]]);
                    samples.push(val as f32 / 32768.0);
                }
            }
            (1, 24) => {
                // 24-bit signed PCM
                let count = pcm_slice.len() / 3;
                samples.reserve(count);
                for i in 0..count {
                    let b0 = pcm_slice[i * 3];
                    let b1 = pcm_slice[i * 3 + 1];
                    let b2 = pcm_slice[i * 3 + 2];
                    // Sign-extend 24-bit to 32-bit
                    let val = ((b2 as i8 as i32) << 16) | ((b1 as i32) << 8) | (b0 as i32);
                    samples.push(val as f32 / 8388608.0);
                }
            }
            (1, 32) => {
                // 32-bit signed PCM
                let count = pcm_slice.len() / 4;
                samples.reserve(count);
                for i in 0..count {
                    let val = i32::from_le_bytes([
                        pcm_slice[i * 4],
                        pcm_slice[i * 4 + 1],
                        pcm_slice[i * 4 + 2],
                        pcm_slice[i * 4 + 3],
                    ]);
                    samples.push(val as f32 / 2147483648.0);
                }
            }
            (3, 32) => {
                // 32-bit IEEE Float
                let count = pcm_slice.len() / 4;
                samples.reserve(count);
                for i in 0..count {
                    let val = f32::from_le_bytes([
                        pcm_slice[i * 4],
                        pcm_slice[i * 4 + 1],
                        pcm_slice[i * 4 + 2],
                        pcm_slice[i * 4 + 3],
                    ]);
                    samples.push(val.clamp(-1.0, 1.0));
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported WAV format: format={}, bits={}",
                    audio_format, bits_per_sample
                ));
            }
        }

        Ok(Self::new(samples, sample_rate, channels))
    }

    /// Decodes an Ogg Opus audio byte buffer (.ogg or .opus) into float samples.
    ///
    /// Pure Rust implementation (RFC 6716 / RFC 7845) without native C dependencies,
    /// supporting Stereo, 64kbps and any Opus configuration across all target platforms.
    pub fn from_ogg_opus_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = opus_pure::OggOpusReader::new(cursor)
            .map_err(|e| format!("Ogg Opus header error: {:?}", e))?;

        let channels = reader.head().channel_count as usize;
        let mut decoder = reader.head().decoder(48000)
            .map_err(|e| format!("Opus decoder init error: {:?}", e))?;

        let mut samples = Vec::new();
        let max_samples = opus_pure::MAX_PACKET_SAMPLES;
        let mut pcm_buf = vec![0.0f32; max_samples * channels];

        while let Some(packet) = reader.read_packet()
            .map_err(|e| format!("Ogg Opus packet read error: {:?}", e))?
        {
            let samples_per_channel = decoder
                .decode(&packet.data, max_samples, &mut pcm_buf)
                .map_err(|e| format!("Opus packet decode error: {:?}", e))?;
            let total_samples = samples_per_channel * channels;
            samples.extend_from_slice(&pcm_buf[..total_samples]);
        }

        if samples.is_empty() {
            return Err("Empty Ogg Opus audio stream".to_string());
        }

        // Discard encoder roll-in delay (pre_skip) per RFC 7845
        let pre_skip = (reader.head().pre_skip as usize * channels).min(samples.len());
        if pre_skip > 0 {
            samples.drain(0..pre_skip);
        }

        Ok(Self::new(samples, 48000, channels as u16))
    }

    /// Decodes an Ogg Vorbis audio byte buffer (.ogg) into float samples.
    pub fn from_ogg_vorbis_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = lewton::inside_ogg::OggStreamReader::new(cursor)
            .map_err(|e| format!("OGG Vorbis header decode error: {:?}", e))?;

        let sample_rate = reader.ident_hdr.audio_sample_rate;
        let channels = reader.ident_hdr.audio_channels as u16;

        let mut samples = Vec::new();
        while let Some(packet) = reader
            .read_dec_packet_itl()
            .map_err(|e| format!("OGG Vorbis packet decode error: {:?}", e))?
        {
            samples.reserve(packet.len());
            for s in packet {
                samples.push(s as f32 / 32768.0);
            }
        }

        if samples.is_empty() {
            return Err("Empty OGG Vorbis audio stream".to_string());
        }

        Ok(Self::new(samples, sample_rate, channels))
    }

    /// Decodes an Ogg container (automatically detecting Opus or Vorbis).
    pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.windows(8).take(128).any(|w| w == b"OpusHead") {
            Self::from_ogg_opus_bytes(bytes)
        } else {
            Self::from_ogg_vorbis_bytes(bytes)
                .or_else(|_| Self::from_ogg_opus_bytes(bytes))
        }
    }

    /// Automatically detects file format from magic bytes (Opus, Vorbis, or WAV) and decodes it.
    pub fn from_file_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
            Self::from_ogg_bytes(bytes)
        } else if bytes.len() >= 4 && &bytes[0..4] == b"RIFF" {
            Self::from_wav_bytes(bytes)
        } else {
            Self::from_ogg_bytes(bytes).or_else(|_| Self::from_wav_bytes(bytes))
        }
    }
}


