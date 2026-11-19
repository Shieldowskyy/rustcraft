//! Dynamic 3D GPU entity batches (A1/A2 opaque, B1/B2 blended) and first-person hand pass (Batch C).
//! Writes transient geometry into triple-buffered CDRAM pools matching PC render order and fog rules.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::app::{CameraPerspective, FishingBobber};
use crate::core::math::{IVec3, Mat4, Vec3};
use crate::features::entities::ArrowEntity;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::Mob;
use crate::gameplay::particles::Particle;
use crate::features::player::ItemStack;
use crate::graphics::camera::Camera;
use crate::net::RemotePlayer;
use crate::world::World;

use super::entities::{
    append_arrows, append_block_break_cracks, append_clouds, append_dropped_item_3d,
    append_fishing_bobber, append_item_shadow, append_mob, append_particles, append_tnt,
    EntityBatch,
};
use super::ffi::*;
use super::models::{append_hand_3d, append_offhand_3d, append_player_model, append_remote_player};
use super::renderer::GxmRenderer;
use super::types::{FrameRenderStats, GxmEntityVertex};
use super::weather::{append_lightning_bolts, append_precipitation};

pub(super) struct WorldEntitiesInput<'a> {
    pub(super) camera: &'a Camera,
    pub(super) world: &'a World,
    pub(super) mvp: &'a Mat4,
    pub(super) player_feet_pos: Vec3,
    pub(super) player_yaw: f32,
    pub(super) player_pitch: f32,
    pub(super) walk_phase: f32,
    pub(super) swing_progress: f32,
    pub(super) player_crouching: bool,
    pub(super) damage_tilt_timer: f32,
    pub(super) held_block: Option<u8>,
    pub(super) player_armor: &'a [Option<ItemStack>; 4],
    pub(super) camera_perspective: CameraPerspective,
    pub(super) break_target: Option<IVec3>,
    pub(super) break_progress: f32,
    pub(super) items: &'a [ItemEntity],
    pub(super) mobs: &'a [Mob],
    pub(super) particles: &'a [Particle],
    pub(super) arrows: &'a [ArrowEntity],
    pub(super) fishing_bobber: Option<&'a FishingBobber>,
    pub(super) remote_players: Option<&'a HashMap<u32, RemotePlayer>>,
    pub(super) clouds_enabled: bool,
    pub(super) fog_enabled: bool,
    pub(super) max_fog: f32,
    pub(super) fog_col: [f32; 3],
    pub(super) daylight: f32,
    pub(super) brightness: f32,
    pub(super) total_time: f32,
    pub(super) is_occluded: bool,
}

impl GxmRenderer {
    /// Builds and draws dynamic 3D world entities (A1 opaque+fog, A2 opaque, B1 blend+fog, B2 blend).
    /// Returns `(v_count, i_count)` cursor so Batch C (first-person hand) can append cleanly later.
    pub(super) unsafe fn render_world_entities(
        &mut self,
        buf_idx: usize,
        input: &WorldEntitiesInput<'_>,
        stats: &mut FrameRenderStats,
    ) -> (usize, usize) {
        let vb_base = (self.entity_vb_mem.as_mut_ptr::<GxmEntityVertex>()).add(buf_idx * Self::MAX_ENTITY_VERTS);
        let ib_base = (self.entity_ib_mem.as_mut_ptr::<u16>()).add(buf_idx * Self::MAX_ENTITY_INDICES);
        let mut batch = EntityBatch::new(vb_base, ib_base, Self::MAX_ENTITY_VERTS, Self::MAX_ENTITY_INDICES);

        let a1_start = 0usize;
        if !input.is_occluded && input.camera_perspective != CameraPerspective::FirstPerson {
            let hurt_tint = if input.damage_tilt_timer > 0.0 { 0.65 } else { 0.0 };
            let held = input.held_block.map(crate::world::BlockId::from_u8);
            append_player_model(
                &mut batch,
                input.player_feet_pos,
                input.player_yaw,
                input.player_pitch,
                input.walk_phase,
                input.swing_progress,
                input.player_crouching,
                hurt_tint,
                held,
                input.player_armor,
                input.camera.position,
                input.daylight,
                input.brightness,
            );
        }
        if let Some(rps) = input.remote_players {
            for rp in rps.values() {
                append_remote_player(&mut batch, rp, input.camera.position, input.daylight, input.brightness);
            }
        }
        if !input.is_occluded {
            for mob in input.mobs {
                append_mob(&mut batch, mob, input.camera.position, input.daylight, input.brightness);
            }
            for tnt in &input.world.primed_tnts {
                append_tnt(&mut batch, tnt, input.camera.position, input.daylight, input.brightness);
            }
            for item in input.items {
                append_dropped_item_3d(&mut batch, item, input.daylight, input.brightness);
            }
        }
        let a1_end = batch.i_count;

        let a2_start = a1_end;
        if !input.is_occluded {
            append_particles(&mut batch, input.particles, input.camera);
            if let Some(bobber) = input.fishing_bobber {
                append_fishing_bobber(&mut batch, bobber, input.camera);
            }
            append_arrows(&mut batch, input.arrows, input.camera);
        }
        let a2_end = batch.i_count;

        let b1_start = a2_end;
        if !input.is_occluded {
            if input.clouds_enabled {
                append_clouds(
                    &mut batch,
                    input.camera.position,
                    input.total_time,
                    input.daylight,
                    input.fog_col,
                    input.world.weather.rain_strength,
                    input.world.weather.thunder_strength,
                );
            }
            for item in input.items {
                append_item_shadow(&mut batch, item, input.world, input.daylight, input.brightness);
            }
            if let Some(target) = input.break_target {
                append_block_break_cracks(&mut batch, target, input.break_progress, input.daylight, input.brightness);
            }
        }
        let b1_end = batch.i_count;

        let b2_start = b1_end;
        if !input.is_occluded {
            append_precipitation(&mut batch, input.world, input.camera, input.total_time);
            append_lightning_bolts(&mut batch, &input.world.weather.active_lightning, input.camera);
        }
        let b2_end = batch.i_count;

        if b2_end == 0 {
            return (batch.v_count, batch.i_count);
        }

        sceGxmSetVertexProgram(self.context, self.shaders.entity_vert_prog);
        sceGxmSetFragmentTexture(self.context, 0, &self.atlas_tex);
        sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);
        bind_entity_vs_uniforms(self.context, &self.shaders, input.mvp);
        sceGxmSetVertexStream(self.context, 0, vb_base as *const core::ffi::c_void);

