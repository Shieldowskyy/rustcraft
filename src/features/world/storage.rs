//! World save slot paths, thumbnail storage, progress persistence, and active world saving.
//! Delegates all OS/console/sandbox path discovery to `crate::platform::storage` (zero `#[cfg]`).

use std::path::PathBuf;

use crate::features::player::{AchievementProgress, Camera, Player, QuestProgress};
use crate::platform::storage::saves_dir;

pub use crate::platform::storage::{app_files_dir, set_custom_save_dir, soundtracks_dir};

use super::world_state::World;

/// Width and height of generated world save thumbnails in pixels.
pub const THUMB_SIZE: usize = 56;

/// Returns the absolute path to the world save binary for a given 1-based `slot` number.
pub fn save_path(slot: u8) -> String {
    saves_dir()
        .join(format!("rustcraft_world_{slot:02}.bin"))
        .to_string_lossy()
        .into_owned()
}

/// Returns the absolute path to the world thumbnail image for a given 1-based `slot` number.
pub fn thumbnail_path(slot: u8) -> String {
    saves_dir()
        .join(format!("rustcraft_world_{slot:02}.thumb"))
        .to_string_lossy()
        .into_owned()
}

/// Returns the path to achievement progress associated with a world save slot.
pub fn achievement_progress_path(slot: u8) -> PathBuf {
    saves_dir().join(format!("rustcraft_world_{slot:02}.ach"))
}

/// Loads achievement progress for a save slot, treating a missing file as a new record.
pub fn load_achievement_progress(slot: u8) -> Result<AchievementProgress, String> {
    let path = achievement_progress_path(slot);
    match std::fs::read(path) {
        Ok(bytes) => AchievementProgress::decode(&bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(AchievementProgress::default())
        }
        Err(error) => Err(format!("Could not read achievement progress: {error}")),
    }
}

/// Writes achievement progress for a save slot.
pub fn save_achievement_progress(
    slot: u8,
    progress: &AchievementProgress,
) -> Result<(), String> {
    std::fs::write(achievement_progress_path(slot), progress.encode())
        .map_err(|error| format!("Could not save achievement progress: {error}"))
}

/// Returns the path to questbook progress associated with a world save slot.
pub fn quest_progress_path(slot: u8) -> PathBuf {
    saves_dir().join(format!("rustcraft_world_{slot:02}.qst"))
}

/// Loads questbook progress for a save slot, treating a missing file as a new record.
pub fn load_quest_progress(slot: u8) -> Result<QuestProgress, String> {
    let path = quest_progress_path(slot);
    match std::fs::read(path) {
        Ok(bytes) => QuestProgress::decode(&bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(QuestProgress::default())
        }
        Err(error) => Err(format!("Could not read quest progress: {error}")),
    }
}

/// Writes questbook progress for a save slot.
pub fn save_quest_progress(slot: u8, progress: &QuestProgress) -> Result<(), String> {
    std::fs::write(quest_progress_path(slot), progress.encode())
        .map_err(|error| format!("Could not save quest progress: {error}"))
}

/// Crops and downsamples the color buffer into a 56x56 square thumbnail.
pub fn capture_thumbnail(color_buffer: &[u32], width: usize, height: usize) -> Vec<u32> {
    let mut thumb = Vec::with_capacity(THUMB_SIZE * THUMB_SIZE);
    if width == 0 || height == 0 || color_buffer.len() < width * height {
        return vec![0; THUMB_SIZE * THUMB_SIZE];
    }
    let crop_size = width.min(height);
    let start_x = (width - crop_size) / 2;
    let start_y = (height - crop_size) / 2;

    for ty in 0..THUMB_SIZE {
        let sy = start_y + ty * crop_size / THUMB_SIZE;
        for tx in 0..THUMB_SIZE {
            let sx = start_x + tx * crop_size / THUMB_SIZE;
            thumb.push(color_buffer[sy * width + sx]);
        }
    }
    thumb
}

/// Writes the thumbnail pixel array to disk for the given save `slot`.
pub fn save_thumbnail(slot: u8, thumb: &[u32]) {
    let path = thumbnail_path(slot);
    let bytes: &[u8] = bytemuck::cast_slice(thumb);
    let _ = std::fs::write(path, bytes);
}

/// Reads the thumbnail pixel array from disk for the given save `slot` if present.
pub fn load_thumbnail(slot: u8) -> Option<Vec<u32>> {
    let path = thumbnail_path(slot);
    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() == THUMB_SIZE * THUMB_SIZE * 4 {
            let mut thumb = vec![0u32; THUMB_SIZE * THUMB_SIZE];
            let slice: &mut [u8] = bytemuck::cast_slice_mut(&mut thumb);
            slice.copy_from_slice(&bytes);
            return Some(thumb);
        }
    }
    None
}

/// Returns the configuration file path for persistent settings.
pub fn settings_file_path() -> PathBuf {
    saves_dir().join("settings.bin")
}

/// Deletes a save file and its associated thumbnail and achievement progress from disk.
pub fn delete_save(slot: u8) -> Result<(), String> {
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())?;
        crate::log_info!("SAVE", "Deleted save file in slot {} ({})", slot, path);
    } else {
        crate::log_info!("SAVE", "No existing save file in slot {} ({}) to delete", slot, path);
    }
    let _ = std::fs::remove_file(thumbnail_path(slot));
    match std::fs::remove_file(achievement_progress_path(slot)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            crate::log_error!("SAVE", "Could not delete achievement progress: {}", error);
            return Err(format!("Could not delete achievement progress: {error}"));
        }
    }
    Ok(())
}

/// Saves the active world state (player position, game mode, modified chunks, and thumbnail) to `slot`.
pub fn save_current_world(
    world: &mut World,
    camera: &Camera,
    player: &Player,
    slot: u8,
    thumbnail: Option<&[u32]>,
) -> Result<(), String> {
    crate::log_info!(
        "SAVE",
        "Saving current world state to slot {} ('{}', mode: {:?}, pos: {:.1}, {:.1}, {:.1})",
        slot,
        world.name,
        player.gamemode,
        camera.position.x,
        camera.position.y,
        camera.position.z
    );
    world.player_position = camera.position;
    world.gamemode = player.gamemode;
    world.saved_player = Some(player.save_state());
    if let Some(thumb) = thumbnail {
        save_thumbnail(slot, thumb);
        crate::log_debug!("SAVE", "Saved world thumbnail image for slot {}", slot);
    }
    let path = save_path(slot);
    world.save(&path)
}
