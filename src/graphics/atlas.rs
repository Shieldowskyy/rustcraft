// Basic procedural 16x16 textures, generated once at startup with simple
// deterministic hash-based noise (no image files, no external assets).

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

fn hash(mut n: u32) -> u32 {
    n = (n ^ 61) ^ (n >> 16);
    n = n.wrapping_add(n << 3);
    n ^= n >> 4;
    n = n.wrapping_mul(0x27d4eb2d);
    n ^= n >> 15;
    n
}

fn rand_at(x: usize, y: usize, seed: u32) -> f32 {
    let h = hash((x as u32).wrapping_mul(1973).wrapping_add((y as u32).wrapping_mul(9277)).wrapping_add(seed.wrapping_mul(26699)));
    (h % 10000) as f32 / 10000.0
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn speckled(base: [f32; 3], variance: f32, seed: u32) -> Texture {
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

fn grass_top() -> Texture {
    let base = [0.36, 0.68, 0.24];
    let light = [0.48, 0.80, 0.32];
    let mut tex = speckled(base, 0.18, 11);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 42) > 0.86 {
                tex.texels[y * TEX_SIZE + x] = light;
            }
        }
    }
    tex
}

fn grass_side() -> Texture {
    let dirt = [0.45, 0.33, 0.20];
    let grass = [0.36, 0.68, 0.24];
    let mut tex = speckled(dirt, 0.15, 7);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn grass_top_swamp() -> Texture {
    let base = [0.34, 0.44, 0.18];
    let dark = [0.26, 0.35, 0.14];
    let mut tex = speckled(base, 0.16, 17);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 55) > 0.75 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

fn grass_side_swamp() -> Texture {
    let dirt = [0.38, 0.28, 0.16];
    let grass = [0.34, 0.44, 0.18];
    let mut tex = speckled(dirt, 0.15, 19);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn leaves_swamp(fancy: bool) -> Texture {
    let dark_green = [0.20, 0.34, 0.12];
    let light_green = [0.32, 0.46, 0.18];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let rib = ((x % 3 == 0) || (y % 3 == 0)) as i32 as f32;
            let noise = rand_at(x, y, 37);
            let mut col = mix(dark_green, light_green, rib);
            col[0] = (col[0] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x + y) % 3 == 0 && noise > 0.45)
                    || ((x ^ y) % 5 == 0 && noise > 0.62)
                    || ((x * 3 + y * 2) % 7 == 0 && noise > 0.54);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn grass_top_savanna() -> Texture {
    let base = [0.65, 0.62, 0.24];
    let light = [0.74, 0.70, 0.30];
    let mut tex = speckled(base, 0.16, 23);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 62) > 0.82 {
                tex.texels[y * TEX_SIZE + x] = light;
            }
        }
    }
    tex
}

fn grass_side_savanna() -> Texture {
    let dirt = [0.45, 0.33, 0.20];
    let grass = [0.65, 0.62, 0.24];
    let mut tex = speckled(dirt, 0.15, 29);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn leaves_savanna(fancy: bool) -> Texture {
    let dark = [0.38, 0.46, 0.16];
    let light = [0.52, 0.60, 0.24];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let rib = ((x % 3 == 0) || (y % 3 == 0)) as i32 as f32;
            let noise = rand_at(x, y, 47);
            let mut col = mix(dark, light, rib);
            col[0] = (col[0] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x + y) % 3 == 0 && noise > 0.42)
                    || ((x ^ y) % 5 == 0 && noise > 0.58)
                    || ((x * 2 + y * 3) % 7 == 0 && noise > 0.50);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn grass_top_cherry() -> Texture {
    let base = [0.44, 0.76, 0.32];
    let petal = [0.95, 0.72, 0.82];
    let mut tex = speckled(base, 0.15, 31);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 88) > 0.90 {
                tex.texels[y * TEX_SIZE + x] = petal;
            }
        }
    }
    tex
}

fn grass_side_cherry() -> Texture {
    let dirt = [0.45, 0.33, 0.20];
    let grass = [0.44, 0.76, 0.32];
    let mut tex = speckled(dirt, 0.15, 33);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn grass_top_autumn() -> Texture {
    let base = [0.55, 0.52, 0.20];
    let leaf = [0.85, 0.40, 0.12];
    let mut tex = speckled(base, 0.16, 43);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 77) > 0.88 {
                tex.texels[y * TEX_SIZE + x] = leaf;
            }
        }
    }
    tex
}

fn grass_side_autumn() -> Texture {
    let dirt = [0.45, 0.33, 0.20];
    let grass = [0.55, 0.52, 0.20];
    let mut tex = speckled(dirt, 0.15, 45);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn grass_top_taiga() -> Texture {
    let base = [0.26, 0.52, 0.34];
    let pine = [0.18, 0.42, 0.26];
    let mut tex = speckled(base, 0.16, 53);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 66) > 0.85 {
                tex.texels[y * TEX_SIZE + x] = pine;
            }
        }
    }
    tex
}

fn grass_side_taiga() -> Texture {
    let dirt = [0.42, 0.30, 0.18];
    let grass = [0.26, 0.52, 0.34];
    let mut tex = speckled(dirt, 0.15, 55);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize;
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn dirt() -> Texture {
    speckled([0.45, 0.33, 0.20], 0.20, 3)
}

fn stone() -> Texture {
    let base = [0.55, 0.55, 0.55];
    let mut tex = speckled(base, 0.14, 21);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 71) > 0.93 {
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], [0.3, 0.3, 0.3], 0.6);
            }
        }
    }
    tex
}

fn sand() -> Texture {
    speckled([0.87, 0.80, 0.55], 0.12, 5)
}

fn wood_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let center = TEX_SIZE as f32 / 2.0 - 0.5;
    let base = [0.55, 0.42, 0.25];
    let dark = [0.40, 0.28, 0.15];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let ring = (dist * 1.6).sin() * 0.5 + 0.5;
            let noise = (rand_at(x, y, 61) - 0.5) * 0.08;
            texels[y * TEX_SIZE + x] = mix(base, dark, (ring + noise).clamp(0.0, 1.0));
        }
    }
    Texture { texels }
}

fn wood_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.42, 0.30, 0.16];
    let dark = [0.32, 0.22, 0.11];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let stripe = ((x as f32 * 0.9).sin() * 0.5 + 0.5 + (rand_at(x, y, 88) - 0.5) * 0.15).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = mix(base, dark, stripe);
        }
    }
    Texture { texels }
}

fn leaves(fancy: bool) -> Texture {
    let dark_green = [0.18, 0.44, 0.12];
    let light_green = [0.30, 0.58, 0.20];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let rib = ((x % 3 == 0) || (y % 3 == 0)) as i32 as f32;
            let noise = rand_at(x, y, 33);
            let mut col = mix(dark_green, light_green, rib);
            col[0] = (col[0] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x + y) % 3 == 0 && noise > 0.42)
                    || ((x ^ y) % 5 == 0 && noise > 0.60)
                    || ((x * 2 + y * 3) % 7 == 0 && noise > 0.52);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn water() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.15, 0.35, 0.75];
    let light = [0.25, 0.48, 0.85];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let wave = ((y as f32 * 0.8 + x as f32 * 0.2).sin() * 0.5 + 0.5) * 0.5
            + (rand_at(x, y, 55) * 0.15);
            texels[y * TEX_SIZE + x] = mix(base, light, wave.clamp(0.0, 1.0));
        }
    }
    Texture { texels }
}

fn snow() -> Texture {
    let base = [0.94, 0.96, 0.99];
    let light = [1.0, 1.0, 1.0];
    let shadow = [0.86, 0.90, 0.96];
    let mut tex = speckled(base, 0.08, 77);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 105);
            if r > 0.90 {
                tex.texels[y * TEX_SIZE + x] = light;
            } else if r < 0.10 {
                tex.texels[y * TEX_SIZE + x] = shadow;
            }
        }
    }
    tex
}

fn cactus_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let center = TEX_SIZE as f32 / 2.0 - 0.5;
    let base = [0.18, 0.55, 0.15];
    let dark = [0.12, 0.40, 0.10];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let n = rand_at(x, y, 44) * 0.1;
            let factor = (dist / 8.0 + n).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = mix(dark, base, factor);
        }
    }
    Texture { texels }
}

fn cactus_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let light_green = [0.22, 0.58, 0.16];
    let dark_green = [0.13, 0.42, 0.10];
    let spine = [0.08, 0.20, 0.06];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let rib = (x as f32 * 1.6).sin() * 0.5 + 0.5;
            let mut col = mix(dark_green, light_green, rib);
            if (x % 4 == 1 || x % 4 == 2) && (y % 4 == 1) {
                col = spine;
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn coal_ore() -> Texture {
    let mut tex = stone();
    let coal_color = [0.10, 0.10, 0.12];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 312);
            if n > 0.72 {
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], coal_color, 0.90);
            }
        }
    }
    tex
}

fn iron_ore() -> Texture {
    let mut tex = stone();
    let iron_color = [0.78, 0.60, 0.45];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 521);
            if n > 0.75 {
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], iron_color, 0.85);
            }
        }
    }
    tex
}

fn bedrock() -> Texture {
    let base = [0.20, 0.20, 0.20];
    let crack = [0.10, 0.10, 0.10];
    let light = [0.30, 0.30, 0.30];
    let mut tex = speckled(base, 0.12, 191);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n1 = rand_at(x, y, 311);
            let n2 = rand_at((x + 3) % TEX_SIZE, (y + 5) % TEX_SIZE, 413);
            if n1 > 0.88 || n2 > 0.88 {
                tex.texels[y * TEX_SIZE + x] = crack;
            } else if n1 < 0.06 {
                tex.texels[y * TEX_SIZE + x] = light;
            }
        }
    }
    tex
}

