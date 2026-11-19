//! 4-pass GPU terrain and Early-Z procedural sky rendering (`Opaque -> Cutout -> Sky -> Transparent`).
//! Matches desktop `VulkanContext` pass order, culling, depth test/write state, and uniform curves.

#![allow(dead_code)]

use crate::core::math::{Mat4, Vec3};
use crate::features::rendering::chunk_spawn_y_offset;
use crate::graphics::camera::Camera;
use crate::world::World;

use super::ffi::*;
use super::programs::LinkedGxmShaders;
use super::renderer::GxmRenderer;
use super::types::{now_us, FrameRenderStats, FrameTimingsUs, GxmVertex};

impl GxmRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn render_terrain(
        &mut self,
        buf_idx: usize,
        camera: &Camera,
        world: &mut World,
        mvp: &Mat4,
        sky_top: [f32; 3],
        sky_bottom: [f32; 3],
        fog_col: [f32; 3],
        daylight: f32,
        sun_dir: Vec3,
        cam_chunk_x: i32,
        cam_chunk_z: i32,
        render_dist: i32,
        fog_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        stats: &mut FrameRenderStats,
        timings: &mut FrameTimingsUs,
    ) -> bool {
        let t_cull = now_us();
        self.chunk_coords_scratch.clear();
        let render_dist_sq = render_dist * render_dist;
        for dz in -render_dist..=render_dist {
            for dx in -render_dist..=render_dist {
                if dx * dx + dz * dz <= render_dist_sq {
                    self.chunk_coords_scratch.push((dx, dz));
                }
            }
        }
        self.chunk_coords_scratch.sort_unstable_by_key(|&(dx, dz)| (dx * dx + dz * dz, dx, dz));
        let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);
        timings.cull_us += now_us().saturating_sub(t_cull);

        let t_stream = now_us();
        self.stream_chunk_slots(buf_idx, world, mvp, cam_chunk_x, cam_chunk_z, render_dist, is_occluded, total_time);
        timings.chunk_mesh_upload_us += now_us().saturating_sub(t_stream);

        let max_fog = if fog_enabled { (render_dist as f32 * 16.0).max(32.0) } else { 0.0 };

        // 4-pass render order matching desktop VulkanContext:
        // Pass 1: Opaque (Front-to-Back, CW cull, LESS, depth-write ON)
        let t_terrain = now_us();
        self.draw_chunk_pass(
            0, false, mvp, camera.position, sky_bottom, fog_col, daylight, max_fog,
            fog_enabled, chunk_anim, total_time, cam_chunk_x, cam_chunk_z, is_occluded, stats,
        );

        // Pass 2: Cutout (Front-to-Back, CW cull, LESS, depth-write ON)
        self.draw_chunk_pass(
            1, false, mvp, camera.position, sky_bottom, fog_col, daylight, max_fog,
            fog_enabled, chunk_anim, total_time, cam_chunk_x, cam_chunk_z, is_occluded, stats,
        );
        timings.terrain_us += now_us().saturating_sub(t_terrain);

        // Pass 3: Early-Z Procedural Sky (depth == 1.0, no depth writes)
        if !is_occluded {
            let t_sky = now_us();
            render_early_z_sky(
                self.context, &self.shaders, &self.clear_vb_mem, &self.clear_ib_mem,
                mvp, camera.position, daylight, sky_top, sky_bottom, sun_dir, stats,
            );
            timings.sky_us += now_us().saturating_sub(t_sky);
        }

        // Pass 4: Transparent Water & Glass (Back-to-Front, cull NONE, LESS_EQUAL, depth-write OFF)
        let t_trans = now_us();
        self.draw_chunk_pass(
            2, true, mvp, camera.position, sky_bottom, fog_col, daylight, max_fog,
            fog_enabled, chunk_anim, total_time, cam_chunk_x, cam_chunk_z, is_occluded, stats,
        );
        timings.trans_us += now_us().saturating_sub(t_trans);

        is_occluded
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_chunk_pass(
        &mut self,
        pass_kind: u8,
        reverse: bool,
        mvp: &Mat4,
        cam_pos: Vec3,
        sky_bottom: [f32; 3],
        fog_col: [f32; 3],
        daylight: f32,
        max_fog: f32,
        fog_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        cam_cx: i32,
        cam_cz: i32,
        is_occluded: bool,
        stats: &mut FrameRenderStats,
    ) {
        let (frag_prog, param_cam_fog, param_fog_color, cull, depth_func, depth_write) = match pass_kind {
            0 => (
                self.shaders.frag_prog_opaque, self.shaders.param_opaque_cam_fog,
                self.shaders.param_opaque_fog_color, SCE_GXM_CULL_CW,
                SCE_GXM_DEPTH_FUNC_LESS, SCE_GXM_DEPTH_WRITE_ENABLED,
            ),
            1 => (
                self.shaders.frag_prog_cutout, self.shaders.param_cutout_cam_fog,
                self.shaders.param_cutout_fog_color, SCE_GXM_CULL_CW,
                SCE_GXM_DEPTH_FUNC_LESS, SCE_GXM_DEPTH_WRITE_ENABLED,
            ),
            _ => (
                self.shaders.frag_prog_trans, self.shaders.param_trans_cam_fog,
                self.shaders.param_trans_fog_color, SCE_GXM_CULL_NONE,
                SCE_GXM_DEPTH_FUNC_LESS_EQUAL, SCE_GXM_DEPTH_WRITE_DISABLED,
            ),
        };

        sceGxmSetVertexProgram(self.context, self.shaders.vert_prog);
        sceGxmSetFragmentProgram(self.context, frag_prog);
        sceGxmSetFragmentTexture(self.context, 0, &self.atlas_tex);
        sceGxmSetCullMode(self.context, cull);
        sceGxmSetFrontDepthFunc(self.context, depth_func);
        sceGxmSetBackDepthFunc(self.context, depth_func);
        sceGxmSetFrontDepthWriteEnable(self.context, depth_write);
        sceGxmSetBackDepthWriteEnable(self.context, depth_write);

        bind_voxel_fs_uniforms(self.context, param_cam_fog, param_fog_color, cam_pos, max_fog, fog_col, fog_enabled);

        let vb_base = self.vb_mem.as_mut_ptr::<GxmVertex>();
        let ib_base = self.ib_mem.as_mut_ptr::<u16>();
        let chunk_count = self.chunk_coords_scratch.len();
        let mut last_y_offset = f32::NAN;

        for idx in 0..chunk_count {
            let (dx, dz) = if reverse {
                self.chunk_coords_scratch[chunk_count - 1 - idx]
            } else {
                self.chunk_coords_scratch[idx]
            };
            if is_occluded && (dx != 0 || dz != 0) {
                if pass_kind == 0 {
                    stats.subchunks_culled += 4;
                }
                continue;
            }
            let cx = cam_cx + dx;
            let cz = cam_cz + dz;
            let key = (cx, cz);
            let Some(&slot) = self.chunk_slots.get(&key) else { continue };
            let ranges = self.slot_ranges[slot];
            let (start_idx, index_count) = match pass_kind {
                0 => (ranges.opaque_start, ranges.opaque_count),
                1 => (ranges.cutout_start, ranges.cutout_count),
                _ => (ranges.trans_start, ranges.trans_count),
            };
            if index_count == 0 {
                continue;
            }
            if !Self::is_chunk_in_frustum(cx, cz, mvp) {
                if pass_kind == 0 {
                    stats.subchunks_culled += 4;
                }
                stats.tri_culled += index_count / 3;
                continue;
            }

            let spawn_time = self.chunk_spawn_times.get(&key).copied().unwrap_or(total_time);
            let y_offset = chunk_spawn_y_offset(chunk_anim, total_time, spawn_time);
            if y_offset != last_y_offset {
                bind_voxel_vs_uniforms(self.context, &self.shaders, mvp, sky_bottom, daylight, y_offset);
                last_y_offset = y_offset;
            }

            let chunk_vb = vb_base.add(slot * Self::MAX_VERTS_PER_SLOT);
            let chunk_ib = ib_base.add(slot * Self::MAX_INDICES_PER_SLOT + start_idx as usize);
            sceGxmSetVertexStream(self.context, 0, chunk_vb as *const core::ffi::c_void);
            if sceGxmDraw(self.context, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U16, chunk_ib as *const core::ffi::c_void, index_count) == SCE_OK {
                stats.tri_count += index_count / 3;
                stats.draw_calls += 1;
                if pass_kind == 0 {
                    stats.subchunks_rendered += 4;
                }
            }
        }
    }
}

