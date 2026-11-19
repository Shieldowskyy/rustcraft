//! Procedural 16x16 farmland, wheat/carrot/potato crops, food, and bonemeal textures.

use super::types::{speckled, Texture, TEX_SIZE};

pub(super) fn farmland() -> Texture {
    let mut tex = speckled([0.34, 0.22, 0.12], 0.12, 101);
    let furrow = [0.22, 0.14, 0.08];
    for y in 0..TEX_SIZE {
        if y % 3 == 0 {
            for x in 0..TEX_SIZE {
                tex.texels[y * TEX_SIZE + x] = furrow;
            }
        }
    }
    tex
}

pub(super) fn farmland_wet() -> Texture {
    let mut tex = speckled([0.20, 0.13, 0.07], 0.10, 102);
    let furrow = [0.12, 0.08, 0.04];
    let glint = [0.35, 0.28, 0.22];
    for y in 0..TEX_SIZE {
        if y % 3 == 0 {
            for x in 0..TEX_SIZE {
                tex.texels[y * TEX_SIZE + x] = furrow;
            }
        } else {
            for x in 0..TEX_SIZE {
                if (x + y * 7) % 11 == 0 {
                    tex.texels[y * TEX_SIZE + x] = glint;
                }
            }
        }
    }
    tex
}

pub(super) fn farmland_side() -> Texture {
    let mut tex = speckled([0.45, 0.33, 0.20], 0.15, 7);
    let top_dirt = [0.34, 0.22, 0.12];
    for y in 0..2 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = top_dirt;
        }
    }
    tex
}

pub(super) fn wheat_seeds() -> Texture {
    let mut tex = speckled([0.22, 0.16, 0.10], 0.10, 201);
    let seed_body = [0.75, 0.70, 0.35];
    let seed_tip = [0.45, 0.65, 0.25];
    let coords = [(4, 5), (8, 4), (11, 7), (6, 9), (9, 11), (5, 12)];
    for (cx, cy) in coords {
        tex.texels[cy * TEX_SIZE + cx] = seed_body;
        tex.texels[(cy + 1) * TEX_SIZE + cx] = seed_body;
        tex.texels[cy * TEX_SIZE + (cx + 1)] = seed_tip;
    }
    tex
}

pub(super) fn wheat_crop() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let stem = [0.35, 0.68, 0.20];
    let stem_light = [0.45, 0.76, 0.25];
    let ear = [0.88, 0.78, 0.32];
    let ear_dark = [0.72, 0.60, 0.20];
    for y in 2..TEX_SIZE {
        texels[y * TEX_SIZE + 7] = stem;
        texels[y * TEX_SIZE + 8] = stem_light;
        if y >= 6 && y % 3 == 0 {
            texels[y * TEX_SIZE + 4] = stem;
            texels[y * TEX_SIZE + 5] = stem;
            texels[y * TEX_SIZE + 6] = stem;
            texels[y * TEX_SIZE + 9] = stem_light;
            texels[y * TEX_SIZE + 10] = stem_light;
            texels[y * TEX_SIZE + 11] = stem_light;
        }
    }
    for y in 2..8 {
        for x in 5..11 {
            texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { ear } else { ear_dark };
        }
    }
    Texture { texels }
}

pub(super) fn wheat() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let gold = [0.88, 0.76, 0.28];
    let gold_dark = [0.70, 0.55, 0.18];
    let ribbon = [0.75, 0.20, 0.15];
    for y in 3..14 {
        for x in 5..11 {
            let col = if y == 8 || y == 9 {
                ribbon
            } else if (x + y) % 2 == 0 {
                gold
            } else {
                gold_dark
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

pub(super) fn bread() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let crust = [0.55, 0.32, 0.12];
    let crumb = [0.82, 0.62, 0.32];
    let score = [0.42, 0.24, 0.08];
    for y in 5..12 {
        for x in 3..13 {
            let is_edge = y == 5 || y == 11 || x == 3 || x == 12;
            let is_score = (x == 6 || x == 9) && (y >= 6 && y <= 10);
            texels[y * TEX_SIZE + x] = if is_score {
                score
            } else if is_edge {
                crust
            } else {
                crumb
            };
        }
    }
    Texture { texels }
}

pub(super) fn carrot() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let orange = [0.96, 0.48, 0.08];
    let orange_dark = [0.80, 0.35, 0.05];
    let green = [0.25, 0.68, 0.15];
    texels[2 * TEX_SIZE + 7] = green;
    texels[2 * TEX_SIZE + 8] = green;
    texels[3 * TEX_SIZE + 6] = green;
    texels[3 * TEX_SIZE + 9] = green;
    for y in 4..14 {
        let width = ((14 - y) / 2).max(1);
        for x in (8 - width)..=(7 + width) {
            texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { orange } else { orange_dark };
        }
    }
    Texture { texels }
}

pub(super) fn carrot_crop() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let green = [0.28, 0.70, 0.18];
    let green_dark = [0.18, 0.48, 0.12];
    let orange = [0.96, 0.48, 0.08];
    for y in 3..13 {
        for x in 3..13 {
            if (x + y * 2) % 3 != 0 {
                texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { green } else { green_dark };
            }
        }
    }
    for x in 6..10 {
        texels[13 * TEX_SIZE + x] = orange;
        texels[14 * TEX_SIZE + x] = orange;
    }
    Texture { texels }
}

pub(super) fn potato() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.68, 0.52, 0.32];
    let eye = [0.42, 0.30, 0.16];
    let hi = [0.78, 0.62, 0.38];
    for y in 4..12 {
        for x in 4..12 {
            texels[y * TEX_SIZE + x] = if (x == 6 && y == 6) || (x == 9 && y == 9) || (x == 7 && y == 10) {
                eye
            } else if x <= 6 && y <= 6 {
                hi
            } else {
                base
            };
        }
    }
    Texture { texels }
}

pub(super) fn potato_crop() -> Texture {
    let mut texels = vec![[-0.001, 0.0, 0.0]; TEX_SIZE * TEX_SIZE];
    let green = [0.30, 0.62, 0.22];
    let flower = [0.95, 0.95, 0.88];
    let flower_core = [0.98, 0.85, 0.20];
    for y in 4..14 {
        for x in 3..13 {
            if (x * 3 + y * 5) % 4 != 0 {
                texels[y * TEX_SIZE + x] = green;
            }
        }
    }
    texels[3 * TEX_SIZE + 6] = flower;
    texels[4 * TEX_SIZE + 6] = flower_core;
    texels[3 * TEX_SIZE + 9] = flower;
    texels[4 * TEX_SIZE + 9] = flower_core;
    Texture { texels }
}

pub(super) fn baked_potato() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let skin = [0.55, 0.38, 0.20];
    let inside = [0.92, 0.85, 0.45];
    let butter = [0.98, 0.92, 0.30];
    for y in 4..12 {
        for x in 4..12 {
            let is_cut = (x >= 6 && x <= 9) && (y >= 6 && y <= 9);
            let is_butter = (x == 7 || x == 8) && (y == 7 || y == 8);
            texels[y * TEX_SIZE + x] = if is_butter {
                butter
            } else if is_cut {
                inside
            } else {
                skin
            };
        }
    }
    Texture { texels }
}

pub(super) fn bone_meal() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let white = [0.95, 0.95, 0.92];
    let shadow = [0.78, 0.78, 0.75];
    for y in 5..12 {
        for x in 4..12 {
            if (x + y) % 3 != 0 {
                texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { white } else { shadow };
            }
        }
    }
    Texture { texels }
}
