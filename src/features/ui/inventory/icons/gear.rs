//! Proceduralne ikony narzędzi, broni, zbroi, tarczy, łuku i wędki.

use crate::features::blocks::BlockId;
use crate::features::ui::draw::pack_color;

use super::IconGrid;

fn get_mat_colors(b: BlockId) -> (u32, u32, u32, u32) {
    match b {
        BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => (
            pack_color(0.85, 0.68, 0.42),
            pack_color(0.68, 0.50, 0.30),
            pack_color(0.48, 0.32, 0.16),
            pack_color(0.32, 0.20, 0.10),
        ),
        BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => (
            pack_color(0.78, 0.78, 0.80),
            pack_color(0.58, 0.58, 0.60),
            pack_color(0.40, 0.40, 0.42),
            pack_color(0.24, 0.24, 0.26),
        ),
        BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => (
            pack_color(1.00, 0.95, 0.40),
            pack_color(0.95, 0.80, 0.20),
            pack_color(0.80, 0.65, 0.10),
            pack_color(0.50, 0.38, 0.05),
        ),
        BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => (
            pack_color(0.75, 1.00, 1.00),
            pack_color(0.35, 0.90, 0.90),
            pack_color(0.18, 0.72, 0.75),
            pack_color(0.08, 0.45, 0.48),
        ),
        _ => (
            pack_color(1.00, 1.00, 1.00),
            pack_color(0.90, 0.93, 0.98),
            pack_color(0.72, 0.76, 0.84),
            pack_color(0.42, 0.46, 0.54),
        ),
    }
}

fn get_armor_colors(b: BlockId) -> (u32, u32, u32, u32, u32) {
    match b {
        BlockId::LeatherHelmet | BlockId::LeatherChestplate | BlockId::LeatherLeggings | BlockId::LeatherBoots => (
            pack_color(0.76, 0.50, 0.28),
            pack_color(0.65, 0.40, 0.20),
            pack_color(0.52, 0.30, 0.14),
            pack_color(0.36, 0.18, 0.06),
            pack_color(0.22, 0.10, 0.04),
        ),
        BlockId::IronHelmet | BlockId::IronChestplate | BlockId::IronLeggings | BlockId::IronBoots => (
            pack_color(1.0, 1.0, 1.0),
            pack_color(0.95, 0.96, 0.98),
            pack_color(0.80, 0.82, 0.86),
            pack_color(0.52, 0.55, 0.60),
            pack_color(0.24, 0.25, 0.28),
        ),
        BlockId::GoldHelmet | BlockId::GoldChestplate | BlockId::GoldLeggings | BlockId::GoldBoots => (
            pack_color(1.0, 1.0, 0.7),
            pack_color(1.00, 0.95, 0.45),
            pack_color(0.92, 0.78, 0.18),
            pack_color(0.68, 0.52, 0.08),
            pack_color(0.38, 0.28, 0.04),
        ),
        _ => (
            pack_color(0.88, 1.0, 1.0),
            pack_color(0.65, 0.98, 0.98),
            pack_color(0.25, 0.82, 0.85),
            pack_color(0.12, 0.52, 0.56),
            pack_color(0.06, 0.30, 0.34),
        ),
    }
}

