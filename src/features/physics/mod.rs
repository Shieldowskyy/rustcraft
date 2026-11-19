//! Player movement physics, AABB collision detection, DDA voxel raycasting, and acoustics facade.
//! Pure voxel-space math and world queries with zero platform branching.

mod aabb;
mod acoustics;
pub mod constants;
mod movement;
mod raycast;

#[cfg(test)]
mod tests;

pub use aabb::{block_intersects_player, box_collides, player_touches_ladder, ray_aabb_intersect};
pub use acoustics::{calculate_acoustic_occlusion, calculate_acoustic_reverb};
pub use constants::*;
pub use movement::{move_player_with_collision, resolve_block_penetration};
pub use raycast::{raycast_camera_collision, raycast_world, RaycastHit};
