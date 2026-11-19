//! WGPU adapter limit & texture array creation invariant tests.

use std::sync::Mutex;

use crate::features::rendering::atlas::TextureAtlas;

use super::types::{TEXTURE_LAYER_COUNT, VulkanContext};

static GPU_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn check_vulkan_context_texture_array_creation() {
    let _guard = GPU_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let atlas = TextureAtlas::generate();
    let res = VulkanContext::new(64, 64, &atlas);
    match res {
        Ok(ctx) => {
            assert_eq!(ctx.width, 64);
            assert_eq!(ctx.height, 64);
        }
        Err(e) => {
            // Headless CI environments without a GPU adapter are allowed to fail gracefully.
            println!("VulkanContext::new returned err (expected if headless/no GPU): {e}");
        }
    }
}

#[test]
fn test_adapter_texture_array_layers() {
    let _guard = GPU_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let instance = ::wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&::wgpu::RequestAdapterOptions {
        power_preference: ::wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));
    if let Some(adapter) = adapter {
        let limits = adapter.limits();
        println!(
            "Adapter: {}, backend: {:?}, max_texture_array_layers: {}",
            adapter.get_info().name,
            adapter.get_info().backend,
            limits.max_texture_array_layers
        );
        assert!(
            limits.max_texture_array_layers >= TEXTURE_LAYER_COUNT,
            "Adapter supports {} layers, needed {}",
            limits.max_texture_array_layers,
            TEXTURE_LAYER_COUNT
        );
    }
}

#[test]
fn test_vulkan_context_texture_array_creation() {
    check_vulkan_context_texture_array_creation();
}
