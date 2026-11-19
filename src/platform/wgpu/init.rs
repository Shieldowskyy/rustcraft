//! WGPU adapter probing, device initialization, and texture array limit verification.

use std::collections::HashMap;

use crate::features::rendering::atlas::TextureAtlas;

use super::types::{TEXTURE_LAYER_COUNT, VulkanContext};

impl VulkanContext {
    /// Creates a WGPU offscreen renderer using `WGPU_BACKEND` or primary + GL auto-selection.
    pub fn new(width: usize, height: usize, atlas: &TextureAtlas) -> Result<Self, String> {
        Self::new_with_backends(width, height, atlas, None)
    }

    /// Creates a WGPU offscreen renderer restricted to `preferred_backends` when provided.
    pub fn new_with_backends(
        width: usize,
        height: usize,
        atlas: &TextureAtlas,
        preferred_backends: Option<::wgpu::Backends>,
    ) -> Result<Self, String> {
        let requested_backends = if let Some(b) = preferred_backends {
            b
        } else if let Ok(val) = std::env::var("WGPU_BACKEND") {
            match val.to_lowercase().as_str() {
                "gl" | "opengl" | "gles" => ::wgpu::Backends::GL,
                "vulkan" | "vk" => ::wgpu::Backends::VULKAN,
                "dx12" => ::wgpu::Backends::DX12,
                "metal" => ::wgpu::Backends::METAL,
                "primary" => ::wgpu::Backends::PRIMARY,
                "all" => ::wgpu::Backends::all(),
                _ => ::wgpu::Backends::PRIMARY | ::wgpu::Backends::GL,
            }
        } else {
            ::wgpu::Backends::PRIMARY | ::wgpu::Backends::GL
        };

        let create_instance = |b: ::wgpu::Backends| {
            ::wgpu::Instance::new(::wgpu::InstanceDescriptor {
                backends: b,
                dx12_shader_compiler: Default::default(),
                gles_minor_version: ::wgpu::Gles3MinorVersion::Automatic,
                flags: ::wgpu::InstanceFlags::default(),
            })
        };

        let instance = create_instance(requested_backends);
        let mut adapter = pollster::block_on(instance.request_adapter(&::wgpu::RequestAdapterOptions {
            power_preference: ::wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));

        // DLACZEGO: Older Intel iGPUs may expose a CPU Vulkan fallback (Lavapipe/LLVMpipe)
        // while supporting hardware OpenGL; upgrade to GL unless a backend was pinned.
        if preferred_backends.is_none()
            && adapter.as_ref().map(|a| a.get_info().device_type) == Some(::wgpu::DeviceType::Cpu)
        {
            let gl_instance = create_instance(::wgpu::Backends::GL);
            if let Some(gl_adapter) =
                pollster::block_on(gl_instance.request_adapter(&::wgpu::RequestAdapterOptions {
                    power_preference: ::wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }))
            {
                if gl_adapter.get_info().device_type != ::wgpu::DeviceType::Cpu {
                    adapter = Some(gl_adapter);
                }
            }
        }

        if adapter.is_none() && requested_backends != ::wgpu::Backends::all() {
            let all_instance = create_instance(::wgpu::Backends::all());
            adapter = pollster::block_on(all_instance.request_adapter(&::wgpu::RequestAdapterOptions {
                power_preference: ::wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }));
        }

        let adapter =
            adapter.ok_or_else(|| "no compatible GPU adapter found supporting rendering".to_owned())?;

        let adapter_info = adapter.get_info();
        let backend = adapter_info.backend;
        let gpu_name = adapter_info.name.clone();
        let driver_info = if !adapter_info.driver_info.is_empty() {
            if !adapter_info.driver.is_empty() && !adapter_info.driver_info.contains(&adapter_info.driver) {
                format!("{} {}", adapter_info.driver, adapter_info.driver_info)
            } else {
                adapter_info.driver_info.clone()
            }
        } else if !adapter_info.driver.is_empty() {
            adapter_info.driver.clone()
        } else {
            "GPU Driver".to_string()
        };
        let backend_version = match adapter_info.backend {
            ::wgpu::Backend::Vulkan => "Vulkan".to_string(),
            ::wgpu::Backend::Gl => "OpenGL / GLES".to_string(),
            ::wgpu::Backend::Dx12 => "DirectX 12".to_string(),
            ::wgpu::Backend::Metal => "Metal".to_string(),
            ::wgpu::Backend::BrowserWebGpu => "WebGPU".to_string(),
            _ => format!("{:?}", adapter_info.backend),
        };

        let adapter_limits = adapter.limits();
        if adapter_limits.max_texture_array_layers < TEXTURE_LAYER_COUNT {
            return Err(format!(
                "GPU adapter '{}' ({:?}) supports at most {} texture array layers, but RustCraft requires at least {}.",
                gpu_name,
                adapter_info.backend,
                adapter_limits.max_texture_array_layers,
                TEXTURE_LAYER_COUNT
            ));
        }

        // DLACZEGO: Forward the adapter's full max_texture_array_layers so the D2Array atlas
        // never gets clipped below TEXTURE_LAYER_COUNT by wgpu's default limits.
        let required_limits = ::wgpu::Limits {
            max_texture_array_layers: adapter_limits.max_texture_array_layers,
            ..::wgpu::Limits::default()
        };

        let (device, queue) = pollster::block_on(adapter.request_device(
            &::wgpu::DeviceDescriptor {
                label: Some("RustCraft GPU device"),
                required_features: ::wgpu::Features::empty(),
                required_limits,
            },
            None,
        ))
        .map_err(|error| format!("failed to create GPU device: {error}"))?;

        let texture = Self::create_texture_array(&device, &queue, atlas);
        let built = Self::build_pipelines(&device, &texture);

        let (
            target_view,
            linear_depth_view,
            depth_view,
            post_bind_group,
            color_ssbo,
            depth_ssbo,
            readback,
            depth_readback,
        ) = Self::create_targets(&device, &built.post_bind_layout, width as u32, height as u32);

        Ok(Self {
            device,
            queue,
            texture,
            pipeline: built.pipeline,
            cutout_pipeline: built.cutout_pipeline,
            transparent_pipeline: built.transparent_pipeline,
            sky_pipeline: built.sky_pipeline,
            bind_group: built.bind_group,
            globals_buffer: built.globals_buffer,
            instance_buffer: built.instance_buffer,
            chunk_buffers: HashMap::new(),
            target_view,
            linear_depth_view,
            depth_view,
            post_pipeline: built.post_pipeline,
            post_bind_layout: built.post_bind_layout,
            post_bind_group,
            color_ssbo,
            depth_ssbo,
            readback,
            depth_readback,
            frame_index: 0,
            width: width as u32,
            height: height as u32,
            gpu_name,
            driver_info,
            backend_version,
            backend,
        })
    }
}
