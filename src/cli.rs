//! Command-line argument parsing and random seed generation for RustCraft.

const USAGE: &str = "Usage: rustcraft [--renderer software|vulkan] [--seed NUMBER]";

/// Parses command-line arguments to determine the desired rendering backend.
///
/// # Returns
/// * `"vulkan"` - Hardware-accelerated GPU pipeline via WGPU (default).
/// * `"software"` - Multi-threaded CPU software rasterizer.
pub fn parse_renderer_choice() -> &'static str {
    let args: Vec<String> = std::env::args().collect();
    let mut choice = "vulkan";
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--renderer" => {
                let value = args.get(index + 1).map(String::as_str).unwrap_or("");
                choice = match value {
                    "software" => "software",
                    "vulkan" => "vulkan",
                    _ => {
                        eprintln!("{USAGE}");
                        std::process::exit(2);
                    }
                };
                index += 1;
            }
            "--vulkan" => choice = "vulkan",
            "--software" => choice = "software",
            "--seed" => index += 1,
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
