use bytemuck::{Pod, Zeroable};
use std::collections::{HashMap, HashSet};
use wgpu::util::DeviceExt;

use crate::math::{Mat4, Vec3};
use crate::mesher::Triangle;
use crate::texture::{TextureAtlas, TextureId, TEX_SIZE};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    uv: [f32; 2],
    ao: f32,
    texture_layer: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 4],
    camera_pitch: [f32; 4],
}

pub struct VulkanContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buffer: wgpu::Buffer,
    chunk_buffers: HashMap<(i32, i32), GpuChunk>,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    readback: wgpu::Buffer,
    depth_readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    opaque_scratch: Vec<GpuVertex>,
    trans_scratch: Vec<GpuVertex>,
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
camera_position: vec4<f32>,
camera_pitch: vec4<f32>,
};
@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var textures: texture_2d_array<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexInput {
@location(0) position: vec3<f32>,
@location(1) uv: vec2<f32>,
@location(2) ao: f32,
@location(3) texture_layer: u32,
};
struct VertexOutput {
@builtin(position) position: vec4<f32>,
@location(0) uv: vec2<f32>,
@location(1) @interpolate(flat) texture_layer: u32,
@location(2) ao: f32,
@location(3) world_position: vec3<f32>,
};

@vertex
fn world_vertex(input: VertexInput) -> VertexOutput {
var output: VertexOutput;
let clip = globals.view_proj * vec4<f32>(input.position, 1.0);
// The CPU matrix uses OpenGL's [-1, 1] depth range; Vulkan uses [0, 1].
output.position = vec4<f32>(clip.x, clip.y, clip.z * 0.5 + clip.w * 0.5, clip.w);
output.uv = input.uv;
output.texture_layer = input.texture_layer;
output.ao = input.ao;
output.world_position = input.position;
return output;
}

@fragment
fn world_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
let texel = textureSample(textures, texture_sampler, input.uv, i32(input.texture_layer));
let lit = texel.rgb * input.ao;
let distance_to_camera = distance(input.world_position, globals.camera_position.xyz);
let fog_t = clamp(distance_to_camera / 180.0, 0.0, 1.0);
let alpha = select(1.0, 0.48, input.texture_layer == 8u || input.texture_layer == 19u);
return vec4<f32>(mix(lit, vec3<f32>(0.75, 0.85, 1.0), fog_t), alpha);
}

