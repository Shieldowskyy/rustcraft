//! Minimalist, thread-safe console logging facility for RustCraft.
//! Enabled via `-log` or `--log` CLI flag.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);
static mut START_TIME: Option<Instant> = None;

/// Initializes the logging system.
pub fn init(enabled: bool) {
    LOGGING_ENABLED.store(enabled, Ordering::SeqCst);
    unsafe {
        START_TIME = Some(Instant::now());
    }
    if enabled {
        println!("============================================================");
        println!(" [RustCraft Debug Logger Initialized] CLI flag: -log");
        println!(" OS: {} | Arch: {}", std::env::consts::OS, std::env::consts::ARCH);
        println!("============================================================");
    }
}

/// Checks whether console debug logging is enabled.
#[inline(always)]
pub fn is_enabled() -> bool {
    LOGGING_ENABLED.load(Ordering::Relaxed)
}

/// Prints a structured log message if logging is enabled.
pub fn log(level: &str, target: &str, message: &str) {
    if !is_enabled() {
        return;
    }
    let elapsed = unsafe {
        START_TIME.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)
    };
    println!("[{:8.3}s] [{:<5}] [{}] {}", elapsed, level, target, message);
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)+) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("INFO", $target, &format!($($arg)+));
        }
    };
    ($msg:expr) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("INFO", "GAME", $msg);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)+) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("DEBUG", $target, &format!($($arg)+));
        }
    };
    ($msg:expr) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("DEBUG", "GAME", $msg);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)+) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("WARN", $target, &format!($($arg)+));
        }
    };
    ($msg:expr) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("WARN", "GAME", $msg);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)+) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("ERROR", $target, &format!($($arg)+));
        }
    };
    ($msg:expr) => {
        if $crate::core::logger::is_enabled() {
            $crate::core::logger::log("ERROR", "GAME", $msg);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_toggle() {
        init(false);
        assert!(!is_enabled());
        init(true);
        assert!(is_enabled());
        log("INFO", "Test", "Test message");
        init(false);
        assert!(!is_enabled());
    }
}
