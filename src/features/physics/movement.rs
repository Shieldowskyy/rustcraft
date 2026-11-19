//! Player AABB movement resolver, crouch edge-clamping, auto-stepping, and penetration recovery.
//! Integrates displacement vectors against voxel geometry and updates `eye_pos` / `vertical_velocity`.

use crate::core::math::Vec3;
use crate::features::world::{Chunk, World, CHUNK_SIZE_Y};

use super::aabb::box_collides;
use super::constants::{PLAYER_HALF_WIDTH, STEP_HEIGHT};

/// Checks if there is solid ground directly beneath the player's feet (sneak anti-fall check).
fn has_ground_below(world: &World, feet: Vec3, half: f32) -> bool {
    let min = Vec3::new(feet.x - half, feet.y - 0.5, feet.z - half);
    let max = Vec3::new(feet.x + half, feet.y - 0.01, feet.z + half);
    box_collides(world, min, max)
}

/// Restricts movement along an axis while crouching using 6-step binary search.
fn clamp_crouch_axis(
    world: &World,
    feet: Vec3,
    delta: f32,
    is_x: bool,
    half: f32,
) -> f32 {
    if delta == 0.0 {
        return 0.0;
    }

    let mut full_feet = feet;
    if is_x { full_feet.x += delta; } else { full_feet.z += delta; }
    if has_ground_below(world, full_feet, half) {
        return delta;
    }

    let mut low = 0.0f32;
    let mut high = delta;

    for _ in 0..6 {
        let mid = (low + high) * 0.5;
        let mut test_feet = feet;
        if is_x { test_feet.x += mid; } else { test_feet.z += mid; }
        if has_ground_below(world, test_feet, half) {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

/// Pushes the player out of solid blocks towards the closest open edge or upwards.
pub fn resolve_block_penetration(
    world: &World,
    feet: &mut Vec3,
    half: f32,
    height: f32,
) -> bool {
    let aabb_min = Vec3::new(feet.x - half, feet.y, feet.z - half);
    let aabb_max = Vec3::new(feet.x + half, feet.y + height, feet.z + half);

    if !box_collides(world, aabb_min, aabb_max) {
        return false;
    }

    let x0 = aabb_min.x.floor() as i32;
    let x1 = (aabb_max.x - 1e-4).floor() as i32;
    let y0 = (aabb_min.y.floor() as i32).max(0);
    let y1 = ((aabb_max.y - 1e-4).floor() as i32).min(CHUNK_SIZE_Y - 1);
    let z0 = aabb_min.z.floor() as i32;
    let z1 = (aabb_max.z - 1e-4).floor() as i32;

    if y0 > y1 {
        return false;
    }

    let center_x = feet.x;
    let center_z = feet.z;

    let mut best_push_x = 0.0f32;
    let mut best_push_z = 0.0f32;
    let mut min_push_dist = f32::MAX;
    let mut ground_push_y = 0.0f32;
    let mut solid_count = 0;

    let mut last_cx = i32::MIN;
    let mut last_cz = i32::MIN;
    let mut cached_chunk: Option<&Chunk> = None;

    for x in x0..=x1 {
        for z in z0..=z1 {
            let (cx, cz, lx, lz) = World::world_to_chunk(x, z);
            if cx != last_cx || cz != last_cz {
                last_cx = cx;
                last_cz = cz;
                cached_chunk = world.chunks.get(&(cx, cz));
            }

            if let Some(chunk) = cached_chunk {
                for y in y0..=y1 {
                    if chunk.get_local(lx, y, lz).is_solid() {
                        solid_count += 1;
                        let bx_min = x as f32;
                        let bx_max = x as f32 + 1.0;
                        let bz_min = z as f32;
                        let bz_max = z as f32 + 1.0;
                        let by_max = y as f32 + 1.0;

                        let overlap_y = by_max - feet.y;
                        if overlap_y > 0.0 && overlap_y <= 1.35 {
                            ground_push_y = ground_push_y.max(overlap_y);
                        }

                        let d_left = (center_x + half) - bx_min;
                        let d_right = bx_max - (center_x - half);
                        let d_front = (center_z + half) - bz_min;
                        let d_back = bz_max - (center_z - half);

                        let push_x = if d_left < d_right { -d_left } else { d_right };
                        let push_z = if d_front < d_back { -d_front } else { d_back };

                        if push_x.abs() < min_push_dist && push_x.abs() > 0.0001 {
                            min_push_dist = push_x.abs();
                            best_push_x = push_x;
                            best_push_z = 0.0;
                        }
                        if push_z.abs() < min_push_dist && push_z.abs() > 0.0001 {
                            min_push_dist = push_z.abs();
                            best_push_z = push_z;
                            best_push_x = 0.0;
                        }
                    }
                }
            }
        }
    }

    if solid_count > 0 {
        if ground_push_y > 0.001 && solid_count >= 2 {
            let mut test_feet = *feet;
            test_feet.y += ground_push_y + 0.05;
            let (min, max) = (
                Vec3::new(test_feet.x - half, test_feet.y, test_feet.z - half),
                Vec3::new(test_feet.x + half, test_feet.y + height, test_feet.z + half),
            );
            if !box_collides(world, min, max) {
                feet.y = test_feet.y;
                return true;
            }
        }

        if best_push_x.abs() > 0.0001 {
            feet.x += best_push_x + best_push_x.signum() * 0.01;
            return true;
        }
        if best_push_z.abs() > 0.0001 {
            feet.z += best_push_z + best_push_z.signum() * 0.01;
            return true;
        }
    }

    false
}

/// Advances the player by `motion`, resolving AABB collisions, 0.5-block auto-steps, and crouching.
/// Returns `true` if resting on solid ground after the vertical pass.
pub fn move_player_with_collision(
    world: &World,
    eye_pos: &mut Vec3,
    motion: Vec3,
    vertical_velocity: &mut f32,
    player_eye_offset: f32,
    player_height: f32,
    is_crouching: bool,
) -> bool {
    let mut feet = Vec3::new(eye_pos.x, eye_pos.y - player_eye_offset, eye_pos.z);
    let half = PLAYER_HALF_WIDTH;
    let height = player_height;

    resolve_block_penetration(world, &mut feet, half, height);

    let aabb = |feet_pos: Vec3| -> (Vec3, Vec3) {
        (
            Vec3::new(feet_pos.x - half, feet_pos.y, feet_pos.z - half),
            Vec3::new(feet_pos.x + half, feet_pos.y + height, feet_pos.z + half),
        )
    };

    let was_on_ground = has_ground_below(world, feet, half);

    if motion.x != 0.0 {
        let dx = if is_crouching && was_on_ground {
            clamp_crouch_axis(world, feet, motion.x, true, half)
        } else {
            motion.x
        };

        if dx != 0.0 {
            feet.x += dx;
            if box_collides(world, aabb(feet).0, aabb(feet).1) {
                let mut stepped = feet;
                stepped.y += STEP_HEIGHT;
                if !box_collides(world, aabb(stepped).0, aabb(stepped).1) {
                    feet = stepped;
                } else {
                    feet.x -= dx;
                }
            }
        }
    }

    if motion.z != 0.0 {
        let dz = if is_crouching && was_on_ground {
            clamp_crouch_axis(world, feet, motion.z, false, half)
        } else {
            motion.z
        };

        if dz != 0.0 {
            feet.z += dz;
            if box_collides(world, aabb(feet).0, aabb(feet).1) {
                let mut stepped = feet;
                stepped.y += STEP_HEIGHT;
                if !box_collides(world, aabb(stepped).0, aabb(stepped).1) {
                    feet = stepped;
                } else {
                    feet.z -= dz;
                }
            }
        }
    }

    let mut on_ground = false;
    feet.y += motion.y;
    {
        let (min, max) = aabb(feet);
        if box_collides(world, min, max) {
            feet.y -= motion.y;
            if motion.y < 0.0 {
                on_ground = true;
            }
            *vertical_velocity = 0.0;
        }
    }

    resolve_block_penetration(world, &mut feet, half, height);

    eye_pos.x = feet.x;
    eye_pos.y = feet.y + player_eye_offset;
    eye_pos.z = feet.z;

    on_ground
}
