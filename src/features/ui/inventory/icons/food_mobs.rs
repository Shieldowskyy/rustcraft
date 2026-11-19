//! Proceduralne ikony żywności, ryb, roślin wodnych, dropów mobów oraz jaj przyzywających.

use crate::features::blocks::BlockId;
use crate::features::ui::draw::pack_color;

use super::IconGrid;

pub(super) fn draw_food_mob_icon(g: &mut IconGrid<'_>, block: BlockId) -> bool {
    match block {
        BlockId::RawPorkchop | BlockId::CookedPorkchop => {
            let (outline, base, hi, sh, bone) = if block == BlockId::RawPorkchop {
                (pack_color(0.48, 0.20, 0.26), pack_color(0.92, 0.58, 0.64), pack_color(0.98, 0.78, 0.82), pack_color(0.75, 0.38, 0.45), pack_color(0.95, 0.95, 0.88))
            } else {
                (pack_color(0.28, 0.12, 0.05), pack_color(0.72, 0.42, 0.18), pack_color(0.90, 0.65, 0.32), pack_color(0.45, 0.22, 0.08), pack_color(0.92, 0.90, 0.82))
            };
            g.pts(bone, &[(12, 3), (13, 2), (14, 2)]);
            g.px(13, 4, outline);
            for ly in 4..=12 {
                for lx in 2..=11 {
                    if (lx + ly >= 7) && (lx + ly <= 21) && (ly - lx <= 8) && (lx - ly <= 6) {
                        let col = if lx == 2 || lx == 11 || ly == 4 || ly == 12 {
                            outline
                        } else if lx <= 5 && ly <= 7 {
                            hi
                        } else if lx >= 8 || ly >= 10 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::RawBeef | BlockId::CookedBeef => {
            let is_raw = block == BlockId::RawBeef;
            let (outline, base, hi, sh, accent) = if is_raw {
                (pack_color(0.38, 0.08, 0.08), pack_color(0.82, 0.20, 0.20), pack_color(0.94, 0.42, 0.42), pack_color(0.52, 0.10, 0.10), pack_color(0.96, 0.90, 0.88))
            } else {
                (pack_color(0.18, 0.08, 0.04), pack_color(0.50, 0.26, 0.14), pack_color(0.70, 0.42, 0.22), pack_color(0.28, 0.12, 0.06), pack_color(0.15, 0.06, 0.02))
            };
            for ly in 3..=13 {
                for lx in 2..=13 {
                    if (lx + ly >= 7) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 8) {
                        let is_acc = if is_raw {
                            (lx == 5 && ly == 7) || (lx == 6 && ly == 8) || (lx == 9 && ly == 6)
                        } else {
                            (lx + ly == 12) || (lx + ly == 16)
                        };
                        let col = if lx == 2 || lx == 13 || ly == 3 || ly == 13 {
                            outline
                        } else if is_acc {
                            accent
                        } else if lx <= 6 && ly <= 6 {
                            hi
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
        BlockId::Apple => {
            let (outline, base, hi, sh, stem, leaf) = (
                pack_color(0.32, 0.06, 0.06),
                pack_color(0.88, 0.14, 0.14),
                pack_color(0.98, 0.48, 0.48),
                pack_color(0.52, 0.08, 0.08),
                pack_color(0.42, 0.26, 0.12),
                pack_color(0.35, 0.78, 0.22),
            );
            g.pts(stem, &[(8, 2), (8, 3)]);
            g.pts(leaf, &[(9, 2), (10, 2)]);
            for ly in 4..=13 {
                for lx in 4..=12 {
                    if (lx + ly >= 9) && (lx + ly <= 23) && (ly - lx <= 8) && (lx - ly <= 7) {
                        let col = if lx == 4 || lx == 12 || ly == 4 || ly == 13 {
                            outline
                        } else if (6..=8).contains(&lx) && (6..=7).contains(&ly) {
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
        BlockId::RawChicken | BlockId::CookedChicken => {
            let (outline, base, hi, sh, bone, bone_sh) = if block == BlockId::RawChicken {
                (pack_color(0.46, 0.24, 0.20), pack_color(0.92, 0.68, 0.62), pack_color(0.98, 0.82, 0.78), pack_color(0.76, 0.46, 0.40), pack_color(0.95, 0.95, 0.88), pack_color(0.78, 0.76, 0.68))
            } else {
                (pack_color(0.30, 0.12, 0.04), pack_color(0.82, 0.46, 0.14), pack_color(0.96, 0.72, 0.28), pack_color(0.55, 0.24, 0.06), pack_color(0.92, 0.90, 0.82), pack_color(0.72, 0.70, 0.62))
            };
            g.pts(bone, &[(12, 4), (13, 3), (14, 2), (13, 2), (12, 3)]);
            g.px(14, 3, bone_sh);
            g.pts(outline, &[(11, 4), (14, 4), (15, 2), (13, 1)]);
            for ly in 4..=13 {
                for lx in 2..=11 {
                    if (lx + ly >= 8) && (lx + ly <= 21) && (ly - lx <= 7) && (lx - ly <= 4) {
                        let col = if lx == 2 || lx == 11 || ly == 4 || ly == 13 || lx + ly == 8 || lx + ly == 21 {
                            outline
                        } else if lx <= 5 && ly <= 7 {
                            hi
                        } else if lx >= 8 || ly >= 10 {
                            sh
                        } else {
                            base
                        };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Feather => {
            let (outline, white, gray, quill, quill_base) = (
                pack_color(0.25, 0.25, 0.28),
                pack_color(0.98, 0.98, 0.99),
                pack_color(0.82, 0.84, 0.88),
                pack_color(0.68, 0.62, 0.55),
                pack_color(0.50, 0.44, 0.38),
            );
            g.pts(quill_base, &[(2, 14), (3, 13)]);
            for x in 4..=13 {
                g.px(x, 16 - x, quill);
            }
            for y in 3..=11 {
                for x in 4..=13 {
                    let diag = x - (15 - y);
                    if (-2..=3).contains(&diag) {
                        let col = if diag == 3 || diag == -2 || y == 3 || x == 13 {
                            outline
                        } else if (x + y) % 2 == 0 {
                            white
                        } else {
                            gray
                        };
                        g.px(x, y, col);
                    }
                }
            }
            g.pts(outline, &[(1, 14), (2, 15), (3, 14)]);
        }
        BlockId::PigSpawnEgg
        | BlockId::CowSpawnEgg
        | BlockId::ChickenSpawnEgg
        | BlockId::SheepSpawnEgg
        | BlockId::DolphinSpawnEgg
        | BlockId::FishSpawnEgg
        | BlockId::VillagerSpawnEgg
        | BlockId::IronGolemSpawnEgg
        | BlockId::SpiderSpawnEgg => {
            let (base, spot, outline) = match block {
                BlockId::PigSpawnEgg => (pack_color(0.95, 0.62, 0.68), pack_color(0.68, 0.32, 0.38), pack_color(0.48, 0.20, 0.25)),
                BlockId::CowSpawnEgg => (pack_color(0.42, 0.30, 0.22), pack_color(0.85, 0.85, 0.85), pack_color(0.22, 0.15, 0.10)),
                BlockId::ChickenSpawnEgg => (pack_color(0.95, 0.95, 0.92), pack_color(0.85, 0.15, 0.15), pack_color(0.35, 0.35, 0.30)),
                BlockId::SheepSpawnEgg => (pack_color(0.92, 0.90, 0.88), pack_color(0.85, 0.68, 0.60), pack_color(0.40, 0.38, 0.36)),
                BlockId::DolphinSpawnEgg => (pack_color(0.22, 0.40, 0.62), pack_color(0.85, 0.88, 0.95), pack_color(0.12, 0.22, 0.35)),
                BlockId::FishSpawnEgg => (pack_color(0.68, 0.52, 0.32), pack_color(0.92, 0.45, 0.22), pack_color(0.38, 0.28, 0.16)),
                BlockId::VillagerSpawnEgg => (pack_color(0.55, 0.38, 0.24), pack_color(0.78, 0.58, 0.45), pack_color(0.32, 0.20, 0.12)),
                BlockId::IronGolemSpawnEgg => (pack_color(0.86, 0.86, 0.84), pack_color(0.45, 0.45, 0.42), pack_color(0.28, 0.28, 0.28)),
                _ => (pack_color(0.20, 0.16, 0.16), pack_color(0.80, 0.12, 0.12), pack_color(0.10, 0.08, 0.08)),
            };
            for ly in 2..=14 {
                for lx in 3..=12 {
                    let dx = (lx as f32 - 7.5).abs();
                    let dy = ly as f32;
                    let width_at_y = if dy < 6.0 {
                        (dy - 1.0) * 0.7 + 0.5
                    } else if dy <= 11.0 {
                        4.2
                    } else {
                        (14.5 - dy) * 1.1 + 0.5
                    };
                    if dx <= width_at_y {
                        if dx > width_at_y - 0.9 || ly == 2 || ly == 14 {
                            g.px(lx, ly, outline);
                        } else {
                            let is_spot = match block {
                                BlockId::PigSpawnEgg => (lx == 6 && ly == 5) || (lx == 9 && ly == 8) || (lx == 5 && ly == 10) || (lx == 8 && ly == 12),
                                BlockId::CowSpawnEgg => ((lx == 6 || lx == 7) && ly == 6) || (lx == 9 && ly == 9) || ((lx == 5 || lx == 6) && ly == 11),
                                BlockId::ChickenSpawnEgg => (lx == 7 && ly == 5) || (lx == 6 && ly == 8) || (lx == 9 && ly == 10) || (lx == 7 && ly == 12),
                                BlockId::SheepSpawnEgg => ((lx == 7 || lx == 8) && ly == 6) || (lx == 6 && ly == 9) || (lx == 9 && ly == 10) || (lx == 7 && ly == 11),
                                BlockId::DolphinSpawnEgg => (lx == 6 && ly == 5) || (lx == 8 && ly == 8) || ((lx == 6 || lx == 9) && ly == 11),
                                BlockId::FishSpawnEgg => (lx == 7 && ly == 5) || (lx == 8 && ly == 7) || (lx == 6 && ly == 9) || (lx == 8 && ly == 12),
                                BlockId::VillagerSpawnEgg => (lx == 6 && ly == 6) || (lx == 8 && ly == 8) || ((lx == 6 || lx == 9) && ly == 11),
                                BlockId::IronGolemSpawnEgg => (lx == 7 && ly == 5) || (lx == 8 && ly == 8) || (lx == 6 && ly == 10) || (lx == 9 && ly == 12),
                                _ => (lx == 6 && ly == 5) || (lx == 8 && ly == 8) || (lx == 5 && ly == 10) || (lx == 9 && ly == 11),
                            };
                            g.px(lx, ly, if is_spot { spot } else { base });
                        }
                    }
                }
            }
        }
        BlockId::StringItem => {
            let (silk, silk_dim) = (pack_color(0.95, 0.95, 0.95), pack_color(0.75, 0.75, 0.78));
            for &(lx, ly) in &[
                (3, 12), (4, 11), (5, 10), (6, 9), (7, 8), (8, 7), (9, 6), (10, 5), (11, 4), (12, 3),
                (5, 12), (6, 11), (7, 10), (8, 9), (9, 8), (10, 7), (11, 6), (7, 6), (8, 5),
            ] {
                g.px(lx, ly, silk);
                g.px(lx, ly + 1, silk_dim);
            }
        }
        BlockId::SpiderEye => {
            let (red_bright, red_dark, highlight, pupil, outline) = (
                pack_color(0.92, 0.12, 0.12),
                pack_color(0.55, 0.06, 0.06),
                pack_color(1.00, 0.70, 0.70),
                pack_color(0.12, 0.02, 0.02),
                pack_color(0.28, 0.04, 0.04),
            );
            for ly in 3..=12 {
                for lx in 3..=12 {
                    let d2 = (lx as f32 - 7.5).powi(2) + (ly as f32 - 7.5).powi(2);
                    if d2 <= 18.0 {
                        g.px(lx, ly, if d2 > 13.0 { outline } else if d2 <= 6.0 { red_bright } else { red_dark });
                    }
                }
            }
            g.px(6, 6, highlight);
            g.pts(pupil, &[(7, 7), (8, 7), (7, 8), (8, 8)]);
        }
        BlockId::RawCod | BlockId::CookedCod => {
            let is_cooked = block == BlockId::CookedCod;
            let (outline, base, belly, fin, eye) = if is_cooked {
                (pack_color(0.35, 0.25, 0.12), pack_color(0.78, 0.60, 0.36), pack_color(0.92, 0.78, 0.52), pack_color(0.60, 0.42, 0.22), pack_color(0.12, 0.12, 0.12))
            } else {
                (pack_color(0.30, 0.28, 0.24), pack_color(0.68, 0.62, 0.54), pack_color(0.85, 0.82, 0.76), pack_color(0.52, 0.46, 0.38), pack_color(0.12, 0.12, 0.12))
            };
            g.pts(fin, &[(2, 6), (2, 8)]);
            g.pts(outline, &[(2, 7), (3, 7), (13, 7)]);
            for lx in 4..=11 {
                for ly in 6..=8 {
                    g.px(lx, ly, if lx >= 10 && ly == 7 { eye } else if ly == 8 { belly } else if ly == 6 { fin } else { base });
                }
            }
            for lx in 3..=12 {
                g.pts(outline, &[(lx, 5), (lx, 9)]);
            }
        }
        BlockId::RawSalmon | BlockId::CookedSalmon => {
            let is_cooked = block == BlockId::CookedSalmon;
            let (outline, base, belly, fin, eye) = if is_cooked {
                (pack_color(0.38, 0.15, 0.08), pack_color(0.85, 0.45, 0.22), pack_color(0.95, 0.65, 0.40), pack_color(0.65, 0.30, 0.12), pack_color(0.12, 0.12, 0.12))
            } else {
                (pack_color(0.38, 0.18, 0.16), pack_color(0.82, 0.32, 0.28), pack_color(0.92, 0.55, 0.48), pack_color(0.55, 0.20, 0.18), pack_color(0.12, 0.12, 0.12))
            };
            g.pts(fin, &[(2, 5), (2, 7)]);
            g.pts(outline, &[(2, 6), (3, 6), (14, 6), (14, 7)]);
            for lx in 4..=12 {
                for ly in 5..=8 {
                    g.px(lx, ly, if lx >= 11 && ly == 6 { eye } else if ly >= 7 { belly } else if ly == 5 { fin } else { base });
                }
            }
            for lx in 3..=13 {
                g.pts(outline, &[(lx, 4), (lx, 9)]);
            }
        }
        BlockId::Pufferfish => {
            let (outline, base, belly, spike, eye) = (
                pack_color(0.40, 0.32, 0.08),
                pack_color(0.92, 0.78, 0.22),
                pack_color(0.96, 0.92, 0.65),
                pack_color(0.72, 0.58, 0.15),
                pack_color(0.15, 0.25, 0.45),
            );
            g.pts(spike, &[(4, 3), (8, 2), (11, 3), (2, 8), (13, 8), (4, 12), (8, 13), (11, 12)]);
            for ly in 4..=11 {
                for lx in 4..=11 {
                    let d2 = (lx - 7) * (lx - 7) + (ly - 7) * (ly - 7);
                    if d2 <= 16 {
                        g.px(lx, ly, if d2 >= 12 { outline } else if lx == 9 && ly == 6 { eye } else if ly >= 8 { belly } else { base });
                    }
                }
            }
        }
        BlockId::TropicalFish => {
            let (outline, orange, white, fin, eye) = (
                pack_color(0.45, 0.18, 0.05),
                pack_color(0.95, 0.48, 0.12),
                pack_color(0.96, 0.96, 0.98),
                pack_color(0.18, 0.72, 0.85),
                pack_color(0.10, 0.10, 0.12),
            );
            g.pts(fin, &[(2, 6), (2, 7), (2, 8), (7, 5), (8, 5)]);
            g.pts(outline, &[(3, 7), (13, 7)]);
            for lx in 4..=12 {
                for ly in 6..=8 {
                    let col = if lx == 11 && ly == 6 { eye } else if lx == 6 || lx == 7 || lx == 10 { white } else { orange };
                    g.px(lx, ly, col);
                }
            }
            for lx in 3..=12 {
                g.pts(outline, &[(lx, 5), (lx, 9)]);
            }
        }
        BlockId::DriedKelp => {
            let (dark, mid, hi, tie) = (pack_color(0.12, 0.22, 0.10), pack_color(0.20, 0.38, 0.16), pack_color(0.28, 0.52, 0.22), pack_color(0.68, 0.55, 0.30));
            for ly in 4..=11 {
                for lx in 4..=11 {
                    let col = if ly == 7 || ly == 8 { tie } else if lx == 4 || lx == 11 || ly == 4 || ly == 11 { dark } else if (lx + ly) % 2 == 0 { hi } else { mid };
                    g.px(lx, ly, col);
                }
            }
        }
        BlockId::Kelp => {
            let (stalk, leaf, hi) = (pack_color(0.16, 0.32, 0.14), pack_color(0.24, 0.54, 0.20), pack_color(0.35, 0.70, 0.28));
            for y in 2..=14 {
                g.pts(stalk, &[(7, y), (8, y)]);
                if y % 3 == 0 {
                    g.px(5, y, leaf);
                    g.px(6, y, hi);
                } else if y % 3 == 1 {
                    g.px(9, y, hi);
                    g.px(10, y, leaf);
                }
            }
        }
        BlockId::Seagrass => {
            let (leaf1, leaf2, dark) = (pack_color(0.22, 0.58, 0.24), pack_color(0.30, 0.68, 0.32), pack_color(0.15, 0.40, 0.18));
            for y in 5..=14 {
                let x1 = 5 + ((14 - y) / 3);
                g.px(x1, y, leaf1);
                g.px(x1 + 1, y, dark);
            }
            for y in 3..=14 {
                let x2 = 9 + ((14 - y) % 2);
                g.px(x2, y, leaf2);
                g.px(x2 + 1, y, dark);
            }
        }
        _ => return false,
    }
    true
}
