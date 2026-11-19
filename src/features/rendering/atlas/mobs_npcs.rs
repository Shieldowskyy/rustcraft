//! Procedural 16x16 villager profession robes, face, and iron golem body/head textures.

use super::types::{speckled, Texture, TEX_SIZE};

pub(super) fn villager_skin() -> Texture {
    speckled([0.78, 0.58, 0.45], 0.06, 601)
}

pub(super) fn villager_face() -> Texture {
    let mut tex = villager_skin();
    let unibrow = [0.24, 0.14, 0.08];
    let eye_white = [0.95, 0.95, 0.95];
    let iris_green = [0.12, 0.72, 0.28];
    let pupil = [0.03, 0.03, 0.03];
    let nose_shadow = [0.65, 0.46, 0.34];
    let mouth = [0.48, 0.32, 0.22];

    for x in 3..=12 {
        tex.texels[5 * TEX_SIZE + x] = unibrow;
    }
    tex.texels[6 * TEX_SIZE + 7] = unibrow;
    tex.texels[6 * TEX_SIZE + 8] = unibrow;

    for y in 6..=7 {
        tex.texels[y * TEX_SIZE + 4] = eye_white;
        tex.texels[y * TEX_SIZE + 5] = iris_green;
        tex.texels[y * TEX_SIZE + 10] = iris_green;
        tex.texels[y * TEX_SIZE + 11] = eye_white;
    }
    tex.texels[7 * TEX_SIZE + 5] = pupil;
    tex.texels[7 * TEX_SIZE + 10] = pupil;

    for y in 7..=11 {
        tex.texels[y * TEX_SIZE + 7] = nose_shadow;
        tex.texels[y * TEX_SIZE + 8] = nose_shadow;
    }

    for x in 6..=9 {
        tex.texels[13 * TEX_SIZE + x] = mouth;
    }

    tex
}

pub(super) fn villager_robe() -> Texture {
    speckled([0.48, 0.28, 0.16], 0.08, 602)
}

pub(super) fn villager_arms() -> Texture {
    let mut tex = villager_robe();
    let skin = [0.78, 0.58, 0.45];
    let seam = [0.32, 0.18, 0.10];
    for y in 4..12 {
        tex.texels[y * TEX_SIZE + 5] = seam;
        tex.texels[y * TEX_SIZE + 10] = seam;
        for x in 6..10 {
            tex.texels[y * TEX_SIZE + x] = skin;
        }
    }
    tex
}

pub(super) fn villager_robe_librarian() -> Texture {
    let mut tex = speckled([0.90, 0.88, 0.85], 0.05, 603);
    let sash = [0.75, 0.18, 0.18];
    for x in 0..TEX_SIZE {
        tex.texels[9 * TEX_SIZE + x] = sash;
        tex.texels[10 * TEX_SIZE + x] = sash;
    }
    tex
}

pub(super) fn villager_robe_blacksmith() -> Texture {
    let mut tex = villager_robe();
    let apron = [0.18, 0.18, 0.20];
    let buckle = [0.85, 0.75, 0.25];
    for y in 4..16 {
        for x in 4..12 {
            tex.texels[y * TEX_SIZE + x] = apron;
        }
    }
    tex.texels[6 * TEX_SIZE + 4] = buckle;
    tex.texels[6 * TEX_SIZE + 11] = buckle;
    tex
}

pub(super) fn villager_robe_cleric() -> Texture {
    let mut tex = speckled([0.45, 0.18, 0.55], 0.06, 604);
    let gold = [0.92, 0.78, 0.22];
    for y in 4..14 {
        tex.texels[y * TEX_SIZE + 7] = gold;
        tex.texels[y * TEX_SIZE + 8] = gold;
    }
    for x in 5..11 {
        tex.texels[7 * TEX_SIZE + x] = gold;
    }
    tex
}

pub(super) fn villager_robe_butcher() -> Texture {
    let mut tex = villager_robe();
    let apron = [0.88, 0.88, 0.88];
    let red = [0.78, 0.15, 0.15];
    for y in 3..16 {
        for x in 4..12 {
            tex.texels[y * TEX_SIZE + x] = apron;
        }
    }
    tex.texels[2 * TEX_SIZE + 7] = red;
    tex.texels[2 * TEX_SIZE + 8] = red;
    tex
}

pub(super) fn iron_golem_body() -> Texture {
    let mut tex = speckled([0.84, 0.84, 0.82], 0.06, 620);
    let vine = [0.32, 0.60, 0.20];
    let vine_dark = [0.22, 0.45, 0.14];
    let seam = [0.65, 0.65, 0.63];
    for x in 0..TEX_SIZE {
        tex.texels[15 * TEX_SIZE + x] = seam;
        tex.texels[0 * TEX_SIZE + x] = seam;
    }
    for y in 4..12 {
        for x in 2..7 {
            if (x * 3 + y * 7) % 5 == 0 {
                tex.texels[y * TEX_SIZE + x] = if (x + y) % 2 == 0 { vine } else { vine_dark };
            }
        }
    }
    tex
}

pub(super) fn iron_golem_head() -> Texture {
    let mut tex = iron_golem_body();
    let eye_red = [0.78, 0.15, 0.12];
    let pupil = [0.35, 0.05, 0.05];
    let brow = [0.68, 0.68, 0.66];
    let mouth = [0.45, 0.45, 0.43];
    for x in 3..13 {
        tex.texels[6 * TEX_SIZE + x] = brow;
    }
    tex.texels[7 * TEX_SIZE + 4] = eye_red;
    tex.texels[7 * TEX_SIZE + 5] = pupil;
    tex.texels[7 * TEX_SIZE + 10] = pupil;
    tex.texels[7 * TEX_SIZE + 11] = eye_red;
    for x in 6..10 {
        tex.texels[13 * TEX_SIZE + x] = mouth;
    }
    tex
}
