//! Crash reporter lifecycle, breadcrumb buffer, and panic hook setup.
//! Delegates SDK-specific telemetry to `crate::platform::system::sentry_bridge`.

mod report;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::platform::system::sentry_bridge::{self, SentrySession};

/// Configuration options for the crash reporter.
#[derive(Debug, Clone)]
pub struct CrashReporterConfig {
    pub dsn: Option<String>,
    pub sentry_enabled: bool,
    pub environment: String,
    pub release: String,
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

/// Guard holding the Sentry client session. Dropping flushes queued events.
pub struct CrashGuard {
    _sentry_guard: SentrySession,
}

impl CrashGuard {
    pub fn flush(&self) {
        self._sentry_guard.flush();
    }
}

pub(super) static INITIALIZED: AtomicBool = AtomicBool::new(false);
pub(super) static SENTRY_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(super) static START_TIME: Mutex<Option<Instant>> = Mutex::new(None);
pub(super) static CONTEXT_MAP: Mutex<Option<Vec<(String, String)>>> = Mutex::new(None);
pub(super) static RECENT_LOGS: Mutex<Option<VecDeque<String>>> = Mutex::new(None);
pub(super) static CRASH_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

const MAX_RECENT_LOGS: usize = 40;

/// Initializes the crash reporter and platform telemetry integration.
pub fn init(mut config: CrashReporterConfig) -> CrashGuard {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return CrashGuard {
            _sentry_guard: sentry_bridge::inactive(),
        };
    }

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

    set_context("os", std::env::consts::OS);
    set_context("arch", std::env::consts::ARCH);
    set_context("version", env!("CARGO_PKG_VERSION"));
    set_context("environment", &config.environment);

    let sentry_guard = if config.sentry_enabled {
        let dsn = config.dsn.take();
        let has_dsn = dsn.is_some();
        let (session, active) = sentry_bridge::init(dsn, &config.release, &config.environment);
        if active {
            SENTRY_ACTIVE.store(true, Ordering::SeqCst);
            crate::log_info!("CRASH", "Sentry telemetry initialized ({})", config.release);
        } else if !has_dsn {
            crate::log_info!("CRASH", "Sentry telemetry is inactive (no DSN provided or disabled)");
        }
        session
    } else {
        crate::log_info!("CRASH", "Sentry telemetry is inactive (no DSN provided or disabled)");
        sentry_bridge::inactive()
    };

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        report::handle_panic(panic_info);
        default_hook(panic_info);
    }));

    CrashGuard {
        _sentry_guard: sentry_guard,
    }
}

pub fn is_sentry_active() -> bool {
    SENTRY_ACTIVE.load(Ordering::Relaxed)
}

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

    if is_sentry_active() {
        sentry_bridge::set_tag(key, value);
    }
}

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

    if is_sentry_active() {
        sentry_bridge::add_breadcrumb(level, target, message);
    }
}

