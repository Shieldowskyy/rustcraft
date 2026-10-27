pub mod build_info;
pub mod controls;
pub mod crash_reporter;
pub mod i18n;
pub mod math;
pub mod noise;
pub mod logger;
pub mod profiler;
pub mod profiler_server;
pub mod settings;

pub use build_info::get_build_info;

pub use controls::{Action, ControlsConfig, ControlsManager};
pub use crash_reporter::{
    init as init_crash_reporter, is_sentry_active, record_log_entry, set_context as set_crash_context,
    CrashGuard, CrashReporterConfig,
};
pub use i18n::{available_languages, current_language, set_language, t, t_fmt, Language};
pub use logger::{init as init_logger, is_enabled as is_log_enabled, log as log_message};
pub use math::{IVec3, Mat4, Vec3};
pub use noise::{fractal_noise2, fractal_noise3, value_noise2, value_noise3};
pub use profiler::{FrameProfiler, MemoryBreakdown, ProfilerSlice, RenderStats};
pub use profiler_server::{ProfilerServer, ProfilerSnapshot};
pub use settings::GameSettings;

