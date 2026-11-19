//! Physics and camera FOV constants for player movement and AABB hitboxes.
//! Tuned to classic voxel engine feel (walking, sprinting, sneaking, swimming, flying).

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
