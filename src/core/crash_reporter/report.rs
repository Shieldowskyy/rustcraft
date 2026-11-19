//! Crash report formatting, UTC timestamp conversion, and disk persistence on panic.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::platform::system::sentry_bridge;

use super::{is_sentry_active, CONTEXT_MAP, CRASH_DIR, RECENT_LOGS, START_TIME};

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

pub(super) fn handle_panic(panic_info: &std::panic::PanicHookInfo<'_>) {
    let payload = panic_info.payload();
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<dyn Any> (unknown panic payload)".to_string()
    };

    let location = panic_info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let backtrace_str = format!("{}", std::backtrace::Backtrace::force_capture());

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

    let quote = QUOTES[(uptime_secs as usize) % QUOTES.len()];
    let now_system = SystemTime::now();
    let timestamp_str = format_system_time(now_system);

    let sentry_event_id = if is_sentry_active() {
        sentry_bridge::capture_panic(thread_name, &message, &location, &contexts)
    } else {
        None
    };

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
    report.push_str(&format!(
        "CPU Logical Cores: {}\n",
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    ));
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

    let crash_dir = CRASH_DIR
        .lock()
        .ok()
        .and_then(|d| d.clone())
        .unwrap_or_else(|| PathBuf::from("crash-reports"));

    let file_path = write_crash_file(&crash_dir, &now_system, &report);

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

fn format_system_time(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (year, month, day, hour, min, sec) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, hour, min, sec)
}

fn format_file_timestamp(st: SystemTime) -> String {
    let dur = st.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (year, month, day, hour, min, sec) = epoch_seconds_to_datetime(dur.as_secs());
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", year, month, day, hour, min, sec)
}

fn epoch_seconds_to_datetime(seconds: u64) -> (u32, u32, u32, u32, u32, u32) {
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
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
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

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::super::{record_log_entry, set_context, CrashReporterConfig};
    use super::*;

    #[test]
    fn test_datetime_conversion() {
        let (y, m, d, h, min, s) = epoch_seconds_to_datetime(1767225600);
        assert_eq!((y, m, d, h, min, s), (2026, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_crash_config_defaults() {
        let config = CrashReporterConfig::default();
        let expected_env = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        };
        assert_eq!(config.environment, expected_env);
        assert!(config.sentry_enabled);
        assert_eq!(config.crash_dir, PathBuf::from("crash-reports"));
    }

    #[test]
    fn test_context_and_logs() {
        set_context("test_key", "test_val");
        record_log_entry("INFO", "TEST", "Testing crash reporter logging");
    }
}

