//! Bezalocational GXM transient geometry batching for mobs, items, TNT, clouds, cracks, and particles.
//! Emits 40-byte `GxmEntityVertex` quads into triple-buffered CDRAM pools matching PC lighting curves.

#![allow(dead_code)]

use crate::core::math::{IVec3, Vec3};
use crate::features::rendering::compute_entity_light_rgb;
use crate::graphics::atlas::TextureId;
use crate::graphics::camera::Camera;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::{Mob, MobKind};
use crate::world::World;

use super::models::append_item_3d_oriented;
use super::textures::atlas_uv;
use super::types::GxmEntityVertex;

/// Non-allocating transient vertex/index writer backed by the active CDRAM entity buffer slice.
pub(super) struct EntityBatch {
    pub(super) vb: *mut GxmEntityVertex,
    pub(super) ib: *mut u16,
    pub(super) v_count: usize,
    pub(super) i_count: usize,
    pub(super) max_verts: usize,
    pub(super) max_idx: usize,
}

impl EntityBatch {
    #[inline]
    pub(super) fn new(
        vb: *mut GxmEntityVertex,
        ib: *mut u16,
        max_verts: usize,
        max_idx: usize,
    ) -> Self {
        Self { vb, ib, v_count: 0, i_count: 0, max_verts, max_idx }
    }

