//! Procedural 16x16 chest, furnace, bed, ladder, torch, shadow, and crack overlays.

use super::types::{rand_at, Texture, TEX_SIZE};

pub(super) fn bed_top() -> Texture {
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

pub(super) fn bed_side() -> Texture {
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

pub(super) fn ladder() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
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
                [-0.001, 0.0, 0.0]
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

pub(super) fn chest_top() -> Texture {
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

pub(super) fn chest_side() -> Texture {
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

pub(super) fn chest_front() -> Texture {
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

pub(super) fn furnace_top() -> Texture {
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

pub(super) fn furnace_side() -> Texture {
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

pub(super) fn furnace_front() -> Texture {
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

pub(super) fn furnace_front_lit() -> Texture {
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

pub(super) fn torch() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let wood_dark = [0.42, 0.28, 0.14];
    let wood_light = [0.58, 0.40, 0.22];
    let wood_shadow = [0.32, 0.20, 0.10];
    let coal = [0.18, 0.16, 0.16];
    let coal_ember = [0.96, 0.48, 0.08];
    let flame_outer = [0.98, 0.42, 0.05];
    let flame_mid = [1.0, 0.78, 0.12];
    let flame_core = [1.0, 1.0, 0.85];

    // 1. Top flame (y: 0..5, centered at x: 7.5, y: 2.0)
    for y in 0..5 {
        for x in 5..11 {
            let dx = (x as f32 - 7.5).abs();
            let dy = (y as f32 - 2.0).abs();
            let dist = dx * dx + dy * dy * 1.2;
            if dist <= 4.0 {
                let col = if dist < 1.0 {
                    flame_core
                } else if dist < 2.5 {
                    flame_mid
                } else {
                    flame_outer
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }

    // 2. Coal / wick base under flame (y: 4..6, x: 6..10)
    for y in 4..6 {
        for x in 6..10 {
            let is_ember = (x == 7 || x == 8) && y == 4;
            texels[y * TEX_SIZE + x] = if is_ember { coal_ember } else { coal };
        }
    }

    // 3. Wooden stick (y: 6..16, x: 6..10, y=15 is bottom)
    for y in 6..TEX_SIZE {
        for x in 6..10 {
            let n = rand_at(x, y, 77);
            let col = if x == 9 {
                wood_shadow
            } else if n > 0.55 {
                wood_light
            } else {
                wood_dark
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }

    Texture { texels }
}

pub(super) fn entity_shadow() -> Texture {
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

pub(super) fn destroy_stage(stage: usize) -> Texture {
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
