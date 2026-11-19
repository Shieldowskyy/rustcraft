//! Mob box emission and 3D oriented box rasterization primitives.
//! Zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::entities::mob::{Mob, MobBox, MobKind, emit_mob_boxes};
use crate::features::meshing::{Triangle, Vertex};
use crate::features::player::Camera;
use crate::features::rendering::atlas::TextureId;

use super::types::Renderer;

impl Renderer {
    pub fn draw_mob(&mut self, mob: &Mob, camera: &Camera, view_proj: &Mat4) {
        let hurt_tint = if mob.hurt_timer > 0.0 {
            0.7
        } else if mob.kind == MobKind::Reaper && mob.fuse > 0.0 && ((mob.fuse * 9.0) as i32 % 2 == 1) {
            0.85
        } else {
            0.0
        };
        let camera_pos = camera.position;
        emit_mob_boxes(mob, |b| {
            self.draw_mob_box(&b, hurt_tint, camera_pos, view_proj);
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_oriented_box(
        &mut self,
        origin: Vec3,
        center: Vec3,
        size: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        tex_top: TextureId,
        tex_bottom: TextureId,
        tex_front: TextureId,
        tex_back: TextureId,
        tex_left: TextureId,
        tex_right: TextureId,
        hurt_tint: f32,
        camera_pos: Vec3,
        view_proj: &Mat4,
    ) {
        let half = Vec3::new(size.x * 0.5, size.y * 0.5, size.z * 0.5);

        let get_pt = |sx: f32, sy: f32, sz: f32| -> Vec3 {
            origin
                + right * (center.x + sx * half.x)
                + up * (center.y + sy * half.y)
                + forward * (center.z + sz * half.z)
        };

        let mut draw_quad = |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, tex: TextureId, shade: f32| {
            let edge1 = p1 - p0;
            let edge2 = p3 - p0;
            let normal = edge2.cross(edge1);
            let c = (p0 + p1 + p2 + p3) * 0.25;
            let to_face = c - camera_pos;
            if to_face.dot(normal) >= 0.0 {
                return;
            }

            let t1 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p1, (1.0, 1.0), shade),
                v2: Vertex::new(p2, (1.0, 0.0), shade),
                texture: tex,
            };
            let t2 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p2, (1.0, 0.0), shade),
                v2: Vertex::new(p3, (0.0, 0.0), shade),
                texture: tex,
            };
            self.raster_triangle_tinted(&t1, hurt_tint, view_proj);
            self.raster_triangle_tinted(&t2, hurt_tint, view_proj);
        };

        draw_quad(get_pt(-1.0, -1.0, 1.0), get_pt(1.0, -1.0, 1.0), get_pt(1.0, 1.0, 1.0), get_pt(-1.0, 1.0, 1.0), tex_front, 0.85);
        draw_quad(get_pt(1.0, -1.0, -1.0), get_pt(-1.0, -1.0, -1.0), get_pt(-1.0, 1.0, -1.0), get_pt(1.0, 1.0, -1.0), tex_back, 0.85);
        draw_quad(get_pt(1.0, -1.0, 1.0), get_pt(1.0, -1.0, -1.0), get_pt(1.0, 1.0, -1.0), get_pt(1.0, 1.0, 1.0), tex_right, 0.70);
        draw_quad(get_pt(-1.0, -1.0, -1.0), get_pt(-1.0, -1.0, 1.0), get_pt(-1.0, 1.0, 1.0), get_pt(-1.0, 1.0, -1.0), tex_left, 0.70);
        draw_quad(get_pt(-1.0, 1.0, 1.0), get_pt(1.0, 1.0, 1.0), get_pt(1.0, 1.0, -1.0), get_pt(-1.0, 1.0, -1.0), tex_top, 1.00);
        draw_quad(get_pt(-1.0, -1.0, -1.0), get_pt(1.0, -1.0, -1.0), get_pt(1.0, -1.0, 1.0), get_pt(-1.0, -1.0, 1.0), tex_bottom, 0.50);
    }

    fn draw_mob_box(&mut self, b: &MobBox, hurt_tint: f32, camera_pos: Vec3, view_proj: &Mat4) {
        b.emit_visible_faces(camera_pos, |p0, p1, p2, p3, tex, shade| {
            let t1 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p1, (1.0, 1.0), shade),
                v2: Vertex::new(p2, (1.0, 0.0), shade),
                texture: tex,
            };
            let t2 = Triangle {
                v0: Vertex::new(p0, (0.0, 1.0), shade),
                v1: Vertex::new(p2, (1.0, 0.0), shade),
                v2: Vertex::new(p3, (0.0, 0.0), shade),
                texture: tex,
            };
            self.raster_triangle_tinted(&t1, hurt_tint, view_proj);
            self.raster_triangle_tinted(&t2, hurt_tint, view_proj);
        });
    }
}
