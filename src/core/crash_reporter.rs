//! Comprehensive Crash Reporter & Sentry telemetry integration for RustCraft.
//!
//! Provides:
//! - Global panic hook capturing backtraces, system info, active renderer, world seed, and logs.
//! - Minecraft-style local crash report files saved to `crash-reports/crash-<timestamp>.txt`.
//! - Sentry error tracking with tags, breadcrumbs, release info, and automated event capture.
//! - CLI and environment variable configuration (`--sentry-dsn`, `SENTRY_DSN`, `--crash-test`).

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
use sentry::ClientInitGuard;

/// Configuration options for the crash reporter.
#[derive(Debug, Clone)]
pub struct CrashReporterConfig {
    /// Sentry DSN endpoint (e.g. "https://public@sentry.example.com/1").
    pub dsn: Option<String>,
    /// Whether Sentry transmission is enabled.
    pub sentry_enabled: bool,
    /// Environment tag (e.g. "production", "development").
    pub environment: String,
    /// Release version tag (e.g. "rustcraft@0.1.0").
    pub release: String,
    /// Directory where local crash dump files will be written.
    pub crash_dir: PathBuf,
}

impl Default for CrashReporterConfig {
    fn default() -> Self {
        let env_dsn = std::env::var("SENTRY_DSN").ok().filter(|s| !s.trim().is_empty());
        let environment = if cfg!(debug_assertions) {
            "development".to_string()
        } else {
            "production".to_string()
        };
        let release = format!("rustcraft@{}", env!("CARGO_PKG_VERSION"));
        let crash_dir = std::env::var("RUSTCRAFT_CRASH_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("crash-reports"));

        Self {
            dsn: env_dsn,
            sentry_enabled: true,
            environment,
            release,
            crash_dir,
        }
    }
}

/// Guard holding the Sentry client session. Dropping this will flush remaining events.
pub struct CrashGuard {
    #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
    _sentry_guard: Option<ClientInitGuard>,
}

impl CrashGuard {
    /// Manually flush queued Sentry telemetry events.
    pub fn flush(&self) {
        #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
        if let Some(client) = sentry::Hub::current().client() {
            client.flush(Some(std::time::Duration::from_secs(2)));
        }
    }
}

// Global runtime state
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static SENTRY_ACTIVE: AtomicBool = AtomicBool::new(false);
static START_TIME: Mutex<Option<Instant>> = Mutex::new(None);
static CONTEXT_MAP: Mutex<Option<Vec<(String, String)>>> = Mutex::new(None);
static RECENT_LOGS: Mutex<Option<VecDeque<String>>> = Mutex::new(None);
static CRASH_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

const MAX_RECENT_LOGS: usize = 40;

const QUOTES: &[&str] = &[
    "Don't panic! Everything is (not) fine.",
    "Who set us up the TNT?",
    "Everything's going to plan. No, really, that was supposed to happen.",
    "Hi. I'm RustCraft, and I'm a crashaholic.",
    "Daisy, Daisy, give me your answer do...",
    "You should try our sister game, Minceraft!",
    "Quite honestly, I wouldn't worry myself about that.",
    "Surprise! Haha. Well, this is awkward.",
    "Ooh. Shiny.",
    "I bet Tyler did this.",
    "Shall we play a game?",
    "Hey, that was not supposed to happen!",
    "It's not a bug, it's a feature!",
];

/// Initializes the crash reporter and Sentry integration.
pub fn init(#[allow(unused_mut)] mut config: CrashReporterConfig) -> CrashGuard {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return CrashGuard {
            #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
            _sentry_guard: None,
        };
    }

    // Set start time and collections
    if let Ok(mut st) = START_TIME.lock() {
        *st = Some(Instant::now());
    }
    if let Ok(mut ctx) = CONTEXT_MAP.lock() {
        *ctx = Some(Vec::new());
    }
    if let Ok(mut logs) = RECENT_LOGS.lock() {
        *logs = Some(VecDeque::with_capacity(MAX_RECENT_LOGS));
    }
    if let Ok(mut cd) = CRASH_DIR.lock() {
        *cd = Some(config.crash_dir.clone());
    }

    // Add baseline context tags
    set_context("os", std::env::consts::OS);
    set_context("arch", std::env::consts::ARCH);
    set_context("version", env!("CARGO_PKG_VERSION"));
    set_context("environment", &config.environment);

    #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
    let sentry_guard = {
        let dsn = config.dsn.take();
        if config.sentry_enabled && dsn.is_some() {
            let dsn_str = dsn.unwrap();
            let mut options = sentry::ClientOptions::default();
            options.release = Some(config.release.clone().into());
            options.environment = Some(config.environment.clone().into());
            options.attach_stacktrace = true;
            options.send_default_pii = false;

            let guard = sentry::init((dsn_str, options));
            if guard.is_enabled() {
                SENTRY_ACTIVE.store(true, Ordering::SeqCst);
                crate::log_info!("CRASH", "Sentry telemetry initialized ({})", config.release);
            }
            Some(guard)
        } else {
            crate::log_info!("CRASH", "Sentry telemetry is inactive (no DSN provided or disabled)");
            None
        }
    };

    #[cfg(any(target_arch = "wasm32", target_arch = "powerpc", target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon"))]
    let _sentry_guard = ();

    // Install the panic hook
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        handle_panic(panic_info);
        default_hook(panic_info);
    }));

    CrashGuard {
        #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
        _sentry_guard: sentry_guard,
    }
}