fn lava() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let hot = [1.0, 0.55, 0.05];
    let dark = [0.55, 0.10, 0.02];
    let bright = [1.0, 0.85, 0.20];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let px = x as f32 / TEX_SIZE as f32 * std::f32::consts::TAU;
            let py = y as f32 / TEX_SIZE as f32 * std::f32::consts::TAU;
            let wave = ((px.sin() + py.cos()) * 0.5 + 0.5)
            + (((px * 2.0 + py).sin()) * 0.3);
            let n = rand_at(x, y, 777) * 0.25;
            let t = (wave + n).clamp(0.0, 1.0);
            let col = if t > 0.80 {
                mix(hot, bright, (t - 0.80) / 0.20)
            } else {
                mix(dark, hot, t / 0.80)
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn snow_leaves(fancy: bool) -> Texture {
    let base = [0.45, 0.62, 0.68];
    let snow = [0.85, 0.92, 0.95];
    let dark_pine = [0.16, 0.32, 0.22];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let noise = rand_at(x, y, 444);
            let mut col = if noise > 0.65 {
                snow
            } else if noise < 0.25 {
                dark_pine
            } else {
                base
            };
            col[0] = (col[0] + (noise - 0.5) * 0.05).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.05).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.05).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x * 3 + y * 5) % 6 == 0 && noise > 0.44)
                    || ((x ^ y) % 5 == 0 && noise > 0.60)
                    || ((x + y * 2) % 7 == 0 && noise > 0.52);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn tnt_side() -> Texture {
    let red = [0.85, 0.15, 0.10];
    let white = [0.90, 0.90, 0.88];
    let black = [0.10, 0.10, 0.10];
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = red;
            if y >= 5 && y <= 10 {
                col = white;
                let is_text_pixel = match (x, y) {
                    (2, 6) | (3, 6) | (4, 6) | (3, 7) | (3, 8) | (3, 9) => true,
                    (6, 6) | (6, 7) | (6, 8) | (6, 9) |
                    (7, 7) | (8, 8) |
                    (9, 6) | (9, 7) | (9, 8) | (9, 9) => true,
                    (11, 6) | (12, 6) | (13, 6) | (12, 7) | (12, 8) | (12, 9) => true,
                    _ => false,
                };
                if is_text_pixel {
                    col = black;
                }
            }
            let n = rand_at(x, y, 808) * 0.08 - 0.04;
            texels[y * TEX_SIZE + x] = [
                (col[0] + n).clamp(0.0, 1.0),
                (col[1] + n).clamp(0.0, 1.0),
                (col[2] + n).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

fn tnt_top() -> Texture {
    let base = [0.75, 0.70, 0.60];
    let dark = [0.45, 0.40, 0.35];
    let red = [0.85, 0.15, 0.10];
    let mut tex = speckled(base, 0.10, 809);

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 3 || x == 12 || y == 3 || y == 12 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
            if (x >= 6 && x <= 9) && (y >= 6 && y <= 9) {
                tex.texels[y * TEX_SIZE + x] = red;
            }
        }
    }
    tex
}

fn planks() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let plank_base_tones = [
        [0.69, 0.53, 0.31],
        [0.64, 0.48, 0.28],
        [0.72, 0.56, 0.34],
        [0.66, 0.50, 0.29],
    ];

    let groove_color = [0.36, 0.24, 0.12];
    let bevel_highlight = [0.77, 0.62, 0.38];
    let nail_color = [0.24, 0.16, 0.08];

    for y in 0..TEX_SIZE {
        let plank_idx = y / 4;
        let in_plank_y = y % 4;
        let base_col = plank_base_tones[plank_idx];

        let joint_x = match plank_idx {
            0 => 8,
            1 => 13,
            2 => 4,
            3 => 10,
            _ => 8,
        };

        for x in 0..TEX_SIZE {
            let mut col = base_col;

            let grain = ((x as f32 * 0.4 + (y as f32 * 1.5).sin()).sin() * 0.5 + 0.5) * 0.06;
            let noise = rand_at(x, y, 713) * 0.06 - 0.03;
            col[0] = (col[0] + grain + noise).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8 + noise).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6 + noise).clamp(0.0, 1.0);

            if in_plank_y == 0 && y > 0 {
                col = mix(col, bevel_highlight, 0.45);
            }

            if in_plank_y == 3 {
                col = groove_color;
            }

            if x == joint_x {
                col = groove_color;
            } else if x == (joint_x + 1) % 16 && in_plank_y > 0 && in_plank_y < 3 {
                col = mix(col, bevel_highlight, 0.30);
            }

            let is_nail = (x == (joint_x + 1) % 16 && in_plank_y == 1)
                || (x == (joint_x + 15) % 16 && in_plank_y == 2);
            if is_nail {
                col = nail_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }

    Texture { texels }
}

fn door_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.38, 0.24, 0.12];
    let frame_light = [0.65, 0.48, 0.26];
    let glass_base = [0.62, 0.80, 0.88];
    let glass_shine = [0.85, 0.95, 1.0];
    let handle_color = [0.82, 0.72, 0.32];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = frame_color;

            let grain = rand_at(x, y, 921) * 0.08 - 0.04;
            col[0] = (col[0] + grain).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6).clamp(0.0, 1.0);

            let in_left_win1 = (x >= 2 && x <= 6) && (y >= 2 && y <= 6);
            let in_right_win1 = (x >= 9 && x <= 13) && (y >= 2 && y <= 6);
            let in_left_win2 = (x >= 2 && x <= 6) && (y >= 8 && y <= 12);
            let in_right_win2 = (x >= 9 && x <= 13) && (y >= 8 && y <= 12);

            let in_win = in_left_win1 || in_right_win1 || in_left_win2 || in_right_win2;

            if in_win {
                let is_win_border = (x == 2 || x == 6 || x == 9 || x == 13)
                    || (y == 2 || y == 6 || y == 8 || y == 12);

                if is_win_border {
                    if y == 2 || y == 8 || x == 2 || x == 9 {
                        col = frame_dark;
                    } else {
                        col = frame_light;
                    }
                } else {
                    let shine = x == y || (x + 1 == y);
                    col = if shine { glass_shine } else { glass_base };
                }
            } else {
                if x == 0 || x == 15 || y == 0 || y == 7 || y == 13 {
                    col = mix(col, frame_dark, 0.4);
                } else if x == 1 || x == 8 {
                    col = mix(col, frame_light, 0.3);
                }
            }

            if (x == 12 || x == 13) && (y == 14 || y == 15) {
                col = handle_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }

    Texture { texels }
}

fn door_bottom() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.35, 0.22, 0.10];
    let frame_light = [0.65, 0.48, 0.26];
    let panel_color = [0.58, 0.42, 0.22];
    let handle_color = [0.82, 0.72, 0.32];
    let nail_color = [0.22, 0.15, 0.08];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = frame_color;

            let grain = rand_at(x, y, 922) * 0.08 - 0.04;
            col[0] = (col[0] + grain).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6).clamp(0.0, 1.0);

            let in_left_panel = (x >= 2 && x <= 6) && (y >= 2 && y <= 13);
            let in_right_panel = (x >= 9 && x <= 13) && (y >= 2 && y <= 13);

            if in_left_panel || in_right_panel {
                let is_panel_border = (x == 2 || x == 6 || x == 9 || x == 13)
                    || (y == 2 || y == 13);

                if is_panel_border {
                    if y == 2 || x == 2 || x == 9 {
                        col = frame_dark;
                    } else {
                        col = frame_light;
                    }
                } else {
                    let p_grain = ((x as f32 * 1.5).sin() * 0.04) + rand_at(x, y, 923) * 0.04;
                    col = [
                        (panel_color[0] + p_grain).clamp(0.0, 1.0),
                        (panel_color[1] + p_grain).clamp(0.0, 1.0),
                        (panel_color[2] + p_grain).clamp(0.0, 1.0),
                    ];
                }
            } else {
                if x == 0 || x == 15 || y == 15 {
                    col = mix(col, frame_dark, 0.4);
                } else if x == 1 || x == 8 || y == 0 {
                    col = mix(col, frame_light, 0.3);
                }
            }

            if (x == 12 || x == 13) && (y == 0 || y == 1) {
                col = handle_color;
            }

            if y == 14 && (x == 3 || x == 7 || x == 11) {
                col = nail_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }


    Texture { texels }
}

