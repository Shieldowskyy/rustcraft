//! Logowanie serwera w stylu Minecrafta (`logs/latest.log`, rotacja po dacie i rozmiarze, kompresja Gzip `.log.gz`).

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::GzEncoder;
use flate2::Compression;

pub const DEFAULT_MAX_LOG_SIZE_BYTES: u64 = 20 * 1024 * 1024;

struct LoggerState {
    logs_dir: PathBuf,
    current_date: String,
    writer: BufWriter<File>,
    current_bytes: u64,
    max_bytes: u64,
}

static SERVER_LOGGER: Mutex<Option<LoggerState>> = Mutex::new(None);
static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn is_initialized() -> bool {
    LOGGER_INITIALIZED.load(Ordering::Relaxed)
}

pub fn init_server_logger(logs_dir: impl AsRef<Path>) -> io::Result<()> {
    init_server_logger_with_max_size(logs_dir, DEFAULT_MAX_LOG_SIZE_BYTES)
}

pub fn init_server_logger_with_max_size(
    logs_dir: impl AsRef<Path>,
    max_bytes: u64,
) -> io::Result<()> {
    let logs_dir = logs_dir.as_ref().to_path_buf();
    fs::create_dir_all(&logs_dir)?;

    let latest_path = logs_dir.join("latest.log");
    if latest_path.exists() {
        if let Ok(metadata) = fs::metadata(&latest_path) {
            if metadata.len() > 0 {
                let mtime = metadata.modified().unwrap_or_else(|_| SystemTime::now());
                if let Err(err) = compress_and_rotate_file(&logs_dir, &latest_path, mtime) {
                    eprintln!(
                        "[SERVER] [Server thread/WARN]: Failed to compress previous latest.log: {}",
                        err
                    );
                }
            }
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&latest_path)?;

    let current_date = format_system_date(SystemTime::now());
    let state = LoggerState {
        logs_dir,
        current_date,
        writer: BufWriter::new(file),
        current_bytes: 0,
        max_bytes,
    };

    let mut lock = SERVER_LOGGER.lock().unwrap();
    *lock = Some(state);
    LOGGER_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn compress_and_rotate_file(
    logs_dir: &Path,
    source_path: &Path,
    file_time: SystemTime,
) -> io::Result<PathBuf> {
    let date_str = format_system_date(file_time);
    let mut index = 1;
    let archive_path = loop {
        let candidate = logs_dir.join(format!("{}-{}.log.gz", date_str, index));
        if !candidate.exists() {
            break candidate;
        }
        index += 1;
    };

    let mut data = Vec::new();
    {
        let mut f = File::open(source_path)?;
        f.read_to_end(&mut data)?;
    }

    let mut encoder = GzEncoder::new(Vec::with_capacity(data.len() / 3), Compression::default());
    encoder.write_all(&data)?;
    let compressed_data = encoder.finish()?;
    fs::write(&archive_path, compressed_data)?;

    let _ = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(source_path)?;

    Ok(archive_path)
}

pub fn server_log(level: &str, thread_name: &str, message: &str) {
    let now = SystemTime::now();
    let time_str = format_system_time_only(now);
    let today_str = format_system_date(now);
    let formatted_line = format!("[{}] [{}/{}]: {}\n", time_str, thread_name, level, message);

    if level == "ERROR" || level == "FATAL" {
        eprint!("{}", formatted_line);
    } else {
        print!("{}", formatted_line);
    }

    crate::core::crash_reporter::record_log_entry(level, "SERVER", message);

    if let Ok(mut lock) = SERVER_LOGGER.lock() {
        if let Some(ref mut state) = *lock {
            let need_size_rotation = state.current_bytes >= state.max_bytes && state.max_bytes > 0;
            let need_date_rotation = state.current_date != today_str;

            if need_size_rotation || need_date_rotation {
                let _ = state.writer.flush();
                let latest_path = state.logs_dir.join("latest.log");
                let rot_date = SystemTime::now();
                if let Ok(archive) =
                    compress_and_rotate_file(&state.logs_dir, &latest_path, rot_date)
                {
                    if let Ok(new_file) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&latest_path)
                    {
                        state.writer = BufWriter::new(new_file);
                        state.current_bytes = 0;
                        state.current_date = today_str;
                        let notice = format!(
                            "[{}] [Server thread/INFO]: [SERVER] Rotated active log to '{}'\n",
                            time_str,
                            archive
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("archive.log.gz")
                        );
                        let _ = state.writer.write_all(notice.as_bytes());
                        let _ = state.writer.flush();
                    }
                }
            }

            let bytes = formatted_line.as_bytes();
            if let Ok(()) = state.writer.write_all(bytes) {
                let _ = state.writer.flush();
                state.current_bytes += bytes.len() as u64;
            }
        }
    }
}

pub fn log_from_core(level: &str, target: &str, message: &str) {
    if !is_initialized() {
        return;
    }
    let now = SystemTime::now();
    let time_str = format_system_time_only(now);
    let today_str = format_system_date(now);
    let formatted_line = format!(
        "[{}] [{}/{}]: [{}] {}\n",
        time_str, "Server thread", level, target, message
    );

    if let Ok(mut lock) = SERVER_LOGGER.lock() {
        if let Some(ref mut state) = *lock {
            if (state.current_bytes >= state.max_bytes && state.max_bytes > 0)
                || state.current_date != today_str
            {
                let _ = state.writer.flush();
                let latest_path = state.logs_dir.join("latest.log");
                if compress_and_rotate_file(&state.logs_dir, &latest_path, SystemTime::now())
                    .is_ok()
                {
                    if let Ok(new_file) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&latest_path)
                    {
                        state.writer = BufWriter::new(new_file);
                        state.current_bytes = 0;
                        state.current_date = today_str;
                    }
                }
            }
            let bytes = formatted_line.as_bytes();
            if let Ok(()) = state.writer.write_all(bytes) {
                let _ = state.writer.flush();
                state.current_bytes += bytes.len() as u64;
            }
        }
    }
}

