//! World persistence, save file discovery, slot metadata, and thumbnail storage.

use crate::graphics::camera::Camera;
use crate::world::World;

static CUSTOM_SAVES_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Overrides the save files directory at runtime (essential for mobile sandboxed storage).
pub fn set_custom_save_dir(dir: std::path::PathBuf) {
    let _ = std::fs::create_dir_all(&dir);
    let _ = CUSTOM_SAVES_DIR.set(dir);
}

/// Returns the root data / files directory for application data (parent of saves).
pub fn app_files_dir() -> std::path::PathBuf {
    saves_dir().parent().map(|p| p.to_path_buf()).unwrap_or_else(saves_dir)
}

/// Returns the active `soundtracks/` directory path on disk.
pub fn soundtracks_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = app_files_dir().join("soundtracks");
        if dir.exists() {
            return dir;
        }
    }
    #[cfg(target_os = "vita")]
    {
        let custom_dir = std::path::PathBuf::from("ux0:data/rustcraft/soundtracks");
        if custom_dir.exists() {
            return custom_dir;
        }
        let app_dir = std::path::PathBuf::from("app0:soundtracks");
        if app_dir.exists() {
            return app_dir;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("soundtracks");
            if dir.is_dir() {
                return dir;
            }
        }
    }
    std::path::PathBuf::from("soundtracks")
}

/// Returns the directory path where world save files are stored.
fn saves_dir() -> std::path::PathBuf {
    if let Some(dir) = CUSTOM_SAVES_DIR.get() {
        return dir.clone();
    }
    use std::sync::OnceLock;
    static SAVES_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

    SAVES_DIR.get_or_init(|| {
        if let Some(dir) = exe_saves_dir_if_writable() {
            return dir;
        }
        fallback_saves_dir()
    }).clone()
}

/// Returns the `saves/` subdirectory adjacent to the executable if it exists and is writable.
fn exe_saves_dir_if_writable() -> Option<std::path::PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    let dir = exe_dir.join("saves");

    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok()?;
    }

    if is_dir_writable(&dir) {
        Some(dir)
    } else {
        None
    }
}

/// Fallback save directory per platform (Android files, ux0 on PS Vita, APPDATA on Windows, ~/.config on Linux).
fn fallback_saves_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = std::path::PathBuf::from("/data/data/com.rustcraft.game/files/saves");
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    #[cfg(target_os = "vita")]
    {
        let dir = std::path::PathBuf::from("ux0:data/rustcraft/saves");
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    #[cfg(any(target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
    {
        let candidates = [
            "game:/saves",
            "Usb0:/RustCraft/saves",
            "Hdd:/RustCraft/saves",
            "uda0:/RustCraft/saves",
            "sda0:/RustCraft/saves",
            "./saves",
        ];
        for path_str in candidates {
            let dir = std::path::PathBuf::from(path_str);
            if std::fs::create_dir_all(&dir).is_ok() && is_dir_writable(&dir) {
                return dir;
            }
        }
        return std::path::PathBuf::from("game:/saves");
    }
    #[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
    {
        #[cfg(windows)]
        {
            if let Some(appdata) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) {
                let dir = appdata.join("rustcraft").join("saves");
                if std::fs::create_dir_all(&dir).is_ok() {
                    return dir;
                }
            }
        }
        if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
            let dir = home.join(".config").join("rustcraft").join("saves");
            if std::fs::create_dir_all(&dir).is_ok() {
                return dir;
            }
        }
        std::path::PathBuf::from(".")
    }
}

/// Checks whether write permissions are granted in `dir` by creating and deleting a temporary test file.
fn is_dir_writable(dir: &std::path::Path) -> bool {
    let test_path = dir.join(".rustcraft_write_test");
    match std::fs::File::create(&test_path) {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_path);
            true
        }
        Err(_) => false,
    }
}

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

/// Width and height of generated world save thumbnails in pixels.
pub const THUMB_SIZE: usize = 56;

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
            let pixel = color_buffer[sy * width + sx];
            thumb.push(pixel);
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
pub fn settings_file_path() -> std::path::PathBuf {
    saves_dir().join("settings.bin")
}

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
    pub gamemode: crate::gameplay::player::GameMode,
    /// Numerical world seed.
    pub seed: u32,
    /// Saved player coordinates.
    pub player_pos: crate::core::math::Vec3,
    /// Number of serialized chunk columns.
    pub chunk_count: usize,
    /// Total file size on disk in bytes.
    pub file_size_bytes: u64,
    /// Human-readable relative timestamp (e.g. "5 min ago", "yesterday").
    pub last_modified_str: String,
    /// 56x56 RGBA/ARGB preview thumbnail if available.
    pub thumbnail: Option<Vec<u32>>,
}

