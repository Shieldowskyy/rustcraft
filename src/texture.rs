// Basic procedural 16x16 textures, generated once at startup with simple
// deterministic hash-based noise (no image files, no external assets).
// Each texture is a flat Vec<[f32;3]> of 16*16 RGB texels in row-major
// order (row 0 = top). The renderer samples these with nearest-neighbor
// filtering using UV coordinates baked into the mesh.

pub const TEX_SIZE: usize = 16;

#[derive(Clone)]
pub struct Texture {
    pub texels: Vec<[f32; 3]>, // len == TEX_SIZE * TEX_SIZE
}

impl Texture {
    #[inline(always)]
    pub fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        let x = (u * TEX_SIZE as f32) as i32;
        let y = (v * TEX_SIZE as f32) as i32;
        let ux = x.rem_euclid(TEX_SIZE as i32) as usize;
        let uy = y.rem_euclid(TEX_SIZE as i32) as usize;
        self.texels[uy * TEX_SIZE + ux]
    }
}

fn hash(mut n: u32) -> u32 {
    n = (n ^ 61) ^ (n >> 16);
    n = n.wrapping_add(n << 3);
    n ^= n >> 4;
    n = n.wrapping_mul(0x27d4eb2d);
    n ^= n >> 15;
    n
}

/// Deterministic per-texel pseudo-random value in [0, 1), seeded by texel
/// coordinates and a texture-specific seed so every block gets a distinct,
/// stable pattern.
fn rand_at(x: usize, y: usize, seed: u32) -> f32 {
    let h = hash((x as u32).wrapping_mul(1973).wrapping_add((y as u32).wrapping_mul(9277)).wrapping_add(seed.wrapping_mul(26699)));
    (h % 10000) as f32 / 10000.0
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn speckled(base: [f32; 3], variance: f32, seed: u32) -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n = rand_at(x, y, seed) - 0.5; // [-0.5, 0.5)
            let shade = 1.0 + n * variance;
            texels[y * TEX_SIZE + x] = [
                (base[0] * shade).clamp(0.0, 1.0),
                (base[1] * shade).clamp(0.0, 1.0),
                (base[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

/// Grass top: green speckled with small lighter tufts.
fn grass_top() -> Texture {
    let base = [0.36, 0.68, 0.24];
    let light = [0.48, 0.80, 0.32];
    let mut tex = speckled(base, 0.18, 11);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 42) > 0.86 {
                tex.texels[y * TEX_SIZE + x] = light;
            }
        }
    }
    tex
}

/// Grass side: dirt base with a strip of grass-green along the top edge,
/// and a jagged (noisy) boundary between the two, like classic Minecraft.
fn grass_side() -> Texture {
    let dirt = [0.45, 0.33, 0.20];
    let grass = [0.36, 0.68, 0.24];
    let mut tex = speckled(dirt, 0.15, 7);
    for x in 0..TEX_SIZE {
        let edge = 3 + (rand_at(x, 0, 99) * 3.0) as usize; // 3..6 texels of grass
        for y in 0..edge.min(TEX_SIZE) {
            let n = rand_at(x, y, 13) - 0.5;
            let shade = 1.0 + n * 0.18;
            tex.texels[y * TEX_SIZE + x] = [
                (grass[0] * shade).clamp(0.0, 1.0),
                (grass[1] * shade).clamp(0.0, 1.0),
                (grass[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    tex
}

fn dirt() -> Texture {
    speckled([0.45, 0.33, 0.20], 0.20, 3)
}

/// Stone with darker speckles and a few faint "cracks" for texture.
fn stone() -> Texture {
    let base = [0.55, 0.55, 0.55];
    let mut tex = speckled(base, 0.14, 21);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 71) > 0.93 {
                tex.texels[y * TEX_SIZE + x] = mix(tex.texels[y * TEX_SIZE + x], [0.3, 0.3, 0.3], 0.6);
            }
        }
    }
    tex
}

fn sand() -> Texture {
    speckled([0.87, 0.80, 0.55], 0.12, 5)
}

/// Wood top/bottom: concentric-ish rings using distance from center.
fn wood_top() -> Texture {
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

/// Wood side: vertical bark stripes.
fn wood_side() -> Texture {
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

/// Leaves: green with random small gaps of a darker shade for a "leafy"
/// look, and occasional tiny fully-see-through-looking dark specks.
fn leaves() -> Texture {
    let base = [0.20, 0.50, 0.16];
    let mut tex = speckled(base, 0.35, 33);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            if rand_at(x, y, 200) > 0.90 {
                tex.texels[y * TEX_SIZE + x] = [0.10, 0.30, 0.08];
            }
        }
    }
    tex
}

/// Water: blue with soft horizontal wave banding.
fn water() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let base = [0.15, 0.35, 0.75];
    let light = [0.25, 0.48, 0.85];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let wave = ((y as f32 * 0.8 + x as f32 * 0.2).sin() * 0.5 + 0.5) * 0.5
            + (rand_at(x, y, 55) * 0.15);
            texels[y * TEX_SIZE + x] = mix(base, light, wave.clamp(0.0, 1.0));
        }
    }
    Texture { texels }
}