pub fn shutdown_server_logger() {
    if let Ok(mut lock) = SERVER_LOGGER.lock() {
        if let Some(ref mut state) = *lock {
            let _ = state.writer.flush();
        }
        *lock = None;
    }
    LOGGER_INITIALIZED.store(false, Ordering::SeqCst);
}

pub fn format_system_date(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (year, month, day, _, _, _) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:04}-{:02}-{:02}", year, month, day)
}

pub fn format_system_time_only(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (_, _, _, hour, min, sec) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:02}:{:02}:{:02}", hour, min, sec)
}

pub fn epoch_seconds_to_datetime(seconds: u64) -> (u32, u32, u32, u32, u32, u32) {
    let sec = (seconds % 60) as u32;
    let total_minutes = seconds / 60;
    let min = (total_minutes % 60) as u32;
    let total_hours = total_minutes / 60;
    let hour = (total_hours % 24) as u32;
    let mut d = (total_hours / 24) as u32;

    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1u32;
    for &m_days in &month_days {
        if d < m_days {
            break;
        }
        d -= m_days;
        month += 1;
    }

    (year, month, d + 1, hour, min, sec)
}

#[inline]
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[macro_export]
macro_rules! server_info {
    ($($arg:tt)+) => {
        $crate::features::net::server_log::server_log("INFO", "Server thread", &format!($($arg)+));
    };
}

#[macro_export]
macro_rules! server_warn {
    ($($arg:tt)+) => {
        $crate::features::net::server_log::server_log("WARN", "Server thread", &format!($($arg)+));
    };
}

#[macro_export]
macro_rules! server_error {
    ($($arg:tt)+) => {
        $crate::features::net::server_log::server_log("ERROR", "Server thread", &format!($($arg)+));
    };
}

#[macro_export]
macro_rules! server_debug {
    ($($arg:tt)+) => {
        $crate::features::net::server_log::server_log("DEBUG", "Server thread", &format!($($arg)+));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;

    #[test]
    fn test_datetime_formatting() {
        let (y, m, d, h, min, s) = epoch_seconds_to_datetime(1700000000);
        assert_eq!((y, m, d, h, min, s), (2023, 11, 14, 22, 13, 20));
        assert_eq!(
            format_system_date(UNIX_EPOCH + std::time::Duration::from_secs(1700000000)),
            "2023-11-14"
        );
        assert_eq!(
            format_system_time_only(UNIX_EPOCH + std::time::Duration::from_secs(1700000000)),
            "22:13:20"
        );
    }

    #[test]
    fn test_log_compression_and_rotation() {
        let test_dir = std::env::temp_dir().join("rustcraft_test_logs_rotation");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let latest = test_dir.join("latest.log");
        let original_content = "This is a Minecraft server test log entry\nSecond line with info\n";
        fs::write(&latest, original_content).unwrap();

        let test_time = UNIX_EPOCH + std::time::Duration::from_secs(1700000000);
        let archive1 = compress_and_rotate_file(&test_dir, &latest, test_time).unwrap();
        assert!(archive1.exists());
        assert_eq!(archive1.file_name().unwrap(), "2023-11-14-1.log.gz");

        let gz_bytes = fs::read(&archive1).unwrap();
        let mut decoder = GzDecoder::new(&gz_bytes[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert_eq!(decompressed, original_content);
        assert_eq!(fs::metadata(&latest).unwrap().len(), 0);

        fs::write(&latest, "Second session log\n").unwrap();
        let archive2 = compress_and_rotate_file(&test_dir, &latest, test_time).unwrap();
        assert_eq!(archive2.file_name().unwrap(), "2023-11-14-2.log.gz");
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_server_logger_workflow() {
        let test_dir = std::env::temp_dir().join("rustcraft_test_logger_workflow");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();
        let latest = test_dir.join("latest.log");
        fs::write(&latest, "Previous server session log\n").unwrap();

        init_server_logger_with_max_size(&test_dir, 200).unwrap();
        assert!(is_initialized());

        let today = format_system_date(SystemTime::now());
        assert!(test_dir.join(format!("{}-1.log.gz", today)).exists());

        server_log("INFO", "Server thread", "Hello from RustCraft dedicated server!");
        server_log("WARN", "Server thread", "Warning test entry");

        let current_log = fs::read_to_string(&latest).unwrap();
        assert!(current_log.contains("[Server thread/INFO]: Hello from RustCraft dedicated server!"));
        assert!(current_log.contains("[Server thread/WARN]: Warning test entry"));

        shutdown_server_logger();
        assert!(!is_initialized());
        let _ = fs::remove_dir_all(&test_dir);
    }
}
