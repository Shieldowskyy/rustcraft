//! Transitional compatibility facade preserving `crate::graphics::gpu::*` and WGPU invariant tests.

pub use crate::platform::wgpu::{TEXTURE_LAYER_COUNT, VulkanContext};

#[cfg(test)]
mod tests {
    #[test]
    fn test_vulkan_context_texture_array_creation() {
        crate::platform::wgpu::tests::check_vulkan_context_texture_array_creation();
    }
}
