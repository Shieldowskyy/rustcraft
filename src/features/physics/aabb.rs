//! Voxel AABB collision probes, ladder overlap tests, and ray-AABB intersection.
//! Pure geometric queries against loaded `World` chunks without mutating game state.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::{World, CHUNK_SIZE_Y};

use super::constants::PLAYER_HALF_WIDTH;

/// Checks whether a block at grid coordinate `place_pos` intersects the player's bounding box.
pub fn block_intersects_player(place_pos: IVec3, eye_pos: Vec3, eye_offset: f32, height: f32) -> bool {
    let half = PLAYER_HALF_WIDTH;
    let feet_y = eye_pos.y - eye_offset;
    let p_min_x = eye_pos.x - half;
    let p_max_x = eye_pos.x + half;
    let p_min_y = feet_y;
    let p_max_y = feet_y + height;
    let p_min_z = eye_pos.z - half;
    let p_max_z = eye_pos.z + half;

    let b_min_x = place_pos.x as f32;
    let b_max_x = (place_pos.x + 1) as f32;
    let b_min_y = place_pos.y as f32;
    let b_max_y = (place_pos.y + 1) as f32;
    let b_min_z = place_pos.z as f32;
    let b_max_z = (place_pos.z + 1) as f32;

    p_min_x < b_max_x
        && p_max_x > b_min_x
        && p_min_y < b_max_y
        && p_max_y > b_min_y
        && p_min_z < b_max_z
        && p_max_z > b_min_z
}

/// Returns `true` if any solid part of the entity's AABB intersects a `BlockId::Ladder`.
pub fn player_touches_ladder(world: &World, feet: Vec3, half: f32, height: f32) -> bool {
    let eps = 1e-4;
    let x0 = (feet.x - half).floor() as i32;
    let x1 = (feet.x + half - eps).floor() as i32;
    let y0 = feet.y.floor().max(0.0) as i32;
    let y1 = (feet.y + height - eps).floor() as i32;
    let z0 = (feet.z - half).floor() as i32;
    let z1 = (feet.z + half - eps).floor() as i32;

    (x0..=x1).any(|x| {
        (y0..=y1).any(|y| {
            (z0..=z1).any(|z| world.get_block(IVec3::new(x, y, z)) == BlockId::Ladder)
        })
    })
}

/// Checks whether the world-space AABB `[min, max]` collides with any solid or half-block voxel.
pub fn box_collides(world: &World, min: Vec3, max: Vec3) -> bool {
    let eps = 1e-4;
    let (x0, x1) = (min.x.floor() as i32, (max.x - eps).floor() as i32);
    let (y0, y1) = ((min.y.floor() as i32).max(0), ((max.y - eps).floor() as i32).min(CHUNK_SIZE_Y - 1));
    let (z0, z1) = (min.z.floor() as i32, (max.z - eps).floor() as i32);
    if y0 > y1 {
        return false;
    }

    let (mut last_cx, mut last_cz, mut cached_chunk) = (i32::MIN, i32::MIN, None);
    for x in x0..=x1 {
        for z in z0..=z1 {
            let (cx, cz, lx, lz) = World::world_to_chunk(x, z);
            if cx != last_cx || cz != last_cz {
                (last_cx, last_cz) = (cx, cz);
                cached_chunk = world.chunks.get(&(cx, cz));
            }
            if let Some(chunk) = cached_chunk {
                for y in y0..=y1 {
                    let b = chunk.get_local(lx, y, lz);
                    if b.is_solid() && (!b.is_half_block() || min.y < y as f32 + 0.5) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Ray-AABB intersection test returning the distance parameter `t` along ray if hit.
pub fn ray_aabb_intersect(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = (min.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    let mut t_max = (max.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    if t_min > t_max { std::mem::swap(&mut t_min, &mut t_max); }

    let mut ty_min = (min.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    let mut ty_max = (max.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    if ty_min > ty_max { std::mem::swap(&mut ty_min, &mut ty_max); }

    if (t_min > ty_max) || (ty_min > t_max) {
        return None;
    }
    if ty_min > t_min { t_min = ty_min; }
    if ty_max < t_max { t_max = ty_max; }

    let mut tz_min = (min.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    let mut tz_max = (max.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    if tz_min > tz_max { std::mem::swap(&mut tz_min, &mut tz_max); }

    if (t_min > tz_max) || (tz_min > t_max) {
        return None;
    }
    if tz_min > t_min { t_min = tz_min; }
    if tz_max < t_max { t_max = tz_max; }

    if t_max < 0.0 {
        return None;
    }

    Some(if t_min >= 0.0 { t_min } else { t_max })
}
