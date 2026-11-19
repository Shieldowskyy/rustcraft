//! Platform filesystem path resolution for saves, soundtracks, mods, and crash reports.
//! Holds optional mobile sandbox save override (`CUSTOM_SAVES_DIR`) and checks writable paths.
//! Pure path discovery; actual world serialization and parsing happen in `features/`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CUSTOM_SAVES_DIR: OnceLock<PathBuf> = OnceLock::new();
static RESOLVED_SAVES_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Overrides the world saves directory at runtime (used by Android JNI sandbox initialization).
pub fn set_custom_save_dir(dir: PathBuf) {
    let _ = std::fs::create_dir_all(&dir);
    let _ = CUSTOM_SAVES_DIR.set(dir);
}

/// Returns the active root directory for world save files.
pub fn saves_dir() -> PathBuf {
    if let Some(dir) = CUSTOM_SAVES_DIR.get() {
        return dir.clone();
    }
    RESOLVED_SAVES_DIR
        .get_or_init(|| exe_saves_dir_if_writable().unwrap_or_else(fallback_saves_dir))
        .clone()
}

/// Returns the root application data directory (parent of `saves/`).
pub fn app_files_dir() -> PathBuf {
    saves_dir()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(saves_dir)
}

fn exe_saves_dir_if_writable() -> Option<PathBuf> {
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

/// Checks whether write permissions are granted in `dir`.
pub fn is_dir_writable(dir: &Path) -> bool {
    let test_path = dir.join(".rustcraft_write_test");
    match std::fs::File::create(&test_path) {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_path);
            true
        }
        Err(_) => false,
    }
}

/// Platform-specific fallback directory for save files when executable-adjacent storage is read-only.
pub fn fallback_saves_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = PathBuf::from("/data/data/com.rustcraft.game/files/saves");
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    #[cfg(target_os = "vita")]
    {
        let dir = PathBuf::from("ux0:data/rustcraft/saves");
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
            let dir = PathBuf::from(path_str);
            if std::fs::create_dir_all(&dir).is_ok() && is_dir_writable(&dir) {
                return dir;
            }
        }
        return PathBuf::from("game:/saves");
    }
    #[cfg(not(any(
        target_os = "vita",
        target_os = "android",
        target_os = "xbox360",
        target_os = "xenon",
        feature = "xbox360"
    )))]
    {
        #[cfg(windows)]
        {
            if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
                let dir = appdata.join("rustcraft").join("saves");
                if std::fs::create_dir_all(&dir).is_ok() {
                    return dir;
                }
            }
        }
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            let dir = home.join(".config").join("rustcraft").join("saves");
            if std::fs::create_dir_all(&dir).is_ok() {
                return dir;
            }
        }
        PathBuf::from(".")
    }
}

/// Resolves the platform-specific `soundtracks/` directory.
pub fn soundtracks_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = app_files_dir().join("soundtracks");
        if dir.exists() {
            return dir;
        }
    }
    #[cfg(target_os = "vita")]
    {
        let custom_dir = PathBuf::from("ux0:data/rustcraft/soundtracks");
        if custom_dir.exists() {
            return custom_dir;
        }
        let app_dir = PathBuf::from("app0:soundtracks");
        if app_dir.exists() {
            return app_dir;
        }
    }

    fn has_audio(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.flatten().any(|e| {
            let p = e.path();
            if p.is_dir() {
                has_audio(&p)
            } else {
                matches!(
                    p.extension()
                        .and_then(|x| x.to_str())
                        .map(|x| x.to_ascii_lowercase())
                        .as_deref(),
                    Some("ogg") | Some("opus") | Some("wav")
                )
            }
        })
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("soundtracks");
            if has_audio(&dir) {
                return dir;
            }
        }
    }
    let cwd_dir = PathBuf::from("soundtracks");
    if has_audio(&cwd_dir) {
        return cwd_dir;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("soundtracks");
            if dir.is_dir() {
                return dir;
            }
        }
    }
    cwd_dir
}

/// Normalizes a user-provided soundtracks directory path for mobile/handheld sandboxes.
pub fn resolve_soundtrack_base_dir(folder: &Path) -> PathBuf {
    #[allow(unused_mut)]
    let mut base_dir = folder.to_path_buf();
    #[cfg(target_os = "android")]
    {
        if !base_dir.is_absolute() {
            let android_dir = app_files_dir().join(&base_dir);
            if android_dir.exists() {
                base_dir = android_dir;
            }
        }
    }
    #[cfg(target_os = "vita")]
    {
        if !base_dir.to_string_lossy().contains(':') {
            let vita_dir = soundtracks_dir();
            if vita_dir.exists() {
                base_dir = vita_dir;
            }
        }
    }
    base_dir
}

/// Returns an optional secondary bundled read-only soundtracks directory (e.g. `app0:soundtracks` on PS Vita).
pub fn secondary_soundtracks_dir(primary: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "vita")]
    {
        let app0_dir = PathBuf::from("app0:soundtracks");
        if app0_dir != primary && app0_dir.exists() {
            return Some(app0_dir);
        }
    }
    let _ = primary;
    None
}

/// Resolves the active `mods/` directory across desktop, Android, and PS Vita.
pub fn mods_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = app_files_dir().join("mods");
        if dir.is_dir() {
            return dir;
        }
    }
    #[cfg(target_os = "vita")]
    {
        let custom_dir = PathBuf::from("ux0:data/rustcraft/mods");
        if custom_dir.is_dir() {
            return custom_dir;
        }
        let app_dir = PathBuf::from("app0:mods");
        if app_dir.is_dir() {
            return app_dir;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("mods");
            if dir.is_dir() {
                return dir;
            }
        }
    }
    PathBuf::from("mods")
}

/// Resolves the platform-specific path to `relay.properties`.
pub fn relay_properties_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        app_files_dir().join("relay.properties")
    }
    #[cfg(not(target_os = "android"))]
    {
        PathBuf::from("relay.properties")
    }
}

