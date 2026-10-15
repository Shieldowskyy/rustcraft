#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
pub mod gamepad;
#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
pub mod mouse_capture;

#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
pub use gamepad::{GamepadEvents, GamepadManager, GamepadState};
#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
pub use mouse_capture::{maximize_window, MouseCapture};

#[cfg(target_os = "vita")]
pub mod vita;

#[cfg(target_os = "vita")]
pub mod vita_gxm;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
pub mod xbox360;