        self.draw_entity_subbatch(vb_base, ib_base, a1_start, a1_end, true, input.fog_enabled, input.camera.position, input.max_fog, input.fog_col, stats);
        self.draw_entity_subbatch(vb_base, ib_base, a2_start, a2_end, true, false, input.camera.position, 0.0, input.fog_col, stats);
        self.draw_entity_subbatch(vb_base, ib_base, b1_start, b1_end, false, input.fog_enabled, input.camera.position, input.max_fog, input.fog_col, stats);
        self.draw_entity_subbatch(vb_base, ib_base, b2_start, b2_end, false, false, input.camera.position, 0.0, input.fog_col, stats);

        (batch.v_count, batch.i_count)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_entity_subbatch(
        &self,
        _vb_base: *const GxmEntityVertex,
        ib_base: *const u16,
        start: usize,
        end: usize,
        opaque: bool,
        fog_enabled: bool,
        cam_pos: Vec3,
        max_fog: f32,
        fog_col: [f32; 3],
        stats: &mut FrameRenderStats,
    ) {
        let count = end.saturating_sub(start);
        if count == 0 {
            return;
        }
        let (frag_prog, depth_func, depth_write) = if opaque {
            (self.shaders.entity_frag_opaque, SCE_GXM_DEPTH_FUNC_LESS, SCE_GXM_DEPTH_WRITE_ENABLED)
        } else {
            (self.shaders.entity_frag_blend, SCE_GXM_DEPTH_FUNC_LESS_EQUAL, SCE_GXM_DEPTH_WRITE_DISABLED)
        };
        sceGxmSetFragmentProgram(self.context, frag_prog);
        sceGxmSetFrontDepthFunc(self.context, depth_func);
        sceGxmSetBackDepthFunc(self.context, depth_func);
        sceGxmSetFrontDepthWriteEnable(self.context, depth_write);
        sceGxmSetBackDepthWriteEnable(self.context, depth_write);
        bind_entity_fs_uniforms(self.context, &self.shaders, cam_pos, max_fog, fog_col, fog_enabled);

        if sceGxmDraw(self.context, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U16, ib_base.add(start) as *const core::ffi::c_void, count as u32) == SCE_OK {
            stats.tri_count += count as u32 / 3;
            stats.draw_calls += 1;
        }
    }