    #[inline]
    pub(super) fn with_cursor(
        vb: *mut GxmEntityVertex,
        ib: *mut u16,
        v_count: usize,
        i_count: usize,
        max_verts: usize,
        max_idx: usize,
    ) -> Self {
        Self { vb, ib, v_count, i_count, max_verts, max_idx }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(super) fn push_quad_raw(
        &mut self,
        c0: Vec3,
        c1: Vec3,
        c2: Vec3,
        c3: Vec3,
        uvs: [(f32, f32); 4],
        color: [f32; 4],
        tint: [u8; 4],
    ) {
        if self.v_count + 4 > self.max_verts || self.i_count + 6 > self.max_idx {
            return;
        }
        let base = self.v_count as u16;
        unsafe {
            *self.vb.add(self.v_count) = GxmEntityVertex { x: c0.x, y: c0.y, z: c0.z, u: uvs[0].0, v: uvs[0].1, color, tint };
            *self.vb.add(self.v_count + 1) = GxmEntityVertex { x: c1.x, y: c1.y, z: c1.z, u: uvs[1].0, v: uvs[1].1, color, tint };
            *self.vb.add(self.v_count + 2) = GxmEntityVertex { x: c2.x, y: c2.y, z: c2.z, u: uvs[2].0, v: uvs[2].1, color, tint };
            *self.vb.add(self.v_count + 3) = GxmEntityVertex { x: c3.x, y: c3.y, z: c3.z, u: uvs[3].0, v: uvs[3].1, color, tint };

            *self.ib.add(self.i_count) = base;
            *self.ib.add(self.i_count + 1) = base + 1;
            *self.ib.add(self.i_count + 2) = base + 2;
            *self.ib.add(self.i_count + 3) = base;
            *self.ib.add(self.i_count + 4) = base + 2;
            *self.ib.add(self.i_count + 5) = base + 3;
        }
        self.v_count += 4;
        self.i_count += 6;
    }

    #[inline(always)]
    pub(super) fn push_textured_quad(
        &mut self,
        c0: Vec3,
        c1: Vec3,
        c2: Vec3,
        c3: Vec3,
        tex: TextureId,
        color: [f32; 4],
        tint: [u8; 4],
    ) {
        let (u0, v0, u1, v1) = atlas_uv(tex);
        self.push_quad_raw(c0, c1, c2, c3, [(u0, v1), (u1, v1), (u1, v0), (u0, v0)], color, tint);
    }

    #[inline(always)]
    pub(super) fn push_untextured_quad(
        &mut self,
        c0: Vec3,
        c1: Vec3,
        c2: Vec3,
        c3: Vec3,
        color: [f32; 4],
    ) {
        // DLACZEGO: u < 0.0 signals entity.frag.cg to skip texture lookup and use flat vertex color.
        self.push_quad_raw(c0, c1, c2, c3, [(-1.0, 0.0); 4], color, [0, 0, 0, 0]);
    }

    #[inline]
    pub(super) fn push_billboard(
        &mut self,
        pos: Vec3,
        size: f32,
        rgb: [f32; 3],
        right_flat: Vec3,
        up: Vec3,
    ) {
        let h = size * 0.5;
        let p0 = pos - right_flat * h - up * h;
        let p1 = pos + right_flat * h - up * h;
        let p2 = pos + right_flat * h + up * h;
        let p3 = pos - right_flat * h + up * h;
        self.push_untextured_quad(p0, p1, p2, p3, [rgb[0], rgb[1], rgb[2], 1.0]);
    }
}

#[inline(always)]
pub(super) fn entity_lit_color(shade: f32, daylight: f32, brightness: f32) -> [f32; 4] {
    let [r, g, b] = compute_entity_light_rgb(shade, 1.0, 0.0, daylight, brightness);
    [r, g, b, 1.0]
}

#[inline(always)]
pub(super) fn hurt_tint_bytes(hurt_tint: f32) -> [u8; 4] {
    [242, 31, 31, (hurt_tint.clamp(0.0, 1.0) * 255.0).round() as u8]
}

pub(super) fn append_clouds(
    batch: &mut EntityBatch,
    player_pos: Vec3,
    time: f32,
    daylight: f32,
    fog_col: [f32; 3],
    rain_strength: f32,
    thunder_strength: f32,
) {
    use crate::graphics::clouds::*;

    if (player_pos.y - CLOUD_Y).abs() > CLOUD_MAX_VERTICAL_DISTANCE {
        return;
    }

    let cloud_base = compute_cloud_color(daylight, fog_col, rain_strength, thunder_strength);
    let scroll = time * CLOUD_SCROLL_SPEED;
    let max_dist_sq = (7.5 * CLOUD_CELL_SIZE).powi(2);
    let (cx, cz) = cloud_origin_cell(player_pos, time);

    for dz in -7..=7 {
        for dx in -7..=7 {
            let gx = cx + dx;
            let gz = cz + dz;
            let center_x = (gx as f32 + 0.5) * CLOUD_CELL_SIZE + scroll;
            let center_z = (gz as f32 + 0.5) * CLOUD_CELL_SIZE;
            let dist_sq = (center_x - player_pos.x).powi(2) + (center_z - player_pos.z).powi(2);
            if dist_sq > max_dist_sq {
                continue;
            }
            emit_cloud_faces(gx, gz, player_pos, time, |p0, p1, p2, p3, shade| {
                let color = [
                    (cloud_base[0] * shade).clamp(0.0, 1.0),
                    (cloud_base[1] * shade).clamp(0.0, 1.0),
                    (cloud_base[2] * shade).clamp(0.0, 1.0),
                    0.80,
                ];
                batch.push_untextured_quad(p0, p1, p2, p3, color);
            });
        }
    }
}

pub(super) fn append_particles(
    batch: &mut EntityBatch,
    particles: &[crate::gameplay::particles::Particle],
    camera: &Camera,
) {
    let right_flat = camera.right_flat();
    let up = Vec3::new(0.0, 1.0, 0.0);
    for p in particles {
        batch.push_billboard(p.pos, p.size, p.color, right_flat, up);
    }
}

pub(super) fn append_fishing_bobber(
    batch: &mut EntityBatch,
    bobber: &crate::app::FishingBobber,
    camera: &Camera,
) {
    let right_flat = camera.right_flat();
    let up = Vec3::new(0.0, 1.0, 0.0);
    let bob_color = if bobber.in_water && bobber.bite_timer <= 0.0 {
        [1.0, 0.3, 0.1]
    } else {
        [0.9, 0.15, 0.15]
    };
    batch.push_billboard(bobber.pos, 0.16, bob_color, right_flat, up);
    batch.push_billboard(bobber.pos + Vec3::new(0.0, 0.08, 0.0), 0.10, [0.95, 0.95, 0.95], right_flat, up);
}

pub(super) fn append_arrows(
    batch: &mut EntityBatch,
    arrows: &[crate::features::entities::ArrowEntity],
    camera: &Camera,
) {
    let right_flat = camera.right_flat();
    let up = Vec3::new(0.0, 1.0, 0.0);
    for arrow in arrows {
        let dir = if arrow.velocity.length() > 0.001 {
            arrow.velocity.normalized()
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        };
        batch.push_billboard(arrow.position, 0.12, [0.35, 0.35, 0.38], right_flat, up);
        batch.push_billboard(arrow.position - dir * 0.18, 0.09, [0.65, 0.45, 0.25], right_flat, up);
        batch.push_billboard(arrow.position - dir * 0.36, 0.11, [0.95, 0.95, 0.95], right_flat, up);
    }
}

pub(super) fn append_block_break_cracks(
    batch: &mut EntityBatch,
    target: IVec3,
    progress: f32,
    daylight: f32,
    brightness: f32,
) {
    if progress <= 0.0 {
        return;
    }
    let stage = ((progress * 10.0).floor() as usize).min(9);
    let crack_tex = TextureId::destroy_stage(stage);
    let color = entity_lit_color(1.0, daylight, brightness);
    let eps = 0.003f32;
    let x0 = target.x as f32 - eps;
    let x1 = target.x as f32 + 1.0 + eps;
    let y0 = target.y as f32 - eps;
    let y1 = target.y as f32 + 1.0 + eps;
    let z0 = target.z as f32 - eps;
    let z1 = target.z as f32 + 1.0 + eps;

    let faces = [
        (Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0)),
        (Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1)),
        (Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1)),
        (Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0)),
        (Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0)),
        (Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1)),
    ];
    for (c0, c1, c2, c3) in faces {
        batch.push_textured_quad(c0, c1, c2, c3, crack_tex, color, [0; 4]);
    }
}