/// Checks whether Sentry reporting is actively running.
pub fn is_sentry_active() -> bool {
    SENTRY_ACTIVE.load(Ordering::Relaxed)
}

/// Associates key-value metadata with current game execution for crash reports and Sentry.
pub fn set_context(key: &str, value: &str) {
    if let Ok(mut lock) = CONTEXT_MAP.lock() {
        if let Some(ctx) = lock.as_mut() {
            if let Some(entry) = ctx.iter_mut().find(|(k, _)| k == key) {
                entry.1 = value.to_string();
            } else {
                ctx.push((key.to_string(), value.to_string()));
            }
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
    if is_sentry_active() {
        sentry::configure_scope(|scope| {
            scope.set_tag(key, value);
        });
    }
}

/// Records a log message into the crash reporter's ring buffer and forwards as a Sentry breadcrumb.
pub fn record_log_entry(level: &str, target: &str, message: &str) {
    let formatted = format!("[{:<5}] [{}] {}", level, target, message);
    if let Ok(mut lock) = RECENT_LOGS.lock() {
        if let Some(logs) = lock.as_mut() {
            if logs.len() >= MAX_RECENT_LOGS {
                logs.pop_front();
            }
            logs.push_back(formatted);
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
    if is_sentry_active() {
        let sentry_level = match level {
            "ERROR" => sentry::Level::Error,
            "WARN" => sentry::Level::Warning,
            "DEBUG" => sentry::Level::Debug,
            _ => sentry::Level::Info,
        };
        sentry::add_breadcrumb(sentry::Breadcrumb {
            category: Some(target.to_string()),
            message: Some(message.to_string()),
            level: sentry_level,
            ..Default::default()
        });
    }
}

/// Internal panic handler that captures diagnostics, writes local file, and sends Sentry event.
fn handle_panic(panic_info: &std::panic::PanicHookInfo<'_>) {
    let payload = panic_info.payload();
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<dyn Any> (unknown panic payload)".to_string()
    };

    let location = panic_info.location().map(|loc| {
        format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
    }).unwrap_or_else(|| "unknown location".to_string());

    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");

    let backtrace = std::backtrace::Backtrace::force_capture();
    let backtrace_str = format!("{}", backtrace);

    let uptime_secs = START_TIME
        .lock()
        .ok()
        .and_then(|st| *st)
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    let contexts = CONTEXT_MAP
        .lock()
        .ok()
        .and_then(|c| c.clone())
        .unwrap_or_default();

    let recent_logs = RECENT_LOGS
        .lock()
        .ok()
        .and_then(|l| l.clone())
        .map(|deque| deque.into_iter().collect::<Vec<_>>())
        .unwrap_or_default();

    // Select random joke quote
    let quote_idx = (uptime_secs as usize) % QUOTES.len();
    let quote = QUOTES[quote_idx];

    let now_system = SystemTime::now();
    let timestamp_str = format_system_time(now_system);

    // Send to Sentry if active
    #[allow(unused_mut)]
    let mut sentry_event_id: Option<String> = None;

    #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon")))]
    if is_sentry_active() {
        let mut event = sentry::protocol::Event {
            message: Some(format!("Panic in thread '{}': {}", thread_name, message)),
            level: sentry::Level::Fatal,
            ..Default::default()
        };

        for (k, v) in &contexts {
            event.tags.insert(k.clone(), v.clone());
        }
        event.tags.insert("panic_location".to_string(), location.clone());
        event.tags.insert("thread".to_string(), thread_name.to_string());

        let id = sentry::capture_event(event);
        sentry_event_id = Some(id.to_string());

        if let Some(client) = sentry::Hub::current().client() {
            client.flush(Some(std::time::Duration::from_secs(2)));
        }
    }

    // Build the crash report document
    let mut report = String::new();
    report.push_str("---- RustCraft Crash Report ----\n");
    report.push_str(&format!("// {}\n\n", quote));
    report.push_str(&format!("Time: {}\n", timestamp_str));
    report.push_str(&format!("Description: Fatal panic in thread '{}'\n\n", thread_name));
    report.push_str(&format!("Panic Reason: {}\n", message));
    report.push_str(&format!("Location:     {}\n\n", location));

    report.push_str("--------------------------------------------------------------------------------\n");
    report.push_str("Stacktrace:\n");
    report.push_str(&backtrace_str);
    report.push_str("\n\n");

    report.push_str("--------------------------------------------------------------------------------\n");
    report.push_str("-- System & Game Details --\n");
    report.push_str(&format!("RustCraft Version: {}\n", env!("CARGO_PKG_VERSION")));
    report.push_str(&format!("Operating System:  {} ({})\n", std::env::consts::OS, std::env::consts::ARCH));
    report.push_str(&format!("CPU Logical Cores: {}\n", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)));
    report.push_str(&format!("Process Uptime:    {:.2}s\n", uptime_secs));
    if let Ok(cwd) = std::env::current_dir() {
        report.push_str(&format!("Working Directory: {}\n", cwd.display()));
    }
    if let Ok(exe) = std::env::current_exe() {
        report.push_str(&format!("Executable Path:   {}\n", exe.display()));
    }

    if !contexts.is_empty() {
        report.push_str("\nActive Context Tags:\n");
        for (k, v) in &contexts {
            report.push_str(&format!("  {} = {}\n", k, v));
        }
    }

    report.push_str(&format!(
        "Sentry Submission: {}\n",
        sentry_event_id
            .as_deref()
            .map(|id| format!("Submitted (Event ID: {})", id))
            .unwrap_or_else(|| "Not sent (Sentry not active or offline)".to_string())
    ));

    if !recent_logs.is_empty() {
        report.push_str("\n--------------------------------------------------------------------------------\n");
        report.push_str("Recent Breadcrumbs / Logs:\n");
        for log_line in &recent_logs {
            report.push_str(&format!("  {}\n", log_line));
        }
    }

    report.push_str("--------------------------------------------------------------------------------\n");
    report.push_str("End of RustCraft Crash Report\n");

    // Save to disk
    let crash_dir = CRASH_DIR
        .lock()
        .ok()
        .and_then(|d| d.clone())
        .unwrap_or_else(|| PathBuf::from("crash-reports"));

    let file_path = write_crash_file(&crash_dir, &now_system, &report);

    // Print bold alert to stderr
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("                       [RUSTCRAFT CRASH DETECTED]");
    eprintln!("RustCraft encountered an unrecoverable panic and has terminated.");
    eprintln!("Panic Reason: {}", message);
    eprintln!("Location:     {}", location);
    if let Some(ref path) = file_path {
        eprintln!("\nA detailed crash report has been saved to:\n  {}", path.display());
    }
    if let Some(ref id) = sentry_event_id {
        eprintln!("Crash report uploaded to Sentry! Event ID: {}", id);
    } else {
        eprintln!("Sentry telemetry: Inactive or offline.");
    }
    eprintln!("{}\n", "=".repeat(80));
}

/// Writes report to a uniquely named file inside target directory.
fn write_crash_file(dir: &Path, time: &SystemTime, content: &str) -> Option<PathBuf> {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("Failed to create crash report directory ({}): {}", dir.display(), e);
        return None;
    }

    let file_name = format!("crash-{}.txt", format_file_timestamp(*time));
    let path = dir.join(file_name);
    match fs::write(&path, content) {
        Ok(_) => Some(path),
        Err(e) => {
            eprintln!("Failed to write crash report to {}: {}", path.display(), e);
            None
        }
    }
}

/// Formats UTC SystemTime to YYYY-MM-DD HH:MM:SS UTC.
fn format_system_time(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (year, month, day, hour, min, sec) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, hour, min, sec)
}

