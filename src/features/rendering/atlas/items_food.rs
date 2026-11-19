//! Procedural 16x16 raw/cooked meat, fish, fruit, and kelp item textures.

use super::types::{rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn raw_chicken() -> Texture {
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

pub(super) fn cooked_chicken() -> Texture {
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

pub(super) fn raw_porkchop() -> Texture {
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

pub(super) fn cooked_porkchop() -> Texture {
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

pub(super) fn raw_beef() -> Texture {
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

pub(super) fn cooked_beef() -> Texture {
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

pub(super) fn apple() -> Texture {
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

pub(super) fn raw_cod_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let scale_mid = [0.65, 0.60, 0.52];
    let scale_hi = [0.78, 0.74, 0.66];
    let scale_dark = [0.48, 0.44, 0.38];
    let belly_white = [0.90, 0.88, 0.82];
    let eye_black = [0.10, 0.10, 0.12];
    let eye_white = [0.98, 0.98, 0.98];

    for y in 6..=10 {
        for x in 2..=14 {
            let is_body = match y {
                6 => (4..=11).contains(&x),
                7 => (3..=12).contains(&x),
                8 => (2..=13).contains(&x),
                9 => (3..=13).contains(&x),
                10 => (4..=11).contains(&x),
                _ => false,
            };
            if is_body {
                let color = if y == 9 || y == 10 {
                    belly_white
                } else if (x + y) % 2 == 0 {
                    scale_hi
                } else if x == 2 || y == 6 {
                    scale_dark
                } else {
                    scale_mid
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[6 * TEX_SIZE + 14] = scale_dark;
    texels[7 * TEX_SIZE + 13] = scale_mid;
    texels[8 * TEX_SIZE + 12] = scale_dark;
    texels[9 * TEX_SIZE + 13] = scale_mid;
    texels[10 * TEX_SIZE + 14] = scale_dark;
    texels[7 * TEX_SIZE + 4] = eye_white;
    texels[7 * TEX_SIZE + 3] = eye_black;
    texels[5 * TEX_SIZE + 7] = scale_dark;
    texels[5 * TEX_SIZE + 8] = scale_mid;
    Texture { texels }
}

pub(super) fn cooked_cod_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let roasted_mid = [0.84, 0.68, 0.44];
    let roasted_hi = [0.94, 0.82, 0.58];
    let roasted_crust = [0.55, 0.38, 0.20];
    let white_meat = [0.96, 0.92, 0.86];

    for y in 6..=10 {
        for x in 3..=13 {
            let is_body = match y {
                6 => (5..=11).contains(&x),
                7 => (4..=12).contains(&x),
                8 => (3..=13).contains(&x),
                9 => (4..=12).contains(&x),
                10 => (5..=11).contains(&x),
                _ => false,
            };
            if is_body {
                let color = if x == 6 || x == 9 || x == 12 {
                    roasted_crust
                } else if (x + y) % 3 == 0 {
                    white_meat
                } else if (x + y) % 2 == 0 {
                    roasted_hi
                } else {
                    roasted_mid
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[7 * TEX_SIZE + 14] = roasted_crust;
    texels[9 * TEX_SIZE + 14] = roasted_crust;
    Texture { texels }
}

pub(super) fn raw_salmon_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let salmon_pink = [0.92, 0.42, 0.36];
    let salmon_light = [0.98, 0.58, 0.52];
    let head_teal = [0.28, 0.52, 0.48];
    let eye = [0.10, 0.10, 0.12];
    let fin_dark = [0.65, 0.28, 0.24];

    for y in 5..=11 {
        for x in 2..=14 {
            let is_body = match y {
                5 => (5..=11).contains(&x),
                6 => (4..=12).contains(&x),
                7 => (3..=13).contains(&x),
                8 => (2..=13).contains(&x),
                9 => (3..=13).contains(&x),
                10 => (4..=12).contains(&x),
                11 => (5..=10).contains(&x),
                _ => false,
            };
            if is_body {
                let color = if x <= 4 {
                    head_teal
                } else if (x + y) % 3 == 0 {
                    salmon_light
                } else {
                    salmon_pink
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[7 * TEX_SIZE + 3] = eye;
    texels[6 * TEX_SIZE + 14] = fin_dark;
    texels[8 * TEX_SIZE + 13] = fin_dark;
    texels[10 * TEX_SIZE + 14] = fin_dark;
    texels[4 * TEX_SIZE + 8] = fin_dark;
    Texture { texels }
}

pub(super) fn cooked_salmon_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let roast_salmon = [0.88, 0.52, 0.32];
    let roast_hi = [0.95, 0.65, 0.42];
    let crust = [0.52, 0.28, 0.15];
    let flake = [0.96, 0.80, 0.65];

    for y in 5..=11 {
        for x in 3..=13 {
            let is_body = match y {
                5 => (5..=11).contains(&x),
                6 => (4..=12).contains(&x),
                7 => (3..=13).contains(&x),
                8 => (3..=13).contains(&x),
                9 => (4..=12).contains(&x),
                10 => (5..=11).contains(&x),
                _ => false,
            };
            if is_body {
                let color = if x == 6 || x == 9 {
                    crust
                } else if (x + y) % 3 == 0 {
                    flake
                } else if (x + y) % 2 == 0 {
                    roast_hi
                } else {
                    roast_salmon
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[6 * TEX_SIZE + 14] = crust;
    texels[8 * TEX_SIZE + 13] = crust;
    texels[10 * TEX_SIZE + 14] = crust;
    Texture { texels }
}

pub(super) fn pufferfish_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let body_yellow = [0.92, 0.82, 0.24];
    let spine_tan = [0.76, 0.64, 0.20];
    let belly_white = [0.96, 0.94, 0.82];
    let blue_fin = [0.28, 0.60, 0.78];
    let eye = [0.10, 0.10, 0.15];

    for y in 4..=12 {
        for x in 4..=12 {
            let dx = (x as f32 - 8.0).abs();
            let dy = (y as f32 - 8.0).abs();
            if dx * dx + dy * dy <= 17.0 {
                let color = if y >= 10 {
                    belly_white
                } else if x == 4 || x == 12 || y == 4 || y == 12 {
                    spine_tan
                } else {
                    body_yellow
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[6 * TEX_SIZE + 6] = eye;
    texels[6 * TEX_SIZE + 10] = eye;
    texels[3 * TEX_SIZE + 8] = spine_tan;
    texels[8 * TEX_SIZE + 3] = spine_tan;
    texels[8 * TEX_SIZE + 13] = spine_tan;
    texels[13 * TEX_SIZE + 8] = spine_tan;
    texels[8 * TEX_SIZE + 2] = blue_fin;
    texels[8 * TEX_SIZE + 14] = blue_fin;
    texels[4 * TEX_SIZE + 11] = blue_fin;
    Texture { texels }
}

pub(super) fn tropical_fish_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let orange = [0.98, 0.48, 0.12];
    let white = [0.96, 0.96, 0.96];
    let black = [0.10, 0.10, 0.12];

    for y in 5..=11 {
        for x in 3..=13 {
            let is_body = match y {
                5 => (6..=10).contains(&x),
                6 => (5..=11).contains(&x),
                7 => (4..=12).contains(&x),
                8 => (3..=13).contains(&x),
                9 => (4..=12).contains(&x),
                10 => (5..=11).contains(&x),
                11 => (6..=10).contains(&x),
                _ => false,
            };
            if is_body {
                let color = if x == 7 || x == 8 || x == 11 {
                    white
                } else if x == 6 || x == 9 || x == 10 {
                    black
                } else {
                    orange
                };
                texels[y * TEX_SIZE + x] = color;
            }
        }
    }
    texels[7 * TEX_SIZE + 4] = black;
    texels[6 * TEX_SIZE + 14] = white;
    texels[8 * TEX_SIZE + 14] = orange;
    texels[10 * TEX_SIZE + 14] = white;
    Texture { texels }
}

pub(super) fn dried_kelp_texture() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let dark_leaf = [0.12, 0.24, 0.10];
    let mid_leaf = [0.18, 0.32, 0.14];
    let salt = [0.78, 0.84, 0.72];

    for y in 4..=12 {
        for x in 4..=12 {
            let is_salt = (x * 7 + y * 13) % 11 == 0;
            texels[y * TEX_SIZE + x] = if is_salt {
                salt
            } else if (x + y) % 2 == 0 {
                mid_leaf
            } else {
                dark_leaf
            };
        }
    }
    Texture { texels }
}
