//! Backend-independent cloud layer layout shared by software and hardware renderers.
//! Pure geometric quad emission with zero platform `#[cfg]`.

use crate::core::math::Vec3;

pub const CLOUD_Y: f32 = 75.0;
pub const CLOUD_THICKNESS: f32 = 4.0;
pub const CLOUD_CELL_SIZE: f32 = 12.0;
pub const CLOUD_SCROLL_SPEED: f32 = 1.5;
/// Cloud layer is skipped when the player is further than this vertically.
pub const CLOUD_MAX_VERTICAL_DISTANCE: f32 = 100.0;

/// Returns whether the cloud grid cell `(gx, gz)` contains a cloud.
#[inline]
pub fn is_cloud_cell(gx: i32, gz: i32) -> bool {
    let hx = gx.wrapping_mul(374761393i32);
    let hz = gz.wrapping_mul(668265263i32);
    let h = (hx ^ hz ^ hx.wrapping_mul(hz >> 3)) as u32;
    let fx = gx as f32 * 0.16;
    let fz = gz as f32 * 0.16;
    let wave = (fx.sin() + fz.cos() + (fx * 0.7 + fz * 0.5).sin()) * 0.35;
    let r = ((h >> 16) % 100) as f32 / 100.0;
    (wave + r * 0.35) > 0.08
}

/// Grid cell containing the player, accounting for the horizontal cloud scroll.
#[inline]
pub fn cloud_origin_cell(player_pos: Vec3, time: f32) -> (i32, i32) {
    let scroll = time * CLOUD_SCROLL_SPEED;
    (
        ((player_pos.x - scroll) / CLOUD_CELL_SIZE).floor() as i32,
        (player_pos.z / CLOUD_CELL_SIZE).floor() as i32,
    )
}

/// Emits the faces of cloud cell `(gx, gz)` that face the player and are not hidden by a
/// neighbouring cloud, as `(p0, p1, p2, p3, shade)` quads.
pub fn emit_cloud_faces<F>(gx: i32, gz: i32, player_pos: Vec3, time: f32, mut emit: F)
where
    F: FnMut(Vec3, Vec3, Vec3, Vec3, f32),
{
    if !is_cloud_cell(gx, gz) {
        return;
    }
    let scroll = time * CLOUD_SCROLL_SPEED;
    let (y_bottom, y_top) = (CLOUD_Y, CLOUD_Y + CLOUD_THICKNESS);
    let x_min = gx as f32 * CLOUD_CELL_SIZE + scroll;
    let x_max = x_min + CLOUD_CELL_SIZE;
    let z_min = gz as f32 * CLOUD_CELL_SIZE;
    let z_max = z_min + CLOUD_CELL_SIZE;

    let p000 = Vec3::new(x_min, y_bottom, z_min);
    let p100 = Vec3::new(x_max, y_bottom, z_min);
    let p001 = Vec3::new(x_min, y_bottom, z_max);
    let p101 = Vec3::new(x_max, y_bottom, z_max);
    let p010 = Vec3::new(x_min, y_top, z_min);
    let p110 = Vec3::new(x_max, y_top, z_min);
    let p011 = Vec3::new(x_min, y_top, z_max);
    let p111 = Vec3::new(x_max, y_top, z_max);

    if player_pos.y >= y_top {
        emit(p010, p011, p111, p110, 1.00);
    }
    if player_pos.y <= y_bottom {
        emit(p001, p000, p100, p101, 0.70);
    }
    if player_pos.z <= z_min && !is_cloud_cell(gx, gz - 1) {
        emit(p100, p000, p010, p110, 0.85);
    }
    if player_pos.z >= z_max && !is_cloud_cell(gx, gz + 1) {
        emit(p001, p101, p111, p011, 0.85);
    }
    if player_pos.x <= x_min && !is_cloud_cell(gx - 1, gz) {
        emit(p000, p001, p011, p010, 0.78);
    }
    if player_pos.x >= x_max && !is_cloud_cell(gx + 1, gz) {
        emit(p101, p100, p110, p111, 0.78);
    }
}
