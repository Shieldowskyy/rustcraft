//! Chunk mesh partitioning into opaque/cutout/transparent GPU vertex buffers.

use std::collections::{HashMap, HashSet};

use bytemuck::Zeroable;
use rayon::prelude::*;

use crate::features::meshing::{Triangle, Vertex};
use crate::features::rendering::atlas::TextureId;

use super::types::{GpuChunk, GpuVertex, VulkanContext};

#[inline]
fn is_cutout_texture(texture: TextureId) -> bool {
    matches!(
        texture,
        TextureId::Leaves
            | TextureId::SnowLeaves
            | TextureId::CherryLeaves
            | TextureId::AutumnLeaves
            | TextureId::LeavesSwamp
            | TextureId::LeavesSavanna
            | TextureId::Door
            | TextureId::DoorTop
            | TextureId::Torch
            | TextureId::Ladder
            | TextureId::RustStoneWire
            | TextureId::RustStoneWireLit
            | TextureId::Lever
            | TextureId::Button
            | TextureId::PressurePlate
            | TextureId::WheatCrop
            | TextureId::CarrotCrop
            | TextureId::PotatoCrop
            | TextureId::Sapling
            | TextureId::Kelp
            | TextureId::Seagrass
            | TextureId::DriedKelp
    )
}

#[inline]
fn pack_gpu_vertex(v: &Vertex, texture_layer: u32) -> GpuVertex {
    GpuVertex {
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
        texture_layer,
    }
}

impl VulkanContext {
    /// Drops all cached GPU chunk vertex buffers (e.g. on world reload).
    pub fn clear_chunk_buffers(&mut self) {
        self.chunk_buffers.clear();
    }

    pub(super) fn upload_dirty_chunks(
        &mut self,
        mesh_cache: &HashMap<(i32, i32), Vec<Triangle>>,
        loaded_keys: &[(i32, i32)],
        dirty_keys: &HashSet<(i32, i32)>,
    ) {
        self.chunk_buffers.retain(|key, _| mesh_cache.contains_key(key));

        let chunks_to_update: Vec<(i32, i32)> = loaded_keys
            .iter()
            .copied()
            .filter(|key| dirty_keys.contains(key) || !self.chunk_buffers.contains_key(key))
            .collect();

        if chunks_to_update.is_empty() {
            return;
        }

        let updated_buffers: Vec<((i32, i32), Vec<GpuVertex>, Vec<GpuVertex>, Vec<GpuVertex>)> =
            chunks_to_update
                .par_iter()
                .map(|key| {
                    let triangles = mesh_cache.get(key).map(Vec::as_slice).unwrap_or(&[]);
                    let mut opaque = Vec::with_capacity(triangles.len() * 3);
                    let mut cutout = Vec::with_capacity(triangles.len() / 4 * 3);
                    let mut trans = Vec::with_capacity(triangles.len() / 4 * 3);

                    for triangle in triangles {
                        let layer = Self::texture_layer(triangle.texture);
                        let is_trans = matches!(triangle.texture, TextureId::Water | TextureId::Glass);
                        let dest = if is_trans {
                            &mut trans
                        } else if is_cutout_texture(triangle.texture) {
                            &mut cutout
                        } else {
                            &mut opaque
                        };

                        dest.push(pack_gpu_vertex(&triangle.v0, layer));
                        dest.push(pack_gpu_vertex(&triangle.v1, layer));
                        dest.push(pack_gpu_vertex(&triangle.v2, layer));
                    }
                    (*key, opaque, cutout, trans)
                })
                .collect();

        let empty_vertex = GpuVertex::zeroed();
        for (key, opaque_verts, cutout_verts, trans_verts) in updated_buffers {
            let opaque_bytes: &[u8] = if opaque_verts.is_empty() {
                bytemuck::bytes_of(&empty_vertex)
            } else {
                bytemuck::cast_slice(&opaque_verts)
            };
            let cutout_bytes: &[u8] = if cutout_verts.is_empty() {
                bytemuck::bytes_of(&empty_vertex)
            } else {
                bytemuck::cast_slice(&cutout_verts)
            };
            let trans_bytes: &[u8] = if trans_verts.is_empty() {
                bytemuck::bytes_of(&empty_vertex)
            } else {
                bytemuck::cast_slice(&trans_verts)
            };

            let update_or_create =
                |existing_buf: Option<(::wgpu::Buffer, usize)>, bytes: &[u8], label: &'static str| -> (::wgpu::Buffer, usize) {
                    if let Some((buf, cap)) = existing_buf {
                        if bytes.len() <= cap {
                            self.queue.write_buffer(&buf, 0, bytes);
                            return (buf, cap);
                        }
                    }
                    let new_cap = bytes.len().max(4096);
                    let buf = self.device.create_buffer(&::wgpu::BufferDescriptor {
                        label: Some(label),
                        size: new_cap as u64,
                        usage: ::wgpu::BufferUsages::VERTEX | ::wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.queue.write_buffer(&buf, 0, bytes);
                    (buf, new_cap)
                };

            let (existing_opaque, existing_cutout, existing_trans) = match self.chunk_buffers.remove(&key) {
                Some(c) => (
                    Some((c.opaque, c.opaque_capacity)),
                    Some((c.cutout, c.cutout_capacity)),
                    Some((c.transparent, c.transparent_capacity)),
                ),
                None => (None, None, None),
            };

            let (opaque_buf, opaque_cap) = update_or_create(
                existing_opaque,
                opaque_bytes,
                "RustCraft opaque chunk buffer",
            );
            let (cutout_buf, cutout_cap) = update_or_create(
                existing_cutout,
                cutout_bytes,
                "RustCraft cutout chunk buffer",
            );
            let (trans_buf, trans_cap) = update_or_create(
                existing_trans,
                trans_bytes,
                "RustCraft transparent chunk buffer",
            );

            self.chunk_buffers.insert(
                key,
                GpuChunk {
                    opaque: opaque_buf,
                    opaque_capacity: opaque_cap,
                    opaque_count: opaque_verts.len() as u32,
                    cutout: cutout_buf,
                    cutout_capacity: cutout_cap,
                    cutout_count: cutout_verts.len() as u32,
                    transparent: trans_buf,
                    transparent_capacity: trans_cap,
                    transparent_count: trans_verts.len() as u32,
                },
            );
        }
    }
}
