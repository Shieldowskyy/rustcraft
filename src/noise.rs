// A small deterministic hash-based value noise implementation.
// Not cryptographic - just enough to generate varied, seed-based terrain
// without pulling in an external noise crate.

fn hash2(x: i32, z: i32, seed: u32) -> f32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((z as u32).wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(2246822519));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 2D value noise in range roughly [-1, 1], sampled at continuous (x, z).
pub fn value_noise2(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let tx = smooth(x - x0 as f32);
    let tz = smooth(z - z0 as f32);

    let v00 = hash2(x0, z0, seed);
    let v10 = hash2(x1, z0, seed);
    let v01 = hash2(x0, z1, seed);
    let v11 = hash2(x1, z1, seed);

    let a = lerp(v00, v10, tx);
    let b = lerp(v01, v11, tx);
    lerp(a, b, tz)
}

/// Fractal (multi-octave) noise for more natural-looking terrain.
pub fn fractal_noise2(x: f32, z: f32, seed: u32, octaves: u32, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_amp = 0.0;

    for i in 0..octaves {
        total += value_noise2(x * frequency, z * frequency, seed.wrapping_add(i * 101)) * amplitude;
        max_amp += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    total / max_amp
}