/// Snow: clean bright white with subtle cool blue-white crystalline flecks.
fn snow() -> Texture {
    let base = [0.94, 0.96, 0.99];
    let light = [1.0, 1.0, 1.0];
    let shadow = [0.86, 0.90, 0.96];
    let mut tex = speckled(base, 0.08, 77);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 105);
            if r > 0.90 {
                tex.texels[y * TEX_SIZE + x] = light;
            } else if r < 0.10 {
                tex.texels[y * TEX_SIZE + x] = shadow;
            }
        }
    }
    tex
}

/// Cactus top: green with concentric ring and dark central core.
fn cactus_top() -> Texture {
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

/// Cactus side: vertical green ribs with dark needle/spine clusters.
fn cactus_side() -> Texture {
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

/// Coal ore: stone with embedded rough black/charcoal nuggets.
fn coal_ore() -> Texture {
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

/// Iron ore: stone with embedded rust-orange/tan ore veins.
fn iron_ore() -> Texture {
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

/// Bedrock: very dark grey with a cracked/irregular pattern.
fn bedrock() -> Texture {
    let base = [0.20, 0.20, 0.20];
    let crack = [0.10, 0.10, 0.10];
    let light = [0.30, 0.30, 0.30];
    let mut tex = speckled(base, 0.12, 191);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let n1 = rand_at(x, y, 311);
            let n2 = rand_at((x + 3) % TEX_SIZE, (y + 5) % TEX_SIZE, 413);
            if n1 > 0.88 || n2 > 0.88 {
                tex.texels[y * TEX_SIZE + x] = crack;
            } else if n1 < 0.06 {
                tex.texels[y * TEX_SIZE + x] = light;
            }
        }
    }
    tex
}

/// Lava: glowing orange-red with molten rock veins.
fn lava() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    let hot = [1.0, 0.55, 0.05];
    let dark = [0.55, 0.10, 0.02];
    let bright = [1.0, 0.85, 0.20];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let px = x as f32 / TEX_SIZE as f32 * std::f32::consts::TAU;
            let py = y as f32 / TEX_SIZE as f32 * std::f32::consts::TAU;
            let wave = ((px.sin() + py.cos()) * 0.5 + 0.5)
            + (((px * 2.0 + py).sin()) * 0.3);
            let n = rand_at(x, y, 777) * 0.25;
            let t = (wave + n).clamp(0.0, 1.0);
            let col = if t > 0.80 {
                mix(hot, bright, (t - 0.80) / 0.20)
            } else {
                mix(dark, hot, t / 0.80)
            };
            texels[y * TEX_SIZE + x] = col;
        }
    }
    Texture { texels }
}

