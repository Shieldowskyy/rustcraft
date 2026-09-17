// Zapisywanie i wczytywanie światów: ustalenie katalogu na sejwy oraz
// budowanie ścieżek do plików poszczególnych slotów.

use crate::camera::Camera;
use crate::world::World;

/// Zwraca (i zapamiętuje) katalog, w którym mają być przechowywane sejwy.
///
/// Preferowany katalog to `saves/` obok pliku wykonywalnego. Jeśli nie da
/// się w nim pisać (np. gra zainstalowana w `/usr/bin` albo w katalogu
/// tylko do odczytu), używamy `~/.config/rustcraft/saves/` jako fallbacku.
/// Wynik jest liczony tylko raz i cache'owany - unika to powtarzania
/// kosztownych operacji na systemie plików przy każdym zapisie/wczytaniu.
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

/// Próbuje przygotować (utworzyć jeśli trzeba i sprawdzić zapisywalność)
/// katalog `saves/` obok pliku wykonywalnego. Zwraca `None`, jeśli katalogu
/// nie da się ustalić, utworzyć, albo nie ma w nim uprawnień do zapisu.
fn exe_saves_dir_if_writable() -> Option<std::path::PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    let dir = exe_dir.join("saves");

    if !dir.exists() {
        // Jeśli nie możemy utworzyć katalogu (np. brak uprawnień do
        // zapisu w katalogu z plikiem wykonywalnym), traktujemy to tak
        // samo jak brak zapisywalności - lecimy do fallbacku.
        std::fs::create_dir_all(&dir).ok()?;
    }

    if is_dir_writable(&dir) {
        Some(dir)
    } else {
        None
    }
}

/// Katalog fallbackowy: `~/.config/rustcraft/saves/`. Tworzony w razie
/// potrzeby; jeśli z jakiegoś powodu nie da się go ustalić lub utworzyć,
/// awaryjnie wraca do katalogu roboczego (stare zachowanie).
fn fallback_saves_dir() -> std::path::PathBuf {
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        let dir = home.join(".config").join("rustcraft").join("saves");
        if std::fs::create_dir_all(&dir).is_ok() {
            return dir;
        }
    }
    // Ostatnia linia obrony - katalog roboczy, jak wcześniej.
    std::path::PathBuf::from(".")
}

/// Sprawdza, czy do danego katalogu faktycznie można zapisywać, próbując
/// utworzyć i natychmiast usunąć w nim tymczasowy plik testowy. Samo
/// sprawdzanie metadanych/uprawnień bywa niewiarygodne (ACL, systemy
/// plików sieciowe, itp.), więc jedynym pewnym testem jest próba realnego
/// zapisu.
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
