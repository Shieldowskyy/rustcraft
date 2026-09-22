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

pub fn settings_file_path() -> std::path::PathBuf {
    saves_dir().join("settings.bin")
}

#[derive(Clone, Debug)]
pub struct SaveMetadata {
    pub slot: u8,
    pub exists: bool,
    pub seed: u32,
    pub player_pos: crate::core::math::Vec3,
    pub chunk_count: usize,
    pub file_size_bytes: u64,
    pub last_modified_str: String,
}

pub fn get_save_metadata(slot: u8) -> SaveMetadata {
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return SaveMetadata {
            slot,
            exists: false,
            seed: 0,
            player_pos: crate::core::math::Vec3::new(8.0, 90.0, 8.0),
            chunk_count: 0,
            file_size_bytes: 0,
            last_modified_str: String::new(),
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

    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() >= 14 {
            let mut cursor = 0;
            if let Ok(fmt) = crate::world::read_bytes(&bytes, &mut cursor, 10) {
                if fmt == b"RUSTCRAFT1" || fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3" {
                    if let Ok(seed_bytes) = crate::world::read_bytes(&bytes, &mut cursor, 4) {
                        let seed = u32::from_le_bytes(seed_bytes.try_into().unwrap_or_default());
                        let mut player_pos = crate::core::math::Vec3::new(8.0, 90.0, 8.0);
                        if (fmt == b"RUSTCRAFT2" || fmt == b"RUSTCRAFT3") && bytes.len() >= 26 {
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
                            seed,
                            player_pos,
                            chunk_count,
                            file_size_bytes: file_size,
                            last_modified_str,
                        };
                    }
                }
            }
        }
    }

    SaveMetadata {
        slot,
        exists: true,
        seed: 0,
        player_pos: crate::core::math::Vec3::new(8.0, 90.0, 8.0),
        chunk_count: 0,
        file_size_bytes: file_size,
        last_modified_str,
    }
}

pub fn delete_save(slot: u8) -> Result<(), String> {
    let path = save_path(slot);
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

/// Zapisuje aktualny stan świata (pozycję gracza z kamery) do slotu.
pub fn save_current_world(world: &mut World, camera: &Camera, slot: u8) -> Result<(), String> {
    world.player_position = camera.position;
    let path = save_path(slot);
    world.save(&path)
}