pub(super) fn append_item_shadow(
    batch: &mut EntityBatch,
    item: &ItemEntity,
    world: &World,
    daylight: f32,
    brightness: f32,
) {
    let ipos_x = item.position.x.floor() as i32;
    let ipos_z = item.position.z.floor() as i32;
    let item_y = item.position.y;
    for dy in 0..=3 {
        let test_y = (item_y.floor() as i32) - dy;
        let b = world.get_block(IVec3::new(ipos_x, test_y, ipos_z));
        if b.is_solid() {
            let top_y = test_y as f32 + 1.0 + 0.015;
            if item_y >= top_y - 0.2 {
                let s_rad = 0.22f32;
                let s_c0 = Vec3::new(item.position.x - s_rad, top_y, item.position.z + s_rad);
                let s_c1 = Vec3::new(item.position.x + s_rad, top_y, item.position.z + s_rad);
                let s_c2 = Vec3::new(item.position.x + s_rad, top_y, item.position.z - s_rad);
                let s_c3 = Vec3::new(item.position.x - s_rad, top_y, item.position.z - s_rad);
                let color = entity_lit_color(1.0, daylight, brightness);
                batch.push_textured_quad(s_c0, s_c1, s_c2, s_c3, TextureId::EntityShadow, color, [0; 4]);
                break;
            }
        }
    }
}

pub(super) fn append_dropped_item_3d(
    batch: &mut EntityBatch,
    item: &ItemEntity,
    daylight: f32,
    brightness: f32,
) {
    let y_offset = (item.age * 3.0).sin() * 0.08;
    let mut pos = item.position;
    pos.y += y_offset;
    let yaw = item.age * 2.5;
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let up = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    append_item_3d_oriented(batch, item.block, pos, right, up, forward, 0.85, daylight, brightness);
}

pub(super) fn append_mob(
    batch: &mut EntityBatch,
    mob: &Mob,
    camera_pos: Vec3,
    daylight: f32,
    brightness: f32,
) {
    let hurt_tint = if mob.hurt_timer > 0.0 {
        0.7
    } else if mob.kind == MobKind::Reaper && mob.fuse > 0.0 && ((mob.fuse * 9.0) as i32 % 2 == 1) {
        0.85
    } else {
        0.0
    };
    let tint = hurt_tint_bytes(hurt_tint);
    crate::gameplay::mob::emit_mob_boxes(mob, |b| {
        b.emit_visible_faces(camera_pos, |p0, p1, p2, p3, tex, shade| {
            let color = entity_lit_color(shade, daylight, brightness);
            batch.push_textured_quad(p0, p1, p2, p3, tex, color, tint);
        });
    });
}

pub(super) fn append_tnt(
    batch: &mut EntityBatch,
    tnt: &crate::world::PrimedTnt,
    camera_pos: Vec3,
    daylight: f32,
    brightness: f32,
) {
    let p = tnt.pos;
    let (x0, x1) = (p.x, p.x + 1.0);
    let (y0, y1) = (p.y, p.y + 1.0);
    let (z0, z1) = (p.z, p.z + 1.0);
    let flash = (tnt.fuse * 12.0).sin().abs() * 0.6;
    let tint = [255, 255, 255, (flash.clamp(0.0, 1.0) * 255.0).round() as u8];

    let faces = [
        (Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0), TextureId::TntTop, 1.00),
        (Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1), TextureId::TntTop, 0.50),
        (Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1), TextureId::TntSide, 0.85),
        (Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0), TextureId::TntSide, 0.85),
        (Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0), TextureId::TntSide, 0.70),
        (Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1), TextureId::TntSide, 0.70),
    ];

    for (c0, c1, c2, c3, tex, ao) in faces {
        let normal = (c1 - c0).cross(c3 - c0);
        let center = (c0 + c1 + c2 + c3) * 0.25;
        if (center - camera_pos).dot(normal) >= 0.0 {
            continue;
        }
        let color = entity_lit_color(ao, daylight, brightness);
        batch.push_textured_quad(c0, c1, c2, c3, tex, color, tint);
    }
}
