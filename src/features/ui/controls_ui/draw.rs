//! Renderowanie ekranu konfiguracji przypisań klawiatury, myszy i pada.

use crate::core::controls::{Action, ControlsConfig};
use crate::core::i18n::{Language, t};
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Button, ButtonVariant, Rect, UiCanvas};

use super::layout::{ControlsRebindLayout, RebindSlot, VISIBLE_REBIND_ROWS, filter_actions};

pub fn draw_controls_rebind_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    config: &ControlsConfig,
    query: &str,
    scroll_offset: usize,
    rebind_target: Option<(Action, RebindSlot)>,
    hovered_row: Option<usize>,
    hovered_col: Option<usize>,
    hovered_btn: Option<usize>,
    lang: Language,
    cursor_timer: f32,
    is_gamepad_active: bool,
    search_focused: bool,
) {
    let layout = ControlsRebindLayout::new(width, height);
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    ui.backdrop(None, pack_color(0.02, 0.03, 0.05), 0.88);

    let title = match lang {
        Language::Polish => "PRZYPISANIE KLAWISZY I KONTROLERA (PC)",
        Language::English => "KEY & CONTROLLER BINDINGS (PC)",
    };
    ui.text_centered_x(title, cx, layout.search_box.1 - 42, 2, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = match lang {
        Language::Polish => "Wyszukaj akcje lub kliknij pole, aby przypisać klawisz / mysz / przycisk pada",
        Language::English => "Search actions or click a slot to rebind keyboard, mouse, or gamepad button",
    };
    ui.text_centered_x(subtitle, cx, layout.search_box.1 - 20, 1, 1, pack_color(0.68, 0.74, 0.84));

    let search_rect = Rect::from_tuple(layout.search_box);
    let border_color = if search_focused {
        pack_color(1.0, 0.85, 0.3)
    } else {
        pack_color(0.35, 0.40, 0.50)
    };
    let bottom_right = if search_focused {
        border_color
    } else {
        pack_color(0.20, 0.22, 0.28)
    };
    ui.fill(search_rect, pack_color(0.12, 0.14, 0.18));
    ui.bevel_border(search_rect, 1, border_color, bottom_right);

    let text_y = search_rect.y + (search_rect.h - 7) / 2;
    if query.is_empty() {
        let placeholder = match lang {
            Language::Polish => "Szukaj akcji... (np. ruch, skok, atak, F5, mysz)",
            Language::English => "Search actions... (e.g. forward, jump, attack, F5, mouse)",
        };
        if search_focused && (cursor_timer * 2.0) as i32 % 2 == 0 {
            ui.text("|", search_rect.x + 10, text_y, 1, 1, pack_color(1.0, 1.0, 0.3));
            ui.text(placeholder, search_rect.x + 18, text_y, 1, 1, pack_color(0.45, 0.50, 0.58));
        } else {
            ui.text(placeholder, search_rect.x + 10, text_y, 1, 1, pack_color(0.45, 0.50, 0.58));
        }
    } else {
        let cursor_char = if search_focused && (cursor_timer * 2.0) as i32 % 2 == 0 {
            "|"
        } else {
            ""
        };
        let full_text = format!("{}{}", query, cursor_char);
        ui.text(&full_text, search_rect.x + 10, text_y, 1, 1, pack_color(1.0, 1.0, 0.3));

        let clear_bg = if hovered_btn == Some(2) {
            pack_color(0.4, 0.2, 0.2)
        } else {
            pack_color(0.25, 0.15, 0.15)
        };
        ui.fill(layout.search_clear_btn, clear_bg);
        ui.text_in_rect(layout.search_clear_btn, "X", 1, 1, pack_color(1.0, 0.7, 0.7));
    }

    let header_y = layout.list_y - 18;
    let col_action_title = match lang {
        Language::Polish => "AKCJA",
        Language::English => "ACTION",
    };
    let col_km_title = match lang {
        Language::Polish => "KLAWIATURA / MYSZ",
        Language::English => "KEYBOARD / MOUSE",
    };
    let col_gp_title = match lang {
        Language::Polish => "KONTROLER (PAD)",
        Language::English => "GAMEPAD",
    };
    let hdr_col = pack_color(0.75, 0.80, 0.90);

    ui.text(col_action_title, cx - 380 + 12, header_y, 1, 1, hdr_col);
    ui.text(col_km_title, cx - 60 + 10, header_y, 1, 1, hdr_col);
    ui.text(col_gp_title, cx + 140 + 10, header_y, 1, 1, hdr_col);
    ui.text("RESET", cx + 310 + 10, header_y, 1, 1, hdr_col);

    let filtered_actions = filter_actions(query, config, lang);
    let total_filtered = filtered_actions.len();
    let max_scroll = total_filtered.saturating_sub(VISIBLE_REBIND_ROWS);
    let scroll_idx = scroll_offset.min(max_scroll);

    if total_filtered == 0 {
        let no_res = match lang {
            Language::Polish => "Nie znaleziono pasujących akcji",
            Language::English => "No matching actions found",
        };
        ui.text_centered_x(no_res, cx, layout.list_y + 60, 1, 1, pack_color(0.6, 0.65, 0.75));
    }

    for visible_idx in 0..VISIBLE_REBIND_ROWS {
        let action_idx = scroll_idx + visible_idx;
        if action_idx >= total_filtered {
            break;
        }

        let action = filtered_actions[action_idx];
        let row_rect = Rect::from_tuple(layout.row_rect(cx, visible_idx));
        let (k_btn, gp_btn, rst_btn) = layout.row_buttons(cx, visible_idx);

        let is_row_hover = hovered_row == Some(visible_idx);
        let row_bg = if is_row_hover {
            pack_color(0.16, 0.20, 0.28)
        } else if visible_idx % 2 == 0 {
            pack_color(0.09, 0.11, 0.15)
        } else {
            pack_color(0.12, 0.14, 0.19)
        };

        ui.fill(row_rect, row_bg);

        let label = format!("{} [{}]", action.display_name(lang), action.category(lang));
        ui.text(&label, row_rect.x + 12, row_rect.y + 8, 1, 1, pack_color(0.92, 0.94, 0.98));

        let is_km_rebind = rebind_target == Some((action, RebindSlot::KeyOrMouse));
        let is_km_hover = is_row_hover && hovered_col == Some(0);

        let binding = &config.bindings[action.index()];
        let km_text: String = if is_km_rebind {
            let pulse = ((cursor_timer * 6.0).sin() * 0.5 + 0.5) > 0.5;
            if pulse {
                "> WCIŚNIJ KLAWISZ <".to_string()
            } else {
                "> WYBIERZ PRZYCISK <".to_string()
            }
        } else if !binding.keys.is_empty() {
            let k_names: Vec<&'static str> = binding.keys.iter().map(|k| k.display_name()).collect();
            let joined = k_names.join(", ");
            if !binding.mouse.is_empty() {
                let m_names: Vec<&'static str> = binding.mouse.iter().map(|m| m.display_name()).collect();
                format!("{}, {}", joined, m_names.join(", "))
            } else {
                joined
            }
        } else if !binding.mouse.is_empty() {
            let m_names: Vec<&'static str> = binding.mouse.iter().map(|m| m.display_name()).collect();
            m_names.join(", ")
        } else {
            "---".to_string()
        };

        let km_bg = if is_km_rebind {
            pack_color(0.40, 0.35, 0.10)
        } else {
            pack_color(0.18, 0.22, 0.30)
        };
        let km_border = if is_km_rebind {
            pack_color(1.0, 0.9, 0.2)
        } else {
            pack_color(0.32, 0.36, 0.45)
        };
        let km_col = if is_km_rebind {
            pack_color(1.0, 1.0, 0.3)
        } else if is_km_hover {
            pack_color(1.0, 1.0, 0.6)
        } else {
            pack_color(0.9, 0.92, 0.96)
        };

        ui.button(
            k_btn,
            Button::new(&km_text)
                .scale(1)
                .hovered(is_km_hover)
                .text_color(km_col)
                .variant(ButtonVariant::Flat {
                    bg: km_bg,
                    hover_bg: if is_km_rebind { km_bg } else { pack_color(0.28, 0.36, 0.52) },
                    border: km_border,
                    hover_border: if is_km_rebind { km_border } else { pack_color(0.65, 0.75, 0.95) },
                })
                .flat_border(),
        );

        let is_gp_rebind = rebind_target == Some((action, RebindSlot::Gamepad));
        let is_gp_hover = is_row_hover && hovered_col == Some(1);

        let gp_text: String = if is_gp_rebind {
            let pulse = ((cursor_timer * 6.0).sin() * 0.5 + 0.5) > 0.5;
            if pulse {
                "> PRZYCISK PADA <".to_string()
            } else {
                "> NACIŚNIJ <".to_string()
            }
        } else if !binding.gamepad.is_empty() {
            let gp_names: Vec<&'static str> = binding.gamepad.iter().map(|gp| gp.display_name()).collect();
            gp_names.join(", ")
        } else {
            "---".to_string()
        };

        let gp_bg = if is_gp_rebind {
            pack_color(0.12, 0.35, 0.40)
        } else {
            pack_color(0.16, 0.20, 0.26)
        };
        let gp_border = if is_gp_rebind {
            pack_color(0.3, 0.9, 1.0)
        } else {
            pack_color(0.30, 0.34, 0.42)
        };
        let gp_col = if is_gp_rebind {
            pack_color(0.4, 0.95, 1.0)
        } else if is_gp_hover {
            pack_color(0.8, 0.95, 1.0)
        } else {
            pack_color(0.88, 0.90, 0.94)
        };

        ui.button(
            gp_btn,
            Button::new(&gp_text)
                .scale(1)
                .hovered(is_gp_hover)
                .text_color(gp_col)
                .variant(ButtonVariant::Flat {
                    bg: gp_bg,
                    hover_bg: if is_gp_rebind { gp_bg } else { pack_color(0.24, 0.34, 0.48) },
                    border: gp_border,
                    hover_border: if is_gp_rebind { gp_border } else { pack_color(0.60, 0.75, 0.90) },
                })
                .flat_border(),
        );

        let is_rst_hover = is_row_hover && hovered_col == Some(2);
        let rst_col = if is_rst_hover {
            pack_color(1.0, 0.8, 0.8)
        } else {
            pack_color(0.8, 0.65, 0.68)
        };
        ui.button(
            rst_btn,
            Button::new("RESET")
                .scale(1)
                .hovered(is_rst_hover)
                .text_color(rst_col)
                .variant(ButtonVariant::Flat {
                    bg: pack_color(0.22, 0.16, 0.18),
                    hover_bg: pack_color(0.38, 0.22, 0.22),
                    border: pack_color(0.45, 0.30, 0.32),
                    hover_border: pack_color(0.9, 0.5, 0.5),
                })
                .flat_border(),
        );
    }

    ui.scrollbar(
        layout.scrollbar,
        VISIBLE_REBIND_ROWS,
        total_filtered,
        scroll_idx,
        pack_color(0.12, 0.14, 0.18),
        pack_color(0.40, 0.50, 0.65),
    );

    let reset_all_label = match lang {
        Language::Polish => "PRZYWRÓĆ DOMYŚLNE",
        Language::English => "RESET TO DEFAULTS",
    };
    ui.button(
        layout.reset_all_btn,
        Button::new(reset_all_label)
            .danger()
            .scale(1)
            .hovered(hovered_btn == Some(0)),
    );

    ui.button(
        layout.back_btn,
        Button::new(t("settings.save_and_back"))
            .primary()
            .scale(1)
            .hovered(hovered_btn == Some(1)),
    );

    let hint = if is_gamepad_active {
        "[D-PAD / L-STICK] Nawigacja | [A] Przypisz | [B / ESC] Wstecz"
    } else {
        "[ESC] Anuluj / Wróć  |  [LPM] Przypisz  |  [Kółko myszy] Przewijaj"
    };
    ui.footer(hint, false);
}
