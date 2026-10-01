//! Settings menu user interface rendering, tabbed navigation, and button layouts.

use crate::core::i18n::{t, t_fmt};
use crate::core::settings::GameSettings;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Computes bounding boxes `(x, y, width, height)` for the 3 tab switch buttons (Graphics, Controls, Debug).
pub fn settings_tab_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 3] {
    let tab_w = 156;
    let tab_h = 24;
    let spacing = 6;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let tab_y = cy - 185;
    let start_x = cx - (tab_w * 3 + spacing * 2) / 2;

    [
        (start_x, tab_y, tab_w, tab_h),
        (start_x + tab_w + spacing, tab_y, tab_w, tab_h),
        (start_x + (tab_w + spacing) * 2, tab_y, tab_w, tab_h),
    ]
}

/// Computes bounding boxes `(x, y, width, height)` for the 13 options in the Graphics settings tab.
pub fn settings_graphics_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 13] {
    let opt_w = 480;
    let opt_h = 20;
    let spacing = 3;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 150;

    let mut rects = [(0, 0, 0, 0); 13];
    for i in 0..12 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 12 with extra margin
    rects[12] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 12 + 3, opt_w, opt_h);
    rects
}

/// Computes bounding boxes `(x, y, width, height)` for the 6 options in the Controls settings tab.
pub fn settings_controls_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 6] {
    let opt_w = 480;
    let opt_h = 28;
    let spacing = 10;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 130;

    let mut rects = [(0, 0, 0, 0); 6];
    for i in 0..5 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 5 with extra margin
    rects[5] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 5 + 10, opt_w, opt_h);
    rects
}

/// Computes bounding boxes `(x, y, width, height)` for the 5 options in the Debug settings tab.
pub fn settings_debug_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 5] {
    let opt_w = 480;
    let opt_h = 28;
    let spacing = 10;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 130;

    let mut rects = [(0, 0, 0, 0); 5];
    for i in 0..4 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 4
    rects[4] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 4 + 10, opt_w, opt_h);
    rects
}

/// Backward-compatible alias for 13 options.
pub fn settings_option_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 13] {
    settings_graphics_rects(width, height)
}

/// Draws the settings menu overlay with tab headers, buttons, and active options.
pub fn draw_settings_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    settings: &GameSettings,
    active_tab: usize,
    hovered_tab: Option<usize>,
    hovered_idx: Option<usize>,
    server_ip: &str,
    profiler_copied: bool,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.78);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = t("settings.title");
    let t_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 220, 3, 2, pack_color(0.95, 0.85, 0.35));

    // Render Tab Buttons (1. Graphics, 2. Controls, 3. Debug)
    let tab_rects = settings_tab_rects(width, height);
    let tab_labels = [t("settings.tab_graphics"), t("settings.tab_controls"), t("settings.tab_debug")];

    for (t_idx, (&label, &(bx, by, bw, bh))) in tab_labels.iter().zip(tab_rects.iter()).enumerate() {
        let is_active = active_tab == t_idx;
        let is_hover = hovered_tab == Some(t_idx);

        let body_color = if is_active {
            pack_color(0.30, 0.48, 0.65)
        } else if is_hover {
            pack_color(0.24, 0.32, 0.45)
        } else {
            pack_color(0.14, 0.16, 0.20)
        };

        let top_left = if is_active {
            pack_color(0.85, 0.92, 1.0)
        } else if is_hover {
            pack_color(0.60, 0.70, 0.85)
        } else {
            pack_color(0.35, 0.38, 0.45)
        };

        let bottom_right = if is_active {
            pack_color(0.18, 0.25, 0.38)
        } else {
            pack_color(0.08, 0.09, 0.12)
        };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_active {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 0.95, 0.4));
        } else if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(0.9, 0.9, 1.0));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let on_str = t("settings.on");
    let off_str = t("settings.off");

    if active_tab == 0 {
        // Tab 0: GRAPHICS
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

        let leaf_str = t(settings.leaf_quality.translation_key());

        let options = [
            t_fmt("settings.language", &[settings.language.display_name()]),
            t_fmt("settings.render_distance", &[&settings.render_distance.to_string()]),
            t_fmt("settings.leaves_quality", &[leaf_str]),
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

        let rects = settings_graphics_rects(width, height);

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
    } else if active_tab == 1 {
        // Tab 1: CONTROLS
        let sprint_str = t(settings.sprint_mode.translation_key());
        let sneak_str = t(settings.sneak_mode.translation_key());
        let jump_str = t(settings.jump_mode.translation_key());

        let options = [
            t_fmt("settings.sprint_mode", &[sprint_str]),
            t_fmt("settings.sneak_mode", &[sneak_str]),
            t_fmt("settings.jump_mode", &[jump_str]),
            format!("{}: {}", t("settings.invert_y"), if settings.invert_y { on_str } else { off_str }),
            format!("{}: {}", t("settings.invert_z"), if settings.invert_z { on_str } else { off_str }),
            t("settings.save_and_back").to_string(),
        ];

        let rects = settings_controls_rects(width, height);

        for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);

            let body_color = if is_hover {
                if i == 5 { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.32, 0.42, 0.60) }
            } else {
                if i == 5 { pack_color(0.18, 0.30, 0.22) } else { pack_color(0.20, 0.22, 0.26) }
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
    } else {
        // Tab 2: DEBUG
        let status_str = if settings.profiler_server {
            if profiler_copied {
                t_fmt("settings.profiler_server_copied", &[server_ip, &settings.profiler_server_port.to_string()])
            } else {
                t_fmt("settings.profiler_server_active", &[server_ip, &settings.profiler_server_port.to_string()])
            }
        } else {
            t("settings.profiler_server_inactive").to_string()
        };

        let options = [
            format!("{}: {}", t("settings.profiler_server"), if settings.profiler_server { on_str } else { off_str }),
            t_fmt("settings.profiler_server_status", &[&status_str]),
            t_fmt("settings.profiler_server_port", &[&settings.profiler_server_port.to_string()]),
            format!("{}: {}", t("settings.debug_overlay"), if settings.debug_overlay { on_str } else { off_str }),
            t("settings.save_and_back").to_string(),
        ];

        let rects = settings_debug_rects(width, height);

        for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);

            let body_color = if is_hover {
                if i == 4 { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.32, 0.42, 0.60) }
            } else {
                if i == 4 { pack_color(0.18, 0.30, 0.22) } else { pack_color(0.20, 0.22, 0.26) }
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

            if i == 1 && settings.profiler_server {
                draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(0.4, 0.9, 0.5));
            } else if is_hover {
                draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
            } else {
                draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
            }
        }
    }

    let hint = t("settings.controls_hint");
    let hw = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, hint, cx - hw / 2, height as i32 - 18, 1, 1, pack_color(0.7, 0.75, 0.85));
}
