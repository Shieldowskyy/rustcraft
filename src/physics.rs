// Fizyka gracza: stałe grawitacji/skoku/wymiarów oraz kolizje AABB
// pomiędzy graczem a blokami świata.

use crate::math::{IVec3, Vec3};
use crate::world::World;

pub const GRAVITY: f32 = 24.0;
pub const TERMINAL_VELOCITY: f32 = 40.0;
pub const JUMP_SPEED: f32 = 8.5;
pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const STANDING_HEIGHT: f32 = 1.8;
pub const STANDING_EYE_OFFSET: f32 = 1.6;
pub const CROUCH_HEIGHT: f32 = 1.4;
pub const CROUCH_EYE_OFFSET: f32 = 1.25;

pub const BASE_FOV_DEG: f32 = 70.0;
pub const SPRINT_FOV_DEG: f32 = 83.0;
pub const CROUCH_FOV_DEG: f32 = 66.0;

/// Sprawdza, czy blok w `place_pos` nakłada się z hitboxem gracza (używane
/// przy stawianiu bloków, żeby nie można było zamurować się samemu).
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

/// Czy pod stopami gracza (w niewielkim odstępie) jest jakiś blok.
/// Używane, żeby kucający gracz nie mógł zejść z krawędzi.
fn has_ground_below(world: &World, feet: Vec3, half: f32) -> bool {
    let min = Vec3::new(feet.x - half, feet.y - 0.4, feet.z - half);
    let max = Vec3::new(feet.x + half, feet.y + 0.05, feet.z + half);
    world.box_collides(min, max)
}

/// Przesuwa gracza o `motion`, rozwiązując kolizje osobno na każdej osi
/// (X, Z, potem Y), i zwraca, czy po tym ruchu gracz stoi na ziemi.
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

    let aabb = |feet: Vec3| -> (Vec3, Vec3) {
        (
            Vec3::new(feet.x - half, feet.y, feet.z - half),
            Vec3::new(feet.x + half, feet.y + height, feet.z + half),
        )
    };

    feet.x += motion.x;
    if world.box_collides(aabb(feet).0, aabb(feet).1)
        || (is_crouching && !has_ground_below(world, feet, half))
    {
        feet.x -= motion.x;
    }

    feet.z += motion.z;
    if world.box_collides(aabb(feet).0, aabb(feet).1)
        || (is_crouching && !has_ground_below(world, feet, half))
    {
        feet.z -= motion.z;
    }

    let mut on_ground = false;
    feet.y += motion.y;
    {
        let (min, max) = aabb(feet);
        if world.box_collides(min, max) {
            feet.y -= motion.y;
            if motion.y < 0.0 {
                on_ground = true;
            }
            *vertical_velocity = 0.0;
        }
    }

    eye_pos.x = feet.x;
    eye_pos.y = feet.y + player_eye_offset;
    eye_pos.z = feet.z;

    on_ground
}
