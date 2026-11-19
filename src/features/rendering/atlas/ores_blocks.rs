//! Procedural 16x16 ore veins, precious mineral blocks, bricks, clay, and flint.

use super::types::{mix, rand_at, Texture, TEX_SIZE};
use super::terrain::{stone};

pub(super) fn coal_ore() -> Texture {
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

pub(super) fn iron_ore() -> Texture {
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

pub(super) fn gold_ore() -> Texture {
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

pub(super) fn diamond_ore() -> Texture {
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

pub(super) fn diamond_block() -> Texture {
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

pub(super) fn diamond() -> Texture {
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

pub(super) fn emerald_ore() -> Texture {
    let mut tex = stone();
    let emerald_base = [0.10, 0.82, 0.30];
    let emerald_hi = [0.65, 0.98, 0.72];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 773);
            if n > 0.80 {
                let col = if n > 0.92 { emerald_hi } else { emerald_base };
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], col, 0.92);
            }
        }
    }
    tex
}

pub(super) fn emerald_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.12, 0.80, 0.32];
    let dark = [0.06, 0.48, 0.16];
    let hi = [0.65, 0.98, 0.72];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_border = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let n = rand_at(x, y, 884);
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

pub(super) fn emerald() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.10, 0.78, 0.28];
    let hi = [0.72, 1.00, 0.72];
    let shadow = [0.05, 0.44, 0.14];
    let border = [0.02, 0.24, 0.08];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 - 7.5).abs();
            let dy = (y as f32 - 7.5).abs();
            let inside = dx <= 4.8 && dy <= 5.8 && (dx * 0.8 + dy <= 7.2);
            if inside {
                let is_edge = dx >= 4.0 || dy >= 5.0 || (dx * 0.8 + dy >= 6.4);
                let n = rand_at(x, y, 993);
                let col = if is_edge {
                    border
                } else if x <= 7 && y <= 6 {
                    hi
                } else if x >= 9 || y >= 9 {
                    shadow
                } else if n > 0.80 {
                    hi
                } else {
                    base
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn iron_block() -> Texture {
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

pub(super) fn gold_block() -> Texture {
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

pub(super) fn coal_block() -> Texture {
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

pub(super) fn clay_ball() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let cx = 7.5;
    let cy = 7.5;
    let radius = 4.8;
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= radius {
                let light_dir_dot = (-dx * 0.6 - dy * 0.8) / radius;
                let shade = (0.5 + 0.5 * light_dir_dot).clamp(0.0, 1.0);
                let col = mix([0.48, 0.50, 0.55], [0.72, 0.74, 0.79], shade);
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn brick_item() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 4..12 {
        for x in 2..14 {
            let is_edge = x == 2 || x == 13 || y == 4 || y == 11;
            let is_highlight = x == 3 || y == 5;
            let n = (rand_at(x, y, 83) - 0.5) * 0.08;
            let col = if is_edge {
                [0.45, 0.18, 0.12]
            } else if is_highlight {
                [0.78, 0.38, 0.28]
            } else {
                [(0.68 + n).clamp(0.0, 1.0), (0.28 + n).clamp(0.0, 1.0), (0.20 + n).clamp(0.0, 1.0)]
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

pub(super) fn bricks() -> Texture {
    let mortar = [0.78, 0.76, 0.73];
    let dark_mortar = [0.60, 0.58, 0.55];
    let mut texels = vec![mortar; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        let row = y / 4;
        let is_horiz_mortar = y % 4 == 0;
        let x_offset = if row % 2 == 1 { 4 } else { 0 };

        for x in 0..TEX_SIZE {
            let local_x = (x + x_offset) % 8;
            let is_vert_mortar = local_x == 0;

            if is_horiz_mortar || is_vert_mortar {
                let n = rand_at(x, y, 31) * 0.1;
                texels[y * TEX_SIZE + x] = mix(mortar, dark_mortar, n);
            } else {
                let r = rand_at(x, y, 77) - 0.5;
                let brick_base = match (x + y * 3) % 4 {
                    0 => [0.68, 0.28, 0.20],
                    1 => [0.62, 0.24, 0.18],
                    2 => [0.72, 0.32, 0.24],
                    _ => [0.58, 0.22, 0.16],
                };
                texels[y * TEX_SIZE + x] = [
                    (brick_base[0] + r * 0.08).clamp(0.0, 1.0),
                    (brick_base[1] + r * 0.06).clamp(0.0, 1.0),
                    (brick_base[2] + r * 0.05).clamp(0.0, 1.0),
                ];
            }
        }
    }
    Texture { texels }
}

pub(super) fn flint() -> Texture {
    let mut texels = vec![[-1.0, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    for y in 3..13 {
        for x in 3..13 {
            let is_in = (x >= 4 && x <= 11 && y >= 4 && y <= 12) && (x + y >= 9) && (x as i32 - y as i32 <= 4);
            if is_in {
                let n = rand_at(x, y, 66) * 0.08;
                let edge = x == 4 || y == 4 || (x + y == 9);
                let col = if edge {
                    [0.45, 0.44, 0.48]
                } else if x == 7 && y == 7 {
                    [0.85, 0.85, 0.90] // glint
                } else {
                    [0.22 + n, 0.20 + n, 0.24 + n]
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}
