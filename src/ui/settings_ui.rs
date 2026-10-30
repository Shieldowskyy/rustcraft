//! Settings menu user interface rendering, tabbed navigation, and button layouts.

use crate::core::i18n::{t, t_fmt};
use crate::core::settings::GameSettings;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Computes bounding boxes `(x, y, width, height)` for the 4 tab switch buttons (Graphics, Audio, Controls, Debug).
pub fn settings_tab_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let tab_w = 116;
    let tab_h = 24;
    let spacing = 6;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let tab_y = cy - 185;
    let start_x = cx - (tab_w * 4 + spacing * 3) / 2;

    [
        (start_x, tab_y, tab_w, tab_h),
        (start_x + tab_w + spacing, tab_y, tab_w, tab_h),
        (start_x + (tab_w + spacing) * 2, tab_y, tab_w, tab_h),
        (start_x + (tab_w + spacing) * 3, tab_y, tab_w, tab_h),
    ]
}

/// Computes bounding boxes `(x, y, width, height)` for the 8 options in the Audio settings tab.
pub fn settings_audio_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 8] {
    let opt_w = 480;
    let opt_h = 24;
    let spacing = 7;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 135;

    let mut rects = [(0, 0, 0, 0); 8];
    for i in 0..7 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 7 with extra margin
    rects[7] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 7 + 6, opt_w, opt_h);
    rects
}

/// Computes bounding boxes `(x, y, width, height)` for the 14 options in the Graphics settings tab.
pub fn settings_graphics_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 14] {
    let opt_w = 480;
    let opt_h = 19;
    let spacing = 2;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 156;

    let mut rects = [(0, 0, 0, 0); 14];
    for i in 0..13 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 13 with extra margin
    rects[13] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 13 + 5, opt_w, opt_h);
    rects
}

/// Computes bounding boxes `(x, y, width, height)` for the 11 options in the Controls settings tab.
pub fn settings_controls_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 11] {
    let opt_w = 480;
    let opt_h = 20;
    let spacing = 5;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 146;

    let mut rects = [(0, 0, 0, 0); 11];
    for i in 0..10 {
        rects[i] = (cx - opt_w / 2, start_y + (opt_h + spacing) * i as i32, opt_w, opt_h);
    }
    // Save & Back button at index 10 with extra margin
    rects[10] = (cx - opt_w / 2, start_y + (opt_h + spacing) * 10 + 6, opt_w, opt_h);
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

/// Backward-compatible alias for 14 options.
pub fn settings_option_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 14] {
    settings_graphics_rects(width, height)
}

