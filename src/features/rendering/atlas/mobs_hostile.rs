//! Procedural 16x16 player, zombie, skeleton, reaper, and spider skin/face textures.

use super::types::{rand_at, speckled, Texture, TEX_SIZE};

pub(super) fn player_skin() -> Texture {
    speckled([0.76, 0.54, 0.42], 0.04, 501)
}

pub(super) fn player_hair() -> Texture {
    speckled([0.28, 0.18, 0.12], 0.05, 502)
}

pub(super) fn player_face() -> Texture {
    let mut tex = player_skin();
    let hair = [0.28, 0.18, 0.12];
    let white = [0.96, 0.96, 0.96];
    let eye_blue = [0.18, 0.22, 0.58];
    let nose = [0.66, 0.44, 0.34];
    let mouth_beard = [0.34, 0.20, 0.14];

    // Top hair band (y = 0..3 full width, y = 3..5 sides)
    for y in 0..3 {
        for x in 0..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
    }
    for y in 3..6 {
        for x in 0..2 {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
        for x in (TEX_SIZE - 2)..TEX_SIZE {
            tex.texels[y * TEX_SIZE + x] = hair;
        }
    }

    // Eyes: y = 7..9
    // Left eye: white at x = 3..4, pupil blue at x = 5..6
    for y in 7..9 {
        tex.texels[y * TEX_SIZE + 3] = white;
        tex.texels[y * TEX_SIZE + 4] = white;
        tex.texels[y * TEX_SIZE + 5] = eye_blue;
        tex.texels[y * TEX_SIZE + 6] = eye_blue;

        // Right eye: pupil blue at x = 9..10, white at x = 11..12
        tex.texels[y * TEX_SIZE + 9] = eye_blue;
        tex.texels[y * TEX_SIZE + 10] = eye_blue;
        tex.texels[y * TEX_SIZE + 11] = white;
        tex.texels[y * TEX_SIZE + 12] = white;
    }

    // Nose: y = 9..11, center
    for y in 9..11 {
        for x in 7..9 {
            tex.texels[y * TEX_SIZE + x] = nose;
        }
    }

    // Mouth / Beard: y = 12..14
    for y in 12..14 {
        for x in 5..11 {
            tex.texels[y * TEX_SIZE + x] = mouth_beard;
        }
    }

    tex
}

pub(super) fn player_shirt() -> Texture {
    speckled([0.0, 0.65, 0.68], 0.05, 503)
}

pub(super) fn player_shirt_front() -> Texture {
    let mut tex = player_shirt();
    let skin = [0.76, 0.54, 0.42];
    // V-neck neckline cutout at top center
    for y in 0..2 {
        for x in 5..11 {
            tex.texels[y * TEX_SIZE + x] = skin;
        }
    }
    for y in 2..4 {
        for x in 6..10 {
            tex.texels[y * TEX_SIZE + x] = skin;
        }
    }
    for x in 7..9 {
        tex.texels[4 * TEX_SIZE + x] = skin;
    }
    tex
}

pub(super) fn player_pants() -> Texture {
    speckled([0.18, 0.24, 0.48], 0.05, 504)
}

pub(super) fn player_shoes() -> Texture {
    speckled([0.20, 0.20, 0.22], 0.04, 505)
}

pub(super) fn zombie_skin() -> Texture {
    speckled([0.31, 0.48, 0.24], 0.08, 401)
}

pub(super) fn zombie_face() -> Texture {
    let mut tex = zombie_skin();
    let dark_eye = [0.08, 0.12, 0.06];
    let pupil = [0.02, 0.02, 0.02];
    let mouth = [0.12, 0.18, 0.09];
    let dark_nose = [0.18, 0.28, 0.14];

    for dy in 6..8 {
        tex.texels[dy * TEX_SIZE + 3] = dark_eye;
        tex.texels[dy * TEX_SIZE + 4] = pupil;
        tex.texels[dy * TEX_SIZE + 11] = pupil;
        tex.texels[dy * TEX_SIZE + 12] = dark_eye;
    }
    tex.texels[9 * TEX_SIZE + 7] = dark_nose;
    tex.texels[9 * TEX_SIZE + 8] = dark_nose;

    for x in 5..11 {
        tex.texels[12 * TEX_SIZE + x] = mouth;
    }
    tex
}

pub(super) fn zombie_shirt() -> Texture {
    speckled([0.16, 0.42, 0.44], 0.06, 402)
}

pub(super) fn zombie_pants() -> Texture {
    speckled([0.22, 0.18, 0.42], 0.06, 403)
}

pub(super) fn skeleton_bone() -> Texture {
    speckled([0.84, 0.83, 0.80], 0.06, 501)
}

pub(super) fn skeleton_face() -> Texture {
    let mut tex = skeleton_bone();
    let eye_socket = [0.10, 0.10, 0.10];
    let dark_slit = [0.22, 0.22, 0.22];

    for dy in 5..8 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = eye_socket;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = eye_socket;
        }
    }

    tex.texels[9 * TEX_SIZE + 7] = eye_socket;
    tex.texels[9 * TEX_SIZE + 8] = eye_socket;

    for x in 4..12 {
        if x % 2 == 0 {
            tex.texels[12 * TEX_SIZE + x] = dark_slit;
            tex.texels[13 * TEX_SIZE + x] = dark_slit;
        }
    }
    tex
}

