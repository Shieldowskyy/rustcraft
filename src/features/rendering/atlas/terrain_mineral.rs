//! Procedural 16x16 clay, gravel, cobblestone, sandstone, smooth stone, and obsidian textures.

use super::types::{mix, rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn clay() -> Texture {
    let base = [0.62, 0.64, 0.69];
    let dark = [0.52, 0.54, 0.58];
    let light = [0.70, 0.72, 0.76];
    let mut tex = speckled(base, 0.12, 117);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 142);
            if r > 0.90 {
                tex.texels[y * TEX_SIZE + x] = light;
            } else if r < 0.12 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

pub(super) fn gravel() -> Texture {
    let base = [0.54, 0.51, 0.48];
    let mut tex = speckled(base, 0.25, 43);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 89);
            if r > 0.88 {
                tex.texels[y * TEX_SIZE + x] = [0.72, 0.68, 0.62];
            } else if r < 0.12 {
                tex.texels[y * TEX_SIZE + x] = [0.35, 0.33, 0.32];
            }
        }
    }
    tex
}

pub(super) fn cobblestone() -> Texture {
    let base = [0.55, 0.55, 0.55];
    let mut tex = speckled(base, 0.20, 57);
    let crevice = [0.28, 0.28, 0.30];
    let highlight = [0.68, 0.68, 0.70];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let h = ((x * 5 + y * 7) ^ (x * 3)) % 11;
            if h == 0 || (x % 5 == 0 && y % 4 == 0) || (x % 6 == 3 && y % 5 == 2) {
                tex.texels[y * TEX_SIZE + x] = crevice;
            } else if h == 1 {
                tex.texels[y * TEX_SIZE + x] = highlight;
            }
        }
    }
    tex
}

pub(super) fn mossy_cobblestone() -> Texture {
    let mut tex = cobblestone();
    let moss = [0.32, 0.56, 0.18];
    let dark_moss = [0.20, 0.38, 0.12];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 92);
            let cluster = ((x as f32 * 0.4).sin() + (y as f32 * 0.4).cos()) > 0.35;
            if cluster && r > 0.30 {
                let col = if r > 0.65 { moss } else { dark_moss };
                tex.texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    tex
}

pub(super) fn sandstone_top() -> Texture {
    let base = [0.86, 0.82, 0.62];
    let mut tex = speckled(base, 0.08, 19);
    for i in 0..TEX_SIZE {
        tex.texels[i] = mix(tex.texels[i], [0.76, 0.72, 0.54], 0.4);
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = mix(tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i], [0.76, 0.72, 0.54], 0.4);
        tex.texels[i * TEX_SIZE] = mix(tex.texels[i * TEX_SIZE], [0.76, 0.72, 0.54], 0.4);
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = mix(tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)], [0.76, 0.72, 0.54], 0.4);
    }
    tex
}

pub(super) fn sandstone_side() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let light = [0.88, 0.84, 0.64];
    let mid = [0.84, 0.80, 0.60];
    let dark = [0.78, 0.74, 0.54];
    for y in 0..TEX_SIZE {
        let layer_col = if y < 4 {
            mid
        } else if y == 4 || y == 11 {
            dark
        } else if y < 11 {
            light
        } else {
            mid
        };
        for x in 0..TEX_SIZE {
            let n = (rand_at(x, y, 48) - 0.5) * 0.08;
            texels[y * TEX_SIZE + x] = [
                (layer_col[0] + n).clamp(0.0, 1.0),
                (layer_col[1] + n).clamp(0.0, 1.0),
                (layer_col[2] + n).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

pub(super) fn sandstone_bottom() -> Texture {
    speckled([0.80, 0.76, 0.56], 0.10, 22)
}

pub(super) fn smooth_stone() -> Texture {
    let base = [0.68, 0.68, 0.70];
    let mut tex = speckled(base, 0.06, 33);
    let border = [0.48, 0.48, 0.50];
    let inner_hl = [0.78, 0.78, 0.80];
    for i in 0..TEX_SIZE {
        tex.texels[i] = border;
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = border;
        tex.texels[i * TEX_SIZE] = border;
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = border;
    }
    for i in 1..TEX_SIZE - 1 {
        tex.texels[TEX_SIZE + i] = inner_hl;
        tex.texels[i * TEX_SIZE + 1] = inner_hl;
    }
    tex
}

pub(super) fn obsidian() -> Texture {
    let base = [0.10, 0.08, 0.14];
    let mut tex = speckled(base, 0.35, 99);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 73);
            if r > 0.92 {
                tex.texels[y * TEX_SIZE + x] = [0.28, 0.18, 0.36];
            } else if r > 0.85 {
                tex.texels[y * TEX_SIZE + x] = [0.18, 0.14, 0.26];
            } else if r < 0.08 {
                tex.texels[y * TEX_SIZE + x] = [0.04, 0.03, 0.06];
            }
        }
    }
    tex
}
