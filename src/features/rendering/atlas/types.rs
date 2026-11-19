//! Core 16x16 texture bitmap primitive and deterministic hash noise helpers.

pub const TEX_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub struct Texture {
    pub texels: Vec<[f32; 3]>, // len == TEX_SIZE * TEX_SIZE
}

impl Texture {
    #[inline(always)]
    pub fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        let x = (u * TEX_SIZE as f32) as i32;
        let y = (v * TEX_SIZE as f32) as i32;
        let ux = x.rem_euclid(TEX_SIZE as i32) as usize;
        let uy = y.rem_euclid(TEX_SIZE as i32) as usize;
        self.texels[uy * TEX_SIZE + ux]
    }
}

pub(super) fn hash(mut n: u32) -> u32 {
    n = (n ^ 61) ^ (n >> 16);
    n = n.wrapping_add(n << 3);
    n ^= n >> 4;
    n = n.wrapping_mul(0x27d4eb2d);
    n ^= n >> 15;
    n
}

pub(super) fn rand_at(x: usize, y: usize, seed: u32) -> f32 {
    let h = hash((x as u32).wrapping_mul(1973).wrapping_add((y as u32).wrapping_mul(9277)).wrapping_add(seed.wrapping_mul(26699)));
    (h % 10000) as f32 / 10000.0
}

pub(super) fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub(super) fn speckled(base: [f32; 3], variance: f32, seed: u32) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, seed) - 0.5;
            let shade = 1.0 + n * variance;
            texels[y * TEX_SIZE + x] = [
                (base[0] * shade).clamp(0.0, 1.0),
                (base[1] * shade).clamp(0.0, 1.0),
                (base[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}
