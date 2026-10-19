//! 3D Spatial Audio mathematics and listener positioning.
//!
//! Provides ultra-fast distance attenuation and constant-power stereo panning
//! based on player/camera orientation.

use std::f32::consts::PI;
use crate::core::math::Vec3;

/// 3D Spatial Audio Listener state (usually attached to the player's camera).
#[derive(Clone, Copy, Debug)]
pub struct AudioListener {
    /// Listener position in world space.
    pub position: Vec3,
    /// Normalized forward view vector.
    pub forward: Vec3,
    /// Normalized right view vector (pointing to the right ear).
    pub right: Vec3,
    /// Normalized up vector.
    pub up: Vec3,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            forward: Vec3::new(0.0, 0.0, -1.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

impl AudioListener {
    /// Creates or updates listener orientation from camera state.
    #[inline]
    pub fn new(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Self {
        Self {
            position,
            forward: forward.normalized(),
            right: right.normalized(),
            up: up.normalized(),
        }
    }

    /// Calculates stereo panning gains (left_gain, right_gain) and distance attenuation
    /// for a sound emitter positioned at `emitter_pos`.
    ///
    /// Performance optimizations:
    /// - Checks `distance_squared > max_dist * max_dist` first for instantaneous culling.
    /// - Uses constant-power panning (`cos` / `sin` on a quarter cycle) to prevent
    ///   the center dip artifact of linear panning.
    #[inline]
    pub fn calculate_spatial_gains(
        &self,
        emitter_pos: Vec3,
        min_dist: f32,
        max_dist: f32,
        rolloff: f32,
    ) -> (f32, f32) {
        let delta = emitter_pos - self.position;
        let dist_sq = delta.length_sq();
        let max_sq = max_dist * max_dist;

        // Fast cull if beyond maximum audible range
        if dist_sq >= max_sq || dist_sq <= 0.0001 {
            if dist_sq <= 0.0001 {
                // Emitter right at listener position (e.g. self footsteps or head)
                return (1.0, 1.0);
            }
            return (0.0, 0.0);
        }

        let dist = dist_sq.sqrt();
        let inv_dist = 1.0 / dist;
        let dir = delta * inv_dist;

        // Distance attenuation: inverse distance with min_dist threshold
        let effective_dist = (dist - min_dist).max(0.0);
        let attenuation = 1.0 / (1.0 + rolloff * effective_dist);

        // Azimuth panning: dot product with right vector gives pan [-1.0 = full left, +1.0 = full right]
        let pan = dir.dot(self.right).clamp(-1.0, 1.0);

        // Constant power panning curve:
        // angle in [0, PI/2]: angle 0 = full left, PI/4 = center, PI/2 = full right
        let angle = (pan + 1.0) * (PI * 0.25);
        let left_gain = angle.cos() * attenuation;
        let right_gain = angle.sin() * attenuation;

        (left_gain, right_gain)
    }
}
