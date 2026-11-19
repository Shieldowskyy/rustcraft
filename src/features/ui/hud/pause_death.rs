//! In-game pause menu overlay, controls reference cards, and death screen overlay.
//! Uses `platform::capabilities()` for confirm/back button hints with zero platform `#[cfg]`.

use crate::platform::capabilities;

use super::super::draw::pack_color;
use super::super::widgets::{Rect, UiCanvas, VStack};

pub fn pause_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 7] {
    VStack::centered(width, height as i32 / 2 - 125, 280, 6).array(30)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_pause_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    fly_mode: bool,
    hovered_idx: Option<usize>,
    is_gamepad_active: bool,
    can_host: bool,
    is_hosting: bool,
    status: Option<&str>,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.0, 0.0, 0.0), 0.72);

    let cx = ui.cx();
    let cy = ui.cy();

    let title = crate::core::i18n::t("pause.title");
    ui.text_centered_x(title, cx, cy - 145, 3, 2, pack_color(0.95, 0.85, 0.35));

    let b0 = if is_gamepad_active {
        format!("{} [START / A]", crate::core::i18n::t("pause.resume"))
    } else {
        format!("{} (ESC)", crate::core::i18n::t("pause.resume"))
    };

    let b1 = crate::core::i18n::t("pause.settings").to_string();

    let b2 = if is_gamepad_active {
        if fly_mode {
            format!("{} [R3]", crate::core::i18n::t("pause.fly_on"))
        } else {
            format!("{} [R3]", crate::core::i18n::t("pause.fly_off"))
        }
    } else if fly_mode {
        format!("{} (F)", crate::core::i18n::t("pause.fly_on"))
    } else {
        format!("{} (F)", crate::core::i18n::t("pause.fly_off"))
    };

    let b3 = if is_hosting {
        crate::core::i18n::t("pause.stop_host").to_string()
    } else if can_host {
        crate::core::i18n::t("pause.host").to_string()
    } else {
        crate::core::i18n::t("pause.host_unavailable").to_string()
    };

    let b4 = crate::core::i18n::t("pause.achievements").to_string();
    let b5 = if is_gamepad_active {
        format!("{} [SELECT / Y]", crate::core::i18n::t("pause.main_menu"))
    } else {
        format!("{} (M)", crate::core::i18n::t("pause.main_menu"))
    };
    let b6 = if is_gamepad_active {
        format!("{} [B]", crate::core::i18n::t("pause.quit"))
    } else {
        format!("{} (Q)", crate::core::i18n::t("pause.quit"))
    };

    let buttons = [b0, b1, b2, b3, b4, b5, b6];
    let rects = pause_menu_button_rects(width, height);

    for (i, (label, &rect_t)) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);
        let r = Rect::from(rect_t);

        let body_color = if is_hover {
            pack_color(0.32, 0.44, 0.68)
        } else {
            pack_color(0.20, 0.20, 0.24)
        };
        let top_left = if is_hover {
            pack_color(0.75, 0.85, 1.0)
        } else {
            pack_color(0.50, 0.50, 0.56)
        };
        let bottom_right = if is_hover {
            pack_color(0.15, 0.20, 0.35)
        } else {
            pack_color(0.10, 0.10, 0.12)
        };

        ui.fill(r, body_color);
        ui.bevel_border(r, 2, top_left, bottom_right);

        let text_col = if is_hover {
            pack_color(1.0, 1.0, 0.3)
        } else {
            pack_color(1.0, 1.0, 1.0)
        };
        ui.text_in_rect(r, label, 2, 1, text_col);
    }

    let tooltip = status.unwrap_or_else(|| {
        if is_gamepad_active {
            match hovered_idx {
                Some(0) => "Resume gameplay and return to world [START] or [A]",
                Some(1) => "Adjust video, sound, controls, and debug options [A]",
                Some(2) => {
                    if fly_mode {
                        "Disable flight mode and restore standard gravity [R3]"
                    } else {
                        "Enable free flight mode without gravity [R3]"
                    }
                }
                Some(3) => "Save current game world and return to save select menu [A]",
                Some(4) => "Review milestones and the paths between them [A]",
                Some(5) => "Save world progress and return to main menu [SELECT / Y]",
                _ => "Navigate with D-PAD / Left Stick, select with [A], back with [B]",
            }
        } else {
            match hovered_idx {
                Some(0) => "Resume gameplay and return to world (ESC / P)",
                Some(1) => "Adjust render distance, fog, 3D clouds, animations, and FOV",
                Some(2) => {
                    if fly_mode {
                        "Disable flight mode and restore standard gravity (F)"
                    } else {
                        "Enable free flight mode without gravity (F)"
                    }
                }
                Some(3) => "Save current game world and return to save select menu (M)",
                Some(4) => "Review milestones and the paths between them (ENTER)",
                Some(5) => "Save world progress and return to main menu (M)",
                _ => "Navigate with mouse, arrow keys, or gamepad stick",
            }
        }
    });

    let tip_w = 460.min(width as i32 - 40);
    let tip_rect = Rect::new(cx - tip_w / 2, cy + 122, tip_w, 24);
    ui.fill_alpha(tip_rect, pack_color(0.08, 0.10, 0.16), 0.85);
    ui.border(tip_rect, 1, pack_color(0.35, 0.45, 0.65));

    let max_tooltip_chars = (tip_w.saturating_sub(20) / 6).max(1) as usize;
    let tooltip_text: String = tooltip.chars().take(max_tooltip_chars).collect();
    let tip_color = if status.is_some() {
        pack_color(0.55, 0.9, 1.0)
    } else if hovered_idx.is_some() {
        pack_color(0.95, 0.90, 0.40)
    } else {
        pack_color(0.65, 0.70, 0.80)
    };
    ui.text_in_rect(tip_rect, &tooltip_text, 1, 1, tip_color);

    let card_w = 260;
    let card_h = 320;
    let card_y = cy - 150;

    let (kb_border, gp_border) = if is_gamepad_active {
        (pack_color(0.20, 0.25, 0.35), pack_color(1.0, 0.85, 0.35))
    } else {
        (pack_color(0.50, 0.70, 1.0), pack_color(0.20, 0.25, 0.35))
    };

    if width >= 860 {
        let left_x = (cx - 140 - card_w - 20).max(12);
        let left_card = Rect::new(left_x, card_y, card_w, card_h);
        ui.modal_panel(
            left_card,
            pack_color(0.08, 0.09, 0.13),
            0.82,
            kb_border,
            pack_color(0.20, 0.25, 0.35),
        );

        ui.text_centered_x(
            "[ KEYBOARD & MOUSE ]",
            left_card.center_x(),
            card_y + 10,
            1,
            1,
            if is_gamepad_active {
                pack_color(0.6, 0.65, 0.7)
            } else {
                pack_color(0.95, 0.85, 0.35)
            },
        );

        let kb_bindings = [
            ("W, A, S, D", "Movement"),
            ("SPACE", "Jump / Ascend"),
            ("SHIFT / C", "Sneak / Descend"),
            ("CTRL", "Sprint"),
            ("LMB", "Mine / Attack"),
            ("RMB", "Place / Consume"),
            ("E", "Inventory / Crafting"),
            ("Q", "Drop Item"),
            ("1-9 / SCROLL", "Hotbar Slot"),
            ("F", "Toggle Fly Mode"),
            ("G", "Creative / Survival"),
            ("F3", "Debug Diagnostics"),
            ("ESC / P", "Pause / Game Menu"),
        ];

        let mut row_y = card_y + 32;
        for (key, desc) in &kb_bindings {
            ui.text(key, left_x + 12, row_y, 1, 1, pack_color(0.45, 0.85, 1.0));
            ui.text_right(desc, left_x + card_w - 12, row_y, 1, 1, pack_color(0.85, 0.88, 0.92));
            row_y += 21;
        }

        let right_x = (cx + 140 + 20).min(width as i32 - card_w - 12);
        let right_card = Rect::new(right_x, card_y, card_w, card_h);
        ui.modal_panel(
            right_card,
            pack_color(0.08, 0.09, 0.13),
            0.82,
            gp_border,
            pack_color(0.20, 0.25, 0.35),
        );

        ui.text_centered_x(
            "[ GAMEPAD CONTROLS ]",
            right_card.center_x(),
            card_y + 10,
            1,
            1,
            if is_gamepad_active {
                pack_color(0.95, 0.85, 0.35)
            } else {
                pack_color(0.6, 0.65, 0.7)
            },
        );

        let gp_bindings = [
            ("L-Stick", "Movement"),
            ("R-Stick", "Look / Aim"),
            ("[X] / A", "Jump / Ascend"),
            ("[O] / B", "Sneak / Descend"),
            ("L3 (Click)", "Sprint"),
            ("R2 / RT", "Mine / Attack"),
            ("L2 / LT", "Place / Consume"),
            ("[^] / Y", "Inventory / Crafting"),
            ("L1 / R1", "Hotbar Slot"),
            ("R3 (Click)", "Toggle Fly Mode"),
            ("SELECT", "Creative / Survival"),
            ("D-PAD UP", "Debug Overlay"),
            ("D-PAD LEFT", "Drop Item"),
        ];

        let mut row_y = card_y + 32;
        for (key, desc) in &gp_bindings {
            ui.text(key, right_x + 12, row_y, 1, 1, pack_color(0.45, 0.85, 1.0));
            ui.text_right(desc, right_x + card_w - 12, row_y, 1, 1, pack_color(0.85, 0.88, 0.92));
            row_y += 21;
        }
    } else {
        let footers = [
            "KEYBOARD: WASD-Move | SPACE-Jump | SHIFT-Sneak | CTRL-Sprint | LMB-Mine | RMB-Place",
            "E-Inventory | Q-Drop | 1-9-Hotbar | F-Flight | G-Creative | F3-Debug | ESC-Pause",
            "GAMEPAD: L/R-Stick | [X]-Jump | [O]-Sneak | L3-Sprint | R2-Mine | L2-Place | [^]-Inventory",
        ];
        let mut fy = cy + 155;
        for &line in &footers {
            ui.text_centered_x(line, cx, fy, 1, 1, pack_color(0.75, 0.78, 0.85));
            fy += 15;
        }
    }

    let build_info = crate::core::build_info::get_build_info();
    ui.text_centered_x(build_info, cx, height as i32 - 14, 1, 1, pack_color(0.50, 0.55, 0.65));
}

