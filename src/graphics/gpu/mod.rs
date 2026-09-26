use bytemuck::{Pod, Zeroable};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use wgpu::util::DeviceExt;

use crate::core::math::{Mat4, Vec3};
use crate::graphics::atlas::{TextureAtlas, TextureId, TEX_SIZE};
use crate::world::mesher::Triangle;

pub const TEXTURE_LAYER_COUNT: u32 = 73;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    uv: [u8; 2],
    _pad: [u8; 2],
    light: [u8; 4],
    texture_layer: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    camera_position: [f32; 4],
    camera_pitch: [f32; 4],
    sky_top: [f32; 4],
    sky_bot: [f32; 4],
    fog_color: [f32; 4],
    sun_dir: [f32; 4],
    moon_dir: [f32; 4],
}

pub struct VulkanContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    chunk_buffers: HashMap<(i32, i32), GpuChunk>,
    target_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    post_pipeline: wgpu::ComputePipeline,
    post_bind_group: wgpu::BindGroup,
    color_ssbo: wgpu::Buffer,
    depth_ssbo: wgpu::Buffer,
    readback: wgpu::Buffer,
    depth_readback: wgpu::Buffer,
    width: u32,
    height: u32,
    gpu_name: String,
    driver_info: String,
    backend_version: String,
}

struct GpuChunk {
    opaque: wgpu::Buffer,
    opaque_count: u32,
    transparent: wgpu::Buffer,
    transparent_count: u32,
}

const SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_position: vec4<f32>,
    camera_pitch: vec4<f32>,
    sky_top: vec4<f32>,
    sky_bot: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    moon_dir: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var textures: texture_2d_array<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light: vec4<f32>,
    @location(3) texture_layer: u32,
    @location(4) y_offset: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) texture_layer: u32,
    @location(2) ao: f32,
    @location(3) sky_light: f32,
    @location(4) block_light: f32,
    @location(5) world_position: vec3<f32>,
};

@vertex
fn world_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = vec3<f32>(input.position.x, input.position.y + input.y_offset, input.position.z);
    let clip = globals.view_proj * vec4<f32>(pos, 1.0);
    output.position = vec4<f32>(clip.x, clip.y, clip.z * 0.5 + clip.w * 0.5, clip.w);
    output.uv = input.uv;
    output.texture_layer = input.texture_layer;
    output.ao = input.light.x;
    output.sky_light = input.light.y;
    output.block_light = input.light.z;
    output.world_position = pos;
    return output;
}

@fragment
fn world_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(textures, texture_sampler, input.uv, i32(input.texture_layer));
    let distance_to_camera = distance(input.world_position, globals.camera_position.xyz);
    var base_rgb = texel.rgb;
    if (texel.a < 0.1) {
        if (distance_to_camera < 120.0) {
            discard;
        }
    }
    let daylight = globals.camera_pitch.w;
    let sky_lit = input.sky_light * daylight;
    let lr = clamp(sky_lit + input.block_light * 1.35, 0.06, 1.1) * input.ao;
    let lg = clamp(sky_lit + input.block_light * 1.15, 0.06, 1.0) * input.ao;
    let lb = clamp(sky_lit * 0.95 + input.block_light * 0.80, 0.06, 1.0) * input.ao;
    let lit = base_rgb * vec3<f32>(lr, lg, lb);
    let max_fog_dist = select(globals.camera_pitch.z, 180.0, globals.camera_pitch.z <= 0.0);
    let fog_t = clamp((distance_to_camera - max_fog_dist * 0.45) / (max_fog_dist * 0.55), 0.0, 1.0) * globals.camera_pitch.y;
    let fog_col = globals.fog_color.xyz;
    let final_rgb = mix(lit, fog_col, fog_t);
    let alpha = select(1.0, 0.48, input.texture_layer == 8u || input.texture_layer == 19u);
    return vec4<f32>(final_rgb, alpha);
}

struct SkyOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn sky_vertex(@builtin(vertex_index) index: u32) -> SkyOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0)
    );
    var output: SkyOutput;
    let position = positions[index];
    output.position = vec4<f32>(position, 1.0, 1.0);
    output.ndc = position;
    return output;
}

