//! Alpha-blended dynamic pass: block cracks, sky bodies, clouds, particles, items, mobs, TNT, and hand.
//! Builds transient per-frame geometry into the entity GPU buffers and submits draw calls.

#![allow(dead_code)]

use crate::core::math::{IVec3, Vec3};
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::Mob;
use crate::gameplay::particles::Particle;
use crate::graphics::atlas::TextureId;
use crate::graphics::camera::Camera;
use crate::world::World;

use super::entities::{
    append_celestial_bodies, append_clouds, append_dropped_item, append_mob, append_particles,
    append_screen_overlay, append_tnt,
};
use super::ffi::*;
use super::renderer::GxmRenderer;
use super::models::append_hand_3d;
use super::textures::atlas_uv;
use super::types::{FrameRenderStats, GxmVertex};

impl GxmRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn render_cracks_and_entities(
        &mut self,
        camera: &Camera,
        world: &World,
        break_target: Option<IVec3>,
        break_progress: f32,
        items: &[ItemEntity],
        mobs: &[Mob],
        hand_block: Option<u8>,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
        clouds_enabled: bool,
        total_time: f32,
        particles: &[Particle],
        damage_flash: f32,
        is_underwater: bool,
        is_in_lava: bool,
        low_health: bool,
        daylight: f32,
        sun_dir: Vec3,
        is_occluded: bool,
        stats: &mut FrameRenderStats,
    ) {
            // Restore depth write and alpha-blended fragment program for entities & overlays
            sceGxmSetFragmentProgram(self.context, self.shaders.frag_prog_trans);
            sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);

            let _ = (break_target, break_progress, atlas_uv);

            // 2c. Draw dynamic objects (celestial bodies, clouds, particles, dropped items, mobs, TNT, 3D hand, full-screen overlays)
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);

            let mut ent_v_count = 0usize;
            let mut ent_i_count = 0usize;
            let ent_vb = self.entity_vb_mem.ptr as *mut GxmVertex;
            let ent_ib = self.entity_ib_mem.ptr as *mut u16;
            let max_ent_verts = Self::MAX_ENTITY_VERTS;
            let max_ent_idx = Self::MAX_ENTITY_INDICES;

            if !is_occluded {
                append_celestial_bodies(
                    camera.position,
                    sun_dir,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if clouds_enabled && !is_occluded {
                append_clouds(
                    camera.position,
                    total_time,
                    daylight,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            append_particles(
                particles,
                camera,
                ent_vb,
                ent_ib,
                &mut ent_v_count,
                &mut ent_i_count,
                max_ent_verts,
                max_ent_idx,
            );

            for item in items {
                append_dropped_item(item, world, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            for mob in mobs {
                append_mob(mob, camera.position, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            for tnt in &world.primed_tnts {
                append_tnt(tnt, camera.position, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            append_hand_3d(
                camera,
                hand_block,
                walk_phase,
                speed,
                attack_animation,
                ent_vb,
                ent_ib,
                &mut ent_v_count,
                &mut ent_i_count,
                max_ent_verts,
                max_ent_idx,
            );

            if is_underwater {
                append_screen_overlay(
                    camera,
                    TextureId::UnderwaterTint,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if is_in_lava {
                append_screen_overlay(
                    camera,
                    TextureId::LavaTint,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if damage_flash > 0.0 {
                append_screen_overlay(
                    camera,
                    TextureId::DamageFlash,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if low_health {
                let pulse = (total_time * 6.5).sin() * 0.5 + 0.5;
                if pulse > 0.25 {
                    append_screen_overlay(
                        camera,
                        TextureId::VignetteTint,
                        ent_vb,
                        ent_ib,
                        &mut ent_v_count,
                        &mut ent_i_count,
                        max_ent_verts,
                        max_ent_idx,
                    );
                }
            }

            if ent_i_count > 0 {
                sceGxmSetVertexStream(self.context, 0, self.entity_vb_mem.ptr);
                let entity_draw_rc = sceGxmDraw(
                    self.context,
                    SCE_GXM_PRIMITIVE_TRIANGLES,
                    SCE_GXM_INDEX_FORMAT_U16,
                    self.entity_ib_mem.ptr,
                    ent_i_count as u32,
                );
                if entity_draw_rc == SCE_OK {
                    stats.tri_count += ent_i_count as u32 / 3;
                    stats.draw_calls += 1;
                } else {
                    crate::vita_log!("sceGxmDraw(entities) failed: 0x{:08X}", entity_draw_rc);
                }
            }
    }
}