pub fn death_screen_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 2] {
    VStack::centered(width, height as i32 / 2 + 10, 340, 16).array(42)
}

pub fn draw_death_screen(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    hovered_idx: Option<usize>,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, pack_color(0.50, 0.05, 0.05), 0.65);

    let cx = ui.cx();
    let cy = ui.cy();

    let title = crate::core::i18n::t("death.title");
    ui.text_centered_x(title, cx, cy - 110, 5, 2, pack_color(1.0, 0.15, 0.15));

    let subtitle = crate::core::i18n::t("death.subtitle");
    ui.text_centered_x(subtitle, cx, cy - 40, 2, 1, pack_color(0.9, 0.85, 0.85));

    let caps = capabilities();
    let b0 = format!("{} {}", crate::core::i18n::t("death.respawn"), caps.confirm_label);
    let b1 = format!("{} {}", crate::core::i18n::t("death.main_menu"), caps.back_label);

    let buttons = [b0, b1];
    let rects = death_screen_button_rects(width, height);

    for (i, (label, &rect_t)) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);
        let r = Rect::from(rect_t);

        let body_color = if is_hover {
            if i == 0 {
                pack_color(0.30, 0.50, 0.30)
            } else {
                pack_color(0.45, 0.28, 0.28)
            }
        } else {
            pack_color(0.22, 0.18, 0.18)
        };
        let top_left = if is_hover {
            pack_color(0.85, 0.95, 0.85)
        } else {
            pack_color(0.55, 0.50, 0.50)
        };
        let bottom_right = if is_hover {
            pack_color(0.20, 0.30, 0.20)
        } else {
            pack_color(0.12, 0.08, 0.08)
        };

        ui.fill(r, body_color);
        ui.bevel_border(r, 2, top_left, bottom_right);

        let text_col = if is_hover {
            pack_color(1.0, 1.0, 0.3)
        } else {
            pack_color(1.0, 1.0, 1.0)
        };
        ui.text_in_rect(r, label, 2, 1, text_col);
    }
}