@fragment
fn sky_fragment(input: SkyOutput) -> @location(0) vec4<f32> {
    let clip = vec4<f32>(input.ndc.x, input.ndc.y, 1.0, 1.0);
    let world_h = globals.inv_view_proj * clip;
    let world_pos = world_h.xyz / world_h.w;
    let ray_dir = normalize(world_pos - globals.camera_position.xyz);

    let t = clamp(ray_dir.y * 0.8 + 0.2, 0.0, 1.0);
    var col = mix(globals.sky_bot.xyz, globals.sky_top.xyz, t);

    let sun_dir = normalize(globals.sun_dir.xyz);
    let sun_dot = dot(ray_dir, sun_dir);
    if (sun_dot > 0.993) {
        let sun_disc = smoothstep(0.993, 0.997, sun_dot);
        col = mix(col, vec3<f32>(1.0, 0.98, 0.75), sun_disc);
    }
    if (sun_dot > 0.85) {
        let sun_glow = pow(sun_dot, 32.0) * 0.45;
        col += vec3<f32>(1.0, 0.65, 0.25) * sun_glow;
    }

    let moon_dir = normalize(globals.moon_dir.xyz);
    let moon_dot = dot(ray_dir, moon_dir);
    if (moon_dot > 0.995) {
        let moon_disc = smoothstep(0.995, 0.998, moon_dot);
        col = mix(col, vec3<f32>(0.92, 0.95, 1.0), moon_disc);
    }
    if (moon_dot > 0.90) {
        let moon_glow = pow(moon_dot, 40.0) * 0.25;
        col += vec3<f32>(0.5, 0.65, 0.95) * moon_glow;
    }

    let daylight = globals.camera_pitch.w;
    if (daylight < 0.65 && ray_dir.y > 0.02) {
        let star_alpha = clamp((0.65 - daylight) / 0.50, 0.0, 1.0);
        let star_grid = floor(ray_dir * 160.0);
        let star_rand = fract(sin(dot(star_grid, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
        if (star_rand > 0.988) {
            let star_bright = (star_rand - 0.988) / 0.012 * star_alpha;
            col += vec3<f32>(star_bright * 0.9, star_bright * 0.95, star_bright * 1.0);
        }
    }

    return vec4<f32>(col, 1.0);
}
"#;

const POST_PROCESS_SHADER: &str = r#"
@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var<storage, read_write> out_color: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_depth: array<f32>;

@compute @workgroup_size(16, 16)
fn post_process(@builtin(global_invocation_id) id: vec3<u32>) {
    let dim = textureDimensions(color_tex);
    if (id.x >= dim.x || id.y >= dim.y) {
        return;
    }
    let idx = id.y * dim.x + id.x;

    let color = textureLoad(color_tex, id.xy, 0);
    let r = u32(color.r * 255.0);
    let g = u32(color.g * 255.0);
    let b = u32(color.b * 255.0);
    out_color[idx] = (r << 16u) | (g << 8u) | b;

    let d = textureLoad(depth_tex, id.xy, 0);
    let near_far = 0.05 * 400.0;
    let far = 400.0;
    let far_minus_near = 400.0 - 0.05;

    if (d < 1.0) {
        out_depth[idx] = near_far / (far - d * far_minus_near);
    } else {
        out_depth[idx] = bitcast<f32>(0x7F800000u);
    }
}
"#;

impl VulkanContext {
    pub fn new(width: usize, height: usize, atlas: &TextureAtlas) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            flags: wgpu::InstanceFlags::default(),
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no Vulkan adapter found supporting rendering".to_owned())?;

        let adapter_info = adapter.get_info();
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
            "Vulkan Driver".to_string()
        };
        let backend_version = "Vulkan 1.3".to_string();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("RustCraft Vulkan device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|error| format!("failed to create Vulkan device: {error}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RustCraft world shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let texture = Self::create_texture_array(&device, &queue, atlas);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            array_layer_count: Some(TEXTURE_LAYER_COUNT),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft camera uniforms"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("RustCraft world bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RustCraft world bind group"),
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: globals_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RustCraft world pipeline layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Unorm8x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Unorm8x4,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 20,
                    shader_location: 3,
                },
            ],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 0,
                shader_location: 4,
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft Vulkan world pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "world_vertex", buffers: &[vertex_layout.clone(), instance_layout.clone()] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "world_fragment", targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(),
            multiview: None,
        });

        let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft transparent pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "world_vertex", buffers: &[vertex_layout, instance_layout] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "world_fragment", targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::LessEqual, stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(),
            multiview: None,
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RustCraft Vulkan sky pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "sky_vertex", buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "sky_fragment", targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Equal,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RustCraft compute post-process"),
            source: wgpu::ShaderSource::Wgsl(POST_PROCESS_SHADER.into()),
        });

        let post_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("RustCraft compute bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RustCraft compute pipeline layout"),
            bind_group_layouts: &[&post_bind_layout],
            push_constant_ranges: &[],
        });

        let post_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("RustCraft compute pipeline"),
            layout: Some(&post_pipeline_layout),
            module: &post_shader,
            entry_point: "post_process",
        });

        let (target_view, depth_view, color_ssbo, depth_ssbo, readback, depth_readback) =
            Self::create_targets(&device, width as u32, height as u32);

        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RustCraft compute bind group"),
            layout: &post_bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&target_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&depth_view) },
                wgpu::BindGroupEntry { binding: 2, resource: color_ssbo.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: depth_ssbo.as_entire_binding() },
            ],
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft chunk instance buffer"),
            size: 8192 * 4,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            device,
            queue,
            texture,
            pipeline,
            transparent_pipeline,
            sky_pipeline,
            bind_group,
            globals_buffer,
            instance_buffer,
            chunk_buffers: HashMap::new(),
            target_view,
            depth_view,
            post_pipeline,
            post_bind_group,
            color_ssbo,
            depth_ssbo,
            readback,
            depth_readback,
            width: width as u32,
            height: height as u32,
            gpu_name,
            driver_info,
            backend_version,
        })
    }

    #[inline(always)]
    pub fn gpu_info(&self) -> (&str, &str, &str) {
        (&self.gpu_name, &self.driver_info, &self.backend_version)
    }

    /// Resizes GPU offscreen render targets, depth buffer and compute post-processing bindings.
    pub fn resize(&mut self, width: usize, height: usize) {
        if width == 0 || height == 0 || (self.width == width as u32 && self.height == height as u32) {
            return;
        }
        self.width = width as u32;
        self.height = height as u32;

        let (target_view, depth_view, color_ssbo, depth_ssbo, readback, depth_readback) =
            Self::create_targets(&self.device, self.width, self.height);

        let post_bind_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("RustCraft compute bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        self.post_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RustCraft compute bind group"),
            layout: &post_bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&target_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&depth_view) },
                wgpu::BindGroupEntry { binding: 2, resource: color_ssbo.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: depth_ssbo.as_entire_binding() },
            ],
        });

        self.target_view = target_view;
        self.depth_view = depth_view;
        self.color_ssbo = color_ssbo;
        self.depth_ssbo = depth_ssbo;
        self.readback = readback;
        self.depth_readback = depth_readback;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_world(
        &mut self,
        mesh_cache: &HashMap<(i32, i32), Vec<Triangle>>,
        loaded_keys: &[(i32, i32)],
        dirty_keys: &HashSet<(i32, i32)>,
        visible_keys: &[(i32, i32)],
        view_proj: &Mat4,
        camera: Vec3,
        pitch: f32,
        render_distance: i32,
        fog_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        chunk_spawn_times: &HashMap<(i32, i32), f32>,
        daylight: f32,
        day_fraction: f32,
        sun_dir: Vec3,
        sky_colors: ([f32; 3], [f32; 3], [f32; 3]),
        out_pixels: &mut [u32],
        out_depths: &mut [f32],
    ) {
        self.chunk_buffers.retain(|key, _| mesh_cache.contains_key(key));

        let chunks_to_update: Vec<(i32, i32)> = loaded_keys
            .iter()
            .copied()
            .filter(|key| dirty_keys.contains(key) || !self.chunk_buffers.contains_key(key))
            .collect();

        if !chunks_to_update.is_empty() {
            let updated_buffers: Vec<((i32, i32), Vec<GpuVertex>, Vec<GpuVertex>)> = chunks_to_update
                .par_iter()
                .map(|key| {
                    let triangles = mesh_cache.get(key).map(Vec::as_slice).unwrap_or(&[]);
                    let mut opaque = Vec::with_capacity(triangles.len() * 3);
                    let mut trans = Vec::with_capacity(triangles.len() / 4 * 3);

                    for triangle in triangles {
                        let layer = Self::texture_layer(triangle.texture);
                        let is_trans = matches!(triangle.texture, TextureId::Water | TextureId::Glass);
                        let dest = if is_trans { &mut trans } else { &mut opaque };

                        let pack_vert = |v: &crate::world::mesher::Vertex| GpuVertex {
                            position: [v.pos.x, v.pos.y, v.pos.z],
                            uv: [
                                (v.uv.0 * 255.0).clamp(0.0, 255.0) as u8,
                                (v.uv.1 * 255.0).clamp(0.0, 255.0) as u8,
                            ],
                            _pad: [0, 0],
                            light: [
                                (v.ao * 255.0).clamp(0.0, 255.0) as u8,
                                (v.sky_light * 255.0).clamp(0.0, 255.0) as u8,
                                (v.block_light * 255.0).clamp(0.0, 255.0) as u8,
                                0,
                            ],
                            texture_layer: layer,
                        };

                        dest.push(pack_vert(&triangle.v0));
                        dest.push(pack_vert(&triangle.v1));
                        dest.push(pack_vert(&triangle.v2));
                    }
                    (*key, opaque, trans)
                })
                .collect();

            let empty_vertex = GpuVertex::zeroed();
            for (key, opaque_verts, trans_verts) in updated_buffers {
                let opaque_bytes: &[u8] = if opaque_verts.is_empty() {
                    bytemuck::bytes_of(&empty_vertex)
                } else {
                    bytemuck::cast_slice(&opaque_verts)
                };
                let opaque = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("RustCraft opaque chunk buffer"),
                    contents: opaque_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let transparent_bytes: &[u8] = if trans_verts.is_empty() {
                    bytemuck::bytes_of(&empty_vertex)
                } else {
                    bytemuck::cast_slice(&trans_verts)
                };
                let transparent = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("RustCraft transparent chunk buffer"),
                    contents: transparent_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                });

                self.chunk_buffers.insert(key, GpuChunk {
                    opaque,
                    opaque_count: opaque_verts.len() as u32,
                    transparent,
                    transparent_count: trans_verts.len() as u32,
                });
            }
        }

        let max_fog_dist = if fog_enabled {
            (render_distance as f32 * 16.0).max(32.0)
        } else {
            0.0
        };

        let inv_view_proj_mat = view_proj.inverse().unwrap_or_else(Mat4::identity);
        let moon_dir = -sun_dir;

        let globals = Globals {
            view_proj: [
                [view_proj.m[0], view_proj.m[1], view_proj.m[2], view_proj.m[3]],
                [view_proj.m[4], view_proj.m[5], view_proj.m[6], view_proj.m[7]],
                [view_proj.m[8], view_proj.m[9], view_proj.m[10], view_proj.m[11]],
                [view_proj.m[12], view_proj.m[13], view_proj.m[14], view_proj.m[15]],
            ],
            inv_view_proj: [
                [inv_view_proj_mat.m[0], inv_view_proj_mat.m[1], inv_view_proj_mat.m[2], inv_view_proj_mat.m[3]],
                [inv_view_proj_mat.m[4], inv_view_proj_mat.m[5], inv_view_proj_mat.m[6], inv_view_proj_mat.m[7]],
                [inv_view_proj_mat.m[8], inv_view_proj_mat.m[9], inv_view_proj_mat.m[10], inv_view_proj_mat.m[11]],
                [inv_view_proj_mat.m[12], inv_view_proj_mat.m[13], inv_view_proj_mat.m[14], inv_view_proj_mat.m[15]],
            ],
            camera_position: [camera.x, camera.y, camera.z, 1.0],
            camera_pitch: [pitch, if fog_enabled { 1.0 } else { 0.0 }, max_fog_dist, daylight],
            sky_top: [sky_colors.0[0], sky_colors.0[1], sky_colors.0[2], day_fraction],
            sky_bot: [sky_colors.1[0], sky_colors.1[1], sky_colors.1[2], 0.0],
            fog_color: [sky_colors.2[0], sky_colors.2[1], sky_colors.2[2], 0.0],
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            moon_dir: [moon_dir.x, moon_dir.y, moon_dir.z, 0.0],
        };
        self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        let mut culled_visible_keys: Vec<(i32, i32)> = visible_keys
            .iter()
            .copied()
            .filter(|&(cx, cz)| Self::is_chunk_in_frustum(cx, cz, view_proj))
            .collect();

        // Sort front-to-back for opaque chunks
        culled_visible_keys.sort_by(|&(a_x, a_z), &(b_x, b_z)| {
            let dist_a = Self::chunk_dist_sq(a_x, a_z, camera.x, camera.z);
            let dist_b = Self::chunk_dist_sq(b_x, b_z, camera.x, camera.z);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut opaque_draws = Vec::with_capacity(culled_visible_keys.len());
        for key in &culled_visible_keys {
            if let Some(chunk) = self.chunk_buffers.get(key) {
                if chunk.opaque_count > 0 {
                    let spawn_time = chunk_spawn_times.get(key).copied().unwrap_or(total_time);
                    let elapsed = total_time - spawn_time;
                    let y_offset = if chunk_anim && elapsed >= 0.0 && elapsed < 0.35 {
                        let t = (elapsed / 0.35).clamp(0.0, 1.0);
                        let ease = 1.0 - (1.0 - t).powi(3);
                        (1.0 - ease) * -14.0
                    } else {
                        0.0
                    };
                    opaque_draws.push((chunk, y_offset));
                }
            }
        }

        // Sort back-to-front for transparent chunks
        let mut trans_visible_keys = culled_visible_keys.clone();
        trans_visible_keys.sort_by(|&(a_x, a_z), &(b_x, b_z)| {
            let dist_a = Self::chunk_dist_sq(a_x, a_z, camera.x, camera.z);
            let dist_b = Self::chunk_dist_sq(b_x, b_z, camera.x, camera.z);
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut trans_draws = Vec::with_capacity(trans_visible_keys.len());
        for key in &trans_visible_keys {
            if let Some(chunk) = self.chunk_buffers.get(key) {
                if chunk.transparent_count > 0 {
                    let spawn_time = chunk_spawn_times.get(key).copied().unwrap_or(total_time);
                    let elapsed = total_time - spawn_time;
                    let y_offset = if chunk_anim && elapsed >= 0.0 && elapsed < 0.35 {
                        let t = (elapsed / 0.35).clamp(0.0, 1.0);
                        let ease = 1.0 - (1.0 - t).powi(3);
                        (1.0 - ease) * -14.0
                    } else {
                        0.0
                    };
                    trans_draws.push((chunk, y_offset));
                }
            }
        }

        let mut all_offsets = Vec::with_capacity(opaque_draws.len() + trans_draws.len() + 1);
        for (_, off) in &opaque_draws {
            all_offsets.push(*off);
        }
        let trans_start_idx = all_offsets.len();
        for (_, off) in &trans_draws {
            all_offsets.push(*off);
        }

        if !all_offsets.is_empty() {
            let max_count = 8192.min(all_offsets.len());
            self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&all_offsets[..max_count]));
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("RustCraft Vulkan frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RustCraft world pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &self.target_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. OPAQUE TERRAIN PASS (Front-to-Back)
            // Depth write is ON, populating Z-buffer with closest opaque voxel depths.
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            for (i, (chunk, _)) in opaque_draws.iter().enumerate() {
                if (i * 4 + 4) <= 8192 * 4 {
                    pass.set_vertex_buffer(0, chunk.opaque.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buffer.slice((i * 4) as u64..(i * 4 + 4) as u64));
                    pass.draw(0..chunk.opaque_count, 0..1);
                }
            }

            // 2. EARLY-Z SKY PASS (Fullscreen triangle at depth = 1.0)
            // Depth compare is EQUAL (1.0 == 1.0), only running fragment shader on unoccluded sky pixels!
            pass.set_pipeline(&self.sky_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);

            // 3. TRANSPARENT PASS (Water, Glass - Back-to-Front)
            pass.set_pipeline(&self.transparent_pipeline);
            for (i, (chunk, _)) in trans_draws.iter().enumerate() {
                let buf_idx = trans_start_idx + i;
                if (buf_idx * 4 + 4) <= 8192 * 4 {
                    pass.set_vertex_buffer(0, chunk.transparent.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buffer.slice((buf_idx * 4) as u64..(buf_idx * 4 + 4) as u64));
                    pass.draw(0..chunk.transparent_count, 0..1);
                }
            }
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("RustCraft Post Process"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.post_pipeline);
            cpass.set_bind_group(0, &self.post_bind_group, &[]);
            cpass.dispatch_workgroups((self.width + 15) / 16, (self.height + 15) / 16, 1);
        }

        let buffer_size = (self.width * self.height * 4) as wgpu::BufferAddress;
        encoder.copy_buffer_to_buffer(&self.color_ssbo, 0, &self.readback, 0, buffer_size);
        encoder.copy_buffer_to_buffer(&self.depth_ssbo, 0, &self.depth_readback, 0, buffer_size);

        self.queue.submit(Some(encoder.finish()));

        let slice = self.readback.slice(..);
        let depth_slice = self.depth_readback.slice(..);

        slice.map_async(wgpu::MapMode::Read, |_| {});
        depth_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let mapped_color = slice.get_mapped_range();
        let mapped_depth = depth_slice.get_mapped_range();

        out_pixels.copy_from_slice(bytemuck::cast_slice(&mapped_color));
        out_depths.copy_from_slice(bytemuck::cast_slice(&mapped_depth));

        drop(mapped_color);
        drop(mapped_depth);
        self.readback.unmap();
        self.depth_readback.unmap();
    }

    fn is_chunk_in_frustum(chunk_x: i32, chunk_z: i32, view_proj: &Mat4) -> bool {
        let min_x = (chunk_x * 16) as f32;
        let max_x = min_x + 16.0;
        let min_y = -32.0f32;
        let max_y = 256.0f32;
        let min_z = (chunk_z * 16) as f32;
        let max_z = min_z + 16.0;

        let corners = [
            [min_x, min_y, min_z],
            [max_x, min_y, min_z],
            [min_x, max_y, min_z],
            [max_x, max_y, min_z],
            [min_x, min_y, max_z],
            [max_x, min_y, max_z],
            [min_x, max_y, max_z],
            [max_x, max_y, max_z],
        ];

        let m = &view_proj.m;

        let mut out_left = true;
        let mut out_right = true;
        let mut out_bottom = true;
        let mut out_top = true;
        let mut out_near = true;
        let mut out_far = true;

        for corner in &corners {
            let x = corner[0];
            let y = corner[1];
            let z = corner[2];

            let clip_x = m[0] * x + m[4] * y + m[8] * z + m[12];
            let clip_y = m[1] * x + m[5] * y + m[9] * z + m[13];
            let clip_z = m[2] * x + m[6] * y + m[10] * z + m[14];
            let clip_w = m[3] * x + m[7] * y + m[11] * z + m[15];

            if clip_x >= -clip_w { out_left = false; }
            if clip_x <= clip_w { out_right = false; }
            if clip_y >= -clip_w { out_bottom = false; }
            if clip_y <= clip_w { out_top = false; }
            if clip_z >= -clip_w { out_near = false; }
            if clip_z <= clip_w { out_far = false; }
        }

        !(out_left || out_right || out_bottom || out_top || out_near || out_far)
    }

    pub fn update_textures(&mut self, atlas: &TextureAtlas) {
        for layer in 0..TEXTURE_LAYER_COUNT {
            let texels = &atlas.get(Self::texture_id_from_layer(layer)).texels;
            Self::upload_layer_mipmaps(&self.queue, &self.texture, layer, texels);
        }
    }

    pub fn clear_chunk_buffers(&mut self) {
        self.chunk_buffers.clear();
    }

    #[inline]
    fn chunk_dist_sq(cx: i32, cz: i32, cam_x: f32, cam_z: f32) -> f32 {
        let dx = (cx as f32 * 16.0 + 8.0) - cam_x;
        let dz = (cz as f32 * 16.0 + 8.0) - cam_z;
        dx * dx + dz * dz
    }

    fn downsample_texels_rgba(src: &[[u8; 4]], src_size: usize) -> Vec<[u8; 4]> {
        let dst_size = src_size / 2;
        let mut dst = Vec::with_capacity(dst_size * dst_size);
        for dy in 0..dst_size {
            for dx in 0..dst_size {
                let p00 = src[(dy * 2) * src_size + (dx * 2)];
                let p10 = src[(dy * 2) * src_size + (dx * 2 + 1)];
                let p01 = src[(dy * 2 + 1) * src_size + (dx * 2)];
                let p11 = src[(dy * 2 + 1) * src_size + (dx * 2 + 1)];
                let a0 = p00[3] as u32;
                let a1 = p10[3] as u32;
                let a2 = p01[3] as u32;
                let a3 = p11[3] as u32;
                let total_a = a0 + a1 + a2 + a3;
                if total_a == 0 {
                    dst.push([0, 0, 0, 0]);
                } else {
                    let r = (((p00[0] as u32 * a0 + p10[0] as u32 * a1 + p01[0] as u32 * a2 + p11[0] as u32 * a3) / total_a).min(255)) as u8;
                    let g = (((p00[1] as u32 * a0 + p10[1] as u32 * a1 + p01[1] as u32 * a2 + p11[1] as u32 * a3) / total_a).min(255)) as u8;
                    let b = (((p00[2] as u32 * a0 + p10[2] as u32 * a1 + p01[2] as u32 * a2 + p11[2] as u32 * a3) / total_a).min(255)) as u8;
                    let a = ((total_a + 2) / 4) as u8;
                    dst.push([r, g, b, a]);
                }
            }
        }
        dst
    }

    fn upload_layer_mipmaps(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        layer: u32,
        base_texels: &[[f32; 3]],
    ) {
        let mut level_rgba: Vec<[u8; 4]> = base_texels.iter().map(|color| {
            if color[0] < 0.0 {
                let real_r = (-color[0] - 0.001).clamp(0.0, 1.0);
                [
                    (real_r * 255.0) as u8,
                    (color[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (color[2].clamp(0.0, 1.0) * 255.0) as u8,
                    0,
                ]
            } else {
                [
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    255,
                ]
            }
        }).collect();

        let mut size = TEX_SIZE;
        for mip in 0..5 {
            let bytes: Vec<u8> = level_rgba.iter().flat_map(|c| *c).collect();
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: mip,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some((size * 4) as u32),
                    rows_per_image: Some(size as u32),
                },
                wgpu::Extent3d { width: size as u32, height: size as u32, depth_or_array_layers: 1 },
            );
            if mip < 4 {
                level_rgba = Self::downsample_texels_rgba(&level_rgba, size);
                size /= 2;
            }
        }
    }

    fn create_texture_array(device: &wgpu::Device, queue: &wgpu::Queue, atlas: &TextureAtlas) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft texture array"),
            size: wgpu::Extent3d { width: TEX_SIZE as u32, height: TEX_SIZE as u32, depth_or_array_layers: TEXTURE_LAYER_COUNT },
            mip_level_count: 5,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for layer in 0..TEXTURE_LAYER_COUNT {
            let texels = &atlas.get(VulkanContext::texture_id_from_layer(layer)).texels;
            Self::upload_layer_mipmaps(queue, &texture, layer, texels);
        }
        texture
    }

    fn create_targets(
        device: &wgpu::Device,
        width: u32,
        height: u32
    ) -> (wgpu::TextureView, wgpu::TextureView, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft color target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft depth target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let elements = (width * height) as u64;

        let color_ssbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft color ssbo"),
            size: elements * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let depth_ssbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft depth ssbo"),
            size: elements * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft frame readback"),
            size: elements * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let depth_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft depth readback"),
            size: elements * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let target_view = target.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());

        (target_view, depth_view, color_ssbo, depth_ssbo, readback, depth_readback)
    }

    #[inline(always)]
    pub const fn texture_layer(id: TextureId) -> u32 {
        match id {
            TextureId::GrassTop => 0,
            TextureId::GrassSide => 1,
            TextureId::Dirt => 2,
            TextureId::Stone => 3,
            TextureId::Sand => 4,
            TextureId::WoodTop => 5,
            TextureId::WoodSide => 6,
            TextureId::Leaves => 7,
            TextureId::Water => 8,
            TextureId::Snow => 9,
            TextureId::CactusTop => 10,
            TextureId::CactusSide => 11,
            TextureId::CoalOre => 12,
            TextureId::IronOre => 13,
            TextureId::Bedrock => 14,
            TextureId::Lava => 15,
            TextureId::SnowLeaves => 16,
            TextureId::Planks => 17,
            TextureId::Door => 18,
            TextureId::Glass => 19,
            TextureId::Ice => 20,
            TextureId::TntTop => 21,
            TextureId::TntSide => 22,
            TextureId::PigSkin => 23,
            TextureId::PigFace => 24,
            TextureId::PigSnout => 25,
            TextureId::CowSkin => 26,
            TextureId::CowFace => 27,
            TextureId::CowSnout => 28,
            TextureId::CowHorns => 29,
            TextureId::CowUdder => 30,
            TextureId::DoorTop => 31,
            TextureId::DestroyStage0 => 32,
            TextureId::DestroyStage1 => 33,
            TextureId::DestroyStage2 => 34,
            TextureId::DestroyStage3 => 35,
            TextureId::DestroyStage4 => 36,
            TextureId::DestroyStage5 => 37,
            TextureId::DestroyStage6 => 38,
            TextureId::DestroyStage7 => 39,
            TextureId::DestroyStage8 => 40,
            TextureId::DestroyStage9 => 41,
            TextureId::CraftingTableTop => 42,
            TextureId::CraftingTableSide => 43,
            TextureId::CraftingTableFront => 44,
            TextureId::DamageFlash => 45,
            TextureId::UnderwaterTint => 46,
            TextureId::VignetteTint => 47,
            TextureId::LavaTint => 48,
            TextureId::CherryLeaves => 49,
            TextureId::AutumnLeaves => 50,
            TextureId::Mud => 51,
            TextureId::Torch => 52,
            TextureId::EntityShadow => 53,
            TextureId::GrassTopSwamp => 54,
            TextureId::GrassSideSwamp => 55,
            TextureId::LeavesSwamp => 56,
            TextureId::GrassTopSavanna => 57,
            TextureId::GrassSideSavanna => 58,
            TextureId::LeavesSavanna => 59,
            TextureId::GrassTopCherry => 60,
            TextureId::GrassSideCherry => 61,
            TextureId::GrassTopAutumn => 62,
            TextureId::GrassSideAutumn => 63,
            TextureId::GrassTopTaiga => 64,
            TextureId::GrassSideTaiga => 65,
            TextureId::ChestTop => 66,
            TextureId::ChestSide => 67,
            TextureId::ChestFront => 68,
            TextureId::FurnaceTop => 69,
            TextureId::FurnaceSide => 70,
            TextureId::FurnaceFront => 71,
            TextureId::FurnaceFrontLit => 72,
        }
    }

    pub const fn texture_id_from_layer(layer: u32) -> TextureId {
        match layer {
            0 => TextureId::GrassTop,
            1 => TextureId::GrassSide,
            2 => TextureId::Dirt,
            3 => TextureId::Stone,
            4 => TextureId::Sand,
            5 => TextureId::WoodTop,
            6 => TextureId::WoodSide,
            7 => TextureId::Leaves,
            8 => TextureId::Water,
            9 => TextureId::Snow,
            10 => TextureId::CactusTop,
            11 => TextureId::CactusSide,
            12 => TextureId::CoalOre,
            13 => TextureId::IronOre,
            14 => TextureId::Bedrock,
            15 => TextureId::Lava,
            16 => TextureId::SnowLeaves,
            17 => TextureId::Planks,
            18 => TextureId::Door,
            19 => TextureId::Glass,
            20 => TextureId::Ice,
            21 => TextureId::TntTop,
            22 => TextureId::TntSide,
            23 => TextureId::PigSkin,
            24 => TextureId::PigFace,
            25 => TextureId::PigSnout,
            26 => TextureId::CowSkin,
            27 => TextureId::CowFace,
            28 => TextureId::CowSnout,
            29 => TextureId::CowHorns,
            30 => TextureId::CowUdder,
            31 => TextureId::DoorTop,
            32 => TextureId::DestroyStage0,
            33 => TextureId::DestroyStage1,
            34 => TextureId::DestroyStage2,
            35 => TextureId::DestroyStage3,
            36 => TextureId::DestroyStage4,
            37 => TextureId::DestroyStage5,
            38 => TextureId::DestroyStage6,
            39 => TextureId::DestroyStage7,
            40 => TextureId::DestroyStage8,
            41 => TextureId::DestroyStage9,
            42 => TextureId::CraftingTableTop,
            43 => TextureId::CraftingTableSide,
            44 => TextureId::CraftingTableFront,
            45 => TextureId::DamageFlash,
            46 => TextureId::UnderwaterTint,
            47 => TextureId::VignetteTint,
            48 => TextureId::LavaTint,
            49 => TextureId::CherryLeaves,
            50 => TextureId::AutumnLeaves,
            51 => TextureId::Mud,
            52 => TextureId::Torch,
            53 => TextureId::EntityShadow,
            54 => TextureId::GrassTopSwamp,
            55 => TextureId::GrassSideSwamp,
            56 => TextureId::LeavesSwamp,
            57 => TextureId::GrassTopSavanna,
            58 => TextureId::GrassSideSavanna,
            59 => TextureId::LeavesSavanna,
            60 => TextureId::GrassTopCherry,
            61 => TextureId::GrassSideCherry,
            62 => TextureId::GrassTopAutumn,
            63 => TextureId::GrassSideAutumn,
            64 => TextureId::GrassTopTaiga,
            65 => TextureId::GrassSideTaiga,
            66 => TextureId::ChestTop,
            67 => TextureId::ChestSide,
            68 => TextureId::ChestFront,
            69 => TextureId::FurnaceTop,
            70 => TextureId::FurnaceSide,
            71 => TextureId::FurnaceFront,
            72 => TextureId::FurnaceFrontLit,
            _ => TextureId::CraftingTableFront,
        }
    }
}
