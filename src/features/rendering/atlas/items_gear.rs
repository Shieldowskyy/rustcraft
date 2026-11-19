//! Procedural 16x16 tools, armor swatches, fishing rod, ingots, and mob drop item textures.

use super::types::{speckled, Texture, TEX_SIZE};

pub(super) fn feather() -> Texture {
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

pub(super) fn iron_ingot() -> Texture {
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

pub(super) fn gold_ingot() -> Texture {
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

pub(super) fn iron_ounce() -> Texture {
    speckled([0.80, 0.80, 0.82], 0.08, 908)
}

pub(super) fn gold_ounce() -> Texture {
    speckled([0.92, 0.78, 0.20], 0.08, 909)
}

pub(super) fn bone() -> Texture {
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

pub(super) fn gunpowder() -> Texture {
    speckled([0.24, 0.24, 0.26], 0.12, 701)
}

pub(super) fn shears_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let iron_hi = [0.90, 0.90, 0.95];
    let iron_mid = [0.72, 0.72, 0.76];
    let iron_sh = [0.45, 0.45, 0.50];
    let screw = [0.30, 0.30, 0.32];

    for i in 2..=13 {
        texels[i * TEX_SIZE + i] = iron_mid;
        if i + 1 < TEX_SIZE { texels[i * TEX_SIZE + (i + 1)] = iron_hi; }
        if i > 0 { texels[i * TEX_SIZE + (i - 1)] = iron_sh; }

        let other_x = 15 - i;
        texels[i * TEX_SIZE + other_x] = iron_mid;
        if other_x + 1 < TEX_SIZE { texels[i * TEX_SIZE + (other_x + 1)] = iron_hi; }
        if other_x > 0 { texels[i * TEX_SIZE + (other_x - 1)] = iron_sh; }
    }
    texels[7 * TEX_SIZE + 7] = screw;
    texels[7 * TEX_SIZE + 8] = screw;
    texels[8 * TEX_SIZE + 7] = screw;
    texels[8 * TEX_SIZE + 8] = screw;

    Texture { texels }
}

pub(super) fn string_item() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let silk = [0.92, 0.92, 0.92];
    let silk_dim = [0.68, 0.68, 0.70];
    let coords = [
        (3, 12), (4, 11), (5, 10), (6, 9), (7, 8),
        (8, 7), (9, 6), (10, 5), (11, 4), (12, 3),
        (5, 12), (6, 11), (7, 10), (8, 9), (9, 8),
        (10, 7), (11, 6), (7, 6), (8, 5)
    ];
    for &(x, y) in &coords {
        texels[y * TEX_SIZE + x] = silk;
        if y + 1 < TEX_SIZE && texels[(y + 1) * TEX_SIZE + x][0] < 0.0 {
            texels[(y + 1) * TEX_SIZE + x] = silk_dim;
        }
    }
    Texture { texels }
}

pub(super) fn spider_eye_texture() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let red_bright = [0.92, 0.12, 0.12];
    let red_dark = [0.55, 0.06, 0.06];
    let highlight = [1.00, 0.65, 0.65];
    let pupil = [0.15, 0.02, 0.02];
    for y in 4..12 {
        for x in 4..12 {
            let dx = (x as f32 - 7.5).abs();
            let dy = (y as f32 - 7.5).abs();
            if dx * dx + dy * dy <= 12.0 {
                texels[y * TEX_SIZE + x] = red_dark;
            }
        }
    }
    for y in 5..11 {
        for x in 5..11 {
            let dx = (x as f32 - 7.5).abs();
            let dy = (y as f32 - 7.5).abs();
            if dx * dx + dy * dy <= 7.0 {
                texels[y * TEX_SIZE + x] = red_bright;
            }
        }
    }
    texels[6 * TEX_SIZE + 6] = highlight;
    texels[7 * TEX_SIZE + 7] = pupil;
    texels[7 * TEX_SIZE + 8] = pupil;
    texels[8 * TEX_SIZE + 7] = pupil;
    texels[8 * TEX_SIZE + 8] = pupil;
    Texture { texels }
}

pub(super) fn leather_armor() -> Texture {
    let mut tex = speckled([0.55, 0.32, 0.16], 0.10, 401);
    let seam = [0.40, 0.22, 0.10];
    for i in 0..TEX_SIZE {
        tex.texels[i] = seam;
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = seam;
        tex.texels[i * TEX_SIZE] = seam;
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = seam;
    }
    tex
}

pub(super) fn iron_armor() -> Texture {
    let mut tex = speckled([0.82, 0.82, 0.85], 0.08, 402);
    let highlight = [0.94, 0.94, 0.97];
    let shadow = [0.65, 0.65, 0.68];
    for i in 0..TEX_SIZE {
        tex.texels[i] = highlight;
        tex.texels[i * TEX_SIZE] = highlight;
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = shadow;
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = shadow;
    }
    tex
}

pub(super) fn gold_armor() -> Texture {
    let mut tex = speckled([0.94, 0.82, 0.22], 0.08, 403);
    let highlight = [1.00, 0.95, 0.45];
    let shadow = [0.72, 0.58, 0.12];
    for i in 0..TEX_SIZE {
        tex.texels[i] = highlight;
        tex.texels[i * TEX_SIZE] = highlight;
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = shadow;
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = shadow;
    }
    tex
}

pub(super) fn diamond_armor() -> Texture {
    let mut tex = speckled([0.22, 0.88, 0.92], 0.08, 404);
    let highlight = [0.60, 0.98, 1.00];
    let shadow = [0.12, 0.65, 0.72];
    for i in 0..TEX_SIZE {
        tex.texels[i] = highlight;
        tex.texels[i * TEX_SIZE] = highlight;
        tex.texels[(TEX_SIZE - 1) * TEX_SIZE + i] = shadow;
        tex.texels[i * TEX_SIZE + (TEX_SIZE - 1)] = shadow;
    }
    tex
}

pub(super) fn fishing_rod_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let wood_dark = [0.42, 0.26, 0.12];
    let wood_light = [0.65, 0.44, 0.22];
    let iron_reel = [0.75, 0.75, 0.80];
    let line = [0.88, 0.88, 0.92];
    let hook = [0.60, 0.60, 0.65];

    for i in 0..12 {
        let x = 2 + i;
        let y = 13 - i;
        texels[y * TEX_SIZE + x] = if i % 2 == 0 { wood_light } else { wood_dark };
    }
    texels[12 * TEX_SIZE + 4] = iron_reel;
    texels[13 * TEX_SIZE + 3] = iron_reel;
    texels[12 * TEX_SIZE + 3] = [0.35, 0.35, 0.38];
    for y in 3..=11 {
        texels[y * TEX_SIZE + 13] = line;
    }
    texels[12 * TEX_SIZE + 13] = hook;
    texels[12 * TEX_SIZE + 12] = hook;
    Texture { texels }
}
