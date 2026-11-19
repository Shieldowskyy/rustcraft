//! Unit tests for player physics, ladder contact, AABB penetration, and ray intersection.
//! Migrated from `src/gameplay/physics.rs`.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

use super::*;

#[test]
fn ladder_contact_covers_player_hitbox_and_ends_at_ladder_top() {
    let mut world = World::new();
    let mut chunk = Chunk {
        cx: 0,
        cz: 0,
        blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
        dirty: false,
    };
    chunk.set_local(0, 10, 0, BlockId::Ladder);
    world.chunks.insert((0, 0), chunk);

    let half = PLAYER_HALF_WIDTH;
    assert!(player_touches_ladder(&world, Vec3::new(1.1, 10.9, 0.5), half, STANDING_HEIGHT));
    assert!(!player_touches_ladder(&world, Vec3::new(1.1, 11.0, 0.5), half, STANDING_HEIGHT));
}

#[test]
fn test_resolve_block_penetration_pushes_out() {
    let mut world = World::new();
    let mut chunk = Chunk::generate(0, 0, 12345);
    chunk.set_local(0, 60, 0, BlockId::Stone);
    world.chunks.insert((0, 0), chunk);

    let mut feet = Vec3::new(0.95, 60.0, 0.5);
    let pushed = resolve_block_penetration(&world, &mut feet, PLAYER_HALF_WIDTH, STANDING_HEIGHT);
    assert!(pushed);
    assert!(feet.x > 0.95 || feet.y > 60.0);
}

#[test]
fn test_ray_aabb_intersect_hit_and_miss() {
    let min = Vec3::new(-0.5, 0.0, -0.5);
    let max = Vec3::new(0.5, 1.8, 0.5);

    let origin = Vec3::new(0.0, 1.0, -3.0);
    let dir = Vec3::new(0.0, 0.0, 1.0);
    let hit = ray_aabb_intersect(origin, dir, min, max);
    assert!(hit.is_some());
    assert!((hit.unwrap() - 2.5).abs() < 1e-4);

    let miss_dir = Vec3::new(0.0, 0.0, -1.0);
    assert!(ray_aabb_intersect(origin, miss_dir, min, max).is_none());

    let side_origin = Vec3::new(5.0, 1.0, -3.0);
    assert!(ray_aabb_intersect(side_origin, dir, min, max).is_none());
}

#[test]
fn test_player_collision_separation_distance() {
    let p1 = Vec3::new(10.0, 64.0, 10.0);
    let p2 = Vec3::new(10.3, 64.0, 10.0);
    let min_dist = PLAYER_HALF_WIDTH * 2.0;
    let dist = (p1 - p2).length();
    assert!(dist < min_dist);
    assert!((min_dist - dist - 0.3).abs() < 1e-4);
}