fn cherry_leaves(fancy: bool) -> Texture {
    let base = [0.92, 0.55, 0.72];
    let light = [0.98, 0.75, 0.88];
    let white = [1.0, 0.94, 0.97];
    let dark_twig = [0.55, 0.28, 0.40];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let noise = rand_at(x, y, 77);
            let mut col = if noise > 0.82 {
                light
            } else if noise > 0.94 {
                white
            } else if noise < 0.15 {
                dark_twig
            } else {
                base
            };
            col[0] = (col[0] + (noise - 0.5) * 0.06).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.06).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.06).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x + y * 2) % 4 == 0 && noise > 0.46)
                    || ((x ^ y) % 5 == 0 && noise > 0.62)
                    || ((x * 3 + y) % 7 == 0 && noise > 0.52);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn autumn_leaves(fancy: bool) -> Texture {
    let base = [0.75, 0.32, 0.08];
    let orange = [0.88, 0.48, 0.10];
    let yellow = [0.92, 0.72, 0.12];
    let red = [0.72, 0.15, 0.08];
    let mut texels = vec![[0.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let noise = rand_at(x, y, 223);
            let mut col = if noise > 0.75 {
                orange
            } else if noise > 0.50 {
                yellow
            } else if noise > 0.30 {
                red
            } else {
                base
            };
            col[0] = (col[0] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[1] = (col[1] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);
            col[2] = (col[2] + (noise - 0.5) * 0.08).clamp(0.0, 1.0);

            if fancy {
                let hole = ((x + y) % 3 == 0 && noise > 0.42)
                    || ((x ^ y) % 5 == 0 && noise > 0.58)
                    || ((x * 2 + y * 3) % 7 == 0 && noise > 0.50);
                if hole {
                    texels[y * TEX_SIZE + x] = [-col[0] - 0.001, col[1], col[2]];
                    continue;
                }
            }
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn mud() -> Texture {
    // Dark muddy brown with subtle wet sheen variation
    let base = [0.30, 0.20, 0.12];
    let dark = [0.20, 0.13, 0.07];
    let wet  = [0.38, 0.27, 0.17];
    let mut tex = speckled(base, 0.20, 557);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 613);
            if r > 0.85 {
                tex.texels[y * TEX_SIZE + x] = dark;
            } else if r > 0.70 {
                tex.texels[y * TEX_SIZE + x] = wet;
            }
            // cracked lines
            if (x == 4 || x == 11) && y % 5 == 2 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
            if (y == 7 || y == 13) && x % 6 == 3 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextureId {
    GrassTop,
    GrassSide,
    Dirt,
    Stone,
    Sand,
    WoodTop,
    WoodSide,
    Leaves,
    Water,
    Snow,
    CactusTop,
    CactusSide,
    CoalOre,
    IronOre,
    Bedrock,
    Lava,
    SnowLeaves,
    Planks,
    Door,
    DoorTop,
    Glass,
    Ice,
    TntTop,
    TntSide,
    PigSkin,
    PigFace,
    PigSnout,
    CowSkin,
    CowFace,
    CowSnout,
    CowHorns,
    CowUdder,
    CraftingTableTop,
    CraftingTableSide,
    CraftingTableFront,
    DestroyStage0,
    DestroyStage1,
    DestroyStage2,
    DestroyStage3,
    DestroyStage4,
    DestroyStage5,
    DestroyStage6,
    DestroyStage7,
    DestroyStage8,
    DestroyStage9,
    DamageFlash,
    UnderwaterTint,
    VignetteTint,
    LavaTint,
    /// Pink cherry-blossom leaves texture
    CherryLeaves,
    /// Autumn/fall leaves in orange-red tones
    AutumnLeaves,
    /// Muddy block for swamp biome
    Mud,
    /// Torch texture
    Torch,
    /// Round circular entity / item shadow
    EntityShadow,
    // Biome Foliage Tinting Variants
    GrassTopSwamp,
    GrassSideSwamp,
    LeavesSwamp,
    GrassTopSavanna,
    GrassSideSavanna,
    LeavesSavanna,
    GrassTopCherry,
    GrassSideCherry,
    GrassTopAutumn,
    GrassSideAutumn,
    GrassTopTaiga,
    GrassSideTaiga,
    // Chest & Furnace Textures
    ChestTop,
    ChestSide,
    ChestFront,
    FurnaceTop,
    FurnaceSide,
    FurnaceFront,
    FurnaceFrontLit,
    IronBlock,
    GoldBlock,
    GoldOre,
    CoalBlock,
    RawChicken,
    CookedChicken,
    Feather,
    ChickenFeathers,
    ChickenFace,
    ChickenBeak,
    ChickenWattle,
    ChickenLeg,
    PigSpawnEgg,
    CowSpawnEgg,
    ChickenSpawnEgg,
    RawPorkchop,
    CookedPorkchop,
    RawBeef,
    CookedBeef,
    Apple,
    IronIngot,
    GoldIngot,
    IronOunce,
    GoldOunce,
    Bone,
    // Player / Steve Textures
    PlayerSkin,
    PlayerFace,
    PlayerHair,
    PlayerShirt,
    PlayerShirtFront,
    PlayerPants,
    PlayerShoes,
    // Diamond, Bed & Architecture
    DiamondOre,
    DiamondBlock,
    Diamond,
    BedTop,
    BedSide,
    Ladder,
    // Mob Textures (Zombie & Skeleton)
    ZombieSkin,
    ZombieFace,
    ZombieShirt,
    ZombiePants,
    SkeletonBone,
    SkeletonFace,
    SkeletonBody,
    CreeperSkin,
    CreeperFace,
    Gunpowder,
    Farmland,
    FarmlandWet,
    FarmlandSide,
    WheatSeeds,
    WheatCrop,
    Wheat,
    Bread,
    Carrot,
    CarrotCrop,
    Potato,
    PotatoCrop,
    BakedPotato,
    Sapling,
    BoneMeal,
    /// Custom modded texture (index 0..=63)
    Custom(u8),
}

impl TextureId {
    #[inline(always)]
    pub fn is_destroy_stage(self) -> bool {
        matches!(
            self,
            TextureId::DestroyStage0
                | TextureId::DestroyStage1
                | TextureId::DestroyStage2
                | TextureId::DestroyStage3
                | TextureId::DestroyStage4
                | TextureId::DestroyStage5
                | TextureId::DestroyStage6
                | TextureId::DestroyStage7
                | TextureId::DestroyStage8
                | TextureId::DestroyStage9
        )
    }

    pub fn destroy_stage(stage: usize) -> Self {
        match stage {
            0 => TextureId::DestroyStage0,
            1 => TextureId::DestroyStage1,
            2 => TextureId::DestroyStage2,
            3 => TextureId::DestroyStage3,
            4 => TextureId::DestroyStage4,
            5 => TextureId::DestroyStage5,
            6 => TextureId::DestroyStage6,
            7 => TextureId::DestroyStage7,
            8 => TextureId::DestroyStage8,
            _ => TextureId::DestroyStage9,
        }
    }
}

fn glass() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let line = x == y || x + y == TEX_SIZE - 1;
            texels[y * TEX_SIZE + x] = if line { [0.72, 0.95, 1.0] } else { [0.40, 0.72, 0.88] };
        }
    }
    Texture { texels }
}

fn pig_skin() -> Texture {
    speckled([0.94, 0.65, 0.65], 0.08, 301)
}

fn pig_face() -> Texture {
    let mut tex = pig_skin();
    let white = [0.98, 0.98, 0.98];
    let pupil = [0.12, 0.08, 0.08];
    let snout_bg = [0.86, 0.52, 0.52];
    let nostril = [0.46, 0.22, 0.22];

    tex.texels[6 * TEX_SIZE + 2] = white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = white;

    for y in 9..14 {
        for x in 4..12 {
            tex.texels[y * TEX_SIZE + x] = snout_bg;
        }
    }
    tex.texels[10 * TEX_SIZE + 6] = nostril;
    tex.texels[11 * TEX_SIZE + 6] = nostril;
    tex.texels[10 * TEX_SIZE + 9] = nostril;
    tex.texels[11 * TEX_SIZE + 9] = nostril;

    tex
}

fn pig_snout() -> Texture {
    let mut tex = speckled([0.88, 0.54, 0.54], 0.06, 302);
    let nostril = [0.46, 0.22, 0.22];
    for y in 6..11 {
        for x in 4..7 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
        for x in 9..12 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
    }
    tex
}

fn cow_skin() -> Texture {
    let dark = [0.26, 0.18, 0.14];
    let white = [0.88, 0.87, 0.84];
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let bx = x / 4;
            let by = y / 4;
            let n = rand_at(bx, by, 701) + rand_at(x, y, 702) * 0.3;
            let is_white = n > 0.62;
            let shade = 0.92 + rand_at(x, y, 703) * 0.16;
            let base = if is_white { white } else { dark };
            texels[y * TEX_SIZE + x] = [
                (base[0] * shade).clamp(0.0, 1.0),
                (base[1] * shade).clamp(0.0, 1.0),
                (base[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

fn cow_face() -> Texture {
    let mut tex = cow_skin();
    let white = [0.90, 0.89, 0.86];
    let eye_white = [0.98, 0.98, 0.98];
    let pupil = [0.10, 0.08, 0.08];
    let muzzle = [0.68, 0.58, 0.58];
    let nostril = [0.22, 0.16, 0.16];

    for y in 1..8 {
        for x in 6..10 {
            let s = 0.95 + rand_at(x, y, 705) * 0.1;
            tex.texels[y * TEX_SIZE + x] = [white[0] * s, white[1] * s, white[2] * s];
        }
    }

    tex.texels[6 * TEX_SIZE + 2] = eye_white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = eye_white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = eye_white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = eye_white;

    for y in 9..15 {
        for x in 3..13 {
            tex.texels[y * TEX_SIZE + x] = muzzle;
        }
    }
    tex.texels[11 * TEX_SIZE + 5] = nostril;
    tex.texels[12 * TEX_SIZE + 5] = nostril;
    tex.texels[11 * TEX_SIZE + 10] = nostril;
    tex.texels[12 * TEX_SIZE + 10] = nostril;

    tex
}

fn cow_snout() -> Texture {
    let mut tex = speckled([0.68, 0.58, 0.58], 0.06, 706);
    let nostril = [0.22, 0.16, 0.16];
    for y in 6..11 {
        for x in 4..7 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
        for x in 9..12 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
    }
    tex
}

fn cow_horns() -> Texture {
    let mut tex = speckled([0.76, 0.74, 0.70], 0.08, 707);
    let dark = [0.48, 0.46, 0.42];
    for y in 12..16 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = dark;
        }
    }
    tex
}

fn cow_udder() -> Texture {
    let mut tex = speckled([0.88, 0.65, 0.68], 0.05, 708);
    let teat = [0.68, 0.42, 0.45];
    for (tx, ty) in [(4, 4), (11, 4), (4, 11), (11, 11)] {
        for dy in 0..2 {
            for dx in 0..2 {
                tex.texels[(ty + dy) * TEX_SIZE + (tx + dx)] = teat;
            }
        }
    }
    tex
}

fn destroy_stage(stage: usize) -> Texture {
    #[rustfmt::skip]
    const CRACK_RANK: [[u8; 16]; 16] = [
        [255, 255,   8,   7,   8, 255, 255, 255,   8,   7,   8, 255, 255,   9, 255, 255],
        [255,   8,   6,   5,   6,   7, 255,   8,   6,   5,   6,   8, 255,   8,   9, 255],
        [  9,   7,   5,   3,   4,   5,   6,   5,   4,   3,   5,   7,   8,   7,   8, 255],
        [255,   8,   5,   3,   2,   2,   3,   3,   2,   2,   4,   6,   7,   6,   8,   9],
        [255,   7,   4,   2,   1,   1,   2,   2,   1,   1,   3,   5,   6,   5,   7, 255],
        [255,   8,   5,   3,   1,   0,   0,   1,   0,   1,   3,   4,   5,   6,   8, 255],
        [  8,   6,   4,   2,   1,   0,   0,   0,   0,   1,   2,   4,   5,   7, 255, 255],
        [  7,   5,   3,   1,   0,   0,   0,   0,   0,   0,   1,   3,   5,   6,   8, 255],
        [  8,   6,   4,   2,   1,   0,   0,   0,   0,   1,   2,   4,   6,   7,   9, 255],
        [255,   7,   5,   3,   1,   0,   1,   0,   1,   2,   4,   5,   7,   8, 255, 255],
        [255,   8,   6,   4,   2,   1,   2,   2,   2,   3,   4,   6,   7, 255, 255, 255],
        [255, 255,   7,   5,   3,   2,   3,   4,   3,   4,   5,   7,   8,   9, 255, 255],
        [255,   9,   8,   6,   4,   3,   4,   5,   5,   5,   6,   8, 255,   8,   9, 255],
        [255,   8,   7,   5,   4,   5,   6,   7,   6,   7,   8, 255, 255,   7,   8, 255],
        [255, 255,   8,   7,   6,   7, 255,   8,   7,   8, 255, 255,   8,   7,   8, 255],
        [255, 255, 255,   9,   8, 255, 255, 255,   9, 255, 255, 255, 255,   9, 255, 255],
    ];

    let mut texels = vec![[-1.0, -1.0, -1.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let rank = CRACK_RANK[y][x];
            if (stage as u8) >= rank {
                let n = rand_at(x, y, 999) * 0.04;
                let shade = (0.07 + n).clamp(0.04, 0.15);
                texels[y * TEX_SIZE + x] = [shade, shade, shade];
            }
        }
    }
    Texture { texels }
}

fn crafting_table_top() -> Texture {
    let mut tex = planks();
    let border = [0.32, 0.20, 0.10];
    let grid_line = [0.28, 0.18, 0.08];
    let tool_metal = [0.80, 0.82, 0.85];
    let tool_handle = [0.45, 0.28, 0.12];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            // Outer frame
            if x <= 1 || x >= 14 || y <= 1 || y >= 14 {
                tex.texels[y * TEX_SIZE + x] = border;
            }
            // 3x3 crafting grid in center (x: 3..13, y: 3..13)
            if (x >= 3 && x <= 12) && (y >= 3 && y <= 12) {
                if x == 6 || x == 9 || y == 6 || y == 9 {
                    tex.texels[y * TEX_SIZE + x] = grid_line;
                }
            }
        }
    }
    // Small saw & shears detail on top corners
    tex.texels[2 * TEX_SIZE + 2] = tool_metal;
    tex.texels[2 * TEX_SIZE + 3] = tool_handle;
    tex.texels[13 * TEX_SIZE + 12] = tool_metal;
    tex.texels[13 * TEX_SIZE + 13] = tool_metal;
    tex
}

fn crafting_table_side() -> Texture {
    let mut tex = planks();
    let frame = [0.30, 0.18, 0.08];
    let iron = [0.75, 0.78, 0.82];
    let handle = [0.40, 0.24, 0.10];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                tex.texels[y * TEX_SIZE + x] = frame;
            }
        }
    }
    // Saw hanging on side
    for y in 4..12 {
        tex.texels[y * TEX_SIZE + 4] = iron;
        if y % 2 == 0 {
            tex.texels[y * TEX_SIZE + 5] = iron;
        }
    }
    tex.texels[3 * TEX_SIZE + 4] = handle;
    tex.texels[2 * TEX_SIZE + 4] = handle;
    tex
}