pub(super) fn draw_gear_icon(g: &mut IconGrid<'_>, block: BlockId) -> bool {
    let wood_dark = pack_color(0.30, 0.18, 0.08);
    let wood_mid = pack_color(0.55, 0.35, 0.15);
    let wood_light = pack_color(0.72, 0.52, 0.28);

    match block {
        BlockId::Stick => {
            for i in 2..=13 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
                g.px(i, 16 - i, wood_dark);
            }
        }
        BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::GoldSword | BlockId::DiamondSword => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            g.pts(wood_dark, &[(2, 13), (2, 12), (3, 10), (5, 12), (2, 9)]);
            g.pts(wood_mid, &[(1, 14), (3, 12), (4, 11)]);
            g.px(3, 11, wood_light);
            g.pts(c_base, &[(4, 10), (5, 11)]);
            g.pts(c_hi, &[(5, 10), (6, 9), (12, 3)]);
            for i in 5..=11 {
                g.px(i, 15 - i, c_base);
                g.px(i - 1, 15 - i, c_hi);
                g.px(i + 1, 15 - i, c_shadow);
            }
            g.pts(c_glint, &[(13, 2), (12, 2)]);
            g.px(13, 3, c_shadow);
        }
        BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe | BlockId::GoldPickaxe | BlockId::DiamondPickaxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=10 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
                g.px(i, 16 - i, wood_dark);
            }
            g.pts(wood_dark, &[(9, 6), (10, 5)]);
            g.pts(c_shadow, &[(4, 7), (12, 5), (11, 6), (10, 7), (9, 8)]);
            g.pts(c_base, &[(4, 6), (5, 5), (6, 4), (8, 3), (12, 4), (11, 5)]);
            g.pts(c_hi, &[(5, 4), (7, 3), (9, 2), (10, 2), (13, 3)]);
            g.pts(c_glint, &[(11, 2), (12, 2), (13, 2)]);
        }
        BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe | BlockId::GoldAxe | BlockId::DiamondAxe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=11 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
                g.px(i, 16 - i, wood_dark);
            }
            g.pts(c_glint, &[(9, 2), (10, 2), (8, 3)]);
            g.pts(c_hi, &[(11, 2), (9, 3), (8, 4), (7, 5)]);
            g.pts(c_base, &[(12, 2), (10, 3), (9, 4), (10, 4), (8, 5), (7, 6)]);
            g.pts(c_shadow, &[(11, 3), (12, 3), (9, 5), (8, 6)]);
        }
        BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel | BlockId::GoldShovel | BlockId::DiamondShovel => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            g.px(1, 13, wood_light);
            g.px(2, 14, wood_dark);
            for i in 3..=9 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
                g.px(i, 16 - i, wood_dark);
            }
            g.pts(c_shadow, &[(10, 5), (12, 4), (11, 5), (10, 6)]);
            g.pts(c_base, &[(11, 4), (10, 4), (9, 5)]);
            g.pts(c_hi, &[(12, 3), (13, 3), (11, 3)]);
            g.pts(c_glint, &[(13, 2), (12, 2)]);
        }
        BlockId::WoodenHoe | BlockId::StoneHoe | BlockId::IronHoe | BlockId::GoldHoe | BlockId::DiamondHoe => {
            let (c_glint, c_hi, c_base, c_shadow) = get_mat_colors(block);
            for i in 2..=10 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
                g.px(i, 16 - i, wood_dark);
            }
            g.pts(c_glint, &[(9, 2), (10, 2)]);
            g.px(11, 2, c_hi);
            g.pts(c_base, &[(12, 2), (11, 3)]);
            g.pts(c_shadow, &[(12, 3), (12, 4)]);
        }
        BlockId::Shears => {
            let (iron_hi, iron_mid, iron_dark, bolt) = (
                pack_color(0.95, 0.95, 0.98),
                pack_color(0.75, 0.75, 0.78),
                pack_color(0.45, 0.45, 0.48),
                pack_color(0.30, 0.30, 0.32),
            );
            g.pts(bolt, &[(7, 8), (8, 9)]);
            g.pts(iron_hi, &[(8, 8), (3, 3), (3, 4), (4, 5), (6, 6), (11, 3), (12, 3), (10, 4), (9, 6), (6, 12), (9, 12)]);
            g.pts(iron_mid, &[(4, 3), (4, 4), (5, 5), (6, 7), (11, 4), (10, 5), (9, 7), (5, 11), (4, 12), (10, 11), (11, 12)]);
            g.pts(iron_dark, &[(7, 9), (5, 4), (12, 4), (9, 5), (6, 10), (5, 13), (6, 13), (9, 10), (10, 13), (9, 13)]);
        }
        BlockId::FishingRod => {
            for i in 2..=13 {
                g.px(i, 15 - i, wood_mid);
                g.px(i - 1, 15 - i, wood_light);
            }
            let string_col = pack_color(0.88, 0.88, 0.90);
            for y in 3..=11 {
                g.px(13, y, string_col);
            }
            g.px(13, 12, pack_color(0.90, 0.15, 0.15));
            g.px(12, 13, pack_color(0.70, 0.70, 0.75));
            g.px(13, 13, pack_color(0.95, 0.95, 0.95));
        }
        BlockId::Leather => {
            let (outline, base, hi, sh) = (
                pack_color(0.26, 0.14, 0.06),
                pack_color(0.62, 0.36, 0.16),
                pack_color(0.76, 0.48, 0.24),
                pack_color(0.44, 0.22, 0.08),
            );
            for ly in 3..=13 {
                for lx in 3..=13 {
                    let in_hide = match ly {
                        3 => (6..=10).contains(&lx),
                        4 | 7..=9 => (4..=12).contains(&lx),
                        5 | 6 => (3..=13).contains(&lx) && lx != 8,
                        10 | 11 => (3..=13).contains(&lx),
                        12 => (4..=12).contains(&lx) && lx != 8,
                        13 => (4..=6).contains(&lx) || (10..=12).contains(&lx),
                        _ => false,
                    };
                    if in_hide {
                        let col = if lx == 3 || lx == 13 || ly == 3 || ly == 13 {
                            outline
                        } else if lx <= 6 && ly <= 7 {
                            hi
                        } else if lx >= 10 || ly >= 11 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherHelmet | BlockId::IronHelmet | BlockId::GoldHelmet | BlockId::DiamondHelmet => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 3..=12 {
                for lx in 3..=12 {
                    let in_helm = match ly {
                        3 => (6..=9).contains(&lx),
                        4 => (4..=11).contains(&lx),
                        5..=8 => (3..=12).contains(&lx),
                        9..=11 => (3..=5).contains(&lx) || (10..=12).contains(&lx) || ((7..=8).contains(&lx) && ly == 9),
                        12 => (3..=5).contains(&lx) || (10..=12).contains(&lx),
                        _ => false,
                    };
                    if in_helm {
                        let is_rim = ly == 3 || (ly == 4 && (lx == 4 || lx == 11)) || lx == 3 || lx == 12 || ly == 12 || (ly == 8 && (6..=9).contains(&lx));
                        let col = if is_rim {
                            outline
                        } else if lx <= 5 && ly <= 6 {
                            if lx == 4 && ly == 4 { glint } else { hi }
                        } else if lx >= 9 || ly >= 10 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherChestplate | BlockId::IronChestplate | BlockId::GoldChestplate | BlockId::DiamondChestplate => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 2..=14 {
                for lx in 2..=13 {
                    let in_chest = match ly {
                        2 => (3..=5).contains(&lx) || (10..=12).contains(&lx),
                        3..=4 => (2..=5).contains(&lx) || (10..=13).contains(&lx),
                        5..=10 => (2..=13).contains(&lx),
                        11..=13 => (4..=11).contains(&lx),
                        14 => (5..=10).contains(&lx),
                        _ => false,
                    };
                    if in_chest {
                        let is_rim = ly == 2 || ly == 14 || lx == 2 || lx == 13 || (ly >= 11 && (lx == 4 || lx == 11)) || (ly <= 4 && (lx == 5 || lx == 10));
                        let col = if is_rim {
                            outline
                        } else if lx <= 6 && ly <= 6 {
                            if lx <= 4 && ly <= 3 { glint } else { hi }
                        } else if lx >= 9 || ly >= 10 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherLeggings | BlockId::IronLeggings | BlockId::GoldLeggings | BlockId::DiamondLeggings => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let in_legs = match ly {
                        2..=5 => (3..=12).contains(&lx),
                        6..=14 => (3..=6).contains(&lx) || (9..=12).contains(&lx),
                        _ => false,
                    };
                    if in_legs {
                        let is_rim = ly == 2 || ly == 14 || lx == 3 || lx == 12 || (ly >= 6 && (lx == 6 || lx == 9)) || (ly == 5 && (7..=8).contains(&lx));
                        let col = if is_rim {
                            outline
                        } else if lx <= 5 && ly <= 5 {
                            if lx == 4 && ly == 3 { glint } else { hi }
                        } else if lx >= 10 || ly >= 10 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::LeatherBoots | BlockId::IronBoots | BlockId::GoldBoots | BlockId::DiamondBoots => {
            let (glint, hi, base, sh, outline) = get_armor_colors(block);
            for ly in 6..=14 {
                for lx in 2..=6 {
                    if ly >= 10 || lx >= 3 {
                        let is_rim = ly == 6 || ly == 14 || lx == 2 || lx == 6 || (ly <= 9 && lx == 3);
                        let col = if is_rim {
                            outline
                        } else if lx <= 4 && ly <= 8 {
                            if lx == 3 && ly == 7 { glint } else { hi }
                        } else if lx >= 5 || ly >= 12 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
                for lx in 9..=13 {
                    if ly >= 10 || lx <= 12 {
                        let is_rim = ly == 6 || ly == 14 || lx == 9 || lx == 13 || (ly <= 9 && lx == 12);
                        let col = if is_rim {
                            outline
                        } else if lx <= 10 && ly <= 8 {
                            hi
                        } else if lx >= 12 || ly >= 12 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Bow => {
            let string = pack_color(0.92, 0.92, 0.94);
            for &(sx, sy) in &[(13, 2), (12, 2), (11, 3), (9, 4), (7, 6), (6, 7), (4, 9), (3, 11), (2, 12), (2, 13)] {
                g.px(sx, sy, wood_mid);
                g.px(sx - 1, sy, wood_light);
                g.px(sx, sy + 1, wood_dark);
            }
            for i in 2..=13 {
                g.px(i, i, string);
            }
        }
        BlockId::Arrow => {
            let (flint, flint_hi, feather, feather_sh) = (
                pack_color(0.40, 0.40, 0.44),
                pack_color(0.65, 0.65, 0.70),
                pack_color(0.95, 0.95, 0.98),
                pack_color(0.70, 0.70, 0.75),
            );
            g.pts(flint_hi, &[(14, 1), (13, 1)]);
            g.pts(flint, &[(13, 2), (14, 2), (12, 3), (13, 3)]);
            for i in 4..=11 {
                g.px(15 - i, i, wood_mid);
            }
            g.pts(feather, &[(3, 12), (2, 13), (2, 11), (1, 14)]);
            g.pts(feather_sh, &[(4, 13), (1, 12)]);
        }
        BlockId::Shield => {
            let (iron_rim, iron_hi, iron_sh, plank_light, plank_dark) = (
                pack_color(0.78, 0.80, 0.84),
                pack_color(0.95, 0.95, 0.98),
                pack_color(0.42, 0.44, 0.48),
                pack_color(0.68, 0.48, 0.28),
                pack_color(0.48, 0.32, 0.16),
            );
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let in_shield = match ly {
                        2 | 10..=11 => (4..=11).contains(&lx),
                        3..=9 => (3..=12).contains(&lx),
                        12..=13 => (5..=10).contains(&lx),
                        14 => (6..=9).contains(&lx),
                        _ => false,
                    };
                    if in_shield {
                        let is_rim = ly == 2 || ly == 14 || lx == 3 || lx == 12 || (ly >= 10 && (lx == 4 || lx == 11)) || (ly >= 12 && (lx == 5 || lx == 10));
                        let col = if is_rim {
                            if lx <= 5 || ly <= 3 {
                                iron_hi
                            } else if lx >= 10 || ly >= 12 {
                                iron_sh
                            } else {
                                iron_rim
                            }
                        } else if (7..=8).contains(&lx) && (7..=8).contains(&ly) {
                            iron_hi
                        } else if (lx + ly) % 2 == 0 {
                            plank_light
                        } else {
                            plank_dark
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        _ => return false,
    }
    true
}
