//! Settings menu user interface rendering and button layout.

use crate::core::i18n::{t, t_fmt};
use crate::core::settings::GameSettings;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Computes bounding boxes `(x, y, width, height)` for each of the 13 settings menu options.
pub fn settings_option_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 13] {
    let opt_w = 460;
    let opt_h = 20;
    let spacing = 3;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 170;

    [
        (cx - opt_w / 2, start_y, opt_w, opt_h),
        (cx - opt_w / 2, start_y + opt_h + spacing, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 2, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 3, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 4, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 5, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 6, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 7, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 8, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 9, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 10, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 11, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 12 + 3, opt_w, opt_h),
    ]
}

/// Draws the graphics & video settings menu overlay with buttons and current values.
pub fn draw_settings_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    settings: &GameSettings,
    hovered_idx: Option<usize>,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.75);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = t("settings.title");
    let t_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 198, 3, 2, pack_color(0.95, 0.85, 0.35));

    let msaa_str = match settings.msaa {
        2 => "2X",
        4 => "4X",
        _ => t("settings.off"),
    };

    let fps_str = if settings.fps_limit == 0 {
        t("settings.fps_unlimited").to_string()
    } else {
        format!("{} FPS", settings.fps_limit)
    };

    let on_str = t("settings.on");
    let off_str = t("settings.off");

    let options = [
        t_fmt("settings.language", &[settings.language.display_name()]),
        t_fmt("settings.render_distance", &[&settings.render_distance.to_string()]),
        format!("{}: {}", t("settings.leaves"), if settings.fancy_leaves { t("settings.leaves_fancy") } else { t("settings.leaves_fast") }),
        format!("{}: {}", t("settings.fog"), if settings.fog { on_str } else { off_str }),
        format!("{}: {}", t("settings.clouds"), if settings.clouds { on_str } else { off_str }),
        format!("{}: {}", t("settings.particles"), if settings.particles { on_str } else { off_str }),
        format!("{}: {}", t("settings.chunk_anim"), if settings.chunk_anim { on_str } else { off_str }),
        format!("{}: < {:.0} DEG >", t("settings.fov"), settings.fov_deg),
        t_fmt("settings.msaa", &[msaa_str]),
        t_fmt("settings.fps_limit", &[&fps_str]),
        format!("{}: {}", t("settings.underwater_fx"), if settings.underwater_fx { on_str } else { off_str }),
        format!("{}: {}", t("settings.debug_overlay"), if settings.debug_overlay { on_str } else { off_str }),
        t("settings.save_and_back").to_string(),
    ];

    let rects = settings_option_rects(width, height);

    for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);

        let body_color = if is_hover {
            if i == 12 { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.32, 0.42, 0.60) }
        } else {
            if i == 12 { pack_color(0.18, 0.30, 0.22) } else { pack_color(0.20, 0.22, 0.26) }
        };

        let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
        let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let hint = t("settings.controls_hint");
    let hw = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, hint, cx - hw / 2, height as i32 - 18, 1, 1, pack_color(0.7, 0.75, 0.85));
}
