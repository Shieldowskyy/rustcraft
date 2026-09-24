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

/// Ogranicza ruch kucającego gracza, aby zatrzymał się dokładnie na krawędzi bloku (Binary Search O(log N)).
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

    // Sprawdź najpierw pełny ruch
    let mut full_feet = feet;
    if is_x { full_feet.x += delta; } else { full_feet.z += delta; }
    if has_ground_below(world, full_feet, half) {
        return delta;
    }

    // Szybkie wyszukiwanie binarne granicy krawędzi (zaledwie 6 kroków zamiast 50)
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

/// Wypycha gracza z bloków stałych w stronę najbliższych krawędzi lub w górę (przy spawnie w ziemi).
pub fn resolve_block_penetration(
    world: &World,
    feet: &mut Vec3,
    half: f32,
    height: f32,
) -> bool {
    let aabb_min = Vec3::new(feet.x - half, feet.y, feet.z - half);
    let aabb_max = Vec3::new(feet.x + half, feet.y + height, feet.z + half);

    if !world.box_collides(aabb_min, aabb_max) {
        return false;
    }

    let x0 = aabb_min.x.floor() as i32;
    let x1 = (aabb_max.x - 1e-4).floor() as i32;
    let y0 = (aabb_min.y.floor() as i32).max(0);
    let y1 = ((aabb_max.y - 1e-4).floor() as i32).min(crate::world::CHUNK_SIZE_Y as i32 - 1);
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
    let mut cached_chunk: Option<&crate::world::Chunk> = None;

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

                        // Oblicz minimalne wypchnięcie poziome do krawędzi bloku
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
        // Jeśli gracz jest mocno zakopany w ziemi od dołu (np. na pierwszym spawnie), wypchnij go w górę ponad blok
        if ground_push_y > 0.001 && solid_count >= 2 {
            let mut test_feet = *feet;
            test_feet.y += ground_push_y + 0.05;
            let (min, max) = (
                Vec3::new(test_feet.x - half, test_feet.y, test_feet.z - half),
                Vec3::new(test_feet.x + half, test_feet.y + height, test_feet.z + half),
            );
            if !world.box_collides(min, max) {
                feet.y = test_feet.y;
                return true;
            }
        }

        // Wypchnij łagodnie w stronę krawędzi
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

/// Przesuwa gracza o `motion`, rozwiązując kolizje AABB z blokami świata,
/// obsługując wchodzenie na stopnie (auto-step 0.5 bloku), wypychanie z krawędzi oraz blokadę kucania.
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

    // Najpierw rozwiąż ewentualne zablokowanie/przenikanie (np. po spawnie)
    resolve_block_penetration(world, &mut feet, half, height);

    let aabb = |feet_pos: Vec3| -> (Vec3, Vec3) {
        (
            Vec3::new(feet_pos.x - half, feet_pos.y, feet_pos.z - half),
            Vec3::new(feet_pos.x + half, feet_pos.y + height, feet_pos.z + half),
        )
    };

    let was_on_ground = has_ground_below(world, feet, half);

    // 1. Ruch w osi X
    if motion.x != 0.0 {
        let dx = if is_crouching && was_on_ground {
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
        let dz = if is_crouching && was_on_ground {
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

    // Dodatkowe zabezpieczenie przed utknięciem w krawędzi bloku
    resolve_block_penetration(world, &mut feet, half, height);

    eye_pos.x = feet.x;
    eye_pos.y = feet.y + player_eye_offset;
    eye_pos.z = feet.z;

    on_ground
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::block::BlockId;

    #[test]
    fn test_resolve_block_penetration_pushes_out() {
        use crate::world::Chunk;
        let mut world = World::new();
        let mut chunk = Chunk::generate(0, 0, 12345);
        chunk.set_local(0, 60, 0, BlockId::Stone);
        world.chunks.insert((0, 0), chunk);

        // Player's feet slightly intersecting stone block at x = 0.95 (inside by 0.05 from right)
        let mut feet = Vec3::new(0.95, 60.0, 0.5);
        let pushed = resolve_block_penetration(&world, &mut feet, PLAYER_HALF_WIDTH, STANDING_HEIGHT);
        assert!(pushed);
        // Feet should have been pushed outwards along X or Y
        assert!(feet.x > 0.95 || feet.y > 60.0);
    }
}
