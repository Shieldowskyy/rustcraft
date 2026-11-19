//! Renderowanie menu ustawień gry, nawigacji po zakładkach (Grafika, Audio, Sterowanie, Debug) i przycisków opcji.
//! Całkowicie wolne od `#[cfg]` — dostępność przypisywania klawiszy pobierana z `platform::capabilities()`.

use crate::core::i18n::{t, t_fmt};
use crate::core::settings::GameSettings;
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Button, HStack, Rect, UiCanvas, VStack};
use crate::platform::capabilities;

pub fn settings_tab_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let cy = height as i32 / 2;
    HStack::centered_row(width, cy - 185, 116, 24, 4, 6).array(116)
}

fn settings_column_rects<const N: usize>(
    width: usize,
    start_y: i32,
    opt_h: i32,
    spacing: i32,
    last_extra_margin: i32,
) -> [(i32, i32, i32, i32); N] {
    let mut stack = VStack::centered(width, start_y, 480, spacing);
    std::array::from_fn(|i| {
        if i + 1 == N {
            stack.skip(last_extra_margin);
        }
        stack.next(opt_h).to_tuple()
    })
}

pub fn settings_audio_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 8] {
    settings_column_rects(width, height as i32 / 2 - 135, 24, 7, 6)
}

pub fn settings_graphics_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 14] {
    settings_column_rects(width, height as i32 / 2 - 156, 19, 2, 5)
}

pub fn settings_controls_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 11] {
    settings_column_rects(width, height as i32 / 2 - 146, 20, 5, 6)
}

pub fn settings_debug_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 5] {
    settings_column_rects(width, height as i32 / 2 - 130, 28, 10, 10)
}

pub fn settings_option_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 14] {
    settings_graphics_rects(width, height)
}

