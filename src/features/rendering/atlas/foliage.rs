//! Procedural 16x16 wood bark, leaves, cactus, sapling, kelp, and seagrass textures.

use super::types::{mix, rand_at, Texture, TEX_SIZE};

pub(super) fn wood_top() -> Texture {
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

pub(super) fn wood_side() -> Texture {
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

pub(super) fn leaves(fancy: bool) -> Texture {
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

pub(super) fn leaves_swamp(fancy: bool) -> Texture {
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

pub(super) fn leaves_savanna(fancy: bool) -> Texture {
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

pub(super) fn cactus_top() -> Texture {
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

pub(super) fn cactus_side() -> Texture {
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

pub(super) fn snow_leaves(fancy: bool) -> Texture {
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

pub(super) fn cherry_leaves(fancy: bool) -> Texture {
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

pub(super) fn autumn_leaves(fancy: bool) -> Texture {
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

pub(super) fn sapling() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
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

pub(super) fn kelp_texture() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let dark_green = [0.10, 0.32, 0.12];
    let mid_green = [0.18, 0.48, 0.20];
    let light_green = [0.28, 0.64, 0.28];
    let highlight = [0.38, 0.78, 0.36];

    // Central stalk running through the middle (x=7, x=8) continuous across block boundaries
    for y in 0..TEX_SIZE {
        texels[y * TEX_SIZE + 7] = if (y % 4) == 0 { highlight } else { mid_green };
        texels[y * TEX_SIZE + 8] = if (y % 4) == 0 { mid_green } else { dark_green };
    }

    // Alternating wavy kelp fronds with lush leaves and dark outlines (Minecraft style)
    for y in 0..TEX_SIZE {
        let side = (y / 3) % 2;
        let width = 3 + (y % 3);
        if side == 0 {
            for x in (8 - width)..7 {
                let is_edge = x == (8 - width);
                texels[y * TEX_SIZE + x] = if is_edge {
                    dark_green
                } else if (x + y) % 2 == 0 {
                    light_green
                } else {
                    mid_green
                };
            }
        } else {
            for x in 9..(9 + width).min(TEX_SIZE) {
                let is_edge = x == (8 + width);
                texels[y * TEX_SIZE + x] = if is_edge {
                    dark_green
                } else if (x + y) % 2 == 0 {
                    light_green
                } else {
                    mid_green
                };
            }
        }
    }
    Texture { texels }
}

pub(super) fn seagrass_texture() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let d_green = [0.12, 0.35, 0.14];
    let m_green = [0.22, 0.58, 0.24];
    let l_green = [0.36, 0.74, 0.36];
    let tip_color = [0.46, 0.85, 0.44];

    // Clustered underwater blades anchored at the seabed (bottom y=15)
    // Stalks: (base_x, height, wave_dir)
    let stalks: &[(usize, usize, i32)] = &[
        (4, 11, -1),
        (6, 14, 0),
        (8, 15, 1),
        (10, 13, 0),
        (12, 9, 1),
    ];
    for &(sx, h, wave_dir) in stalks {
        let top_y = TEX_SIZE - h;
        for y in top_y..TEX_SIZE {
            let sway = if y < 7 { wave_dir } else { 0 };
            let x = ((sx as i32) + sway).clamp(1, 14) as usize;
            let col = if y == top_y {
                tip_color
            } else if (x + y) % 2 == 0 {
                l_green
            } else {
                m_green
            };
            texels[y * TEX_SIZE + x] = col;
            if y > 8 && x > 0 {
                texels[y * TEX_SIZE + x - 1] = d_green;
            }
        }
    }
    // Solid anchor cluster at the base (y=14..15)
    for x in 4..=11 {
        if texels[15 * TEX_SIZE + x][0] < 0.0 {
            texels[15 * TEX_SIZE + x] = d_green;
        }
    }
    Texture { texels }
}
