//! Procedural 16x16 creature and aquatic mob spawn egg item textures.

use super::types::{rand_at, Texture, TEX_SIZE};

pub(super) fn spawn_egg_texture(base: [f32; 3], spot: [f32; 3], outline: [f32; 3], seed: u32) -> Texture {
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

pub(super) fn pig_spawn_egg() -> Texture {
    spawn_egg_texture([0.95, 0.62, 0.68], [0.68, 0.32, 0.38], [0.48, 0.20, 0.25], 701)
}

pub(super) fn cow_spawn_egg() -> Texture {
    spawn_egg_texture([0.42, 0.30, 0.22], [0.85, 0.85, 0.85], [0.22, 0.15, 0.10], 702)
}

pub(super) fn chicken_spawn_egg() -> Texture {
    spawn_egg_texture([0.95, 0.95, 0.92], [0.85, 0.15, 0.15], [0.35, 0.35, 0.30], 703)
}

pub(super) fn sheep_spawn_egg() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.88, 0.88, 0.86];
    let spot = [0.85, 0.65, 0.68];
    let outline = [0.35, 0.35, 0.35];

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
                let is_spot = (x == 6 && y == 5) || (x == 9 && y == 8) || (x == 5 && y == 10) || (x == 8 && y == 12);
                texels[y * TEX_SIZE + x] = if is_border {
                    outline
                } else if is_spot {
                    spot
                } else {
                    base
                };
            }
        }
    }
    Texture { texels }
}

pub(super) fn villager_spawn_egg() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.55, 0.38, 0.24];
    let spot = [0.78, 0.58, 0.45];
    let border = [0.32, 0.20, 0.12];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32 - 8.0;
            let egg_width = if dy < 0.0 {
                (4.8 + dy * 0.45).max(1.0)
            } else {
                (4.8 - dy * 0.65).max(0.5)
            };
            if dx <= egg_width {
                let is_border = dx >= egg_width - 0.9;
                let n = rand_at(x, y, 612);
                let col = if is_border {
                    border
                } else if n > 0.72 {
                    spot
                } else {
                    base
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn iron_golem_spawn_egg() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.82, 0.82, 0.80];
    let spot = [0.45, 0.45, 0.43];
    let border = [0.35, 0.35, 0.35];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32 - 8.0;
            let egg_width = if dy < 0.0 {
                (4.8 + dy * 0.45).max(1.0)
            } else {
                (4.8 - dy * 0.65).max(0.5)
            };
            if dx <= egg_width {
                let is_border = dx >= egg_width - 0.9;
                let n = rand_at(x, y, 621);
                let col = if is_border {
                    border
                } else if n > 0.72 {
                    spot
                } else {
                    base
                };
                texels[y * TEX_SIZE + x] = col;
            }
        }
    }
    Texture { texels }
}

pub(super) fn spider_spawn_egg() -> Texture {
    spawn_egg_texture([0.20, 0.15, 0.15], [0.75, 0.12, 0.12], [0.10, 0.08, 0.08], 708)
}

pub(super) fn dolphin_spawn_egg_texture() -> Texture {
    let base = [0.55, 0.68, 0.78];
    let spot = [0.82, 0.90, 0.95];
    let outline = [0.38, 0.48, 0.58];
    render_aquatic_egg_tex(base, spot, outline)
}

pub(super) fn fish_spawn_egg_texture() -> Texture {
    let base = [0.76, 0.56, 0.38];
    let spot = [0.94, 0.46, 0.20];
    let outline = [0.52, 0.36, 0.24];
    render_aquatic_egg_tex(base, spot, outline)
}

pub(super) fn render_aquatic_egg_tex(base: [f32; 3], spot: [f32; 3], outline: [f32; 3]) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 2..=14 {
        for x in 3..=12 {
            let dx = (x as f32 - 7.5).abs();
            let dy = y as f32;
            let width_at_y = if dy <= 6.0 {
                (dy - 1.0) * 0.7 + 0.5
            } else if dy <= 11.0 {
                4.2
            } else {
                (14.5 - dy) * 1.1 + 0.5
            };
            if dx <= width_at_y {
                let is_border = dx > width_at_y - 0.9 || y == 2 || y == 14;
                let is_spot = (x == 6 && y == 5) || (x == 9 && y == 8) || (x == 5 && y == 10) || (x == 8 && y == 12);
                texels[y * TEX_SIZE + x] = if is_border {
                    outline
                } else if is_spot {
                    spot
                } else {
                    base
                };
            }
        }
    }
    Texture { texels }
}
