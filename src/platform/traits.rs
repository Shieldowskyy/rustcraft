//! Platform contracts (`PlatformBackend`, `PlatformConfig`, `PlatformCapabilities`).
//! Defines compile-time and runtime capabilities without exposing `#[cfg]` to `core/` or `features/`.
//! Implemented by desktop, PS Vita, Android, macOS, and Web/WASM platform backends.

use crate::app::{AppOutput, InputState};

/// Configuration options for creating a platform host window / context.
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub title: String,
    pub width: usize,
    pub height: usize,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            title: "RustCraft".to_string(),
            width: 960,
            height: 540,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// Abstract hardware/OS platform backend interface.
pub trait PlatformBackend {
    fn init(&mut self, config: &PlatformConfig) -> Result<(), String>;
    fn poll_events(&mut self, input: &mut InputState) -> bool;
    fn present_framebuffer(&mut self, buffer: &[u32], width: usize, height: usize) -> Result<(), String>;
    fn set_title(&mut self, title: &str);
    fn capture_mouse(&mut self, captured: bool);
    fn current_time_seconds(&self) -> f64;

    fn handle_output(&mut self, output: &AppOutput) {
        if output.request_capture_mouse {
            self.capture_mouse(true);
        }
        if output.request_release_mouse {
            self.capture_mouse(false);
        }
        if !output.title.is_empty() {
            self.set_title(&output.title);
        }
    }
}

/// Target capabilities exposed to `core`, `features`, and `app` to eliminate platform `#[cfg]`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub is_vita: bool,
    pub is_android: bool,
    pub is_wasm: bool,
    pub is_macos: bool,
    pub is_desktop: bool,
    pub is_xbox360: bool,
    pub supports_threads: bool,
    pub supports_async_world_load: bool,
    pub supports_wgpu: bool,
    pub supports_lan_host: bool,
    pub supports_rebind_controls: bool,
    pub supports_dev_logo_file: bool,
    pub supports_profiler_server: bool,
    pub uses_hardware_gxm: bool,
    pub skip_transparent_hud_tint: bool,
    pub default_msaa: u8,
    pub default_fps_limit: u32,
    pub initial_spawn_radius: i32,
    pub chunk_voxel_ram_bytes: usize,
    pub max_chunk_mesh_triangles: Option<usize>,
    pub profiler_sample_interval: u32,
    pub short_id: &'static str,
    pub display_name: &'static str,
    pub confirm_label: &'static str,
    pub back_label: &'static str,
}

/// Returns compile-time target capabilities for the active build target.
pub const fn capabilities() -> PlatformCapabilities {
    #[cfg(target_os = "vita")]
    {
        return PlatformCapabilities {
            is_vita: true,
            is_android: false,
            is_wasm: false,
            is_macos: false,
            is_desktop: false,
            is_xbox360: false,
            supports_threads: true,
            supports_async_world_load: false,
            supports_wgpu: false,
            supports_lan_host: false,
            supports_rebind_controls: false,
            supports_dev_logo_file: false,
            supports_profiler_server: true,
            uses_hardware_gxm: true,
            skip_transparent_hud_tint: true,
            default_msaa: 4,
            default_fps_limit: 30,
            initial_spawn_radius: 1,
            chunk_voxel_ram_bytes: 16 * 16 * 64,
            // DLACZEGO: Matches GxmRenderer::MAX_VERTS_PER_SLOT / 3 (18432 / 3 = 6144 tris) to fit Vita's fixed per-slot VB budget.
            max_chunk_mesh_triangles: Some(6144),
            profiler_sample_interval: 3,
            short_id: "psvita",
            display_name: "PS Vita",
            confirm_label: "[X]",
            back_label: "[O]",
        };
    }
    #[cfg(target_os = "android")]
    {
        return PlatformCapabilities {
            is_vita: false,
            is_android: true,
            is_wasm: false,
            is_macos: false,
            is_desktop: false,
            is_xbox360: false,
            supports_threads: true,
            supports_async_world_load: true,
            supports_wgpu: true,
            supports_lan_host: true,
            supports_rebind_controls: false,
            supports_dev_logo_file: true,
            supports_profiler_server: true,
            uses_hardware_gxm: false,
            skip_transparent_hud_tint: false,
            default_msaa: 1,
            default_fps_limit: 0,
            initial_spawn_radius: 2,
            chunk_voxel_ram_bytes: 16 * 16 * 128,
            max_chunk_mesh_triangles: None,
            profiler_sample_interval: 1,
            short_id: "android",
            display_name: "Android",
            confirm_label: "(ENTER)",
            back_label: "(ESC)",
        };
    }
    #[cfg(target_arch = "wasm32")]
    {
        return PlatformCapabilities {
            is_vita: false,
            is_android: false,
            is_wasm: true,
            is_macos: false,
            is_desktop: false,
            is_xbox360: false,
            supports_threads: false,
            supports_async_world_load: false,
            supports_wgpu: false,
            supports_lan_host: false,
            supports_rebind_controls: true,
            supports_dev_logo_file: false,
            supports_profiler_server: false,
            uses_hardware_gxm: false,
            skip_transparent_hud_tint: true,
            default_msaa: 1,
            default_fps_limit: 0,
            initial_spawn_radius: 2,
            chunk_voxel_ram_bytes: 16 * 16 * 128,
            max_chunk_mesh_triangles: None,
            profiler_sample_interval: 1,
            short_id: "wasm",
            display_name: "WebAssembly",
            confirm_label: "(ENTER)",
            back_label: "(ESC)",
        };
    }
    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    {
        return PlatformCapabilities {
            is_vita: false,
            is_android: false,
            is_wasm: false,
            is_macos: false,
            is_desktop: false,
            is_xbox360: true,
            supports_threads: false,
            supports_async_world_load: false,
            supports_wgpu: false,
            supports_lan_host: false,
            supports_rebind_controls: false,
            supports_dev_logo_file: false,
            supports_profiler_server: false,
            uses_hardware_gxm: false,
            skip_transparent_hud_tint: false,
            default_msaa: 1,
            default_fps_limit: 0,
            initial_spawn_radius: 1,
            chunk_voxel_ram_bytes: 16 * 16 * 128,
            max_chunk_mesh_triangles: None,
            profiler_sample_interval: 1,
            short_id: "xbox360",
            display_name: "Xbox 360",
            confirm_label: "(ENTER)",
            back_label: "(ESC)",
        };
    }
    #[cfg(not(any(
        target_os = "vita",
        target_os = "android",
        target_arch = "wasm32",
        target_os = "xbox360",
        target_os = "xenon",
        target_arch = "powerpc",
        feature = "xbox360"
    )))]
    {
        PlatformCapabilities {
            is_vita: false,
            is_android: false,
            is_wasm: false,
            is_macos: cfg!(target_os = "macos"),
            is_desktop: true,
            is_xbox360: false,
            supports_threads: true,
            supports_async_world_load: true,
            supports_wgpu: true,
            supports_lan_host: true,
            supports_rebind_controls: true,
            supports_dev_logo_file: true,
            supports_profiler_server: true,
            uses_hardware_gxm: false,
            skip_transparent_hud_tint: false,
            default_msaa: 1,
            default_fps_limit: 0,
            initial_spawn_radius: 2,
            chunk_voxel_ram_bytes: 16 * 16 * 128,
            max_chunk_mesh_triangles: None,
            profiler_sample_interval: 1,
            short_id: if cfg!(target_os = "macos") { "macos" } else { "desktop" },
            display_name: "PC / Desktop",
            confirm_label: "(ENTER)",
            back_label: "(ESC)",
        }
    }
}

