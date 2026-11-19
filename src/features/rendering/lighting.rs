//! Shared lighting and chunk spawn animation curves across software, WGPU, and PS Vita GXM.
//! Pure deterministic math with zero platform `#[cfg]`.

/// Computes vertical Y pop-in animation offset for newly streamed chunks.
#[inline]
pub fn chunk_spawn_y_offset(chunk_anim: bool, total_time: f32, spawn_time: f32) -> f32 {
    if !chunk_anim {
        return 0.0;
    }
    let elapsed = total_time - spawn_time;
    if (0.0..0.35).contains(&elapsed) {
        let t = (elapsed / 0.35).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - t).powi(3);
        (1.0 - ease) * -14.0
    } else {
        0.0
    }
}

/// Computes RGB entity/dynamic-geometry lighting matching the PC software rasterizer curve.
#[inline(always)]
pub fn compute_entity_light_rgb(
    ao: f32,
    sky_light: f32,
    block_light: f32,
    daylight: f32,
    brightness: f32,
) -> [f32; 3] {
    let sky_lit = sky_light * daylight;
    let natural_r = 1.14 * sky_lit;
    let natural_g = 1.12 * sky_lit;
    let natural_b = 1.10 * sky_lit;

    let torch_r = 1.30 * block_light;
    let torch_g = 0.88 * block_light;
    let torch_b = 0.42 * block_light;

    let ambient_boost = brightness * 0.16;
    let bright_scale = 1.0 + brightness * 0.22;
    [
        (((natural_r + torch_r + 0.045 + ambient_boost) * ao) * bright_scale).clamp(0.04, 1.45),
        (((natural_g + torch_g + 0.045 + ambient_boost) * ao) * bright_scale).clamp(0.04, 1.45),
        (((natural_b + torch_b + 0.055 + ambient_boost) * ao) * bright_scale).clamp(0.04, 1.45),
    ]
}
