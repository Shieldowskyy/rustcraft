//! Particle billboards, primed TNT cubes, and block-break crack overlays.
//! Zero platform `#[cfg]`.

use crate::core::BlockFace;
use crate::core::math::{IVec3, Mat4, Vec3};
use crate::features::blocks::BlockId;
use crate::features::meshing::{Triangle, Vertex};
use crate::features::player::Camera;
use crate::features::rendering::atlas::TextureId;

use super::clip::{edge, pack_color};
use super::types::Renderer;

impl Renderer {
    pub fn draw_billboard(
        &mut self,
        pos: Vec3,
        size: f32,
        color: [f32; 3],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let right = camera.right_flat();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let h = size * 0.5;

        let corners = [
            pos - right * h - up * h,
            pos + right * h - up * h,
            pos + right * h + up * h,
            pos - right * h + up * h,
        ];

        let mut pts = [(0.0f32, 0.0f32); 4];
        for (i, &p) in corners.iter().enumerate() {
            let c = view_proj.transform_point(p);
            if c[3] <= 1e-4 {
                return;
            }
            let inv_w = 1.0 / c[3];
            let sx = (c[0] * inv_w * 0.5 + 0.5) * self.width as f32;
            let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * self.height as f32;
            pts[i] = (sx, sy);
        }

        let depth_at_pos = {
            let c = view_proj.transform_point(pos);
            if c[3] <= 1e-4 {
                return;
            }
            c[3]
        };

        let packed = pack_color(color[0], color[1], color[2]);
        for &(ai, bi, ci) in &[(0usize, 1usize, 2usize), (0usize, 2usize, 3usize)] {
            let (ax, ay) = pts[ai];
            let (bx, by) = pts[bi];
            let (cx, cy) = pts[ci];
            let area = edge(ax, ay, bx, by, cx, cy);
            if area <= 0.0 {
                continue;
            }
            let min_x = ax.min(bx).min(cx).floor().max(0.0) as i32;
            let max_x = ax.max(bx).max(cx).ceil().min(self.width as f32 - 1.0) as i32;
            let min_y = ay.min(by).min(cy).floor().max(0.0) as i32;
            let max_y = ay.max(by).max(cy).ceil().min(self.height as f32 - 1.0) as i32;
            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let fx = px as f32 + 0.5;
                    let fy = py as f32 + 0.5;
                    let w0 = edge(bx, by, cx, cy, fx, fy);
                    let w1 = edge(cx, cy, ax, ay, fx, fy);
                    let w2 = edge(ax, ay, bx, by, fx, fy);
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let idx = py as usize * self.width + px as usize;
                        if depth_at_pos < self.depth_buffer[idx] {
                            self.color_buffer[idx] = packed;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_block_face(
        &mut self,
        pos: Vec3,
        face: BlockFace,
        _color: [f32; 3],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        if face == BlockFace::Bottom && pos.y <= 0.0 {
            return;
        }
        let cam_p = camera.position;
        let is_backfacing = match face {
            BlockFace::Top => cam_p.y < pos.y + 1.0,
            BlockFace::Bottom => cam_p.y > pos.y,
            BlockFace::North => cam_p.z > pos.z,
            BlockFace::South => cam_p.z < pos.z + 1.0,
            BlockFace::West => cam_p.x > pos.x,
            BlockFace::East => cam_p.x < pos.x + 1.0,
        };
        if is_backfacing {
            return;
        }

        let (v0, v1, v2, v3) = match face {
            BlockFace::Top => (
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            BlockFace::Bottom => (
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
            ),
            BlockFace::North => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
            ),
            BlockFace::South => (
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
            ),
            BlockFace::West => (
                Vec3::new(pos.x, pos.y, pos.z),
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            BlockFace::East => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
            ),
        };

        let tex_id = match face {
            BlockFace::Top | BlockFace::Bottom => TextureId::TntTop,
            _ => TextureId::TntSide,
        };

        let t1 = Triangle {
            v0: Vertex::new(v0, (0.0, 0.0), 1.0),
            v1: Vertex::new(v1, (1.0, 0.0), 1.0),
            v2: Vertex::new(v2, (1.0, 1.0), 1.0),
            texture: tex_id,
        };
        let t2 = Triangle {
            v0: Vertex::new(v0, (0.0, 0.0), 1.0),
            v1: Vertex::new(v2, (1.0, 1.0), 1.0),
            v2: Vertex::new(v3, (0.0, 1.0), 1.0),
            texture: tex_id,
        };

        self.raster_triangle(&t1, view_proj);
        self.raster_triangle(&t2, view_proj);
    }

    pub fn draw_tnt_cube(&mut self, pos: Vec3, fuse: f32, camera: &Camera, view_proj: &Mat4) {
        let flash = (fuse * 12.0).sin().abs() * 0.6;
        let white_tint = Vec3::new(flash, flash, flash);

        for face in [
            BlockFace::Top,
            BlockFace::Bottom,
            BlockFace::North,
            BlockFace::South,
            BlockFace::East,
            BlockFace::West,
        ] {
            let color_base = BlockId::Tnt.face_color(face);
            let color = [
                (color_base[0] + white_tint.x).min(1.0),
                (color_base[1] + white_tint.y).min(1.0),
                (color_base[2] + white_tint.z).min(1.0),
            ];

            self.draw_block_face(pos, face, color, camera, view_proj);
        }
    }

    pub fn draw_block_break_cracks(&mut self, target: IVec3, progress: f32, view_proj: &Mat4) {
        if progress <= 0.0 {
            return;
        }
        let stage = ((progress * 10.0).floor() as usize).min(9);
        let crack_tex = TextureId::destroy_stage(stage);

        let eps = 0.003;
        let x0 = target.x as f32 - eps;
        let x1 = target.x as f32 + 1.0 + eps;
        let y0 = target.y as f32 - eps;
        let y1 = target.y as f32 + 1.0 + eps;
        let z0 = target.z as f32 - eps;
        let z1 = target.z as f32 + 1.0 + eps;

        let faces = [
            [Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0)],
            [Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1)],
            [Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1)],
            [Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0)],
            [Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0)],
            [Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1)],
        ];

        for corners in faces {
            let t1 = Triangle {
                v0: Vertex::new(corners[0], (0.0, 1.0), 1.0),
                v1: Vertex::new(corners[1], (1.0, 1.0), 1.0),
                v2: Vertex::new(corners[2], (1.0, 0.0), 1.0),
                texture: crack_tex,
            };
            let t2 = Triangle {
                v0: Vertex::new(corners[0], (0.0, 1.0), 1.0),
                v1: Vertex::new(corners[2], (1.0, 0.0), 1.0),
                v2: Vertex::new(corners[3], (0.0, 0.0), 1.0),
                texture: crack_tex,
            };
            self.raster_triangle(&t1, view_proj);
            self.raster_triangle(&t2, view_proj);
        }
    }
}
