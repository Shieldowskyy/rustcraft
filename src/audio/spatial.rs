//! 3D Spatial Audio mathematics, listener positioning, and acoustic physics.
//!
//! Provides ultra-fast distance attenuation, constant-power stereo panning,
//! atmospheric air absorption, and voxel acoustic occlusion filtering.

use std::f32::consts::PI;
use crate::core::math::Vec3;

/// Result of 3D spatial acoustic evaluation for a sound emitter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpatialResult {
    /// Left channel volume multiplier.
    pub left_gain: f32,
    /// Right channel volume multiplier.
    pub right_gain: f32,
    /// 1-pole IIR low-pass filter alpha coefficient in [0.005, 1.0].
    /// 1.0 = completely transparent (no high-frequency damping).
    /// Lower values muffle the sound (air absorption and wall occlusion).
    pub filter_alpha: f32,
}

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

    /// Calculates comprehensive 3D spatial acoustics:
    /// 1. Distance Attenuation (Inverse square/rolloff curve with smooth boundary window).
    /// 2. Constant-Power Azimuth Panning (Left / Right stereo gains).
    /// 3. Atmospheric Air Absorption (Distance-dependent high-frequency molecular damping).
    /// 4. Acoustic Occlusion (Low-pass muffling and direct transmission loss through solid blocks).
    #[inline]
    pub fn calculate_spatial(
        &self,
        emitter_pos: Vec3,
        min_dist: f32,
        max_dist: f32,
        rolloff: f32,
        occlusion: f32,
    ) -> SpatialResult {
        let delta = emitter_pos - self.position;
        let dist_sq = delta.length_sq();
        let max_sq = max_dist * max_dist;

        // Fast cull if beyond maximum audible range
        if dist_sq >= max_sq {
            return SpatialResult {
                left_gain: 0.0,
                right_gain: 0.0,
                filter_alpha: 1.0,
            };
        }

        if dist_sq <= 0.0001 {
            // Emitter right at listener position (e.g. self footsteps or head)
            return SpatialResult {
                left_gain: 1.0,
                right_gain: 1.0,
                filter_alpha: 1.0,
            };
        }

        let dist = dist_sq.sqrt();
        let inv_dist = 1.0 / dist;
        let dir = delta * inv_dist;

        // 1. Distance attenuation: inverse distance with reference min_dist
        let ref_dist = min_dist.max(0.5);
        let effective_dist = (dist - ref_dist).max(0.0);
        let mut attenuation = ref_dist / (ref_dist + rolloff * effective_dist);

        // Smooth cubic Hermite horizon fade near max_dist to eliminate boundary popping
        let fade_start = max_dist * 0.75;
        if dist > fade_start {
            let t = ((dist - fade_start) / (max_dist - fade_start)).clamp(0.0, 1.0);
            let smooth_fade = 1.0 - (3.0 * t * t - 2.0 * t * t * t);
            attenuation *= smooth_fade;
        }

        // 2. Azimuth panning: dot product with right ear vector [-1.0 = left, +1.0 = right]
        let pan = dir.dot(self.right).clamp(-1.0, 1.0);

        // Constant power panning curve: angle in [0, PI/2]
        let angle = (pan + 1.0) * (PI * 0.25);
        let mut left_gain = angle.cos() * attenuation;
        let mut right_gain = angle.sin() * attenuation;

        // 3. Atmospheric Air Absorption (ISO 9613 high frequency damping)
        // High frequencies attenuate more with distance due to air viscosity and relaxation.
        let f_air = 22000.0 / (1.0 + 0.045 * dist);

        // 4. Acoustic Occlusion (Sound traveling through walls / solid blocks)
        // Solid materials heavily muffle high frequencies while allowing low frequencies through.
        let occ = occlusion.clamp(0.0, 1.0);
        let transmission_gain = 1.0 - 0.65 * occ;
        left_gain *= transmission_gain;
        right_gain *= transmission_gain;

        let f_occ = 22000.0 * (1.0 - occ).powi(2) + 380.0;

        // Effective cutoff is the most restrictive of air absorption and occlusion
        let f_cutoff = f_air.min(f_occ);

        // Convert cutoff frequency to 1-pole IIR filter alpha (assuming standard 48 kHz output)
        let filter_alpha = if f_cutoff >= 20000.0 {
            1.0
        } else {
            let w = 2.0 * PI * f_cutoff / 48000.0;
            (w / (1.0 + w)).clamp(0.008, 1.0)
        };

        SpatialResult {
            left_gain,
            right_gain,
            filter_alpha,
        }
    }

    /// Calculates stereo panning gains (left_gain, right_gain) and distance attenuation
    /// for a sound emitter positioned at `emitter_pos` (backward-compatible).
    #[inline]
    pub fn calculate_spatial_gains(
        &self,
        emitter_pos: Vec3,
        min_dist: f32,
        max_dist: f32,
        rolloff: f32,
    ) -> (f32, f32) {
        let res = self.calculate_spatial(emitter_pos, min_dist, max_dist, rolloff, 0.0);
        (res.left_gain, res.right_gain)
    }
}