fn draw_option_button(
    ui: &mut UiCanvas<'_>,
    label: &str,
    rect: impl Into<Rect>,
    is_hover: bool,
    is_action_btn: bool,
    mouse_pos: Option<(f32, f32)>,
) {
    let r = rect.into();
    let has_arrows = label.contains('<') && label.contains('>');

    if !has_arrows {
        let mut btn = Button::new(label).hovered(is_hover);
        if is_action_btn {
            btn = btn.primary();
        }
        ui.button(r, btn);
        return;
    }

    let body_color = if is_hover {
        pack_color(0.32, 0.42, 0.60)
    } else {
        pack_color(0.20, 0.22, 0.26)
    };
    let top_left = if is_hover {
        pack_color(0.75, 0.85, 1.0)
    } else {
        pack_color(0.55, 0.55, 0.60)
    };
    let bottom_right = if is_hover {
        pack_color(0.15, 0.20, 0.35)
    } else {
        pack_color(0.10, 0.10, 0.12)
    };

    ui.fill(r, body_color);
    ui.bevel_border(r, 2, top_left, bottom_right);

    let arrow_w = 28;
    let left_arrow = Rect::new(r.x + 4, r.y + 2, arrow_w, r.h - 4);
    let right_arrow = Rect::new(r.right() - arrow_w - 4, r.y + 2, arrow_w, r.h - 4);

    let (is_left_hover, is_right_hover) = if let Some((mx, my)) = mouse_pos {
        let px = mx as i32;
        let py = my as i32;
        if r.contains_point(px, py) {
            (
                px < r.x + arrow_w + 12 || px < r.x + r.w / 3,
                px >= r.right() - arrow_w - 12 || px >= r.x + r.w * 2 / 3,
            )
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };

    for (arrow_rect, arrow_hover, sym) in [
        (left_arrow, is_left_hover, "<"),
        (right_arrow, is_right_hover, ">"),
    ] {
        let a_bg = if arrow_hover {
            pack_color(0.38, 0.52, 0.78)
        } else if is_hover {
            pack_color(0.26, 0.34, 0.48)
        } else {
            pack_color(0.16, 0.18, 0.22)
        };
        let a_border = if arrow_hover {
            pack_color(1.0, 0.95, 0.3)
        } else if is_hover {
            pack_color(0.65, 0.75, 0.92)
        } else {
            pack_color(0.32, 0.35, 0.42)
        };
        let a_col = if arrow_hover {
            pack_color(1.0, 1.0, 0.3)
        } else {
            pack_color(0.85, 0.90, 0.98)
        };
        ui.bordered_box(arrow_rect, 1, a_bg, a_border);
        ui.text_in_rect(arrow_rect, sym, 2, 1, a_col);
    }

    let display_label = label
        .replace("< ", "")
        .replace(" >", "")
        .replace('<', "")
        .replace('>', "");
    let text_color = if is_hover {
        pack_color(1.0, 1.0, 0.3)
    } else {
        pack_color(0.90, 0.92, 0.95)
    };
    ui.text_in_rect(r, &display_label, 2, 1, text_color);
}

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
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.0, 0.0, 0.0), 0.78);

    let cx = ui.cx();
    let cy = ui.cy();

    ui.text_centered_x(
        t("settings.title"),
        cx,
        cy - 220,
        3,
        2,
        pack_color(0.95, 0.85, 0.35),
    );

    let tab_rects = settings_tab_rects(width, height);
    let tab_labels = [
        t("settings.tab_graphics"),
        t("settings.tab_audio"),
        t("settings.tab_controls"),
        t("settings.tab_debug"),
    ];

    for (t_idx, (&label, &rect)) in tab_labels.iter().zip(tab_rects.iter()).enumerate() {
        ui.button(
            rect,
            Button::new(label)
                .tab(active_tab == t_idx)
                .hovered(hovered_tab == Some(t_idx)),
        );
    }

    let on_str = t("settings.on");
    let off_str = t("settings.off");

    if active_tab == 0 {
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
        for (i, (label, &rect)) in options.iter().zip(rects.iter()).enumerate() {
            draw_option_button(&mut ui, label, rect, hovered_idx == Some(i), i == 13, mouse_pos);
        }
    } else if active_tab == 1 {
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
        for (i, (label, &rect)) in options.iter().zip(rects.iter()).enumerate() {
            draw_option_button(&mut ui, label, rect, hovered_idx == Some(i), i == 7, mouse_pos);
        }
    } else if active_tab == 2 {
        let sprint_str = t(settings.sprint_mode.translation_key());
        let sneak_str = t(settings.sneak_mode.translation_key());
        let jump_str = t(settings.jump_mode.translation_key());
        let mouse_sens_pct = (settings.mouse_sensitivity * 100.0).round() as i32;
        let gp_sens_pct = (settings.gamepad_sensitivity * 100.0).round() as i32;

        let rebind_label = if capabilities().supports_rebind_controls {
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
        for (i, (label, &rect)) in options.iter().zip(rects.iter()).enumerate() {
            let is_action_btn = i == 9 || i == 10;
            draw_option_button(&mut ui, label, rect, hovered_idx == Some(i), is_action_btn, mouse_pos);
        }
    } else {
        let status_str = if settings.profiler_server {
            if profiler_copied {
                t_fmt(
                    "settings.profiler_server_copied",
                    &[server_ip, &settings.profiler_server_port.to_string()],
                )
            } else {
                t_fmt(
                    "settings.profiler_server_active",
                    &[server_ip, &settings.profiler_server_port.to_string()],
                )
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
        for (i, (label, &rect)) in options.iter().zip(rects.iter()).enumerate() {
            if i == 1 && settings.profiler_server {
                ui.fill(rect, pack_color(0.18, 0.28, 0.20));
                ui.text_in_rect(rect, label, 2, 1, pack_color(0.4, 0.9, 0.5));
            } else {
                draw_option_button(&mut ui, label, rect, hovered_idx == Some(i), i == 4, mouse_pos);
            }
        }
    }

    let hint = if is_gamepad_active {
        t("settings.controls_hint_gamepad")
    } else {
        t("settings.controls_hint")
    };
    ui.footer(hint, true);
}
