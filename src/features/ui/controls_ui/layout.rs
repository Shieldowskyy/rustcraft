//! Geometria układu menu przypisywania klawiszy/pada oraz filtrowanie akcji z normalizacją polskich znaków.

use crate::core::controls::{Action, ControlsConfig};
use crate::core::i18n::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebindSlot {
    KeyOrMouse,
    Gamepad,
}

pub const VISIBLE_REBIND_ROWS: usize = 9;

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

    pub fn row_buttons(
        &self,
        cx: i32,
        visible_idx: usize,
    ) -> (
        (i32, i32, i32, i32),
        (i32, i32, i32, i32),
        (i32, i32, i32, i32),
    ) {
        let y = self.list_y + visible_idx as i32 * (self.row_h + self.row_spacing) + 2;
        let h = self.row_h - 4;
        let key_btn = (cx - 60, y, 190, h);
        let gp_btn = (cx + 140, y, 160, h);
        let reset_btn = (cx + 310, y, 58, h);
        (key_btn, gp_btn, reset_btn)
    }
}

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

            if name_pl.contains(&q)
                || name_en.contains(&q)
                || cat_pl.contains(&q)
                || cat_en.contains(&q)
                || key_str.contains(&q)
            {
                return true;
            }

            let binding = &config.bindings[action.index()];

            if (q.contains("mysz") || q.contains("mouse")) && !binding.mouse.is_empty() {
                return true;
            }
            if (q.contains("pad") || q.contains("gamepad") || q.contains("kontroler"))
                && !binding.gamepad.is_empty()
            {
                return true;
            }
            if (q.contains("klawisz") || q.contains("klawiatura") || q.contains("keyboard"))
                && !binding.keys.is_empty()
            {
                return true;
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_actions_search() {
        let config = ControlsConfig::default();
        let all = filter_actions("", &config, Language::Polish);
        assert_eq!(all.len(), Action::ALL.len());

        let jump_en = filter_actions("jump", &config, Language::English);
        assert!(jump_en.contains(&Action::Jump));

        let jump_pl = filter_actions("skok", &config, Language::Polish);
        assert!(jump_pl.contains(&Action::Jump));

        let fwd1 = filter_actions("przód", &config, Language::Polish);
        let fwd2 = filter_actions("przod", &config, Language::Polish);
        assert!(fwd1.contains(&Action::MoveForward));
        assert!(fwd2.contains(&Action::MoveForward));

        let mouse_actions = filter_actions("mysz", &config, Language::Polish);
        assert!(!mouse_actions.is_empty());
        assert!(mouse_actions.contains(&Action::AttackBreak) || mouse_actions.contains(&Action::UsePlace));
    }
}
