//! Day/night cycle, sun/sky colors, sky exposure checks, occlusion test, and torch/light chunk dirtying.
//! Keeps dynamic light indices (`torches`, `ruststone_lights`) and neighboring subchunks synchronized.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;

use super::chunk::Chunk;
use super::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, MAX_BLOCK_LIGHT_RADIUS};
use super::world_state::{ruststone_light_params, World};

impl World {
    pub fn advance_time(&mut self, dt: f32) {
        self.time_of_day = (self.time_of_day + dt).rem_euclid(Self::DAY_DURATION);
    }

    #[inline]
    pub fn day_fraction(&self) -> f32 { self.time_of_day / Self::DAY_DURATION }

    #[inline]
    pub fn sun_angle(&self) -> f32 {
        self.day_fraction() * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2
    }

    #[inline]
    pub fn sun_direction(&self) -> Vec3 {
        let a = self.sun_angle();
        Vec3::new(a.cos(), a.sin(), 0.25).normalized()
    }

    #[inline]
    pub fn sunlight_factor(&self) -> f32 {
        let t = ((self.sun_angle().sin() + 0.2) / 0.4).clamp(0.0, 1.0);
        let ease = t * t * (3.0 - 2.0 * t);
        self.weather.modulate_daylight(0.08 + 0.92 * ease)
    }

    pub fn sky_colors(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let elev = self.sun_angle().sin();
        let day_t = ((elev + 0.25) / 0.50).clamp(0.0, 1.0);
        let day_smooth = day_t * day_t * (3.0 - 2.0 * day_t);
        let sunset_factor = (1.0 - (elev.abs() / 0.28)).max(0.0);
        let sunset_smooth = sunset_factor * sunset_factor;

        let lerp3 = |a: [f32; 3], b: [f32; 3], t: f32| [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ];

        let mut final_top = lerp3(lerp3([0.015, 0.025, 0.07], [0.32, 0.54, 0.96], day_smooth), [0.24, 0.28, 0.58], sunset_smooth * 0.7);
        let mut final_bot = lerp3(lerp3([0.05, 0.07, 0.15], [0.72, 0.84, 0.98], day_smooth), [0.98, 0.48, 0.18], sunset_smooth);
        let mut final_fog = lerp3(lerp3([0.04, 0.06, 0.12], [0.72, 0.84, 0.98], day_smooth), [0.92, 0.52, 0.25], sunset_smooth);

        if self.weather.rain_strength > 0.001 {
            let (r, t) = (self.weather.rain_strength, self.weather.thunder_strength);
            final_top = lerp3(final_top, lerp3([0.28, 0.32, 0.38], [0.14, 0.16, 0.20], t), r * 0.75);
            final_bot = lerp3(final_bot, lerp3([0.42, 0.46, 0.52], [0.22, 0.25, 0.30], t), r * 0.85);
            final_fog = lerp3(final_fog, lerp3([0.40, 0.44, 0.50], [0.20, 0.22, 0.27], t), r * 0.90);
        }

        if self.weather.lightning_flash > 0.005 {
            let f = self.weather.lightning_flash;
            let flash_col = [0.95, 0.97, 1.0];
            final_top = lerp3(final_top, flash_col, f * 0.85);
            final_bot = lerp3(final_bot, flash_col, f * 0.95);
            final_fog = lerp3(final_fog, flash_col, f * 0.95);
        }

        (final_top, final_bot, final_fog)
    }

    pub fn get_highest_block_y(&self, wx: i32, wz: i32) -> i32 {
        let (cx, cz, lx, lz) = World::world_to_chunk(wx, wz);
        if let Some(chunk) = self.chunks.get(&(cx, cz)) {
            for y in (0..CHUNK_SIZE_Y).rev() {
                if chunk.get_local(lx, y, lz) != BlockId::Air {
                    return y;
                }
            }
        }
        0
    }

    pub fn is_exposed_to_sky(&self, pos: IVec3) -> bool {
        self.get_highest_block_y(pos.x, pos.z) <= pos.y
    }

