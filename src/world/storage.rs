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

/// Zapisuje aktualny stan świata (pozycję gracza z kamery) do slotu.
pub fn save_current_world(world: &mut World, camera: &Camera, slot: u8) -> Result<(), String> {
    world.player_position = camera.position;
    let path = save_path(slot);
    world.save(&path)
}
