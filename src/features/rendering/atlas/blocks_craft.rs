//! Procedural 16x16 planks, doors, glass, TNT, crafting table, and bookshelf textures.

use super::types::{mix, rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn planks() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let plank_base_tones = [
        [0.69, 0.53, 0.31],
        [0.64, 0.48, 0.28],
        [0.72, 0.56, 0.34],
        [0.66, 0.50, 0.29],
    ];

    let groove_color = [0.36, 0.24, 0.12];
    let bevel_highlight = [0.77, 0.62, 0.38];
    let nail_color = [0.24, 0.16, 0.08];

    for y in 0..TEX_SIZE {
        let plank_idx = y / 4;
        let in_plank_y = y % 4;
        let base_col = plank_base_tones[plank_idx];

        let joint_x = match plank_idx {
            0 => 8,
            1 => 13,
            2 => 4,
            3 => 10,
            _ => 8,
        };

        for x in 0..TEX_SIZE {
            let mut col = base_col;

            let grain = ((x as f32 * 0.4 + (y as f32 * 1.5).sin()).sin() * 0.5 + 0.5) * 0.06;
            let noise = rand_at(x, y, 713) * 0.06 - 0.03;
            col[0] = (col[0] + grain + noise).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8 + noise).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6 + noise).clamp(0.0, 1.0);

            if in_plank_y == 0 && y > 0 {
                col = mix(col, bevel_highlight, 0.45);
            }

            if in_plank_y == 3 {
                col = groove_color;
            }

            if x == joint_x {
                col = groove_color;
            } else if x == (joint_x + 1) % 16 && in_plank_y > 0 && in_plank_y < 3 {
                col = mix(col, bevel_highlight, 0.30);
            }

            let is_nail = (x == (joint_x + 1) % 16 && in_plank_y == 1)
                || (x == (joint_x + 15) % 16 && in_plank_y == 2);
            if is_nail {
                col = nail_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }

    Texture { texels }
}

pub(super) fn glass() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let line = x == y || x + y == TEX_SIZE - 1;
            texels[y * TEX_SIZE + x] = if line { [0.72, 0.95, 1.0] } else { [0.40, 0.72, 0.88] };
        }
    }
    Texture { texels }
}

pub(super) fn door_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.38, 0.24, 0.12];
    let frame_light = [0.65, 0.48, 0.26];
    let glass_base = [0.62, 0.80, 0.88];
    let glass_shine = [0.85, 0.95, 1.0];
    let handle_color = [0.82, 0.72, 0.32];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = frame_color;

            let grain = rand_at(x, y, 921) * 0.08 - 0.04;
            col[0] = (col[0] + grain).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6).clamp(0.0, 1.0);

            let in_left_win1 = (x >= 2 && x <= 6) && (y >= 2 && y <= 6);
            let in_right_win1 = (x >= 9 && x <= 13) && (y >= 2 && y <= 6);
            let in_left_win2 = (x >= 2 && x <= 6) && (y >= 8 && y <= 12);
            let in_right_win2 = (x >= 9 && x <= 13) && (y >= 8 && y <= 12);

            let in_win = in_left_win1 || in_right_win1 || in_left_win2 || in_right_win2;

            if in_win {
                let is_win_border = (x == 2 || x == 6 || x == 9 || x == 13)
                    || (y == 2 || y == 6 || y == 8 || y == 12);

                if is_win_border {
                    if y == 2 || y == 8 || x == 2 || x == 9 {
                        col = frame_dark;
                    } else {
                        col = frame_light;
                    }
                } else {
                    let shine = x == y || (x + 1 == y);
                    col = if shine { glass_shine } else { glass_base };
                }
            } else {
                if x == 0 || x == 15 || y == 0 || y == 7 || y == 13 {
                    col = mix(col, frame_dark, 0.4);
                } else if x == 1 || x == 8 {
                    col = mix(col, frame_light, 0.3);
                }
            }

            if (x == 12 || x == 13) && (y == 14 || y == 15) {
                col = handle_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }

    Texture { texels }
}

pub(super) fn door_bottom() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.35, 0.22, 0.10];
    let frame_light = [0.65, 0.48, 0.26];
    let panel_color = [0.58, 0.42, 0.22];
    let handle_color = [0.82, 0.72, 0.32];
    let nail_color = [0.22, 0.15, 0.08];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = frame_color;

            let grain = rand_at(x, y, 922) * 0.08 - 0.04;
            col[0] = (col[0] + grain).clamp(0.0, 1.0);
            col[1] = (col[1] + grain * 0.8).clamp(0.0, 1.0);
            col[2] = (col[2] + grain * 0.6).clamp(0.0, 1.0);

            let in_left_panel = (x >= 2 && x <= 6) && (y >= 2 && y <= 13);
            let in_right_panel = (x >= 9 && x <= 13) && (y >= 2 && y <= 13);

            if in_left_panel || in_right_panel {
                let is_panel_border = (x == 2 || x == 6 || x == 9 || x == 13)
                    || (y == 2 || y == 13);

                if is_panel_border {
                    if y == 2 || x == 2 || x == 9 {
                        col = frame_dark;
                    } else {
                        col = frame_light;
                    }
                } else {
                    let p_grain = ((x as f32 * 1.5).sin() * 0.04) + rand_at(x, y, 923) * 0.04;
                    col = [
                        (panel_color[0] + p_grain).clamp(0.0, 1.0),
                        (panel_color[1] + p_grain).clamp(0.0, 1.0),
                        (panel_color[2] + p_grain).clamp(0.0, 1.0),
                    ];
                }
            } else {
                if x == 0 || x == 15 || y == 15 {
                    col = mix(col, frame_dark, 0.4);
                } else if x == 1 || x == 8 || y == 0 {
                    col = mix(col, frame_light, 0.3);
                }
            }

            if (x == 12 || x == 13) && (y == 0 || y == 1) {
                col = handle_color;
            }

            if y == 14 && (x == 3 || x == 7 || x == 11) {
                col = nail_color;
            }

            texels[y * TEX_SIZE + x] = col;
        }
    }


    Texture { texels }
}

