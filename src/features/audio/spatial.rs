//! 3D Spatial Audio mathematics, listener positioning, air absorption, and voxel occlusion filtering.

use std::f32::consts::PI;

use crate::core::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpatialResult {
    pub left_gain: f32,
    pub right_gain: f32,
    pub filter_alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
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
    #[inline]
    pub fn new(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Self {
        Self {
            position,
            forward: forward.normalized(),
            right: right.normalized(),
            up: up.normalized(),
        }
    }

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

        if dist_sq >= max_sq {
            return SpatialResult {
                left_gain: 0.0,
                right_gain: 0.0,
                filter_alpha: 1.0,
            };
        }

        if dist_sq <= 0.0001 {
            return SpatialResult {
                left_gain: 1.0,
                right_gain: 1.0,
                filter_alpha: 1.0,
            };
        }

        let dist = dist_sq.sqrt();
        let inv_dist = 1.0 / dist;
        let dir = delta * inv_dist;

        let ref_dist = min_dist.max(0.5);
        let effective_dist = (dist - ref_dist).max(0.0);
        let mut attenuation = ref_dist / (ref_dist + rolloff * effective_dist);

        let fade_start = max_dist * 0.75;
        if dist > fade_start {
            let t = ((dist - fade_start) / (max_dist - fade_start)).clamp(0.0, 1.0);
            let smooth_fade = 1.0 - (3.0 * t * t - 2.0 * t * t * t);
            attenuation *= smooth_fade;
        }

        let pan = dir.dot(self.right).clamp(-1.0, 1.0);
        let angle = (pan + 1.0) * (PI * 0.25);
        let mut left_gain = angle.cos() * attenuation;
        let mut right_gain = angle.sin() * attenuation;

        let f_air = 22000.0 / (1.0 + 0.045 * dist);
        let occ = occlusion.clamp(0.0, 1.0);
        let transmission_gain = 1.0 - 0.65 * occ;
        left_gain *= transmission_gain;
        right_gain *= transmission_gain;

        let f_occ = 22000.0 * (1.0 - occ).powi(2) + 380.0;
        let f_cutoff = f_air.min(f_occ);

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
