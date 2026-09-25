use crate::graphics::camera::Camera;
use crate::world::World;

/// Abstraction trait for rendering backends (Software CPU rasterizer, WGPU / Vulkan / DX12 / Metal / GXM).
///
/// This trait decouples the world simulation and camera pipeline from specific graphics APIs,
/// allowing any backend to render voxel meshes, entities, and UI overlays.
pub trait RenderBackend {
    /// Returns the human-readable identifier of the active backend (e.g. `"software"`, `"vulkan"`, `"vita_gxm"`).
    fn backend_name(&self) -> &'static str;

    /// Returns an immutable reference to the front color pixel buffer formatted as packed `0x00RRGGBB` / `xRGB`.
    fn pixel_buffer(&self) -> &[u32];

    /// Returns an immutable slice of the floating-point depth buffer (Z-buffer).
    fn depth_buffer(&self) -> &[f32];

    /// Returns the framebuffer viewport dimensions `(width, height)` in pixels.
    fn dimensions(&self) -> (usize, usize);

    /// Clears the color and depth buffers (e.g., drawing the procedural dynamic sky gradient based on camera pitch).
    fn clear(&mut self, camera_pitch: f32);

    /// Renders a single 3D scene frame of voxel terrain, particles, and entities from the camera's perspective.
    fn render_frame(&mut self, world: &mut World, camera: &Camera);
}
