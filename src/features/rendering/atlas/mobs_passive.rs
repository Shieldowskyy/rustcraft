//! Procedural 16x16 passive livestock and aquatic mob skin/face textures.

use super::types::{rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn pig_skin() -> Texture {
    speckled([0.94, 0.65, 0.65], 0.08, 301)
}

pub(super) fn pig_face() -> Texture {
    let mut tex = pig_skin();
    let white = [0.98, 0.98, 0.98];
    let pupil = [0.12, 0.08, 0.08];
    let snout_bg = [0.86, 0.52, 0.52];
    let nostril = [0.46, 0.22, 0.22];

    tex.texels[6 * TEX_SIZE + 2] = white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = white;

    for y in 9..14 {
        for x in 4..12 {
            tex.texels[y * TEX_SIZE + x] = snout_bg;
        }
    }
    tex.texels[10 * TEX_SIZE + 6] = nostril;
    tex.texels[11 * TEX_SIZE + 6] = nostril;
    tex.texels[10 * TEX_SIZE + 9] = nostril;
    tex.texels[11 * TEX_SIZE + 9] = nostril;

    tex
}

pub(super) fn pig_snout() -> Texture {
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

pub(super) fn cow_skin() -> Texture {
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

pub(super) fn cow_face() -> Texture {
    let mut tex = cow_skin();
    let white = [0.90, 0.89, 0.86];
    let eye_white = [0.98, 0.98, 0.98];
    let pupil = [0.10, 0.08, 0.08];
    let muzzle = [0.68, 0.58, 0.58];
    let nostril = [0.22, 0.16, 0.16];

    for y in 1..8 {
        for x in 6..10 {
            let s = 0.95 + rand_at(x, y, 705) * 0.1;
            tex.texels[y * TEX_SIZE + x] = [white[0] * s, white[1] * s, white[2] * s];
        }
    }

    tex.texels[6 * TEX_SIZE + 2] = eye_white;
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[7 * TEX_SIZE + 2] = eye_white;
    tex.texels[7 * TEX_SIZE + 3] = pupil;

    tex.texels[6 * TEX_SIZE + 12] = pupil;
    tex.texels[6 * TEX_SIZE + 13] = eye_white;
    tex.texels[7 * TEX_SIZE + 12] = pupil;
    tex.texels[7 * TEX_SIZE + 13] = eye_white;

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

pub(super) fn cow_snout() -> Texture {
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

pub(super) fn cow_horns() -> Texture {
    let mut tex = speckled([0.76, 0.74, 0.70], 0.08, 707);
    let dark = [0.48, 0.46, 0.42];
    for y in 12..16 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = dark;
        }
    }
    tex
}

pub(super) fn cow_udder() -> Texture {
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

pub(super) fn chicken_feathers() -> Texture {
    speckled([0.95, 0.95, 0.93], 0.04, 884)
}

pub(super) fn chicken_face() -> Texture {
    let mut tex = chicken_feathers();
    let eye_white = [0.99, 0.99, 0.99];
    let pupil = [0.05, 0.05, 0.05];
    let pupil_shine = [0.35, 0.35, 0.40];
    let beak = [0.96, 0.68, 0.10];
    let wattle = [0.88, 0.12, 0.12];

    // High-contrast, bold chicken eyes at the top corners of the face
    // Left eye (x=1..4, y=2..5)
    for y in 2..=5 {
        for x in 1..=4 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
    }
    tex.texels[3 * TEX_SIZE + 2] = pupil;
    tex.texels[4 * TEX_SIZE + 2] = pupil;
    tex.texels[3 * TEX_SIZE + 3] = pupil;
    tex.texels[4 * TEX_SIZE + 3] = pupil;
    tex.texels[2 * TEX_SIZE + 3] = pupil_shine;

    // Right eye (x=11..14, y=2..5)
    for y in 2..=5 {
        for x in 11..=14 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
    }
    tex.texels[3 * TEX_SIZE + 12] = pupil;
    tex.texels[4 * TEX_SIZE + 12] = pupil;
    tex.texels[3 * TEX_SIZE + 13] = pupil;
    tex.texels[4 * TEX_SIZE + 13] = pupil;
    tex.texels[2 * TEX_SIZE + 12] = pupil_shine;

    // Beak center in lower middle
    for y in 7..=10 {
        for x in 5..=10 {
            tex.texels[y * TEX_SIZE + x] = beak;
        }
    }

    // Wattle under beak
    for y in 11..=14 {
        for x in 6..=9 {
            tex.texels[y * TEX_SIZE + x] = wattle;
        }
    }

    tex
}

pub(super) fn chicken_beak() -> Texture {
    speckled([0.95, 0.65, 0.10], 0.05, 885)
}

pub(super) fn chicken_wattle() -> Texture {
    speckled([0.85, 0.15, 0.15], 0.05, 886)
}

pub(super) fn chicken_leg() -> Texture {
    speckled([0.92, 0.60, 0.10], 0.05, 887)
}

pub(super) fn sheep_skin() -> Texture {
    let base = [0.88, 0.76, 0.70];
    speckled(base, 0.08, 431)
}

pub(super) fn sheep_face() -> Texture {
    let mut tex = sheep_skin();
    let eye_white = [0.95, 0.95, 0.95];
    let pupil = [0.12, 0.10, 0.08];
    let snout_pink = [0.86, 0.58, 0.62];
    let nostril = [0.65, 0.40, 0.44];

    for y in 5..=7 {
        for x in 2..=4 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
        for x in 11..=13 {
            tex.texels[y * TEX_SIZE + x] = eye_white;
        }
    }
    tex.texels[6 * TEX_SIZE + 3] = pupil;
    tex.texels[6 * TEX_SIZE + 4] = pupil;
    tex.texels[6 * TEX_SIZE + 11] = pupil;
    tex.texels[6 * TEX_SIZE + 12] = pupil;

    for y in 10..=13 {
        for x in 5..=10 {
            tex.texels[y * TEX_SIZE + x] = snout_pink;
        }
    }
    tex.texels[12 * TEX_SIZE + 6] = nostril;
    tex.texels[12 * TEX_SIZE + 9] = nostril;

    tex
}

pub(super) fn dolphin_skin() -> Texture {
    speckled([0.52, 0.62, 0.74], 0.04, 881)
}

pub(super) fn dolphin_belly() -> Texture {
    speckled([0.84, 0.90, 0.94], 0.03, 882)
}

pub(super) fn cod_skin() -> Texture {
    speckled([0.68, 0.62, 0.52], 0.06, 883)
}

pub(super) fn salmon_skin() -> Texture {
    speckled([0.88, 0.38, 0.34], 0.06, 884)
}

pub(super) fn pufferfish_skin() -> Texture {
    speckled([0.90, 0.80, 0.25], 0.06, 885)
}

pub(super) fn tropical_fish_skin() -> Texture {
    speckled([0.96, 0.50, 0.14], 0.05, 886)
}