fn draw_option_button(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    label: &str,
    bx: i32,
    by: i32,
    bw: i32,
    bh: i32,
    is_hover: bool,
    is_action_btn: bool,
    mouse_pos: Option<(f32, f32)>,
) {
    let body_color = if is_hover {
        if is_action_btn { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.32, 0.42, 0.60) }
    } else {
        if is_action_btn { pack_color(0.18, 0.30, 0.22) } else { pack_color(0.20, 0.22, 0.26) }
    };

    let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
    let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

    fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
    fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
    fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
    fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
    fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

    let scale = 2;
    let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

    let has_arrows = label.contains('<') && label.contains('>');

    if has_arrows {
        let arrow_w = 28;
        let l_bx = bx + 4;
        let l_by = by + 2;
        let l_bw = arrow_w;
        let l_bh = bh - 4;

        let r_bx = bx + bw - arrow_w - 4;
        let r_by = by + 2;
        let r_bw = arrow_w;
        let r_bh = bh - 4;

        let (is_left_hover, is_right_hover) = if let Some((mx, my)) = mouse_pos {
            let px = mx as i32;
            let py = my as i32;
            if py >= by && py < by + bh && px >= bx && px < bx + bw {
                (px < bx + arrow_w + 12 || px < bx + bw / 3, px >= bx + bw - arrow_w - 12 || px >= bx + bw * 2 / 3)
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };

        // Left Arrow Button [ < ]
        let l_bg = if is_left_hover {
            pack_color(0.38, 0.52, 0.78)
        } else if is_hover {
            pack_color(0.26, 0.34, 0.48)
        } else {
            pack_color(0.16, 0.18, 0.22)
        };
        let l_border = if is_left_hover {
            pack_color(1.0, 0.95, 0.3)
        } else if is_hover {
            pack_color(0.65, 0.75, 0.92)
        } else {
            pack_color(0.32, 0.35, 0.42)
        };
        fill_rect(buffer, width, height, l_bx, l_by, l_bw, l_bh, l_bg);
        fill_rect(buffer, width, height, l_bx, l_by, l_bw, 1, l_border);
        fill_rect(buffer, width, height, l_bx, l_by + l_bh - 1, l_bw, 1, l_border);
        fill_rect(buffer, width, height, l_bx, l_by, 1, l_bh, l_border);
        fill_rect(buffer, width, height, l_bx + l_bw - 1, l_by, 1, l_bh, l_border);

        let l_col = if is_left_hover { pack_color(1.0, 1.0, 0.3) } else { pack_color(0.85, 0.90, 0.98) };
        let l_tw = font::measure_text("<", scale, 1);
        draw_text_colored_shadowed(buffer, width, height, "<", l_bx + (l_bw - l_tw) / 2, ty, scale, 1, l_col);

        // Right Arrow Button [ > ]
        let r_bg = if is_right_hover {
            pack_color(0.38, 0.52, 0.78)
        } else if is_hover {
            pack_color(0.26, 0.34, 0.48)
        } else {
            pack_color(0.16, 0.18, 0.22)
        };
        let r_border = if is_right_hover {
            pack_color(1.0, 0.95, 0.3)
        } else if is_hover {
            pack_color(0.65, 0.75, 0.92)
        } else {
            pack_color(0.32, 0.35, 0.42)
        };
        fill_rect(buffer, width, height, r_bx, r_by, r_bw, r_bh, r_bg);
        fill_rect(buffer, width, height, r_bx, r_by, r_bw, 1, r_border);
        fill_rect(buffer, width, height, r_bx, r_by + r_bh - 1, r_bw, 1, r_border);
        fill_rect(buffer, width, height, r_bx, r_by, 1, r_bh, r_border);
        fill_rect(buffer, width, height, r_bx + r_bw - 1, r_by, 1, r_bh, r_border);

        let r_col = if is_right_hover { pack_color(1.0, 1.0, 0.3) } else { pack_color(0.85, 0.90, 0.98) };
        let r_tw = font::measure_text(">", scale, 1);
        draw_text_colored_shadowed(buffer, width, height, ">", r_bx + (r_bw - r_tw) / 2, ty, scale, 1, r_col);

        // Clean label in the center
        let display_label = label.replace("< ", "").replace(" >", "").replace('<', "").replace('>', "");
        let tw = font::measure_text(&display_label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let text_color = if is_hover {
            pack_color(1.0, 1.0, 0.3)
        } else {
            pack_color(0.90, 0.92, 0.95)
        };
        draw_text_colored_shadowed(buffer, width, height, &display_label, tx, ty, scale, 1, text_color);
    } else {
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }
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
    mouse_pos: Option<(f32, f32)>,
    is_gamepad_active: bool,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.78);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = t("settings.title");
    let t_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 220, 3, 2, pack_color(0.95, 0.85, 0.35));

    // Render Tab Buttons (1. Graphics, 2. Audio, 3. Controls, 4. Debug)
    let tab_rects = settings_tab_rects(width, height);
    let tab_labels = [
        t("settings.tab_graphics"),
        t("settings.tab_audio"),
        t("settings.tab_controls"),
        t("settings.tab_debug"),
    ];

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

        let bright_pct = (settings.brightness * 100.0).round() as i32;
        let bright_str = if bright_pct == 0 {
            t("settings.brightness_moody").to_string()
        } else if bright_pct == 100 {
            t("settings.brightness_bright").to_string()
        } else {
            format!("+{}%", bright_pct)
        };

        let options = [
            t_fmt("settings.language", &[settings.language.display_name()]),
            t_fmt("settings.render_distance", &[&settings.render_distance.to_string()]),
            t_fmt("settings.leaves_quality", &[leaf_str]),
            format!("{}: {}", t("settings.fog"), if settings.fog { on_str } else { off_str }),
            format!("{}: {}", t("settings.clouds"), if settings.clouds { on_str } else { off_str }),
            format!("{}: {}", t("settings.particles"), if settings.particles { on_str } else { off_str }),
            format!("{}: {}", t("settings.chunk_anim"), if settings.chunk_anim { on_str } else { off_str }),
            format!("{}: < {:.0} DEG >", t("settings.fov"), settings.fov_deg),
            t_fmt("settings.brightness", &[&bright_str]),
            t_fmt("settings.msaa", &[msaa_str]),
            t_fmt("settings.fps_limit", &[&fps_str]),
            format!("{}: {}", t("settings.underwater_fx"), if settings.underwater_fx { on_str } else { off_str }),
            t_fmt("settings.force_unicode_font", &[if settings.force_unicode_font { on_str } else { off_str }]),
            t("settings.save_and_back").to_string(),
        ];

        let rects = settings_graphics_rects(width, height);
        for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);
            draw_option_button(buffer, width, height, label, bx, by, bw, bh, is_hover, i == 13, mouse_pos);
        }
    } else if active_tab == 1 {
        // Tab 1: AUDIO
        let master_pct = (settings.master_volume * 100.0).round() as i32;
        let sfx_pct = (settings.sfx_volume * 100.0).round() as i32;
        let music_pct = (settings.music_volume * 100.0).round() as i32;
        let ambient_pct = (settings.ambient_volume * 100.0).round() as i32;
        let ui_pct = (settings.ui_volume * 100.0).round() as i32;

        let options = [
            t_fmt("settings.master_volume", &[&master_pct.to_string()]),
            t_fmt("settings.sfx_volume", &[&sfx_pct.to_string()]),
            t_fmt("settings.music_volume", &[&music_pct.to_string()]),
            t_fmt("settings.ambient_volume", &[&ambient_pct.to_string()]),
            t_fmt("settings.ui_volume", &[&ui_pct.to_string()]),
            t_fmt("settings.mute_audio", &[if settings.mute_audio { on_str } else { off_str }]),
            t_fmt("settings.music_toast", &[if settings.show_music_toast { on_str } else { off_str }]),
            t("settings.save_and_back").to_string(),
        ];

        let rects = settings_audio_rects(width, height);
        for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);
            draw_option_button(buffer, width, height, label, bx, by, bw, bh, is_hover, i == 7, mouse_pos);
        }
    } else if active_tab == 2 {
        // Tab 2: CONTROLS
        let sprint_str = t(settings.sprint_mode.translation_key());
        let sneak_str = t(settings.sneak_mode.translation_key());
        let jump_str = t(settings.jump_mode.translation_key());
        let mouse_sens_pct = (settings.mouse_sensitivity * 100.0).round() as i32;
        let gp_sens_pct = (settings.gamepad_sensitivity * 100.0).round() as i32;

        let is_pc = !cfg!(any(target_os = "vita", target_os = "xbox360", target_os = "xenon", target_os = "android"));
        let rebind_label = if is_pc {
            t("settings.rebind_controls").to_string()
        } else {
            t("settings.rebind_controls_pc_only").to_string()
        };

        let options = [
            t_fmt("settings.sprint_mode", &[sprint_str]),
            t_fmt("settings.sneak_mode", &[sneak_str]),
            t_fmt("settings.jump_mode", &[jump_str]),
            format!("{}: {}", t("settings.invert_y"), if settings.invert_y { on_str } else { off_str }),
            format!("{}: {}", t("settings.invert_z"), if settings.invert_z { on_str } else { off_str }),
            t_fmt("settings.mouse_sensitivity", &[&mouse_sens_pct.to_string()]),
            t_fmt("settings.gamepad_sensitivity", &[&gp_sens_pct.to_string()]),
            format!("{}: {}", t("settings.vibration"), if settings.vibration { on_str } else { off_str }),
            format!("{}: {}", t("settings.damage_tilt"), if settings.damage_tilt { on_str } else { off_str }),
            rebind_label,
            t("settings.save_and_back").to_string(),
        ];

        let rects = settings_controls_rects(width, height);
        for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);
            let is_action_btn = i == 9 || i == 10;
            draw_option_button(buffer, width, height, label, bx, by, bw, bh, is_hover, is_action_btn, mouse_pos);
        }
    } else {
        // Tab 3: DEBUG
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
            if i == 1 && settings.profiler_server {
                fill_rect(buffer, width, height, bx, by, bw, bh, pack_color(0.18, 0.28, 0.20));
                let scale = 2;
                let tw = font::measure_text(label, scale, 1);
                let tx = bx + (bw - tw) / 2;
                let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;
                draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(0.4, 0.9, 0.5));
            } else {
                draw_option_button(buffer, width, height, label, bx, by, bw, bh, is_hover, i == 4, mouse_pos);
            }
        }
    }

    let hint = if is_gamepad_active {
        t("settings.controls_hint_gamepad")
    } else {
        t("settings.controls_hint")
    };
    let hw = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, hint, cx - hw / 2, height as i32 - 24, 1, 1, pack_color(0.7, 0.75, 0.85));

    let build_info = crate::core::build_info::get_build_info();
    let bw = font::measure_text(build_info, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, build_info, (width as i32 - bw) / 2, height as i32 - 12, 1, 1, pack_color(0.48, 0.54, 0.65));
}
