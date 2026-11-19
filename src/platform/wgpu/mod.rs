//! Hardware WGPU (Vulkan / DX12 / Metal / OpenGL) offscreen world renderer backend.
//! Compiled only on targets with WGPU support (desktop + Android).

mod init;
mod pipelines;
mod render;
mod shaders;
mod targets;
mod textures;
mod types;
mod upload;

#[cfg(test)]
pub(crate) mod tests;

pub use types::{TEXTURE_LAYER_COUNT, VulkanContext};
