//! Procedural 16x16 RustStone ores, dust/wires, lamps, switches, plates, and dyed wool.

use super::types::{mix, rand_at, Texture, TEX_SIZE};
use super::terrain::{stone};

pub(super) fn ruststone_ore() -> Texture {
    let mut tex = stone();
    let rust_dark = [0.48, 0.14, 0.08];
    let rust_base = [0.78, 0.24, 0.12];
    let rust_glow = [0.98, 0.42, 0.16];
    let ember_core = [1.00, 0.76, 0.35];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, 941);
            if n > 0.74 {
                let col = if n > 0.94 {
                    ember_core
                } else if n > 0.86 {
                    rust_glow
                } else if n > 0.80 {
                    rust_base
                } else {
                    rust_dark
                };
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], col, 0.92);
            }
        }
    }
    tex
}

pub(super) fn ruststone_dust() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let dark = [0.45, 0.12, 0.06];
    let base = [0.82, 0.26, 0.12];
    let hi = [1.00, 0.48, 0.18];
    let ember = [1.00, 0.80, 0.40];
    for y in 4..12 {
        for x in 4..12 {
            let dist_sq = (x as f32 - 7.5).powi(2) + (y as f32 - 7.5).powi(2);
            if dist_sq <= 16.0 {
                let n = rand_at(x, y, 317);
                let col = if n > 0.85 {
                    ember
                } else if n > 0.60 {
                    hi
                } else if n > 0.30 {
                    base
                } else {
                    dark
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn ruststone_wire(lit: bool) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let (c_center, c_line, c_edge) = if lit {
        ([1.00, 0.82, 0.40], [1.00, 0.40, 0.14], [0.65, 0.18, 0.08])
    } else {
        ([0.62, 0.18, 0.10], [0.45, 0.12, 0.06], [0.28, 0.08, 0.04])
    };
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let in_x_strip = y >= 6 && y <= 9;
            let in_y_strip = x >= 6 && x <= 9;
            let in_center = in_x_strip && in_y_strip;
            if in_center {
                let is_core = (x == 7 || x == 8) && (y == 7 || y == 8);
                texels[y * TEX_SIZE + x] = if is_core { c_center } else { c_line };
            } else if in_x_strip || in_y_strip {
                let is_edge = x == 6 || x == 9 || y == 6 || y == 9;
                texels[y * TEX_SIZE + x] = if is_edge { c_edge } else { c_line };
            }
        }
    }
    Texture { texels }
}

pub(super) fn ruststone_block() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let border = [0.38, 0.10, 0.05];
    let base = [0.75, 0.22, 0.10];
    let hi = [0.95, 0.40, 0.18];
    let glow = [1.00, 0.65, 0.28];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_edge = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let n = rand_at(x, y, 621);
            let col = if is_edge {
                border
            } else if n > 0.88 {
                glow
            } else if n > 0.65 {
                hi
            } else if n < 0.25 {
                border
            } else {
                base
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

pub(super) fn ruststone_lamp(lit: bool) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let frame = [0.32, 0.16, 0.10];
    let frame_hi = [0.52, 0.26, 0.16];
    let glass_unlit = [0.28, 0.14, 0.08];
    let glass_lit = [1.00, 0.84, 0.42];
    let glass_lit_glow = [0.98, 0.58, 0.20];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_border = x == 0 || x == TEX_SIZE - 1 || y == 0 || y == TEX_SIZE - 1;
            let is_grid = x == 5 || x == 10 || y == 5 || y == 10;
            if is_border || is_grid {
                let is_corner_rivet = (x == 1 && (y == 1 || y == 14)) || (x == 14 && (y == 1 || y == 14));
                texels[y * TEX_SIZE + x] = if is_corner_rivet { frame_hi } else { frame };
            } else if lit {
                let is_bright = (x >= 6 && x <= 9) && (y >= 6 && y <= 9);
                texels[y * TEX_SIZE + x] = if is_bright { glass_lit } else { glass_lit_glow };
            } else {
                let n = rand_at(x, y, 404);
                let col = if n > 0.70 { frame } else { glass_unlit };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn lever_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let stone_base = [0.55, 0.55, 0.58];
    let stone_sh = [0.38, 0.38, 0.40];
    let wood_shaft = [0.55, 0.35, 0.15];
    let shaft_hi = [0.72, 0.50, 0.25];
    let tip = [0.42, 0.42, 0.44];

    // Cobblestone mounting plate
    for y in 10..15 {
        for x in 5..11 {
            let edge = x == 5 || x == 10 || y == 14;
            texels[y * TEX_SIZE + x] = if edge { stone_sh } else { stone_base };
        }
    }
    // Lever handle angled
    for i in 0..7 {
        let x = 6 + i / 2;
        let y = 10 - i;
        if y < TEX_SIZE && x < TEX_SIZE {
            texels[y * TEX_SIZE + x] = if i == 6 { tip } else if (x + y) % 2 == 0 { shaft_hi } else { wood_shaft };
        }
    }
    Texture { texels }
}

pub(super) fn button_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let hi = [0.75, 0.75, 0.78];
    let base = [0.58, 0.58, 0.60];
    let sh = [0.38, 0.38, 0.40];
    for y in 5..11 {
        for x in 4..12 {
            let top_left = x == 4 || y == 5;
            let bot_right = x == 11 || y == 10;
            texels[y * TEX_SIZE + x] = if top_left {
                hi
            } else if bot_right {
                sh
            } else {
                base
            };
        }
    }
    Texture { texels }
}

pub(super) fn pressure_plate_texture() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let border = [0.40, 0.40, 0.42];
    let base = [0.60, 0.60, 0.62];
    let hi = [0.72, 0.72, 0.75];
    for y in 1..15 {
        for x in 1..15 {
            let is_edge = x == 1 || x == 14 || y == 1 || y == 14;
            let is_inner_hi = x == 2 || y == 2;
            texels[y * TEX_SIZE + x] = if is_edge {
                border
            } else if is_inner_hi {
                hi
            } else {
                base
            };
        }
    }
    Texture { texels }
}

pub(super) fn wool_texture(base: [f32; 3], seed: u32) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let knit = if (x + y) % 2 == 0 { 0.04 } else { -0.04 };
            let sub_pattern = if (x % 4 == 0) || (y % 4 == 0) { -0.03 } else { 0.02 };
            let noise = (rand_at(x, y, seed) - 0.5) * 0.12;
            let shade = (1.0 + knit + sub_pattern + noise).clamp(0.0, 1.8);
            texels[y * TEX_SIZE + x] = [
                (base[0] * shade).clamp(0.0, 1.0),
                (base[1] * shade).clamp(0.0, 1.0),
                (base[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}
