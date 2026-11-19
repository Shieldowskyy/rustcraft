//! 3D DDA voxel traversal (`traverse_voxels`), block interaction raycasting, and camera pullback.
//! Shared between player block targeting, third-person camera collision, and audio occlusion.

use crate::core::math::{IVec3, Vec3};
use crate::features::world::World;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RaycastHit {
    pub block_pos: IVec3,
    pub normal: IVec3,
    pub distance: f32,
}

pub(crate) fn traverse_voxels<T>(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    mut visitor: impl FnMut(IVec3, IVec3, f32) -> Option<T>,
) -> Option<T> {
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    let step_x = if dir.x > 0.0 { 1 } else { -1 };
    let step_y = if dir.y > 0.0 { 1 } else { -1 };
    let step_z = if dir.z > 0.0 { 1 } else { -1 };

    let t_delta_x = if dir.x.abs() > 1e-8 { 1.0 / dir.x.abs() } else { f32::INFINITY };
    let t_delta_y = if dir.y.abs() > 1e-8 { 1.0 / dir.y.abs() } else { f32::INFINITY };
    let t_delta_z = if dir.z.abs() > 1e-8 { 1.0 / dir.z.abs() } else { f32::INFINITY };

    let next_b = |p: f32, s: i32| if s > 0 { p.floor() + 1.0 - p } else { p - p.floor() };
    let mut t_max_x = if dir.x.abs() > 1e-8 { next_b(origin.x, step_x) * t_delta_x } else { f32::INFINITY };
    let mut t_max_y = if dir.y.abs() > 1e-8 { next_b(origin.y, step_y) * t_delta_y } else { f32::INFINITY };
    let mut t_max_z = if dir.z.abs() > 1e-8 { next_b(origin.z, step_z) * t_delta_z } else { f32::INFINITY };

    let mut last_normal = IVec3::new(0, 0, 0);
    let mut traveled = 0.0f32;

    while traveled <= max_dist {
        if let Some(res) = visitor(IVec3::new(x, y, z), last_normal, traveled) {
            return Some(res);
        }
        if t_max_x < t_max_y && t_max_x < t_max_z {
            x += step_x;
            traveled = t_max_x;
            t_max_x += t_delta_x;
            last_normal = IVec3::new(-step_x, 0, 0);
        } else if t_max_y < t_max_z {
            y += step_y;
            traveled = t_max_y;
            t_max_y += t_delta_y;
            last_normal = IVec3::new(0, -step_y, 0);
        } else {
            z += step_z;
            traveled = t_max_z;
            t_max_z += t_delta_z;
            last_normal = IVec3::new(0, 0, -step_z);
        }
    }
    None
}

/// Casts a DDA ray through `world` returning the first targetable voxel within `max_dist`.
pub fn raycast_world(world: &World, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RaycastHit> {
    let dir = dir.normalized();
    if dir.length_sq() < 1e-8 {
        return None;
    }
    traverse_voxels(origin, dir, max_dist, |pos, normal, distance| {
        world
            .get_block(pos)
            .is_targetable()
            .then_some(RaycastHit { block_pos: pos, normal, distance })
    })
}

/// Returns the clamped camera distance pulled in front of solid blocks.
pub fn raycast_camera_collision(world: &World, origin: Vec3, dir: Vec3, max_dist: f32) -> f32 {
    let dir = dir.normalized();
    if dir.length_sq() < 1e-8 {
        return max_dist;
    }
    traverse_voxels(origin, dir, max_dist, |pos, _, traveled| {
        world.get_block(pos).is_solid().then(|| (traveled - 0.25).max(0.25))
    })
    .unwrap_or(max_dist)
}