fn crafting_table_front() -> Texture {
    let mut tex = planks();
    let frame = [0.30, 0.18, 0.08];
    let iron = [0.75, 0.78, 0.82];
    let handle = [0.40, 0.24, 0.10];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                tex.texels[y * TEX_SIZE + x] = frame;
            }
        }
    }
    // Hammer and shears on front
    for y in 4..11 {
        tex.texels[y * TEX_SIZE + 11] = handle;
    }
    for x in 9..14 {
        tex.texels[4 * TEX_SIZE + x] = iron;
    }
    tex
}

pub struct TextureAtlas {
    pub grass_top: Texture,
    pub grass_side: Texture,
    pub dirt: Texture,
    pub stone: Texture,
    pub sand: Texture,
    pub wood_top: Texture,
    pub wood_side: Texture,
    pub leaves: Texture,
    pub water: Texture,
    pub snow: Texture,
    pub cactus_top: Texture,
    pub cactus_side: Texture,
    pub coal_ore: Texture,
    pub iron_ore: Texture,
    pub bedrock: Texture,
    pub lava: Texture,
    pub snow_leaves: Texture,
    pub planks: Texture,
    pub door: Texture,
    pub door_top: Texture,
    pub glass: Texture,
    pub ice: Texture,
    pub tnt_top: Texture,
    pub tnt_side: Texture,
    pub pig_skin: Texture,
    pub pig_face: Texture,
    pub pig_snout: Texture,
    pub cow_skin: Texture,
    pub cow_face: Texture,
    pub cow_snout: Texture,
    pub cow_horns: Texture,
    pub cow_udder: Texture,
    pub crafting_table_top: Texture,
    pub crafting_table_side: Texture,
    pub crafting_table_front: Texture,
    pub destroy_stages: [Texture; 10],
    pub damage_flash: Texture,
    pub underwater_tint: Texture,
    pub vignette_tint: Texture,
    pub lava_tint: Texture,
    pub cherry_leaves: Texture,
    pub autumn_leaves: Texture,
    pub mud: Texture,
    pub torch: Texture,
    pub entity_shadow: Texture,
    pub grass_top_swamp: Texture,
    pub grass_side_swamp: Texture,
    pub leaves_swamp: Texture,
    pub grass_top_savanna: Texture,
    pub grass_side_savanna: Texture,
    pub leaves_savanna: Texture,
    pub grass_top_cherry: Texture,
    pub grass_side_cherry: Texture,
    pub grass_top_autumn: Texture,
    pub grass_side_autumn: Texture,
    pub grass_top_taiga: Texture,
    pub grass_side_taiga: Texture,
    pub chest_top: Texture,
    pub chest_side: Texture,
    pub chest_front: Texture,
    pub furnace_top: Texture,
    pub furnace_side: Texture,
    pub furnace_front: Texture,
    pub furnace_front_lit: Texture,
    pub iron_block: Texture,
    pub gold_block: Texture,
    pub gold_ore: Texture,
    pub coal_block: Texture,
    pub raw_chicken: Texture,
    pub cooked_chicken: Texture,
    pub feather: Texture,
    pub chicken_feathers: Texture,
    pub chicken_face: Texture,
    pub chicken_beak: Texture,
    pub chicken_wattle: Texture,
    pub chicken_leg: Texture,
    pub pig_spawn_egg: Texture,
    pub cow_spawn_egg: Texture,
    pub chicken_spawn_egg: Texture,
    pub raw_porkchop: Texture,
    pub cooked_porkchop: Texture,
    pub raw_beef: Texture,
    pub cooked_beef: Texture,
    pub apple: Texture,
    pub iron_ingot: Texture,
    pub gold_ingot: Texture,
    pub iron_ounce: Texture,
    pub gold_ounce: Texture,
    pub bone: Texture,
    pub player_skin: Texture,
    pub player_face: Texture,
    pub player_hair: Texture,
    pub player_shirt: Texture,
    pub player_shirt_front: Texture,
    pub player_pants: Texture,
    pub player_shoes: Texture,
    pub diamond_ore: Texture,
    pub diamond_block: Texture,
    pub diamond: Texture,
    pub bed_top: Texture,
    pub bed_side: Texture,
    pub ladder: Texture,
    pub zombie_skin: Texture,
    pub zombie_face: Texture,
    pub zombie_shirt: Texture,
    pub zombie_pants: Texture,
    pub skeleton_bone: Texture,
    pub skeleton_face: Texture,
    pub skeleton_body: Texture,
    pub creeper_skin: Texture,
    pub creeper_face: Texture,
    pub gunpowder: Texture,
    pub farmland: Texture,
    pub farmland_wet: Texture,
    pub farmland_side: Texture,
    pub wheat_seeds: Texture,
    pub wheat_crop: Texture,
    pub wheat: Texture,
    pub bread: Texture,
    pub carrot: Texture,
    pub carrot_crop: Texture,
    pub potato: Texture,
    pub potato_crop: Texture,
    pub baked_potato: Texture,
    pub sapling: Texture,
    pub bone_meal: Texture,
    pub custom: Vec<Texture>,
}

fn diamond_ore() -> Texture {
    let mut tex = stone();
    let diamond_base = [0.22, 0.88, 0.92];
    let diamond_hi = [0.65, 0.98, 1.00];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 771);
            if n > 0.80 {
                let col = if n > 0.92 { diamond_hi } else { diamond_base };
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], col, 0.92);
            }
        }
    }
    tex
}

