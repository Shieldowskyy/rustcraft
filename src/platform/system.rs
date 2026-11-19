//! OS-specific system utilities: hard RAM limits, local IP discovery, monotonic fallback, and Sentry bridge.
//! Concentrates `RLIMIT_DATA`, Win32 Job Objects, Vita `SceNetCtl`, Xbox360 clock, and `sentry` SDK in `platform/`.
//! Called by `core/` subsystems so `core/` stays free of OS/SDK `#[cfg]` gates.

use std::time::Instant;

/// Enforces a hard process memory cap on supported desktop targets (`RLIMIT_DATA` on Linux, Job Object on Windows).
#[cfg(target_os = "linux")]
pub fn apply_hard_ram_limit(mb: u64) -> Result<(), String> {
    #[repr(C)]
    struct RLimit {
        cur: u64,
        max: u64,
    }
    extern "C" {
        fn setrlimit(resource: i32, rlim: *const RLimit) -> i32;
    }
    const RLIMIT_DATA: i32 = 2;
    let bytes = mb.saturating_mul(1024 * 1024);
    let lim = RLimit {
        cur: bytes,
        max: bytes,
    };
    if unsafe { setrlimit(RLIMIT_DATA, &lim) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

#[cfg(windows)]
pub fn apply_hard_ram_limit(mb: u64) -> Result<(), String> {
    use windows_sys::Win32::System::JobObjects::*;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        info.ProcessMemoryLimit = (mb.saturating_mul(1024 * 1024)) as usize;
        let ok = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if ok == 0 || AssignProcessToJobObject(job, GetCurrentProcess()) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", windows)))]
pub fn apply_hard_ram_limit(_mb: u64) -> Result<(), String> {
    Err("hard RAM limit is not supported on this platform".to_string())
}

/// Discovers local LAN IP across desktop and PS Vita (`SceNetCtl`).
pub fn detect_local_ip() -> String {
    #[cfg(target_os = "vita")]
    {
        #[repr(C)]
        struct SceNetCtlInfo {
            ip_address: [u8; 16],
            _padding: [u8; 240],
        }

        #[link(name = "SceNetCtl_stub")]
        extern "C" {
            fn sceNetCtlInit() -> i32;
            fn sceNetCtlInetGetInfo(code: i32, info: *mut SceNetCtlInfo) -> i32;
        }

        unsafe {
            let _ = sceNetCtlInit();
            let mut info = std::mem::zeroed::<SceNetCtlInfo>();
            // 4 = SCE_NETCTL_INFO_IP_ADDRESS
            if sceNetCtlInetGetInfo(4, &mut info) >= 0 {
                let bytes = &info.ip_address;
                let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
                if let Ok(s) = std::str::from_utf8(&bytes[..len]) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() && trimmed != "0.0.0.0" {
                        return trimmed.to_string();
                    }
                }
            }
        }
    }

    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "0.0.0.0" && !ip.starts_with("127.") {
                    return ip;
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Returns elapsed log timestamp in seconds, using Xbox 360 timebase on Xenon/PowerPC targets.
#[inline]
pub fn elapsed_log_seconds(start: Option<Instant>) -> f64 {
    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    {
        let _ = start;
        crate::platform::xbox360::get_xbox_elapsed_ms() as f64 / 1000.0
    }
    #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
    {
        start.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)
    }
}

/// Logs a diagnostic line to the PS Vita debug log (`ux0:data/rustcraft/debug.log`) on Vita, no-op elsewhere.
#[inline]
pub fn vita_debug_log(msg: &str) {
    #[cfg(target_os = "vita")]
    {
        crate::vita_log!("{}", msg);
    }
    #[cfg(not(target_os = "vita"))]
    {
        let _ = msg;
    }
}

/// Sentry telemetry bridge (active on desktop targets, zero-cost stub on embedded/WASM).
pub mod sentry_bridge {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_arch = "powerpc"),
        not(target_os = "vita"),
        not(target_os = "android"),
        not(target_os = "xbox360"),
        not(target_os = "xenon")
    ))]
    pub struct SentrySession {
        guard: Option<sentry::ClientInitGuard>,
    }

    #[cfg(any(
        target_arch = "wasm32",
        target_arch = "powerpc",
        target_os = "vita",
        target_os = "android",
        target_os = "xbox360",
        target_os = "xenon"
    ))]
    pub struct SentrySession;

    impl SentrySession {
        pub fn flush(&self) {
            #[cfg(all(
                not(target_arch = "wasm32"),
                not(target_arch = "powerpc"),
                not(target_os = "vita"),
                not(target_os = "android"),
                not(target_os = "xbox360"),
                not(target_os = "xenon")
            ))]
            if self.guard.is_some() {
                if let Some(client) = sentry::Hub::current().client() {
                    client.flush(Some(std::time::Duration::from_secs(2)));
                }
            }
        }
    }

    pub fn inactive() -> SentrySession {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon")
        ))]
        {
            SentrySession { guard: None }
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon"
        ))]
        {
            SentrySession
        }
    }

    pub fn init(dsn: Option<String>, release: &str, environment: &str) -> (SentrySession, bool) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon")
        ))]
        {
            if let Some(dsn_str) = dsn {
                let mut options = sentry::ClientOptions::default();
                options.release = Some(release.to_string().into());
                options.environment = Some(environment.to_string().into());
                options.attach_stacktrace = true;
                options.send_default_pii = false;

                let guard = sentry::init((dsn_str, options));
                let active = guard.is_enabled();
                return (SentrySession { guard: Some(guard) }, active);
            }
            (SentrySession { guard: None }, false)
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon"
        ))]
        {
            let _ = (dsn, release, environment);
            (SentrySession, false)
        }
    }

    pub fn set_tag(key: &str, value: &str) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon")
        ))]
        sentry::configure_scope(|scope| {
            scope.set_tag(key, value);
        });
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon"
        ))]
        let _ = (key, value);
    }

    pub fn add_breadcrumb(level: &str, target: &str, message: &str) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon")
        ))]
        {
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
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon"
        ))]
        let _ = (level, target, message);
    }

    pub fn capture_panic(
        thread_name: &str,
        message: &str,
        location: &str,
        contexts: &[(String, String)],
    ) -> Option<String> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon")
        ))]
        {
            let mut event = sentry::protocol::Event {
                message: Some(format!("Panic in thread '{}': {}", thread_name, message)),
                level: sentry::Level::Fatal,
                ..Default::default()
            };
            for (k, v) in contexts {
                event.tags.insert(k.clone(), v.clone());
            }
            event.tags.insert("panic_location".to_string(), location.to_string());
            event.tags.insert("thread".to_string(), thread_name.to_string());

            let id = sentry::capture_event(event);
            if let Some(client) = sentry::Hub::current().client() {
                client.flush(Some(std::time::Duration::from_secs(2)));
            }
            Some(id.to_string())
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon"
        ))]
        {
            let _ = (thread_name, message, location, contexts);
            None
        }
    }
}