pub(super) fn skeleton_body() -> Texture {
    let mut tex = skeleton_bone();
    let shadow = [0.20, 0.20, 0.20];
    for y in 0..TEX_SIZE {
        tex.texels[y * TEX_SIZE + 7] = shadow;
        tex.texels[y * TEX_SIZE + 8] = shadow;
    }
    for y in [4, 7, 10, 13] {
        for x in 2..14 {
            if x != 7 && x != 8 {
                tex.texels[y * TEX_SIZE + x] = shadow;
            }
        }
    }
    tex
}

pub(super) fn reaper_skin() -> Texture {
    let mut tex = speckled([0.22, 0.62, 0.16], 0.18, 601);
    let lime = [0.38, 0.76, 0.22];
    let dark = [0.12, 0.38, 0.08];
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let r = rand_at(x, y, 602);
            if r > 0.72 {
                tex.texels[y * TEX_SIZE + x] = lime;
            } else if r < 0.28 {
                tex.texels[y * TEX_SIZE + x] = dark;
            }
        }
    }
    tex
}

pub(super) fn reaper_face() -> Texture {
    let mut tex = reaper_skin();
    let black = [0.03, 0.03, 0.03];

    // Left and right eyes (3x3 blocks)
    for dy in 4..7 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Center vertical bridge between eyes
    for dy in 7..9 {
        for dx in 7..9 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Top mouth bar
    for dy in 9..11 {
        for dx in 5..11 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    // Mouth corners dropping down
    for dy in 11..14 {
        for dx in 3..6 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
        for dx in 10..13 {
            tex.texels[dy * TEX_SIZE + dx] = black;
        }
    }

    tex
}

pub(super) fn spider_skin() -> Texture {
    speckled([0.16, 0.14, 0.13], 0.12, 831)
}

pub(super) fn spider_face() -> Texture {
    let mut tex = spider_skin();
    let eye_bright = [0.95, 0.12, 0.12];
    let eye_dim = [0.65, 0.08, 0.08];
    tex.texels[7 * TEX_SIZE + 5] = eye_bright;
    tex.texels[7 * TEX_SIZE + 6] = eye_bright;
    tex.texels[7 * TEX_SIZE + 9] = eye_bright;
    tex.texels[7 * TEX_SIZE + 10] = eye_bright;
    tex.texels[8 * TEX_SIZE + 3] = eye_dim;
    tex.texels[8 * TEX_SIZE + 4] = eye_bright;
    tex.texels[8 * TEX_SIZE + 11] = eye_bright;
    tex.texels[8 * TEX_SIZE + 12] = eye_dim;
    tex
}
