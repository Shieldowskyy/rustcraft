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

/// Fractal 2D (multi-octave) noise for natural terrain height & temperature.
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

fn hash3(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut h: u32 = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(1103515245))
        .wrapping_add((z as u32).wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(2246822519));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// 3D value noise in range roughly [-1, 1], sampled at continuous (x, y, z).
pub fn value_noise3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    let tx = smooth(x - x0 as f32);
    let ty = smooth(y - y0 as f32);
    let tz = smooth(z - z0 as f32);

    let v000 = hash3(x0, y0, z0, seed);
    let v100 = hash3(x1, y0, z0, seed);
    let v010 = hash3(x0, y1, z0, seed);
    let v110 = hash3(x1, y1, z0, seed);
    let v001 = hash3(x0, y0, z1, seed);
    let v101 = hash3(x1, y0, z1, seed);
    let v011 = hash3(x0, y1, z1, seed);
    let v111 = hash3(x1, y1, z1, seed);

    let x00 = lerp(v000, v100, tx);
    let x10 = lerp(v010, v110, tx);
    let x01 = lerp(v001, v101, tx);
    let x11 = lerp(v011, v111, tx);

    let y0 = lerp(x00, x10, ty);
    let y1 = lerp(x01, x11, ty);

    lerp(y0, y1, tz)
}

/// Fractal 3D noise for underground cave systems and hollows.
pub fn fractal_noise3(x: f32, y: f32, z: f32, seed: u32, octaves: u32, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_amp = 0.0;

    for i in 0..octaves {
        total += value_noise3(x * frequency, y * frequency, z * frequency, seed.wrapping_add(i * 101)) * amplitude;
        max_amp += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    total / max_amp
}
