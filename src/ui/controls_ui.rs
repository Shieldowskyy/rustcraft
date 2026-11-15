//! Full PC Key and Controller Rebinding Menu UI.
//! Allows searching, rebinding keyboard keys (including F13-F24),
//! mouse buttons (standard and non-standard), and gamepad buttons.

use crate::core::controls::{Action, ControlsConfig};
use crate::core::i18n::{t, Language};
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Button, ButtonVariant, Rect, UiCanvas};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebindSlot {
    KeyOrMouse,
    Gamepad,
}

pub const VISIBLE_REBIND_ROWS: usize = 9;

/// Computes the layout bounding boxes for controls rebinding screen.
pub struct ControlsRebindLayout {
    pub search_box: (i32, i32, i32, i32),
    pub search_clear_btn: (i32, i32, i32, i32),
    pub list_y: i32,
    pub list_h: i32,
    pub row_h: i32,
    pub row_spacing: i32,
    pub reset_all_btn: (i32, i32, i32, i32),
    pub back_btn: (i32, i32, i32, i32),
    pub scrollbar: (i32, i32, i32, i32),
}

impl ControlsRebindLayout {
    pub fn new(width: usize, height: usize) -> Self {
        let cx = width as i32 / 2;
        let cy = height as i32 / 2;

        let search_w = 480;
        let search_h = 24;
        let search_y = cy - 200;

        let list_y = search_y + 48;
        let row_h = 28;
        let row_spacing = 4;
        let list_h = (row_h + row_spacing) * VISIBLE_REBIND_ROWS as i32 - row_spacing;

        let bottom_btn_w = 210;
        let bottom_btn_h = 28;
        let bottom_y = list_y + list_h + 16;

        Self {
            search_box: (cx - search_w / 2, search_y, search_w, search_h),
            search_clear_btn: (cx + search_w / 2 - 22, search_y + 2, 20, 20),
            list_y,
            list_h,
            row_h,
            row_spacing,
            reset_all_btn: (cx - bottom_btn_w - 10, bottom_y, bottom_btn_w, bottom_btn_h),
            back_btn: (cx + 10, bottom_y, bottom_btn_w, bottom_btn_h),
            scrollbar: (cx + 372, list_y, 8, list_h),
        }
    }

    pub fn row_rect(&self, cx: i32, visible_idx: usize) -> (i32, i32, i32, i32) {
        let y = self.list_y + visible_idx as i32 * (self.row_h + self.row_spacing);
        (cx - 380, y, 760, self.row_h)
    }

    pub fn row_buttons(&self, cx: i32, visible_idx: usize) -> ((i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32)) {
        let y = self.list_y + visible_idx as i32 * (self.row_h + self.row_spacing) + 2;
        let h = self.row_h - 4;
        let key_btn = (cx - 60, y, 190, h);
        let gp_btn = (cx + 140, y, 160, h);
        let reset_btn = (cx + 310, y, 58, h);
        (key_btn, gp_btn, reset_btn)
    }
}

/// Normalizes Polish and English characters for search matching.
pub fn normalize_search(s: &str) -> String {
    s.to_lowercase()
        .replace('ą', "a")
        .replace('ć', "c")
        .replace('ę', "e")
        .replace('ł', "l")
        .replace('ń', "n")
        .replace('ó', "o")
        .replace('ś', "s")
        .replace('ź', "z")
        .replace('ż', "z")
}

