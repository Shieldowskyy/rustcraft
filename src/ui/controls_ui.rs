//! Full PC Key and Controller Rebinding Menu UI.
//! Allows searching, rebinding keyboard keys (including F13-F24),
//! mouse buttons (standard and non-standard), and gamepad buttons.

use crate::core::controls::{Action, ControlsConfig};
use crate::core::i18n::{t, Language};
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};

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
    let cx = width as i32 / 2;

    // Fullscreen sleek dark backdrop
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.02, 0.03, 0.05), 0.88);

    // Header Title
    let title = match lang {
        Language::Polish => "PRZYPISANIE KLAWISZY I KONTROLERA (PC)",
        Language::English => "KEY & CONTROLLER BINDINGS (PC)",
    };
    let tw = font::measure_text(title, 2, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - tw / 2, layout.search_box.1 - 42, 2, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = match lang {
        Language::Polish => "Wyszukaj akcje lub kliknij pole, aby przypisać klawisz / mysz / przycisk pada",
        Language::English => "Search actions or click a slot to rebind keyboard, mouse, or gamepad button",
    };
    let sub_w = font::measure_text(subtitle, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - sub_w / 2, layout.search_box.1 - 20, 1, 1, pack_color(0.68, 0.74, 0.84));

    // Search bar box
    let (sx, sy, sw, sh) = layout.search_box;
    fill_rect(buffer, width, height, sx, sy, sw, sh, pack_color(0.12, 0.14, 0.18));
    let border_color = if search_focused { pack_color(1.0, 0.85, 0.3) } else { pack_color(0.35, 0.40, 0.50) };
    fill_rect(buffer, width, height, sx, sy, sw, 1, border_color);
    fill_rect(buffer, width, height, sx, sy + sh - 1, sw, 1, if search_focused { border_color } else { pack_color(0.20, 0.22, 0.28) });
    fill_rect(buffer, width, height, sx, sy, 1, sh, border_color);
    fill_rect(buffer, width, height, sx + sw - 1, sy, 1, sh, if search_focused { border_color } else { pack_color(0.20, 0.22, 0.28) });

    // Search text / prompt
    let search_scale = 1;
    let text_y = sy + (sh - font::GLYPH_H as i32 * search_scale) / 2;
    if query.is_empty() {
        let placeholder = match lang {
            Language::Polish => "Szukaj akcji... (np. ruch, skok, atak, F5, mysz)",
            Language::English => "Search actions... (e.g. forward, jump, attack, F5, mouse)",
        };
        if search_focused && (cursor_timer * 2.0) as i32 % 2 == 0 {
            draw_text_colored_shadowed(buffer, width, height, "|", sx + 10, text_y, search_scale, 1, pack_color(1.0, 1.0, 0.3));
            draw_text_colored_shadowed(buffer, width, height, placeholder, sx + 18, text_y, search_scale, 1, pack_color(0.45, 0.50, 0.58));
        } else {
            draw_text_colored_shadowed(buffer, width, height, placeholder, sx + 10, text_y, search_scale, 1, pack_color(0.45, 0.50, 0.58));
        }
    } else {
        let cursor_char = if search_focused && (cursor_timer * 2.0) as i32 % 2 == 0 { "|" } else { "" };
        let full_text = format!("{}{}", query, cursor_char);
        draw_text_colored_shadowed(buffer, width, height, &full_text, sx + 10, text_y, search_scale, 1, pack_color(1.0, 1.0, 0.3));

        // Clear button [x]
        let (cx_b, cy_b, cw_b, ch_b) = layout.search_clear_btn;
        let is_clear_hover = hovered_btn == Some(2);
        let clear_bg = if is_clear_hover { pack_color(0.4, 0.2, 0.2) } else { pack_color(0.25, 0.15, 0.15) };
        fill_rect(buffer, width, height, cx_b, cy_b, cw_b, ch_b, clear_bg);
        draw_text_colored_shadowed(buffer, width, height, "X", cx_b + 6, cy_b + 4, 1, 1, pack_color(1.0, 0.7, 0.7));
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
    let col_reset_title = match lang {
        Language::Polish => "RESET",
        Language::English => "RESET",
    };

    draw_text_colored_shadowed(buffer, width, height, col_action_title, cx - 380 + 12, header_y, 1, 1, pack_color(0.75, 0.80, 0.90));
    draw_text_colored_shadowed(buffer, width, height, col_km_title, cx - 60 + 10, header_y, 1, 1, pack_color(0.75, 0.80, 0.90));
    draw_text_colored_shadowed(buffer, width, height, col_gp_title, cx + 140 + 10, header_y, 1, 1, pack_color(0.75, 0.80, 0.90));
    draw_text_colored_shadowed(buffer, width, height, col_reset_title, cx + 310 + 10, header_y, 1, 1, pack_color(0.75, 0.80, 0.90));

    // Filtered actions list
    let filtered_actions = filter_actions(query, config, lang);
    let total_filtered = filtered_actions.len();

    let max_scroll = if total_filtered > VISIBLE_REBIND_ROWS {
        total_filtered - VISIBLE_REBIND_ROWS
    } else {
        0
    };
    let scroll_idx = scroll_offset.min(max_scroll);

    if total_filtered == 0 {
        let no_res = match lang {
            Language::Polish => "Nie znaleziono pasujących akcji",
            Language::English => "No matching actions found",
        };
        let nr_w = font::measure_text(no_res, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, no_res, cx - nr_w / 2, layout.list_y + 60, 1, 1, pack_color(0.6, 0.65, 0.75));
    }

    for visible_idx in 0..VISIBLE_REBIND_ROWS {
        let action_idx = scroll_idx + visible_idx;
        if action_idx >= total_filtered {
            break;
        }

        let action = filtered_actions[action_idx];
        let (rx, ry, rw, rh) = layout.row_rect(cx, visible_idx);
        let (k_btn, gp_btn, rst_btn) = layout.row_buttons(cx, visible_idx);

        let is_row_hover = hovered_row == Some(visible_idx);
        let row_bg = if is_row_hover {
            pack_color(0.16, 0.20, 0.28)
        } else if visible_idx % 2 == 0 {
            pack_color(0.09, 0.11, 0.15)
        } else {
            pack_color(0.12, 0.14, 0.19)
        };

        fill_rect(buffer, width, height, rx, ry, rw, rh, row_bg);

        // Action Name & Category
        let action_name = action.display_name(lang);
        let cat_name = action.category(lang);
        let label = format!("{} [{}]", action_name, cat_name);
        draw_text_colored_shadowed(buffer, width, height, &label, rx + 12, ry + 8, 1, 1, pack_color(0.92, 0.94, 0.98));

        // Column 1: Keyboard / Mouse binding button
        let is_km_rebind = rebind_target == Some((action, RebindSlot::KeyOrMouse));
        let is_km_hover = is_row_hover && hovered_col == Some(0);

        let binding = &config.bindings[action.index()];
        let km_text: String = if is_km_rebind {
            let pulse = ((cursor_timer * 6.0).sin() * 0.5 + 0.5) > 0.5;
            if pulse { "> WCIŚNIJ KLAWISZ <".to_string() } else { "> WYBIERZ PRZYCISK <".to_string() }
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
        } else if is_km_hover {
            pack_color(0.28, 0.36, 0.52)
        } else {
            pack_color(0.18, 0.22, 0.30)
        };
        let km_border = if is_km_rebind {
            pack_color(1.0, 0.9, 0.2)
        } else if is_km_hover {
            pack_color(0.65, 0.75, 0.95)
        } else {
            pack_color(0.32, 0.36, 0.45)
        };

        fill_rect(buffer, width, height, k_btn.0, k_btn.1, k_btn.2, k_btn.3, km_bg);
        fill_rect(buffer, width, height, k_btn.0, k_btn.1, k_btn.2, 1, km_border);
        fill_rect(buffer, width, height, k_btn.0, k_btn.1 + k_btn.3 - 1, k_btn.2, 1, km_border);
        fill_rect(buffer, width, height, k_btn.0, k_btn.1, 1, k_btn.3, km_border);
        fill_rect(buffer, width, height, k_btn.0 + k_btn.2 - 1, k_btn.1, 1, k_btn.3, km_border);

        let km_w = font::measure_text(&km_text, 1, 1);
        let km_tx = k_btn.0 + (k_btn.2 - km_w) / 2;
        let km_col = if is_km_rebind { pack_color(1.0, 1.0, 0.3) } else if is_km_hover { pack_color(1.0, 1.0, 0.6) } else { pack_color(0.9, 0.92, 0.96) };
        draw_text_colored_shadowed(buffer, width, height, &km_text, km_tx, k_btn.1 + 6, 1, 1, km_col);

        // Column 2: Gamepad binding button
        let is_gp_rebind = rebind_target == Some((action, RebindSlot::Gamepad));
        let is_gp_hover = is_row_hover && hovered_col == Some(1);

        let gp_text: String = if is_gp_rebind {
            let pulse = ((cursor_timer * 6.0).sin() * 0.5 + 0.5) > 0.5;
            if pulse { "> PRZYCISK PADA <".to_string() } else { "> NACIŚNIJ <".to_string() }
        } else if !binding.gamepad.is_empty() {
            let gp_names: Vec<&'static str> = binding.gamepad.iter().map(|gp| gp.display_name()).collect();
            gp_names.join(", ")
        } else {
            "---".to_string()
        };

        let gp_bg = if is_gp_rebind {
            pack_color(0.12, 0.35, 0.40)
        } else if is_gp_hover {
            pack_color(0.24, 0.34, 0.48)
        } else {
            pack_color(0.16, 0.20, 0.26)
        };
        let gp_border = if is_gp_rebind {
            pack_color(0.3, 0.9, 1.0)
        } else if is_gp_hover {
            pack_color(0.60, 0.75, 0.90)
        } else {
            pack_color(0.30, 0.34, 0.42)
        };

        fill_rect(buffer, width, height, gp_btn.0, gp_btn.1, gp_btn.2, gp_btn.3, gp_bg);
        fill_rect(buffer, width, height, gp_btn.0, gp_btn.1, gp_btn.2, 1, gp_border);
        fill_rect(buffer, width, height, gp_btn.0, gp_btn.1 + gp_btn.3 - 1, gp_btn.2, 1, gp_border);
        fill_rect(buffer, width, height, gp_btn.0, gp_btn.1, 1, gp_btn.3, gp_border);
        fill_rect(buffer, width, height, gp_btn.0 + gp_btn.2 - 1, gp_btn.1, 1, gp_btn.3, gp_border);

        let gp_w = font::measure_text(&gp_text, 1, 1);
        let gp_tx = gp_btn.0 + (gp_btn.2 - gp_w) / 2;
        let gp_col = if is_gp_rebind { pack_color(0.4, 0.95, 1.0) } else if is_gp_hover { pack_color(0.8, 0.95, 1.0) } else { pack_color(0.88, 0.90, 0.94) };
        draw_text_colored_shadowed(buffer, width, height, &gp_text, gp_tx, gp_btn.1 + 6, 1, 1, gp_col);

        // Column 3: Row Reset button
        let is_rst_hover = is_row_hover && hovered_col == Some(2);
        let rst_bg = if is_rst_hover { pack_color(0.38, 0.22, 0.22) } else { pack_color(0.22, 0.16, 0.18) };
        let rst_border = if is_rst_hover { pack_color(0.9, 0.5, 0.5) } else { pack_color(0.45, 0.30, 0.32) };

        fill_rect(buffer, width, height, rst_btn.0, rst_btn.1, rst_btn.2, rst_btn.3, rst_bg);
        fill_rect(buffer, width, height, rst_btn.0, rst_btn.1, rst_btn.2, 1, rst_border);
        fill_rect(buffer, width, height, rst_btn.0, rst_btn.1 + rst_btn.3 - 1, rst_btn.2, 1, rst_border);
        fill_rect(buffer, width, height, rst_btn.0, rst_btn.1, 1, rst_btn.3, rst_border);
        fill_rect(buffer, width, height, rst_btn.0 + rst_btn.2 - 1, rst_btn.1, 1, rst_btn.3, rst_border);

        let rst_label = "RESET";
        let rst_w = font::measure_text(rst_label, 1, 1);
        let rst_col = if is_rst_hover { pack_color(1.0, 0.8, 0.8) } else { pack_color(0.8, 0.65, 0.68) };
        draw_text_colored_shadowed(buffer, width, height, rst_label, rst_btn.0 + (rst_btn.2 - rst_w) / 2, rst_btn.1 + 6, 1, 1, rst_col);
    }

    // Scrollbar (if content overflows)
    if total_filtered > VISIBLE_REBIND_ROWS {
        let (sbx, sby, sbw, sbh) = layout.scrollbar;
        fill_rect(buffer, width, height, sbx, sby, sbw, sbh, pack_color(0.12, 0.14, 0.18));

        let thumb_h = ((VISIBLE_REBIND_ROWS as f32 / total_filtered as f32) * sbh as f32).max(18.0) as i32;
        let thumb_y = sby + ((scroll_idx as f32 / max_scroll as f32) * (sbh - thumb_h) as f32) as i32;
        fill_rect(buffer, width, height, sbx, thumb_y, sbw, thumb_h, pack_color(0.40, 0.50, 0.65));
        fill_rect(buffer, width, height, sbx, thumb_y, sbw, 1, pack_color(0.70, 0.80, 0.95));
    }

    // Bottom Action Buttons
    // 0: Reset All Defaults button
    let (rx, ry, rw, rh) = layout.reset_all_btn;
    let is_reset_all_hover = hovered_btn == Some(0);
    let reset_all_bg = if is_reset_all_hover { pack_color(0.45, 0.25, 0.25) } else { pack_color(0.28, 0.18, 0.20) };
    let reset_all_border = if is_reset_all_hover { pack_color(1.0, 0.6, 0.6) } else { pack_color(0.55, 0.35, 0.38) };

    fill_rect(buffer, width, height, rx, ry, rw, rh, reset_all_bg);
    fill_rect(buffer, width, height, rx, ry, rw, 2, reset_all_border);
    fill_rect(buffer, width, height, rx, ry + rh - 2, rw, 2, pack_color(0.15, 0.08, 0.10));
    fill_rect(buffer, width, height, rx, ry, 2, rh, reset_all_border);
    fill_rect(buffer, width, height, rx + rw - 2, ry, 2, rh, pack_color(0.15, 0.08, 0.10));

    let reset_all_label = match lang {
        Language::Polish => "PRZYWRÓĆ DOMYŚLNE",
        Language::English => "RESET TO DEFAULTS",
    };
    let raw_w = font::measure_text(reset_all_label, 1, 1);
    let raw_col = if is_reset_all_hover { pack_color(1.0, 0.9, 0.9) } else { pack_color(0.9, 0.75, 0.78) };
    draw_text_colored_shadowed(buffer, width, height, reset_all_label, rx + (rw - raw_w) / 2, ry + 8, 1, 1, raw_col);

    // 1: Save & Back button
    let (bx, by, bw, bh) = layout.back_btn;
    let is_back_hover = hovered_btn == Some(1);
    let back_bg = if is_back_hover { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.18, 0.30, 0.22) };
    let back_border = if is_back_hover { pack_color(0.75, 0.95, 0.8) } else { pack_color(0.45, 0.65, 0.5) };

    fill_rect(buffer, width, height, bx, by, bw, bh, back_bg);
    fill_rect(buffer, width, height, bx, by, bw, 2, back_border);
    fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, pack_color(0.10, 0.18, 0.12));
    fill_rect(buffer, width, height, bx, by, 2, bh, back_border);
    fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, pack_color(0.10, 0.18, 0.12));

    let back_label = t("settings.save_and_back");
    let back_w = font::measure_text(back_label, 1, 1);
    let back_col = if is_back_hover { pack_color(1.0, 1.0, 0.6) } else { pack_color(0.90, 0.95, 0.92) };
    draw_text_colored_shadowed(buffer, width, height, back_label, bx + (bw - back_w) / 2, by + 8, 1, 1, back_col);

    // Footer Navigation Hints
    let hint = if is_gamepad_active {
        "[D-PAD / L-STICK] Nawigacja | [A] Przypisz | [B / ESC] Wstecz"
    } else {
        "[ESC] Anuluj / Wróć  |  [LPM] Przypisz  |  [Kółko myszy] Przewijaj"
    };
    let hw = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, hint, cx - hw / 2, height as i32 - 18, 1, 1, pack_color(0.60, 0.65, 0.75));
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
