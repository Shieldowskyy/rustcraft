//! Embedded HTTP profiler server module facade and static web dashboard asset.

mod server;
mod snapshot;

#[cfg(test)]
mod tests;

pub use server::ProfilerServer;
pub use snapshot::{encode_bmp, ProfilerSnapshot, ScreenshotData};

/// Embedded self-contained HTML/CSS/JS dark-mode dashboard.
pub const DASHBOARD_HTML: &str = include_str!("dashboard.html");

