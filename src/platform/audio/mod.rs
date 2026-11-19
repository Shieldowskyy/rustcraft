//! Warstwa abstrakcji sprzętowego odtwarzania dźwięku (Linux, Windows, PS Vita, Android, WASM, Xbox 360, Null).

#[cfg(target_os = "android")]
pub mod android;
pub mod desktop;
pub mod null;
pub mod vita;
pub mod wasm;
pub mod xbox360;

pub trait AudioBackend: Send {
    fn init(&mut self, sample_rate: u32, channels: u16) -> Result<(), String>;
    fn write_samples(&mut self, samples: &[f32]);
    fn is_active(&self) -> bool;
    fn name(&self) -> &'static str;
}

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
