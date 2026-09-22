// Fizyka gracza 1:1 z modelem fizyki Minecraft: stałe grawitacji, prędkości
// chodu/biegu/kucania, skoku, auto-step (wchodzenie na stopnie 0.5 bloku)
// oraz krawędziowe blokowanie kucania (anti-fall).

use crate::core::math::{IVec3, Vec3};
use crate::world::World;

pub const GRAVITY: f32 = 22.0;
pub const TERMINAL_VELOCITY: f32 = 50.0;
pub const JUMP_SPEED: f32 = 7.4;
pub const SPRINT_JUMP_BOOST: f32 = 1.8;

pub const WALK_SPEED: f32 = 4.5;
pub const SPRINT_SPEED: f32 = 8.5;
pub const SNEAK_SPEED: f32 = 1.8;
pub const SWIM_SPEED: f32 = 2.8;
pub const FLY_SPEED: f32 = 12.0;

pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const STANDING_HEIGHT: f32 = 1.8;
pub const STANDING_EYE_OFFSET: f32 = 1.62;
pub const CROUCH_HEIGHT: f32 = 1.5;
pub const CROUCH_EYE_OFFSET: f32 = 1.27;
pub const STEP_HEIGHT: f32 = 0.5;

pub const BASE_FOV_DEG: f32 = 70.0;
pub const SPRINT_FOV_DEG: f32 = 84.0;
pub const CROUCH_FOV_DEG: f32 = 66.0;

/// Sprawdza, czy blok w `place_pos` nakłada się z hitboxem gracza.
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

/// Czy pod stopami gracza jest solidne podłoże (używane przy kucaniu na krawędzi bloku).
fn has_ground_below(world: &World, feet: Vec3, half: f32) -> bool {
    let min = Vec3::new(feet.x - half, feet.y - 0.5, feet.z - half);
    let max = Vec3::new(feet.x + half, feet.y - 0.01, feet.z + half);
    world.box_collides(min, max)
}

/// Ogranicza ruch kucającego gracza, aby zatrzymał się dokładnie na krawędzi bloku.
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
    let step_size = 0.02 * delta.signum();
    let num_steps = (delta.abs() / 0.02).ceil() as i32;
    let mut allowed = 0.0f32;

    for _ in 0..num_steps {
        let test_d = if (allowed + step_size).abs() > delta.abs() {
            delta
        } else {
            allowed + step_size
        };
        let mut test_feet = feet;
        if is_x {
            test_feet.x += test_d;
        } else {
            test_feet.z += test_d;
        }

        if has_ground_below(world, test_feet, half) {
            allowed = test_d;
            if (allowed - delta).abs() < 1e-4 {
                break;
            }
        } else {
            break;
        }
    }
    allowed
}

/// Przesuwa gracza o `motion`, rozwiązując kolizje AABB z blokami świata,
/// obsługując wchodzenie na stopnie (auto-step 0.5 bloku) oraz blokadę krawędzi przy kucaniu.
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

    let aabb = |feet_pos: Vec3| -> (Vec3, Vec3) {
        (
            Vec3::new(feet_pos.x - half, feet_pos.y, feet_pos.z - half),
            Vec3::new(feet_pos.x + half, feet_pos.y + height, feet_pos.z + half),
        )
    };

    // 1. Ruch w osi X
    if motion.x != 0.0 {
        let dx = if is_crouching {
            clamp_crouch_axis(world, feet, motion.x, true, half)
        } else {
            motion.x
        };

        if dx != 0.0 {
            feet.x += dx;
            if world.box_collides(aabb(feet).0, aabb(feet).1) {
                // Próba pokonania stopnia (auto-step)
                let mut stepped = feet;
                stepped.y += STEP_HEIGHT;
                if !world.box_collides(aabb(stepped).0, aabb(stepped).1) {
                    feet = stepped;
                } else {
                    feet.x -= dx;
                }
            }
        }
    }

    // 2. Ruch w osi Z
    if motion.z != 0.0 {
        let dz = if is_crouching {
            clamp_crouch_axis(world, feet, motion.z, false, half)
        } else {
            motion.z
        };

        if dz != 0.0 {
            feet.z += dz;
            if world.box_collides(aabb(feet).0, aabb(feet).1) {
                // Próba pokonania stopnia (auto-step)
                let mut stepped = feet;
                stepped.y += STEP_HEIGHT;
                if !world.box_collides(aabb(stepped).0, aabb(stepped).1) {
                    feet = stepped;
                } else {
                    feet.z -= dz;
                }
            }
        }
    }

    // 3. Ruch w osi Y (grawitacja i skok)
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
