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
            // Add spine dots every 4 pixels vertically on ridges
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
    pub glass: Texture,
    pub ice: Texture,
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
            planks: speckled([0.62, 0.42, 0.20], 0.12, 901),
            door: speckled([0.50, 0.30, 0.12], 0.10, 902),
            glass: glass(),
            ice: speckled([0.70, 0.88, 1.0], 0.08, 903),
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
            TextureId::Glass => &self.glass,
            TextureId::Ice => &self.ice,
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
    Glass,
    Ice,
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