/// Snow Leaves: icy blue-green winter foliage.
fn snow_leaves() -> Texture {
    let base = [0.50, 0.68, 0.72];
    let light = [0.70, 0.85, 0.90];
    let dark = [0.35, 0.52, 0.60];
    let mut tex = speckled(base, 0.22, 444);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 555);
            if r > 0.88 {
                tex.texels[y * TEX_SIZE + x] = light;
            } else if r < 0.10 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

/// Side texture for TNT: red body with white center strip and black markings.
fn tnt_side() -> Texture {
    let red = [0.85, 0.15, 0.10];
    let white = [0.90, 0.90, 0.88];
    let black = [0.10, 0.10, 0.10];
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut col = red;
            // Central white band
            if y >= 5 && y <= 10 {
                col = white;
                // Simple pixel art pattern resembling "TNT" lettering
                let is_text_pixel = match (x, y) {
                    // 'T'
                    (2, 6) | (3, 6) | (4, 6) | (3, 7) | (3, 8) | (3, 9) => true,
                    // 'N'
                    (6, 6) | (6, 7) | (6, 8) | (6, 9) |
                    (7, 7) | (8, 8) |
                    (9, 6) | (9, 7) | (9, 8) | (9, 9) => true,
                    // 'T'
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

/// Top/Bottom texture for TNT: gunpowder surface with inner square/fuse markings.
fn tnt_top() -> Texture {
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

/// Oak planks: 4 horizontal wooden planks with beveled shadow grooves,
/// staggered vertical joints, nail studs, and oak wood grain.
fn planks() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let plank_base_tones = [
        [0.69, 0.53, 0.31], // Plank 0: warm medium oak
        [0.64, 0.48, 0.28], // Plank 1: slightly darker oak
        [0.72, 0.56, 0.34], // Plank 2: slightly lighter oak
        [0.66, 0.50, 0.29], // Plank 3: golden oak
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

/// Top half of a 2-block wooden door: outer frame, 4 latticed window cutouts,
/// center mullion, and upper portion of the door latch/handle.
fn door_top() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.38, 0.24, 0.12];
    let frame_light = [0.65, 0.48, 0.26];
    let glass_base = [0.62, 0.80, 0.88];
    let glass_shine = [0.85, 0.95, 1.0];
    let handle_color = [0.82, 0.72, 0.32]; // Brass handle

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

/// Bottom half of a 2-block wooden door: matching frame, 2 recessed carved
/// wooden panels with 3D bevel shading, and lower portion of door handle.
fn door_bottom() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    let frame_color = [0.52, 0.36, 0.18];
    let frame_dark = [0.35, 0.22, 0.10];
    let frame_light = [0.65, 0.48, 0.26];
    let panel_color = [0.58, 0.42, 0.22];
    let handle_color = [0.82, 0.72, 0.32]; // Brass handle
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

/// All textures needed to render the world, indexed by `TextureId`.
pub struct TextureAtlas {
    pub grass_top: Texture,
    pub grass_side: Texture,
    pub dirt: Texture,
    pub stone: Texture,
    pub sand: Texture,
    pub wood_top: Texture,
    pub wood_side: Texture,
    pub leaves: Texture,
    pub water: Texture,
    pub snow: Texture,
    pub cactus_top: Texture,
    pub cactus_side: Texture,
    pub coal_ore: Texture,
    pub iron_ore: Texture,
    pub bedrock: Texture,
    pub lava: Texture,
    pub snow_leaves: Texture,
    pub planks: Texture,
    pub door: Texture,
    pub door_top: Texture,
    pub glass: Texture,
    pub ice: Texture,
    pub tnt_top: Texture,
    pub tnt_side: Texture,
    pub pig_skin: Texture,
    pub pig_face: Texture,
    pub pig_snout: Texture,
    pub cow_skin: Texture,
    pub cow_face: Texture,
    pub cow_snout: Texture,
    pub cow_horns: Texture,
    pub cow_udder: Texture,
}

impl TextureAtlas {
    pub fn generate() -> TextureAtlas {
        TextureAtlas {
            grass_top: grass_top(),
            grass_side: grass_side(),
            dirt: dirt(),
            stone: stone(),
            sand: sand(),
            wood_top: wood_top(),
            wood_side: wood_side(),
            leaves: leaves(),
            water: water(),
            snow: snow(),
            cactus_top: cactus_top(),
            cactus_side: cactus_side(),
            coal_ore: coal_ore(),
            iron_ore: iron_ore(),
            bedrock: bedrock(),
            lava: lava(),
            snow_leaves: snow_leaves(),
            planks: planks(),
            door: door_bottom(),
            door_top: door_top(),
            glass: glass(),
            ice: speckled([0.70, 0.88, 1.0], 0.08, 903),
            tnt_top: tnt_top(),
            tnt_side: tnt_side(),
            pig_skin: pig_skin(),
            pig_face: pig_face(),
            pig_snout: pig_snout(),
            cow_skin: cow_skin(),
            cow_face: cow_face(),
            cow_snout: cow_snout(),
            cow_horns: cow_horns(),
            cow_udder: cow_udder(),
        }
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        match id {
            TextureId::GrassTop => &self.grass_top,
            TextureId::GrassSide => &self.grass_side,
            TextureId::Dirt => &self.dirt,
            TextureId::Stone => &self.stone,
            TextureId::Sand => &self.sand,
            TextureId::WoodTop => &self.wood_top,
            TextureId::WoodSide => &self.wood_side,
            TextureId::Leaves => &self.leaves,
            TextureId::Water => &self.water,
            TextureId::Snow => &self.snow,
            TextureId::CactusTop => &self.cactus_top,
            TextureId::CactusSide => &self.cactus_side,
            TextureId::CoalOre => &self.coal_ore,
            TextureId::IronOre => &self.iron_ore,
            TextureId::Bedrock => &self.bedrock,
            TextureId::Lava => &self.lava,
            TextureId::SnowLeaves => &self.snow_leaves,
            TextureId::Planks => &self.planks,
            TextureId::Door => &self.door,
            TextureId::DoorTop => &self.door_top,
            TextureId::Glass => &self.glass,
            TextureId::Ice => &self.ice,
            TextureId::TntTop => &self.tnt_top,
            TextureId::TntSide => &self.tnt_side,
            TextureId::PigSkin => &self.pig_skin,
            TextureId::PigFace => &self.pig_face,
            TextureId::PigSnout => &self.pig_snout,
            TextureId::CowSkin => &self.cow_skin,
            TextureId::CowFace => &self.cow_face,
            TextureId::CowSnout => &self.cow_snout,
            TextureId::CowHorns => &self.cow_horns,
            TextureId::CowUdder => &self.cow_udder,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextureId {
    GrassTop,
    GrassSide,
    Dirt,
    Stone,
    Sand,
    WoodTop,
    WoodSide,
    Leaves,
    Water,
    Snow,
    CactusTop,
    CactusSide,
    CoalOre,
    IronOre,
    Bedrock,
    Lava,
    SnowLeaves,
    Planks,
    Door,
    DoorTop,
    Glass,
    Ice,
    TntTop,
    TntSide,
    PigSkin,
    PigFace,
    PigSnout,
    CowSkin,
    CowFace,
    CowSnout,
    CowHorns,
    CowUdder,
}

fn glass() -> Texture {
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let line = x == y || x + y == TEX_SIZE - 1;
            texels[y * TEX_SIZE + x] = if line { [0.72, 0.95, 1.0] } else { [0.40, 0.72, 0.88] };
        }
    }
    Texture { texels }
}

fn pig_skin() -> Texture {
    speckled([0.94, 0.65, 0.65], 0.08, 301)
}

fn pig_face() -> Texture {
    let mut tex = pig_skin();
    let white = [0.98, 0.98, 0.98];
    let pupil = [0.12, 0.08, 0.08];
    let snout_bg = [0.86, 0.52, 0.52];
    let nostril = [0.46, 0.22, 0.22];

    // Left eye (looking out left)
    tex.texels[6 * TEX_SIZE + 2] = white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    // Right eye (looking out right)
    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = white;

    // Snout area on front face
    for y in 9..14 {
        for x in 4..12 {
            tex.texels[y * TEX_SIZE + x] = snout_bg;
        }
    }
    // Nostrils
    tex.texels[10 * TEX_SIZE + 6] = nostril;
    tex.texels[11 * TEX_SIZE + 6] = nostril;
    tex.texels[10 * TEX_SIZE + 9] = nostril;
    tex.texels[11 * TEX_SIZE + 9] = nostril;

    tex
}

fn pig_snout() -> Texture {
    let mut tex = speckled([0.88, 0.54, 0.54], 0.06, 302);
    let nostril = [0.46, 0.22, 0.22];
    for y in 6..11 {
        for x in 4..7 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
        for x in 9..12 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
    }
    tex
}

fn cow_skin() -> Texture {
    let dark = [0.26, 0.18, 0.14];
    let white = [0.88, 0.87, 0.84];
    let mut texels = vec![[0.0; 3]; TEX_SIZE * TEX_SIZE];

    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let bx = x / 4;
            let by = y / 4;
            let n = rand_at(bx, by, 701) + rand_at(x, y, 702) * 0.3;
            let is_white = n > 0.62;
            let shade = 0.92 + rand_at(x, y, 703) * 0.16;
            let base = if is_white { white } else { dark };
            texels[y * TEX_SIZE + x] = [
                (base[0] * shade).clamp(0.0, 1.0),
                (base[1] * shade).clamp(0.0, 1.0),
                (base[2] * shade).clamp(0.0, 1.0),
            ];
        }
    }
    Texture { texels }
}

fn cow_face() -> Texture {
    let mut tex = cow_skin();
    let white = [0.90, 0.89, 0.86];
    let eye_white = [0.98, 0.98, 0.98];
    let pupil = [0.10, 0.08, 0.08];
    let muzzle = [0.68, 0.58, 0.58];
    let nostril = [0.22, 0.16, 0.16];

    // White blaze between eyes
    for y in 1..8 {
        for x in 6..10 {
            let s = 0.95 + rand_at(x, y, 705) * 0.1;
            tex.texels[y * TEX_SIZE + x] = [white[0] * s, white[1] * s, white[2] * s];
        }
    }

    // Eyes
    tex.texels[6 * TEX_SIZE + 2] = eye_white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = eye_white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = eye_white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = eye_white;

    // Muzzle / snout
    for y in 9..15 {
        for x in 3..13 {
            tex.texels[y * TEX_SIZE + x] = muzzle;
        }
    }
    tex.texels[11 * TEX_SIZE + 5] = nostril;
    tex.texels[12 * TEX_SIZE + 5] = nostril;
    tex.texels[11 * TEX_SIZE + 10] = nostril;
    tex.texels[12 * TEX_SIZE + 10] = nostril;

    tex
}

fn cow_snout() -> Texture {
    let mut tex = speckled([0.68, 0.58, 0.58], 0.06, 706);
    let nostril = [0.22, 0.16, 0.16];
    for y in 6..11 {
        for x in 4..7 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
        for x in 9..12 {
            tex.texels[y * TEX_SIZE + x] = nostril;
        }
    }
    tex
}

fn cow_horns() -> Texture {
    let mut tex = speckled([0.76, 0.74, 0.70], 0.08, 707);
    let dark = [0.48, 0.46, 0.42];
    for y in 12..16 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = dark;
        }
    }
    tex
}

fn cow_udder() -> Texture {
    let mut tex = speckled([0.88, 0.65, 0.68], 0.05, 708);
    let teat = [0.68, 0.42, 0.45];
    for (tx, ty) in [(4, 4), (11, 4), (4, 11), (11, 11)] {
        for dy in 0..2 {
            for dx in 0..2 {
                tex.texels[(ty + dy) * TEX_SIZE + (tx + dx)] = teat;
            }
        }
    }
    tex
}

