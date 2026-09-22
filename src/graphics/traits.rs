use crate::graphics::camera::Camera;
use crate::world::World;

/// Standardowy trait abstrakcji backendu graficznego (Software CPU, WGPU / Vulkan / DX12 / Metal itp.).
pub trait RenderBackend {
    /// Nazwa aktywnego backendu (np. "software", "vulkan", "webgpu")
    fn backend_name(&self) -> &'static str;

    /// Zwraca referencję do bufora pikseli koloru ramki (xRGB / ARGB)
    fn pixel_buffer(&self) -> &[u32];

    /// Zwraca bufor głębokości (z-buffer)
    fn depth_buffer(&self) -> &[f32];

    /// Wymiary ramki renderera (szerokość, wysokość)
    fn dimensions(&self) -> (usize, usize);

    /// Wyczyszczenie bufora (np. gradientem nieba)
    fn clear(&mut self, camera_pitch: f32);

    /// Renderowanie klatki świata 3D
    fn render_frame(&mut self, world: &mut World, camera: &Camera);
}
