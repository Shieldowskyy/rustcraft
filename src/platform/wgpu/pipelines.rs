//! WGPU bind groups, vertex layouts, world render pipelines, and compute post-process pipeline.

use super::shaders::{POST_PROCESS_SHADER, SHADER};
use super::types::{Globals, GpuVertex, TEXTURE_LAYER_COUNT, VulkanContext};

pub(super) struct BuiltPipelines {
    pub(super) pipeline: ::wgpu::RenderPipeline,
    pub(super) cutout_pipeline: ::wgpu::RenderPipeline,
    pub(super) transparent_pipeline: ::wgpu::RenderPipeline,
    pub(super) sky_pipeline: ::wgpu::RenderPipeline,
    pub(super) bind_group: ::wgpu::BindGroup,
    pub(super) globals_buffer: ::wgpu::Buffer,
    pub(super) instance_buffer: ::wgpu::Buffer,
    pub(super) post_pipeline: ::wgpu::ComputePipeline,
    pub(super) post_bind_layout: ::wgpu::BindGroupLayout,
}

impl VulkanContext {
    pub(super) fn build_pipelines(
        device: &::wgpu::Device,
        texture: &::wgpu::Texture,
    ) -> BuiltPipelines {
        let shader = device.create_shader_module(::wgpu::ShaderModuleDescriptor {
            label: Some("RustCraft world shader"),
            source: ::wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let texture_view = texture.create_view(&::wgpu::TextureViewDescriptor {
            dimension: Some(::wgpu::TextureViewDimension::D2Array),
            array_layer_count: Some(TEXTURE_LAYER_COUNT),
            ..Default::default()
        });

        let sampler = device.create_sampler(&::wgpu::SamplerDescriptor {
            mag_filter: ::wgpu::FilterMode::Nearest,
            min_filter: ::wgpu::FilterMode::Nearest,
            mipmap_filter: ::wgpu::FilterMode::Linear,
            address_mode_u: ::wgpu::AddressMode::Repeat,
            address_mode_v: ::wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let globals_buffer = device.create_buffer(&::wgpu::BufferDescriptor {
            label: Some("RustCraft camera uniforms"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: ::wgpu::BufferUsages::UNIFORM | ::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&::wgpu::BindGroupLayoutDescriptor {
            label: Some("RustCraft world bindings"),
            entries: &[
                ::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: ::wgpu::BindingType::Buffer {
                        ty: ::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                ::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ::wgpu::ShaderStages::FRAGMENT,
                    ty: ::wgpu::BindingType::Texture {
                        sample_type: ::wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: ::wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                ::wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ::wgpu::ShaderStages::FRAGMENT,
                    ty: ::wgpu::BindingType::Sampler(::wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&::wgpu::BindGroupDescriptor {
            label: Some("RustCraft world bind group"),
            layout: &bind_layout,
            entries: &[
                ::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                ::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ::wgpu::BindingResource::TextureView(&texture_view),
                },
                ::wgpu::BindGroupEntry {
                    binding: 2,
                    resource: ::wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&::wgpu::PipelineLayoutDescriptor {
            label: Some("RustCraft world pipeline layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = ::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as u64,
            step_mode: ::wgpu::VertexStepMode::Vertex,
            attributes: &[
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Unorm8x2,
                    offset: 12,
                    shader_location: 1,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Unorm8x4,
                    offset: 16,
                    shader_location: 2,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Uint32,
                    offset: 20,
                    shader_location: 3,
                },
            ],
        };

        let instance_layout = ::wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: ::wgpu::VertexStepMode::Instance,
            attributes: &[::wgpu::VertexAttribute {
                format: ::wgpu::VertexFormat::Float32,
                offset: 0,
                shader_location: 4,
            }],
        };

        let color_targets = [
            Some(::wgpu::ColorTargetState {
                format: ::wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: ::wgpu::ColorWrites::ALL,
            }),
            Some(::wgpu::ColorTargetState {
                format: ::wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: ::wgpu::ColorWrites::ALL,
            }),
        ];

        let pipeline = device.create_render_pipeline(&::wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft Vulkan world pipeline"),
            layout: Some(&pipeline_layout),
            vertex: ::wgpu::VertexState {
                module: &shader,
                entry_point: "world_vertex",
                buffers: &[vertex_layout.clone(), instance_layout.clone()],
            },
            fragment: Some(::wgpu::FragmentState {
                module: &shader,
                entry_point: "world_fragment",
                targets: &color_targets,
            }),
            primitive: ::wgpu::PrimitiveState {
                cull_mode: Some(::wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(::wgpu::DepthStencilState {
                format: ::wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: ::wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let cutout_pipeline = device.create_render_pipeline(&::wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft Vulkan world cutout pipeline"),
            layout: Some(&pipeline_layout),
            vertex: ::wgpu::VertexState {
                module: &shader,
                entry_point: "world_vertex",
                buffers: &[vertex_layout.clone(), instance_layout.clone()],
            },
            fragment: Some(::wgpu::FragmentState {
                module: &shader,
                entry_point: "world_fragment_cutout",
                targets: &color_targets,
            }),
            primitive: ::wgpu::PrimitiveState {
                cull_mode: Some(::wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(::wgpu::DepthStencilState {
                format: ::wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: ::wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let transparent_pipeline = device.create_render_pipeline(&::wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft transparent pipeline"),
            layout: Some(&pipeline_layout),
            vertex: ::wgpu::VertexState {
                module: &shader,
                entry_point: "world_vertex",
                buffers: &[vertex_layout, instance_layout],
            },
            fragment: Some(::wgpu::FragmentState {
                module: &shader,
                entry_point: "world_fragment_trans",
                targets: &[
                    Some(::wgpu::ColorTargetState {
                        format: ::wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(::wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: ::wgpu::ColorWrites::ALL,
                    }),
                    Some(::wgpu::ColorTargetState {
                        format: ::wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: ::wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: ::wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(::wgpu::DepthStencilState {
                format: ::wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: ::wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let sky_pipeline = device.create_render_pipeline(&::wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft Vulkan sky pipeline"),
            layout: Some(&pipeline_layout),
            vertex: ::wgpu::VertexState {
                module: &shader,
                entry_point: "sky_vertex",
                buffers: &[],
            },
            fragment: Some(::wgpu::FragmentState {
                module: &shader,
                entry_point: "sky_fragment",
                targets: &color_targets,
            }),
            primitive: Default::default(),
            depth_stencil: Some(::wgpu::DepthStencilState {
                format: ::wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: ::wgpu::CompareFunction::Equal,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let post_shader = device.create_shader_module(::wgpu::ShaderModuleDescriptor {
            label: Some("RustCraft compute post-process"),
            source: ::wgpu::ShaderSource::Wgsl(POST_PROCESS_SHADER.into()),
        });

        let post_bind_layout = device.create_bind_group_layout(&::wgpu::BindGroupLayoutDescriptor {
            label: Some("RustCraft compute bind layout"),
            entries: &[
                ::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ::wgpu::ShaderStages::COMPUTE,
                    ty: ::wgpu::BindingType::Texture {
                        sample_type: ::wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: ::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                ::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ::wgpu::ShaderStages::COMPUTE,
                    ty: ::wgpu::BindingType::Texture {
                        sample_type: ::wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: ::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                ::wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ::wgpu::ShaderStages::COMPUTE,
                    ty: ::wgpu::BindingType::Buffer {
                        ty: ::wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                ::wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ::wgpu::ShaderStages::COMPUTE,
                    ty: ::wgpu::BindingType::Buffer {
                        ty: ::wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let post_pipeline_layout = device.create_pipeline_layout(&::wgpu::PipelineLayoutDescriptor {
            label: Some("RustCraft compute pipeline layout"),
            bind_group_layouts: &[&post_bind_layout],
            push_constant_ranges: &[],
        });

        let post_pipeline = device.create_compute_pipeline(&::wgpu::ComputePipelineDescriptor {
            label: Some("RustCraft compute pipeline"),
            layout: Some(&post_pipeline_layout),
            module: &post_shader,
            entry_point: "post_process",
        });

        let instance_buffer = device.create_buffer(&::wgpu::BufferDescriptor {
            label: Some("RustCraft chunk instance buffer"),
            size: 8192 * 4,
            usage: ::wgpu::BufferUsages::VERTEX | ::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        BuiltPipelines {
            pipeline,
            cutout_pipeline,
            transparent_pipeline,
            sky_pipeline,
            bind_group,
            globals_buffer,
            instance_buffer,
            post_pipeline,
            post_bind_layout,
        }
    }
}
