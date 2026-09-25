// Zapisywanie i wczytywanie światów: ustalenie katalogu na sejwy oraz
// budowanie ścieżek do plików poszczególnych slotów.

use crate::graphics::camera::Camera;
use crate::world::World;

/// Zwraca (i zapamiętuje) katalog, w którym mają być przechowywane sejwy.
fn saves_dir() -> &'static std::path::PathBuf {
    use std::sync::OnceLock;
    static SAVES_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

    SAVES_DIR.get_or_init(|| {
        if let Some(dir) = exe_saves_dir_if_writable() {
            return dir;
        }
        fallback_saves_dir()
    })
}

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

fn fallback_saves_dir() -> std::path::PathBuf {
    #[cfg(target_os = "vita")]
    {
        let dir = std::path::PathBuf::from("ux0:data/rustcraft/saves");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }
    #[cfg(not(target_os = "vita"))]
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

/// Ścieżka do pliku sejwu dla danego slotu (1-based).
pub fn save_path(slot: u8) -> String {
    saves_dir()
        .join(format!("rustcraft_world_{slot:02}.bin"))
        .to_string_lossy()
        .into_owned()
}

pub fn thumbnail_path(slot: u8) -> String {
    saves_dir()
        .join(format!("rustcraft_world_{slot:02}.thumb"))
        .to_string_lossy()
        .into_owned()
}

pub const THUMB_SIZE: usize = 56;

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

pub fn save_thumbnail(slot: u8, thumb: &[u32]) {
    let path = thumbnail_path(slot);
    let bytes: &[u8] = bytemuck::cast_slice(thumb);
    let _ = std::fs::write(path, bytes);
}

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

pub fn settings_file_path() -> std::path::PathBuf {
    saves_dir().join("settings.bin")
}

#[derive(Clone, Debug)]
pub struct SaveMetadata {
    pub slot: u8,
    pub exists: bool,
    pub name: String,
    pub gamemode: crate::gameplay::player::GameMode,
    pub seed: u32,
    pub player_pos: crate::core::math::Vec3,
    pub chunk_count: usize,
    pub file_size_bytes: u64,
    pub last_modified_str: String,
    pub thumbnail: Option<Vec<u32>>,
}

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
                "przed chwila".to_string()
            } else if secs < 3600 {
                format!("{} min temu", secs / 60)
            } else if secs < 86400 {
                format!("{} godz temu", secs / 3600)
            } else {
                format!("{} dni temu", secs / 86400)
            }
        })
        .unwrap_or_else(|| "zapisany".to_string());

    let thumbnail = load_thumbnail(slot);

    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() >= 14 {
            let mut cursor = 0;
            if let Ok(fmt) = crate::world::read_bytes(&bytes, &mut cursor, 10) {
                if fmt == b"RUSTCRAFT1" || fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3" || fmt == b"RUSTCRAFT4" {
                    if let Ok(seed_bytes) = crate::world::read_bytes(&bytes, &mut cursor, 4) {
                        let seed = u32::from_le_bytes(seed_bytes.try_into().unwrap_or_default());
                        let (gamemode, name) = if fmt == b"RUSTCRAFT4" {
                            let gm_byte = crate::world::read_bytes(&bytes, &mut cursor, 1).map(|b| b[0]).unwrap_or(0);
                            let gm = if gm_byte == 1 { crate::gameplay::player::GameMode::Creative } else { crate::gameplay::player::GameMode::Survival };
                            let name_len = crate::world::read_bytes(&bytes, &mut cursor, 1).map(|b| b[0] as usize).unwrap_or(0);
                            let name_str = if name_len > 0 {
                                if let Ok(nb) = crate::world::read_bytes(&bytes, &mut cursor, name_len) {
                                    String::from_utf8_lossy(nb).to_string()
                                } else {
                                    format!("Swiat {}", seed)
                                }
                            } else {
                                format!("Swiat {}", seed)
                            };
                            (gm, name_str)
                        } else {
                            (crate::gameplay::player::GameMode::Survival, format!("Swiat {}", seed))
                        };

                        let mut player_pos = crate::world::World::find_spawn_position(seed);
                        if (fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3" || fmt == b"RUSTCRAFT4") && bytes.len() >= cursor + 12 {
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

pub fn delete_save(slot: u8) -> Result<(), String> {
    let _ = std::fs::remove_file(thumbnail_path(slot));
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

/// Zapisuje aktualny stan świata (pozycję gracza z kamery, tryb gry, screenshot) do slotu.
pub fn save_current_world(
    world: &mut World,
    camera: &Camera,
    player: &crate::gameplay::player::Player,
    slot: u8,
    thumbnail: Option<&[u32]>,
) -> Result<(), String> {
    world.player_position = camera.position;
    world.gamemode = player.gamemode;
    if let Some(thumb) = thumbnail {
        save_thumbnail(slot, thumb);
    }
    let path = save_path(slot);
    world.save(&path)
}
