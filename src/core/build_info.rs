//! Compile-time build information, platform telemetry, and compiler flags.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATETIME: &str = env!("RUSTCRAFT_BUILD_DATETIME");
pub const BUILD_PROFILE: &str = env!("RUSTCRAFT_BUILD_PROFILE");
pub const BUILD_TARGET: &str = env!("RUSTCRAFT_BUILD_TARGET");
pub const BUILD_PLATFORM: &str = env!("RUSTCRAFT_BUILD_PLATFORM");
pub const BUILD_FLAGS: &str = env!("RUSTCRAFT_BUILD_FLAGS");
pub const BUILD_INFO: &str = env!("RUSTCRAFT_BUILD_INFO");

/// Returns the formatted build information string (version, profile, date/time, platform, flags).
#[inline]
pub fn get_build_info() -> &'static str {
    BUILD_INFO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_contents() {
        let info = get_build_info();
        assert!(!info.is_empty(), "Build info string must not be empty");
        assert!(info.contains("RustCraft v"), "Build info must contain version prefix");
        assert!(!BUILD_DATETIME.is_empty(), "Build datetime must not be empty");
        assert!(!BUILD_PLATFORM.is_empty(), "Build platform must not be empty");
        assert!(!BUILD_PROFILE.is_empty(), "Build profile must not be empty");
        assert!(!BUILD_FLAGS.is_empty(), "Build flags must not be empty");
        println!("Test build info output: {}", info);
    }
}

