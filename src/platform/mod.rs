//! Platform Abstraction Layer (PAL): isolates all target-specific `#[cfg]` backends,
//! threading, storage paths, clipboard, OS hooks, and entrypoints from `core/` and `features/`.

pub mod audio;
pub mod clipboard;
pub mod gpu_bridge;
pub mod storage;
pub mod system;
pub mod threading;
pub mod traits;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_arch = "powerpc"),
    not(target_os = "vita"),
    not(target_os = "xbox360"),
    not(target_os = "xenon"),
    not(feature = "xbox360")
))]
pub mod wgpu;

pub use traits::{capabilities, PlatformBackend, PlatformCapabilities, PlatformConfig};

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_arch = "powerpc"),
    not(target_os = "vita"),
    not(target_os = "android"),
    not(target_os = "xbox360"),
    not(target_os = "xenon"),
    not(feature = "xbox360")
))]
pub mod desktop;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_arch = "powerpc"),
    not(target_os = "vita"),
    not(target_os = "android"),
    not(target_os = "xbox360"),
    not(target_os = "xenon"),
    not(feature = "xbox360")
))]
pub use desktop::{
    gamepad, mouse_capture, GamepadEvents, GamepadManager, GamepadState, MouseCapture,
    maximize_window,
};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "vita")]
pub mod vita;

#[cfg(target_os = "vita")]
pub use vita::gxm as vita_gxm;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(any(
    target_os = "xbox360",
    target_os = "xenon",
    target_arch = "powerpc",
    feature = "xbox360"
))]
pub mod xbox360;

/// Launches the active target's native application entrypoint.
pub fn run_app() {
    #[cfg(target_os = "vita")]
    {
        vita::install_vita_panic_hook();
        crate::vita_log!("Entered main() on PS Vita, launching run_vita_app()...");
        vita::run_vita_app();
    }
    #[cfg(target_os = "android")]
    {
        // Android entrypoints live in `platform::android` (JNI / cdylib).
    }
    #[cfg(any(
        target_os = "xbox360",
        target_os = "xenon",
        target_arch = "powerpc",
        feature = "xbox360"
    ))]
    {
        xbox360::run_xbox360_app();
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_arch = "powerpc"),
        not(target_os = "vita"),
        not(target_os = "android"),
        not(target_os = "xbox360"),
        not(target_os = "xenon"),
        not(feature = "xbox360")
    ))]
    {
        desktop::run_desktop_app();
    }
}
