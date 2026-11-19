//! WGPU frame rendering: chunk sorting, 4-pass render, compute post-process, and async readback.

use std::collections::{HashMap, HashSet};

use crate::core::math::{Mat4, Vec3};
use crate::features::meshing::Triangle;
use crate::features::rendering::chunk_spawn_y_offset;

use super::types::{Globals, VulkanContext};

#[inline]
fn chunk_y_offset(
    chunk_anim: bool,
    total_time: f32,
    spawn_times: &HashMap<(i32, i32), f32>,
    key: &(i32, i32),
) -> f32 {
    let spawn_time = spawn_times.get(key).copied().unwrap_or(total_time);
    chunk_spawn_y_offset(chunk_anim, total_time, spawn_time)
}

impl VulkanContext {
    /// Returns the short lowercase name of the active WGPU backend (`"vulkan"`, `"opengl"`, etc.).
    #[inline(always)]
    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            ::wgpu::Backend::Vulkan => "vulkan",
            ::wgpu::Backend::Gl => "opengl",
            ::wgpu::Backend::Dx12 => "dx12",
            ::wgpu::Backend::Metal => "metal",
            _ => "gpu",
        }
    }

    /// Returns `(adapter_name, driver_info, backend_version)` strings for diagnostics/HUD.
    #[inline(always)]
    pub fn gpu_info(&self) -> (&str, &str, &str) {
        (&self.gpu_name, &self.driver_info, &self.backend_version)
    }

    #[inline]
    fn chunk_dist_sq(cx: i32, cz: i32, cam_x: f32, cam_z: f32) -> f32 {
        let dx = (cx as f32 * 16.0 + 8.0) - cam_x;
        let dz = (cz as f32 * 16.0 + 8.0) - cam_z;
        dx * dx + dz * dz
    }

    #[allow(dead_code)]
    fn is_chunk_in_frustum(chunk_x: i32, chunk_z: i32, view_proj: &Mat4) -> bool {
        let min_x = (chunk_x * 16) as f32;
        let max_x = min_x + 16.0;
        let min_y = -16.0f32;
        let max_y = 66.0f32;
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

    /// Renders visible chunks + procedural sky on GPU and copies color/depth readback into CPU slices.
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
        let current_slot = self.frame_index % 2;
        let prev_slot = (self.frame_index + 1) % 2;

        if self.frame_index > 1 {
            let slice_prev = self.readback[prev_slot].slice(..);
            let depth_slice_prev = self.depth_readback[prev_slot].slice(..);

            self.device.poll(::wgpu::Maintain::Wait);

            let mapped_color = slice_prev.get_mapped_range();
            let mapped_depth = depth_slice_prev.get_mapped_range();

            out_pixels.copy_from_slice(bytemuck::cast_slice(&mapped_color));
            out_depths.copy_from_slice(bytemuck::cast_slice(&mapped_depth));

            drop(mapped_color);
            drop(mapped_depth);
            self.readback[prev_slot].unmap();
            self.depth_readback[prev_slot].unmap();
        }

        self.upload_dirty_chunks(mesh_cache, loaded_keys, dirty_keys);

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

        let mut culled_visible_keys: Vec<(i32, i32)> = visible_keys.to_vec();
        culled_visible_keys.sort_by(|&(a_x, a_z), &(b_x, b_z)| {
            let dist_a = Self::chunk_dist_sq(a_x, a_z, camera.x, camera.z);
            let dist_b = Self::chunk_dist_sq(b_x, b_z, camera.x, camera.z);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut opaque_draws = Vec::with_capacity(culled_visible_keys.len());
        let mut cutout_draws = Vec::with_capacity(culled_visible_keys.len());
        for key in &culled_visible_keys {
            if let Some(chunk) = self.chunk_buffers.get(key) {
                let y_offset = chunk_y_offset(chunk_anim, total_time, chunk_spawn_times, key);
                if chunk.opaque_count > 0 {
                    opaque_draws.push((chunk, y_offset));
                }
                if chunk.cutout_count > 0 {
                    cutout_draws.push((chunk, y_offset));
                }
            }
        }

        let mut trans_visible_keys = culled_visible_keys;
        trans_visible_keys.sort_by(|&(a_x, a_z), &(b_x, b_z)| {
            let dist_a = Self::chunk_dist_sq(a_x, a_z, camera.x, camera.z);
            let dist_b = Self::chunk_dist_sq(b_x, b_z, camera.x, camera.z);
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut trans_draws = Vec::with_capacity(trans_visible_keys.len());
        for key in &trans_visible_keys {
            if let Some(chunk) = self.chunk_buffers.get(key) {
                if chunk.transparent_count > 0 {
                    let y_offset = chunk_y_offset(chunk_anim, total_time, chunk_spawn_times, key);
                    trans_draws.push((chunk, y_offset));
                }
            }
        }

        let mut all_offsets =
            Vec::with_capacity(opaque_draws.len() + cutout_draws.len() + trans_draws.len() + 1);
        for (_, off) in &opaque_draws {
            all_offsets.push(*off);
        }
        let cutout_start_idx = all_offsets.len();
        for (_, off) in &cutout_draws {
            all_offsets.push(*off);
        }
        let trans_start_idx = all_offsets.len();
        for (_, off) in &trans_draws {
            all_offsets.push(*off);
        }

        if !all_offsets.is_empty() {
            let max_count = 8192.min(all_offsets.len());
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&all_offsets[..max_count]),
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&::wgpu::CommandEncoderDescriptor {
                label: Some("RustCraft Vulkan frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&::wgpu::RenderPassDescriptor {
                label: Some("RustCraft world pass"),
                color_attachments: &[
                    Some(::wgpu::RenderPassColorAttachment {
                        view: &self.target_view,
                        resolve_target: None,
                        ops: ::wgpu::Operations {
                            load: ::wgpu::LoadOp::Clear(::wgpu::Color::BLACK),
                            store: ::wgpu::StoreOp::Store,
                        },
                    }),
                    Some(::wgpu::RenderPassColorAttachment {
                        view: &self.linear_depth_view,
                        resolve_target: None,
                        ops: ::wgpu::Operations {
                            load: ::wgpu::LoadOp::Clear(::wgpu::Color {
                                r: 1e9,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: ::wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(::wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(::wgpu::Operations {
                        load: ::wgpu::LoadOp::Clear(1.0),
                        store: ::wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. OPAQUE TERRAIN PASS (Front-to-Back, No Discard -> Early-Z / Hi-Z active)
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            for (i, (chunk, _)) in opaque_draws.iter().enumerate() {
                if (i * 4 + 4) <= 8192 * 4 {
                    pass.set_vertex_buffer(0, chunk.opaque.slice(..));
                    pass.draw(0..chunk.opaque_count, (i as u32)..(i as u32 + 1));
                }
            }

            // 2. CUTOUT TERRAIN PASS (Leaves, Doors, Torches, Ladders -> discard active)
            pass.set_pipeline(&self.cutout_pipeline);
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            for (i, (chunk, _)) in cutout_draws.iter().enumerate() {
                let buf_idx = cutout_start_idx + i;
                if (buf_idx * 4 + 4) <= 8192 * 4 {
                    pass.set_vertex_buffer(0, chunk.cutout.slice(..));
                    pass.draw(0..chunk.cutout_count, (buf_idx as u32)..(buf_idx as u32 + 1));
                }
            }

            // 3. EARLY-Z SKY PASS (Fullscreen triangle at depth = 1.0)
            pass.set_pipeline(&self.sky_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);

            // 4. TRANSPARENT PASS (Water, Glass - Back-to-Front)
            pass.set_pipeline(&self.transparent_pipeline);
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            for (i, (chunk, _)) in trans_draws.iter().enumerate() {
                let buf_idx = trans_start_idx + i;
                if (buf_idx * 4 + 4) <= 8192 * 4 {
                    pass.set_vertex_buffer(0, chunk.transparent.slice(..));
                    pass.draw(0..chunk.transparent_count, (buf_idx as u32)..(buf_idx as u32 + 1));
                }
            }
        }

        {
            let mut cpass = encoder.begin_compute_pass(&::wgpu::ComputePassDescriptor {
                label: Some("RustCraft Post Process"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.post_pipeline);
            cpass.set_bind_group(0, &self.post_bind_group[current_slot], &[]);
            cpass.dispatch_workgroups((self.width + 15) / 16, (self.height + 15) / 16, 1);
        }

        let buffer_size = (self.width * self.height * 4) as ::wgpu::BufferAddress;
        encoder.copy_buffer_to_buffer(
            &self.color_ssbo[current_slot],
            0,
            &self.readback[current_slot],
            0,
            buffer_size,
        );
        encoder.copy_buffer_to_buffer(
            &self.depth_ssbo[current_slot],
            0,
            &self.depth_readback[current_slot],
            0,
            buffer_size,
        );

        self.queue.submit(Some(encoder.finish()));

        let slice_curr = self.readback[current_slot].slice(..);
        let depth_slice_curr = self.depth_readback[current_slot].slice(..);
        slice_curr.map_async(::wgpu::MapMode::Read, |_| {});
        depth_slice_curr.map_async(::wgpu::MapMode::Read, |_| {});

        if self.frame_index == 0 {
            self.device.poll(::wgpu::Maintain::Wait);
            let mapped_color = slice_curr.get_mapped_range();
            let mapped_depth = depth_slice_curr.get_mapped_range();

            out_pixels.copy_from_slice(bytemuck::cast_slice(&mapped_color));
            out_depths.copy_from_slice(bytemuck::cast_slice(&mapped_depth));

            drop(mapped_color);
            drop(mapped_depth);
            self.readback[current_slot].unmap();
            self.depth_readback[current_slot].unmap();
        }

        self.frame_index += 1;
    }
}