unsafe fn bind_voxel_vs_uniforms(
    ctx: *mut SceGxmContext,
    shaders: &LinkedGxmShaders,
    mvp: &Mat4,
    sky_bottom: [f32; 3],
    daylight: f32,
    y_offset: f32,
) {
    let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
    sceGxmReserveVertexDefaultUniformBuffer(ctx, &mut buf);
    if buf.is_null() {
        return;
    }
    if !shaders.param_voxel_mvp.is_null() {
        sceGxmSetUniformDataF(buf, shaders.param_voxel_mvp, 0, 16, mvp.m.as_ptr());
    }
    if !shaders.param_voxel_sky_daylight.is_null() {
        let sky_day = [sky_bottom[0], sky_bottom[1], sky_bottom[2], daylight];
        sceGxmSetUniformDataF(buf, shaders.param_voxel_sky_daylight, 0, 4, sky_day.as_ptr());
    }
    if !shaders.param_voxel_chunk_offset.is_null() {
        let off = [0.0f32, y_offset, 0.0, 0.0];
        sceGxmSetUniformDataF(buf, shaders.param_voxel_chunk_offset, 0, 4, off.as_ptr());
    }
}

unsafe fn bind_voxel_fs_uniforms(
    ctx: *mut SceGxmContext,
    param_cam_fog: *const SceGxmProgramParameter,
    param_fog_color: *const SceGxmProgramParameter,
    cam_pos: Vec3,
    max_fog: f32,
    fog_col: [f32; 3],
    fog_enabled: bool,
) {
    let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
    sceGxmReserveFragmentDefaultUniformBuffer(ctx, &mut buf);
    if buf.is_null() {
        return;
    }
    if !param_cam_fog.is_null() {
        let data = [cam_pos.x, cam_pos.y, cam_pos.z, max_fog];
        sceGxmSetUniformDataF(buf, param_cam_fog, 0, 4, data.as_ptr());
    }
    if !param_fog_color.is_null() {
        let data = [fog_col[0], fog_col[1], fog_col[2], if fog_enabled { 1.0 } else { 0.0 }];
        sceGxmSetUniformDataF(buf, param_fog_color, 0, 4, data.as_ptr());
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn render_early_z_sky(
    ctx: *mut SceGxmContext,
    shaders: &LinkedGxmShaders,
    clear_vb: &super::mem::GpuMem,
    clear_ib: &super::mem::GpuMem,
    mvp: &Mat4,
    cam_pos: Vec3,
    daylight: f32,
    sky_top: [f32; 3],
    sky_bottom: [f32; 3],
    sun_dir: Vec3,
    stats: &mut FrameRenderStats,
) {
    sceGxmSetVertexProgram(ctx, shaders.sky_vert_prog);
    sceGxmSetFragmentProgram(ctx, shaders.sky_frag_prog);
    sceGxmSetCullMode(ctx, SCE_GXM_CULL_NONE);
    sceGxmSetFrontDepthFunc(ctx, SCE_GXM_DEPTH_FUNC_EQUAL);
    sceGxmSetBackDepthFunc(ctx, SCE_GXM_DEPTH_FUNC_EQUAL);
    sceGxmSetFrontDepthWriteEnable(ctx, SCE_GXM_DEPTH_WRITE_DISABLED);
    sceGxmSetBackDepthWriteEnable(ctx, SCE_GXM_DEPTH_WRITE_DISABLED);

    let inv_vp = mvp.inverse().unwrap_or_else(Mat4::identity);
    let moon_dir = -sun_dir;
    let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
    sceGxmReserveFragmentDefaultUniformBuffer(ctx, &mut buf);
    if !buf.is_null() {
        if !shaders.param_sky_inv_vp.is_null() {
            sceGxmSetUniformDataF(buf, shaders.param_sky_inv_vp, 0, 16, inv_vp.m.as_ptr());
        }
        if !shaders.param_sky_cam_daylight.is_null() {
            let v = [cam_pos.x, cam_pos.y, cam_pos.z, daylight];
            sceGxmSetUniformDataF(buf, shaders.param_sky_cam_daylight, 0, 4, v.as_ptr());
        }
        if !shaders.param_sky_top.is_null() {
            let v = [sky_top[0], sky_top[1], sky_top[2], 1.0];
            sceGxmSetUniformDataF(buf, shaders.param_sky_top, 0, 4, v.as_ptr());
        }
        if !shaders.param_sky_bot.is_null() {
            let v = [sky_bottom[0], sky_bottom[1], sky_bottom[2], 1.0];
            sceGxmSetUniformDataF(buf, shaders.param_sky_bot, 0, 4, v.as_ptr());
        }
        if !shaders.param_sky_sun_dir.is_null() {
            let v = [sun_dir.x, sun_dir.y, sun_dir.z, 0.0];
            sceGxmSetUniformDataF(buf, shaders.param_sky_sun_dir, 0, 4, v.as_ptr());
        }
        if !shaders.param_sky_moon_dir.is_null() {
            let v = [moon_dir.x, moon_dir.y, moon_dir.z, 0.0];
            sceGxmSetUniformDataF(buf, shaders.param_sky_moon_dir, 0, 4, v.as_ptr());
        }
    }

    sceGxmSetVertexStream(ctx, 0, clear_vb.ptr);
    if sceGxmDraw(ctx, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U16, clear_ib.ptr, 3) == SCE_OK {
        stats.draw_calls += 1;
        stats.tri_count += 1;
    }
}
