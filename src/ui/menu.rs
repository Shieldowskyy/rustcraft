//! Main menu screen, voxel logo rendering, nickname box, and button hitboxes.

use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Calculates layout rectangles for the 4 main menu buttons `[(x, y, width, height); 4]`.
/// 0: Singleplayer
/// 1: Multiplayer
/// 2: Settings
/// 3: Quit
pub fn main_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let btn_w = 380;
    let btn_h = 40;
    let spacing = 10;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 - 15;
    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 3, btn_w, btn_h),
    ]
}

/// Bounding rectangle for the clickable nickname display badge.
pub fn main_menu_nickname_rect(width: usize, height: usize) -> (i32, i32, i32, i32) {
    let btn_w = 380;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 - 15;
    (cx - btn_w / 2, start_y - 38, btn_w, 28)
}

/// Draws the procedural pixel-art voxel logo on the main menu.
pub fn draw_rustcraft_logo(buffer: &mut [u32], width: usize, height: usize, cx: i32, y: i32) {
    let gold = pack_color(0.95, 0.72, 0.20);
    let light = pack_color(1.0, 0.90, 0.48);
    let dark = pack_color(0.38, 0.22, 0.08);
    let blocks = [
        (-48, 0, 8, 8, gold), (-40, 0, 8, 8, light), (-32, 0, 8, 8, gold),
        (-24, 0, 8, 8, dark), (-16, 0, 8, 8, gold), (-8, 0, 8, 8, light),
        (-40, 8, 8, 8, gold), (-32, 16, 8, 8, light), (-24, 24, 8, 8, gold),
        (-16, 32, 8, 8, dark), (-8, 40, 8, 8, gold), (0, 32, 8, 8, light),
        (8, 24, 8, 8, gold), (16, 16, 8, 8, dark), (24, 8, 8, 8, gold),
        (32, 0, 8, 8, light), (40, -8, 8, 8, gold),
    ];
    for (x, offset_y, w, h, color) in blocks {
        fill_rect(buffer, width, height, cx + x, y + offset_y, w, h, color);
    }
}

/// Renders the title screen, animated logo, nickname badge, menu choices, and platform controls guide.
pub fn draw_main_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    hovered_idx: Option<usize>,
    nickname: &str,
    nick_hovered: bool,
    is_gamepad_active: bool,
    splash_text: &str,
    time_sec: f32,
) {
    let cx = width as i32 / 2;
    // Clear screen to prevent ghost frames from other menus
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.06, 0.07, 0.09));
    fill_rect_alpha(buffer, width, height, cx - 240, 25, 480, 480, pack_color(0.02, 0.03, 0.05), 0.75);
    draw_rustcraft_logo(buffer, width, height, cx, 60);

    let title = "RUSTCRAFT";
    let title_w = font::measure_text(title, 4, 3);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, 125, 4, 3, pack_color(0.95, 0.85, 0.42));

    // Dynamic bouncing Minecraft-style splash text positioned at the bottom-right of the title logo
    if !splash_text.is_empty() {
        let splash_cx = cx + title_w / 2 + 10;
        let splash_cy = 138;
        crate::ui::splashes::draw_splash_text(buffer, width, height, splash_text, splash_cx, splash_cy, time_sec);
    }

    let subtitle = "VOXEL ENGINE";
    let subtitle_w = font::measure_text(subtitle, 2, 2);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - subtitle_w / 2, 172, 2, 2, pack_color(0.7, 0.75, 0.85));

    // Nickname badge box
    let (nx, ny, nw, nh) = main_menu_nickname_rect(width, height);
    let nick_bg = if nick_hovered { pack_color(0.18, 0.24, 0.35) } else { pack_color(0.10, 0.12, 0.16) };
    let nick_border = if nick_hovered { pack_color(0.4, 0.65, 0.95) } else { pack_color(0.25, 0.30, 0.40) };
    fill_rect(buffer, width, height, nx, ny, nw, nh, nick_bg);
    fill_rect(buffer, width, height, nx, ny, nw, 1, nick_border);
    fill_rect(buffer, width, height, nx, ny, 1, nh, nick_border);
    fill_rect(buffer, width, height, nx, ny + nh - 1, nw, 1, nick_border);
    fill_rect(buffer, width, height, nx + nw - 1, ny, 1, nh, nick_border);

    let nick_text = if is_gamepad_active {
        format!("{} [Y / N]", crate::core::i18n::t_fmt("menu.nickname", &[nickname]))
    } else if nick_hovered {
        format!("{} [CLICK TO CHANGE]", crate::core::i18n::t_fmt("menu.nickname", &[nickname]))
    } else {
        crate::core::i18n::t_fmt("menu.nickname", &[nickname])
    };
    let ntw = font::measure_text(&nick_text, 1, 1);
    let n_col = if nick_hovered { pack_color(1.0, 0.95, 0.4) } else { pack_color(0.85, 0.90, 1.0) };
    draw_text_colored_shadowed(buffer, width, height, &nick_text, cx - ntw / 2, ny + 8, 1, 1, n_col);

    let buttons = [
        crate::core::i18n::t("menu.play"),
        crate::core::i18n::t("menu.multiplayer"),
        crate::core::i18n::t("menu.settings"),
        crate::core::i18n::t("menu.quit"),
    ];
    let rects = main_menu_button_rects(width, height);
    for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);
        let body_color = if is_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.22, 0.26) };
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
        let ty = by + (bh - font::GLYPH_H as i32 * scale) / 2;
        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let footer = if is_gamepad_active {
        crate::core::i18n::t("menu.controls_gamepad")
    } else {
        crate::core::i18n::t("menu.controls")
    };
    let fw = font::measure_text(footer, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, footer, cx - fw / 2, 470, 1, 1, pack_color(0.6, 0.65, 0.75));

    // Build version & platform telemetry at the bottom of the screen
    let build_info = crate::core::build_info::get_build_info();
    let bw = font::measure_text(build_info, 1, 1);
    let build_y = height as i32 - 16;
    draw_text_colored_shadowed(buffer, width, height, build_info, (width as i32 - bw) / 2, build_y, 1, 1, pack_color(0.52, 0.58, 0.68));
}
