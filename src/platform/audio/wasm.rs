//! Backend audio HTML5 Web Audio API dla przeglądarek (WASM).

use super::AudioBackend;

pub struct WasmAudioBackend {
    active: bool,
    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    ctx: Option<web_sys::AudioContext>,
}

impl WasmAudioBackend {
    pub fn new() -> Self {
        Self {
            active: false,
            #[cfg(target_arch = "wasm32")]
            ctx: None,
        }
    }
}

impl Default for WasmAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for WasmAudioBackend {
    fn init(&mut self, _sample_rate: u32, _channels: u16) -> Result<(), String> {
        self.active = true;
        Ok(())
    }

    fn write_samples(&mut self, _samples: &[f32]) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn name(&self) -> &'static str {
        "HTML5 Web Audio API"
    }
}