/// Formats UTC SystemTime for filenames: YYYY-MM-DD_HH-MM-SS.
fn format_file_timestamp(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (year, month, day, hour, min, sec) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", year, month, day, hour, min, sec)
}

/// Converts epoch seconds to (year, month, day, hour, minute, second) UTC without external crates.
fn epoch_seconds_to_datetime(seconds: u64) -> (u32, u32, u32, u32, u32, u32) {
    let sec = (seconds % 60) as u32;
    let total_minutes = seconds / 60;
    let min = (total_minutes % 60) as u32;
    let total_hours = total_minutes / 60;
    let hour = (total_hours % 24) as u32;
    let days = (total_hours / 24) as u32;

    // Days since Jan 1, 1970
    let mut d = days;
    let mut year = 1970u32;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31
    ];

    let mut month = 1u32;
    for &m_days in &month_days {
        if d < m_days {
            break;
        }
        d -= m_days;
        month += 1;
    }
    let day = d + 1;

    (year, month, day, hour, min, sec)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_conversion() {
        // 2026-01-01 00:00:00 UTC = 1767225600
        let (y, m, d, h, min, s) = epoch_seconds_to_datetime(1767225600);
        assert_eq!((y, m, d, h, min, s), (2026, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_crash_config_defaults() {
        let config = CrashReporterConfig::default();
        assert_eq!(config.environment, "development");
        assert!(config.sentry_enabled);
        assert_eq!(config.crash_dir, PathBuf::from("crash-reports"));
    }

    #[test]
    fn test_context_and_logs() {
        set_context("test_key", "test_val");
        record_log_entry("INFO", "TEST", "Testing crash reporter logging");
    }
}
