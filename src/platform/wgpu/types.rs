//! Shared WGPU types, uniform buffer layout, and `VulkanContext` handle.
//! Declares `TEXTURE_LAYER_COUNT` synchronized with `features::rendering::atlas`.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use crate::features::rendering::atlas::{BUILTIN_TEXTURE_COUNT, CUSTOM_TEXTURE_LAYERS};

pub const TEXTURE_LAYER_COUNT: u32 = (BUILTIN_TEXTURE_COUNT as u32) + CUSTOM_TEXTURE_LAYERS;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct GpuVertex {
    pub(super) position: [f32; 3],
    pub(super) uv: [u8; 2],
    pub(super) _pad: [u8; 2],
    pub(super) light: [u8; 4],
    pub(super) texture_layer: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Globals {
    pub(super) view_proj: [[f32; 4]; 4],
    pub(super) inv_view_proj: [[f32; 4]; 4],
    pub(super) camera_position: [f32; 4],
    pub(super) camera_pitch: [f32; 4],
    pub(super) sky_top: [f32; 4],
    pub(super) sky_bot: [f32; 4],
    pub(super) fog_color: [f32; 4],
    pub(super) sun_dir: [f32; 4],
    pub(super) moon_dir: [f32; 4],
}

pub(super) struct GpuChunk {
    pub(super) opaque: ::wgpu::Buffer,
    pub(super) opaque_capacity: usize,
    pub(super) opaque_count: u32,
    pub(super) cutout: ::wgpu::Buffer,
    pub(super) cutout_capacity: usize,
    pub(super) cutout_count: u32,
    pub(super) transparent: ::wgpu::Buffer,
    pub(super) transparent_capacity: usize,
    pub(super) transparent_count: u32,
}

/// Offscreen WGPU world renderer supporting Vulkan, DX12, Metal, and OpenGL/GLES backends.
pub struct VulkanContext {
    pub(super) device: ::wgpu::Device,
    pub(super) queue: ::wgpu::Queue,
    pub(super) texture: ::wgpu::Texture,
    pub(super) pipeline: ::wgpu::RenderPipeline,
    pub(super) cutout_pipeline: ::wgpu::RenderPipeline,
    pub(super) transparent_pipeline: ::wgpu::RenderPipeline,
    pub(super) sky_pipeline: ::wgpu::RenderPipeline,
    pub(super) bind_group: ::wgpu::BindGroup,
    pub(super) globals_buffer: ::wgpu::Buffer,
    pub(super) instance_buffer: ::wgpu::Buffer,
    pub(super) chunk_buffers: HashMap<(i32, i32), GpuChunk>,
    pub(super) target_view: ::wgpu::TextureView,
    pub(super) linear_depth_view: ::wgpu::TextureView,
    pub(super) depth_view: ::wgpu::TextureView,
    pub(super) post_pipeline: ::wgpu::ComputePipeline,
    pub(super) post_bind_layout: ::wgpu::BindGroupLayout,
    pub(super) post_bind_group: [::wgpu::BindGroup; 2],
    pub(super) color_ssbo: [::wgpu::Buffer; 2],
    pub(super) depth_ssbo: [::wgpu::Buffer; 2],
    pub(super) readback: [::wgpu::Buffer; 2],
    pub(super) depth_readback: [::wgpu::Buffer; 2],
    pub(super) frame_index: usize,
    pub width: u32,
    pub height: u32,
    pub(super) gpu_name: String,
    pub(super) driver_info: String,
    pub(super) backend_version: String,
    pub(super) backend: ::wgpu::Backend,
}
