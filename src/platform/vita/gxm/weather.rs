//! 3D GPU precipitation (rain streaks, ground splashes, swaying snowflakes) and lightning bolts.
//! Matches `features::rendering::software::weather` volume, timing, biome checks, and palette.

#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::features::world::{LightningStrike, PrecipitationKind, World};
use crate::graphics::camera::Camera;

use super::entities::EntityBatch;

pub(super) fn append_precipitation(
    batch: &mut EntityBatch,
    world: &World,
    camera: &Camera,
    time: f32,
) {
    let rain_strength = world.weather.rain_strength;
    if rain_strength <= 0.01 {
        return;
    }

    let cam_pos = camera.position;
    let right_flat = camera.right_flat();
    let cam_right = camera.right();
    let cam_up = camera.up();

    let radius = 9i32;
    let r_sq = radius * radius;
    let cx = cam_pos.x.floor() as i32;
    let cz = cam_pos.z.floor() as i32;

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
            let y_max = (cam_pos.y + 9.0).min(64.0);
            let y_min = surface_y.max(cam_pos.y - 6.0);
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
                        let p2 = Vec3::new(col_x + 0.04, (drop_y - drop_len).max(y_min), col_z + 0.04);
                        let side = right_flat * 0.016;
                        let alpha = (0.42 * rain_strength).clamp(0.08, 0.65);
                        batch.push_untextured_quad(
                            p2 - side, p2 + side, p1 + side, p1 - side,
                            [0.55, 0.68, 0.88, alpha],
                        );
                        if (drop_y - y_min).abs() < 0.4 {
                            let s = 0.045;
                            let sy = Vec3::new(0.0, s, 0.0);
                            let sx = right_flat * s;
                            batch.push_untextured_quad(
                                p2 - sx, p2 + sx, p2 + sx + sy, p2 - sx + sy,
                                [0.68, 0.78, 0.95, 0.85],
                            );
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
                        let h = 0.045;
                        let alpha = (0.75 * rain_strength).clamp(0.2, 0.88);
                        batch.push_untextured_quad(
                            p - cam_right * h - cam_up * h,
                            p + cam_right * h - cam_up * h,
                            p + cam_right * h + cam_up * h,
                            p - cam_right * h + cam_up * h,
                            [0.94, 0.96, 1.0, alpha],
                        );
                    }
                }
                PrecipitationKind::None => {}
            }
        }
    }
}

pub(super) fn append_lightning_bolts(
    batch: &mut EntityBatch,
    active_bolts: &[LightningStrike],
    camera: &Camera,
) {
    let fallback_right = camera.right_flat();
    for strike in active_bolts {
        let fade = strike.alpha().clamp(0.0, 1.0);
        if fade <= 0.005 {
            continue;
        }
        let glow_alpha = (fade * 0.5).clamp(0.0, 1.0);
        let segments = strike.generate_segments();
        for &(p0, p1) in &segments {
            let seg = p1 - p0;
            if seg.length() <= 1e-4 {
                continue;
            }
            let mid = (p0 + p1) * 0.5;
            let to_mid = mid - camera.position;
            let cross = seg.cross(to_mid);
            let perp = if cross.length() > 1e-4 {
                cross.normalized()
            } else {
                fallback_right
            };

            let g = perp * 0.24;
            batch.push_untextured_quad(p0 - g, p0 + g, p1 + g, p1 - g, [0.45, 0.70, 1.0, glow_alpha]);

            let c = perp * 0.08;
            batch.push_untextured_quad(p0 - c, p0 + c, p1 + c, p1 - c, [1.0, 1.0, 1.0, fade]);
        }
    }
}