/// Parses the save file header on disk for `slot` and returns its `SaveMetadata`.
pub fn get_save_metadata(slot: u8) -> SaveMetadata {
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return SaveMetadata {
            slot,
            exists: false,
            name: format!("Slot {:02}", slot),
            gamemode: crate::gameplay::player::GameMode::Survival,
            seed: 0,
            player_pos: crate::core::math::Vec3::new(8.0, 90.0, 8.0),
            chunk_count: 0,
            file_size_bytes: 0,
            last_modified_str: String::new(),
            thumbnail: None,
        };
    }

    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let last_modified_str = std::fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|time| {
            let dur = std::time::SystemTime::now().duration_since(time).unwrap_or_default();
            let secs = dur.as_secs();
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

    let thumbnail = load_thumbnail(slot);

    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() >= 14 {
            let mut cursor = 0;
            if let Ok(fmt) = crate::world::read_bytes(&bytes, &mut cursor, 10) {
                if fmt == b"RUSTCRAFT1" || fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3" || fmt == b"RUSTCRAFT4" || fmt == b"RUSTCRAFT5" || fmt == b"RUSTCRAFT6" {
                    if let Ok(seed_bytes) = crate::world::read_bytes(&bytes, &mut cursor, 4) {
                        let seed = u32::from_le_bytes(seed_bytes.try_into().unwrap_or_default());
                        let (gamemode, name) = if fmt == b"RUSTCRAFT4" || fmt == b"RUSTCRAFT5" || fmt == b"RUSTCRAFT6" {
                            let gm_byte = crate::world::read_bytes(&bytes, &mut cursor, 1).map(|b| b[0]).unwrap_or(0);
                            let gm = if gm_byte == 1 { crate::gameplay::player::GameMode::Creative } else { crate::gameplay::player::GameMode::Survival };
                            let name_len = crate::world::read_bytes(&bytes, &mut cursor, 1).map(|b| b[0] as usize).unwrap_or(0);
                            let name_str = if name_len > 0 {
                                if let Ok(nb) = crate::world::read_bytes(&bytes, &mut cursor, name_len) {
                                    String::from_utf8_lossy(nb).to_string()
                                } else {
                                    format!("World {}", seed)
                                }
                            } else {
                                format!("World {}", seed)
                            };
                            (gm, name_str)
                        } else {
                            (crate::gameplay::player::GameMode::Survival, format!("World {}", seed))
                        };

                        let mut player_pos = crate::world::World::find_spawn_position(seed);
                        if (fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3" || fmt == b"RUSTCRAFT4" || fmt == b"RUSTCRAFT5" || fmt == b"RUSTCRAFT6") && bytes.len() >= cursor + 12 {
                            if let (Ok(x), Ok(y), Ok(z)) = (
                                crate::world::read_bytes(&bytes, &mut cursor, 4),
                                crate::world::read_bytes(&bytes, &mut cursor, 4),
                                crate::world::read_bytes(&bytes, &mut cursor, 4),
                            ) {
                                player_pos = crate::core::math::Vec3::new(
                                    f32::from_le_bytes(x.try_into().unwrap_or_default()),
                                    f32::from_le_bytes(y.try_into().unwrap_or_default()),
                                    f32::from_le_bytes(z.try_into().unwrap_or_default()),
                                );
                            }
                        }
                        let mut chunk_count = 0;
                        if let Ok(count_bytes) = crate::world::read_bytes(&bytes, &mut cursor, 4) {
                            chunk_count = u32::from_le_bytes(count_bytes.try_into().unwrap_or_default()) as usize;
                        }
                        return SaveMetadata {
                            slot,
                            exists: true,
                            name,
                            gamemode,
                            seed,
                            player_pos,
                            chunk_count,
                            file_size_bytes: file_size,
                            last_modified_str,
                            thumbnail,
                        };
                    }
                }
            }
        }
    }

    SaveMetadata {
        slot,
        exists: true,
        name: format!("Slot {:02}", slot),
        gamemode: crate::gameplay::player::GameMode::Survival,
        seed: 0,
        player_pos: crate::core::math::Vec3::new(8.0, 90.0, 8.0),
        chunk_count: 0,
        file_size_bytes: file_size,
        last_modified_str,
        thumbnail,
    }
}

/// Deletes a save file and its associated thumbnail preview from disk.
pub fn delete_save(slot: u8) -> Result<(), String> {
    let _ = std::fs::remove_file(thumbnail_path(slot));
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())?;
        crate::log_info!("SAVE", "Deleted save file in slot {} ({})", slot, path);
        Ok(())
    } else {
        crate::log_info!("SAVE", "No existing save file in slot {} ({}) to delete", slot, path);
        Ok(())
    }
}

/// Saves the active world state (player position, game mode, modified chunks, and thumbnail) to `slot`.
pub fn save_current_world(
    world: &mut World,
    camera: &Camera,
    player: &crate::gameplay::player::Player,
    slot: u8,
    thumbnail: Option<&[u32]>,
) -> Result<(), String> {
    crate::log_info!("SAVE", "Saving current world state to slot {} ('{}', mode: {:?}, pos: {:.1}, {:.1}, {:.1})", slot, world.name, player.gamemode, camera.position.x, camera.position.y, camera.position.z);
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