/// Filters actions matching the query string.
pub fn filter_actions(query: &str, config: &ControlsConfig, _lang: Language) -> Vec<Action> {
    let q = normalize_search(query.trim());
    if q.is_empty() {
        return Action::ALL.to_vec();
    }

    Action::ALL
        .iter()
        .copied()
        .filter(|&action| {
            let name_pl = normalize_search(action.display_name(Language::Polish));
            let name_en = normalize_search(action.display_name(Language::English));
            let cat_pl = normalize_search(action.category(Language::Polish));
            let cat_en = normalize_search(action.category(Language::English));
            let key_str = action.as_str();

            if name_pl.contains(&q) || name_en.contains(&q) || cat_pl.contains(&q) || cat_en.contains(&q) || key_str.contains(&q) {
                return true;
            }

            let binding = &config.bindings[action.index()];

            // Check aliases
            if (q.contains("mysz") || q.contains("mouse")) && !binding.mouse.is_empty() {
                return true;
            }
            if (q.contains("pad") || q.contains("gamepad") || q.contains("kontroler")) && !binding.gamepad.is_empty() {
                return true;
            }
            if (q.contains("klawisz") || q.contains("klawiatura") || q.contains("keyboard")) && !binding.keys.is_empty() {
                return true;
            }

            // Search through bound keys / mouse buttons
            for k in &binding.keys {
                let dn = normalize_search(k.display_name());
                let sn = normalize_search(k.as_str());
                if dn.contains(&q) || sn.contains(&q) {
                    return true;
                }
            }
            for m in &binding.mouse {
                let dn = normalize_search(m.display_name());
                let sn = normalize_search(m.as_str());
                if dn.contains(&q) || sn.contains(&q) {
                    return true;
                }
            }
            for gp in &binding.gamepad {
                let dn = normalize_search(gp.display_name());
                let sn = normalize_search(gp.as_str());
                if dn.contains(&q) || sn.contains(&q) {
                    return true;
                }
            }

            // Common synonyms & keywords
            match action {
                Action::MoveForward => q.contains("przod") || q.contains("forward") || q.contains("chodz") || q == "w",
                Action::MoveBackward => q.contains("tyl") || q.contains("back") || q.contains("cofaj") || q == "s",
                Action::StrafeLeft => q.contains("lewo") || q.contains("left") || q.contains("krok") || q == "a",
                Action::StrafeRight => q.contains("prawo") || q.contains("right") || q.contains("krok") || q == "d",
                Action::Jump => q.contains("skok") || q.contains("jump") || q.contains("spacja") || q.contains("space"),
                Action::Sneak => q.contains("kuc") || q.contains("skrad") || q.contains("sneak") || q.contains("crouch") || q.contains("shift"),
                Action::Sprint => q.contains("bieg") || q.contains("sprint") || q.contains("run") || q.contains("ctrl"),
                Action::Inventory => q.contains("ekwipunek") || q.contains("plecak") || q.contains("inv") || q.contains("inventory") || q.contains("item") || q == "e",
                Action::DropItem => q.contains("wyrzuc") || q.contains("drop") || q == "q",
                Action::ToggleFly => q.contains("lot") || q.contains("lata") || q.contains("fly"),
                Action::ToggleGamemode => q.contains("tryb") || q.contains("creative") || q.contains("survival") || q.contains("gamemode"),
                Action::Pause => q.contains("pauza") || q.contains("menu") || q.contains("pause") || q.contains("esc"),
                Action::Save => q.contains("zapis") || q.contains("save") || q.contains("f5"),
                Action::AttackBreak => q.contains("atak") || q.contains("niszcz") || q.contains("kop") || q.contains("attack") || q.contains("break") || q.contains("hit") || q == "lpm",
                Action::UsePlace => q.contains("uzyj") || q.contains("staw") || q.contains("buduj") || q.contains("place") || q.contains("use") || q == "ppm",
                Action::HotbarPrev | Action::HotbarNext => q.contains("pasek") || q.contains("hotbar") || q.contains("slot") || q.contains("kolko") || q.contains("scroll"),
                Action::OpenChat => q.contains("czat") || q.contains("chat") || q.contains("pisz") || q == "t",
                Action::OpenCommand => q.contains("komend") || q.contains("command") || q.contains("slash") || q == "/",
                Action::PickBlock => q.contains("wybierz") || q.contains("pick") || q.contains("kolko") || q.contains("srodkowy") || q == "mmb",
                Action::Questbook => q.contains("quest") || q.contains("zad") || q.contains("ksieg") || q.contains("journal") || q == "j",
            }
        })
        .collect()
}

