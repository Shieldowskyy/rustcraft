//! Proceduralna synteza tekstur 16×16 dla bloków i przedmiotów modów (`parse_texture_pattern`, `synthesize_texture`).

use std::collections::HashMap;

use crate::features::rendering::atlas::{Texture, TEX_SIZE};

use super::json::JsonNode;
use super::types::{rand_hash, TexturePattern};

pub(super) fn parse_texture_pattern(obj: &HashMap<String, JsonNode>) -> TexturePattern {
    let pattern_type = obj
        .get("pattern")
        .and_then(|v| v.as_str())
        .unwrap_or("mineral");
    match pattern_type {
        "ore" => {
            let bg = obj
                .get("background_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.55, 0.55, 0.55]);
            let ore = obj
                .get("ore_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.1, 0.9, 0.4]);
            TexturePattern::Ore {
                background_color: bg,
                ore_color: ore,
            }
        }
        "speckled" => {
            let base = obj
                .get("base_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.3, 0.8, 0.3]);
            let var = obj.get("variance").and_then(|v| v.as_f32()).unwrap_or(0.15);
            TexturePattern::Speckled {
                base_color: base,
                variance: var,
            }
        }
        "gem" | "crystal" => {
            let col = obj
                .get("color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.1, 0.95, 0.45]);
            TexturePattern::Gem { color: col }
        }
        "apple" | "food" => {
            let col = obj
                .get("color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.9, 0.1, 0.2]);
            TexturePattern::Apple { color: col }
        }
        "ingot" => {
            let col = obj
                .get("color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.8, 0.85, 0.9]);
            TexturePattern::Ingot { color: col }
        }
        "sword" => {
            let blade = obj
                .get("blade_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.1, 0.9, 0.4]);
            let handle = obj
                .get("handle_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.45, 0.28, 0.14]);
            TexturePattern::ToolSword {
                blade_color: blade,
                handle_color: handle,
            }
        }
        "pickaxe" => {
            let head = obj
                .get("head_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.1, 0.9, 0.4]);
            let handle = obj
                .get("handle_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.45, 0.28, 0.14]);
            TexturePattern::ToolPickaxe {
                head_color: head,
                handle_color: handle,
            }
        }
        _ => {
            let base = obj
                .get("base_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([0.2, 0.7, 0.4]);
            let border = obj
                .get("border_color")
                .and_then(|v| v.as_color3())
                .unwrap_or([base[0] * 0.6, base[1] * 0.6, base[2] * 0.6]);
            TexturePattern::MineralBlock {
                base_color: base,
                border_color: border,
            }
        }
    }
}

pub(super) fn synthesize_texture(pattern: &TexturePattern) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    match pattern {
        TexturePattern::Speckled {
            base_color,
            variance,
        } => {
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                    let shade = 1.0 + n * variance;
                    texels[y * TEX_SIZE + x] = [
                        (base_color[0] * shade).clamp(0.0, 1.0),
                        (base_color[1] * shade).clamp(0.0, 1.0),
                        (base_color[2] * shade).clamp(0.0, 1.0),
                    ];
                }
            }
        }
        TexturePattern::Ore {
            background_color,
            ore_color,
        } => {
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let stone_n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                    let stone_shade = 1.0 + stone_n * 0.15;
                    texels[y * TEX_SIZE + x] = [
                        (background_color[0] * stone_shade).clamp(0.0, 1.0),
                        (background_color[1] * stone_shade).clamp(0.0, 1.0),
                        (background_color[2] * stone_shade).clamp(0.0, 1.0),
                    ];
                }
            }
            let ore_pixels = [
                (3, 4), (4, 4), (4, 5), (3, 5),
                (9, 2), (10, 2), (10, 3), (11, 3),
                (6, 8), (7, 8), (7, 9), (8, 9), (6, 9),
                (11, 10), (12, 10), (12, 11),
                (2, 12), (3, 12), (3, 13),
            ];
            for (ox, oy) in ore_pixels {
                if ox < TEX_SIZE && oy < TEX_SIZE {
                    let sparkle = if (ox + oy) % 2 == 0 { 1.15 } else { 0.95 };
                    texels[oy * TEX_SIZE + ox] = [
                        (ore_color[0] * sparkle).clamp(0.0, 1.0),
                        (ore_color[1] * sparkle).clamp(0.0, 1.0),
                        (ore_color[2] * sparkle).clamp(0.0, 1.0),
                    ];
                }
            }
        }
        TexturePattern::MineralBlock {
            base_color,
            border_color,
        } => {
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let is_border = x == 0
                        || x == TEX_SIZE - 1
                        || y == 0
                        || y == TEX_SIZE - 1
                        || x == 1
                        || x == TEX_SIZE - 2
                        || y == 1
                        || y == TEX_SIZE - 2;
                    let n = (rand_hash(x as u32, y as u32) % 1000) as f32 / 1000.0 - 0.5;
                    let col = if is_border {
                        *border_color
                    } else {
                        let shade = 1.0 + n * 0.12;
                        [
                            (base_color[0] * shade).clamp(0.0, 1.0),
                            (base_color[1] * shade).clamp(0.0, 1.0),
                            (base_color[2] * shade).clamp(0.0, 1.0),
                        ]
                    };
                    texels[y * TEX_SIZE + x] = col;
                }
            }
        }
        TexturePattern::Gem { color } => {
            let dark = [color[0] * 0.5, color[1] * 0.5, color[2] * 0.5];
            let light = [
                (color[0] * 1.3).min(1.0),
                (color[1] * 1.3).min(1.0),
                (color[2] * 1.3).min(1.0),
            ];
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let dx = (x as i32 - 8).abs();
                    let dy = (y as i32 - 8).abs();
                    if dx + dy <= 6 {
                        let col = if x < 8 && y < 8 {
                            light
                        } else if x >= 8 && y >= 8 {
                            dark
                        } else {
                            *color
                        };
                        texels[y * TEX_SIZE + x] = col;
                    }
                }
            }
        }
        TexturePattern::Apple { color } => {
            let dark = [color[0] * 0.6, color[1] * 0.6, color[2] * 0.6];
            let stem = [0.45, 0.28, 0.14];
            let leaf = [0.2, 0.8, 0.2];
            for y in 0..TEX_SIZE {
                for x in 0..TEX_SIZE {
                    let dx = (x as i32 - 8).abs();
                    let dy = (y as i32 - 9).abs();
                    if dx * dx + dy * dy <= 20 {
                        texels[y * TEX_SIZE + x] = if x > 8 { dark } else { *color };
                    } else if x == 8 && (y == 3 || y == 4) {
                        texels[y * TEX_SIZE + x] = stem;
                    } else if x == 9 && y == 3 {
                        texels[y * TEX_SIZE + x] = leaf;
                    }
                }
            }
        }
        TexturePattern::Ingot { color } => {
            let dark = [color[0] * 0.65, color[1] * 0.65, color[2] * 0.65];
            let light = [
                (color[0] * 1.25).min(1.0),
                (color[1] * 1.25).min(1.0),
                (color[2] * 1.25).min(1.0),
            ];
            for y in 5..12 {
                for x in 3..14 {
                    let col = if y == 5 || x == 3 {
                        light
                    } else if y == 11 || x == 13 {
                        dark
                    } else {
                        *color
                    };
                    texels[y * TEX_SIZE + x] = col;
                }
            }
        }
        TexturePattern::ToolSword {
            blade_color,
            handle_color,
        } => {
            for i in 2..10 {
                texels[i * TEX_SIZE + (15 - i)] = *blade_color;
                texels[i * TEX_SIZE + (14 - i)] = [
                    blade_color[0] * 0.8,
                    blade_color[1] * 0.8,
                    blade_color[2] * 0.8,
                ];
            }
            texels[10 * TEX_SIZE + 4] = [0.2, 0.2, 0.2];
            texels[10 * TEX_SIZE + 5] = [0.2, 0.2, 0.2];
            texels[10 * TEX_SIZE + 6] = [0.2, 0.2, 0.2];
            texels[11 * TEX_SIZE + 4] = *handle_color;
            texels[12 * TEX_SIZE + 3] = *handle_color;
            texels[13 * TEX_SIZE + 2] = [
                handle_color[0] * 0.6,
                handle_color[1] * 0.6,
                handle_color[2] * 0.6,
            ];
        }
        TexturePattern::ToolPickaxe {
            head_color,
            handle_color,
        } => {
            for i in 2..10 {
                texels[(14 - i) * TEX_SIZE + (14 - i)] = *handle_color;
            }
            let head_coords = [
                (11, 2), (12, 2), (13, 3), (13, 4),
                (4, 11), (3, 12), (2, 12), (2, 13),
            ];
            for (hx, hy) in head_coords {
                texels[hy * TEX_SIZE + hx] = *head_color;
            }
            texels[3 * TEX_SIZE + 12] = [
                head_color[0] * 1.2,
                head_color[1] * 1.2,
                head_color[2] * 1.2,
            ];
        }
        TexturePattern::Pixels(pxs) => {
            for (i, p) in pxs.iter().enumerate().take(TEX_SIZE * TEX_SIZE) {
                texels[i] = *p;
            }
        }
    }

    Texture { texels }
}

pub(super) fn compute_texture_average_color(tex: &Texture) -> [f32; 3] {
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut count = 0.0;
    for c in &tex.texels {
        if c[0] > 0.001 || c[1] > 0.001 || c[2] > 0.001 {
            r += c[0];
            g += c[1];
            b += c[2];
            count += 1.0;
        }
    }
    if count == 0.0 {
        return [0.5, 0.5, 0.5];
    }
    [r / count, g / count, b / count]
}
