//! Procedural sun, moon, star dome, and 3D cloud layer software rendering.
//! Zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::meshing::Vertex;
use crate::features::player::Camera;
use crate::features::rendering::clouds::{
    CLOUD_CELL_SIZE, CLOUD_MAX_VERTICAL_DISTANCE, CLOUD_SCROLL_SPEED, CLOUD_THICKNESS, CLOUD_Y,
    cloud_origin_cell, emit_cloud_faces, is_cloud_cell,
};

use super::clip::{
    clip_near_ao, cloud_column_visible, edge, pack_color, project_to_screen_ao, to_clip_ao,
    unpack_color,
};
use super::types::Renderer;

impl Renderer {
    pub fn draw_clouds(
        &mut self,
        player_pos: Vec3,
        view_proj: &Mat4,
        time: f32,
        rain_strength: f32,
        thunder_strength: f32,
    ) {
        let (y_bottom, y_top) = (CLOUD_Y, CLOUD_Y + CLOUD_THICKNESS);

        if (player_pos.y - CLOUD_Y).abs() > CLOUD_MAX_VERTICAL_DISTANCE {
            return;
        }

        let cloud_size = CLOUD_CELL_SIZE;
        let scroll = time * CLOUD_SCROLL_SPEED;
        let (cx, cz) = cloud_origin_cell(player_pos, time);

        let alpha = 0.80f32;

        for dz in -7..=7 {
            for dx in -7..=7 {
                let gx = cx + dx;
                let gz = cz + dz;

                let x_min = gx as f32 * cloud_size + scroll;
                let x_max = x_min + cloud_size;
                let z_min = gz as f32 * cloud_size;
                let z_max = z_min + cloud_size;

                let center_x = (x_min + x_max) * 0.5;
                let center_z = (z_min + z_max) * 0.5;
                let dist_sq =
                    (center_x - player_pos.x).powi(2) + (center_z - player_pos.z).powi(2);
                if dist_sq > (7.5 * cloud_size).powi(2) {
                    continue;
                }

                if !cloud_column_visible(x_min, x_max, y_bottom, y_top, z_min, z_max, view_proj) {
                    continue;
                }

                if !is_cloud_cell(gx, gz) {
                    continue;
                }

                let draw_quad = |renderer: &mut Renderer,
                                 v0: Vec3,
                                 v1: Vec3,
                                 v2: Vec3,
                                 v3: Vec3,
                                 shade: f32,
                                 view_proj: &Mat4| {
                    let mut emit_triangle = |ta: Vec3, tb: Vec3, tc: Vec3| {
                        let c0 = to_clip_ao(Vertex::new(ta, (0.0, 0.0), shade), view_proj);
                        let c1 = to_clip_ao(Vertex::new(tb, (0.0, 0.0), shade), view_proj);
                        let c2 = to_clip_ao(Vertex::new(tc, (0.0, 0.0), shade), view_proj);

                        let (clipped, clipped_len) = clip_near_ao(&[c0, c1, c2]);
                        if clipped_len < 3 {
                            return;
                        }

                        let width = renderer.width;
                        let height = renderer.height;

                        for i in 1..clipped_len - 1 {
                            let sa = project_to_screen_ao(&clipped[0], width, height);
                            let sb = project_to_screen_ao(&clipped[i], width, height);
                            let sc = project_to_screen_ao(&clipped[i + 1], width, height);

                            let area = edge(sa.x, sa.y, sb.x, sb.y, sc.x, sc.y);
                            if area <= 0.0 {
                                continue;
                            }

                            let min_x = sa.x.min(sb.x).min(sc.x).floor().max(0.0) as i32;
                            let max_x =
                                sa.x.max(sb.x).max(sc.x).ceil().min(width as f32 - 1.0) as i32;
                            let min_y = sa.y.min(sb.y).min(sc.y).floor().max(0.0) as i32;
                            let max_y =
                                sa.y.max(sb.y).max(sc.y).ceil().min(height as f32 - 1.0) as i32;

                            if min_x > max_x || min_y > max_y {
                                continue;
                            }

                            let inv_area = 1.0 / area;
                            let dx0 = sc.x - sb.x;
                            let dy0 = sc.y - sb.y;
                            let dx1 = sa.x - sc.x;
                            let dy1 = sa.y - sc.y;
                            let dx2 = sb.x - sa.x;
                            let dy2 = sb.y - sa.y;

                            let fx0 = min_x as f32 + 0.5;
                            let fy0 = min_y as f32 + 0.5;
                            let mut w0_row = edge(sb.x, sb.y, sc.x, sc.y, fx0, fy0);
                            let mut w1_row = edge(sc.x, sc.y, sa.x, sa.y, fx0, fy0);
                            let mut w2_row = edge(sa.x, sa.y, sb.x, sb.y, fx0, fy0);

                            for py in min_y..=max_y {
                                let mut w0 = w0_row;
                                let mut w1 = w1_row;
                                let mut w2 = w2_row;
                                let row_offset = py as usize * width;

                                for px in min_x..=max_x {
                                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                                        let l0 = w0 * inv_area;
                                        let l1 = w1 * inv_area;
                                        let l2 = w2 * inv_area;

                                        let inv_w =
                                            l0 * sa.inv_w + l1 * sb.inv_w + l2 * sc.inv_w;
                                        if inv_w > 0.0 {
                                            let depth = 1.0 / inv_w;
                                            let idx = row_offset + px as usize;
                                            if depth < renderer.depth_buffer[idx] {
                                                let daylight =
                                                    renderer.current_daylight.clamp(0.0, 1.0);
                                                let fog_col = renderer.current_fog_color;
                                                let day_cloud = [0.94, 0.95, 0.98];
                                                let night_cloud = [0.07, 0.09, 0.16];
                                                let storm_factor = (rain_strength * 0.50
                                                    + thunder_strength * 0.40)
                                                    .clamp(0.0, 0.85);
                                                let storm_cloud = [0.15, 0.17, 0.22];
                                                let cloud_base = [
                                                    (night_cloud[0]
                                                        + (day_cloud[0] - night_cloud[0])
                                                            * daylight
                                                        + (fog_col[0] - 0.72).max(0.0) * 0.35)
                                                        * (1.0 - storm_factor)
                                                        + storm_cloud[0] * storm_factor,
                                                    (night_cloud[1]
                                                        + (day_cloud[1] - night_cloud[1])
                                                            * daylight
                                                        + (fog_col[1] - 0.84).max(0.0) * 0.35)
                                                        * (1.0 - storm_factor)
                                                        + storm_cloud[1] * storm_factor,
                                                    (night_cloud[2]
                                                        + (day_cloud[2] - night_cloud[2])
                                                            * daylight)
                                                        * (1.0 - storm_factor)
                                                        + storm_cloud[2] * storm_factor,
                                                ];

                                                let ao = (l0 * sa.ao_over_w
                                                    + l1 * sb.ao_over_w
                                                    + l2 * sc.ao_over_w)
                                                    * depth;
                                                let cloud_r =
                                                    (cloud_base[0] * ao).clamp(0.0, 1.0);
                                                let cloud_g =
                                                    (cloud_base[1] * ao).clamp(0.0, 1.0);
                                                let cloud_b =
                                                    (cloud_base[2] * ao).clamp(0.0, 1.0);

                                                let fog_dist = if renderer.fog_enabled {
                                                    (renderer.render_distance as f32 * 16.0)
                                                        .max(32.0)
                                                } else {
                                                    0.0
                                                };
                                                let fog_t = if fog_dist > 0.0 {
                                                    ((depth - fog_dist * 0.45)
                                                        / (fog_dist * 0.55))
                                                        .clamp(0.0, 1.0)
                                                } else {
                                                    0.0
                                                };
                                                let cr = cloud_r * (1.0 - fog_t)
                                                    + fog_col[0] * fog_t;
                                                let cg = cloud_g * (1.0 - fog_t)
                                                    + fog_col[1] * fog_t;
                                                let cb = cloud_b * (1.0 - fog_t)
                                                    + fog_col[2] * fog_t;

                                                let (br, bg, bb) =
                                                    unpack_color(renderer.color_buffer[idx]);
                                                renderer.color_buffer[idx] = pack_color(
                                                    cr * alpha + br * (1.0 - alpha),
                                                    cg * alpha + bg * (1.0 - alpha),
                                                    cb * alpha + bb * (1.0 - alpha),
                                                );
                                            }
                                        }
                                    }
                                    w0 += dy0;
                                    w1 += dy1;
                                    w2 += dy2;
                                }
                                w0_row -= dx0;
                                w1_row -= dx1;
                                w2_row -= dx2;
                            }
                        }
                    };

                    emit_triangle(v0, v1, v2);
                    emit_triangle(v0, v2, v3);
                };

                emit_cloud_faces(gx, gz, player_pos, time, |v0, v1, v2, v3, shade| {
                    draw_quad(self, v0, v1, v2, v3, shade, view_proj);
                });
            }
        }
    }

    pub fn draw_celestials(&mut self, camera: &Camera, view_proj: &Mat4, day_fraction: f32) {
        let angle = day_fraction * std::f32::consts::TAU;
        let sun_dir = Vec3::new(angle.sin(), angle.cos(), -0.2).normalized();
        let moon_dir = -sun_dir;

        if sun_dir.y > -0.25 {
            let sun_pos = camera.position + sun_dir * 120.0;
            self.draw_celestial_billboard(sun_pos, 22.0, [1.0, 0.95, 0.65], camera, view_proj);
        }

        if moon_dir.y > -0.25 {
            let moon_pos = camera.position + moon_dir * 120.0;
            self.draw_celestial_billboard(moon_pos, 16.0, [0.92, 0.95, 1.0], camera, view_proj);
        }

        let daylight = self.current_daylight;
        if daylight < 0.65 {
            let star_alpha = ((0.65 - daylight) / 0.50).clamp(0.0, 1.0);
            self.draw_stars(camera, view_proj, star_alpha);
        }
    }

    fn draw_celestial_billboard(
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

        let depth_val = 5000.0f32;
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
                        if depth_val < self.depth_buffer[idx] {
                            self.color_buffer[idx] = packed;
                            self.depth_buffer[idx] = depth_val;
                        }
                    }
                }
            }
        }
    }

    fn draw_stars(&mut self, camera: &Camera, view_proj: &Mat4, star_alpha: f32) {
        if star_alpha <= 0.01 {
            return;
        }
        const STAR_COUNT: usize = 64;
        let width = self.width;
        let height = self.height;

        for i in 0..STAR_COUNT {
            let fi = i as f32;
            let phi = fi * 2.3999632;
            let y = 0.15 + (fi / STAR_COUNT as f32) * 0.85;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let x = phi.cos() * r;
            let z = phi.sin() * r;

            let star_dir = Vec3::new(x, y, z).normalized();
            let star_world = camera.position + star_dir * 100.0;
            let c = view_proj.transform_point(star_world);
            if c[3] <= 0.1 {
                continue;
            }
            let inv_w = 1.0 / c[3];
            let sx = ((c[0] * inv_w * 0.5 + 0.5) * width as f32) as i32;
            let sy = ((1.0 - (c[1] * inv_w * 0.5 + 0.5)) * height as f32) as i32;

            if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                let idx = sy as usize * width + sx as usize;
                if 5000.0 < self.depth_buffer[idx] {
                    let twinkle =
                        0.7 + 0.3 * ((fi * 1.7 + self.total_time * 2.5).sin() * 0.5 + 0.5);
                    let intensity = star_alpha * twinkle;
                    let star_col = pack_color(intensity * 0.95, intensity * 0.95, intensity);
                    self.color_buffer[idx] = star_col;
                }
            }
        }
    }
}