/// Renders the complete Key & Controller Rebinding interface.
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
    hovered_btn: Option<usize>, // 0: reset all, 1: back, 2: search clear
    lang: Language,
    cursor_timer: f32,
    is_gamepad_active: bool,
    search_focused: bool,
) {
    let layout = ControlsRebindLayout::new(width, height);
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    // Fullscreen sleek dark backdrop
    ui.backdrop(None, pack_color(0.02, 0.03, 0.05), 0.88);

    // Header Title
    let title = match lang {
        Language::Polish => "PRZYPISANIE KLAWISZY I KONTROLERA (PC)",
        Language::English => "KEY & CONTROLLER BINDINGS (PC)",
    };
    ui.text_centered_x(
        title,
        cx,
        layout.search_box.1 - 42,
        2,
        2,
        pack_color(0.95, 0.85, 0.35),
    );

    let subtitle = match lang {
        Language::Polish => "Wyszukaj akcje lub kliknij pole, aby przypisać klawisz / mysz / przycisk pada",
        Language::English => "Search actions or click a slot to rebind keyboard, mouse, or gamepad button",
    };
    ui.text_centered_x(
        subtitle,
        cx,
        layout.search_box.1 - 20,
        1,
        1,
        pack_color(0.68, 0.74, 0.84),
    );

    // Search bar box
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

    // Search text / prompt
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

        // Clear button [x]
        let clear_bg = if hovered_btn == Some(2) {
            pack_color(0.4, 0.2, 0.2)
        } else {
            pack_color(0.25, 0.15, 0.15)
        };
        ui.fill(layout.search_clear_btn, clear_bg);
        ui.text_in_rect(layout.search_clear_btn, "X", 1, 1, pack_color(1.0, 0.7, 0.7));
    }

    // Column Headers
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
    let col_reset_title = "RESET";
    let hdr_col = pack_color(0.75, 0.80, 0.90);

    ui.text(col_action_title, cx - 380 + 12, header_y, 1, 1, hdr_col);
    ui.text(col_km_title, cx - 60 + 10, header_y, 1, 1, hdr_col);
    ui.text(col_gp_title, cx + 140 + 10, header_y, 1, 1, hdr_col);
    ui.text(col_reset_title, cx + 310 + 10, header_y, 1, 1, hdr_col);

    // Filtered actions list
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

        // Action Name & Category
        let label = format!("{} [{}]", action.display_name(lang), action.category(lang));
        ui.text(
            &label,
            row_rect.x + 12,
            row_rect.y + 8,
            1,
            1,
            pack_color(0.92, 0.94, 0.98),
        );

        // Column 1: Keyboard / Mouse binding button
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

        // Column 2: Gamepad binding button
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

        // Column 3: Row Reset button
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

    // Scrollbar (if content overflows)
    ui.scrollbar(
        layout.scrollbar,
        VISIBLE_REBIND_ROWS,
        total_filtered,
        scroll_idx,
        pack_color(0.12, 0.14, 0.18),
        pack_color(0.40, 0.50, 0.65),
    );

    // Bottom Action Buttons
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

    // Footer Navigation Hints
    let hint = if is_gamepad_active {
        "[D-PAD / L-STICK] Nawigacja | [A] Przypisz | [B / ESC] Wstecz"
    } else {
        "[ESC] Anuluj / Wróć  |  [LPM] Przypisz  |  [Kółko myszy] Przewijaj"
    };
    ui.footer(hint, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::controls::{Action, ControlsConfig};

    #[test]
    fn test_filter_actions_search() {
        let config = ControlsConfig::default();
        // Empty query returns all actions
        let all = filter_actions("", &config, Language::Polish);
        assert_eq!(all.len(), Action::ALL.len());

        // Search for jump in English
        let jump_en = filter_actions("jump", &config, Language::English);
        assert!(jump_en.contains(&Action::Jump));

        // Search for skok in Polish
        let jump_pl = filter_actions("skok", &config, Language::Polish);
        assert!(jump_pl.contains(&Action::Jump));

        // Polish diacritics normalized: "przód" vs "przod"
        let fwd1 = filter_actions("przód", &config, Language::Polish);
        let fwd2 = filter_actions("przod", &config, Language::Polish);
        assert!(fwd1.contains(&Action::MoveForward));
        assert!(fwd2.contains(&Action::MoveForward));

        // Mouse alias
        let mouse_actions = filter_actions("mysz", &config, Language::Polish);
        assert!(!mouse_actions.is_empty());
        assert!(mouse_actions.contains(&Action::AttackBreak) || mouse_actions.contains(&Action::UsePlace));
    }
}
