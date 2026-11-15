//! Main menu screen, voxel logo rendering, nickname box, and button hitboxes.

use crate::graphics::font;
use crate::ui::draw::{fill_rect, pack_color};
use crate::ui::widgets::{Button, Rect, UiCanvas, VStack};

/// Calculates layout rectangles for the 4 main menu buttons `[(x, y, width, height); 4]`.
/// 0: Singleplayer
/// 1: Multiplayer
/// 2: Settings
/// 3: Quit
pub fn main_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let start_y = height as i32 / 2 - 15;
    VStack::centered(width, start_y, 380, 10).array(40)
}

/// Bounding rectangle for the clickable nickname display badge.
pub fn main_menu_nickname_rect(width: usize, height: usize) -> (i32, i32, i32, i32) {
    let start_y = height as i32 / 2 - 15;
    Rect::centered_x(width, start_y - 38, 380, 28).to_tuple()
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
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    // Clear screen to prevent ghost frames from other menus
    ui.fill(ui.screen_rect(), pack_color(0.06, 0.07, 0.09));
    ui.fill_alpha(
        Rect::centered_x(width, 25, 480, 480),
        pack_color(0.02, 0.03, 0.05),
        0.75,
    );
    draw_rustcraft_logo(ui.buffer, ui.width, ui.height, cx, 60);

    let title = "RUSTCRAFT";
    let title_w = font::measure_text(title, 4, 3);
    ui.text_centered_x(title, cx, 125, 4, 3, pack_color(0.95, 0.85, 0.42));

    // Dynamic bouncing Minecraft-style splash text positioned at the bottom-right of the title logo
    if !splash_text.is_empty() {
        let splash_cx = cx + title_w / 2 + 10;
        crate::ui::splashes::draw_splash_text(
            ui.buffer,
            ui.width,
            ui.height,
            splash_text,
            splash_cx,
            138,
            time_sec,
        );
    }

    ui.text_centered_x("VOXEL ENGINE", cx, 172, 2, 2, pack_color(0.7, 0.75, 0.85));

    // Nickname badge box
    let nick_rect = Rect::from_tuple(main_menu_nickname_rect(width, height));
    let nick_bg = if nick_hovered {
        pack_color(0.18, 0.24, 0.35)
    } else {
        pack_color(0.10, 0.12, 0.16)
    };
    let nick_border = if nick_hovered {
        pack_color(0.4, 0.65, 0.95)
    } else {
        pack_color(0.25, 0.30, 0.40)
    };
    ui.bordered_box(nick_rect, 1, nick_bg, nick_border);

    let nick_text = if is_gamepad_active {
        format!("{} [Y / N]", crate::core::i18n::t_fmt("menu.nickname", &[nickname]))
    } else if nick_hovered {
        format!(
            "{} [CLICK TO CHANGE]",
            crate::core::i18n::t_fmt("menu.nickname", &[nickname])
        )
    } else {
        crate::core::i18n::t_fmt("menu.nickname", &[nickname])
    };
    let n_col = if nick_hovered {
        pack_color(1.0, 0.95, 0.4)
    } else {
        pack_color(0.85, 0.90, 1.0)
    };
    ui.text_in_rect(nick_rect, &nick_text, 1, 1, n_col);

    let buttons = [
        crate::core::i18n::t("menu.play"),
        crate::core::i18n::t("menu.multiplayer"),
        crate::core::i18n::t("menu.settings"),
        crate::core::i18n::t("menu.quit"),
    ];
    let rects = main_menu_button_rects(width, height);
    for (i, (&label, &rect)) in buttons.iter().zip(rects.iter()).enumerate() {
        ui.button(rect, Button::new(label).hovered(hovered_idx == Some(i)));
    }

    let footer = if is_gamepad_active {
        crate::core::i18n::t("menu.controls_gamepad")
    } else {
        crate::core::i18n::t("menu.controls")
    };
    ui.text_centered_x(footer, cx, 470, 1, 1, pack_color(0.6, 0.65, 0.75));

    let build_info = crate::core::build_info::get_build_info();
    ui.text_centered_x(
        build_info,
        cx,
        height as i32 - 16,
        1,
        1,
        pack_color(0.52, 0.58, 0.68),
    );
}