fn diamond_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.32, 0.88, 0.92];
    let dark = [0.18, 0.65, 0.72];
    let hi = [0.75, 0.98, 1.00];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_border = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let n = rand_at(x, y, 882);
            let col = if is_border {
                dark
            } else if n > 0.82 {
                hi
            } else if n < 0.25 {
                dark
            } else {
                base
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn diamond() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.28, 0.85, 0.90];
    let hi = [0.82, 0.98, 1.00];
    let shadow = [0.12, 0.52, 0.60];
    let border = [0.08, 0.35, 0.42];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32 - 7.5;
            let inside = if dy < 0.0 {
                dx + dy.abs() * 0.8 <= 5.0
            } else {
                dx + dy * 0.9 <= 5.0
            };
            if inside {
                let n = rand_at(x, y, 991);
                let col = if dx + dy.abs() >= 4.5 {
                    border
                } else if n > 0.75 {
                    hi
                } else if dy > 1.5 {
                    shadow
                } else {
                    base
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

fn bed_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let pillow_base = [0.90, 0.90, 0.92];
    let pillow_shade = [0.78, 0.78, 0.82];
    let blanket_base = [0.82, 0.16, 0.16];
    let blanket_dark = [0.65, 0.10, 0.10];
    let blanket_light = [0.92, 0.25, 0.25];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 412);
            if y < 5 {
                let is_edge = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == 4;
                let col = if is_edge || n > 0.75 { pillow_shade } else { pillow_base };
                texels[y * TEX_SIZE + x] = col;
            } else {
                let col = if y == 5 || n < 0.2 {
                    blanket_dark
                } else if n > 0.8 {
                    blanket_light
                } else {
                    blanket_base
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

fn bed_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood = [0.55, 0.38, 0.20];
    let wood_dark = [0.38, 0.24, 0.12];
    let blanket = [0.82, 0.16, 0.16];
    let pillow = [0.88, 0.88, 0.90];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_leg = (x <= 1 || x >= 14) && y < 6;
            let n = rand_at(x, y, 513);
            let col = if y < 4 || is_leg {
                if n > 0.6 { wood_dark } else { wood }
            } else if y >= 11 && x < 5 {
                pillow
            } else {
                if n > 0.6 { [0.70, 0.12, 0.12] } else { blanket }
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn ladder() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood_light = [0.65, 0.45, 0.25];
    let wood_dark = [0.42, 0.28, 0.14];
    let nail = [0.20, 0.20, 0.22];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_rail = (x >= 2 && x <= 4) || (x >= 11 && x <= 13);
            let is_rung = (y % 4 == 1 || y % 4 == 2) && x >= 4 && x <= 11;
            let is_nail = (x == 3 || x == 12) && (y % 4 == 1);
            let n = rand_at(x, y, 614);
            let col = if is_nail {
                nail
            } else if is_rail || is_rung {
                if n > 0.5 { wood_light } else { wood_dark }
            } else {
                [0.0, 0.0, 0.0]
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn chest_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood_base = [0.58, 0.40, 0.22];
    let wood_dark = [0.45, 0.28, 0.14];
    let rim_dark = [0.22, 0.14, 0.08];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_rim = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let n = rand_at(x, y, 331);
            let mut col = if is_rim {
                rim_dark
            } else if n > 0.65 {
                wood_dark
            } else {
                wood_base
            };
            col[0] = (col[0] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn chest_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood_base = [0.55, 0.38, 0.20];
    let wood_dark = [0.42, 0.26, 0.12];
    let rim_dark = [0.22, 0.14, 0.08];
    let lock_split = [0.15, 0.10, 0.06];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_rim = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let is_split = y == 4;
            let n = rand_at(x, y, 442);
            let mut col = if is_split || is_rim {
                if is_split { lock_split } else { rim_dark }
            } else if n > 0.65 {
                wood_dark
            } else {
                wood_base
            };
            col[0] = (col[0] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn chest_front() -> Texture {
    let mut base = chest_side();
    let lock_silver = [0.85, 0.85, 0.88];
    let lock_shadow = [0.35, 0.35, 0.38];
    let keyhole = [0.12, 0.12, 0.14];
    // Center lock latch
    for y in 3..=6 {
        for x in 7..=8 {
            base.texels[y * TEX_SIZE + x] = if y == 3 || y == 6 { lock_shadow } else { lock_silver };
        }
    }
    base.texels[4 * TEX_SIZE + 7] = keyhole;
    base.texels[4 * TEX_SIZE + 8] = keyhole;
    base
}

fn furnace_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let stone_base = [0.48, 0.48, 0.48];
    let ring_dark = [0.32, 0.32, 0.32];
    let ring_light = [0.60, 0.60, 0.60];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 + 0.5) - 8.0;
            let dy = (y as f32 + 0.5) - 8.0;
            let r = (dx * dx + dy * dy).sqrt();
            let n = rand_at(x, y, 719);
            let mut col = if r >= 4.0 && r <= 6.0 {
                if n > 0.5 { ring_light } else { ring_dark }
            } else {
                stone_base
            };
            col[0] = (col[0] + (n - 0.5) * 0.08).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.08).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.08).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn furnace_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let stone_base = [0.48, 0.48, 0.48];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 823);
            let mut col = stone_base;
            let var = (n - 0.5) * 0.16;
            col[0] = (col[0] + var).clamp(0.0, 1.0);
            col[1] = (col[1] + var).clamp(0.0, 1.0);
            col[2] = (col[2] + var).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn furnace_front() -> Texture {
    let mut base = furnace_side();
    let dark_opening = [0.12, 0.12, 0.14];
    let dark_grate = [0.22, 0.22, 0.24];
    let rim_light = [0.65, 0.65, 0.65];
    for y in 2..=10 {
        for x in 3..=12 {
            let is_border = x == 3 || x == 12 || y == 2 || y == 10;
            if is_border {
                base.texels[y * TEX_SIZE + x] = rim_light;
            } else {
                let is_grate = (y == 5 || y == 8) || (x % 3 == 0);
                base.texels[y * TEX_SIZE + x] = if is_grate { dark_grate } else { dark_opening };
            }
        }
    }
    base
}

fn furnace_front_lit() -> Texture {
    let mut base = furnace_side();
    let fire_core = [1.0, 0.95, 0.40];
    let fire_mid = [1.0, 0.60, 0.10];
    let fire_dark = [0.85, 0.25, 0.05];
    let rim_lit = [0.80, 0.65, 0.40];
    for y in 2..=10 {
        for x in 3..=12 {
            let is_border = x == 3 || x == 12 || y == 2 || y == 10;
            if is_border {
                base.texels[y * TEX_SIZE + x] = rim_lit;
            } else {
                let n = rand_at(x, y, 991);
                let dist_y = (10 - y) as f32 / 8.0;
                let col = if dist_y + n * 0.4 > 0.85 {
                    fire_dark
                } else if dist_y + n * 0.4 > 0.45 {
                    fire_mid
                } else {
                    fire_core
                };
                base.texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    base
}

fn iron_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.86, 0.86, 0.88];
    let highlight = [0.95, 0.95, 0.98];
    let shadow = [0.70, 0.70, 0.74];
    let rivet = [0.55, 0.55, 0.60];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_top_left = x == 0 || y == 0;
            let is_bot_right = x == TEX_SIZE - 1 || y == TEX_SIZE - 1;
            let is_rivet = (x == 2 && y == 2)
                || (x == TEX_SIZE - 3 && y == 2)
                || (x == 2 && y == TEX_SIZE - 3)
                || (x == TEX_SIZE - 3 && y == TEX_SIZE - 3);
            let n = rand_at(x, y, 771);
            let mut col = if is_top_left {
                highlight
            } else if is_bot_right {
                shadow
            } else if is_rivet {
                rivet
            } else {
                base
            };
            col[0] = (col[0] + (n - 0.5) * 0.03).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.03).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.03).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn gold_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.95, 0.82, 0.22];
    let highlight = [1.0, 0.95, 0.45];
    let shadow = [0.78, 0.62, 0.10];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_top_left = x == 0 || y == 0 || (x == 2 && y >= 2 && y <= TEX_SIZE - 3) || (y == 2 && x >= 2 && x <= TEX_SIZE - 3);
            let is_bot_right = x == TEX_SIZE - 1 || y == TEX_SIZE - 1 || (x == TEX_SIZE - 3 && y >= 2 && y <= TEX_SIZE - 3) || (y == TEX_SIZE - 3 && x >= 2 && x <= TEX_SIZE - 3);
            let n = rand_at(x, y, 882);
            let mut col = if is_top_left {
                highlight
            } else if is_bot_right {
                shadow
            } else {
                base
            };
            col[0] = (col[0] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn gold_ore() -> Texture {
    let mut tex = stone();
    let gold_color = [0.98, 0.82, 0.20];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 631);
            if n > 0.75 {
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], gold_color, 0.90);
            }
        }
    }
    tex
}

fn coal_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.12, 0.12, 0.13];
    let dark = [0.06, 0.06, 0.07];
    let light = [0.18, 0.18, 0.20];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 993);
            let mut col = if n > 0.75 {
                light
            } else if n < 0.25 {
                dark
            } else {
                base
            };
            col[0] = (col[0] + (n - 0.5) * 0.02).clamp(0.0, 1.0);
            col[1] = (col[1] + (n - 0.5) * 0.02).clamp(0.0, 1.0);
            col[2] = (col[2] + (n - 0.5) * 0.02).clamp(0.0, 1.0);
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn torch() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood_dark = [0.42, 0.28, 0.14];
    let wood_light = [0.55, 0.38, 0.20];
    let coal = [0.15, 0.14, 0.14];
    let flame_outer = [0.98, 0.42, 0.05];
    let flame_mid = [1.0, 0.78, 0.12];
    let flame_core = [1.0, 0.98, 0.70];

    // Background default (darker transparent or wood tone)
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            texels[y * TEX_SIZE + x] = [0.0, 0.0, 0.0];
        }
    }

    // Wooden stick (x: 6..10, y: 0..10)
    for y in 0..10 {
        for x in 6..10 {
            let n = rand_at(x, y, 77);
            let col = if n > 0.5 { wood_light } else { wood_dark };
            texels[y * TEX_SIZE + x] = col;
        }
    }

    // Coal/wick top (x: 6..10, y: 9..11)
    for y in 9..12 {
        for x in 6..10 {
            texels[y * TEX_SIZE + x] = coal;
        }
    }

    // Flame (y: 11..16, x: 5..11)
    for y in 11..16 {
        for x in 5..11 {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32 - 12.0;
            if dx <= 2.2 && dy <= 3.5 {
                let dist = dx * dx + dy * dy * 0.5;
                let col = if dist < 1.0 {
                    flame_core
                } else if dist < 3.0 {
                    flame_mid
                } else {
                    flame_outer
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }

    Texture { texels }
}

fn entity_shadow() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 + 0.5) - 8.0;
            let dy = (y as f32 + 0.5) - 8.0;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= 7.0 {
                texels[y * TEX_SIZE + x] = [0.0, 0.0, 0.0];
            }
        }
    }
    Texture { texels }
}

fn raw_chicken() -> Texture {
    let mut tex = speckled([0.90, 0.70, 0.65], 0.08, 881);
    let bone = [0.95, 0.95, 0.90];
    let meat_dark = [0.80, 0.52, 0.50];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x < 3 && y < 3) || (x > 12 && y > 12) {
                tex.texels[y * TEX_SIZE + x] = bone;
            } else if (x + y) % 3 == 0 {
                tex.texels[y * TEX_SIZE + x] = meat_dark;
            }
        }
    }
    tex
}

fn cooked_chicken() -> Texture {
    let mut tex = speckled([0.72, 0.45, 0.20], 0.10, 882);
    let crispy = [0.55, 0.30, 0.12];
    let golden = [0.85, 0.60, 0.25];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 883);
            if n > 0.75 {
                tex.texels[y * TEX_SIZE + x] = golden;
            } else if n < 0.25 {
                tex.texels[y * TEX_SIZE + x] = crispy;
            }
        }
    }
    tex
}

fn feather() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let white = [0.96, 0.96, 0.98];
    let gray = [0.82, 0.84, 0.88];
    let quill = [0.65, 0.60, 0.55];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let diag = x as i32 - y as i32;
            if diag == 0 {
                texels[y * TEX_SIZE + x] = quill;
            } else if diag.abs() <= 3 && y >= 2 && y <= 14 {
                let col = if (x + y) % 2 == 0 { white } else { gray };
                texels[y * TEX_SIZE + x] = col;
            } else {
                texels[y * TEX_SIZE + x] = [0.0, 0.0, 0.0];
            }
        }
    }
    Texture { texels }
}

fn chicken_feathers() -> Texture {
    speckled([0.95, 0.95, 0.93], 0.04, 884)
}

fn chicken_face() -> Texture {
    let mut tex = chicken_feathers();
    let eye_white = [0.99, 0.99, 0.99];
    let pupil = [0.05, 0.05, 0.05];
    let pupil_shine = [0.35, 0.35, 0.40];
    let beak = [0.96, 0.68, 0.10];
    let wattle = [0.88, 0.12, 0.12];

    // High-contrast, bold chicken eyes at the top corners of the face
    // Left eye (x=1..4, y=2..5)
    for y in 2..=5 {
        for x in 1..=4 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
    }
    tex.texels[3 * TEX_SIZE + 2] = pupil;
    tex.texels[4 * TEX_SIZE + 2] = pupil;
    tex.texels[3 * TEX_SIZE + 3] = pupil;
    tex.texels[4 * TEX_SIZE + 3] = pupil;
    tex.texels[2 * TEX_SIZE + 3] = pupil_shine;

    // Right eye (x=11..14, y=2..5)
    for y in 2..=5 {
        for x in 11..=14 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
    }
    tex.texels[3 * TEX_SIZE + 12] = pupil;
    tex.texels[4 * TEX_SIZE + 12] = pupil;
    tex.texels[3 * TEX_SIZE + 13] = pupil;
    tex.texels[4 * TEX_SIZE + 13] = pupil;
    tex.texels[2 * TEX_SIZE + 12] = pupil_shine;

    // Beak center in lower middle
    for y in 7..=10 {
        for x in 5..=10 {
            tex.texels[y * TEX_SIZE + x] = beak;
        }
    }

    // Wattle under beak
    for y in 11..=14 {
        for x in 6..=9 {
            tex.texels[y * TEX_SIZE + x] = wattle;
        }
    }

    tex
}

fn chicken_beak() -> Texture {
    speckled([0.95, 0.65, 0.10], 0.05, 885)
}

fn chicken_wattle() -> Texture {
    speckled([0.85, 0.15, 0.15], 0.05, 886)
}

fn chicken_leg() -> Texture {
    speckled([0.92, 0.60, 0.10], 0.05, 887)
}

