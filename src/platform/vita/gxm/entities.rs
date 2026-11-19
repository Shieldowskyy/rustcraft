//! GXM transient vertex/index builders for sky bodies, clouds, particles, screen tints, items, mobs, and TNT.
//! Emits textured quads into `entity_vb_mem` / `entity_ib_mem` without per-frame heap allocations.

#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::graphics::atlas::TextureId;
use crate::graphics::camera::Camera;
use crate::world::BlockId;

use super::models::{append_feather_3d, append_meat_3d, append_procedural_tool_3d};
use super::textures::atlas_uv;
use super::types::GxmVertex;

pub(super) fn append_quad(
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
    c0: Vec3,
    c1: Vec3,
    c2: Vec3,
    c3: Vec3,
    texture: TextureId,
    ao: f32,
) {
    if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
        return;
    }
    let (u0, v0, u1, v1) = atlas_uv(texture);
    let base = *v_count as u16;
    unsafe {
        *vb.add(*v_count)     = GxmVertex { x: c0.x, y: c0.y, z: c0.z, u: u0, v: v1, ao };
        *vb.add(*v_count + 1) = GxmVertex { x: c1.x, y: c1.y, z: c1.z, u: u1, v: v1, ao };
        *vb.add(*v_count + 2) = GxmVertex { x: c2.x, y: c2.y, z: c2.z, u: u1, v: v0, ao };
        *vb.add(*v_count + 3) = GxmVertex { x: c3.x, y: c3.y, z: c3.z, u: u0, v: v0, ao };

        *ib.add(*i_count)     = base;
        *ib.add(*i_count + 1) = base + 1;
        *ib.add(*i_count + 2) = base + 2;
        *ib.add(*i_count + 3) = base;
        *ib.add(*i_count + 4) = base + 2;
        *ib.add(*i_count + 5) = base + 3;
    }
    *v_count += 4;
    *i_count += 6;
}

pub(super) fn append_celestial_bodies(
    camera_pos: Vec3,
    sun_dir: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let dist = 180.0f32;
    let bodies = [
        (sun_dir.normalized(), 16.0f32, TextureId::GoldBlock),
        ((-sun_dir).normalized(), 12.0f32, TextureId::IronBlock),
    ];

    for (dir, half_size, tex) in bodies {
        let up_ref = if dir.y.abs() > 0.99 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let right = dir.cross(up_ref).normalized();
        let up = right.cross(dir).normalized();
        let center = camera_pos + dir * dist;

        let p0 = center - right * half_size - up * half_size;
        let p1 = center + right * half_size - up * half_size;
        let p2 = center + right * half_size + up * half_size;
        let p3 = center - right * half_size + up * half_size;

        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, tex, 1.0);
    }
}

pub(super) fn append_clouds(
    player_pos: Vec3,
    time: f32,
    daylight: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    use crate::graphics::clouds::*;

    if (player_pos.y - CLOUD_Y).abs() > CLOUD_MAX_VERTICAL_DISTANCE {
        return;
    }

    let light_scale = daylight.clamp(0.15, 1.0);
    let (cx, cz) = cloud_origin_cell(player_pos, time);
    for dz in -5..=5 {
        for dx in -5..=5 {
            emit_cloud_faces(cx + dx, cz + dz, player_pos, time, |p0, p1, p2, p3, shade| {
                append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, TextureId::Snow, shade * light_scale);
            });
        }
    }
}

pub(super) fn append_particles(
    particles: &[crate::gameplay::particles::Particle],
    camera: &Camera,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let right = camera.right();
    let up = camera.up();

    for p in particles {
        if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
            break;
        }
        let half = p.size * 0.5;
        let p0 = p.pos - right * half - up * half;
        let p1 = p.pos + right * half - up * half;
        let p2 = p.pos + right * half + up * half;
        let p3 = p.pos - right * half + up * half;

        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, TextureId::Dirt, 1.0);
    }
}

pub(super) fn append_screen_overlay(
    camera: &Camera,
    texture: TextureId,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
        return;
    }
    let fwd = camera.forward();
    let right = camera.right();
    let up = camera.up();
    let dist = 0.06; // Near-plane is 0.05
    let center = camera.position + fwd * dist;
    let half_h = dist * (camera.fov_y * 0.5).tan() * 1.5;
    let half_w = half_h * (960.0 / 544.0);

    let p0 = center - right * half_w - up * half_h;
    let p1 = center + right * half_w - up * half_h;
    let p2 = center + right * half_w + up * half_h;
    let p3 = center - right * half_w + up * half_h;

    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, texture, 1.0);
}

