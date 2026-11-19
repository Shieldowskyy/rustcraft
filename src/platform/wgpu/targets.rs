//! Offscreen color, linear depth, hardware Z-buffer, and double-buffered readback targets.

use super::types::VulkanContext;

impl VulkanContext {
    /// Resizes GPU offscreen render targets, depth buffer, and compute post-processing bindings.
    pub fn resize(&mut self, width: usize, height: usize) {
        if width == 0
            || height == 0
            || (self.width == width as u32 && self.height == height as u32)
        {
            return;
        }
        self.width = width as u32;
        self.height = height as u32;
        self.frame_index = 0;

        let (
            target_view,
            linear_depth_view,
            depth_view,
            post_bind_group,
            color_ssbo,
            depth_ssbo,
            readback,
            depth_readback,
        ) = Self::create_targets(&self.device, &self.post_bind_layout, self.width, self.height);

        self.target_view = target_view;
        self.linear_depth_view = linear_depth_view;
        self.depth_view = depth_view;
        self.post_bind_group = post_bind_group;
        self.color_ssbo = color_ssbo;
        self.depth_ssbo = depth_ssbo;
        self.readback = readback;
        self.depth_readback = depth_readback;
    }

    pub(super) fn create_targets(
        device: &::wgpu::Device,
        post_bind_layout: &::wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> (
        ::wgpu::TextureView,
        ::wgpu::TextureView,
        ::wgpu::TextureView,
        [::wgpu::BindGroup; 2],
        [::wgpu::Buffer; 2],
        [::wgpu::Buffer; 2],
        [::wgpu::Buffer; 2],
        [::wgpu::Buffer; 2],
    ) {
        let target = device.create_texture(&::wgpu::TextureDescriptor {
            label: Some("RustCraft color target"),
            size: ::wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ::wgpu::TextureDimension::D2,
            format: ::wgpu::TextureFormat::Rgba8Unorm,
            usage: ::wgpu::TextureUsages::RENDER_ATTACHMENT | ::wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let linear_depth = device.create_texture(&::wgpu::TextureDescriptor {
            label: Some("RustCraft linear depth target"),
            size: ::wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ::wgpu::TextureDimension::D2,
            format: ::wgpu::TextureFormat::R32Float,
            usage: ::wgpu::TextureUsages::RENDER_ATTACHMENT | ::wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth = device.create_texture(&::wgpu::TextureDescriptor {
            label: Some("RustCraft depth target"),
            size: ::wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ::wgpu::TextureDimension::D2,
            format: ::wgpu::TextureFormat::Depth32Float,
            usage: ::wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let elements = (width * height) as u64;
        let buf_size = elements * 4;

        let create_ssbo = |label: &'static str| {
            device.create_buffer(&::wgpu::BufferDescriptor {
                label: Some(label),
                size: buf_size,
                usage: ::wgpu::BufferUsages::STORAGE | ::wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        };

        let create_readback = |label: &'static str| {
            device.create_buffer(&::wgpu::BufferDescriptor {
                label: Some(label),
                size: buf_size,
                usage: ::wgpu::BufferUsages::COPY_DST | ::wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };

        let color_ssbo = [
            create_ssbo("RustCraft color ssbo 0"),
            create_ssbo("RustCraft color ssbo 1"),
        ];
        let depth_ssbo = [
            create_ssbo("RustCraft depth ssbo 0"),
            create_ssbo("RustCraft depth ssbo 1"),
        ];
        let readback = [
            create_readback("RustCraft frame readback 0"),
            create_readback("RustCraft frame readback 1"),
        ];
        let depth_readback = [
            create_readback("RustCraft depth readback 0"),
            create_readback("RustCraft depth readback 1"),
        ];

        let target_view = target.create_view(&Default::default());
        let linear_depth_view = linear_depth.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());

        let post_bind_group = [
            device.create_bind_group(&::wgpu::BindGroupDescriptor {
                label: Some("RustCraft compute bind group 0"),
                layout: post_bind_layout,
                entries: &[
                    ::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: ::wgpu::BindingResource::TextureView(&target_view),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ::wgpu::BindingResource::TextureView(&linear_depth_view),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: color_ssbo[0].as_entire_binding(),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 3,
                        resource: depth_ssbo[0].as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&::wgpu::BindGroupDescriptor {
                label: Some("RustCraft compute bind group 1"),
                layout: post_bind_layout,
                entries: &[
                    ::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: ::wgpu::BindingResource::TextureView(&target_view),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ::wgpu::BindingResource::TextureView(&linear_depth_view),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: color_ssbo[1].as_entire_binding(),
                    },
                    ::wgpu::BindGroupEntry {
                        binding: 3,
                        resource: depth_ssbo[1].as_entire_binding(),
                    },
                ],
            }),
        ];

        (
            target_view,
            linear_depth_view,
            depth_view,
            post_bind_group,
            color_ssbo,
            depth_ssbo,
            readback,
            depth_readback,
        )
    }
}
