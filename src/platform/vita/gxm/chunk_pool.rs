//! GPU chunk slot pool management, LRU/frustum eviction, ring-buffered retirement, and mesh upload.
//! Separates chunk memory slot bookkeeping from the 4-pass world draw calls in `render_world.rs`.

#![allow(dead_code)]

use crate::core::math::Mat4;
use crate::features::rendering::atlas::TextureRenderPass;
use crate::world::mesher::Triangle;
use crate::world::World;

use super::renderer::GxmRenderer;
use super::textures::atlas_uv;
use super::types::{ChunkSlotRanges, GxmVertex};

impl GxmRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn stream_chunk_slots(
        &mut self,
        buf_idx: usize,
        world: &mut World,
        mvp: &Mat4,
        cam_chunk_x: i32,
        cam_chunk_z: i32,
        render_dist: i32,
        is_occluded: bool,
        total_time: f32,
    ) {
        while let Some(reclaimed) = self.evicted_slots_ring[buf_idx].pop() {
            self.free_slots.push(reclaimed);
        }

        let vb_base = self.vb_mem.as_mut_ptr::<GxmVertex>();
        let ib_base = self.ib_mem.as_mut_ptr::<u16>();
        let max_dist_sq = (render_dist + 1) * (render_dist + 1);

        self.freed_slots_scratch.clear();
        let freed_ref = &mut self.freed_slots_scratch;
        self.chunk_slots.retain(|&(cx, cz), &mut slot_idx| {
            let dx = cx - cam_chunk_x;
            let dz = cz - cam_chunk_z;
            let chunk_unloaded = !world.chunks.contains_key(&(cx, cz));
            if dx * dx + dz * dz > max_dist_sq || chunk_unloaded {
                freed_ref.push(slot_idx);
                false
            } else {
                true
            }
        });
        self.evicted_slots_ring[buf_idx].extend(self.freed_slots_scratch.iter().copied());
        self.chunk_spawn_times.retain(|k, _| self.chunk_slots.contains_key(k));

        self.completed_meshes.extend(self.mesh_worker.drain_completed());
        // Sort farthest-first so `pop()` removes the closest chunk in O(1).
        self.completed_meshes.sort_unstable_by_key(|((mx, mz), _)| {
            let dx = *mx - cam_chunk_x;
            let dz = *mz - cam_chunk_z;
            core::cmp::Reverse((dx * dx + dz * dz, *mx, *mz))
        });

        if let Some((mesh_key, mesh_tris)) = self.completed_meshes.pop() {
            self.pending_meshes.remove(&mesh_key);
            let (mcx, mcz) = mesh_key;
            let mdx = mcx - cam_chunk_x;
            let mdz = mcz - cam_chunk_z;
            let in_range = mdx * mdx + mdz * mdz <= max_dist_sq;
            let still_exists = world.chunks.contains_key(&mesh_key);
            let still_visible = Self::is_chunk_in_frustum(mcx, mcz, mvp);
            let allow_store = still_visible || !self.free_slots.is_empty();
            let changed_during_build = world.chunks.get(&mesh_key).map(|c| c.dirty).unwrap_or(false);

            if in_range && still_exists && allow_store && !changed_during_build {
                let existing_slot = self.chunk_slots.get(&mesh_key).copied();
                let target_slot = if let Some(old_slot) = existing_slot {
                    if let Some(fresh) = self.free_slots.pop() {
                        // DLACZEGO: Retiring old_slot into evicted_slots_ring prevents CPU writes
                        // from tearing chunks still in flight across triple-buffered GXM frames.
                        self.evicted_slots_ring[buf_idx].push(old_slot);
                        Some(fresh)
                    } else {
                        Some(old_slot)
                    }
                } else if let Some(slot) = self.free_slots.pop() {
                    Some(slot)
                } else {
                    self.reclaim_slot_for_new_chunk(cam_chunk_x, cam_chunk_z, mdx, mdz, mesh_key, mvp)
                };

                if let Some(slot) = target_slot {
                    let ranges = unsafe {
                        Self::upload_mesh(
                            &mesh_tris,
                            vb_base,
                            ib_base,
                            slot,
                            &mut self.scratch_vb,
                            &mut self.scratch_opaque_ib,
                            &mut self.scratch_cutout_ib,
                            &mut self.scratch_trans_ib,
                        )
                    };
                    self.slot_ranges[slot] = ranges;
                    self.chunk_slots.insert(mesh_key, slot);
                    if existing_slot.is_none() {
                        self.chunk_spawn_times.insert(mesh_key, total_time);
                    }
                } else {
                    self.pending_meshes.insert(mesh_key);
                    self.completed_meshes.push((mesh_key, mesh_tris));
                }
            }
        }

        const MAX_PENDING_MESHES: usize = 4;
        if self.pending_meshes.len() < MAX_PENDING_MESHES {
            let mut dirty_fallback: Option<(i32, i32)> = None;
            for &(dx, dz) in &self.chunk_coords_scratch {
                if is_occluded && (dx != 0 || dz != 0) {
                    continue;
                }
                let key = (cam_chunk_x + dx, cam_chunk_z + dz);
                if self.pending_meshes.contains(&key) || !Self::is_chunk_in_frustum(key.0, key.1, mvp) {
                    continue;
                }
                let Some(chunk) = world.chunks.get(&key) else {
                    continue;
                };
                if !self.chunk_slots.contains_key(&key) {
                    if let Some(job) = world.create_mesh_job(key) {
                        if self.mesh_worker.submit_job(job) {
                            self.pending_meshes.insert(key);
                            if let Some(c) = world.chunks.get_mut(&key) {
                                c.dirty = false;
                            }
                            dirty_fallback = None;
                            break;
                        }
                    }
                } else if dirty_fallback.is_none() && chunk.dirty {
                    dirty_fallback = Some(key);
                }
            }
            if let Some(key) = dirty_fallback {
                if let Some(job) = world.create_mesh_job(key) {
                    if self.mesh_worker.submit_job(job) {
                        self.pending_meshes.insert(key);
                        if let Some(c) = world.chunks.get_mut(&key) {
                            c.dirty = false;
                        }
                    }
                }
            }
        }
    }

    fn reclaim_slot_for_new_chunk(
        &mut self,
        cam_cx: i32,
        cam_cz: i32,
        mdx: i32,
        mdz: i32,
        mesh_key: (i32, i32),
        mvp: &Mat4,
    ) -> Option<usize> {
        let current_dist_sq = mdx * mdx + mdz * mdz;
        let offscreen = self.chunk_slots.iter()
            .filter_map(|(&(scx, scz), &slot)| {
                if !Self::is_chunk_in_frustum(scx, scz, mvp) {
                    let sdx = scx - cam_cx;
                    let sdz = scz - cam_cz;
                    Some(((scx, scz), slot, sdx * sdx + sdz * sdz))
                } else {
                    None
                }
            })
            .max_by_key(|&(_, _, d)| d);

        if let Some((old_key, old_slot, _)) = offscreen {
            self.chunk_slots.remove(&old_key);
            self.chunk_spawn_times.remove(&old_key);
            return Some(old_slot);
        }

        let farthest = self.chunk_slots.iter()
            .map(|(&(scx, scz), &slot)| {
                let sdx = scx - cam_cx;
                let sdz = scz - cam_cz;
                ((scx, scz), slot, sdx * sdx + sdz * sdz)
            })
            .max_by_key(|&(_, _, d)| d);
        if let Some((old_key, old_slot, old_dist_sq)) = farthest {
            if old_dist_sq > current_dist_sq || (old_dist_sq == current_dist_sq && old_key > mesh_key) {
                self.chunk_slots.remove(&old_key);
                self.chunk_spawn_times.remove(&old_key);
                return Some(old_slot);
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn upload_mesh(
        tris: &[Triangle],
        vb_base: *mut GxmVertex,
        ib_base: *mut u16,
        slot_idx: usize,
        scratch_vb: &mut Vec<GxmVertex>,
        scratch_opaque_ib: &mut Vec<u16>,
        scratch_cutout_ib: &mut Vec<u16>,
        scratch_trans_ib: &mut Vec<u16>,
    ) -> ChunkSlotRanges {
        scratch_vb.clear();
        scratch_opaque_ib.clear();
        scratch_cutout_ib.clear();
        scratch_trans_ib.clear();

        for tri in tris {
            let total_idx = scratch_opaque_ib.len() + scratch_cutout_ib.len() + scratch_trans_ib.len();
            if scratch_vb.len() + 3 > Self::MAX_VERTS_PER_SLOT || total_idx + 3 > Self::MAX_INDICES_PER_SLOT {
                break;
            }
            let (u0, v0, u1, v1) = atlas_uv(tri.texture);
            let base = scratch_vb.len() as u16;
            scratch_vb.push(GxmVertex::from_vertex(&tri.v0, u0, v0, u1, v1));
            scratch_vb.push(GxmVertex::from_vertex(&tri.v1, u0, v0, u1, v1));
            scratch_vb.push(GxmVertex::from_vertex(&tri.v2, u0, v0, u1, v1));

            let target_ib = match tri.texture.render_pass() {
                TextureRenderPass::Transparent => &mut *scratch_trans_ib,
                TextureRenderPass::Cutout => &mut *scratch_cutout_ib,
                TextureRenderPass::Opaque => &mut *scratch_opaque_ib,
            };
            target_ib.push(base);
            target_ib.push(base + 1);
            target_ib.push(base + 2);
        }

        let slot_vb = vb_base.add(slot_idx * Self::MAX_VERTS_PER_SLOT);
        let slot_ib = ib_base.add(slot_idx * Self::MAX_INDICES_PER_SLOT);
        if !scratch_vb.is_empty() {
            core::ptr::copy_nonoverlapping(scratch_vb.as_ptr(), slot_vb, scratch_vb.len());
        }

        let mut offset = 0usize;
        let opaque_start = offset as u32;
        let opaque_count = scratch_opaque_ib.len() as u32;
        if !scratch_opaque_ib.is_empty() {
            core::ptr::copy_nonoverlapping(scratch_opaque_ib.as_ptr(), slot_ib.add(offset), scratch_opaque_ib.len());
            offset += scratch_opaque_ib.len();
        }
        let cutout_start = offset as u32;
        let cutout_count = scratch_cutout_ib.len() as u32;
        if !scratch_cutout_ib.is_empty() {
            core::ptr::copy_nonoverlapping(scratch_cutout_ib.as_ptr(), slot_ib.add(offset), scratch_cutout_ib.len());
            offset += scratch_cutout_ib.len();
        }
        let trans_start = offset as u32;
        let trans_count = scratch_trans_ib.len() as u32;
        if !scratch_trans_ib.is_empty() {
            core::ptr::copy_nonoverlapping(scratch_trans_ib.as_ptr(), slot_ib.add(offset), scratch_trans_ib.len());
        }
        ChunkSlotRanges { opaque_start, opaque_count, cutout_start, cutout_count, trans_start, trans_count }
    }
}
