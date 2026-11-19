//! Proceduralne ikony minerałów, mechanizmów RustStone, płyt, schodów i domyślnych bloków.

use crate::features::blocks::{BlockId, Face};
use crate::features::ui::draw::{fill_rect, pack_color};

use super::IconGrid;

pub(super) fn draw_block_or_misc_icon(g: &mut IconGrid<'_>, block: BlockId) {
    let wood_dark = pack_color(0.30, 0.18, 0.08);

    match block {
        BlockId::CraftingTable => {
            let (top_light, top_dark, side_frame, side_plank, iron_tool) = (
                pack_color(0.78, 0.62, 0.38),
                pack_color(0.58, 0.42, 0.22),
                pack_color(0.32, 0.20, 0.10),
                pack_color(0.64, 0.48, 0.28),
                pack_color(0.78, 0.82, 0.88),
            );
            for ly in 1..=6 {
                for lx in 2..=13 {
                    if (lx + ly >= 6) && (lx - ly <= 8) && (ly - lx <= 3) && (lx + ly <= 16) {
                        g.px(lx, ly, if ((lx / 2) + (ly / 2)) % 2 == 0 { top_light } else { top_dark });
                    }
                }
            }
            for ly in 6..=14 {
                for lx in 2..=13 {
                    let is_border = lx == 2 || lx == 7 || lx == 8 || lx == 13 || ly == 14;
                    g.px(lx, ly, if is_border { side_frame } else { side_plank });
                }
            }
            g.pts(iron_tool, &[(4, 8), (4, 9), (5, 9), (10, 8), (11, 8)]);
            g.px(10, 9, wood_dark);
        }
        BlockId::Door => {
            let (frame, glass, knob, panel) = (
                pack_color(0.52, 0.36, 0.18),
                pack_color(0.65, 0.82, 0.90),
                pack_color(0.85, 0.75, 0.30),
                pack_color(0.42, 0.28, 0.14),
            );
            for ly in 1..=14 {
                for lx in 4..=11 {
                    g.px(lx, ly, frame);
                }
            }
            for ly in 3..=6 {
                g.pts(glass, &[(5, ly), (6, ly), (9, ly), (10, ly)]);
            }
            g.px(10, 8, knob);
            for ly in 9..=12 {
                g.pts(panel, &[(5, ly), (6, ly), (9, ly), (10, ly)]);
            }
        }
        BlockId::Tnt => {
            let (red, white, dark, fuse) = (
                pack_color(0.85, 0.15, 0.10),
                pack_color(0.92, 0.92, 0.92),
                pack_color(0.25, 0.05, 0.05),
                pack_color(0.40, 0.35, 0.30),
            );
            for ly in 3..=13 {
                for lx in 3..=12 {
                    g.px(lx, ly, if (7..=9).contains(&ly) { white } else { red });
                }
            }
            g.pts(dark, &[(5, 8), (7, 8), (8, 8), (10, 8)]);
            g.pts(fuse, &[(7, 1), (7, 2)]);
        }
        BlockId::Emerald | BlockId::Diamond => {
            let (outline, glint, hi, base, sh) = if block == BlockId::Emerald {
                (pack_color(0.04, 0.28, 0.08), pack_color(0.75, 1.00, 0.75), pack_color(0.18, 0.85, 0.32), pack_color(0.10, 0.68, 0.22), pack_color(0.06, 0.45, 0.14))
            } else {
                (pack_color(0.05, 0.35, 0.38), pack_color(0.88, 1.00, 1.00), pack_color(0.40, 0.92, 0.94), pack_color(0.20, 0.75, 0.80), pack_color(0.10, 0.52, 0.58))
            };
            for ly in 3..=13 {
                for lx in 3..=12 {
                    let in_shape = match ly {
                        3 | 10 | 11 => (6..=9).contains(&lx),
                        4 | 9 => (5..=10).contains(&lx),
                        5 | 8 => (4..=11).contains(&lx),
                        6..=7 => (3..=12).contains(&lx),
                        12 | 13 => lx == 7 || lx == 8,
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || ((ly == 6 || ly == 7) && (lx == 3 || lx == 12)) || (ly == 12 && (lx == 7 || lx == 8));
                        let col = if is_border {
                            outline
                        } else if ly <= 5 && (6..=8).contains(&lx) {
                            glint
                        } else if lx <= 6 {
                            hi
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
        BlockId::ClayBall => {
            let (outline, hi, base, sh) = (pack_color(0.32, 0.34, 0.38), pack_color(0.82, 0.85, 0.90), pack_color(0.66, 0.70, 0.76), pack_color(0.50, 0.53, 0.58));
            for ly in 4..=12 {
                for lx in 4..=12 {
                    let d2 = (lx as f32 - 8.0).powi(2) + (ly as f32 - 8.0).powi(2);
                    if d2 <= 18.0 {
                        let col = if d2 > 12.0 { outline } else if lx <= 6 && ly <= 6 { hi } else if lx >= 10 || ly >= 10 { sh } else { base };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Brick => {
            let (outline, hi, base, sh) = (pack_color(0.38, 0.16, 0.10), pack_color(0.82, 0.45, 0.35), pack_color(0.68, 0.32, 0.24), pack_color(0.50, 0.22, 0.16));
            for ly in 5..=11 {
                for lx in 3..=13 {
                    let col = if ly == 5 || ly == 11 || lx == 3 || lx == 13 { outline } else if ly == 6 || lx == 4 { hi } else if ly == 10 || lx == 12 { sh } else { base };
                    g.px(lx, ly, col);
                }
            }
        }
        BlockId::Flint => {
            let (outline, hi, base, sh) = (pack_color(0.10, 0.10, 0.12), pack_color(0.45, 0.45, 0.48), pack_color(0.28, 0.28, 0.30), pack_color(0.18, 0.18, 0.20));
            for ly in 3..=13 {
                for lx in 4..=12 {
                    let in_shape = match ly {
                        3..=4 => (7..=9).contains(&lx),
                        5..=7 => (5..=10).contains(&lx),
                        8..=10 => (4..=12).contains(&lx),
                        11..=12 => (5..=11).contains(&lx),
                        13 => (6..=9).contains(&lx),
                        _ => false,
                    };
                    if in_shape {
                        let is_border = ly == 3 || ly == 13 || (ly == 5 && (lx == 5 || lx == 10)) || (ly == 8 && (lx == 4 || lx == 12));
                        let col = if is_border { outline } else if lx <= 6 && ly <= 7 { hi } else if lx >= 10 || ly >= 11 { sh } else { base };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Bed => {
            let (wood, white, red, red_sh, frame) = (
                pack_color(0.45, 0.28, 0.12),
                pack_color(0.92, 0.92, 0.94),
                pack_color(0.80, 0.15, 0.15),
                pack_color(0.55, 0.10, 0.10),
                pack_color(0.30, 0.18, 0.08),
            );
            g.pts(frame, &[(2, 13), (2, 14), (13, 13), (13, 14), (1, 6), (1, 7), (1, 8), (1, 9), (1, 10), (1, 11)]);
            for lx in 2..=13 {
                g.px(lx, 11, wood);
                g.px(lx, 12, frame);
            }
            for ly in 8..=10 {
                for lx in 2..=5 {
                    g.px(lx, ly, white);
                }
                for lx in 6..=13 {
                    g.px(lx, ly, if ly == 10 { red_sh } else { red });
                }
            }
        }
        BlockId::Ladder => {
            let (wood_rail, rail_hi, rung, rung_sh) = (
                pack_color(0.48, 0.32, 0.16),
                pack_color(0.65, 0.45, 0.24),
                pack_color(0.58, 0.40, 0.20),
                pack_color(0.35, 0.22, 0.10),
            );
            for ly in 1..=14 {
                g.pts(rail_hi, &[(3, ly), (11, ly)]);
                g.pts(wood_rail, &[(4, ly), (12, ly)]);
            }
            for &ry in &[3, 6, 9, 12] {
                for lx in 5..=10 {
                    g.px(lx, ry, rung);
                    g.px(lx, ry + 1, rung_sh);
                }
            }
        }
        BlockId::Torch => {
            let (wood_light, wood_mid, wood_dark, coal, flame_core, flame_gold, flame_orange) = (
                pack_color(0.68, 0.48, 0.26),
                pack_color(0.52, 0.35, 0.18),
                pack_color(0.35, 0.22, 0.10),
                pack_color(0.20, 0.18, 0.16),
                pack_color(1.00, 1.00, 0.88),
                pack_color(1.00, 0.80, 0.15),
                pack_color(0.98, 0.42, 0.05),
            );
            g.pts(flame_orange, &[(7, 1), (6, 2), (9, 3), (7, 4), (8, 4)]);
            g.pts(flame_gold, &[(8, 1), (9, 2), (6, 3), (8, 3)]);
            g.pts(flame_core, &[(7, 2), (8, 2), (7, 3)]);
            g.pts(coal, &[(7, 5), (8, 5), (7, 6)]);
            g.px(8, 6, wood_dark);
            for ly in 7..=14 {
                g.px(7, ly, if ly % 3 == 0 { wood_light } else { wood_mid });
                g.px(8, ly, wood_dark);
            }
        }
        BlockId::RustStone | BlockId::RustStoneWireLit => {
            let (dark, base, hi, ember) = (pack_color(0.45, 0.12, 0.06), pack_color(0.82, 0.26, 0.12), pack_color(1.00, 0.48, 0.18), pack_color(1.00, 0.82, 0.40));
            for ly in 4..=12 {
                for lx in 4..=12 {
                    let dist_sq = (lx as f32 - 7.5).powi(2) + (ly as f32 - 7.5).powi(2);
                    if dist_sq <= 15.0 {
                        let col = if dist_sq < 3.0 { ember } else if (lx + ly) % 3 == 0 { hi } else if (lx + ly) % 2 == 0 { base } else { dark };
                        g.px(lx, ly, col);
                    }
                }
            }
        }
        BlockId::Lever | BlockId::LeverOn => {
            let (stone_base, stone_sh, wood_shaft, tip) = (pack_color(0.55, 0.55, 0.58), pack_color(0.35, 0.35, 0.38), pack_color(0.55, 0.35, 0.15), pack_color(0.80, 0.25, 0.15));
            for ly in 11..=14 {
                for lx in 4..=11 {
                    g.px(lx, ly, if ly == 14 { stone_sh } else { stone_base });
                }
            }
            let is_on = block == BlockId::LeverOn;
            for i in 0..7 {
                g.px(if is_on { 9 - i / 2 } else { 6 + i / 2 }, 10 - i, if i >= 5 { tip } else { wood_shaft });
            }
        }
        BlockId::Button | BlockId::ButtonPressed => {
            let (hi, base, sh) = (pack_color(0.75, 0.75, 0.78), pack_color(0.58, 0.58, 0.60), pack_color(0.38, 0.38, 0.40));
            for ly in 6..=10 {
                for lx in 4..=11 {
                    g.px(lx, ly, if lx == 4 || ly == 6 { hi } else if lx == 11 || ly == 10 { sh } else { base });
                }
            }
        }
        BlockId::PressurePlate | BlockId::PressurePlatePressed => {
            let (border, base, hi) = (pack_color(0.40, 0.40, 0.42), pack_color(0.60, 0.60, 0.62), pack_color(0.75, 0.75, 0.78));
            for ly in 10..=13 {
                for lx in 2..=13 {
                    g.px(lx, ly, if ly == 10 { hi } else if ly == 13 || lx == 2 || lx == 13 { border } else { base });
                }
            }
        }
        BlockId::WoodenSlab | BlockId::StoneSlab => {
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);
            let half_h = g.size / 2;
            let start_y = g.y + half_h;
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, start_y, g.size, half_h, packed);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, start_y, g.size, 2, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, start_y, 2, half_h, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, start_y + half_h - 2, g.size, 2, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 2, start_y, 2, half_h, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, start_y + half_h - 1, g.size, 1, dark_border);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 1, start_y, 1, half_h, dark_border);
        }
        BlockId::WoodenStairs | BlockId::StoneStairs => {
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);
            let half = g.size / 2;
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y + half, g.size, half, packed);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, half, half, packed);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, half, 2, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, 2, g.size, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + half, g.y + half, half, 2, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + half - 2, g.y, 2, half, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 2, g.y + half, 2, half, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y + g.size - 1, g.size, 1, dark_border);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 1, g.y + half, 1, half, dark_border);
        }
        _ => {
            let color = block.face_color(Face::Top);
            let packed = pack_color(color[0], color[1], color[2]);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, g.size, g.size, packed);
            let top_highlight = pack_color((color[0] * 1.25).min(1.0), (color[1] * 1.25).min(1.0), (color[2] * 1.25).min(1.0));
            let shadow = pack_color(color[0] * 0.65, color[1] * 0.65, color[2] * 0.65);
            let dark_border = pack_color(color[0] * 0.40, color[1] * 0.40, color[2] * 0.40);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, g.size, 2, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y, 2, g.size, top_highlight);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y + g.size - 2, g.size, 2, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 2, g.y, 2, g.size, shadow);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x, g.y + g.size - 1, g.size, 1, dark_border);
            fill_rect(g.buffer, g.buf_w, g.buf_h, g.x + g.size - 1, g.y, 1, g.size, dark_border);
        }
    }
}