    /// Batch C: clears only depth buffer to 1.0 and draws first-person main hand + offhand (FOV 68 deg).
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn render_first_person_hand(
        &self,
        buf_idx: usize,
        camera: &Camera,
        hand_block: Option<u8>,
        offhand_block: Option<u8>,
        is_blocking: bool,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
        daylight: f32,
        brightness: f32,
        v_start: usize,
        i_start: usize,
        stats: &mut FrameRenderStats,
    ) {
        let vb_base = (self.entity_vb_mem.as_mut_ptr::<GxmEntityVertex>()).add(buf_idx * Self::MAX_ENTITY_VERTS);
        let ib_base = (self.entity_ib_mem.as_mut_ptr::<u16>()).add(buf_idx * Self::MAX_ENTITY_INDICES);
        let mut batch = EntityBatch::with_cursor(vb_base, ib_base, v_start, i_start, Self::MAX_ENTITY_VERTS, Self::MAX_ENTITY_INDICES);

        append_hand_3d(&mut batch, camera, hand_block, walk_phase, speed, attack_animation, daylight, brightness);
        append_offhand_3d(&mut batch, camera, offhand_block, walk_phase, speed, is_blocking, daylight, brightness);
        let hand_indices = batch.i_count.saturating_sub(i_start);
        if hand_indices == 0 {
            return;
        }

        // Depth-only clear (color mask NONE) so world pixels behind the hand remain intact
        sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 1.0, 0.0);
        sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);
        sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
        sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
        sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
        sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
        sceGxmSetVertexProgram(self.context, self.shaders.clear_vert_prog);
        sceGxmSetFragmentProgram(self.context, self.shaders.clear_frag_depth_only);

        let mut clear_buf: *mut core::ffi::c_void = core::ptr::null_mut();
        sceGxmReserveFragmentDefaultUniformBuffer(self.context, &mut clear_buf);
        if !clear_buf.is_null() && !self.shaders.param_clear_color.is_null() {
            let dummy = [0.0f32, 0.0, 0.0, 1.0];
            sceGxmSetUniformDataF(clear_buf, self.shaders.param_clear_color, 0, 4, dummy.as_ptr());
        }
        sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
        if sceGxmDraw(self.context, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U16, self.clear_ib_mem.ptr, 3) == SCE_OK {
            stats.draw_calls += 1;
        }

        sceGxmSetViewport(self.context, 480.0, 480.0, 272.0, -272.0, 0.0, 1.0);
        let aspect = VITA_WIDTH as f32 / VITA_HEIGHT as f32;
        let hand_proj = Self::gxm_perspective_matrix(68.0_f32.to_radians(), aspect, 0.01, 20.0);
        let hand_mvp = hand_proj.mul(&camera.view_matrix());

        sceGxmSetVertexProgram(self.context, self.shaders.entity_vert_prog);
        sceGxmSetFragmentProgram(self.context, self.shaders.entity_frag_opaque);
        sceGxmSetFragmentTexture(self.context, 0, &self.atlas_tex);
        sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);
        sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_LESS);
        sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_LESS);
        sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
        sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);

        bind_entity_vs_uniforms(self.context, &self.shaders, &hand_mvp);
        bind_entity_fs_uniforms(self.context, &self.shaders, camera.position, 0.0, [0.0; 3], false);
        sceGxmSetVertexStream(self.context, 0, vb_base as *const core::ffi::c_void);
        if sceGxmDraw(self.context, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U16, ib_base.add(i_start) as *const core::ffi::c_void, hand_indices as u32) == SCE_OK {
            stats.tri_count += hand_indices as u32 / 3;
            stats.draw_calls += 1;
        }
    }
}

unsafe fn bind_entity_vs_uniforms(
    ctx: *mut SceGxmContext,
    shaders: &super::programs::LinkedGxmShaders,
    mvp: &Mat4,
) {
    let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
    sceGxmReserveVertexDefaultUniformBuffer(ctx, &mut buf);
    if !buf.is_null() && !shaders.param_entity_mvp.is_null() {
        sceGxmSetUniformDataF(buf, shaders.param_entity_mvp, 0, 16, mvp.m.as_ptr());
    }
}

unsafe fn bind_entity_fs_uniforms(
    ctx: *mut SceGxmContext,
    shaders: &super::programs::LinkedGxmShaders,
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
    if !shaders.param_entity_cam_fog.is_null() {
        let data = [cam_pos.x, cam_pos.y, cam_pos.z, max_fog];
        sceGxmSetUniformDataF(buf, shaders.param_entity_cam_fog, 0, 4, data.as_ptr());
    }
    if !shaders.param_entity_fog_color.is_null() {
        let data = [fog_col[0], fog_col[1], fog_col[2], if fog_enabled { 1.0 } else { 0.0 }];
        sceGxmSetUniformDataF(buf, shaders.param_entity_fog_color, 0, 4, data.as_ptr());
    }
}
