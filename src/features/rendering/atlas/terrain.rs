//! Procedural 16x16 terrain, stone, biome grass, fluid, and mud textures.

use super::types::{mix, rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn grass_top() -> Texture {
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

pub(super) fn grass_side() -> Texture {
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

pub(super) fn grass_top_swamp() -> Texture {
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

pub(super) fn grass_side_swamp() -> Texture {
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

pub(super) fn grass_top_savanna() -> Texture {
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

pub(super) fn grass_side_savanna() -> Texture {
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

pub(super) fn grass_top_cherry() -> Texture {
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

pub(super) fn grass_side_cherry() -> Texture {
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

pub(super) fn grass_top_autumn() -> Texture {
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

pub(super) fn grass_side_autumn() -> Texture {
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

pub(super) fn grass_top_taiga() -> Texture {
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

pub(super) fn grass_side_taiga() -> Texture {
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

pub(super) fn dirt() -> Texture {
    speckled([0.45, 0.33, 0.20], 0.20, 3)
}

pub(super) fn stone() -> Texture {
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

pub(super) fn sand() -> Texture {
    speckled([0.87, 0.80, 0.55], 0.12, 5)
}

pub(super) fn water() -> Texture {
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

pub(super) fn snow() -> Texture {
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

pub(super) fn bedrock() -> Texture {
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

pub(super) fn lava() -> Texture {
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

pub(super) fn mud() -> Texture {
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
