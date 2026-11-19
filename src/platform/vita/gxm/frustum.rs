//! Subchunk/chunk frustum culling, cache invalidation, perf telemetry, and `GxmRenderer::Drop`.
//! Releases linked shader programs, sync objects, render target, patcher, and GXM context cleanly.

#![allow(dead_code)]

use crate::graphics::atlas::TextureAtlas;
use crate::world::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

use super::ffi::*;
use super::renderer::GxmRenderer;
use super::textures::populate_atlas;
use super::types::ChunkSlotRanges;

impl GxmRenderer {
    pub fn invalidate_chunk(&mut self, cx: i32, cz: i32) {
        if let Some(slot) = self.chunk_slots.remove(&(cx, cz)) {
            self.evicted_slots_ring[self.back_buf].push(slot);
            self.chunk_spawn_times.remove(&(cx, cz));
        }
    }

    #[inline]
    pub fn is_subchunk_in_frustum(chunk_x: i32, sub_y: usize, chunk_z: i32, mvp: &crate::core::math::Mat4) -> bool {
        let min_x = (chunk_x * CHUNK_SIZE_X as i32) as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let min_y = (sub_y as i32 * crate::world::SUBCHUNK_SIZE) as f32;
        let max_y = min_y + crate::world::SUBCHUNK_SIZE as f32;
        let min_z = (chunk_z * CHUNK_SIZE_Z as i32) as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        Self::aabb_in_frustum(min_x, min_y, min_z, max_x, max_y, max_z, mvp)
    }

    #[inline]
    pub(super) fn is_chunk_in_frustum(chunk_x: i32, chunk_z: i32, mvp: &crate::core::math::Mat4) -> bool {
        let min_x = (chunk_x * CHUNK_SIZE_X as i32) as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let min_y = -16.0f32;
        let max_y = CHUNK_SIZE_Y as f32;
        let min_z = (chunk_z * CHUNK_SIZE_Z as i32) as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        Self::aabb_in_frustum(min_x, min_y, min_z, max_x, max_y, max_z, mvp)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn aabb_in_frustum(
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
        mvp: &crate::core::math::Mat4,
    ) -> bool {
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
        let m = &mvp.m;

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
            if clip_z >= 0.0 { out_near = false; }
            if clip_z <= clip_w { out_far = false; }
        }

        !(out_left || out_right || out_bottom || out_top || out_near || out_far)
    }

    /// Updates the GPU atlas in CDRAM after graphics/mod settings change.
    pub fn update_textures(&mut self, atlas: &TextureAtlas) {
        let pixels = self._atlas_mem.ptr as *mut u32;
        unsafe {
            sceGxmFinish(self.context);
            populate_atlas(pixels, atlas);
        }
    }

    /// Clears all allocated GPU chunk slots, forcing all chunks to be remeshed and reuploaded.
    pub fn clear_mesh_cache(&mut self) {
        self.chunk_slots.clear();
        self.free_slots = (0..Self::MAX_CHUNK_SLOTS).rev().collect();
        for ring in &mut self.evicted_slots_ring {
            ring.clear();
        }
        self.slot_ranges = [ChunkSlotRanges::default(); Self::MAX_CHUNK_SLOTS];
        self.chunk_spawn_times.clear();
        self.pending_meshes.clear();
        self.completed_meshes.clear();
        self.mesh_worker.invalidate_pending();
    }

    /// Alias matching desktop `VulkanContext::clear_chunk_buffers`.
    pub fn clear_chunk_buffers(&mut self) {
        self.clear_mesh_cache();
    }

    /// Returns GPU memory statistics: `(used_chunk_slots, max_chunk_slots, cdram_bytes, ram_bytes)`.
    pub fn get_perf_stats(&self) -> (usize, usize, usize, usize) {
        let used_slots = self.chunk_slots.len();
        let mems: [&super::mem::GpuMem; 17] = [
            &self._vdm_mem,
            &self._vertex_ring_mem,
            &self._fragment_ring_mem,
            &self._frag_usse_mem,
            &self.display_bufs[0],
            &self.display_bufs[1],
            &self.display_bufs[2],
            &self._depth_mem,
            &self._patcher_buf_mem,
            &self._patcher_vert_usse,
            &self._patcher_frag_usse,
            &self.clear_vb_mem,
            &self.clear_ib_mem,
            &self._atlas_mem,
            &self.hud_bufs[0],
            &self.hud_bufs[1],
            &self.hud_bufs[2],
        ];
        let pool_mems: [&super::mem::GpuMem; 4] = [
            &self.vb_mem,
            &self.ib_mem,
            &self.entity_vb_mem,
            &self.entity_ib_mem,
        ];
        let cdram: usize = mems.iter().chain(pool_mems.iter()).map(|m| m.cdram_bytes()).sum();
        let ram: usize = mems.iter().chain(pool_mems.iter()).map(|m| m.ram_bytes()).sum();
        (used_slots, Self::MAX_CHUNK_SLOTS, cdram, ram)
    }

    /// Returns last frame telemetry: `(draw_calls, tris_rendered, tris_culled, subchunks_rendered, subchunks_culled)`.
    pub fn last_render_telemetry(&self) -> (u32, u32, u32, u32, u32) {
        (
            self.last_draw_calls,
            self.last_triangles_rendered,
            self.last_triangles_culled,
            self.last_subchunks_rendered,
            self.last_subchunks_culled,
        )
    }
}

impl Drop for GxmRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = sceGxmFinish(self.context);
            self.shaders.release(self.patcher);
            for sync in self.display_sync {
                if !sync.is_null() {
                    let _ = sceGxmSyncObjectDestroy(sync);
                }
            }
            if !self.render_target.is_null() {
                let _ = sceGxmDestroyRenderTarget(self.render_target);
            }
            if !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherDestroy(self.patcher);
            }
            if !self.context.is_null() {
                let _ = sceGxmDestroyContext(self.context);
            }
            if !self.host_mem.is_null() {
                free(self.host_mem);
            }
            let _ = sceGxmTerminate();
        }
    }
}
