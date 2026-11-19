//! Software rain/snow precipitation and 3D lightning bolts rendering.
//! Zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::player::Camera;
use crate::features::world::{LightningStrike, PrecipitationKind, World};

use super::clip::{pack_color, unpack_color};
use super::types::Renderer;

impl Renderer {
    /// Renders a depth-tested alpha-blended line in screen space.
    #[inline]
    pub fn draw_screen_line_blended(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        x1: i32,
        y1: i32,
        depth: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        let w = self.width as i32;
        let h = self.height as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let [fr, fg, fb] = color;
        let inv_alpha = 1.0 - alpha;

        loop {
            if x0 >= 0 && x0 < w && y0 >= 0 && y0 < h {
                let idx = y0 as usize * self.width + x0 as usize;
                if depth < self.depth_buffer[idx] {
                    let (br, bg, bb) = unpack_color(self.color_buffer[idx]);
                    self.color_buffer[idx] = pack_color(
                        br * inv_alpha + fr * alpha,
                        bg * inv_alpha + fg * alpha,
                        bb * inv_alpha + fb * alpha,
                    );
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Renders rain streaks and snowflakes within a frustum volume around the player.
    pub fn draw_precipitation(
        &mut self,
        world: &World,
        camera: &Camera,
        view_proj: &Mat4,
        time: f32,
    ) {
        let rain_strength = world.weather.rain_strength;
        if rain_strength <= 0.01 {
            return;
        }

        let cam_pos = camera.position;
        let (cam_x, cam_y, cam_z) = (cam_pos.x, cam_pos.y, cam_pos.z);

        let radius = 9;
        let r_sq = radius * radius;
        let cx = cam_x.floor() as i32;
        let cz = cam_z.floor() as i32;

        let w_f32 = self.width as f32;
        let h_f32 = self.height as f32;

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dz * dz > r_sq {
                    continue;
                }

                let wx = cx + dx;
                let wz = cz + dz;
                let biome = world.get_biome_at(wx, wz);
                let precip = world.weather.precipitation_at(biome);
                if precip == PrecipitationKind::None {
                    continue;
                }

                let surface_y = world.get_highest_block_y(wx, wz) as f32;
                let y_max = (cam_y + 9.0).min(64.0);
                let y_min = surface_y.max(cam_y - 6.0);
                if y_max <= y_min {
                    continue;
                }

                let hash = (wx.wrapping_mul(374761393) ^ wz.wrapping_mul(668265263)) as u32;
                let offset_phase = ((hash & 0xFF) as f32) / 256.0;
                let col_x = wx as f32 + (((hash >> 8) & 0x0F) as f32 + 0.5) / 16.0;
                let col_z = wz as f32 + (((hash >> 12) & 0x0F) as f32 + 0.5) / 16.0;

                match precip {
                    PrecipitationKind::Rain => {
                        let fall_speed = 28.0;
                        let span = 7.0;
                        let drop_y = y_max - ((time * fall_speed + offset_phase * span) % span);

                        if drop_y >= y_min && drop_y <= y_max {
                            let drop_len = 0.65;
                            let p1 = Vec3::new(col_x, drop_y, col_z);
                            let p2 = Vec3::new(
                                col_x + 0.04,
                                (drop_y - drop_len).max(y_min),
                                col_z + 0.04,
                            );

                            let c1 = view_proj.transform_point(p1);
                            let c2 = view_proj.transform_point(p2);
                            if c1[3] > 0.1 && c2[3] > 0.1 {
                                let sx1 = (c1[0] / c1[3] * 0.5 + 0.5) * w_f32;
                                let sy1 = (1.0 - (c1[1] / c1[3] * 0.5 + 0.5)) * h_f32;
                                let sx2 = (c2[0] / c2[3] * 0.5 + 0.5) * w_f32;
                                let sy2 = (1.0 - (c2[1] / c2[3] * 0.5 + 0.5)) * h_f32;

                                let depth = c1[3].min(c2[3]);
                                let alpha = (0.42 * rain_strength).clamp(0.08, 0.65);
                                self.draw_screen_line_blended(
                                    sx1 as i32,
                                    sy1 as i32,
                                    sx2 as i32,
                                    sy2 as i32,
                                    depth,
                                    [0.55, 0.68, 0.88],
                                    alpha,
                                );

                                if (drop_y - y_min).abs() < 0.4 {
                                    let splash_y = (sy2 as i32).clamp(0, self.height as i32 - 1);
                                    let splash_x = (sx2 as i32).clamp(1, self.width as i32 - 2);
                                    let idx = splash_y as usize * self.width + splash_x as usize;
                                    if depth < self.depth_buffer[idx] {
                                        self.color_buffer[idx] = pack_color(0.68, 0.78, 0.95);
                                        self.color_buffer[idx - 1] = pack_color(0.55, 0.68, 0.88);
                                    }
                                }
                            }
                        }
                    }
                    PrecipitationKind::Snow => {
                        let fall_speed = 3.6;
                        let span = 6.0;
                        let drop_y = y_max - ((time * fall_speed + offset_phase * span) % span);

                        if drop_y >= y_min && drop_y <= y_max {
                            let sway_x = ((time * 2.2 + offset_phase * 6.28).sin()) * 0.30;
                            let sway_z = ((time * 1.8 + offset_phase * 6.28).cos()) * 0.30;
                            let p = Vec3::new(col_x + sway_x, drop_y, col_z + sway_z);

                            let c = view_proj.transform_point(p);
                            if c[3] > 0.1 {
                                let sx = (c[0] / c[3] * 0.5 + 0.5) * w_f32;
                                let sy = (1.0 - (c[1] / c[3] * 0.5 + 0.5)) * h_f32;
                                let px = sx as i32;
                                let py = sy as i32;

                                if px >= 0
                                    && px < self.width as i32
                                    && py >= 0
                                    && py < self.height as i32
                                {
                                    let idx = py as usize * self.width + px as usize;
                                    let depth = c[3];
                                    if depth < self.depth_buffer[idx] {
                                        let (br, bg, bb) = unpack_color(self.color_buffer[idx]);
                                        let alpha = (0.75 * rain_strength).clamp(0.2, 0.88);
                                        let inv = 1.0 - alpha;
                                        self.color_buffer[idx] = pack_color(
                                            br * inv + 0.94 * alpha,
                                            bg * inv + 0.96 * alpha,
                                            bb * inv + 1.0 * alpha,
                                        );
                                        if depth < 5.0
                                            && px + 1 < self.width as i32
                                            && py + 1 < self.height as i32
                                        {
                                            let idx2 = idx + 1;
                                            if depth < self.depth_buffer[idx2] {
                                                self.color_buffer[idx2] =
                                                    pack_color(0.92, 0.94, 0.98);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PrecipitationKind::None => {}
                }
            }
        }
    }

    /// Renders dramatic 3D zig-zagging electric lightning bolts and branches.
    pub fn draw_lightning_bolts(&mut self, active_bolts: &[LightningStrike], view_proj: &Mat4) {
        let w_f32 = self.width as f32;
        let h_f32 = self.height as f32;

        for strike in active_bolts {
            let fade = strike.alpha();
            let core_alpha = fade;
            let glow_alpha = (fade * 0.5).clamp(0.0, 1.0);
            let segments = strike.generate_segments();

            for &(p0, p1) in &segments {
                let c0 = view_proj.transform_point(p0);
                let c1 = view_proj.transform_point(p1);

                if c0[3] <= 0.1 || c1[3] <= 0.1 {
                    continue;
                }

                let sx0 = (c0[0] / c0[3] * 0.5 + 0.5) * w_f32;
                let sy0 = (1.0 - (c0[1] / c0[3] * 0.5 + 0.5)) * h_f32;
                let sx1 = (c1[0] / c1[3] * 0.5 + 0.5) * w_f32;
                let sy1 = (1.0 - (c1[1] / c1[3] * 0.5 + 0.5)) * h_f32;

                let depth = c0[3].min(c1[3]);

                self.draw_screen_line_blended(
                    sx0 as i32 + 1,
                    sy0 as i32,
                    sx1 as i32 + 1,
                    sy1 as i32,
                    depth,
                    [0.45, 0.70, 1.0],
                    glow_alpha,
                );
                self.draw_screen_line_blended(
                    sx0 as i32 - 1,
                    sy0 as i32,
                    sx1 as i32 - 1,
                    sy1 as i32,
                    depth,
                    [0.45, 0.70, 1.0],
                    glow_alpha,
                );
                self.draw_screen_line_blended(
                    sx0 as i32,
                    sy0 as i32 + 1,
                    sx1 as i32,
                    sy1 as i32 + 1,
                    depth,
                    [0.45, 0.70, 1.0],
                    glow_alpha,
                );
                self.draw_screen_line_blended(
                    sx0 as i32,
                    sy0 as i32,
                    sx1 as i32,
                    sy1 as i32,
                    depth,
                    [1.0, 1.0, 1.0],
                    core_alpha,
                );
            }
        }
    }
}