    #[inline]
    pub fn is_view_fully_occluded(&self, camera_pos: Vec3, yaw: f32, pitch: f32) -> bool {
        let eye_block = camera_pos.floor_to_ivec();
        if self.get_block(eye_block).is_opaque() {
            return true;
        }
        let (sin_y, cos_y) = yaw.sin_cos();
        let (sin_p, cos_p) = pitch.sin_cos();
        // DLACZEGO: Match Camera::forward/right/up basis (+X at yaw=0, -Z at yaw=-PI/2, +Y up on +pitch)
        // and require probes to stay in the camera's chunk so walls on a chunk border never cull the adjacent chunk.
        let forward = Vec3::new(cos_y * cos_p, sin_p, sin_y * cos_p);
        let right = Vec3::new(-sin_y, 0.0, cos_y) * 0.85;
        let up = Vec3::new(-cos_y * sin_p, cos_p, -sin_y * sin_p) * 0.55;
        let c = camera_pos + forward * 0.38;
        let cam_chunk = (eye_block.x >> 4, eye_block.z >> 4);

        [
            c,
            c - right + up,
            c + right + up,
            c - right - up,
            c + right - up,
            c - right,
            c + right,
            c - up,
            c + up,
        ]
        .into_iter()
        .all(|p| {
            let bp = p.floor_to_ivec();
            (bp.x >> 4, bp.z >> 4) == cam_chunk && self.get_block(bp).is_opaque()
        })
    }

    #[inline(always)]

    pub fn index_chunk_lights(&mut self, chunk: &Chunk) {
        let torch_id = BlockId::Torch.to_u8();
        let ore_id = BlockId::RustStoneOre.to_u8();
        let block_id = BlockId::RustStoneBlock.to_u8();
        let lamp_id = BlockId::RustStoneLampLit.to_u8();
        let wire_id = BlockId::RustStoneWireLit.to_u8();
        let (base_x, base_z) = (chunk.cx * CHUNK_SIZE_X, chunk.cz * CHUNK_SIZE_Z);

        // Linear slice scan keeps 16 KB chunk block reads sequential in L1 cache.
        for (idx, &b_u8) in chunk.blocks.iter().enumerate() {
            if b_u8 == torch_id
                || b_u8 == ore_id
                || b_u8 == block_id
                || b_u8 == lamp_id
                || b_u8 == wire_id
            {
                let lx = (idx >> 10) as i32;
                let ly = ((idx >> 4) & 63) as i32;
                let lz = (idx & 15) as i32;
                let pos = IVec3::new(base_x + lx, ly, base_z + lz);
                let b = BlockId::from_u8(b_u8);
                if b == BlockId::Torch {
                    self.torches.insert(pos);
                    self.mark_light_affected_chunks_dirty(pos);
                } else if let Some(params) = ruststone_light_params(b) {
                    self.ruststone_lights.insert(pos, params);
                    self.mark_light_affected_chunks_dirty(pos);
                }
            }
        }
    }


    pub(super) fn mark_dirty(&mut self, cx: i32, cz: i32) {
        if let Some(c) = self.chunks.get_mut(&(cx, cz)) { c.dirty = true; }
    }

    pub(super) fn mark_light_affected_chunks_dirty(&mut self, pos: IVec3) {
        let min_cx = pos.x.saturating_sub(MAX_BLOCK_LIGHT_RADIUS) >> 4;
        let max_cx = pos.x.saturating_add(MAX_BLOCK_LIGHT_RADIUS) >> 4;
        let min_cz = pos.z.saturating_sub(MAX_BLOCK_LIGHT_RADIUS) >> 4;
        let max_cz = pos.z.saturating_add(MAX_BLOCK_LIGHT_RADIUS) >> 4;

        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                self.mark_dirty(cx, cz);
            }
        }
    }

    pub(super) fn has_nearby_light_source(&self, pos: IVec3) -> bool {
        let is_within_radius = |source: IVec3, radius: f32| {
            let dx = (pos.x - source.x) as f32;
            let dy = (pos.y - source.y) as f32;
            let dz = (pos.z - source.z) as f32;
            dx * dx + dy * dy + dz * dz <= radius * radius
        };

        self.torches
        .iter()
        .any(|&source| is_within_radius(source, MAX_BLOCK_LIGHT_RADIUS as f32))
        || self
        .ruststone_lights
        .iter()
        .any(|(&source, &(radius, _))| is_within_radius(source, radius))
    }

    pub(super) fn mark_neighbors_dirty(&mut self, cx: i32, cz: i32) {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.mark_dirty(cx + dx, cz + dz);
        }
    }

}