pub(super) fn append_dropped_item(
    item: &crate::gameplay::item::ItemEntity,
    world: &crate::world::World,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    use crate::world::block::Face;

    // 1. Circular ground drop shadow under the item entity (Minecraft style)
    let ipos_x = item.position.x.floor() as i32;
    let ipos_z = item.position.z.floor() as i32;
    let item_y = item.position.y;
    for dy in 0..=3 {
        let test_y = (item_y.floor() as i32) - dy;
        let b = world.get_block(crate::core::math::IVec3::new(ipos_x, test_y, ipos_z));
        if b.is_solid() {
            let top_y = test_y as f32 + 1.0 + 0.015;
            if item_y >= top_y - 0.2 {
                let s_rad = 0.22f32;
                let sx0 = item.position.x - s_rad;
                let sx1 = item.position.x + s_rad;
                let sz0 = item.position.z - s_rad;
                let sz1 = item.position.z + s_rad;
                let s_c0 = Vec3::new(sx0, top_y, sz1);
                let s_c1 = Vec3::new(sx1, top_y, sz1);
                let s_c2 = Vec3::new(sx1, top_y, sz0);
                let s_c3 = Vec3::new(sx0, top_y, sz0);
                append_quad(
                    vb, ib, v_count, i_count, max_verts, max_idx,
                    s_c0, s_c1, s_c2, s_c3,
                    TextureId::EntityShadow,
                    1.0,
                );
                break;
            }
        }
    }

    let y_offset = (item.age * 3.0).sin() * 0.08;
    let mut pos = item.position;
    pos.y += y_offset;
    let yaw = item.age * 2.5;

    let size = 0.28f32;
    let half = size * 0.5f32;
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let up = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

    let corner = |x: f32, y: f32, z: f32| {
        pos + right * (x * half) + up * (y * half) + forward * (z * half)
    };

    let block = item.block;
    if block.is_meat() {
        let tilt_angle = 0.32f32;
        let base_up = Vec3::new(0.0, 1.0, 0.0);
        let base_right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let base_forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

        let meat_up = (base_up * tilt_angle.cos() + base_right * tilt_angle.sin()).normalized();
        let meat_right = (base_right * tilt_angle.cos() - base_up * tilt_angle.sin()).normalized();
        let meat_forward = base_forward;

        append_meat_3d(
            block,
            pos,
            meat_right,
            meat_up,
            meat_forward,
            0.85,
            vb,
            ib,
            v_count,
            i_count,
            max_verts,
            max_idx,
        );
        return;
    }
    if block == BlockId::Feather {
        let tilt_angle = 0.35f32;
        let base_up = Vec3::new(0.0, 1.0, 0.0);
        let base_right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let base_forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

        let feather_up = (base_up * tilt_angle.cos() + base_right * tilt_angle.sin()).normalized();
        let feather_right = (base_right * tilt_angle.cos() - base_up * tilt_angle.sin()).normalized();
        let feather_forward = base_forward;

        append_feather_3d(
            pos,
            feather_right,
            feather_up,
            feather_forward,
            0.95,
            vb,
            ib,
            v_count,
            i_count,
            max_verts,
            max_idx,
        );
        return;
    }
    if block.is_tool() {
        let tilt_angle = 0.38f32;
        let base_up = Vec3::new(0.0, 1.0, 0.0);
        let base_right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let base_forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

        let tool_up = (base_up * tilt_angle.cos() + base_right * tilt_angle.sin()).normalized();
        let tool_right = (base_right * tilt_angle.cos() - base_up * tilt_angle.sin()).normalized();
        let tool_forward = base_forward;

        append_procedural_tool_3d(
            block,
            pos,
            tool_right,
            tool_up,
            tool_forward,
            0.85,
            vb,
            ib,
            v_count,
            i_count,
            max_verts,
            max_idx,
        );
        return;
    }

    let face_data = [
        ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::Top), 1.00),
        ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], block.texture_id(Face::Bottom), 0.55),
        ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], block.texture_id(Face::South), 0.85),
        ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], block.texture_id(Face::North), 0.85),
        ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::West), 0.70),
        ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], block.texture_id(Face::East), 0.70),
    ];

    for (corners, tex, ao) in face_data {
        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, corners[0], corners[1], corners[2], corners[3], tex, ao);
    }
}

pub(super) fn append_mob(
    mob: &crate::gameplay::mob::Mob,
    camera_pos: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    crate::gameplay::mob::emit_mob_boxes(mob, |b| {
        b.emit_visible_faces(camera_pos, |p0, p1, p2, p3, tex, shade| {
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, tex, shade);
        });
    });
}

pub(super) fn append_tnt(
    tnt: &crate::world::PrimedTnt,
    camera_pos: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let p = tnt.pos;
    let (x0, x1) = (p.x, p.x + 1.0);
    let (y0, y1) = (p.y, p.y + 1.0);
    let (z0, z1) = (p.z, p.z + 1.0);

    let faces = [
        (Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0), TextureId::TntTop, 1.00),
        (Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1), TextureId::TntTop, 0.50),
        (Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1), TextureId::TntSide, 0.85),
        (Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0), TextureId::TntSide, 0.85),
        (Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0), TextureId::TntSide, 0.70),
        (Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1), TextureId::TntSide, 0.70),
    ];

    for (c0, c1, c2, c3, tex, ao) in faces {
        let edge1 = c1 - c0;
        let edge2 = c3 - c0;
        let normal = edge1.cross(edge2);
        let center = (c0 + c1 + c2 + c3) * 0.25;
        let to_face = center - camera_pos;
        if to_face.dot(normal) >= 0.0 {
            continue;
        }
        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c0, c1, c2, c3, tex, ao);
    }
}

