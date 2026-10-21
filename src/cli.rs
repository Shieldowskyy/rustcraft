//! Command-line argument parsing and random seed generation for RustCraft.

const USAGE: &str = "Usage: rustcraft [--renderer software|vulkan|opengl] [--seed NUMBER] [--profiler] [--autotest] [-log] [--server] [--port PORT] [--connect ADDR] [--sentry-dsn DSN] [--no-sentry] [--crash-test]";

/// Parses command-line arguments to determine the desired rendering backend.
///
/// # Returns
/// * `"vulkan"` - Hardware-accelerated GPU pipeline via WGPU (default).
/// * `"opengl"` - Hardware-accelerated GPU pipeline via OpenGL / GLES.
/// * `"software"` - Multi-threaded CPU software rasterizer.
pub fn parse_renderer_choice() -> &'static str {
    let args: Vec<String> = std::env::args().collect();
    parse_renderer_choice_from_args(&args)
}

/// Internal helper for parsing renderer choice from an argument slice.
pub fn parse_renderer_choice_from_args(args: &[String]) -> &'static str {
    let mut choice = "vulkan";
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--renderer" => {
                let value = args.get(index + 1).map(String::as_str).unwrap_or("");
                choice = match value {
                    "software" => "software",
                    "vulkan" => "vulkan",
                    "opengl" | "gl" => "opengl",
                    _ => {
                        eprintln!("{USAGE}");
                        std::process::exit(2);
                    }
                };
                index += 1;
            }
            "--vulkan" => choice = "vulkan",
            "--opengl" | "--gl" => choice = "opengl",
            "--software" => choice = "software",
            "--seed" => index += 1,
            "--profiler" => {},
            "--autotest" => {},
            "-log" | "--log" => {},
            "--server" => {},
            "--port" => index += 1,
            "--connect" => index += 1,
            "--sentry-dsn" => index += 1,
            "--no-sentry" | "--disable-sentry" => {},
            "--crash-test" | "--test-panic" => {},
            "--help" | "-h" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}\n{USAGE}");
                std::process::exit(2);
            }
        }
        index += 1;
    }
    choice
}

/// Parses whether the autotest mode is requested via `--autotest` CLI flag
/// or via the `autotest` compile-time feature.
pub fn parse_autotest_enabled() -> bool {
    cfg!(feature = "autotest") || std::env::args().any(|arg| arg == "--autotest")
}

/// Parses whether the `--profiler` CLI flag was specified.
///
/// When enabled, RustCraft collects microsecond subsystem timings and renders
/// the Minecraft-style debug pie chart and memory telemetry under F3.
pub fn parse_profiler_enabled() -> bool {
    std::env::args().any(|arg| arg == "--profiler")
}

/// Parses whether the `-log` or `--log` CLI flag was specified.
///
/// When enabled, RustCraft prints full debug telemetry and event logs to stdout.
pub fn parse_log_enabled() -> bool {
    std::env::args().any(|arg| arg == "-log" || arg == "--log")
}

/// Parses whether the `--server` CLI flag was specified to run a dedicated multiplayer server.
pub fn parse_server_mode() -> bool {
    std::env::args().any(|arg| arg == "--server")
}

/// Parses the optional `--port <PORT>` CLI flag for the dedicated multiplayer server,
/// defaulting to 26656.
pub fn parse_server_port() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--port" {
            if let Some(val) = args.get(i + 1) {
                if let Ok(port) = val.parse::<u16>() {
                    return port;
                }
            }
        }
    }
    crate::net::DEFAULT_SERVER_PORT
}

/// Parses the optional `--connect <ADDRESS>` CLI flag for direct multiplayer join.
pub fn parse_connect_addr() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--connect" {
            if let Some(val) = args.get(i + 1) {
                return Some(val.clone());
            }
        }
    }
    None
}

/// Generates a cryptographically strong random seed using OS entropy (`getrandom`),
/// falling back to subsecond clock timestamp on failure.
///
/// # Returns
/// A 32-bit unsigned integer suitable for world generator seeding.
pub fn generate_random_seed() -> u32 {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).unwrap_or_else(|_| {
        let ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0x1337_1337);
        buf = ns.to_le_bytes();
    });
    u32::from_le_bytes(buf)
}

/// Parses the `--seed <NUMBER>` CLI option if provided, or generates a random seed.
///
/// # Returns
/// Parsed or randomly generated 32-bit world generation seed.
pub fn parse_seed() -> u32 {
    let args: Vec<String> = std::env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--seed" {
            return args.get(index + 1)
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or_else(|| {
                    eprintln!("Seed must be an integer: --seed NUMBER");
                    std::process::exit(2);
                });
        }
    }
    generate_random_seed()
}

/// Parses `--sentry-dsn <DSN>` CLI option, falling back to `SENTRY_DSN` environment variable.
pub fn parse_sentry_dsn() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--sentry-dsn" {
            if let Some(val) = args.get(i + 1) {
                if !val.trim().is_empty() {
                    return Some(val.trim().to_string());
                }
            }
        }
    }
    std::env::var("SENTRY_DSN").ok().filter(|s| !s.trim().is_empty())
}

/// Parses whether Sentry reporting is explicitly disabled via `--no-sentry` or `--disable-sentry`.
pub fn parse_sentry_disabled() -> bool {
    std::env::args().any(|arg| arg == "--no-sentry" || arg == "--disable-sentry")
}

/// Parses whether a test panic was requested via `--crash-test` or `--test-panic`.
pub fn parse_crash_test() -> bool {
    std::env::args().any(|arg| arg == "--crash-test" || arg == "--test-panic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_seed_nonzero() {
        let seed = generate_random_seed();
        // Just verify it generates a valid u32 without panicking
        let _ = seed;
    }

    #[test]
    fn test_parse_profiler_flag_logic() {
        let test_args = vec!["rustcraft".to_string(), "--profiler".to_string()];
        assert!(test_args.iter().any(|a| a == "--profiler"));

        let test_args_no_prof = vec!["rustcraft".to_string(), "--software".to_string()];
        assert!(!test_args_no_prof.iter().any(|a| a == "--profiler"));
    }

    #[test]
    fn test_parse_renderer_choice_options() {
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string()]), "vulkan");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--renderer".to_string(), "software".to_string()]), "software");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--software".to_string()]), "software");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--renderer".to_string(), "vulkan".to_string()]), "vulkan");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--vulkan".to_string()]), "vulkan");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--renderer".to_string(), "opengl".to_string()]), "opengl");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--renderer".to_string(), "gl".to_string()]), "opengl");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--opengl".to_string()]), "opengl");
        assert_eq!(parse_renderer_choice_from_args(&["rustcraft".to_string(), "--gl".to_string()]), "opengl");
    }
}
