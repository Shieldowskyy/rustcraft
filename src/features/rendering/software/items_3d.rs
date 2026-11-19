//! 3D dropped item entities, procedural tools/weapons/food boxes, and entity shadows.
//! Zero platform `#[cfg]`.

use crate::core::BlockFace;
use crate::core::math::{Mat4, Vec3};
use crate::features::blocks::BlockId;
use crate::features::entities::item::{
    emit_arrow_boxes, emit_feather_boxes, emit_meat_boxes, emit_procedural_tool_boxes,
    emit_torch_boxes,
};
use crate::features::meshing::{Triangle, Vertex};
use crate::features::rendering::atlas::TextureId;

use super::types::Renderer;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn draw_procedural_tool_3d(
        &mut self,
        block: BlockId,
        pos: Vec3,
        tool_right: Vec3,
        tool_up: Vec3,
        tool_forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        emit_procedural_tool_boxes(block, pos, tool_right, tool_up, scale, |part_pos, half, tex| {
            self.draw_cube_3d(
                part_pos,
                tool_right,
                tool_up,
                tool_forward,
                half,
                tex,
                view_proj,
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_meat_3d(
        &mut self,
        block: BlockId,
        pos: Vec3,
        meat_right: Vec3,
        meat_up: Vec3,
        meat_forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        emit_meat_boxes(block, pos, meat_right, meat_up, scale, |part_pos, half, tex| {
            self.draw_cube_3d(
                part_pos,
                meat_right,
                meat_up,
                meat_forward,
                half,
                tex,
                view_proj,
            );
        });
    }

    pub fn draw_feather_3d(
        &mut self,
        pos: Vec3,
        feather_right: Vec3,
        feather_up: Vec3,
        feather_forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        emit_feather_boxes(pos, feather_right, feather_up, scale, |part_pos, half, tex| {
            self.draw_cube_3d(
                part_pos,
                feather_right,
                feather_up,
                feather_forward,
                half,
                tex,
                view_proj,
            );
        });
    }

    pub fn draw_arrow_3d(
        &mut self,
        pos: Vec3,
        arrow_right: Vec3,
        arrow_up: Vec3,
        arrow_forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        emit_arrow_boxes(pos, arrow_up, scale, |part_pos, half, tex| {
            self.draw_cube_3d(
                part_pos,
                arrow_right,
                arrow_up,
                arrow_forward,
                half,
                tex,
                view_proj,
            );
        });
    }

    pub fn draw_torch_3d(
        &mut self,
        pos: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        emit_torch_boxes(pos, up, scale, |part_pos, half, tex| {
            self.draw_cube_3d(part_pos, right, up, forward, half, tex, view_proj);
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_item_3d_oriented(
        &mut self,
        block: BlockId,
        pos: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        scale: f32,
        view_proj: &Mat4,
    ) {
        if block == BlockId::Torch {
            self.draw_torch_3d(pos, right, up, forward, scale, view_proj);
        } else if block.is_tool() {
            let tilt_angle = 0.38f32;
            let tool_up = (up * tilt_angle.cos() + right * tilt_angle.sin()).normalized();
            let tool_right = (right * tilt_angle.cos() - up * tilt_angle.sin()).normalized();
            self.draw_procedural_tool_3d(
                block, pos, tool_right, tool_up, forward, scale, view_proj,
            );
        } else if block.is_meat() {
            let tilt_angle = 0.32f32;
            let meat_up = (up * tilt_angle.cos() + right * tilt_angle.sin()).normalized();
            let meat_right = (right * tilt_angle.cos() - up * tilt_angle.sin()).normalized();
            self.draw_meat_3d(block, pos, meat_right, meat_up, forward, scale, view_proj);
        } else if block == BlockId::Feather {
            let tilt_angle = 0.35f32;
            let feather_up = (up * tilt_angle.cos() + right * tilt_angle.sin()).normalized();
            let feather_right = (right * tilt_angle.cos() - up * tilt_angle.sin()).normalized();
            self.draw_feather_3d(
                pos,
                feather_right,
                feather_up,
                forward,
                scale * 1.1,
                view_proj,
            );
        } else if matches!(
            block,
            BlockId::WheatCrop
                | BlockId::CarrotCrop
                | BlockId::PotatoCrop
                | BlockId::Sapling
                | BlockId::Kelp
                | BlockId::Seagrass
        ) {
            let size = 0.35 * scale;
            let half = size * 0.5;
            let tex = block.texture_id(BlockFace::Top);
            let ao = 0.95;

            let p0 = pos + (right - forward) * half - up * half;
            let p1 = pos + (-right + forward) * half - up * half;
            let p2 = pos + (-right + forward) * half + up * half;
            let p3 = pos + (right - forward) * half + up * half;

            let q0 = pos + (right + forward) * half - up * half;
            let q1 = pos + (-right - forward) * half - up * half;
            let q2 = pos + (-right - forward) * half + up * half;
            let q3 = pos + (right + forward) * half + up * half;

            let mut add_quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| {
                let t1 = Triangle {
                    v0: Vertex::new(a, (0.0, 1.0), ao),
                    v1: Vertex::new(b, (1.0, 1.0), ao),
                    v2: Vertex::new(c, (1.0, 0.0), ao),
                    texture: tex,
                };
                let t2 = Triangle {
                    v0: Vertex::new(a, (0.0, 1.0), ao),
                    v1: Vertex::new(c, (1.0, 0.0), ao),
                    v2: Vertex::new(d, (0.0, 0.0), ao),
                    texture: tex,
                };
                self.raster_triangle(&t1, view_proj);
                self.raster_triangle(&t2, view_proj);
            };

            add_quad(p0, p1, p2, p3);
            add_quad(p1, p0, p3, p2);
            add_quad(q0, q1, q2, q3);
            add_quad(q1, q0, q3, q2);
        } else {
            let size = 0.28 * scale;
            let half = size * 0.5;
            let corner = |x: f32, y: f32, z: f32| {
                pos + right * (x * half) + up * (y * half) + forward * (z * half)
            };

            let face_data = [
                ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(BlockFace::Top), 1.00),
                ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], block.texture_id(BlockFace::Bottom), 0.55),
                ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], block.texture_id(BlockFace::South), 0.85),
                ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], block.texture_id(BlockFace::North), 0.85),
                ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(BlockFace::West), 0.70),
                ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], block.texture_id(BlockFace::East), 0.70),
            ];

            for (corners, tex, ao) in face_data {
                let t1 = Triangle {
                    v0: Vertex::new(corners[0], (0.0, 1.0), ao),
                    v1: Vertex::new(corners[1], (1.0, 1.0), ao),
                    v2: Vertex::new(corners[2], (1.0, 0.0), ao),
                    texture: tex,
                };
                let t2 = Triangle {
                    v0: Vertex::new(corners[0], (0.0, 1.0), ao),
                    v1: Vertex::new(corners[2], (1.0, 0.0), ao),
                    v2: Vertex::new(corners[3], (0.0, 0.0), ao),
                    texture: tex,
                };
                self.raster_triangle(&t1, view_proj);
                self.raster_triangle(&t2, view_proj);
            }
        }
    }

    pub fn draw_item_3d(&mut self, block: BlockId, pos: Vec3, yaw: f32, view_proj: &Mat4) {
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let up = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        self.draw_item_3d_oriented(block, pos, right, up, forward, 0.85, view_proj);
    }

    pub fn draw_item_shadow(&mut self, pos: Vec3, ground_y: f32, view_proj: &Mat4) {
        let s_rad = 0.22f32;
        let (sx0, sx1) = (pos.x - s_rad, pos.x + s_rad);
        let (sz0, sz1) = (pos.z - s_rad, pos.z + s_rad);
        let s_c0 = Vec3::new(sx0, ground_y, sz1);
        let s_c1 = Vec3::new(sx1, ground_y, sz1);
        let s_c2 = Vec3::new(sx1, ground_y, sz0);
        let s_c3 = Vec3::new(sx0, ground_y, sz0);

        let t1 = Triangle {
            v0: Vertex::new(s_c0, (0.0, 1.0), 1.0),
            v1: Vertex::new(s_c1, (1.0, 1.0), 1.0),
            v2: Vertex::new(s_c2, (1.0, 0.0), 1.0),
            texture: TextureId::EntityShadow,
        };
        let t2 = Triangle {
            v0: Vertex::new(s_c0, (0.0, 1.0), 1.0),
            v1: Vertex::new(s_c2, (1.0, 0.0), 1.0),
            v2: Vertex::new(s_c3, (0.0, 0.0), 1.0),
            texture: TextureId::EntityShadow,
        };
        self.raster_triangle(&t1, view_proj);
        self.raster_triangle(&t2, view_proj);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_cube_3d(
        &mut self,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        half: [f32; 3],
        texture: TextureId,
        view_proj: &Mat4,
    ) {
        self.draw_multitex_cube_3d(
            center, right, up, forward, half, texture, texture, texture, view_proj,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_multitex_cube_3d(
        &mut self,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        forward: Vec3,
        half: [f32; 3],
        top_tex: TextureId,
        bot_tex: TextureId,
        side_tex: TextureId,
        view_proj: &Mat4,
    ) {
        let corner = |x: f32, y: f32, z: f32| {
            center + right * (x * half[0]) + up * (y * half[1]) + forward * (z * half[2])
        };
        let faces = [
            ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], side_tex, 0.85),
            ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], side_tex, 0.85),
            ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], side_tex, 0.70),
            ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], side_tex, 0.70),
            ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], top_tex, 1.00),
            ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], bot_tex, 0.55),
        ];
        for (corners, texture, ao) in faces {
            let vertices = [
                Vertex::new(corners[0], (0.0, 1.0), ao),
                Vertex::new(corners[1], (1.0, 1.0), ao),
                Vertex::new(corners[2], (1.0, 0.0), ao),
                Vertex::new(corners[3], (0.0, 0.0), ao),
            ];
            let first = Triangle { v0: vertices[0], v1: vertices[1], v2: vertices[2], texture };
            let second = Triangle { v0: vertices[0], v1: vertices[2], v2: vertices[3], texture };
            self.raster_triangle(&first, view_proj);
            self.raster_triangle(&second, view_proj);
            self.raster_triangle(&Triangle { v0: first.v2, v1: first.v1, v2: first.v0, texture }, view_proj);
            self.raster_triangle(&Triangle { v0: second.v2, v1: second.v1, v2: second.v0, texture }, view_proj);
        }
    }
}
