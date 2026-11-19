//! Terrain chunk streaming, GPU slot LRU eviction, mesh upload, and 3-pass terrain drawing.
//! Draws Opaque -> Cutout -> Transparent chunk geometry with frustum culling and spawn animation.

#![allow(dead_code)]

use crate::core::math::Mat4;
use crate::graphics::camera::Camera;
use crate::world::mesher::{ChunkMeshJob, Triangle};
use crate::world::World;

use super::ffi::*;
use super::renderer::GxmRenderer;
use super::textures::atlas_uv;
use super::types::{classify_texture_pass, ChunkSlotRanges, FrameRenderStats, GxmVertex};

impl GxmRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn render_terrain(
        &mut self,
        camera: &Camera,
        world: &mut World,
        mvp: &Mat4,
        daylight: f32,
        cam_chunk_x: i32,
        cam_chunk_z: i32,
        render_distance: i32,
        render_dist: i32,
        fog_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        stats: &mut FrameRenderStats,
    ) -> bool {
        let vb_base = self.vb_mem.as_mut_ptr::<GxmVertex>();
        let ib_base = self.ib_mem.as_mut_ptr::<u16>();
        let max_dist = render_dist + 1;
            // 1. Remove chunk slots that are outside the render distance
                self.freed_slots_scratch.clear();
            let freed_ref = &mut self.freed_slots_scratch;
            self.chunk_slots.retain(|&(cx, cz), &mut slot_idx| {
                let dx = cx - cam_chunk_x;
                let dz = cz - cam_chunk_z;
                let chunk_unloaded = !world.chunks.contains_key(&(cx, cz));
                if dx.abs() > max_dist || dz.abs() > max_dist || chunk_unloaded {
                    freed_ref.push(slot_idx);
                    false
                } else {
                    true
                }
            });
            self.free_slots.extend(self.freed_slots_scratch.iter().copied());
            self.chunk_spawn_times.retain(|k, _| self.chunk_slots.contains_key(k));


            // Process nearer chunks first. The order is shared by mesh requests,
            // uploads and draw calls so the most useful terrain is prioritized.
            self.chunk_coords_scratch.clear();
            for dz in -render_dist..=render_dist {
                for dx in -render_dist..=render_dist {
                    self.chunk_coords_scratch.push((dx, dz));
                }
            }
            self.chunk_coords_scratch.sort_unstable_by_key(|&(dx, dz)| (dx * dx + dz * dz, dx, dz));
            let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);

            // Drain worker results without waiting.
            self.completed_meshes.extend(self.mesh_worker.drain_completed());

            // Process at most one completed mesh per frame. Prefer chunks nearest to the player.
            self.completed_meshes.sort_unstable_by_key(|((mx, mz), _)| {
                let dx = *mx - cam_chunk_x;
                let dz = *mz - cam_chunk_z;
                (dx * dx + dz * dz, *mx, *mz)
            });
            if !self.completed_meshes.is_empty() {
                let (mesh_key, mesh_tris) = self.completed_meshes.remove(0);
                self.pending_meshes.remove(&mesh_key);
                let (mcx, mcz) = mesh_key;
                let mdx = mcx - cam_chunk_x;
                let mdz = mcz - cam_chunk_z;
                let in_range = mdx.abs() <= max_dist && mdz.abs() <= max_dist;
                let still_exists = world.chunks.contains_key(&mesh_key);
                let still_visible = Self::is_chunk_in_frustum(mcx, mcz, &mvp);
                let changed_during_build = world.chunks.get(&mesh_key).map(|c| c.dirty).unwrap_or(false);

                if in_range && still_exists && still_visible && !changed_during_build {
                    let existing_slot = self.chunk_slots.get(&mesh_key).copied();
                    let target_slot = if let Some(slot) = existing_slot {
                        Some(slot)
                    } else if let Some(slot) = self.free_slots.pop() {
                        Some(slot)
                    } else {
                        let current_dist_sq = mdx * mdx + mdz * mdz;

                        // First reclaim a slot from a chunk that is no longer in the current view.
                        let offscreen = self.chunk_slots.iter()
                            .filter_map(|(&(scx, scz), &slot)| {
                                if !Self::is_chunk_in_frustum(scx, scz, &mvp) {
                                    let sdx = scx - cam_chunk_x;
                                    let sdz = scz - cam_chunk_z;
                                    Some(((scx, scz), slot, sdx * sdx + sdz * sdz))
                                } else {
                                    None
                                }
                            })
                            .max_by_key(|&(_, _, distance)| distance);

                        if let Some((old_key, old_slot, _)) = offscreen {
                            self.chunk_slots.remove(&old_key);
                            self.chunk_spawn_times.remove(&old_key);
                            Some(old_slot)
                        } else {
                            let farthest = self.chunk_slots.iter()
                                .map(|(&(scx, scz), &slot)| {
                                    let sdx = scx - cam_chunk_x;
                                    let sdz = scz - cam_chunk_z;
                                    ((scx, scz), slot, sdx * sdx + sdz * sdz)
                                })
                                .max_by_key(|&(_, _, distance)| distance);
                            if let Some((old_key, old_slot, old_dist_sq)) = farthest {
                                let replace_farthest = old_dist_sq > current_dist_sq
                                    || (old_dist_sq == current_dist_sq && old_key > mesh_key);
                                if replace_farthest {
                                    self.chunk_slots.remove(&old_key);
                                    self.chunk_spawn_times.remove(&old_key);
                                    Some(old_slot)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(slot) = target_slot {
                        let ranges = Self::upload_mesh(
                            &mesh_tris,
                            vb_base,
                            ib_base,
                            slot,
                            daylight,
                            &mut self.scratch_vb,
                            &mut self.scratch_opaque_ib,
                            &mut self.scratch_cutout_ib,
                            &mut self.scratch_trans_ib,
                        );
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

            // Submit at most one new job per frame and keep the in-flight queue bounded.
            const MAX_PENDING_MESHES: usize = 4;
            if self.pending_meshes.len() < MAX_PENDING_MESHES {
                for &(dx, dz) in &self.chunk_coords_scratch {
                    if is_occluded && (dx != 0 || dz != 0) {
                        continue;
                    }
                    let key = (cam_chunk_x + dx, cam_chunk_z + dz);
                    if self.pending_meshes.contains(&key) {
                        continue;
                    }
                    if !Self::is_chunk_in_frustum(key.0, key.1, &mvp) {
                        continue;
                    }
                    let needs_mesh = !self.chunk_slots.contains_key(&key)
                        || world.chunks.get(&key).map(|chunk| chunk.dirty).unwrap_or(false);
                    if !needs_mesh {
                        continue;
                    }
                    if let Some(chunk) = world.chunks.get(&key) {
                        let job = ChunkMeshJob::from_world(chunk, world);
                        if self.mesh_worker.submit_job(job) {
                            self.pending_meshes.insert(key);
                            if let Some(chunk) = world.chunks.get_mut(&key) {
                                chunk.dirty = false;
                            }
                            break;
                        }
                    }
                }
            }

            // 3-pass chunk rendering matching desktop VulkanContext:
            // Pass 0: Opaque (Front-to-Back, Early-Z/HSR active)
            // Pass 1: Cutout (Front-to-Back, leaves/doors/torches/crops)
            // Pass 2: Transparent (Back-to-Front, Alpha Blending, DepthWrite OFF)
            let chunk_count = self.chunk_coords_scratch.len();
            for pass_kind in 0..3u8 {
                if pass_kind == 2 {
                    sceGxmSetFragmentProgram(self.context, self.frag_prog_trans);
                    sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_DISABLED);
                    sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_DISABLED);
                } else if pass_kind == 0 {
                    sceGxmSetFragmentProgram(self.context, self.frag_prog_opaque);
                    sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
                    sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
                }

                let mut last_had_offset = false;
                for idx in 0..chunk_count {
                    let (dx, dz) = if pass_kind == 2 {
                        self.chunk_coords_scratch[chunk_count - 1 - idx]
                    } else {
                        self.chunk_coords_scratch[idx]
                    };

                    let cx = cam_chunk_x + dx;
                    let cz = cam_chunk_z + dz;
                    let key = (cx, cz);
                    if is_occluded && (dx != 0 || dz != 0) {
                        if pass_kind == 0 {
                            stats.subchunks_culled += 4;
                        }
                        continue;
                    }

                    if let Some(&slot) = self.chunk_slots.get(&key) {
                        let ranges = self.slot_ranges[slot];
                        let (start_idx, index_count) = match pass_kind {
                            0 => (ranges.opaque_start, ranges.opaque_count),
                            1 => (ranges.cutout_start, ranges.cutout_count),
                            _ => (ranges.trans_start, ranges.trans_count),
                        };
                        if index_count == 0 {
                            continue;
                        }
                        if !Self::is_chunk_in_frustum(cx, cz, &mvp) {
                            if pass_kind == 0 {
                                stats.subchunks_culled += 4;
                            }
                            stats.tri_culled += index_count / 3;
                            continue;
                        }

                        let spawn_time = self.chunk_spawn_times.get(&key).copied().unwrap_or(0.0);
                        let elapsed = total_time - spawn_time;
                        let y_offset = if chunk_anim && elapsed >= 0.0 && elapsed < 0.35 {
                            let t = (elapsed / 0.35).clamp(0.0, 1.0);
                            let ease = 1.0 - (1.0 - t).powi(3);
                            (1.0 - ease) * -14.0
                        } else {
                            0.0
                        };

                        if y_offset != 0.0 || last_had_offset {
                            let target_mvp = if y_offset != 0.0 {
                                mvp.mul(&crate::core::math::Mat4::translation(0.0, y_offset, 0.0))
                            } else {
                                *mvp
                            };
                            let mut chunk_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
                            sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut chunk_uniform_buf);
                            if !self.param_mvp.is_null() && !chunk_uniform_buf.is_null() {
                                sceGxmSetUniformDataF(chunk_uniform_buf, self.param_mvp, 0, 16, target_mvp.m.as_ptr());
                            }
                            if !self.param_fog.is_null() && !chunk_uniform_buf.is_null() {
                                let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                                let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                                sceGxmSetUniformDataF(chunk_uniform_buf, self.param_fog, 0, 4, fog_data.as_ptr());
                            }
                            last_had_offset = y_offset != 0.0;
                        }

                        let chunk_vb = vb_base.add(slot * Self::MAX_VERTS_PER_SLOT);
                        let chunk_ib = ib_base.add(slot * Self::MAX_INDICES_PER_SLOT + start_idx as usize);
                        sceGxmSetVertexStream(self.context, 0, chunk_vb as *const core::ffi::c_void);
                        let draw_rc = sceGxmDraw(
                            self.context,
                            SCE_GXM_PRIMITIVE_TRIANGLES,
                            SCE_GXM_INDEX_FORMAT_U16,
                            chunk_ib as *const core::ffi::c_void,
                            index_count,
                        );
                        if draw_rc == SCE_OK {
                            stats.tri_count += index_count / 3;
                            stats.draw_calls += 1;
                            if pass_kind == 0 {
                                stats.subchunks_rendered += 4;
                            }
                        } else {
                            crate::vita_log!("sceGxmDraw(chunk {}, {}, pass {}) failed: 0x{:08X}", cx, cz, pass_kind, draw_rc);
                        }
                    }
                }

                if last_had_offset {
                    let mut chunk_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
                    sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut chunk_uniform_buf);
                    if !self.param_mvp.is_null() && !chunk_uniform_buf.is_null() {
                        sceGxmSetUniformDataF(chunk_uniform_buf, self.param_mvp, 0, 16, mvp.m.as_ptr());
                    }
                    if !self.param_fog.is_null() && !chunk_uniform_buf.is_null() {
                        let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                        let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                        sceGxmSetUniformDataF(chunk_uniform_buf, self.param_fog, 0, 4, fog_data.as_ptr());
                    }
                }
            }

        is_occluded
    }

    pub(super) unsafe fn upload_mesh(
        tris: &[Triangle],
        vb_base: *mut GxmVertex,
        ib_base: *mut u16,
        slot_idx: usize,
        daylight: f32,
        scratch_vb: &mut Vec<GxmVertex>,
        scratch_opaque_ib: &mut Vec<u16>,
        scratch_cutout_ib: &mut Vec<u16>,
        scratch_trans_ib: &mut Vec<u16>,
    ) -> ChunkSlotRanges {
        let max_verts = Self::MAX_VERTS_PER_SLOT;
        let max_idx = Self::MAX_INDICES_PER_SLOT;
        scratch_vb.clear();
        scratch_opaque_ib.clear();
        scratch_cutout_ib.clear();
        scratch_trans_ib.clear();

        for tri in tris {
            let total_idx = scratch_opaque_ib.len() + scratch_cutout_ib.len() + scratch_trans_ib.len();
            if scratch_vb.len() + 3 > max_verts || total_idx + 3 > max_idx {
                break;
            }
            let (u0, v0, u1, v1) = atlas_uv(tri.texture);
            let base = scratch_vb.len() as u16;
            scratch_vb.push(GxmVertex::from_vertex(&tri.v0, u0, v0, u1, v1, daylight));
            scratch_vb.push(GxmVertex::from_vertex(&tri.v1, u0, v0, u1, v1, daylight));
            scratch_vb.push(GxmVertex::from_vertex(&tri.v2, u0, v0, u1, v1, daylight));

            let target_ib = match classify_texture_pass(tri.texture) {
                2 => &mut *scratch_trans_ib,
                1 => &mut *scratch_cutout_ib,
                _ => &mut *scratch_opaque_ib,
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

        ChunkSlotRanges {
            opaque_start,
            opaque_count,
            cutout_start,
            cutout_count,
            trans_start,
            trans_count,
        }
    }
}
