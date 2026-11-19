//! World save slot header inspection (`SaveMetadata`, `get_save_metadata`).
//! Reads slot file headers via `codec::read_bytes` without loading chunk payloads into memory.

use crate::core::math::Vec3;
use crate::features::player::GameMode;

use super::biome::WorldGeneratorType;
use super::codec::read_bytes;
use super::storage::{load_thumbnail, save_path};
use super::world_state::World;

const SUPPORTED_FORMATS: [&[u8]; 8] = [
    b"RUSTCRAFT1",
    b"RUSTCRAFT2",
    b"RUSTCRAFT3",
    b"RUSTCRAFT4",
    b"RUSTCRAFT5",
    b"RUSTCRAFT6",
    b"RUSTCRAFT7",
    b"RUSTCRAFT8",
];

/// Metadata header describing an individual world save slot.
#[derive(Clone, Debug)]
pub struct SaveMetadata {
    /// Slot number (1-based).
    pub slot: u8,
    /// Whether a valid save file currently exists on disk.
    pub exists: bool,
    /// Human-readable world name.
    pub name: String,
    /// Game mode (Survival or Creative).
    pub gamemode: GameMode,
    /// World generator type (Default or Superflat).
    pub generator_type: WorldGeneratorType,
    /// Numerical world seed.
    pub seed: u32,
    /// Saved player coordinates.
    pub player_pos: Vec3,
    /// Number of serialized chunk columns.
    pub chunk_count: usize,
    /// Total file size on disk in bytes.
    pub file_size_bytes: u64,
    /// Human-readable relative timestamp (e.g. "5 min ago", "yesterday").
    pub last_modified_str: String,
    /// 56x56 RGBA/ARGB preview thumbnail if available.
    pub thumbnail: Option<Vec<u32>>,
}

fn format_last_modified(path: &str) -> (u64, String) {
    let meta = std::fs::metadata(path).ok();
    let file_size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let label = meta
        .and_then(|m| m.modified().ok())
        .map(|time| {
            let secs = std::time::SystemTime::now()
                .duration_since(time)
                .unwrap_or_default()
                .as_secs();
            if secs < 60 {
                "just now".to_string()
            } else if secs < 3600 {
                format!("{} min ago", secs / 60)
            } else if secs < 86400 {
                format!("{} hours ago", secs / 3600)
            } else {
                format!("{} days ago", secs / 86400)
            }
        })
        .unwrap_or_else(|| "saved".to_string());
    (file_size, label)
}

fn read_header_name(bytes: &[u8], cursor: &mut usize, seed: u32) -> String {
    let name_len = read_bytes(bytes, cursor, 1).map(|b| b[0] as usize).unwrap_or(0);
    if name_len > 0 {
        if let Ok(nb) = read_bytes(bytes, cursor, name_len) {
            return String::from_utf8_lossy(nb).to_string();
        }
    }
    format!("World {}", seed)
}

fn parse_save_header(
    slot: u8,
    bytes: &[u8],
    file_size_bytes: u64,
    last_modified_str: &str,
    thumbnail: &Option<Vec<u32>>,
) -> Option<SaveMetadata> {
    if bytes.len() < 14 {
        return None;
    }
    let mut cursor = 0;
    let fmt = read_bytes(bytes, &mut cursor, 10).ok()?;
    if !SUPPORTED_FORMATS.contains(&fmt) {
        return None;
    }
    let seed = u32::from_le_bytes(read_bytes(bytes, &mut cursor, 4).ok()?.try_into().unwrap_or_default());

    let (gamemode, generator_type, name) = if fmt == b"RUSTCRAFT8" {
        let gm = if read_bytes(bytes, &mut cursor, 1).map(|b| b[0]).unwrap_or(0) == 1 {
            GameMode::Creative
        } else {
            GameMode::Survival
        };
        let gen = if read_bytes(bytes, &mut cursor, 1).map(|b| b[0]).unwrap_or(0) == 1 {
            WorldGeneratorType::Superflat
        } else {
            WorldGeneratorType::Default
        };
        (gm, gen, read_header_name(bytes, &mut cursor, seed))
    } else if matches!(fmt, b"RUSTCRAFT4" | b"RUSTCRAFT5" | b"RUSTCRAFT6" | b"RUSTCRAFT7") {
        let gm = if read_bytes(bytes, &mut cursor, 1).map(|b| b[0]).unwrap_or(0) == 1 {
            GameMode::Creative
        } else {
            GameMode::Survival
        };
        (gm, WorldGeneratorType::Default, read_header_name(bytes, &mut cursor, seed))
    } else {
        (GameMode::Survival, WorldGeneratorType::Default, format!("World {}", seed))
    };

    let mut player_pos = World::find_spawn_position_for_type(seed, generator_type);
    if fmt != b"RUSTCRAFT1" && bytes.len() >= cursor + 12 {
        if let (Ok(x), Ok(y), Ok(z)) = (
            read_bytes(bytes, &mut cursor, 4),
            read_bytes(bytes, &mut cursor, 4),
            read_bytes(bytes, &mut cursor, 4),
        ) {
            player_pos = Vec3::new(
                f32::from_le_bytes(x.try_into().unwrap_or_default()),
                f32::from_le_bytes(y.try_into().unwrap_or_default()),
                f32::from_le_bytes(z.try_into().unwrap_or_default()),
            );
        }
    }
    let chunk_count = read_bytes(bytes, &mut cursor, 4)
        .map(|cb| u32::from_le_bytes(cb.try_into().unwrap_or_default()) as usize)
        .unwrap_or(0);

    Some(SaveMetadata {
        slot,
        exists: true,
        name,
        gamemode,
        generator_type,
        seed,
        player_pos,
        chunk_count,
        file_size_bytes,
        last_modified_str: last_modified_str.to_string(),
        thumbnail: thumbnail.clone(),
    })
}

/// Parses the save file header on disk for `slot` and returns its `SaveMetadata`.
pub fn get_save_metadata(slot: u8) -> SaveMetadata {
    let path = save_path(slot);
    if !std::path::Path::new(&path).exists() {
        return SaveMetadata {
            slot,
            exists: false,
            name: format!("Slot {:02}", slot),
            gamemode: GameMode::Survival,
            generator_type: WorldGeneratorType::Default,
            seed: 0,
            player_pos: Vec3::new(8.0, 90.0, 8.0),
            chunk_count: 0,
            file_size_bytes: 0,
            last_modified_str: String::new(),
            thumbnail: None,
        };
    }

    let (file_size_bytes, last_modified_str) = format_last_modified(&path);
    let thumbnail = load_thumbnail(slot);

    if let Ok(bytes) = std::fs::read(&path) {
        if let Some(parsed) =
            parse_save_header(slot, &bytes, file_size_bytes, &last_modified_str, &thumbnail)
        {
            return parsed;
        }
    }

    SaveMetadata {
        slot,
        exists: true,
        name: format!("Slot {:02}", slot),
        gamemode: GameMode::Survival,
        generator_type: WorldGeneratorType::Default,
        seed: 0,
        player_pos: Vec3::new(8.0, 90.0, 8.0),
        chunk_count: 0,
        file_size_bytes,
        last_modified_str,
        thumbnail,
    }
}