fn spawn_egg_texture(base: [f32; 3], spot: [f32; 3], outline: [f32; 3], seed: u32) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 2..=14 {
        for x in 3..=12 {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32;
            let width_at_y = if dy < 6.0 {
                (dy - 1.0) * 0.7 + 0.5
            } else if dy <= 11.0 {
                4.2
            } else {
                (14.5 - dy) * 1.1 + 0.5
            };
            if dx <= width_at_y {
                let is_border = dx > width_at_y - 0.9 || y == 2 || y == 14;
                if is_border {
                    texels[y * TEX_SIZE + x] = outline;
                } else {
                    let n = rand_at(x, y, seed);
                    let is_spot = n > 0.75;
                    let mut col = if is_spot { spot } else { base };
                    col[0] = (col[0] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
                    col[1] = (col[1] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
                    col[2] = (col[2] + (n - 0.5) * 0.04).clamp(0.0, 1.0);
                    texels[y * TEX_SIZE + x] = col;
                }
            }
        }
    }
    Texture { texels }
}

fn pig_spawn_egg() -> Texture {
    spawn_egg_texture([0.95, 0.62, 0.68], [0.68, 0.32, 0.38], [0.48, 0.20, 0.25], 701)
}

fn cow_spawn_egg() -> Texture {
    spawn_egg_texture([0.42, 0.30, 0.22], [0.85, 0.85, 0.85], [0.22, 0.15, 0.10], 702)
}

fn chicken_spawn_egg() -> Texture {
    spawn_egg_texture([0.95, 0.95, 0.92], [0.85, 0.15, 0.15], [0.35, 0.35, 0.30], 703)
}

fn raw_porkchop() -> Texture {
    let mut tex = speckled([0.92, 0.58, 0.64], 0.08, 901);
    let bone = [0.95, 0.95, 0.88];
    let hi = [0.98, 0.78, 0.82];
    let sh = [0.75, 0.38, 0.45];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x < 3 && y < 3) || (x == 2 && y == 3) {
                tex.texels[y * TEX_SIZE + x] = bone;
            } else if x <= 5 && y <= 6 {
                tex.texels[y * TEX_SIZE + x] = hi;
            } else if x >= 11 || y >= 11 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn cooked_porkchop() -> Texture {
    let mut tex = speckled([0.72, 0.42, 0.18], 0.10, 902);
    let bone = [0.92, 0.90, 0.82];
    let hi = [0.90, 0.65, 0.32];
    let sear = [0.45, 0.22, 0.08];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x < 3 && y < 3) || (x == 2 && y == 3) {
                tex.texels[y * TEX_SIZE + x] = bone;
            } else if (x + y) % 4 == 0 {
                tex.texels[y * TEX_SIZE + x] = sear;
            } else if x <= 5 && y <= 6 {
                tex.texels[y * TEX_SIZE + x] = hi;
            }
        }
    }
    tex
}

fn raw_beef() -> Texture {
    let mut tex = speckled([0.82, 0.20, 0.20], 0.10, 903);
    let fat = [0.96, 0.90, 0.88];
    let sh = [0.52, 0.10, 0.10];
    let hi = [0.94, 0.42, 0.42];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x == 5 && y == 7) || (x == 6 && y == 8) || (x == 9 && y == 6) || (x == 10 && y == 7) || (x + y == 12 && x % 2 == 0) {
                tex.texels[y * TEX_SIZE + x] = fat;
            } else if x <= 4 && y <= 5 {
                tex.texels[y * TEX_SIZE + x] = hi;
            } else if x >= 11 || y >= 11 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn cooked_beef() -> Texture {
    let mut tex = speckled([0.50, 0.26, 0.14], 0.10, 904);
    let grill = [0.18, 0.08, 0.04];
    let hi = [0.70, 0.42, 0.22];
    let sh = [0.28, 0.12, 0.06];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x + y == 10) || (x + y == 14) || (x + y == 18) {
                tex.texels[y * TEX_SIZE + x] = grill;
            } else if x <= 4 && y <= 5 {
                tex.texels[y * TEX_SIZE + x] = hi;
            } else if x >= 11 || y >= 11 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn apple() -> Texture {
    let mut tex = speckled([0.88, 0.14, 0.14], 0.08, 905);
    let stem = [0.42, 0.26, 0.12];
    let leaf = [0.35, 0.78, 0.22];
    let hi = [0.98, 0.48, 0.48];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x == 7 || x == 8) && (y == 0 || y == 1) {
                tex.texels[y * TEX_SIZE + x] = stem;
            } else if (x == 9 || x == 10) && y == 1 {
                tex.texels[y * TEX_SIZE + x] = leaf;
            } else if (x >= 4 && x <= 6) && (y >= 4 && y <= 6) {
                tex.texels[y * TEX_SIZE + x] = hi;
            }
        }
    }
    tex
}

fn iron_ingot() -> Texture {
    let mut tex = speckled([0.85, 0.85, 0.88], 0.06, 906);
    let hi = [0.96, 0.96, 0.98];
    let sh = [0.65, 0.65, 0.70];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x <= 2 || y <= 2 {
                tex.texels[y * TEX_SIZE + x] = hi;
            } else if x >= 13 || y >= 13 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn gold_ingot() -> Texture {
    let mut tex = speckled([0.95, 0.82, 0.22], 0.06, 907);
    let hi = [1.00, 0.95, 0.55];
    let sh = [0.75, 0.58, 0.10];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x <= 2 || y <= 2 {
                tex.texels[y * TEX_SIZE + x] = hi;
            } else if x >= 13 || y >= 13 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn iron_ounce() -> Texture {
    speckled([0.80, 0.80, 0.82], 0.08, 908)
}

fn gold_ounce() -> Texture {
    speckled([0.92, 0.78, 0.20], 0.08, 909)
}

fn bone() -> Texture {
    let mut tex = speckled([0.95, 0.95, 0.88], 0.04, 910);
    let sh = [0.82, 0.80, 0.72];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if (x + y) % 5 == 0 {
                tex.texels[y * TEX_SIZE + x] = sh;
            }
        }
    }
    tex
}

fn player_skin() -> Texture {
    speckled([0.76, 0.54, 0.42], 0.04, 501)
}

fn player_hair() -> Texture {
    speckled([0.28, 0.18, 0.12], 0.05, 502)
}

fn player_face() -> Texture {
    let mut tex = player_skin();
    let hair = [0.28, 0.18, 0.12];
    let white = [0.96, 0.96, 0.96];
    let eye_blue = [0.18, 0.22, 0.58];
    let nose = [0.66, 0.44, 0.34];
    let mouth_beard = [0.34, 0.20, 0.14];

    // Top hair band (y = 0..3 full width, y = 3..5 sides)
    for y in 0..3 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
    }
    for y in 3..6 {
        for x in 0..2 {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
        for x in (TEX_SIZE - 2)..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
    }

    // Eyes: y = 7..9
    // Left eye: white at x = 3..4, pupil blue at x = 5..6
    for y in 7..9 {
        tex.texels[y * TEX_SIZE + 3] = white;
        tex.texels[y * TEX_SIZE + 4] = white;
        tex.texels[y * TEX_SIZE + 5] = eye_blue;
        tex.texels[y * TEX_SIZE + 6] = eye_blue;

        // Right eye: pupil blue at x = 9..10, white at x = 11..12
        tex.texels[y * TEX_SIZE + 9] = eye_blue;
        tex.texels[y * TEX_SIZE + 10] = eye_blue;
        tex.texels[y * TEX_SIZE + 11] = white;
        tex.texels[y * TEX_SIZE + 12] = white;
    }

    // Nose: y = 9..11, center
    for y in 9..11 {
        for x in 7..9 {
            tex.texels[y * TEX_SIZE + x] = nose;
        }
    }

    // Mouth / Beard: y = 12..14
    for y in 12..14 {
        for x in 5..11 {
            tex.texels[y * TEX_SIZE + x] = mouth_beard;
        }
    }

    tex
}

fn player_shirt() -> Texture {
    speckled([0.0, 0.65, 0.68], 0.05, 503)
}

fn player_shirt_front() -> Texture {
    let mut tex = player_shirt();
    let skin = [0.76, 0.54, 0.42];
    // V-neck neckline cutout at top center
    for y in 0..2 {
        for x in 5..11 {
            tex.texels[y * TEX_SIZE + x] = skin;
        }
    }
    for y in 2..4 {
        for x in 6..10 {
            tex.texels[y * TEX_SIZE + x] = skin;
        }
    }
    for x in 7..9 {
        tex.texels[4 * TEX_SIZE + x] = skin;
    }
    tex
}

fn player_pants() -> Texture {
    speckled([0.18, 0.24, 0.48], 0.05, 504)
}

fn player_shoes() -> Texture {
    speckled([0.20, 0.20, 0.22], 0.04, 505)
}

fn zombie_skin() -> Texture {
    speckled([0.31, 0.48, 0.24], 0.08, 401)
}

fn zombie_face() -> Texture {
    let mut tex = zombie_skin();
    let dark_eye = [0.08, 0.12, 0.06];
    let pupil = [0.02, 0.02, 0.02];
    let mouth = [0.12, 0.18, 0.09];
    let dark_nose = [0.18, 0.28, 0.14];

    for dy in 6..8 {
        tex.texels[dy * TEX_SIZE + 3] = dark_eye;
        tex.texels[dy * TEX_SIZE + 4] = pupil;
        tex.texels[dy * TEX_SIZE + 11] = pupil;
        tex.texels[dy * TEX_SIZE + 12] = dark_eye;
    }
    tex.texels[9 * TEX_SIZE + 7] = dark_nose;
    tex.texels[9 * TEX_SIZE + 8] = dark_nose;

    for x in 5..11 {
        tex.texels[12 * TEX_SIZE + x] = mouth;
    }
    tex
}

fn zombie_shirt() -> Texture {
    speckled([0.16, 0.42, 0.44], 0.06, 402)
}

fn zombie_pants() -> Texture {
    speckled([0.22, 0.18, 0.42], 0.06, 403)
}

fn skeleton_bone() -> Texture {
    speckled([0.84, 0.83, 0.80], 0.06, 501)
}

fn skeleton_face() -> Texture {
    let mut tex = skeleton_bone();
    let eye_socket = [0.10, 0.10, 0.10];
    let dark_slit = [0.22, 0.22, 0.22];

    for dy in 5..8 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = eye_socket;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = eye_socket;
        }
    }

    tex.texels[9 * TEX_SIZE + 7] = eye_socket;
    tex.texels[9 * TEX_SIZE + 8] = eye_socket;

    for x in 4..12 {
        if x % 2 == 0 {
            tex.texels[12 * TEX_SIZE + x] = dark_slit;
            tex.texels[13 * TEX_SIZE + x] = dark_slit;
        }
    }
    tex
}

fn skeleton_body() -> Texture {
    let mut tex = skeleton_bone();
    let shadow = [0.20, 0.20, 0.20];
    for y in 0..TEX_SIZE {
        tex.texels[y * TEX_SIZE + 7] = shadow;
        tex.texels[y * TEX_SIZE + 8] = shadow;
    }
    for y in [4, 7, 10, 13] {
        for x in 2..14 {
            if x != 7 && x != 8 {
                tex.texels[y * TEX_SIZE + x] = shadow;
            }
        }
    }
    tex
}

