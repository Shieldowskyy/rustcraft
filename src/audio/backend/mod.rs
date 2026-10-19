//! Platform audio hardware abstraction layer.

pub mod null;
pub mod desktop;
pub mod wasm;
pub mod vita;
pub mod xbox360;
pub mod android;

/// Abstract hardware audio playback backend.
pub trait AudioBackend: Send {
    /// Initializes the audio device with the requested sample rate and channel layout.
    fn init(&mut self, sample_rate: u32, channels: u16) -> Result<(), String>;

    /// Outputs interleaved 32-bit floating point audio samples to the audio device.
    fn write_samples(&mut self, samples: &[f32]);

    /// Returns whether the audio backend is successfully open and playing.
    fn is_active(&self) -> bool;

    /// Identifier name of the active audio backend for logging.
    fn name(&self) -> &'static str;
}

/// Automatically selects and instantiates the optimal audio backend for current target.
pub fn create_default_backend() -> Box<dyn AudioBackend> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(wasm::WasmAudioBackend::new())
    }

    #[cfg(target_os = "vita")]
    {
        Box::new(vita::VitaAudioBackend::new())
    }

    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc"))]
    {
        Box::new(xbox360::Xbox360AudioBackend::new())
    }

    #[cfg(target_os = "android")]
    {
        Box::new(android::AndroidAudioBackend::new())
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_os = "xbox360"),
        not(target_os = "xenon"),
        not(target_arch = "powerpc"),
        not(target_os = "android")
    ))]
    {
        Box::new(desktop::DesktopAudioBackend::new())
    }
}
