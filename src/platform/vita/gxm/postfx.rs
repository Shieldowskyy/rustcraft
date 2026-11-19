//! Fullscreen GPU Post-FX passes: underwater/lava/burning overlays, 2-pass biome tint, vitals, and HUD compositor.
//! Uses `postfx.vert/frag`, `biome_tint.frag`, and `hud.frag` matching PC software/WGSL math 1:1.

#![allow(dead_code)]

use super::ffi::*;
use super::renderer::GxmRenderer;
use super::textures::upload_hud_buffer;
use super::types::FrameRenderStats;

impl GxmRenderer {
    /// Draws fullscreen lava, underwater caustics, and/or burning flame post-FX (premultiplied alpha).
    pub(super) unsafe fn render_env_postfx(
        &self,
        is_underwater: bool,
        is_in_lava: bool,
        is_burning: bool,
        time: f32,
        stats: &mut FrameRenderStats,
    ) {
        if !is_underwater && !is_in_lava && !is_burning {
            return;
        }
        let p0 = [
            if is_underwater { 1.0 } else { 0.0 },
            if is_in_lava { 1.0 } else { 0.0 },
            if is_burning { 1.0 } else { 0.0 },
            time,
        ];
        let p1 = [0.0, 0.0, VITA_WIDTH as f32, VITA_HEIGHT as f32];
        self.draw_postfx_pass(p0, p1, stats);
    }

    /// Applies exact biome RGB color multiplier across the rendered 3D framebuffer.
    /// Channels `<= 1.0` scale via `DST_COLOR * SRC`; channels `> 1.0` boost cleanly via `DST_COLOR * SRC + DST`.
    pub(super) unsafe fn render_biome_tint(&self, tint: [f32; 3], stats: &mut FrameRenderStats) {
        if (tint[0] - 1.0).abs() < 0.003
            && (tint[1] - 1.0).abs() < 0.003
            && (tint[2] - 1.0).abs() < 0.003
        {
            return;
        }

        if tint[0] < 0.997 || tint[1] < 0.997 || tint[2] < 0.997 {
            let mul = [
                tint[0].clamp(0.0, 1.0),
                tint[1].clamp(0.0, 1.0),
                tint[2].clamp(0.0, 1.0),
                1.0,
            ];
            self.draw_biome_tint_pass(self.shaders.biome_tint_mul_prog, mul, stats);
        }

        if tint[0] > 1.003 || tint[1] > 1.003 || tint[2] > 1.003 {
            let add = [
                (tint[0] - 1.0).clamp(0.0, 1.0),
                (tint[1] - 1.0).clamp(0.0, 1.0),
                (tint[2] - 1.0).clamp(0.0, 1.0),
                1.0,
            ];
            self.draw_biome_tint_pass(self.shaders.biome_tint_add_prog, add, stats);
        }
    }

    /// Draws smooth damage red flash and pulsing 55px low-health border vignette.
    pub(super) unsafe fn render_vitals_postfx(
        &self,
        damage_alpha: f32,
        low_health: bool,
        time: f32,
        stats: &mut FrameRenderStats,
    ) {
        if damage_alpha <= 0.001 && !low_health {
            return;
        }
        let p0 = [0.0, 0.0, 0.0, time];
        let p1 = [
            damage_alpha,
            if low_health { 1.0 } else { 0.0 },
            VITA_WIDTH as f32,
            VITA_HEIGHT as f32,
        ];
        self.draw_postfx_pass(p0, p1, stats);
    }

    /// Uploads the 960x544 premultiplied-alpha HUD/UI buffer into CDRAM and composites it on the GPU.
    pub(super) unsafe fn render_hud_overlay(
        &mut self,
        buf_idx: usize,
        hud_pixels: &[u32],
        width: usize,
        height: usize,
        stats: &mut FrameRenderStats,
    ) {
        upload_hud_buffer(
            self.hud_bufs[buf_idx].ptr as *mut u32,
            hud_pixels,
            width,
            height,
            &mut self.hud_row_bounds[buf_idx],
        );

        self.prepare_fullscreen_overlay_state();
        sceGxmSetVertexProgram(self.context, self.shaders.postfx_vert_prog);
        sceGxmSetFragmentProgram(self.context, self.shaders.hud_frag_prog);
        sceGxmSetFragmentTexture(self.context, 0, &self.hud_texs[buf_idx]);
        sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);

        if sceGxmDraw(
            self.context,
            SCE_GXM_PRIMITIVE_TRIANGLES,
            SCE_GXM_INDEX_FORMAT_U16,
            self.clear_ib_mem.ptr,
            3,
        ) == SCE_OK
        {
            stats.draw_calls += 1;
            stats.tri_count += 1;
        }
    }

    unsafe fn prepare_fullscreen_overlay_state(&self) {
        sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);
        sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
        sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
        sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_DISABLED);
        sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_DISABLED);
    }

    unsafe fn draw_postfx_pass(
        &self,
        params0: [f32; 4],
        params1: [f32; 4],
        stats: &mut FrameRenderStats,
    ) {
        self.prepare_fullscreen_overlay_state();
        sceGxmSetVertexProgram(self.context, self.shaders.postfx_vert_prog);
        sceGxmSetFragmentProgram(self.context, self.shaders.postfx_frag_prog);

        let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
        sceGxmReserveFragmentDefaultUniformBuffer(self.context, &mut buf);
        if !buf.is_null() {
            if !self.shaders.param_postfx_p0.is_null() {
                sceGxmSetUniformDataF(buf, self.shaders.param_postfx_p0, 0, 4, params0.as_ptr());
            }
            if !self.shaders.param_postfx_p1.is_null() {
                sceGxmSetUniformDataF(buf, self.shaders.param_postfx_p1, 0, 4, params1.as_ptr());
            }
        }

        sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
        if sceGxmDraw(
            self.context,
            SCE_GXM_PRIMITIVE_TRIANGLES,
            SCE_GXM_INDEX_FORMAT_U16,
            self.clear_ib_mem.ptr,
            3,
        ) == SCE_OK
        {
            stats.draw_calls += 1;
            stats.tri_count += 1;
        }
    }

    unsafe fn draw_biome_tint_pass(
        &self,
        frag_prog: *mut SceGxmFragmentProgram,
        rgba: [f32; 4],
        stats: &mut FrameRenderStats,
    ) {
        self.prepare_fullscreen_overlay_state();
        sceGxmSetVertexProgram(self.context, self.shaders.postfx_vert_prog);
        sceGxmSetFragmentProgram(self.context, frag_prog);

        let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
        sceGxmReserveFragmentDefaultUniformBuffer(self.context, &mut buf);
        if !buf.is_null() && !self.shaders.param_biome_tint.is_null() {
            sceGxmSetUniformDataF(buf, self.shaders.param_biome_tint, 0, 4, rgba.as_ptr());
        }

        sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
        if sceGxmDraw(
            self.context,
            SCE_GXM_PRIMITIVE_TRIANGLES,
            SCE_GXM_INDEX_FORMAT_U16,
            self.clear_ib_mem.ptr,
            3,
        ) == SCE_OK
        {
            stats.draw_calls += 1;
            stats.tri_count += 1;
        }
    }
}