pub(super) fn tnt_side() -> Texture {
    let red = [0.85, 0.15, 0.10];
    let white = [0.90, 0.90, 0.88];
    let black = [0.10, 0.10, 0.10];
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = red;
            if y >= 5 && y <= 10 {
                col = white;
                let is_text_pixel = match (x, y) {
                    (2, 6) | (3, 6) | (4, 6) | (3, 7) | (3, 8) | (3, 9) => true,
                    (6, 6) | (6, 7) | (6, 8) | (6, 9) |
                    (7, 7) | (8, 8) |
                    (9, 6) | (9, 7) | (9, 8) | (9, 9) => true,
                    (11, 6) | (12, 6) | (13, 6) | (12, 7) | (12, 8) | (12, 9) => true,
                    _ => false,
                };
                if is_text_pixel {
                    col = black;
                }
            }
            let n = rand_at(x, y, 808) * 0.08 - 0.04;
            texels[y * TEX_SIZE + x] = [
                (col[0] + n).clamp(0.0, 1.0),
                (col[1] + n).clamp(0.0, 1.0),
                (col[2] + n).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

pub(super) fn tnt_top() -> Texture {
    let base = [0.75, 0.70, 0.60];
    let dark = [0.45, 0.40, 0.35];
    let red = [0.85, 0.15, 0.10];
    let mut tex = speckled(base, 0.10, 809);

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 3 || x == 12 || y == 3 || y == 12 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
            if (x >= 6 && x <= 9) && (y >= 6 && y <= 9) {
                tex.texels[y * TEX_SIZE + x] = red;
            }
        }
    }
    tex
}

pub(super) fn crafting_table_top() -> Texture {
    let mut tex = planks();
    let border = [0.32, 0.20, 0.10];
    let grid_line = [0.28, 0.18, 0.08];
    let tool_metal = [0.80, 0.82, 0.85];
    let tool_handle = [0.45, 0.28, 0.12];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            // Outer frame
            if x <= 1 || x >= 14 || y <= 1 || y >= 14 {
                tex.texels[y * TEX_SIZE + x] = border;
            }
            // 3x3 crafting grid in center (x: 3..13, y: 3..13)
            if (x >= 3 && x <= 12) && (y >= 3 && y <= 12) {
                if x == 6 || x == 9 || y == 6 || y == 9 {
                    tex.texels[y * TEX_SIZE + x] = grid_line;
                }
            }
        }
    }
    // Small saw & shears detail on top corners
    tex.texels[2 * TEX_SIZE + 2] = tool_metal;
    tex.texels[2 * TEX_SIZE + 3] = tool_handle;
    tex.texels[13 * TEX_SIZE + 12] = tool_metal;
    tex.texels[13 * TEX_SIZE + 13] = tool_metal;
    tex
}

pub(super) fn crafting_table_side() -> Texture {
    let mut tex = planks();
    let frame = [0.30, 0.18, 0.08];
    let iron = [0.75, 0.78, 0.82];
    let handle = [0.40, 0.24, 0.10];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                tex.texels[y * TEX_SIZE + x] = frame;
            }
        }
    }
    // Saw hanging on side
    for y in 4..12 {
        tex.texels[y * TEX_SIZE + 4] = iron;
        if y % 2 == 0 {
            tex.texels[y * TEX_SIZE + 5] = iron;
        }
    }
    tex.texels[3 * TEX_SIZE + 4] = handle;
    tex.texels[2 * TEX_SIZE + 4] = handle;
    tex
}

pub(super) fn crafting_table_front() -> Texture {
    let mut tex = planks();
    let frame = [0.30, 0.18, 0.08];
    let iron = [0.75, 0.78, 0.82];
    let handle = [0.40, 0.24, 0.10];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                tex.texels[y * TEX_SIZE + x] = frame;
            }
        }
    }
    // Hammer and shears on front
    for y in 4..11 {
        tex.texels[y * TEX_SIZE + 11] = handle;
    }
    for x in 9..14 {
        tex.texels[4 * TEX_SIZE + x] = iron;
    }
    tex
}

pub(super) fn bookshelf_side() -> Texture {
    let wood = [0.65, 0.50, 0.30];
    let dark_wood = [0.45, 0.32, 0.18];
    let book_colors = [
        [0.68, 0.18, 0.18],
        [0.18, 0.42, 0.22],
        [0.20, 0.28, 0.65],
        [0.55, 0.35, 0.18],
        [0.72, 0.60, 0.20],
    ];
    let mut texels = vec![dark_wood; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let is_frame = y == 0 || y == 1 || y == 7 || y == 8 || y == 14 || y == 15 || x == 0 || x == 15;
            if is_frame {
                let n = (rand_at(x, y, 51) - 0.5) * 0.08;
                texels[y * TEX_SIZE + x] = mix(wood, dark_wood, n + 0.3);
            } else {
                let book_idx = (x / 2 + (y / 8) * 3) % book_colors.len();
                let col = book_colors[book_idx];
                let is_binding = x % 2 == 0;
                let page_edge = y == 2 || y == 9;
                if page_edge {
                    texels[y * TEX_SIZE + x] = [0.88, 0.85, 0.78];
                } else if is_binding {
                    texels[y * TEX_SIZE + x] = [col[0] * 0.8, col[1] * 0.8, col[2] * 0.8];
                } else {
                    texels[y * TEX_SIZE + x] = col;
                }
            }
        }
    }
    Texture { texels }
}