fn creeper_skin() -> Texture {
    let mut tex = speckled([0.22, 0.62, 0.16], 0.18, 601);
    let lime = [0.38, 0.76, 0.22];
    let dark = [0.12, 0.38, 0.08];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 602);
            if r > 0.72 {
                tex.texels[y * TEX_SIZE + x] = lime;
            } else if r < 0.28 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

fn creeper_face() -> Texture {
    let mut tex = creeper_skin();
    let black = [0.03, 0.03, 0.03];

    // Left and right eyes (3x3 blocks)
    for dy in 4..7 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Center vertical bridge between eyes
    for dy in 7..9 {
        for dx in 7..9 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Top mouth bar
    for dy in 9..11 {
        for dx in 5..11 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Mouth corners dropping down
    for dy in 11..14 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    tex
}

fn gunpowder() -> Texture {
    speckled([0.24, 0.24, 0.26], 0.12, 701)
}

fn farmland() -> Texture {
    let mut tex = speckled([0.34, 0.22, 0.12], 0.12, 101);
    let furrow = [0.22, 0.14, 0.08];
    for y in 0..TEX_SIZE {
        if y % 3 == 0 {
            for x in 0..TEX_SIZE {
                tex.texels[y * TEX_SIZE + x] = furrow;
            }
        }
    }
    tex
}

fn farmland_wet() -> Texture {
    let mut tex = speckled([0.20, 0.13, 0.07], 0.10, 102);
    let furrow = [0.12, 0.08, 0.04];
    let glint = [0.35, 0.28, 0.22];
    for y in 0..TEX_SIZE {
        if y % 3 == 0 {
            for x in 0..TEX_SIZE {
                tex.texels[y * TEX_SIZE + x] = furrow;
            }
        } else {
            for x in 0..TEX_SIZE {
                if (x + y * 7) % 11 == 0 {
                    tex.texels[y * TEX_SIZE + x] = glint;
                }
            }
        }
    }
    tex
}

fn farmland_side() -> Texture {
    let mut tex = speckled([0.45, 0.33, 0.20], 0.15, 7);
    let top_dirt = [0.34, 0.22, 0.12];
    for y in 0..2 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = top_dirt;
        }
    }
    tex
}

fn wheat_seeds() -> Texture {
    let mut tex = speckled([0.22, 0.16, 0.10], 0.10, 201);
    let seed_body = [0.75, 0.70, 0.35];
    let seed_tip = [0.45, 0.65, 0.25];
    let coords = [(4, 5), (8, 4), (11, 7), (6, 9), (9, 11), (5, 12)];
    for (cx, cy) in coords {
        tex.texels[cy * TEX_SIZE + cx] = seed_body;
        tex.texels[(cy + 1) * TEX_SIZE + cx] = seed_body;
        tex.texels[cy * TEX_SIZE + (cx + 1)] = seed_tip;
    }
    tex
}

fn wheat_crop() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let stem = [0.35, 0.68, 0.20];
    let ear = [0.88, 0.78, 0.32];
    let ear_dark = [0.72, 0.60, 0.20];
    for y in 2..TEX_SIZE {
        texels[y * TEX_SIZE + 7] = stem;
        texels[y * TEX_SIZE + 8] = stem;
        if y >= 6 && y % 3 == 0 {
            texels[y * TEX_SIZE + 5] = stem;
            texels[y * TEX_SIZE + 6] = stem;
            texels[y * TEX_SIZE + 9] = stem;
            texels[y * TEX_SIZE + 10] = stem;
        }
    }
    for y in 2..7 {
        for x in 6..10 {
            texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { ear } else { ear_dark };
        }
    }
    Texture { texels }
}

fn wheat() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let gold = [0.88, 0.76, 0.28];
    let gold_dark = [0.70, 0.55, 0.18];
    let ribbon = [0.75, 0.20, 0.15];
    for y in 3..14 {
        for x in 5..11 {
            let col = if y == 8 || y == 9 {
                ribbon
            } else if (x + y) % 2 == 0 {
                gold
            } else {
                gold_dark
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

fn bread() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let crust = [0.55, 0.32, 0.12];
    let crumb = [0.82, 0.62, 0.32];
    let score = [0.42, 0.24, 0.08];
    for y in 5..12 {
        for x in 3..13 {
            let is_edge = y == 5 || y == 11 || x == 3 || x == 12;
            let is_score = (x == 6 || x == 9) && (y >= 6 && y <= 10);
            texels[y * TEX_SIZE + x] = if is_score {
                score
            } else if is_edge {
                crust
            } else {
                crumb
            };
        }
    }
    Texture { texels }
}

fn carrot() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let orange = [0.96, 0.48, 0.08];
    let orange_dark = [0.80, 0.35, 0.05];
    let green = [0.25, 0.68, 0.15];
    texels[2 * TEX_SIZE + 7] = green;
    texels[2 * TEX_SIZE + 8] = green;
    texels[3 * TEX_SIZE + 6] = green;
    texels[3 * TEX_SIZE + 9] = green;
    for y in 4..14 {
        let width = ((14 - y) / 2).max(1);
        for x in (8 - width)..=(7 + width) {
            texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { orange } else { orange_dark };
        }
    }
    Texture { texels }
}

fn carrot_crop() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let green = [0.28, 0.70, 0.18];
    let green_dark = [0.18, 0.48, 0.12];
    let orange = [0.96, 0.48, 0.08];
    for y in 4..13 {
        for x in 4..12 {
            if (x + y * 2) % 3 != 0 {
                texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { green } else { green_dark };
            }
        }
    }
    for x in 6..10 {
        texels[14 * TEX_SIZE + x] = orange;
    }
    Texture { texels }
}

fn potato() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.68, 0.52, 0.32];
    let eye = [0.42, 0.30, 0.16];
    let hi = [0.78, 0.62, 0.38];
    for y in 4..12 {
        for x in 4..12 {
            texels[y * TEX_SIZE + x] = if (x == 6 && y == 6) || (x == 9 && y == 9) || (x == 7 && y == 10) {
                eye
            } else if x <= 6 && y <= 6 {
                hi
            } else {
                base
            };
        }
    }
    Texture { texels }
}

fn potato_crop() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let green = [0.30, 0.62, 0.22];
    let flower = [0.95, 0.95, 0.88];
    for y in 5..14 {
        for x in 4..12 {
            if (x * 3 + y * 5) % 4 != 0 {
                texels[y * TEX_SIZE + x] = green;
            }
        }
    }
    texels[4 * TEX_SIZE + 6] = flower;
    texels[4 * TEX_SIZE + 9] = flower;
    Texture { texels }
}

fn baked_potato() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let skin = [0.55, 0.38, 0.20];
    let inside = [0.92, 0.85, 0.45];
    let butter = [0.98, 0.92, 0.30];
    for y in 4..12 {
        for x in 4..12 {
            let is_cut = (x >= 6 && x <= 9) && (y >= 6 && y <= 9);
            let is_butter = (x == 7 || x == 8) && (y == 7 || y == 8);
            texels[y * TEX_SIZE + x] = if is_butter {
                butter
            } else if is_cut {
                inside
            } else {
                skin
            };
        }
    }
    Texture { texels }
}

fn sapling() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let stem = [0.45, 0.30, 0.15];
    let leaf_hi = [0.38, 0.75, 0.25];
    let leaf_lo = [0.24, 0.55, 0.16];
    for y in 7..15 {
        texels[y * TEX_SIZE + 7] = stem;
        texels[y * TEX_SIZE + 8] = stem;
    }
    for y in 2..8 {
        for x in 4..12 {
            if (x == 4 || x == 11) && (y == 2 || y == 7) {
                continue;
            }
            texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { leaf_hi } else { leaf_lo };
        }
    }
    Texture { texels }
}

fn bone_meal() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let white = [0.95, 0.95, 0.92];
    let shadow = [0.78, 0.78, 0.75];
    for y in 5..12 {
        for x in 4..12 {
            if (x + y) % 3 != 0 {
                texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { white } else { shadow };
            }
        }
    }
    Texture { texels }
}

impl TextureAtlas {
    pub fn generate() -> TextureAtlas {
        Self::generate_with_fancy(true)
    }