struct SkyOutput {
@builtin(position) position: vec4<f32>,
@location(0) sky_y: f32,
};
@vertex
fn sky_vertex(@builtin(vertex_index) index: u32) -> SkyOutput {
var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0)
    );
    var output: SkyOutput;
    let position = positions[index];
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.sky_y = position.y * 0.5 + 0.5;
    return output;
    }
    @fragment
    fn sky_fragment(input: SkyOutput) -> @location(0) vec4<f32> {
    let t = clamp(input.sky_y + globals.camera_pitch.x * 0.05, 0.0, 1.0);
    return vec4<f32>(mix(vec3<f32>(0.35, 0.55, 0.95), vec3<f32>(0.75, 0.85, 1.0), t), 1.0);
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
            .ok_or_else(|| "brak urządzenia Vulkan obsługującego renderowanie".to_owned())?;
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RustCraft Vulkan device"),
                                                                            required_features: wgpu::Features::empty(),
                                                                            required_limits: wgpu::Limits::default(),
                },
                None,
            ))
            .map_err(|error| format!("nie udało się utworzyć urządzenia Vulkan: {error}"))?;

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RustCraft world shader"),
                                                     source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
            let texture = create_texture_array(&device, &queue, atlas);
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                                                   array_layer_count: Some(23),
                                                   ..Default::default()
            });
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
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
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32, 3 => Uint32],
            };
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("RustCraft Vulkan world pipeline"),
                                                         layout: Some(&pipeline_layout),
                                                         vertex: wgpu::VertexState { module: &shader, entry_point: "world_vertex", buffers: &[vertex_layout.clone()] },
                                                         fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "world_fragment", targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
                                                         primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
                                                         depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
                                                         multisample: Default::default(),
                                                         multiview: None,
            });
            let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("RustCraft transparent pipeline"),
                                                                     layout: Some(&pipeline_layout),
                                                                     vertex: wgpu::VertexState { module: &shader, entry_point: "world_vertex", buffers: &[vertex_layout] },
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
                                                                     depth_compare: wgpu::CompareFunction::Always,
                                                                     stencil: Default::default(),
                                                                                 bias: Default::default(),
                                                             }),
                                                             multisample: Default::default(), multiview: None,
            });
            let (target, target_view, depth_texture, depth_view, readback, depth_readback, padded_bytes_per_row) = create_targets(&device, width as u32, height as u32);
            Ok(Self {
                device,
                queue,
                pipeline,
                transparent_pipeline,
                sky_pipeline,
                bind_group,
                globals_buffer,
                chunk_buffers: HashMap::new(),
               target,
               target_view,
               depth_texture,
               depth_view,
               readback,
               depth_readback,
               width: width as u32,
               height: height as u32,
               padded_bytes_per_row,
               opaque_scratch: Vec::with_capacity(4096),
               trans_scratch: Vec::with_capacity(1024),
            })
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
                            out_pixels: &mut [u32],
                            out_depths: &mut [f32],
        ) {
            self.chunk_buffers.retain(|key, _| loaded_keys.contains(key));
            for key in loaded_keys {
                if !dirty_keys.contains(key) && self.chunk_buffers.contains_key(key) {
                    continue;
                }
                let triangles = mesh_cache.get(key).map(Vec::as_slice).unwrap_or(&[]);
                self.opaque_scratch.clear();
                self.trans_scratch.clear();

                for triangle in triangles {
                    let layer = texture_layer(triangle.texture);
                    let is_trans = matches!(triangle.texture, TextureId::Water | TextureId::Glass);
                    let dest = if is_trans { &mut self.trans_scratch } else { &mut self.opaque_scratch };
                    dest.push(GpuVertex {
                        position: [triangle.v0.pos.x, triangle.v0.pos.y, triangle.v0.pos.z],
                        uv: [triangle.v0.uv.0, triangle.v0.uv.1],
                        ao: triangle.v0.ao,
                        texture_layer: layer,
                    });
                    dest.push(GpuVertex {
                        position: [triangle.v1.pos.x, triangle.v1.pos.y, triangle.v1.pos.z],
                        uv: [triangle.v1.uv.0, triangle.v1.uv.1],
                        ao: triangle.v1.ao,
                        texture_layer: layer,
                    });
                    dest.push(GpuVertex {
                        position: [triangle.v2.pos.x, triangle.v2.pos.y, triangle.v2.pos.z],
                        uv: [triangle.v2.uv.0, triangle.v2.uv.1],
                        ao: triangle.v2.ao,
                        texture_layer: layer,
                    });
                }

                let empty_vertex = GpuVertex::zeroed();
                let vertex_bytes: &[u8] = if self.opaque_scratch.is_empty() {
                    bytemuck::bytes_of(&empty_vertex)
                } else {
                    bytemuck::cast_slice(&self.opaque_scratch)
                };
                let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("RustCraft chunk vertex buffer"),
                                                            contents: vertex_bytes,
                                                            usage: wgpu::BufferUsages::VERTEX,
                });
                let transparent_bytes: &[u8] = if self.trans_scratch.is_empty() {
                    bytemuck::bytes_of(&empty_vertex)
                } else {
                    bytemuck::cast_slice(&self.trans_scratch)
                };
                let transparent = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("RustCraft transparent chunk buffer"),
                                                                 contents: transparent_bytes,
                                                                 usage: wgpu::BufferUsages::VERTEX,
                });
                self.chunk_buffers.insert(*key, GpuChunk {
                    opaque: buffer,
                    opaque_count: self.opaque_scratch.len() as u32,
                                          transparent,
                                          transparent_count: self.trans_scratch.len() as u32,
                });
            }
            let globals = Globals {
                view_proj: [
                    [view_proj.m[0], view_proj.m[1], view_proj.m[2], view_proj.m[3]],
                    [view_proj.m[4], view_proj.m[5], view_proj.m[6], view_proj.m[7]],
                    [view_proj.m[8], view_proj.m[9], view_proj.m[10], view_proj.m[11]],
                    [view_proj.m[12], view_proj.m[13], view_proj.m[14], view_proj.m[15]],
                ],
                camera_position: [camera.x, camera.y, camera.z, 1.0],
                camera_pitch: [pitch, 0.0, 0.0, 0.0],
            };
            self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("RustCraft Vulkan frame") });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("RustCraft world pass"),
                                                         color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &self.target_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                                                         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                                                         timestamp_writes: None,
                                                         occlusion_query_set: None,
                });
                pass.set_pipeline(&self.sky_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.draw(0..3, 0..1);
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                for key in visible_keys {
                    if let Some(chunk) = self.chunk_buffers.get(key) {
                        if chunk.opaque_count > 0 {
                            pass.set_vertex_buffer(0, chunk.opaque.slice(..));
                            pass.draw(0..chunk.opaque_count, 0..1);
                        }
                    }
                }
                pass.set_pipeline(&self.transparent_pipeline);
                for key in visible_keys {
                    if let Some(chunk) = self.chunk_buffers.get(key) {
                        if chunk.transparent_count > 0 {
                            pass.set_vertex_buffer(0, chunk.transparent.slice(..));
                            pass.draw(0..chunk.transparent_count, 0..1);
                        }
                    }
                }
            }
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture { texture: &self.target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::ImageCopyBuffer { buffer: &self.readback, layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(self.padded_bytes_per_row), rows_per_image: Some(self.height) } },
                                           wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            );
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture { texture: &self.depth_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::DepthOnly },
                wgpu::ImageCopyBuffer { buffer: &self.depth_readback, layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(self.padded_bytes_per_row), rows_per_image: Some(self.height) } },
                                           wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            );
            self.queue.submit(Some(encoder.finish()));
            let slice = self.readback.slice(..);
            let depth_slice = self.depth_readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            depth_slice.map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::Maintain::Wait);

            let mapped = slice.get_mapped_range();
            let depth_mapped = depth_slice.get_mapped_range();

            if self.padded_bytes_per_row == self.width * 4 {
                for (out, chunk) in out_pixels.iter_mut().zip(mapped.chunks_exact(4)) {
                    *out = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
                }
            } else {
                for y in 0..self.height as usize {
                    let row = &mapped[y * self.padded_bytes_per_row as usize..];
                    let out_row = &mut out_pixels[y * self.width as usize..(y + 1) * self.width as usize];
                    for (x, out) in out_row.iter_mut().enumerate() {
                        let offset = x * 4;
                        *out = ((row[offset] as u32) << 16) | ((row[offset + 1] as u32) << 8) | (row[offset + 2] as u32);
                    }
                }
            }

            const NEAR_FAR: f32 = 0.05 * 400.0;
            const FAR: f32 = 400.0;
            const FAR_MINUS_NEAR: f32 = 400.0 - 0.05;

            if self.padded_bytes_per_row == self.width * 4 {
                let depth_f32s: &[f32] = bytemuck::cast_slice(&depth_mapped);
                for (out, &depth) in out_depths.iter_mut().zip(depth_f32s) {
                    if depth < 1.0 {
                        *out = NEAR_FAR / (FAR - depth * FAR_MINUS_NEAR);
                    } else {
                        *out = f32::INFINITY;
                    }
                }
            } else {
                for y in 0..self.height as usize {
                    let row_bytes = &depth_mapped[y * self.padded_bytes_per_row as usize..y * self.padded_bytes_per_row as usize + (self.width * 4) as usize];
                    let depth_f32s: &[f32] = bytemuck::cast_slice(row_bytes);
                    let out_row = &mut out_depths[y * self.width as usize..(y + 1) * self.width as usize];
                    for (out, &depth) in out_row.iter_mut().zip(depth_f32s) {
                        if depth < 1.0 {
                            *out = NEAR_FAR / (FAR - depth * FAR_MINUS_NEAR);
                        } else {
                            *out = f32::INFINITY;
                        }
                    }
                }
            }

            drop(depth_mapped);
            drop(mapped);
            self.readback.unmap();
            self.depth_readback.unmap();
        }
    }

    fn create_texture_array(device: &wgpu::Device, queue: &wgpu::Queue, atlas: &TextureAtlas) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft texture array"),
                                            size: wgpu::Extent3d { width: TEX_SIZE as u32, height: TEX_SIZE as u32, depth_or_array_layers: 23 },
                                            mip_level_count: 1,
                                            sample_count: 1,
                                            dimension: wgpu::TextureDimension::D2,
                                            format: wgpu::TextureFormat::Rgba8Unorm,
                                                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                                view_formats: &[],
        });
        for layer in 0..23u32 {
            let bytes: Vec<u8> = atlas.get(texture_id_from_layer(layer)).texels.iter().flat_map(|color| [
                (color[0] * 255.0) as u8,
                                                                                                (color[1] * 255.0) as u8,
                                                                                                (color[2] * 255.0) as u8,
                                                                                                255,
            ]).collect();
            queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d { x: 0, y: 0, z: layer }, aspect: wgpu::TextureAspect::All },
                &bytes,
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some((TEX_SIZE * 4) as u32), rows_per_image: Some(TEX_SIZE as u32) },
                                wgpu::Extent3d { width: TEX_SIZE as u32, height: TEX_SIZE as u32, depth_or_array_layers: 1 },
            );
        }
        texture
    }

    fn create_targets(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView, wgpu::Buffer, wgpu::Buffer, u32) {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft color target"),
                                           size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                                           mip_level_count: 1,
                                           sample_count: 1,
                                           dimension: wgpu::TextureDimension::D2,
                                           format: wgpu::TextureFormat::Rgba8Unorm,
                                               usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                                               view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustCraft depth target"),
                                          size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                                          mip_level_count: 1,
                                          sample_count: 1,
                                          dimension: wgpu::TextureDimension::D2,
                                          format: wgpu::TextureFormat::Depth32Float,
                                              usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                                              view_formats: &[],
        });
        let padded_bytes_per_row = ((width * 4 + 255) / 256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft frame readback"),
                                            size: padded_bytes_per_row as u64 * height as u64,
                                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                                            mapped_at_creation: false,
        });
        let depth_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustCraft depth readback"),
                                                  size: padded_bytes_per_row as u64 * height as u64,
                                                  usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                                                  mapped_at_creation: false,
        });
        let target_view = target.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        (target, target_view, depth, depth_view, readback, depth_readback, padded_bytes_per_row)
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
            _ => TextureId::TntSide,
        }
    }