    pub fn generate_with_fancy(fancy_leaves: bool) -> TextureAtlas {
        TextureAtlas {
            grass_top: grass_top(),
            grass_side: grass_side(),
            dirt: dirt(),
            stone: stone(),
            sand: sand(),
            wood_top: wood_top(),
            wood_side: wood_side(),
            leaves: leaves(fancy_leaves),
            water: water(),
            snow: snow(),
            cactus_top: cactus_top(),
            cactus_side: cactus_side(),
            coal_ore: coal_ore(),
            iron_ore: iron_ore(),
            bedrock: bedrock(),
            lava: lava(),
            snow_leaves: snow_leaves(fancy_leaves),
            planks: planks(),
            door: door_bottom(),
            door_top: door_top(),
            glass: glass(),
            ice: speckled([0.70, 0.88, 1.0], 0.08, 903),
            tnt_top: tnt_top(),
            tnt_side: tnt_side(),
            pig_skin: pig_skin(),
            pig_face: pig_face(),
            pig_snout: pig_snout(),
            cow_skin: cow_skin(),
            cow_face: cow_face(),
            cow_snout: cow_snout(),
            cow_horns: cow_horns(),
            cow_udder: cow_udder(),
            crafting_table_top: crafting_table_top(),
            crafting_table_side: crafting_table_side(),
            crafting_table_front: crafting_table_front(),
            destroy_stages: std::array::from_fn(destroy_stage),
            damage_flash: speckled([0.95, 0.05, 0.05], 0.0, 0),
            underwater_tint: speckled([0.05, 0.25, 0.75], 0.0, 0),
            vignette_tint: speckled([0.85, 0.02, 0.02], 0.0, 0),
            lava_tint: speckled([0.95, 0.25, 0.02], 0.0, 0),
            cherry_leaves: cherry_leaves(fancy_leaves),
            autumn_leaves: autumn_leaves(fancy_leaves),
            mud: mud(),
            torch: torch(),
            entity_shadow: entity_shadow(),
            grass_top_swamp: grass_top_swamp(),
            grass_side_swamp: grass_side_swamp(),
            leaves_swamp: leaves_swamp(fancy_leaves),
            grass_top_savanna: grass_top_savanna(),
            grass_side_savanna: grass_side_savanna(),
            leaves_savanna: leaves_savanna(fancy_leaves),
            grass_top_cherry: grass_top_cherry(),
            grass_side_cherry: grass_side_cherry(),
            grass_top_autumn: grass_top_autumn(),
            grass_side_autumn: grass_side_autumn(),
            grass_top_taiga: grass_top_taiga(),
            grass_side_taiga: grass_side_taiga(),
            chest_top: chest_top(),
            chest_side: chest_side(),
            chest_front: chest_front(),
            furnace_top: furnace_top(),
            furnace_side: furnace_side(),
            furnace_front: furnace_front(),
            furnace_front_lit: furnace_front_lit(),
            iron_block: iron_block(),
            gold_block: gold_block(),
            gold_ore: gold_ore(),
            coal_block: coal_block(),
            raw_chicken: raw_chicken(),
            cooked_chicken: cooked_chicken(),
            feather: feather(),
            chicken_feathers: chicken_feathers(),
            chicken_face: chicken_face(),
            chicken_beak: chicken_beak(),
            chicken_wattle: chicken_wattle(),
            chicken_leg: chicken_leg(),
            pig_spawn_egg: pig_spawn_egg(),
            cow_spawn_egg: cow_spawn_egg(),
            chicken_spawn_egg: chicken_spawn_egg(),
            raw_porkchop: raw_porkchop(),
            cooked_porkchop: cooked_porkchop(),
            raw_beef: raw_beef(),
            cooked_beef: cooked_beef(),
            apple: apple(),
            iron_ingot: iron_ingot(),
            gold_ingot: gold_ingot(),
            iron_ounce: iron_ounce(),
            gold_ounce: gold_ounce(),
            bone: bone(),
            player_skin: player_skin(),
            player_face: player_face(),
            player_hair: player_hair(),
            player_shirt: player_shirt(),
            player_shirt_front: player_shirt_front(),
            player_pants: player_pants(),
            player_shoes: player_shoes(),
            diamond_ore: diamond_ore(),
            diamond_block: diamond_block(),
            diamond: diamond(),
            bed_top: bed_top(),
            bed_side: bed_side(),
            ladder: ladder(),
            zombie_skin: zombie_skin(),
            zombie_face: zombie_face(),
            zombie_shirt: zombie_shirt(),
            zombie_pants: zombie_pants(),
            skeleton_bone: skeleton_bone(),
            skeleton_face: skeleton_face(),
            skeleton_body: skeleton_body(),
            creeper_skin: creeper_skin(),
            creeper_face: creeper_face(),
            gunpowder: gunpowder(),
            farmland: farmland(),
            farmland_wet: farmland_wet(),
            farmland_side: farmland_side(),
            wheat_seeds: wheat_seeds(),
            wheat_crop: wheat_crop(),
            wheat: wheat(),
            bread: bread(),
            carrot: carrot(),
            carrot_crop: carrot_crop(),
            potato: potato(),
            potato_crop: potato_crop(),
            baked_potato: baked_potato(),
            sapling: sapling(),
            bone_meal: bone_meal(),
            custom: crate::engine::modding::generate_mod_textures(),
        }
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        match id {
            TextureId::GrassTop => &self.grass_top,
            TextureId::GrassSide => &self.grass_side,
            TextureId::Dirt => &self.dirt,
            TextureId::Stone => &self.stone,
            TextureId::Sand => &self.sand,
            TextureId::WoodTop => &self.wood_top,
            TextureId::WoodSide => &self.wood_side,
            TextureId::Leaves => &self.leaves,
            TextureId::Water => &self.water,
            TextureId::Snow => &self.snow,
            TextureId::CactusTop => &self.cactus_top,
            TextureId::CactusSide => &self.cactus_side,
            TextureId::CoalOre => &self.coal_ore,
            TextureId::IronOre => &self.iron_ore,
            TextureId::Bedrock => &self.bedrock,
            TextureId::Lava => &self.lava,
            TextureId::SnowLeaves => &self.snow_leaves,
            TextureId::Planks => &self.planks,
            TextureId::Door => &self.door,
            TextureId::DoorTop => &self.door_top,
            TextureId::Glass => &self.glass,
            TextureId::Ice => &self.ice,
            TextureId::TntTop => &self.tnt_top,
            TextureId::TntSide => &self.tnt_side,
            TextureId::PigSkin => &self.pig_skin,
            TextureId::PigFace => &self.pig_face,
            TextureId::PigSnout => &self.pig_snout,
            TextureId::CowSkin => &self.cow_skin,
            TextureId::CowFace => &self.cow_face,
            TextureId::CowSnout => &self.cow_snout,
            TextureId::CowHorns => &self.cow_horns,
            TextureId::CowUdder => &self.cow_udder,
            TextureId::CraftingTableTop => &self.crafting_table_top,
            TextureId::CraftingTableSide => &self.crafting_table_side,
            TextureId::CraftingTableFront => &self.crafting_table_front,
            TextureId::DestroyStage0 => &self.destroy_stages[0],
            TextureId::DestroyStage1 => &self.destroy_stages[1],
            TextureId::DestroyStage2 => &self.destroy_stages[2],
            TextureId::DestroyStage3 => &self.destroy_stages[3],
            TextureId::DestroyStage4 => &self.destroy_stages[4],
            TextureId::DestroyStage5 => &self.destroy_stages[5],
            TextureId::DestroyStage6 => &self.destroy_stages[6],
            TextureId::DestroyStage7 => &self.destroy_stages[7],
            TextureId::DestroyStage8 => &self.destroy_stages[8],
            TextureId::DestroyStage9 => &self.destroy_stages[9],
            TextureId::DamageFlash => &self.damage_flash,
            TextureId::UnderwaterTint => &self.underwater_tint,
            TextureId::VignetteTint => &self.vignette_tint,
            TextureId::LavaTint => &self.lava_tint,
            TextureId::CherryLeaves => &self.cherry_leaves,
            TextureId::AutumnLeaves => &self.autumn_leaves,
            TextureId::Mud => &self.mud,
            TextureId::Torch => &self.torch,
            TextureId::EntityShadow => &self.entity_shadow,
            TextureId::GrassTopSwamp => &self.grass_top_swamp,
            TextureId::GrassSideSwamp => &self.grass_side_swamp,
            TextureId::LeavesSwamp => &self.leaves_swamp,
            TextureId::GrassTopSavanna => &self.grass_top_savanna,
            TextureId::GrassSideSavanna => &self.grass_side_savanna,
            TextureId::LeavesSavanna => &self.leaves_savanna,
            TextureId::GrassTopCherry => &self.grass_top_cherry,
            TextureId::GrassSideCherry => &self.grass_side_cherry,
            TextureId::GrassTopAutumn => &self.grass_top_autumn,
            TextureId::GrassSideAutumn => &self.grass_side_autumn,
            TextureId::GrassTopTaiga => &self.grass_top_taiga,
            TextureId::GrassSideTaiga => &self.grass_side_taiga,
            TextureId::ChestTop => &self.chest_top,
            TextureId::ChestSide => &self.chest_side,
            TextureId::ChestFront => &self.chest_front,
            TextureId::FurnaceTop => &self.furnace_top,
            TextureId::FurnaceSide => &self.furnace_side,
            TextureId::FurnaceFront => &self.furnace_front,
            TextureId::FurnaceFrontLit => &self.furnace_front_lit,
            TextureId::IronBlock => &self.iron_block,
            TextureId::GoldBlock => &self.gold_block,
            TextureId::GoldOre => &self.gold_ore,
            TextureId::CoalBlock => &self.coal_block,
            TextureId::RawChicken => &self.raw_chicken,
            TextureId::CookedChicken => &self.cooked_chicken,
            TextureId::Feather => &self.feather,
            TextureId::ChickenFeathers => &self.chicken_feathers,
            TextureId::ChickenFace => &self.chicken_face,
            TextureId::ChickenBeak => &self.chicken_beak,
            TextureId::ChickenWattle => &self.chicken_wattle,
            TextureId::ChickenLeg => &self.chicken_leg,
            TextureId::PigSpawnEgg => &self.pig_spawn_egg,
            TextureId::CowSpawnEgg => &self.cow_spawn_egg,
            TextureId::ChickenSpawnEgg => &self.chicken_spawn_egg,
            TextureId::RawPorkchop => &self.raw_porkchop,
            TextureId::CookedPorkchop => &self.cooked_porkchop,
            TextureId::RawBeef => &self.raw_beef,
            TextureId::CookedBeef => &self.cooked_beef,
            TextureId::Apple => &self.apple,
            TextureId::IronIngot => &self.iron_ingot,
            TextureId::GoldIngot => &self.gold_ingot,
            TextureId::IronOunce => &self.iron_ounce,
            TextureId::GoldOunce => &self.gold_ounce,
            TextureId::Bone => &self.bone,
            TextureId::PlayerSkin => &self.player_skin,
            TextureId::PlayerFace => &self.player_face,
            TextureId::PlayerHair => &self.player_hair,
            TextureId::PlayerShirt => &self.player_shirt,
            TextureId::PlayerShirtFront => &self.player_shirt_front,
            TextureId::PlayerPants => &self.player_pants,
            TextureId::PlayerShoes => &self.player_shoes,
            TextureId::DiamondOre => &self.diamond_ore,
            TextureId::DiamondBlock => &self.diamond_block,
            TextureId::Diamond => &self.diamond,
            TextureId::BedTop => &self.bed_top,
            TextureId::BedSide => &self.bed_side,
            TextureId::Ladder => &self.ladder,
            TextureId::ZombieSkin => &self.zombie_skin,
            TextureId::ZombieFace => &self.zombie_face,
            TextureId::ZombieShirt => &self.zombie_shirt,
            TextureId::ZombiePants => &self.zombie_pants,
            TextureId::SkeletonBone => &self.skeleton_bone,
            TextureId::SkeletonFace => &self.skeleton_face,
            TextureId::SkeletonBody => &self.skeleton_body,
            TextureId::CreeperSkin => &self.creeper_skin,
            TextureId::CreeperFace => &self.creeper_face,
            TextureId::Gunpowder => &self.gunpowder,
            TextureId::Farmland => &self.farmland,
            TextureId::FarmlandWet => &self.farmland_wet,
            TextureId::FarmlandSide => &self.farmland_side,
            TextureId::WheatSeeds => &self.wheat_seeds,
            TextureId::WheatCrop => &self.wheat_crop,
            TextureId::Wheat => &self.wheat,
            TextureId::Bread => &self.bread,
            TextureId::Carrot => &self.carrot,
            TextureId::CarrotCrop => &self.carrot_crop,
            TextureId::Potato => &self.potato,
            TextureId::PotatoCrop => &self.potato_crop,
            TextureId::BakedPotato => &self.baked_potato,
            TextureId::Sapling => &self.sapling,
            TextureId::BoneMeal => &self.bone_meal,
            TextureId::Custom(idx) => self.custom.get(idx as usize).unwrap_or(&self.stone),
        }
    }
    /// Returns a reference to the texture corresponding to the specified `TextureId`.
    pub fn texture_for(&self, id: TextureId) -> &Texture {
        self.get(id)
    }
}
